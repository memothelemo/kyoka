use error_stack::{FutureExt, Result};
use futures::future::TryFutureExt;
use std::{fmt::Debug, future::IntoFuture, sync::Arc};
use thiserror::Error;
use twilight_http::client::InteractionClient;
use twilight_http::Client as Http;
use twilight_model::oauth::Application;

use crate::config::Config;
use crate::util::Sensitive;

#[derive(Clone)]
pub struct State {
    // Application info of a bot based on a token
    // provided from config
    application: Application,
    config: Arc<Config>,
    http: Arc<Http>,
}

#[derive(Debug, Error)]
#[error("Failed to initialize bot state")]
pub struct StateError;

impl State {
    #[tracing::instrument(name = "init")]
    pub async fn new(cfg: Config) -> Result<Self, StateError> {
        let mut http = Http::builder().token(cfg.bot().token().to_string());
        if let Some(proxy) = cfg.bot().proxy() {
            http = http.proxy(proxy.url().to_string(), proxy.use_http());
        }

        let http = Arc::new(http.build());

        tracing::debug!("Retrieving application info");
        let application = http
            .current_user_application()
            .into_future()
            .change_context(StateError)
            .and_then(|v| v.model().change_context(StateError))
            .await?;

        Ok(Self { application, config: Arc::new(cfg), http })
    }

    /// Shows the application id of a bot
    #[must_use]
    pub fn application(&self) -> &Application {
        &self.application
    }

    #[must_use]
    pub fn config(&self) -> &Config {
        &self.config
    }

    /// Convenient method of `self.http.interaction(self.application.id)`
    #[must_use]
    pub fn interaction(&self) -> InteractionClient<'_> {
        self.http.interaction(self.application.id)
    }

    #[must_use]
    pub fn http(&self) -> &Http {
        &self.http
    }
}

impl Debug for State {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        struct AppDebug<'a>(&'a Application);

        impl<'a> Debug for AppDebug<'a> {
            fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                f.debug_struct("Application")
                    .field("public", &self.0.bot_public)
                    .field("guild_id", &self.0.guild_id)
                    .field("id", &self.0.id)
                    .field("name", &self.0.name)
                    .finish()
            }
        }

        f.debug_struct("State")
            .field("application", &AppDebug(&self.application))
            .field("config", &*self.config)
            .field("http", &Sensitive::new(()))
            .finish()
    }
}
