mod state;

pub mod cmd;
pub mod config;
pub mod setup;
pub mod util;

pub use config::Config;
pub use state::State;

use thiserror::Error;

#[derive(Debug, Error)]
#[error("Failed to setup Kyoka bot")]
pub struct SetupError;
