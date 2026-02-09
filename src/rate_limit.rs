use crate::config::RateLimitConfig;
use crate::error::{AppError, Result};
use dashmap::DashMap;
use governor::{Quota, RateLimiter as GovernorRateLimiter};
use std::num::NonZeroU32;
use std::sync::Arc;
use tracing::warn;

/// Rate limiter for protecting against abuse
#[derive(Clone)]
pub struct RateLimiter {
    // Per-user rate limiters
    limiters: Arc<
        DashMap<
            String,
            Arc<
                GovernorRateLimiter<
                    governor::state::direct::NotKeyed,
                    governor::state::InMemoryState,
                    governor::clock::DefaultClock,
                >,
            >,
        >,
    >,
    quota: Quota,
}

impl RateLimiter {
    pub fn new(config: &RateLimitConfig) -> Self {
        let quota = Quota::per_second(NonZeroU32::new(config.messages_per_second).unwrap())
            .allow_burst(NonZeroU32::new(config.burst_size).unwrap());

        Self {
            limiters: Arc::new(DashMap::new()),
            quota,
        }
    }

    /// Check if a user can send a message (rate limit check)
    pub fn check_rate_limit(&self, user_id: &str) -> Result<()> {
        let limiter = self
            .limiters
            .entry(user_id.to_string())
            .or_insert_with(|| Arc::new(GovernorRateLimiter::direct(self.quota)))
            .clone();

        match limiter.check() {
            Ok(_) => Ok(()),
            Err(_) => {
                warn!("Rate limit exceeded for user: {}", user_id);
                Err(AppError::RateLimitExceeded)
            }
        }
    }

    /// Clean up rate limiters for disconnected users
    pub fn cleanup_user(&self, user_id: &str) {
        self.limiters.remove(user_id);
    }

    /// Get current limiter count (for monitoring)
    pub fn active_limiters(&self) -> usize {
        self.limiters.len()
    }
}
