use error_stack::{Result, ResultExt};
use thiserror::Error;
use tokio_util::task::TaskTracker;
use tracing::Instrument;
use twilight_gateway::error::ReceiveMessageErrorType;
use twilight_gateway::{CloseFrame, Event, Message, Shard};
use twilight_interactions::command::CommandModel;
use twilight_model::application::interaction::InteractionData;
use twilight_model::application::interaction::{
    application_command::CommandData, Interaction,
};
use twilight_model::http::interaction::{
    InteractionResponse, InteractionResponseType,
};
use twilight_util::builder::InteractionResponseDataBuilder;

use super::cmd::{RunError, Runner};
use super::State;

#[derive(Debug, Error)]
#[error("Failed to process event")]
pub struct EventFailed;

#[tracing::instrument(skip_all)]
async fn command(
    state: &State,
    interaction: &Interaction,
    data: CommandData,
) -> Result<(), RunError> {
    match &*data.name {
        "ping" => kyoka::cmd::Ping::from_interaction(data.into())
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

#[tracing::instrument(skip_all, name = "event", fields(kind = ?event.kind()))]
async fn process_event(state: State, event: Event) -> Result<(), EventFailed> {
    match event {
        Event::GatewayHello(..) => {
            tracing::debug!("Received Hello event, identifying bot...");
        },
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

            if let Err(error) = command(&state, &interaction, data).await {
                tracing::error!(
                    ?error,
                    "Failed to process command interaction"
                );

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
    };
    Ok(())
}

#[tracing::instrument(skip_all, fields(id = %shard.id()))]
pub async fn shard(state: State, shard: &mut Shard) {
    let tracker = TaskTracker::new();

    loop {
        tokio::select! {
            result = shard.next_event() => {
                let event = match result {
                    Ok(event) => event,
                    Err(source) => {
                        if source.is_fatal() {
                            tracing::error!(?source, "Got fatal shard message error");
                            state.app().perform_shutdown("Fatal error given from a shard");
                            break;
                        }
                        tracing::warn!(?source, "Got shard message error");
                        continue;
                    },
                };
                state.songbird().process(&event).await;
                state.app().metrics().events_processed().add(1);

                if matches!(event, Event::GatewayHello(..) | Event::GatewayHeartbeatAck) {
                    let latency = shard.latency().average().unwrap_or_default();
                    state
                        .app()
                        .metrics()
                        .shard_latency()
                        .set(latency.as_secs_f64());
                }

                let state = state.clone();
                tracker.spawn(
                    async move {
                        if let Err(error) = process_event(state, event).await {
                            tracing::error!(?error, "Failed to process event");
                        }
                    }
                    .in_current_span(),
                );
            },
            _ = state.app().shutdown_signal() => {
                break;
            },
        }
    }

    if !shard.status().is_disconnected() {
        tracing::info!("Disconnecting shard...");

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
                    if source.is_fatal() {
                        tracing::error!(
                            ?source,
                            "Got fatal shard message error"
                        );
                    } else {
                        tracing::warn!(?source, "Got shard message error");
                    }
                },
            }
        }
    }

    if tracker.close() {
        tracing::info!("Waiting for all tasks to be completed");
        tracker.wait().await;
    }
}
