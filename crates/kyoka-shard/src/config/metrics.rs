use error_stack::{Result, ResultExt};
use kyoka::util::env;
use std::net::{IpAddr, Ipv4Addr};

use super::LoadError;

#[derive(Debug)]
pub struct Metrics {
    host: IpAddr,
    port: u16,
}

const DEFAULT_PORT: u16 = 3421;
const DEFAULT_HOST: IpAddr = IpAddr::V4(Ipv4Addr::new(127, 0, 0, 1));

impl Metrics {
    pub fn is_enabled() -> bool {
        env::var_parse::<bool>("METRICS_ENABLED")
            .ok()
            .and_then(|v| v)
            .unwrap_or_default()
    }

    #[track_caller]
    pub fn from_env() -> Result<Self, LoadError> {
        let host = env::var_parse("METRICS_HOST")
            .change_context(LoadError)?
            .unwrap_or(DEFAULT_HOST);

        let port = env::var_parse("METRICS_PORT")
            .change_context(LoadError)?
            .unwrap_or(DEFAULT_PORT);

        Ok(Self { host, port })
    }
}

impl Metrics {
    #[must_use]
    pub const fn host(&self) -> IpAddr {
        self.host
    }

    #[must_use]
    pub const fn port(&self) -> u16 {
        self.port
    }
}
