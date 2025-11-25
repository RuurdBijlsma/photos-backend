use axum::extract::ws::{Message, Utf8Bytes, WebSocket};
use color_eyre::Result;
use sqlx::postgres::PgListener;
use sqlx::PgPool;
use tokio::sync::broadcast;
use tracing::{error, info, warn};
use crate::api_state::ApiContext;

pub async fn handle_timeline_socket(mut socket: WebSocket, context: ApiContext) {
    info!("Client connected to timeline websocket");

    // Subscribe to the broadcaster
    let mut rx = context.timeline_broadcaster.subscribe();

    loop {
        match rx.recv().await {
            Ok(msg) => {
                // Send the payload (JSON string of the new media_item) to the client
                if let Err(e) = socket.send(Message::Text(Utf8Bytes::from(msg))).await {
                    warn!("Client disconnected abruptly: {}", e);
                    break;
                }
            }
            Err(broadcast::error::RecvError::Lagged(count)) => {
                warn!("Client is lagging, skipped {} messages", count);
            }
            Err(broadcast::error::RecvError::Closed) => {
                break;
            }
        }
    }
}

pub fn create_media_item_transmitter(pool: &PgPool) -> Result<broadcast::Sender<String>> {
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
                    let payload = notification.payload();
                    // Broadcast the raw JSON payload to all connected WebSocket clients
                    if let Err(e) = listener_tx.send(payload.to_string()) {
                        // This usually happens if no one is connected, which is fine
                        warn!(
                            "Failed to broadcast media item (no active listeners?): {}",
                            e
                        );
                    }
                }
                Err(e) => {
                    error!("Error receiving notification: {}", e);
                    // Add a small delay/retry logic here in a real production scenario
                    tokio::time::sleep(std::time::Duration::from_secs(5)).await;
                }
            }
        }
    });

    Ok(tx)
}
