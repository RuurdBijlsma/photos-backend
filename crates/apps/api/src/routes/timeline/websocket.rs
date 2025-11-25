use std::sync::Arc;
use crate::api_state::ApiContext;
use axum::extract::ws::{Message, Utf8Bytes, WebSocket};
use color_eyre::Result;
use serde_json::Value;
use common_services::database::app_user::User;
use sqlx::postgres::PgListener;
use sqlx::PgPool;
use tokio::select;
use tokio::sync::broadcast;
use tracing::{error, info, warn};
use serde::Deserialize;

#[derive(Clone, Debug, Deserialize)]
pub struct MediaPayload {
    pub user_id: i32,
    #[serde(skip)]
    pub raw_json: String,
}

pub async fn handle_timeline_socket(mut socket: WebSocket, context: ApiContext, user: User) {
    info!("WS connected for user: {} ({})", user.name, user.email);

    let mut rx = context.timeline_broadcaster.subscribe();

    loop {
        select! {
            // 1. Handle incoming broadcast messages from Postgres/Server
            result = rx.recv() => {
                match result {
                    Ok(msg) => {
                        // Parse JSON to check ownership
                        let is_owner = serde_json::from_str::<Value>(&msg)
                            .ok()
                            .and_then(|v| v.get("user_id").and_then(|id| id.as_i64()))
                            .map(|owner_id| owner_id as i32 == user.id)
                            .unwrap_or(false);

                        if is_owner {
                            if let Err(e) = socket.send(Message::Text(msg.into())).await {
                                // Client disconnected during send
                                warn!("Client disconnected (send error): {}", e);
                                break;
                            }
                        }
                    }
                    Err(tokio::sync::broadcast::error::RecvError::Lagged(_)) => {
                        // Creating a warning here might be noisy, usually safe to ignore or log debug
                    }
                    Err(tokio::sync::broadcast::error::RecvError::Closed) => {
                        // Broadcaster itself closed
                        break;
                    }
                }
            }

            // 2. Handle incoming messages/disconnects from the Client
            client_msg = socket.recv() => {
                match client_msg {
                    // Client sent a close frame OR the stream ended (None)
                    Some(Ok(Message::Close(_))) | None => {
                        info!("Client disconnected (socket closed)");
                        break;
                    }
                    // Error reading from socket
                    Some(Err(e)) => {
                        warn!("Client error: {}", e);
                        break;
                    }
                    // Client sent text/binary/ping - we just ignore it to keep connection alive
                    _ => {}
                }
            }
        }
    }
}

pub fn create_media_item_transmitter(pool: &PgPool) -> Result<broadcast::Sender<Arc<MediaPayload>>> {
    let (tx, _rx) = broadcast::channel(100);

    // 3. Spawn the Postgres Listener Task
    let listener_pool = pool.clone();
    let listener_tx = tx.clone();

    tokio::spawn(async move {
        info!("🔌 Connecting to Postgres notification stream...");
        let mut listener = match PgListener::connect_with(&listener_pool).await {
            Ok(l) => l,
            Err(e) => {
                error!("Failed to connect PgListener: {}", e);
                return;
            }
        };

        if let Err(e) = listener.listen("media_item_added").await {
            error!("Failed to listen to channel 'media_item_added': {}", e);
            return;
        }

        info!("📡 Listening for new media items...");
        loop {
            match listener.recv().await {
                Ok(notification) => {
                    let payload_str = notification.payload().to_string();

                    // Optimization: Parse minimal info ONCE here
                    // We interpret the payload as a generic Value first to extract user_id safely
                    if let Ok(val) = serde_json::from_str::<Value>(&payload_str) {
                        if let Some(user_id) = val.get("user_id").and_then(|v| v.as_i64()) {

                            let event = Arc::new(MediaPayload {
                                user_id: user_id as i32,
                                raw_json: payload_str, // Pass the original string along
                            });

                            // Broadcast the Arc (very cheap, just a pointer copy)
                            if let Err(e) = listener_tx.send(event) {
                                warn!("No active listeners for media update: {}", e);
                            }
                        }
                    }
                }
                Err(e) => { /* ... */ }
            }
        }
    });

    Ok(tx)
}
