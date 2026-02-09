use crate::config::Config;
use crate::error::{AppError, Result};
use redis::AsyncCommands;
use std::sync::Arc;
use tracing::{info, warn};

/// Manages user presence across regions
#[derive(Clone)]
pub struct PresenceManager {
    redis_client: redis::Client,
    config: Arc<Config>,
}

impl PresenceManager {
    pub async fn new(config: Arc<Config>) -> Result<Self> {
        let redis_client = redis::Client::open(config.redis.url.as_str())
            .map_err(|e| AppError::Internal(format!("Failed to create Redis client: {}", e)))?;

        // Test connection
        let mut conn = redis_client.get_multiplexed_async_connection().await?;
        redis::cmd("PING")
            .query_async::<_, String>(&mut conn)
            .await?;

        info!("Presence manager initialized with Redis");

        Ok(Self {
            redis_client,
            config,
        })
    }

    /// Register a user's presence in the current region
    pub async fn register_user(&self, user_id: &str) -> Result<()> {
        let mut conn = self.redis_client.get_multiplexed_async_connection().await?;
        let key = format!("presence:{}", user_id);
        let ttl = self.config.redis.presence_ttl;

        let _: () = conn.set_ex(&key, &self.config.region.name, ttl).await?;

        info!(
            "Registered user {} in region {}",
            user_id, self.config.region.name
        );
        Ok(())
    }

    /// Update user's presence TTL (heartbeat)
    pub async fn update_presence(&self, user_id: &str) -> Result<()> {
        let mut conn = self.redis_client.get_multiplexed_async_connection().await?;
        let key = format!("presence:{}", user_id);
        let ttl = self.config.redis.presence_ttl;

        let _: () = conn.expire(&key, ttl as i64).await?;
        Ok(())
    }

    /// Remove user's presence
    pub async fn unregister_user(&self, user_id: &str) -> Result<()> {
        let mut conn = self.redis_client.get_multiplexed_async_connection().await?;
        let key = format!("presence:{}", user_id);

        let _: () = conn.del(&key).await?;

        info!(
            "Unregistered user {} from region {}",
            user_id, self.config.region.name
        );
        Ok(())
    }

    /// Get the region where a user is connected
    pub async fn get_user_region(&self, user_id: &str) -> Result<Option<String>> {
        let mut conn = self.redis_client.get_multiplexed_async_connection().await?;
        let key = format!("presence:{}", user_id);

        let region: Option<String> = conn.get(&key).await?;

        if let Some(ref r) = region {
            info!("User {} found in region {}", user_id, r);
        } else {
            warn!("User {} not found in any region", user_id);
        }

        Ok(region)
    }

    /// Check if user is in the local region
    pub async fn is_user_local(&self, user_id: &str) -> Result<bool> {
        let region = self.get_user_region(user_id).await?;
        Ok(region.as_ref() == Some(&self.config.region.name))
    }
}
