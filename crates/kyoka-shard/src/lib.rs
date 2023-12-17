mod app;

pub mod bot;
pub mod config;
pub mod metrics;
pub mod queue;
pub mod util;

pub use app::App;
pub use queue::BotQueue;

use thiserror::Error;

#[derive(Debug, Error)]
#[error("Failed to setup Kyoka shard instance")]
pub struct SetupError;
