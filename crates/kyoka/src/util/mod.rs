mod sensitive;

pub(crate) mod twilight;

pub mod env;
pub use sensitive::*;

use error_stack::{Result, ResultExt};
use std::path::Path;
use std::time::Instant;
use thiserror::Error;
use tracing_subscriber::filter::Targets;
use tracing_subscriber::{prelude::*, Layer, Registry};
use twilight_http::client::InteractionClient;

use crate::config;

#[derive(Debug, Error)]
#[error("Failed to setup commands")]
pub struct SetupCommandsError;

#[derive(Debug, Error)]
#[error("Failed to initialize logging")]
pub struct InitLoggingError;

pub fn make_http_client(cfg: &config::Bot) -> twilight_http::Client {
    let mut http =
        twilight_http::Client::builder().token(cfg.token().to_string());

    if let Some(proxy_url) = cfg.proxy_url() {
        http = http.proxy(proxy_url.into(), cfg.proxy_use_http());
    }

    http.build()
}

pub async fn setup_cmds(
    interaction_client: InteractionClient<'_>,
) -> Result<(), SetupCommandsError> {
    use crate::{cmd, perform_request};
    use twilight_interactions::command::CreateCommand;

    let required_cmds = &[cmd::Ping::create_command().into()];

    let now = Instant::now();
    perform_request!(
        interaction_client.set_global_commands(required_cmds),
        SetupCommandsError
    )
    .await?;

    let elapsed = now.elapsed();
    tracing::info!(
        ?elapsed,
        "Sent {} global command/s to Discord",
        required_cmds.len()
    );

    Ok(())
}

pub fn init_logging() -> Result<(), InitLoggingError> {
    let targets = dotenvy::var("RUST_LOG")
        .unwrap_or_else(|_| "info".into())
        .trim()
        .trim_matches('"')
        .parse::<Targets>()
        .change_context(InitLoggingError)?;

    let format_layer = tracing_subscriber::fmt::layer();
    let sentry_layer = sentry::integrations::tracing::layer();

    // Docker logged every emitted line of stdout along with the
    // timestamp of when it was emitted.
    //
    // tracing-subscriber's Format config layer has very explicit types
    if is_running_in_docker() {
        let format_layer =
            format_layer.without_time().with_filter(targets.clone());

        let subscriber = Registry::default().with(format_layer);
        tracing::subscriber::set_global_default(subscriber)
            .change_context(InitLoggingError)?;
    } else {
        let format_layer = format_layer.with_filter(targets.clone());
        let subscriber =
            Registry::default().with(format_layer).with(sentry_layer);

        tracing::subscriber::set_global_default(subscriber)
            .change_context(InitLoggingError)?;
    }

    Ok(())
}

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
/// **For Windows / unsupported platforms**: It detects if `CTRL+C` is triggered
///
/// **For Unix systems**: It detects whether `SIGINT` or `SIGTERM` is triggered
#[cfg(not(unix))]
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
