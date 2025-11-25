use std::sync::Arc;
use crate::api_state::ApiContext;
use axum::extract::ws::{Message, WebSocket};
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
            // 1. Handle Broadcast Messages (From Postgres/Server)
            result = rx.recv() => {
                match result {
                    Ok(payload) => {
                        // 'payload' is now Arc<MediaPayload>
                        // No JSON parsing needed here! Just check the Integer ID.
                        if payload.user_id == user.id {
                            // We clone the inner String (raw_json).
                            // This is much cheaper than parsing/serializing again.
                            if let Err(e) = socket.send(Message::Text(payload.raw_json.clone().into())).await {
                                warn!("Client disconnected during send: {}", e);
                                break;
                            }
                        }
                    }
                    Err(broadcast::error::RecvError::Lagged(skipped)) => {
                         // This happens if the server generates events faster than
                         // this specific client can read them.
                         warn!("User {} lagged, skipped {} messages", user.id, skipped);
                    }
                    Err(broadcast::error::RecvError::Closed) => {
                        break;
                    }
                }
            }

            // 2. Handle Client Disconnects
            client_msg = socket.recv() => {
                match client_msg {
                    Some(Ok(Message::Close(_))) | None => {
                        info!("Client disconnected (socket closed)");
                        break;
                    }
                    Some(Err(e)) => {
                        warn!("Client error: {}", e);
                        break;
                    }
                    _ => {} // Ignore text/binary from client for now
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
                    let payload_str = notification.payload();

                    if let Ok(val) = serde_json::from_str::<Value>(payload_str)
                        && let Some(user_id) = val.get("user_id").and_then(Value::as_i64) {

                            let event = Arc::new(MediaPayload {
                                user_id: user_id as i32,
                                raw_json: payload_str.to_owned(),
                            });

                            // Broadcast the Arc (very cheap, just a pointer copy)
                            if let Err(e) = listener_tx.send(event) {
                                warn!("No active listeners for media update: {}", e);
                            }
                        }
                }
                Err(e) => { warn!("Error receiving from timeline listener: {e:?}") }
            }
        }
    });

    Ok(tx)
}
