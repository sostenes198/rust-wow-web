//! Component tests: the real server on real TCP sockets (loopback only).

use std::{net::SocketAddr, time::Duration};

use tokio::{
    io::AsyncReadExt,
    net::{TcpListener, TcpStream},
    time::timeout,
};

/// Starts the server on a free port (port 0 lets the OS pick) and returns its address.
async fn start_server() -> SocketAddr {
    let listener = TcpListener::bind("127.0.0.1:0").await.unwrap();
    let addr = listener.local_addr().unwrap();

    tokio::spawn(auth_server::run(listener));

    addr
}

/// Reads until the server closes the connection and returns the bytes read.
/// Fails (instead of hanging) if the connection stays open for too long.
async fn read_until_closed(client: &mut TcpStream) -> usize {
    let mut buffer = [0u8; 16];

    timeout(Duration::from_secs(2), client.read(&mut buffer))
        .await
        .expect("server did not close the connection in time")
        .unwrap()
}

#[tokio::test]
async fn server_accepts_and_closes_the_connection() {
    let addr = start_server().await;

    let mut client = TcpStream::connect(addr).await.unwrap();

    // Reading 0 bytes means the other side closed the connection.
    assert_eq!(read_until_closed(&mut client).await, 0);
}

#[tokio::test]
async fn server_keeps_accepting_after_a_client_leaves() {
    let addr = start_server().await;

    for _ in 0..3 {
        let mut client = TcpStream::connect(addr).await.unwrap();
        assert_eq!(read_until_closed(&mut client).await, 0);
    }
}

#[tokio::test]
async fn server_serves_several_clients_at_the_same_time() {
    let addr = start_server().await;

    // Connect all clients first, then check them: none may block the others.
    let mut clients = Vec::new();
    for _ in 0..5 {
        clients.push(TcpStream::connect(addr).await.unwrap());
    }

    for client in &mut clients {
        assert_eq!(read_until_closed(client).await, 0);
    }
}
