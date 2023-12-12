use error_stack::{Result, ResultExt};
use std::time::Instant;
use tracing_subscriber::{filter::Targets, prelude::*, Layer, Registry};

use crate::{Config, SetupError, State};

pub fn init_logging(cfg: &Config) -> Result<(), SetupError> {
    let log_description =
        std::env::var("RUST_LOG").unwrap_or_else(|_| "info".into());

    let targets = log_description
        .trim()
        .trim_matches('"')
        .parse::<Targets>()
        .change_context(SetupError)?;

    let format_layer = tracing_subscriber::fmt::layer()
        .pretty()
        .with_ansi(cfg.log().color_enabled())
        .with_filter(cfg.log().level())
        .with_filter(targets.clone());

    let subscriber = Registry::default().with(format_layer);
    tracing::subscriber::set_global_default(subscriber)
        .change_context(SetupError)?;

    Ok(())
}

#[tracing::instrument]
pub async fn cmd(state: &State) -> Result<(), SetupError> {
    use crate::{cmd, perform_request};
    use twilight_interactions::command::CreateCommand;

    let required_cmds = &[cmd::Ping::create_command().into()];

    let now = Instant::now();
    perform_request!(
        state.interaction().set_global_commands(required_cmds),
        SetupError
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
