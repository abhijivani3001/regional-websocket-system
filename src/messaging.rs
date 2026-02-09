use crate::config::Config;
use crate::error::{AppError, Result};
use crate::models::InterRegionMessage;
use std::sync::Arc;
use tokio::sync::broadcast;
use tracing::{error, info, warn};

/// Handles inter-region message routing via Redis Pub/Sub
#[derive(Clone)]
pub struct MessageBroker {
    redis_client: redis::Client,
    config: Arc<Config>,
    local_tx: broadcast::Sender<InterRegionMessage>,
}

impl MessageBroker {
    pub async fn new(config: Arc<Config>) -> Result<Self> {
        let redis_client = redis::Client::open(config.redis.url.as_str())
            .map_err(|e| AppError::Internal(format!("Failed to create Redis client: {}", e)))?;

        // Create broadcast channel for local message distribution
        let (local_tx, _) = broadcast::channel(1024);

        info!("Message broker initialized");

        Ok(Self {
            redis_client,
            config,
            local_tx,
        })
    }

    /// Publish a message to a specific region
    pub async fn publish_to_region(&self, message: InterRegionMessage) -> Result<()> {
        let mut conn = self.redis_client.get_multiplexed_async_connection().await?;
        let channel = self.get_region_channel(&message.target_region);
        let payload = serde_json::to_string(&message)?;

        redis::cmd("PUBLISH")
            .arg(&channel)
            .arg(&payload)
            .query_async::<_, i32>(&mut conn)
            .await?;

        info!(
            "Published message {} to region {} via channel {}",
            message.message.id, message.target_region, channel
        );

        Ok(())
    }

    /// Subscribe to messages for the current region
    pub async fn subscribe_to_region(&self) -> Result<()> {
        let client = self.redis_client.clone();
        let channel = self.get_region_channel(&self.config.region.name);
        let local_tx = self.local_tx.clone();
        let channel_log = channel.clone();

        tokio::spawn(async move {
            loop {
                match Self::run_subscriber(client.clone(), channel.clone(), local_tx.clone()).await
                {
                    Ok(_) => {
                        warn!("Subscriber task ended, restarting...");
                    }
                    Err(e) => {
                        error!("Subscriber error: {}, reconnecting in 5s...", e);
                        tokio::time::sleep(tokio::time::Duration::from_secs(5)).await;
                    }
                }
            }
        });

        info!("Subscribed to region channel: {}", channel_log);
        Ok(())
    }

    async fn run_subscriber(
        client: redis::Client,
        channel: String,
        local_tx: broadcast::Sender<InterRegionMessage>,
    ) -> Result<()> {
        let conn = client.get_async_connection().await?;
        let mut pubsub = conn.into_pubsub();
        pubsub.subscribe(&channel).await?;

        info!("Active subscription on channel: {}", channel);

        let mut stream = pubsub.on_message();

        while let Some(msg) = stream.next().await {
            let payload: String = msg.get_payload()?;

            match serde_json::from_str::<InterRegionMessage>(&payload) {
                Ok(inter_msg) => {
                    info!(
                        "Received inter-region message {} from region {}",
                        inter_msg.message.id, inter_msg.source_region
                    );

                    if let Err(e) = local_tx.send(inter_msg) {
                        warn!("Failed to broadcast message locally: {}", e);
                    }
                }
                Err(e) => {
                    error!("Failed to deserialize inter-region message: {}", e);
                }
            }
        }

        Ok(())
    }

    /// Get a receiver for incoming inter-region messages
    pub fn subscribe_local(&self) -> broadcast::Receiver<InterRegionMessage> {
        self.local_tx.subscribe()
    }

    fn get_region_channel(&self, region: &str) -> String {
        format!("{}:{}", self.config.redis.channel_prefix, region)
    }
}

// Import needed for stream
use futures_util::StreamExt;
