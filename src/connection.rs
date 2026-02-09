use crate::error::Result;
use crate::models::WsResponse;
use dashmap::DashMap;
use std::sync::Arc;
use tokio::sync::mpsc;
use tracing::{info, warn};

pub type ClientSender = mpsc::UnboundedSender<WsResponse>;

/// Manages active WebSocket connections in the current region
#[derive(Clone)]
pub struct ConnectionManager {
    connections: Arc<DashMap<String, ClientSender>>,
}

impl ConnectionManager {
    pub fn new() -> Self {
        Self {
            connections: Arc::new(DashMap::new()),
        }
    }

    /// Register a new client connection
    pub fn add_connection(&self, user_id: String, sender: ClientSender) -> Result<()> {
        self.connections.insert(user_id.clone(), sender);
        info!("Added connection for user: {}", user_id);
        Ok(())
    }

    /// Remove a client connection
    pub fn remove_connection(&self, user_id: &str) -> Result<()> {
        self.connections.remove(user_id);
        info!("Removed connection for user: {}", user_id);
        Ok(())
    }

    /// Send a message to a locally connected client
    pub fn send_to_client(&self, user_id: &str, message: WsResponse) -> Result<bool> {
        if let Some(sender) = self.connections.get(user_id) {
            match sender.send(message) {
                Ok(_) => {
                    info!("Sent message to user: {}", user_id);
                    Ok(true)
                }
                Err(e) => {
                    warn!("Failed to send message to user {}: {}", user_id, e);
                    // Connection likely closed, remove it
                    self.connections.remove(user_id);
                    Ok(false)
                }
            }
        } else {
            warn!("User {} not found in local connections", user_id);
            Ok(false)
        }
    }

    /// Check if a user is connected locally
    pub fn is_connected(&self, user_id: &str) -> bool {
        self.connections.contains_key(user_id)
    }

    /// Get total number of active connections
    pub fn connection_count(&self) -> usize {
        self.connections.len()
    }

    /// Broadcast a message to all connected clients
    pub fn broadcast(&self, message: WsResponse) {
        let mut failed_users = Vec::new();

        for entry in self.connections.iter() {
            let user_id = entry.key();
            let sender = entry.value();

            if let Err(e) = sender.send(message.clone()) {
                warn!("Failed to broadcast to user {}: {}", user_id, e);
                failed_users.push(user_id.clone());
            }
        }

        // Clean up failed connections
        for user_id in failed_users {
            self.connections.remove(&user_id);
        }
    }

    /// Get list of all connected user IDs
    pub fn get_connected_users(&self) -> Vec<String> {
        self.connections.iter().map(|e| e.key().clone()).collect()
    }
}

impl Default for ConnectionManager {
    fn default() -> Self {
        Self::new()
    }
}
