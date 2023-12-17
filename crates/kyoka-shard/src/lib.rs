mod app;

pub mod bot;
pub mod config;
pub mod metrics;
pub mod util;

pub use app::App;

use thiserror::Error;

#[derive(Debug, Error)]
#[error("Failed to setup Kyoka shard instance")]
pub struct SetupError;
