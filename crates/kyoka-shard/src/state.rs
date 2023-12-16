use kyoka::config::Config;
use kyoka::util::Sensitive;
use songbird::Songbird;
use std::fmt::Debug;
use std::sync::Arc;
use tokio_util::sync::{CancellationToken, WaitForCancellationFuture};
use twilight_http::client::InteractionClient;
use twilight_model::oauth::Application;

#[derive(Clone)]
pub struct State {
    pub(super) application: Application,
    pub(super) config: Arc<Config>,
    pub(super) http: Arc<twilight_http::Client>,
    pub(super) songbird: Arc<Songbird>,
    pub(super) shutdown_token: CancellationToken,
}

impl State {
    /// Shows the application id of a bot
    #[must_use]
    pub fn application(&self) -> &Application {
        &self.application
    }

    #[must_use]
    pub fn config(&self) -> &Config {
        &self.config
    }

    /// Convenient method of `self.http.interaction(self.application().id)`
    #[must_use]
    pub fn interaction(&self) -> InteractionClient<'_> {
        self.http.interaction(self.application.id)
    }

    #[must_use]
    pub fn http(&self) -> &twilight_http::Client {
        &self.http
    }

    /// Gets the [`Songbird`] object.
    pub fn songbird(&self) -> &Songbird {
        &self.songbird
    }

    /// Attempts to shut down bot session
    pub fn shutdown(&self) {
        self.shutdown_token.cancel();
    }

    pub fn has_shut_down(&self) -> bool {
        self.shutdown_token.is_cancelled()
    }

    /// Waits until it is announced to shut down
    pub fn wait_for_shutdown(&self) -> WaitForCancellationFuture<'_> {
        self.shutdown_token.cancelled()
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
