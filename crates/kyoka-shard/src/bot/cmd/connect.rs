use super::prelude::*;

#[async_trait]
impl Runner for cmd::Join {
    async fn run(
        &self,
        state: &State,
        interaction: &Interaction,
    ) -> Result<(), RunError> {
        todo!()
    }
}
