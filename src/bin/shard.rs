use error_stack::{Result, ResultExt};
use kyoka::{
    cmd::{self, RunError, Runner},
    Config, SetupError, State,
};
use thiserror::Error;
use tokio_util::{sync::CancellationToken, task::TaskTracker};
use twilight_gateway::{
    error::ReceiveMessageErrorType, CloseFrame, Event, Intents, Message, Shard,
    ShardId,
};
use twilight_interactions::command::CommandModel;
use twilight_model::{
    application::interaction::{
        application_command::CommandData, Interaction, InteractionData,
    },
    http::interaction::{InteractionResponse, InteractionResponseType},
};
use twilight_util::builder::InteractionResponseDataBuilder;

#[derive(Debug, Error)]
#[error("Failed to process event")]
struct ProcessEventFailed;

#[allow(clippy::single_match)]
async fn process_cmd_interaction(
    state: &State,
    data: CommandData,
    interaction: &Interaction,
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

#[allow(clippy::single_match)]
#[tracing::instrument]
async fn process_event(
    state: State,
    event: Event,
) -> Result<(), ProcessEventFailed> {
    match event {
        Event::Ready(info) => {
            tracing::info!(
                "Logged in as {:?} ({})",
                info.user.name,
                info.user.id
            );
        },
        Event::InteractionCreate(interaction) => {
            let mut interaction = interaction.0;
            let data = match std::mem::take(&mut interaction.data) {
                Some(InteractionData::ApplicationCommand(data)) => *data,
                _ => return Ok(()),
            };

            if let Err(error) =
                process_cmd_interaction(&state, data, &interaction).await
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
                    .change_context(ProcessEventFailed)?;
            }
        },
        _ => {},
    };
    Ok(())
}

#[tracing::instrument(skip(cancel_token, shard))]
async fn gateway_runner(
    state: State,
    cancel_token: CancellationToken,
    mut shard: Shard,
) {
    // We need to keep track all tasks (where we process events)
    // so that this thread won't accidentally cancel event threads
    // which it ruins the user experience.
    let mut force_shutdown = false;
    let tracker = TaskTracker::new();

    loop {
        tokio::select! {
            result = shard.next_event() => {
                let event = match result {
                    Ok(event) => event,
                    Err(source) => {
                        tracing::warn!(?source, "error receiving event");
                        if source.is_fatal() {
                            force_shutdown = true;
                            break;
                        }
                        continue;
                    },
                };
                let state = state.clone();
                tracker.spawn(async move {
                    if let Err(error) = process_event(state, event).await {
                        tracing::warn!(?error, "Failed to process event");
                    }
                });
            },
            _ = cancel_token.cancelled() => {
                break;
            },
        }
    }

    if force_shutdown && !cancel_token.is_cancelled() {
        tracing::warn!("Fatal error given; shutting down shard...");
        cancel_token.cancel();
    }

    if !shard.status().is_disconnected() {
        tracing::debug!("Closing WebSocket connection");
        if let Err(error) = shard.close(CloseFrame::NORMAL).await {
            tracing::error!(?error, "Failed to close shard connection");
        }

        // Wait until WebSocket connection is FINALLY CLOSED
        loop {
            match shard.next_message().await {
                Ok(Message::Close(..)) | Ok(Message::Text(..)) => break,
                Err(source)
                    if matches!(source.kind(), ReceiveMessageErrorType::Io) =>
                {
                    break;
                },
                Err(source) => {
                    tracing::warn!(?source, "error receiving message")
                },
            }
        }
    }

    if tracker.close() {
        tracing::info!("Waiting for all tasks to be completed");
        tracker.wait().await;
    }
}

#[tokio::main]
async fn main() -> Result<(), SetupError> {
    let cfg = Config::from_env().change_context(SetupError)?;
    kyoka::setup::init_logging(&cfg)?;

    let state = State::new(cfg).await.change_context(SetupError)?;
    if state.config().bot().reload_commands_on_start() {
        kyoka::setup::cmd(&state).await?;
    }

    tracing::info!("Initializing shard...");
    let shard = Shard::new(
        ShardId::ONE,
        state.config().bot().token().to_string(),
        Intents::GUILDS | Intents::GUILD_MESSAGES,
    );

    let cancel_token = CancellationToken::new();
    let handle = tokio::spawn(gateway_runner(
        state.clone(),
        cancel_token.clone(),
        shard,
    ));

    tokio::select! {
        _ = kyoka::util::shutdown_signal() => {
            tracing::info!("Shutdown signal is triggered. Shutting down shard...");
            cancel_token.cancel();
        },
        _ = cancel_token.cancelled() => {}
    }
    handle.await.expect("failed to cleanup leftover tasks");
    tracing::info!("Shard has been successfully shut down");

    Ok(())
}
