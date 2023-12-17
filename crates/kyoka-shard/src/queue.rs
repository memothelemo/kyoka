use error_stack::{Result, ResultExt};
use std::{fmt::Debug, sync::Arc};
use tokio::sync::Mutex;
use twilight_gateway_queue::{LocalQueue, Queue};

use crate::SetupError;

#[derive(Debug, Clone)]
pub struct BotQueue {
    client: reqwest::Client,
    queue_url: String,
    // Fallback queue if things fall apart
    local: Arc<Mutex<Option<LocalQueue>>>,
}

impl BotQueue {
    pub async fn new(queue_url: &str) -> Result<Self, SetupError> {
        // Test the service first before we actually connect
        // all shards in a single process otherwise we're wasting
        // the identify cap from Discord
        let queue = Self {
            client: reqwest::Client::builder()
                .use_rustls_tls()
                .build()
                .expect("Failed to configure reqwest client"),
            queue_url: queue_url.to_string(),
            local: Arc::new(Mutex::new(None)),
        };

        queue.request_for_shard(0).await.change_context(SetupError)?;
        Ok(queue)
    }

    async fn request_for_shard(&self, id: u64) -> reqwest::Result<()> {
        let local = self.local.lock().await;
        if let Some(local) = local.as_ref() {
            local.request([id, 1]).await;
        } else {
            self.client
                .post(format!("{}queue?shard={id}", self.queue_url))
                .send()
                .await?;
        }

        Ok(())
    }
}

impl Queue for BotQueue {
    fn request<'a>(
        &'a self,
        [id, ..]: [u64; 2],
    ) -> std::pin::Pin<Box<dyn futures::Future<Output = ()> + Send + 'a>> {
        Box::pin(async move {
            tracing::info!("waiting for allowance on shard {id}");
            if let Err(error) = self.request_for_shard(id).await {
                tracing::error!(
                    ?error,
                    "Failed to request for shard from a server queue"
                );
                tracing::info!("Falling back to local queue");

                let mut local = self.local.lock().await;
                *local = Some(LocalQueue::new());
                drop(local);

                self.request_for_shard(id)
                    .await
                    .expect("Should not crash with local queue");
            }
        })
    }
}
