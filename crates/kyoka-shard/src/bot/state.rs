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

// impl State {
//     pub fn new(cfg: config::Bot) {
//         let mut gateway_cfg_builder = twilight_gateway::Config::builder(
//             cfg.bot().token().to_string(),
//             // Default gateway intents
//             Intents::GUILDS | Intents::GUILD_MESSAGES,
//         );
//         let mut http = Http::builder().token(cfg.bot().token().to_string());
//         if let Some(proxy) = cfg.bot().proxy() {
//             http = http.proxy(proxy.url().to_string(), proxy.use_http());
//             gateway_cfg_builder =
//                 gateway_cfg_builder.proxy_url(proxy.url().to_string());
//         }

//         let http = Arc::new(http.build());

//         tracing::debug!("Retrieving application info");
//         let application =
//             perform_request!(http.current_user_application(), SetupError)
//                 .await?;

//         tracing::debug!("Getting recommended amount of shards");
//         let shards = twilight_gateway::stream::create_recommended(
//             &http,
//             gateway_cfg_builder.build(),
//             |_, builder| builder.build(),
//         )
//         .await
//         .change_context(SetupError)?
//         .collect::<Vec<_>>();

//         let clusters = songbird::shards::TwilightMap::new({
//             let mut map = std::collections::HashMap::new();
//             for shard in shards.iter() {
//                 map.insert(shard.id().number(), shard.sender());
//             }
//             map
//         });

//         let songbird = Songbird::twilight(
//             clusters.into(),
//             application.id.cast::<UserMarker>(),
//         );

//         let state = State {
//             application,
//             config: Arc::new(cfg),
//             http,
//             songbird: Arc::new(songbird),
//             shutdown_token: CancellationToken::new(),
//         };
//     }
// }

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
