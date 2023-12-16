mod defaults;
mod structs;
mod util;

#[cfg(test)]
mod tests;

use dotenvy::dotenv;
use error_stack::{Report, Result, ResultExt};
use figment::{providers::Format, Figment};
use std::fmt::Debug;
// use std::num::NonZeroU64;
use std::path::{Path, PathBuf};
use thiserror::Error;
use validator::Validate;

use self::util::FigmentErrorAttachable;
use crate::config::util::IntoValidatorReport;

pub use structs::*;

impl Debug for BotConfig {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("BotConfig")
            .field("proxy", &self.proxy)
            .field("token", &self.token)
            .finish()
    }
}

// impl Default for ShardConfig {
//     fn default() -> Self {
//         Self {
//             id: 0,
//             amount: NonZeroU64::new(1).unwrap(),
//             total: NonZeroU64::new(1).unwrap(),
//         }
//     }
// }

// impl Validate for ShardConfig {
//     fn validate(&self) -> std::result::Result<(), validator::ValidateError> {
//         use validator::ValidateError;

//         let mut fields = ValidateError::field_builder();

//         let min = self.id;
//         let max = self.id + self.amount();

//         // Amount of shards to be connected in a single process
//         // must not exceed to the total amount of shards that a bot
//         // must connected recommended by Discord.

//         todo!()
//     }
// }

#[derive(Debug, Error)]
#[error("Failed to load configuration")]
pub struct ConfigLoadError;

const DEFAULT_NAME: &str = "kyoka.toml";
const FILE_ENV: &str = "KYOKA_CONFIG";
const ALT_FILE_PATH: &str = "config/kyoka.toml";

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

        // Fail safeover in case if it is running under Docker
        // container. This is just to deal with the limitations
        // of binding a host path as a volume for a Docker container.
        if crate::util::is_running_in_docker()
            && std::fs::metadata(ALT_FILE_PATH)
                .map(|meta| meta.is_file())
                .unwrap_or_default()
        {
            return Ok(Some(ALT_FILE_PATH.into()));
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
            .field("path", &self.metrics())
            .finish()
    }
}
