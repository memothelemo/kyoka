use actix_web_prom::PrometheusMetrics;
use error_stack::{Result, ResultExt};
use prometheus::Gauge;
use systemstat::Platform;
use thiserror::Error;

#[derive(Debug, Error)]
#[error("Failed to setup metrics")]
pub struct MetricsSetupError;

#[derive(Debug, Clone)]
pub struct SystemMetrics {
    pub cpu_usage: Gauge,
    pub mem_usage: Gauge,
    pub mem_swap_usage: Gauge,
}

impl SystemMetrics {
    pub fn new() -> Result<Self, MetricsSetupError> {
        let cpu_usage = Gauge::new("cpu_usage", "Current CPU usage in percent")
            .change_context(MetricsSetupError)?;

        let mem_usage =
            Gauge::new("mem_usage", "Current memory usage in percent")
                .change_context(MetricsSetupError)?;

        let mem_swap_usage = Gauge::new(
            "mem_swap_usage",
            "Current memory swap usage in percent",
        )
        .change_context(MetricsSetupError)?;

        Ok(Self { cpu_usage, mem_usage, mem_swap_usage })
    }

    pub fn setup(
        &self,
        metrics: &PrometheusMetrics,
    ) -> Result<(), MetricsSetupError> {
        metrics
            .registry
            .register(Box::new(self.cpu_usage.clone()))
            .change_context(MetricsSetupError)?;

        metrics
            .registry
            .register(Box::new(self.mem_usage.clone()))
            .change_context(MetricsSetupError)?;

        metrics
            .registry
            .register(Box::new(self.mem_swap_usage.clone()))
            .change_context(MetricsSetupError)?;

        Ok(())
    }

    /// Updates CPU and memory usage for system metrics.
    ///
    /// This function will yield.
    pub fn update_usage(&self) {
        let sys = systemstat::System::new();
        match sys.cpu_load_aggregate().and_then(|v| {
            std::thread::sleep(std::time::Duration::from_secs(1));
            v.done()
        }) {
            Ok(usage) => {
                let usage = (usage.system * 100.) + (usage.user * 100.);
                self.cpu_usage.set(f64::trunc(usage.into()));
            },
            Err(error) => {
                tracing::warn!(
                    ?error,
                    "Failed to load CPU usage of the system"
                );
            },
        }

        match sys.memory_and_swap() {
            Ok((memory, swap)) => {
                let memory_used = memory.total.0 - memory.free.0;
                let memory_used =
                    (memory_used as f64 / memory.total.0 as f64) * 100.;

                let swap_used = swap.total.0 - swap.free.0;
                let swap_used = (swap_used as f64 / swap.total.0 as f64) * 100.;

                self.mem_usage.set(f64::trunc(memory_used));
                self.mem_usage.set(f64::trunc(swap_used));
            },
            Err(error) => {
                tracing::warn!(
                    ?error,
                    "Failed to load memory usage of the system"
                );
            },
        }
    }
}
