use std::{future::Future, io, net::SocketAddr};

use tokio::{
    io::{AsyncRead, AsyncWrite},
    net::{TcpListener, TcpStream},
};
use tracing::{error, info};

/// Something that hands out incoming client streams.
///
/// `TcpListener` is the real implementation. Unit tests use an in-memory fake,
/// so they never open a socket.
pub trait Acceptor {
    type Stream: AsyncRead + AsyncWrite + Unpin + Send + 'static;

    fn accept(&mut self) -> impl Future<Output = io::Result<(Self::Stream, SocketAddr)>> + Send;
}

impl Acceptor for TcpListener {
    type Stream = TcpStream;

    async fn accept(&mut self) -> io::Result<(TcpStream, SocketAddr)> {
        TcpListener::accept(self).await
    }
}

/// Accepts connections forever and serves each client in its own task.
pub async fn run<A: Acceptor>(mut acceptor: A) {
    loop {
        match acceptor.accept().await {
            Ok((stream, peer)) => {
                tokio::spawn(async move {
                    handle_connection(stream, peer).await;
                });
            }
            Err(e) => error!("Failed to accept connection: {e}"),
        }
    }
}

async fn handle_connection<S>(stream: S, peer: SocketAddr)
where
    S: AsyncRead + AsyncWrite + Unpin,
{
    info!(%peer, "client connected");
    drop(stream);
    info!(%peer, "client disconnected");
}

#[cfg(test)]
mod tests {
    use std::time::Duration;

    use tokio::{
        io::{AsyncReadExt, DuplexStream, duplex},
        sync::mpsc::{UnboundedReceiver, UnboundedSender, unbounded_channel},
        time::timeout,
    };

    use super::*;

    fn peer() -> SocketAddr {
        "127.0.0.1:4000".parse().unwrap()
    }

    /// In-memory pipe: (client end, server end). No socket involved.
    fn pipe() -> (DuplexStream, DuplexStream) {
        duplex(64)
    }

    /// Fake acceptor fed by the test through a channel, one client at a time.
    ///
    /// Streams the test has not sent yet stay in the test's hands. If `run` stops
    /// early, the receiver is dropped and the next `send` fails, so a `run` that
    /// quits is never mistaken for a handler that closed a stream.
    struct FakeAcceptor {
        incoming: UnboundedReceiver<io::Result<DuplexStream>>,
    }

    type Incoming = UnboundedSender<io::Result<DuplexStream>>;

    fn fake_acceptor() -> (Incoming, FakeAcceptor) {
        let (sender, incoming) = unbounded_channel();

        (sender, FakeAcceptor { incoming })
    }

    impl Acceptor for FakeAcceptor {
        type Stream = DuplexStream;

        async fn accept(&mut self) -> io::Result<(DuplexStream, SocketAddr)> {
            match self.incoming.recv().await {
                Some(next) => next.map(|stream| (stream, peer())),
                // The test is done sending: behave like an idle listener.
                None => std::future::pending().await,
            }
        }
    }

    /// Reads until the server end closes and returns the bytes read.
    /// Fails (instead of hanging) if it stays open for too long.
    async fn read_until_closed(client: &mut DuplexStream) -> usize {
        let mut buffer = [0u8; 16];

        timeout(Duration::from_secs(2), client.read(&mut buffer))
            .await
            .expect("stream was not closed in time")
            .unwrap()
    }

    // ---------- handle_connection ----------

    #[tokio::test]
    async fn handle_connection_closes_the_stream() {
        let (mut client, server) = pipe();

        handle_connection(server, peer()).await;

        assert_eq!(read_until_closed(&mut client).await, 0);
    }

    // ---------- run ----------

    #[tokio::test]
    async fn run_closes_every_accepted_stream() {
        let (incoming, acceptor) = fake_acceptor();
        tokio::spawn(run(acceptor));

        for _ in 0..3 {
            let (mut client, server) = pipe();

            incoming.send(Ok(server)).expect("run stopped accepting");

            assert_eq!(read_until_closed(&mut client).await, 0);
        }
    }

    #[tokio::test]
    async fn run_keeps_accepting_after_an_accept_error() {
        let (incoming, acceptor) = fake_acceptor();
        tokio::spawn(run(acceptor));

        incoming
            .send(Err(io::Error::other("boom")))
            .expect("run stopped accepting");
        // Let `run` handle the error before the next client shows up.
        tokio::task::yield_now().await;

        let (mut client, server) = pipe();
        incoming
            .send(Ok(server))
            .expect("run stopped after an accept error");

        assert_eq!(read_until_closed(&mut client).await, 0);
    }
}
