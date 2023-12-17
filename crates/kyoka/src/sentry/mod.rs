use sentry::ClientInitGuard;
use std::borrow::Cow;

use crate::config;

/// Already cached
fn release_name() -> Option<Cow<'static, str>> {
    sentry::release_name!()
}

pub fn init() -> Option<ClientInitGuard> {
    let config = match config::Sentry::from_env() {
        Ok(inner) => {
            if inner.dsn().is_none() {
                tracing::info!("Sentry integration is disabled");
                return None;
            }
            inner
        },
        Err(error) => {
            tracing::warn!(
                ?error,
                "Failed to read Sentry config from environment"
            );
            return None;
        },
    };

    let opts = sentry::ClientOptions {
        dsn: config.dsn(),
        environment: config.environment().map(|v| Cow::Owned(v.to_string())),
        release: release_name(),
        session_mode: sentry::SessionMode::Request,
        traces_sample_rate: config.traces_sample_rate(),
        ..Default::default()
    };

    tracing::info!(
        cfg.env = ?config.environment(),
        release = ?release_name(),
        "Sentry integration is enabled"
    );
    Some(sentry::init(opts))
}
