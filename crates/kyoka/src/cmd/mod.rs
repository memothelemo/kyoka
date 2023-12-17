mod ping;
pub use ping::*;

mod prelude {
    #[allow(unused)]
    pub(crate) use crate::perform_request;

    pub use error_stack::{Result, ResultExt};
    pub use twilight_interactions::command::{CommandModel, CreateCommand};
    pub use twilight_model::{
        application::interaction::Interaction, http::interaction::*,
    };
}
