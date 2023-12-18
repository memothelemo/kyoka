mod router;
pub mod server;

use actix_web_prom::PrometheusMetrics;
use error_stack::{Result, ResultExt};
use kyoka::metrics::MetricsSetupError;
use prometheus::{Gauge, IntGauge};
use prometheus_macros::composite_metric;

composite_metric! {
    #[derive(Debug, Clone)]
    pub struct Metrics {
        #[name = "events_processed"]
        #[desc = "All events processed in all shards"]
        events_processed: IntGauge,
        #[name = "shard_latency"]
        #[desc = "Latency of each shards"]
        #[labels = ["shard"]]
        shard_latency: Gauge,
    }
}

impl Metrics {
    pub fn setup(
        &self,
        metrics: &PrometheusMetrics,
    ) -> Result<(), MetricsSetupError> {
        metrics
            .registry
            .register(Box::new(self.shard_latency.clone()))
            .change_context(MetricsSetupError)?;

        metrics
            .registry
            .register(Box::new(self.events_processed.clone()))
            .change_context(MetricsSetupError)?;

        Ok(())
    }
}

// #[derive(Debug, Clone)]
// pub struct Metrics {
//     pub system: SystemMetrics,
//     pub shard_latency: HistogramVec,
// }

// impl Metrics {
//     pub fn new() -> Result<Self, MetricsSetupError> {
//         // let shard_latency = HistogramVec::new(op, &["shard"]);
//         Ok(Self { , system: SystemMetrics::new()? })
//     }

//     pub fn setup(
//         &self,
//         metrics: &PrometheusMetrics,
//     ) -> Result<(), MetricsSetupError> {
//         self.system.setup(metrics)?;
//         Ok(())
//     }
// }
