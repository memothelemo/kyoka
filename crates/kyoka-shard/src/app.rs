use error_stack::{Result, ResultExt};
use tokio_util::sync::{CancellationToken, WaitForCancellationFuture};

use crate::{metrics::Metrics, SetupError};

/// Holds all state for metrics server and Discord bot client
#[derive(Debug, Clone)]
pub struct App {
    metrics: Metrics,
    shutdown_signal: CancellationToken,
}

impl App {
    #[must_use]
    pub fn new() -> Result<Self, SetupError> {
        Ok(Self {
            metrics: Metrics::register(prometheus::default_registry())
                .change_context(SetupError)?,
            shutdown_signal: CancellationToken::new(),
        })
    }
}

impl App {
    pub fn metrics(&self) -> &Metrics {
        &self.metrics
    }

    pub fn has_shutdown(&self) -> bool {
        self.shutdown_signal.is_cancelled()
    }

    pub fn perform_shutdown(&self, reason: &'static str) {
        if self.has_shutdown() {
            return;
        }
        tracing::info!("{reason}; starting graceful shutdown...");
        self.shutdown_signal.cancel();
    }

    pub fn shutdown_signal(&self) -> WaitForCancellationFuture<'_> {
        self.shutdown_signal.cancelled()
    }
}
