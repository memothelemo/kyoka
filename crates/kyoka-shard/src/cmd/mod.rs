mod ping;

use async_trait::async_trait;
use error_stack::Result;
use thiserror::Error;
use twilight_interactions::command::{CommandModel, CreateCommand};
use twilight_model::application::interaction::Interaction;

use crate::state::State;

#[derive(Debug, Error)]
#[error("Failed to run command")]
pub struct RunError;

#[async_trait]
pub trait Runner: CreateCommand + CommandModel {
    async fn run(
        &self,
        state: &State,
        interaction: &Interaction,
    ) -> Result<(), RunError>;
}

mod prelude {
    #[allow(unused)]
    pub(crate) use kyoka::perform_request;

    pub use super::{RunError, Runner};
    pub use crate::state::State;

    pub use async_trait::async_trait;
    pub use error_stack::{Result, ResultExt};
    pub use kyoka::cmd;
    pub use twilight_interactions::command::{CommandModel, CreateCommand};
    pub use twilight_model::{
        application::interaction::Interaction, http::interaction::*,
    };
    pub use twilight_util::builder::InteractionResponseDataBuilder;
}
