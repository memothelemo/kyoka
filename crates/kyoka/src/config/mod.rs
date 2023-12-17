mod sentry;
pub use self::sentry::{Sentry, SentryError};

use thiserror::Error;

#[derive(Debug, Error)]
#[error("Failed to load configuration")]
pub struct LoadError;
