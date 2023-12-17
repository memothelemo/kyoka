use kyoka::util::Sensitive;
use songbird::Songbird;
use std::fmt::Debug;
use std::sync::Arc;
use twilight_http::client::InteractionClient;
use twilight_model::oauth::Application;

use crate::{config, App};

#[derive(Clone)]
pub struct State {
    pub(super) app: App,
    pub(super) config: Arc<config::Shard>,
    pub(super) http: Arc<twilight_http::Client>,
    pub(super) info: Application,
    pub(super) songbird: Arc<Songbird>,
}

impl State {
    pub(super) fn new(
        app: App,
        config: config::Shard,
        http: Arc<twilight_http::Client>,
        info: Application,
        songbird: Songbird,
    ) -> Self {
        Self {
            app: app.clone(),
            config: Arc::new(config),
            http,
            info,
            songbird: Arc::new(songbird),
        }
    }
}

impl State {
    #[must_use]
    pub fn app(&self) -> &crate::App {
        &self.app
    }

    /// Shows the application information of a Discord bot
    #[must_use]
    pub fn info(&self) -> &Application {
        &self.info
    }

    #[must_use]
    pub fn config(&self) -> &config::Shard {
        &self.config
    }

    /// Convenient method of `self.http.interaction(self.application().id)`
    #[must_use]
    pub fn interaction(&self) -> InteractionClient<'_> {
        self.http.interaction(self.info.id)
    }

    #[must_use]
    pub fn http(&self) -> &twilight_http::Client {
        &self.http
    }

    /// Gets the [`Songbird`] object.
    pub fn songbird(&self) -> &Songbird {
        &self.songbird
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
            .field("application", &AppDebug(&self.info))
            .field("config", &*self.config)
            .field("http", &Sensitive::new(()))
            .finish()
    }
}
