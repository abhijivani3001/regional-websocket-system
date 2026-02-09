use actix_ws::{Message as ActixWsMessage, MessageStream, Session};
use futures_util::StreamExt;
use std::sync::Arc;
use std::time::{Duration, Instant};
use tokio::sync::mpsc;
use tokio::time::interval;
use tracing::{error, info, warn};

use crate::auth::AuthService;
use crate::config::Config;
use crate::connection::ConnectionManager;
use crate::error::{AppError, Result};
use crate::messaging::MessageBroker;
use crate::models::{
    InterRegionMessage, Message, ServerMessage, WsMessage as AppWsMessage, WsResponse,
};
use crate::presence::PresenceManager;
use crate::rate_limit::RateLimiter;

pub struct WebSocketHandler {
    pub config: Arc<Config>,
    pub auth: Arc<AuthService>,
    pub connections: Arc<ConnectionManager>,
    pub presence: Arc<PresenceManager>,
    pub broker: Arc<MessageBroker>,
    pub rate_limiter: Arc<RateLimiter>,
}

impl WebSocketHandler {
    pub fn new(
        config: Arc<Config>,
        auth: Arc<AuthService>,
        connections: Arc<ConnectionManager>,
        presence: Arc<PresenceManager>,
        broker: Arc<MessageBroker>,
        rate_limiter: Arc<RateLimiter>,
    ) -> Self {
        Self {
            config,
            auth,
            connections,
            presence,
            broker,
            rate_limiter,
        }
    }

    /// Handle a new WebSocket connection
    pub async fn handle_connection(
        self: Arc<Self>,
        mut session: Session,
        mut msg_stream: MessageStream,
    ) {
        let (tx, mut rx) = mpsc::unbounded_channel::<WsResponse>();

        let mut user_id: Option<String> = None;
        let mut authenticated = false;
        let mut last_heartbeat = Instant::now();
        let mut heartbeat_interval = interval(Duration::from_secs(30));

        // Spawn task to send messages to client
        let mut session_clone = session.clone();
        let send_task = actix_rt::spawn(async move {
            while let Some(response) = rx.recv().await {
                if let Ok(msg) = serde_json::to_string(&response) {
                    if session_clone.text(msg).await.is_err() {
                        break;
                    }
                }
            }
        });

        loop {
            tokio::select! {
                // Handle incoming messages
                Some(Ok(msg)) = msg_stream.next() => {
                    match msg {
                        ActixWsMessage::Text(text) => {
                            last_heartbeat = Instant::now();

                            let text_str = text.to_string();
                            match serde_json::from_str::<AppWsMessage>(&text_str) {
                                Ok(app_msg) => {
                                    match app_msg {
                                        AppWsMessage::Auth {
                                            user_id: uid,
                                            token,
                                        } => {
                                            match self.handle_auth(&uid, &token, tx.clone()).await {
                                                Ok(true) => {
                                                    user_id = Some(uid);
                                                    authenticated = true;
                                                }
                                                Ok(false) => {
                                                    // Auth failed, close connection
                                                    let _ = session.close(None).await;
                                                    break;
                                                }
                                                Err(e) => {
                                                    error!("Auth error: {}", e);
                                                    let _ = session.close(None).await;
                                                    break;
                                                }
                                            }
                                        }
                                        AppWsMessage::Message(client_msg) => {
                                            if !authenticated {
                                                let _ = tx.send(WsResponse::Error {
                                                    message: "Not authenticated".to_string(),
                                                });
                                                continue;
                                            }

                                            if let Some(ref uid) = user_id {
                                                if let Err(e) = self.handle_message(uid, client_msg).await {
                                                    error!("Message handling error: {}", e);
                                                    let _ = tx.send(WsResponse::Error {
                                                        message: format!("Failed to send message: {}", e),
                                                    });
                                                }
                                            }
                                        }
                                        AppWsMessage::Ping => {
                                            let _ = tx.send(WsResponse::Pong);

                                            // Update presence on ping
                                            if let Some(ref uid) = user_id {
                                                let _ = self.presence.update_presence(uid).await;
                                            }
                                        }
                                        AppWsMessage::Pong => {
                                            // Handle pong
                                        }
                                    }
                                }
                                Err(e) => {
                                    warn!("Failed to parse message: {}", e);
                                    let _ = tx.send(WsResponse::Error {
                                        message: "Invalid message format".to_string(),
                                    });
                                }
                            }
                        }
                        ActixWsMessage::Ping(bytes) => {
                            last_heartbeat = Instant::now();
                            let _ = session.pong(&bytes).await;
                        }
                        ActixWsMessage::Pong(_) => {
                            last_heartbeat = Instant::now();
                        }
                        ActixWsMessage::Close(reason) => {
                            info!("WebSocket connection closed: {:?}", reason);
                            let _ = session.close(reason).await;
                            break;
                        }
                        _ => {}
                    }
                }

                // Heartbeat check
                _ = heartbeat_interval.tick() => {
                    if Instant::now().duration_since(last_heartbeat) > Duration::from_secs(60) {
                        warn!("Client heartbeat timeout");
                        let _ = session.close(None).await;
                        break;
                    }

                    // Send ping to client
                    if session.ping(b"").await.is_err() {
                        break;
                    }
                }

                // Check if stream ended
                else => {
                    break;
                }
            }
        }

        // Cleanup on disconnect
        if let Some(uid) = user_id {
            info!("Cleaning up connection for user: {}", uid);
            let _ = self.connections.remove_connection(&uid);
            let _ = self.presence.unregister_user(&uid).await;
            self.rate_limiter.cleanup_user(&uid);
        }

        send_task.abort();
    }

    async fn handle_auth(
        &self,
        user_id: &str,
        token: &str,
        tx: mpsc::UnboundedSender<WsResponse>,
    ) -> Result<bool> {
        if self.auth.authenticate(user_id, token).await? {
            // Register connection
            self.connections
                .add_connection(user_id.to_string(), tx.clone())?;

            // Register presence
            self.presence.register_user(user_id).await?;

            // Send auth success
            tx.send(WsResponse::AuthSuccess {
                user_id: user_id.to_string(),
            })
            .map_err(|e| AppError::Internal(format!("Failed to send auth response: {}", e)))?;

            info!("User {} authenticated and connected", user_id);
            Ok(true)
        } else {
            tx.send(WsResponse::AuthFailure {
                reason: "Invalid credentials".to_string(),
            })
            .map_err(|e| AppError::Internal(format!("Failed to send auth response: {}", e)))?;

            warn!("Authentication failed for user: {}", user_id);
            Ok(false)
        }
    }

    async fn handle_message(
        &self,
        from_user: &str,
        client_msg: crate::models::ClientMessage,
    ) -> Result<()> {
        // Rate limiting
        self.rate_limiter.check_rate_limit(from_user)?;

        let message = Message::new(
            from_user.to_string(),
            client_msg.to.clone(),
            client_msg.content.clone(),
            client_msg.message_type.clone(),
        );

        info!(
            "Processing message {} from {} to {}",
            message.id, message.from, message.to
        );

        // Check if recipient is in local region
        let is_local = self.presence.is_user_local(&message.to).await?;

        if is_local {
            // Deliver locally
            self.deliver_local_message(&message)?;
        } else {
            // Route to another region
            self.route_to_region(&message).await?;
        }

        Ok(())
    }

    fn deliver_local_message(&self, message: &Message) -> Result<()> {
        let server_msg = ServerMessage {
            id: message.id.clone(),
            from: message.from.clone(),
            content: message.content.clone(),
            timestamp: message.timestamp,
        };

        let delivered = self
            .connections
            .send_to_client(&message.to, WsResponse::Message(server_msg))?;

        if delivered {
            info!("Message {} delivered locally to {}", message.id, message.to);
        } else {
            warn!("Failed to deliver message {} to {}", message.id, message.to);
        }

        Ok(())
    }

    async fn route_to_region(&self, message: &Message) -> Result<()> {
        // Get target region
        let target_region = self.presence.get_user_region(&message.to).await?;

        if let Some(region) = target_region {
            let inter_msg = InterRegionMessage {
                message: message.clone(),
                source_region: self.config.region.name.clone(),
                target_region: region.clone(),
            };

            self.broker.publish_to_region(inter_msg).await?;
            info!(
                "Message {} routed to region {} for user {}",
                message.id, region, message.to
            );
        } else {
            warn!(
                "User {} not found in any region, cannot deliver message {}",
                message.to, message.id
            );
            return Err(AppError::UserNotFound(message.to.clone()));
        }

        Ok(())
    }

    /// Start listening for inter-region messages
    pub async fn start_inter_region_listener(self: Arc<Self>) {
        let mut rx = self.broker.subscribe_local();

        tokio::spawn(async move {
            while let Ok(inter_msg) = rx.recv().await {
                info!(
                    "Received inter-region message {} for user {}",
                    inter_msg.message.id, inter_msg.message.to
                );

                if let Err(e) = self.deliver_local_message(&inter_msg.message) {
                    error!("Failed to deliver inter-region message: {}", e);
                }
            }
        });

        info!("Inter-region message listener started");
    }
}
