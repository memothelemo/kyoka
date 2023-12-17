mod cmd;
mod handler;
mod state;

pub use cmd::{RunError, Runner};
pub use state::State;

use crate::BotQueue;
use crate::{config, App, SetupError};
use std::sync::Arc;

use error_stack::{Result, ResultExt};
use kyoka::perform_request;
use songbird::Songbird;
use tokio::task::JoinSet;
use twilight_gateway::Intents;
use twilight_gateway::Shard;
use twilight_gateway_queue::{LargeBotQueue, Queue};
use twilight_http::Client as Http;
use twilight_model::id::marker::UserMarker;

#[must_use]
fn default_gateway_intents() -> Intents {
    Intents::GUILDS | Intents::GUILD_MESSAGES
}

async fn init_shards(
    cfg: &config::Shard,
    http: &Arc<Http>,
) -> Result<Vec<Shard>, SetupError> {
    let mut gateway_cfg = twilight_gateway::Config::builder(
        cfg.bot().token().into(),
        default_gateway_intents(),
    );

    if let Some(proxy_url) = cfg.bot().gateway_proxy_url() {
        gateway_cfg = gateway_cfg.proxy_url(proxy_url.into());
    }

    let gateway_cfg = gateway_cfg.build();
    let gateway_connect_info =
        perform_request!(http.gateway().authed(), SetupError).await?;

    let (id, amount, total) = match cfg.connect_amount() {
        config::ShardConnectAmount::Manual { id, amount, total } => {
            (*id, amount.get(), total.get())
        },
        config::ShardConnectAmount::UseRecommended => {
            tracing::debug!("Getting recommended amount of shards...");
            (0, gateway_connect_info.shards, gateway_connect_info.shards)
        },
    };

    tracing::info!("Setting up gateway queue...");
    let queue: Arc<dyn Queue> = if let Some(queue_url) = cfg.gateway_queue_url()
    {
        Arc::new(BotQueue::new(queue_url).await?)
    } else {
        let buckets = gateway_connect_info
            .session_start_limit
            .max_concurrency
            .try_into()
            .unwrap();

        let queue = LargeBotQueue::new(buckets, http.clone()).await;
        Arc::new(queue)
    };

    tracing::debug!(
        id = %id,
        amount = %amount,
        total = %total,
        "Creating {amount} shard/s..."
    );

    let min = id;
    let max = id + amount;

    let shards = twilight_gateway::stream::create_range(
        min..max,
        amount,
        gateway_cfg,
        |_, builder| builder.queue(queue.clone()).build(),
    )
    .collect::<Vec<_>>();

    Ok(shards)
}

#[tracing::instrument(skip(app))]
async fn init(app: App) -> Result<(State, Vec<Shard>), SetupError> {
    let cfg = config::Shard::from_env().change_context(SetupError)?;
    tracing::info!("Starting Discord bot client");

    let http = Arc::new(kyoka::util::make_http_client(cfg.bot()));
    tracing::info!("Retrieving application info");
    let info =
        perform_request!(http.current_user_application(), SetupError).await?;

    if cfg.bot().reload_commands_on_start() {
        tracing::info!(
            "Reload commands on start is enabled; reloading all commands"
        );
        kyoka::util::setup_cmds(http.interaction(info.id))
            .await
            .change_context(SetupError)?;
    }

    let shards = init_shards(&cfg, &http).await?;
    let clusters = songbird::shards::TwilightMap::new({
        let mut map = std::collections::HashMap::new();
        for shard in shards.iter() {
            map.insert(shard.id().number(), shard.sender());
        }
        map
    });

    let songbird =
        Songbird::twilight(clusters.into(), info.id.cast::<UserMarker>());

    let state = State::new(app, cfg, http, info, songbird);
    Ok((state, shards))
}

pub async fn start(app: App) -> Result<(), SetupError> {
    let mut handle = JoinSet::new();
    let (state, shards) = init(app.clone()).await?;
    tracing::info!("Starting bot with {} shard/s", shards.len());

    for mut shard in shards {
        let state = state.clone();
        handle.spawn(async move {
            handler::shard(state, &mut shard).await;
        });
    }

    tokio::select! {
        _ = kyoka::util::shutdown_signal() => {
            app.perform_shutdown("Received shutdown signal");
        },
        _ = app.shutdown_signal() => {}
    };

    tracing::info!("Waiting for all shards to finish their tasks");
    while handle.join_next().await.is_some() {}

    tracing::info!("All shards are successfully shut down");
    Ok(())
}
