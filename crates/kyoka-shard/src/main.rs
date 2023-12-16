use actix_web_prom::PrometheusMetricsBuilder;
use error_stack::{Result, ResultExt};
use kyoka::config::Config;
use kyoka::setup::SetupError;
use kyoka_shard::State;
use tokio::task::JoinSet;
use twilight_gateway::Shard;

#[tracing::instrument(skip_all, fields(state = ?state, shards.len = %shards.len()))]
async fn bot(state: State, shards: Vec<Shard>) -> Result<(), SetupError> {
    tracing::info!("Starting bot with {} shard/s", shards.len());

    let mut handle = JoinSet::new();
    let shard_len = shards.len();
    for mut shard in shards {
        let state = state.clone();
        handle.spawn(async move {
            let span = tracing::info_span!("shard_thread", ?state, shard.id = ?shard.id(), shards.len = %shard_len);
            let _guard = span.enter();
            kyoka_shard::shard_runner(state, &mut shard).await;
        });
    }

    tokio::select! {
        _ = kyoka::util::shutdown_signal() => {
            tracing::info!("Shutdown signal is triggered. Shutting down shard...");
            state.shutdown();
        },
        _ = state.wait_for_shutdown() => {}
    };

    // We need to safely shut down all shards to avoid ruining
    // user experience and maybe data loss
    tracing::info!("Waiting for all shards to be shut down");
    while handle.join_next().await.is_some() {}

    tracing::info!("All shards are successfully shut down");

    Ok(())
}

async fn http_metrics(state: State) -> Result<(), SetupError> {
    let Some(cfg) = state.config().metrics() else { return Ok(()) };
    tracing::info!(cfg.metrics = ?cfg, "Starting HTTP metrics server");

    let prometheus = PrometheusMetricsBuilder::new("api")
        .endpoint("/metrics")
        .build()
        .unwrap();

    tracing::info!(
        "HTTP metrics is listening at http://{}:{}",
        cfg.host(),
        cfg.port()
    );

    let server = actix_web::HttpServer::new(move || {
        actix_web::App::new()
            .configure(kyoka_shard::router::configure)
            .wrap(prometheus.clone())
    })
    .workers(1)
    .bind((cfg.host(), cfg.port()))
    .change_context(SetupError)?
    .run();

    tokio::select! {
        Err(e) = server => {
            return Err(e).change_context(SetupError);
        },
        _ = state.wait_for_shutdown() => {
            tracing::info!("Shutting down HTTP metrics service");
        }
    }

    Ok(())
}

#[tracing::instrument]
async fn init(cfg: Config) -> Result<(), SetupError> {
    let (state, shards) = kyoka_shard::init(cfg).await?;
    let (bot, http) =
        tokio::join!(bot(state.clone(), shards), http_metrics(state));

    bot?;
    http?;

    tracing::info!("Finished shard session");
    Ok(())
}

fn main() -> Result<(), SetupError> {
    let cfg = Config::from_env().change_context(SetupError)?;
    kyoka::setup::init_logging(&cfg)?;

    tokio::runtime::Builder::new_current_thread()
        .enable_all()
        .build()
        .unwrap()
        .block_on(init(cfg))
}
