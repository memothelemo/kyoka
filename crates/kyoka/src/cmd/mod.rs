use error_stack::{Result, ResultExt};
use twilight_interactions::command::{CommandModel, CreateCommand};
use twilight_model::{
    application::interaction::Interaction, http::interaction::*,
};

#[derive(Debug, CommandModel, CreateCommand)]
#[command(name = "ping", desc = "Responds back with pong")]
pub struct Ping;

#[derive(Debug, CommandModel, CreateCommand)]
#[command(
    name = "join",
    desc = "Connects the bot to the voice channel you've joined"
)]
pub struct Join;
