mod defaults;
mod util;

#[cfg(test)]
mod tests;

use dotenvy::dotenv;
use error_stack::{Report, Result, ResultExt};
use figment::{providers::Format, Figment};
use once_cell::sync::OnceCell;
use serde::Deserialize;
use std::fmt::Debug;
use std::path::{Path, PathBuf};
use thiserror::Error;
use validator::Validate;

use crate::config::util::IntoValidatorReport;
use crate::util::Sensitive;

use self::util::FigmentErrorAttachable;

#[derive(Deserialize, Validate)]
pub struct BotConfig {
    #[validate(optional, nested)]
    proxy: Option<Proxy>,
    // TODO: too wordy, make it simpler
    #[serde(default = "defaults::reload_commands_on_start")]
    reload_commands_on_start: bool,
    #[validate(length(min = 1), error = "Discord bot token must not be empty")]
    token: Sensitive<String>,
}

impl Debug for BotConfig {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("BotConfig")
            .field("proxy", &self.proxy)
            .field("token", &self.token)
            .finish()
    }
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

#[derive(Debug, Deserialize, Validate)]
pub struct Proxy {
    #[validate(
        with = "validator::extras::validate_url",
        error = "Invalid proxy URL"
    )]
    url: String,
    #[serde(default = "defaults::proxy_use_http")]
    use_http: bool,
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
    color: bool,
    level: Option<LogLevel>,
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
    pub const fn color_enabled(&self) -> bool {
        self.color
    }

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

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
enum LogLevel {
    Trace,
    Debug,
    Info,
    Warn,
    Error,
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

#[derive(Deserialize, Validate)]
pub struct Config {
    #[validate(nested)]
    bot: BotConfig,
    #[serde(default)]
    #[validate(nested)]
    log: LogConfig,
    #[serde(skip)]
    path: OnceCell<PathBuf>,
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
    pub fn path(&self) -> Option<&PathBuf> {
        self.path.get()
    }
}

#[derive(Debug, Error)]
#[error("Failed to load configuration")]
pub struct ConfigLoadError;

const DEFAULT_NAME: &str = "kyoka.toml";
const FILE_ENV: &str = "KYOKA_CONFIG";

impl Config {
    pub fn from_file(path: &Path) -> Result<Self, ConfigLoadError> {
        use figment::providers::Toml;
        dotenv().ok();

        let contents = std::fs::read_to_string(path)
            .change_context(ConfigLoadError)
            .attach_printable_lazy(|| {
                format!("with config file: {}", path.display())
            })?;

        let config = Self::figment()
            .merge(Toml::string(&contents))
            .extract::<Self>()
            .map_err(|e| Report::new(ConfigLoadError).attach_figment_error(e))
            .attach_printable_lazy(|| {
                format!("with config file: {}", path.display())
            })?;

        config
            .validate()
            .into_validator_report()
            .change_context(ConfigLoadError)
            .attach_printable_lazy(|| {
                format!("with config file: {}", path.display())
            })?;

        Ok(config)
    }

    pub fn from_env() -> Result<Self, ConfigLoadError> {
        dotenv().ok();

        let file_path = Self::search_file().change_context(ConfigLoadError)?;
        if let Some(path) = file_path {
            return Self::from_file(&path);
        }

        let config = Self::figment().extract::<Self>().map_err(|e| {
            Report::new(ConfigLoadError).attach_figment_error(e)
        })?;

        config
            .validate()
            .into_validator_report()
            .change_context(ConfigLoadError)?;

        Ok(config)
    }

    pub fn search_file() -> std::io::Result<Option<PathBuf>> {
        let file_env = match dotenvy::var(FILE_ENV) {
            Ok(p) => Some(PathBuf::from(p)),
            Err(dotenvy::Error::Io(e)) => return Err(e),
            Err(..) => None,
        };

        if let Some(file) = file_env {
            return Ok(Some(file));
        }

        let cwd = std::env::current_dir()?.into_boxed_path();
        let mut cwd = Some(&*cwd);

        while let Some(dir) = cwd {
            let entry = dir.join(DEFAULT_NAME);
            let file_exists = std::fs::metadata(&entry)
                .map(|meta| meta.is_file())
                .unwrap_or_default();

            if file_exists {
                return Ok(Some(entry));
            }

            cwd = dir.parent();
        }

        Ok(None)
    }
}

impl Config {
    fn figment() -> Figment {
        use figment::providers::Env;
        Figment::new().merge(Env::prefixed("KYOKA_").map(
            |v| match v.as_str() {
                "BOT_PROXY_USE_HTTP" => "bot.proxy.use_http".into(),
                "BOT_RELOAD_COMMANDS_ON_START" => {
                    "bot.reload_commands_on_start".into()
                },

                v => v.replace('_', ".").into(),
            },
        ))
    }
}

impl Debug for Config {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("Config")
            .field("bot", &self.bot)
            .field("log", &self.log)
            .field("path", &self.path())
            .finish()
    }
}
