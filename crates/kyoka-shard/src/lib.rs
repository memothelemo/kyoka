mod cmd;
mod state;

pub mod process;
pub mod router;
pub use self::state::State;

use error_stack::{Result, ResultExt};
use kyoka::config::Config;
use kyoka::perform_request;
use kyoka::setup::SetupError;
use songbird::Songbird;
use std::sync::Arc;
use tokio_util::{sync::CancellationToken, task::TaskTracker};
use twilight_gateway::{
    error::ReceiveMessageErrorType, CloseFrame, Intents, Message, Shard,
};
use twilight_http::Client as Http;
use twilight_model::id::marker::UserMarker;

#[tracing::instrument(skip(shard), fields(id = %shard.id()))]
pub async fn shard_runner(state: State, shard: &mut Shard) {
    let mut fatal_shutdown = false;
    let tracker = TaskTracker::new();

    loop {
        tokio::select! {
            result = shard.next_event() => {
                let event = match result {
                    Ok(event) => event,
                    Err(source) => {
                        tracing::warn!(?source, "Got shard message error");
                        if source.is_fatal() {
                            fatal_shutdown = true;
                            break;
                        }
                        continue;
                    },
                };
                state.songbird().process(&event).await;

                let state = state.clone();
                tracker.spawn(async move {
                    if let Err(error) = process::event(state, event).await {
                        tracing::warn!(?error, "Failed to process event");
                    }
                });
            },
            _ = state.wait_for_shutdown() => {
                break;
            },
        }
    }

    if fatal_shutdown && !state.has_shut_down() {
        tracing::warn!("Fatal error given; shutting down all shards...");
        state.shutdown();
    }

    if !shard.status().is_disconnected() {
        if let Err(error) = shard.close(CloseFrame::NORMAL).await {
            tracing::error!(?error, "Failed to close shard connection");
        }

        // Wait until WebSocket connection is FINALLY CLOSED
        loop {
            match shard.next_message().await {
                Ok(Message::Close(..)) | Ok(Message::Text(..)) => break,
                Err(source)
                    if matches!(source.kind(), ReceiveMessageErrorType::Io) =>
                {
                    break;
                },
                Err(source) => {
                    tracing::warn!(?source, "Got shard message error");
                },
            }
        }
    }

    if tracker.close() {
        tracing::info!("Waiting for all tasks to be completed");
        tracker.wait().await;
    }
}

#[tracing::instrument]
pub async fn init(cfg: Config) -> Result<(State, Vec<Shard>), SetupError> {
    let mut gateway_cfg_builder = twilight_gateway::Config::builder(
        cfg.bot().token().to_string(),
        // Default gateway intents
        Intents::GUILDS | Intents::GUILD_MESSAGES,
    );
    let mut http = Http::builder().token(cfg.bot().token().to_string());
    if let Some(proxy) = cfg.bot().proxy() {
        http = http.proxy(proxy.url().to_string(), proxy.use_http());
        gateway_cfg_builder =
            gateway_cfg_builder.proxy_url(proxy.url().to_string());
    }

    let http = Arc::new(http.build());

    tracing::debug!("Retrieving application info");
    let application =
        perform_request!(http.current_user_application(), SetupError).await?;

    tracing::debug!("Getting recommended amount of shards");
    let shards = twilight_gateway::stream::create_recommended(
        &http,
        gateway_cfg_builder.build(),
        |_, builder| builder.build(),
    )
    .await
    .change_context(SetupError)?
    .collect::<Vec<_>>();

    let clusters = songbird::shards::TwilightMap::new({
        let mut map = std::collections::HashMap::new();
        for shard in shards.iter() {
            map.insert(shard.id().number(), shard.sender());
        }
        map
    });

    let songbird = Songbird::twilight(
        clusters.into(),
        application.id.cast::<UserMarker>(),
    );

    let state = State {
        application,
        config: Arc::new(cfg),
        http,
        songbird: Arc::new(songbird),
        shutdown_token: CancellationToken::new(),
    };

    Ok((state, shards))
}
