use error_stack::{Result, ResultExt};
use kyoka::cmd;
use kyoka::util::Sensitive;
use thiserror::Error;
use twilight_gateway::Event;
use twilight_interactions::command::CommandModel;
use twilight_model::{
    application::interaction::{
        application_command::CommandData, Interaction, InteractionData,
    },
    http::interaction::{InteractionResponse, InteractionResponseType},
};
use twilight_util::builder::InteractionResponseDataBuilder;

use crate::cmd::{RunError, Runner};
use crate::State;

#[derive(Debug, Error)]
#[error("Failed to process event")]
pub struct EventFailed;

#[tracing::instrument(skip(interaction), fields(
    interaction.id = %interaction.id,
    interaction.author_id = ?interaction.author_id(),
    interaction.dm = %interaction.is_dm(),
    interaction.kind = ?interaction.kind,
    interaction.locale = ?interaction.locale,
    interaction.token = %Sensitive::new(()),
))]
pub async fn command_interaction(
    state: &State,
    interaction: &Interaction,
    data: CommandData,
) -> Result<(), RunError> {
    match &*data.name {
        "ping" => cmd::Ping::from_interaction(data.into())
            .change_context(RunError)?
            .run(state, interaction)
            .await
            .change_context(RunError),
        _ => {
            tracing::warn!("Unknown command: {:?}", data.name);
            Err(RunError.into())
        },
    }
}

#[tracing::instrument]
pub async fn event(state: State, event: Event) -> Result<(), EventFailed> {
    match event {
        Event::Ready(info) => {
            tracing::info!(
                "Logged in as {:?} ({})",
                info.user.name,
                info.user.id
            );
        },
        Event::InteractionCreate(data) => {
            let mut interaction = data.0;
            let data = match std::mem::take(&mut interaction.data) {
                Some(InteractionData::ApplicationCommand(data)) => *data,
                _ => return Ok(()),
            };

            if let Err(error) =
                command_interaction(&state, &interaction, data).await
            {
                tracing::warn!(?error, "Failed to process command interaction");

                let data = InteractionResponseDataBuilder::new()
                    .content("There's something wrong with your request. Please report this to the developers immediately!")
                    .build();

                let response = InteractionResponse {
                    kind: InteractionResponseType::ChannelMessageWithSource,
                    data: Some(data),
                };

                state
                    .interaction()
                    .create_response(
                        interaction.id,
                        &interaction.token,
                        &response,
                    )
                    .await
                    .change_context(EventFailed)?;
            }
        },
        _ => {},
    }
    Ok(())
}
