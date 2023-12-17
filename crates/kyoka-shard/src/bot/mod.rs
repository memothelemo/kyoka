// mod state;
// pub use state::State;

// use tokio::task::JoinSet;
// use twilight_gateway_queue::LargeBotQueue;

// use crate::{config, App, SetupError};
// use std::sync::Arc;

// use error_stack::{Result, ResultExt};
// use kyoka::perform_request;
// use songbird::Songbird;
// use twilight_gateway::Intents;
// use twilight_gateway::Shard;
// use twilight_http::Client as Http;
// use twilight_model::id::marker::UserMarker;

// #[must_use]
// fn default_gateway_intents() -> Intents {
//     Intents::GUILDS | Intents::GUILD_MESSAGES
// }

// #[tracing::instrument(skip(app))]
// async fn init(app: App) -> Result<(State, Vec<Shard>), SetupError> {
//     let cfg = config::Shard::from_env().change_context(SetupError)?;
//     tracing::info!("Starting Discord bot client");

//     let mut http = Http::builder().token(cfg.bot().token().to_string());
//     let mut gateway = twilight_gateway::Config::builder(
//         cfg.bot().token().into(),
//         default_gateway_intents(),
//     );

//     if let Some(proxy_url) = cfg.bot().proxy_url() {
//         http = http.proxy(proxy_url.into(), cfg.bot().proxy_use_http());
//     }

//     if let Some(proxy_url) = cfg.bot().gateway_proxy_url() {
//         gateway = gateway.proxy_url(proxy_url.into());
//     }

//     let http = Arc::new(http.build());
//     let gateway = gateway.build();

//     tracing::info!("Retrieving application info");
//     let info =
//         perform_request!(http.current_user_application(), SetupError).await?;

//     let queue = Arc::new(LargeBotQueue::new(concurrency, http.clone()).await);
//     let shards = {
//         let (shard_id, shard_amount, shard_total, concurrency) =
//             match cfg.connect_amount() {
//                 config::ShardConnectAmount::Manual { id, amount, total } => {
//                     (*id, amount.get(), total.get(), 1)
//                 },
//                 config::ShardConnectAmount::UseRecommended => {
//                     tracing::debug!("Getting recommended amount of shards...");
//                     let info =
//                         perform_request!(http.gateway().authed(), SetupError)
//                             .await?;

//                     (
//                         0,
//                         info.shards,
//                         info.shards,
//                         info.session_start_limit
//                             .max_concurrency
//                             .try_into()
//                             .unwrap(),
//                     )
//                 },
//             };

//         tracing::debug!(
//             id = %shard_id,
//             amount = %shard_amount,
//             total = %shard_total,
//             "Creating {shard_amount} shard/s..."
//         );

//         let min = shard_id;
//         let max = shard_id + shard_amount;

//         twilight_gateway::stream::create_range(
//             min..max,
//             shard_amount,
//             gateway,
//             |_, builder| builder.queue(queue.clone()).build(),
//         )
//     }
//     .collect::<Vec<_>>();

//     let clusters = songbird::shards::TwilightMap::new({
//         let mut map = std::collections::HashMap::new();
//         for shard in shards.iter() {
//             map.insert(shard.id().number(), shard.sender());
//         }
//         map
//     });

//     let songbird =
//         Songbird::twilight(clusters.into(), info.id.cast::<UserMarker>());

//     let state = State {
//         app: app.clone(),
//         config: Arc::new(cfg),
//         http,
//         info,
//         songbird: Arc::new(songbird),
//     };

//     Ok((state, shards))
// }

// pub async fn start(app: App) -> Result<(), SetupError> {
//     let (state, shards) = init(app.clone()).await?;
//     tracing::info!("Starting bot with {} shard/s", shards.len());

//     let mut handle = JoinSet::new();

//     for mut shard in shards {
//         let app = app.clone();
//         handle.spawn(async move {
//             loop {
//                 tokio::select! {
//                     _ = shard.next_event() => {},
//                     _ = app.shutdown_signal() => {
//                         break;
//                     }
//                 }
//             }
//             shard.close(twilight_gateway::CloseFrame::NORMAL).await;
//         });
//     }
//     tracing::info!("All shards are successfully connected");

//     tokio::select! {
//         _ = kyoka::util::shutdown_signal() => {
//             app.perform_shutdown("Received shutdown signal");
//         },
//         _ = app.shutdown_signal() => {}
//     };

//     tracing::info!("Waiting for all shards to finish their tasks");
//     while handle.join_next().await.is_some() {}

//     tracing::info!("All shards are successfully shut down");
//     Ok(())
// }
