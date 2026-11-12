use domain::{AppConfig, Logging};
use std::net::SocketAddr;
use tokio::net::{TcpListener, TcpStream};
use tracing::{error, info};

#[tokio::main]
async fn main() {
    let config = match AppConfig::load() {
        Ok(config) => config,
        Err(e) => {
            eprintln!("Invalid config: {e}");
            std::process::exit(1);
        }
    };

    Logging::initialize(&config.log_level).map_err(|e| {
        eprint!("Failed to initialize log: {e}");
        std::process::exit(1);
    });

    let addr = config.auth_addr();

    let listener = match TcpListener::bind(&addr).await {
        Ok(listener) => listener,
        Err(e) => {
            error!("Could not open {addr}: {e}");
            std::process::exit(1);
        }
    };

    info!("auth-server listening in {addr}");

    loop {
        match listener.accept().await {
            Ok((socket, peer)) => {
                tokio::spawn(async move {
                    handle_connection(socket, peer);
                });
            }
            Err(e) => error!("Failed to accept connection: {e}"),
        }
    }
}

fn handle_connection(socket: TcpStream, peer: SocketAddr) {
    info!(%peer, "client connected");
    drop(socket);
    info!(%peer, "client desconected");
}
