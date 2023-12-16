use once_cell::sync::OnceCell;
use serde::Deserialize;
use std::fmt::Debug;
use std::net::IpAddr;
// use std::num::NonZeroU64;
use std::path::PathBuf;
use validator::Validate;

use super::defaults;
use crate::util::Sensitive;

#[derive(Deserialize, Validate)]
pub struct BotConfig {
    #[validate(optional, nested)]
    pub(super) proxy: Option<Proxy>,
    // TODO: too wordy, make it simpler
    #[serde(default = "defaults::reload_commands_on_start")]
    pub(super) reload_commands_on_start: bool,
    #[validate(length(min = 1), error = "Discord bot token must not be empty")]
    pub(super) token: Sensitive<String>,
}

impl BotConfig {
    #[must_use]
    pub fn proxy(&self) -> Option<&Proxy> {
        self.proxy.as_ref()
    }

    /// Reloads all required commands upon startup
    #[must_use]
    pub const fn reload_commands_on_start(&self) -> bool {
        self.reload_commands_on_start
    }

    #[must_use]
    pub fn token(&self) -> &str {
        &self.token
    }
}

// // TODO: Implement global gateway queue
// #[derive(Debug, Deserialize)]
// pub struct ShardConfig {
//     /// Starting index of a shard
//     ///
//     /// It defaults to `0` if not specified in config
//     #[serde(default = "defaults::shard_id")]
//     pub(super) id: u64,
//     /// Amount of shards will be connected in a single process.
//     ///
//     /// It defaults to `1` if not specified in config
//     #[serde(default = "defaults::shard_cfg_amount")]
//     pub(super) amount: NonZeroU64,
//     /// Total amount of shards must be connected
//     ///
//     /// It defaults to `1` if not specified in config
//     #[serde(default = "defaults::shard_cfg_amount")]
//     pub(super) total: NonZeroU64,
// }

// impl ShardConfig {
//     #[must_use]
//     pub const fn id(&self) -> u64 {
//         self.id
//     }

//     #[must_use]
//     pub const fn amount(&self) -> u64 {
//         self.amount.get()
//     }

//     #[must_use]
//     pub const fn total(&self) -> u64 {
//         self.total.get()
//     }
// }

#[derive(Debug, Deserialize, Validate)]
pub struct Proxy {
    #[validate(
        with = "validator::extras::validate_url",
        error = "Invalid proxy URL"
    )]
    pub(super) url: String,
    #[serde(default = "defaults::proxy_use_http")]
    pub(super) use_http: bool,
}

impl Proxy {
    #[must_use]
    pub fn url(&self) -> &str {
        self.url.as_str()
    }

    #[must_use]
    pub const fn use_http(&self) -> bool {
        self.use_http
    }
}

#[derive(Debug, Deserialize, Validate)]
#[serde(default)]
pub struct LogConfig {
    pub(super) color: bool,
    pub(self) level: Option<LogLevel>,
}

impl LogConfig {
    #[must_use]
    pub const fn color_enabled(&self) -> bool {
        self.color
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
enum LogLevel {
    Trace,
    Debug,
    Info,
    Warn,
    Error,
}

impl Default for LogConfig {
    fn default() -> Self {
        // Really want to see the debug logging in dev environments
        #[cfg(debug_assertions)]
        let level = Some(LogLevel::Debug);
        #[cfg(not(debug_assertions))]
        let level = Some(LogLevel::Debug);

        Self { color: false, level }
    }
}

impl LogConfig {
    #[must_use]
    pub const fn level(&self) -> tracing::level_filters::LevelFilter {
        use tracing::level_filters::LevelFilter;
        match self.level {
            Some(LogLevel::Trace) => LevelFilter::TRACE,
            Some(LogLevel::Debug) => LevelFilter::DEBUG,
            Some(LogLevel::Info) => LevelFilter::INFO,
            Some(LogLevel::Warn) => LevelFilter::WARN,
            Some(LogLevel::Error) => LevelFilter::ERROR,
            None => LevelFilter::OFF,
        }
    }
}

impl<'de> Deserialize<'de> for LogLevel {
    fn deserialize<D>(deserializer: D) -> std::result::Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        struct Visitor;

        impl<'de> serde::de::Visitor<'de> for Visitor {
            type Value = LogLevel;

            fn expecting(
                &self,
                f: &mut std::fmt::Formatter<'_>,
            ) -> std::fmt::Result {
                f.write_str("log level")
            }

            fn visit_str<E>(
                self,
                v: &str,
            ) -> std::result::Result<Self::Value, E>
            where
                E: serde::de::Error,
            {
                match v {
                    "trace" => Ok(LogLevel::Trace),
                    "debug" => Ok(LogLevel::Debug),
                    "info" => Ok(LogLevel::Info),
                    "warn" => Ok(LogLevel::Warn),
                    "error" => Ok(LogLevel::Error),
                    _ => Err(serde::de::Error::unknown_variant(
                        v,
                        &["trace", "debug", "info", "warn", "error"],
                    )),
                }
            }
        }

        deserializer.deserialize_str(Visitor)
    }
}

#[derive(Debug, Deserialize, Validate)]
pub struct Metrics {
    #[serde(default = "defaults::metrics_host")]
    pub(super) host: IpAddr,
    #[serde(default = "defaults::metrics_port")]
    pub(super) port: u16,
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

#[derive(Deserialize, Validate)]
pub struct Config {
    #[validate(nested)]
    pub(super) bot: BotConfig,
    #[serde(default)]
    #[validate(nested)]
    pub(super) log: LogConfig,
    #[serde(default)]
    pub(super) metrics: Option<Metrics>,
    #[serde(skip)]
    pub(super) path: OnceCell<PathBuf>,
}

impl Config {
    #[must_use]
    pub fn bot(&self) -> &BotConfig {
        &self.bot
    }

    #[must_use]
    pub fn log(&self) -> &LogConfig {
        &self.log
    }

    #[must_use]
    pub fn metrics(&self) -> Option<&Metrics> {
        self.metrics.as_ref()
    }

    #[must_use]
    pub fn path(&self) -> Option<&PathBuf> {
        self.path.get()
    }
}
