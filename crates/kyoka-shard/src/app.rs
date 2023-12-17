use tokio_util::sync::{CancellationToken, WaitForCancellationFuture};

/// Holds all state for metrics server and Discord bot client
#[derive(Debug, Clone)]
pub struct App {
    shutdown_signal: CancellationToken,
}

impl App {
    #[must_use]
    pub fn new() -> Self {
        Self { shutdown_signal: CancellationToken::new() }
    }
}

impl App {
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
