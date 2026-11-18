use infrastructure::{AppConfig, Logging};
use tokio::net::TcpListener;
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

    if let Err(e) = Logging::initialize(&config.log_level) {
        eprintln!("Failed to initialize log: {e}");
        std::process::exit(1);
    }

    let addr = config.auth_addr();

    let listener = match TcpListener::bind(&addr).await {
        Ok(listener) => listener,
        Err(e) => {
            error!("Could not open {addr}: {e}");
            std::process::exit(1);
        }
    };

    info!("auth-server listening on {addr}");

    auth_server::run(listener).await;
}