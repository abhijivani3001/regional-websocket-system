use crate::error::Result;
use std::collections::HashSet;
use std::sync::Arc;
use tokio::sync::RwLock;
use tracing::{info, warn};

/// Simple authentication service
/// In production, this would integrate with OAuth, JWT, or other auth providers
#[derive(Clone)]
pub struct AuthService {
    // In-memory valid tokens (in production, use Redis or a database)
    valid_tokens: Arc<RwLock<HashSet<String>>>,
}

impl AuthService {
    pub fn new() -> Self {
        Self {
            valid_tokens: Arc::new(RwLock::new(HashSet::new())),
        }
    }

    /// Validate a user's authentication token
    pub async fn authenticate(&self, user_id: &str, token: &str) -> Result<bool> {
        let tokens = self.valid_tokens.read().await;

        // For demo purposes, accept any non-empty token or check the valid set
        if token.is_empty() {
            warn!("Empty token provided for user: {}", user_id);
            return Ok(false);
        }

        // Simple validation: token must be at least 10 characters
        // OR must be in the valid tokens set
        if token.len() >= 10 || tokens.contains(token) {
            info!("User {} authenticated successfully", user_id);
            Ok(true)
        } else {
            warn!("Invalid token for user: {}", user_id);
            Ok(false)
        }
    }

    /// Add a valid token (for testing/demo purposes)
    pub async fn add_valid_token(&self, token: String) {
        let mut tokens = self.valid_tokens.write().await;
        tokens.insert(token);
    }

    /// Remove a token (logout)
    pub async fn revoke_token(&self, token: &str) {
        let mut tokens = self.valid_tokens.write().await;
        tokens.remove(token);
    }

    /// Generate a simple demo token for a user
    pub fn generate_demo_token(user_id: &str) -> String {
        use uuid::Uuid;
        format!("{}_{}", user_id, Uuid::new_v4())
    }
}

impl Default for AuthService {
    fn default() -> Self {
        Self::new()
    }
}
