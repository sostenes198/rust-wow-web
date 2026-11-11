use std::{
    io,
    sync::{Arc, Mutex},
};

use tracing::{debug, error, info, trace, warn};
use tracing_subscriber::EnvFilter;

const LEVELS: [&str; 5] = ["trace", "debug", "info", "warn", "error"];

#[derive(Clone, Default)]
pub struct LoggingCapture(Arc<Mutex<Vec<u8>>>);

impl LoggingCapture {
    pub fn contents(&self) -> String {
        String::from_utf8(self.0.lock().unwrap().clone()).unwrap()
    }

    pub fn emit_all_levels(filter: EnvFilter) -> String {
        let capture = Self::default();
        let subscriber = tracing_subscriber::fmt()
            .with_env_filter(filter)
            .with_writer({
                let capture = capture.clone();
                move || capture.clone()
            })
            .with_ansi(false)
            .finish();

        tracing::subscriber::with_default(subscriber, || {
            trace!("msg-trace");
            debug!("msg-debug");
            info!("msg-info");
            warn!("msg-warn");
            error!("msg-error");
        });

        capture.contents()
    }

    pub fn levels_present(output: &str) -> Vec<&'static str> {
        LEVELS
            .into_iter()
            .filter(|name| output.contains(&format!("msg-{name}")))
            .collect()
    }
}

impl io::Write for LoggingCapture {
    fn write(&mut self, buf: &[u8]) -> io::Result<usize> {
        self.0.lock().unwrap().extend_from_slice(buf);
        Ok(buf.len())
    }

    fn flush(&mut self) -> io::Result<()> {
        Ok(())
    }
}
