mod auth;
mod config;
mod connection;
mod error;
mod handler;
mod messaging;
mod models;
mod presence;
mod rate_limit;

use actix_web::{App, HttpRequest, HttpResponse, HttpServer, web};
use std::sync::Arc;
use tracing::info;

use crate::auth::AuthService;
use crate::config::Config;
use crate::connection::ConnectionManager;
use crate::handler::WebSocketHandler;
use crate::messaging::MessageBroker;
use crate::presence::PresenceManager;
use crate::rate_limit::RateLimiter;

#[derive(Clone)]
pub struct AppState {
    pub handler: Arc<WebSocketHandler>,
}

/// WebSocket route handler
async fn websocket_handler(
    req: HttpRequest,
    stream: web::Payload,
    data: web::Data<AppState>,
) -> Result<HttpResponse, actix_web::Error> {
    let (response, session, msg_stream) = actix_ws::handle(&req, stream)?;

    let handler = data.handler.clone();

    // Spawn task to handle this WebSocket connection
    actix_rt::spawn(async move {
        handler.handle_connection(session, msg_stream).await;
    });

    Ok(response)
}

/// Health check endpoint
async fn health_check() -> HttpResponse {
    HttpResponse::Ok().json(serde_json::json!({
        "status": "healthy",
        "service": "regional-websocket-system"
    }))
}

/// Metrics endpoint
async fn metrics_handler(data: web::Data<AppState>) -> HttpResponse {
    HttpResponse::Ok().json(serde_json::json!({
        "active_connections": data.handler.connections.connection_count(),
        "region": data.handler.config.region.name,
    }))
}

#[actix_web::main]
async fn main() -> anyhow::Result<()> {
    // Initialize tracing
    tracing_subscriber::fmt()
        .with_env_filter(
            tracing_subscriber::EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| "info,regional_websocket_system=debug".into()),
        )
        .init();

    // Load configuration
    let config = Arc::new(Config::from_env()?);
    info!("Configuration loaded for region: {}", config.region.name);

    // Initialize services
    let auth = Arc::new(AuthService::new());
    let connections = Arc::new(ConnectionManager::new());
    let presence = Arc::new(PresenceManager::new(config.clone()).await?);
    let broker = Arc::new(MessageBroker::new(config.clone()).await?);
    let rate_limiter = Arc::new(RateLimiter::new(&config.rate_limit));

    // Subscribe to inter-region messages
    broker.subscribe_to_region().await?;

    // Create WebSocket handler
    let handler = Arc::new(WebSocketHandler::new(
        config.clone(),
        auth.clone(),
        connections.clone(),
        presence.clone(),
        broker.clone(),
        rate_limiter.clone(),
    ));

    // Start inter-region message listener
    handler.clone().start_inter_region_listener().await;

    // Create app state
    let state = AppState {
        handler: handler.clone(),
    };

    let addr = format!("{}:{}", config.server.host, config.server.port);
    info!("WebSocket server listening on {}", addr);
    info!("Region: {}", config.region.name);
    info!("Connect to: ws://{}/ws", addr);

    // Start HTTP server
    HttpServer::new(move || {
        App::new()
            .app_data(web::Data::new(state.clone()))
            .wrap(tracing_actix_web::TracingLogger::default())
            .route("/ws", web::get().to(websocket_handler))
            .route("/health", web::get().to(health_check))
            .route("/metrics", web::get().to(metrics_handler))
    })
    .bind(&addr)?
    .run()
    .await?;

    Ok(())
}
