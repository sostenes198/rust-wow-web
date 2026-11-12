use std::time::Duration;

use domain::{AppConfig, Logging};
use tokio::{net::TcpStream, time::sleep};
use tracing::{error, info};

#[tokio::main]
async fn main() {
    let config = match AppConfig::load() {
        Ok(config) => config,
        Err(e) => {
            eprintln!("invalid config: {e}");
            std::process::exit(1);
        }
    };

    Logging::initialize(&config.log_level).map_err(|e| {
        eprint!("Failed to initialize log: {e}");
        std::process::exit(1);
    });

    let addr = config.auth_addr();
    info!("connected in {addr}");

    match TcpStream::connect(&addr).await {
        Ok(stream) => {
            info!("connected");
            sleep(Duration::from_secs(1)).await;
            drop(stream);
            info!("disconected");
        }
        Err(e) => {
            error!("Could not connect in {addr}: {e}");
            std::process::exit(1);
        }
    }
}
