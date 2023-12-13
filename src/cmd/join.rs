use super::prelude::*;

#[derive(Debug, CommandModel, CreateCommand)]
#[command(name = "join", desc = "Attempts the bot to join on a voice channel")]
pub struct Join;

#[async_trait]
impl Runner for Join {
    #[tracing::instrument(skip(interaction))]
    async fn run(
        &self,
        state: &State,
        interaction: &Interaction,
    ) -> Result<(), RunError> {
        // Who made this command and had joined any voice channel?
        let client = state.interaction();
        let condition = interaction.author().zip(interaction.guild_id);

        let Some((author, guild_id)) = condition else {
            let data = InteractionResponseDataBuilder::new()
                .content("You can use this command only in servers!")
                .build();

            let response = InteractionResponse {
                kind: InteractionResponseType::ChannelMessageWithSource,
                data: Some(data),
            };

            client
                .create_response(interaction.id, &interaction.token, &response)
                .await
                .change_context(RunError)?;

            return Ok(());
        };

        todo!()
    }
}
