use std::time::Duration;

use actix_web_prom::PrometheusMetricsBuilder;
use error_stack::{Result, ResultExt};
use kyoka::metrics::SystemMetrics;
use tokio::task::JoinSet;

use crate::{config, App, SetupError};

pub async fn start(app: App) -> Result<(), SetupError> {
    if !config::Metrics::is_enabled() {
        return Ok(());
    }

    let mut tasks = JoinSet::new();
    let cfg = config::Metrics::from_env().change_context(SetupError)?;
    tracing::info!(cfg.metrics = ?cfg, "Starting HTTP metrics server");

    let prometheus = PrometheusMetricsBuilder::new("api")
        .endpoint("/metrics")
        .build()
        .unwrap();

    let system_metrics = SystemMetrics::new().change_context(SetupError)?;
    app.metrics().setup(&prometheus).change_context(SetupError)?;
    system_metrics.setup(&prometheus).change_context(SetupError)?;

    tracing::info!(
        "HTTP metrics server is listening at http://{}:{}",
        cfg.host(),
        cfg.port()
    );

    let system_metrics = system_metrics.clone();
    let job_app = app.clone();
    tasks.spawn(async move {
        tracing::debug!("Starting system usage metrics measure job");
        let mut interval = tokio::time::interval(Duration::from_secs(3));
        let app = job_app;
        loop {
            tokio::select! {
                _ = interval.tick() => {}
                _ = app.shutdown_signal() => {
                    break;
                }
            };

            let system_metrics = system_metrics.clone();
            tokio::task::spawn_blocking(move || {
                system_metrics.update_usage();
            })
            .await
            .expect("Failed to update metrics");
        }
        tracing::debug!("Ending system usage metrics measure job");
    });

    let server = actix_web::HttpServer::new(move || {
        actix_web::App::new()
            .configure(super::router::configure)
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
            } else {
                app.perform_shutdown("Received shutdown signal");
            }
        },
        _ = app.shutdown_signal() => {
            app.perform_shutdown("Received shutdown signal");
            handle.stop(true).await;
        }
    }

    tracing::info!("Waiting all metrics tasks to be finished");
    while tasks.join_next().await.is_some() {}

    Ok(())
}
