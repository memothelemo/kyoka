use sentry::ClientInitGuard;
use std::borrow::Cow;

use crate::config;

/// Already cached
fn release_name(root_name: &'static str) -> Option<Cow<'static, str>> {
    use std::sync::Once;
    static mut INIT: Once = Once::new();
    static mut RELEASE: Option<String> = None;

    // SAFETY: Copied from sentry::release_name!() macro
    unsafe {
        // One for all (CARGO_PKG_VERSION)
        INIT.call_once(|| {
            RELEASE = option_env!("CARGO_PKG_VERSION")
                .map(|version| format!("{}@{}", root_name, version))
        });
        RELEASE.as_ref().map(|x| {
            let release: &'static str = ::std::mem::transmute(x.as_str());
            ::std::borrow::Cow::Borrowed(release)
        })
    }
}

pub fn init(root_name: &'static str) -> Option<ClientInitGuard> {
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
        release: release_name(root_name),
        session_mode: sentry::SessionMode::Request,
        traces_sample_rate: config.traces_sample_rate(),
        ..Default::default()
    };

    tracing::info!(
        cfg.env = ?config.environment(),
        release = ?release_name(root_name),
        "Sentry integration is enabled"
    );
    Some(sentry::init(opts))
}
