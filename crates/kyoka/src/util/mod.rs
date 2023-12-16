mod sensitive;
pub use sensitive::*;

pub(crate) mod twilight;

use std::path::Path;

/// Checks whether this environment/process running in a Docker container.
pub fn is_running_in_docker() -> bool {
    // https://stackoverflow.com/a/23558932/23025722
    let proc_1_group = std::fs::read_to_string("/proc/1/group")
        .map(|content| content.contains("/docker/"))
        .unwrap_or_default();

    let proc_mount = std::fs::read_to_string("/proc/self/mountinfo")
        .map(|content| content.contains("/docker/"))
        .unwrap_or_default();

    Path::new("/.dockerenv").exists()
        || Path::new("/run/.containerenv").exists()
        || proc_1_group
        || proc_mount
}

/// Cross-platform compatible function that yields the
/// current thread until one of the exit signals is triggered
/// by the operating system.
///
/// It allows programs to implement graceful shutdown to
/// prevent from any data loss or unexpected behavior to
/// the Discord bot (for example).
///
/// **For Windows**: It detects if `CTRL+C` is triggered
///
/// **For Linux**: It detects whether `SIGINT` or `SIGTERM` is triggered
#[cfg(windows)]
pub async fn shutdown_signal() {
    tokio::signal::ctrl_c()
        .await
        .expect("failed to install CTRL+C signal handler");
}

/// Cross-platform compatible function that yields the
/// current thread until one of the exit signals is triggered
/// by the operating system.
///
/// It allows programs to implement graceful shutdown to
/// prevent from any data loss or unexpected behavior to
/// the Discord bot (for example).
///
/// **For Windows**: It detects if `CTRL+C` is triggered
///
/// **For Linux**: It detects whether `SIGINT` or `SIGTERM` is triggered
#[cfg(unix)]
pub async fn shutdown_signal() {
    use tokio::signal::unix::{signal, SignalKind};
    let mut sigint = signal(SignalKind::interrupt())
        .expect("failed to install SIGINT handler");

    let mut sigterm = signal(SignalKind::terminate())
        .expect("failed to install SIGTERM handler");

    tokio::select! {
        _ = sigint.recv() => {},
        _ = sigterm.recv() => {},
    };
}
