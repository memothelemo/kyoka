pub mod config;
pub mod server;

use thiserror::Error;

#[derive(Debug, Error)]
#[error("Failed to setup Kyoka gateway queue server")]
pub struct SetupError;
