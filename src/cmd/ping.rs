use super::prelude::*;

#[derive(Debug, CommandModel, CreateCommand)]
#[command(name = "ping", desc = "Responds back with pong")]
pub struct Ping;

#[async_trait]
impl Runner for Ping {
    #[tracing::instrument(skip(interaction))]
    async fn run(
        &self,
        state: &State,
        interaction: &Interaction,
    ) -> Result<(), RunError> {
        let client = state.interaction();
        let data =
            InteractionResponseDataBuilder::new().content("Pong!").build();

        let response = InteractionResponse {
            kind: InteractionResponseType::ChannelMessageWithSource,
            data: Some(data),
        };

        client
            .create_response(interaction.id, &interaction.token, &response)
            .await
            .change_context(RunError)?;

        Ok(())
    }
}
