use error_stack::{Report, Result, ResultExt};
use sentry::{types::Dsn, IntoDsn};
use thiserror::Error;

use crate::util::{env, Sensitive};

#[derive(Debug)]
pub struct Sentry {
    dsn: Option<Sensitive<Dsn>>,
    environment: Option<String>,
    traces_sample_rate: f32,
}

#[derive(Debug, Error)]
pub enum SentryError {
    #[error("Failed to load Sentry config")]
    General,
    #[error("\"SENTRY_DSN\" must be a valid Sentry DSN value")]
    InvalidDsn,
    #[error("\"SENTRY_TRACES_SAMPLE_RATE\" must be a valid floating point")]
    InvalidSampleTrace,
    #[error(
        "\"SENTRY_ENV\" is required for enabling Sentry with \"SENTRY_DSN\""
    )]
    EnvRequired,
}

impl Sentry {
    #[track_caller]
    pub fn from_env() -> Result<Self, SentryError> {
        let dsn = env::var("SENTRY_DSN")
            .change_context(SentryError::General)?
            .into_dsn()
            .change_context(SentryError::InvalidDsn)?;

        let environment = match dsn {
            None => None,
            Some(_) => Some(
                env::var("SENTRY_ENV")
                    .change_context(SentryError::General)?
                    .ok_or_else(|| Report::new(SentryError::EnvRequired))?,
            ),
        };

        Ok(Self {
            dsn: dsn.map(Sensitive::new),
            environment,
            traces_sample_rate: env::var_parse("SENTRY_TRACES_SAMPLE_RATE")
                .change_context(SentryError::InvalidSampleTrace)?
                .unwrap_or(0.0),
        })
    }
}

impl Sentry {
    #[must_use]
    pub fn dsn(&self) -> Option<Dsn> {
        self.dsn.as_ref().map(|v| v.as_ref().clone())
    }

    #[must_use]
    pub fn environment(&self) -> Option<&str> {
        self.environment.as_deref()
    }

    #[must_use]
    pub fn traces_sample_rate(&self) -> f32 {
        self.traces_sample_rate
    }
}
