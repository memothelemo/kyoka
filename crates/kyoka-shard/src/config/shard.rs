use error_stack::{Report, Result, ResultExt};
use kyoka::util::env;
use std::num::NonZeroU64;
use thiserror::Error;

use super::LoadError;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ShardConnectAmount {
    UseRecommended,
    Manual {
        id: u64,
        /// How many shards must be connected in a
        /// single or every process
        amount: NonZeroU64,
        /// Total shards, recommended by Discord
        total: NonZeroU64,
    },
}

impl ShardConnectAmount {
    #[track_caller]
    fn from_env() -> Result<Self, LoadError> {
        let use_recommended = env::var_parse("SHARD_USE_RECOMMENDED")
            .change_context(LoadError)?
            .unwrap_or(true);

        if use_recommended {
            return Ok(Self::UseRecommended);
        }

        let id = env::required_var_parse("SHARD_ID")
            .change_context(LoadError)
            .attach_printable(RECOMMENDED_SUGGESTION)?;

        let total: NonZeroU64 = env::required_var_parse("SHARD_TOTAL")
            .change_context(LoadError)
            .attach_printable(RECOMMENDED_SUGGESTION)?;

        let amount = env::var_parse("SHARD_AMOUNT")
            .change_context(LoadError)?
            .unwrap_or(NonZeroU64::new(1).unwrap());

        if amount.get() > total.get() {
            return Err(InvalidShardConfig::AmountGtTotal)
                .attach_printable_lazy(|| format!("SHARD_ID: {id}"))
                .attach_printable_lazy(|| {
                    format!("SHARD_AMOUNT: {:?}", amount.get())
                })
                .attach_printable_lazy(|| {
                    format!("SHARD_TOTAL: {:?}", total.get())
                })
                .change_context(LoadError);
        }

        if id >= total.get() {
            return Err(InvalidShardConfig::IdTooBig)
                .attach_printable_lazy(|| format!("SHARD_ID: {id}"))
                .attach_printable_lazy(|| {
                    format!("SHARD_AMOUNT: {:?}", amount.get())
                })
                .attach_printable_lazy(|| {
                    format!("SHARD_TOTAL: {:?}", total.get())
                })
                .change_context(LoadError);
        }

        let tip_id = id + amount.get() - 1;
        if tip_id >= total.get() {
            return Err(Report::new(InvalidShardConfig::AmountTooMany))
                .attach_printable_lazy(|| format!("SHARD_ID: {id}"))
                .attach_printable_lazy(|| {
                    format!("SHARD_AMOUNT: {:?}", amount.get())
                })
                .attach_printable_lazy(|| {
                    format!("SHARD_TOTAL: {:?}", total.get())
                })
                .change_context(LoadError);
        }

        Ok(Self::Manual { id, amount, total })
    }
}

#[derive(Debug)]
pub struct Shard {
    bot: super::Bot,
    connect_amount: ShardConnectAmount,
    gateway_queue_url: Option<String>,
}

const RECOMMENDED_SUGGESTION: &str = concat!(
    "Suggestion: If you want to use Discord's recommended amount of ",
    "shards to connect, please set `SHARD_USE_RECOMMENDED` to true"
);

#[derive(Debug, Error)]
enum InvalidShardConfig {
    #[error("\"SHARD_AMOUNT\" must not be greater than \"SHARD_TOTAL\"")]
    AmountGtTotal,
    #[error(
        "\"SHARD_ID\" must not be greater than or equal to \"SHARD_TOTAL\""
    )]
    IdTooBig,
    #[error(
        "\"SHARD_AMOUNT\" cannot be fit with \"SHARD_TOTAL\" frm \"SHARD_ID\""
    )]
    AmountTooMany,
    #[error(
        "\"GATEWAY_QUEUE_URL\" must be in valid URL form or in HTTP/HTTPS"
    )]
    InvalidQueuerUrl,
}

impl Shard {
    #[track_caller]
    pub fn from_env() -> Result<Self, LoadError> {
        let queuer_url = if let Some(url) =
            env::var("GATEWAY_QUEUE_URL").change_context(LoadError)?
        {
            let Ok(mut parsed) = url::Url::parse(&url) else {
                return Err(InvalidShardConfig::InvalidQueuerUrl)
                    .change_context(LoadError);
            };

            parsed.set_path("");
            if !matches!(parsed.scheme(), "http" | "https") {
                return Err(InvalidShardConfig::InvalidQueuerUrl)
                    .change_context(LoadError);
            }

            Some(parsed.to_string())
        } else {
            None
        };

        Ok(Self {
            bot: super::Bot::from_env()?,
            connect_amount: ShardConnectAmount::from_env()?,
            gateway_queue_url: queuer_url,
        })
    }
}

impl Shard {
    #[must_use]
    pub const fn bot(&self) -> &super::Bot {
        &self.bot
    }

    #[must_use]
    pub const fn connect_amount(&self) -> &ShardConnectAmount {
        &self.connect_amount
    }

    #[must_use]
    pub fn gateway_queue_url(&self) -> Option<&str> {
        self.gateway_queue_url.as_deref()
    }
}
