use actix_web::{App, HttpServer};
use error_stack::{Result, ResultExt};
use kyoka::perform_request;
use std::net::SocketAddr;
use std::sync::Arc;
use twilight_gateway_queue::{LargeBotQueue, LocalQueue, Queue};

use crate::{config, SetupError};

mod router;

#[derive(Debug)]
pub struct AppContext {
    pub big_queue: bool,
    pub queue: Arc<dyn Queue>,
}

pub async fn run(cfg: config::Server) -> Result<(), SetupError> {
    tracing::info!(host = %cfg.host(), port = %cfg.port(), "Starting gateway queue server...");

    let (queue, big_queue): (Arc<dyn Queue>, bool) = if let Some(token) =
        cfg.token()
    {
        tracing::info!(
            "`DISCORD_BOT_TOKEN` is present, getting maximum concurrent sessions"
        );
        let mut http =
            twilight_http::Client::builder().token(token.to_string());

        if let Some(proxy_url) = cfg.proxy_url() {
            http = http.proxy(proxy_url.into(), cfg.proxy_use_http());
        }

        let http = Arc::new(http.build());
        let gateway =
            perform_request!(http.gateway().authed(), SetupError).await?;

        let concurrency =
            gateway.session_start_limit.max_concurrency.try_into().unwrap();

        let queue = Arc::new(LargeBotQueue::new(concurrency, http).await);

        tracing::info!("Received {concurrency} maximum concurrent sessions");
        (queue, concurrency > 1)
    } else {
        tracing::info!("`DISCORD_BOT_TOKEN` is not present, using local queue");
        (Arc::new(LocalQueue::new()), false)
    };

    let address = SocketAddr::from((cfg.host(), cfg.port()));
    tracing::info!("Listening at http://{address}");

    // let prometheus = PrometheusMetricsBuilder::new("gateway_queue")
    //     .endpoint("/metrics")
    //     .build()
    //     .expect("failed to initialize metrics");

    let context = actix_web::web::Data::new(AppContext { big_queue, queue });
    HttpServer::new(move || {
        App::new()
            .wrap(sentry_actix::Sentry::new())
            // .wrap(prometheus.clone())
            .app_data(context.clone())
            .configure(router::configure)
    })
    .workers(1)
    .bind((cfg.host(), cfg.port()))
    .change_context(SetupError)?
    .run()
    .await
    .change_context(SetupError)?;

    tracing::info!("Stopping gateway queue server...");
    Ok(())
}
