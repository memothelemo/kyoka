use error_stack::{Result, ResultExt};
use std::{sync::Arc, time::Instant};
use thiserror::Error;
use tracing_subscriber::{filter::Targets, prelude::*, Layer, Registry};
use twilight_http::client::InteractionClient;

use crate::{config::Config, util};

#[derive(Debug, Error)]
#[error("Failed to setup Kyoka")]
pub struct SetupError;

pub fn init_logging(cfg: &Config) -> Result<(), SetupError> {
    let log_description =
        std::env::var("RUST_LOG").unwrap_or_else(|_| "info".into());

    let targets = log_description
        .trim()
        .trim_matches('"')
        .parse::<Targets>()
        .change_context(SetupError)?;

    let format_layer =
        tracing_subscriber::fmt::layer().with_ansi(cfg.log().color_enabled());

    // Docker already logged timestamp for every line of stdout
    //
    // tracing-subscriber's Format config layer has very explicit types
    if util::is_running_in_docker() {
        let format_layer = format_layer
            .pretty()
            .without_time()
            .with_filter(cfg.log().level())
            .with_filter(targets.clone());

        let subscriber = Registry::default().with(format_layer);
        tracing::subscriber::set_global_default(subscriber)
            .change_context(SetupError)?;
    } else {
        let format_layer = format_layer
            .pretty()
            .with_filter(cfg.log().level())
            .with_filter(targets.clone());

        let subscriber = Registry::default().with(format_layer);
        tracing::subscriber::set_global_default(subscriber)
            .change_context(SetupError)?;
    }

    Ok(())
}

pub fn make_http_client(cfg: &Config) -> Arc<twilight_http::Client> {
    let mut http =
        twilight_http::Client::builder().token(cfg.bot().token().to_string());

    if let Some(proxy) = cfg.bot().proxy() {
        http = http.proxy(proxy.url().to_string(), proxy.use_http());
    }

    Arc::new(http.build())
}

#[tracing::instrument(skip(interaction_client))]
pub async fn load_commands(
    interaction_client: InteractionClient<'_>,
) -> Result<(), SetupError> {
    use crate::{cmd, perform_request};
    use twilight_interactions::command::CreateCommand;

    let required_cmds = &[cmd::Ping::create_command().into()];

    let now = Instant::now();
    perform_request!(
        interaction_client.set_global_commands(required_cmds),
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
