use actix_web_prom::PrometheusMetricsBuilder;
use error_stack::{Result, ResultExt};

use crate::{config, App, SetupError};

mod router;

pub async fn start(app: App) -> Result<(), SetupError> {
    if !config::Metrics::is_enabled() {
        return Ok(());
    }

    let cfg = config::Metrics::from_env().change_context(SetupError)?;
    tracing::info!(cfg.metrics = ?cfg, "Starting HTTP metrics server");

    let prometheus = PrometheusMetricsBuilder::new("api")
        .endpoint("/metrics")
        .build()
        .unwrap();

    tracing::info!(
        "HTTP metrics server is listening at http://{}:{}",
        cfg.host(),
        cfg.port()
    );

    let server = actix_web::HttpServer::new(move || {
        actix_web::App::new()
            .configure(router::configure)
            .wrap(prometheus.clone())
            .wrap(sentry_actix::Sentry::new())
    })
    .workers(1)
    .bind((cfg.host(), cfg.port()))
    .change_context(SetupError)?
    .run();

    // actix will handle graceful shutdowns
    let handle = server.handle();
    tokio::select! {
        result = server => {
            if let Err(error) = result {
                tracing::error!(?error, "Metrics server failed");
                app.perform_shutdown("Received metrics server fatal error");
            }
        },
        _ = app.shutdown_signal() => {
            handle.stop(true).await;
        }
    }

    Ok(())
}
