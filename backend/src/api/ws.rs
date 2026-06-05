use axum::extract::ws::{Message, WebSocket, WebSocketUpgrade};
use axum::extract::State;
use axum::response::IntoResponse;
use tokio::sync::broadcast;
use tracing::{info, warn};

use crate::engine::EngineEvent;
use crate::ApiState;

/// GET /api/v1/ws — WebSocket upgrade handler
pub async fn ws_handler(
    ws: WebSocketUpgrade,
    State(state): State<ApiState>,
) -> impl IntoResponse {
    ws.on_upgrade(move |socket| handle_ws(socket, state.engine.ws_broadcaster.clone()))
}

async fn handle_ws(mut socket: WebSocket, broadcaster: broadcast::Sender<EngineEvent>) {
    let mut rx = broadcaster.subscribe();
    info!("WebSocket 客户端已连接");

    loop {
        tokio::select! {
            // Read from broadcast channel and forward to WS
            broadcast_msg = rx.recv() => {
                match broadcast_msg {
                    Ok(event) => {
                        let json = serde_json::to_string(&event).unwrap_or_default();
                        if socket.send(Message::Text(json.into())).await.is_err() {
                            break;
                        }
                    }
                    Err(broadcast::error::RecvError::Lagged(n)) => {
                        warn!(n, "WebSocket 客户端落后");
                    }
                    Err(broadcast::error::RecvError::Closed) => break,
                }
            }
            // Read from WebSocket client
            ws_msg = socket.recv() => {
                match ws_msg {
                    Some(Ok(Message::Close(_))) | None => break,
                    Some(Ok(Message::Text(text))) => {
                        info!(message = %text, "WebSocket 收到消息");
                    }
                    Some(Err(e)) => {
                        warn!(error = %e, "WebSocket 错误");
                        break;
                    }
                    _ => {}
                }
            }
        }
    }

    info!("WebSocket 客户端断开");
}
