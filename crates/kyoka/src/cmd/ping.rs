use super::prelude::*;

#[derive(Debug, CommandModel, CreateCommand)]
#[command(name = "ping", desc = "Responds back with pong")]
pub struct Ping;
