use super::prelude::*;

#[async_trait]
impl Runner for cmd::Ping {
    #[tracing::instrument]
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
