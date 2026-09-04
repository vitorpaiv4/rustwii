#[cfg(not(target_arch = "wasm32"))]
use axum::{
    extract::{
        ws::{Message, WebSocket, WebSocketUpgrade},
        Path, Query, State,
    },
    response::IntoResponse,
};
#[cfg(not(target_arch = "wasm32"))]
use futures_util::{SinkExt, StreamExt};
#[cfg(not(target_arch = "wasm32"))]
use serde::Deserialize;
#[cfg(not(target_arch = "wasm32"))]
use crate::server::room::RoomManager;
#[cfg(not(target_arch = "wasm32"))]
use crate::types::{ClientMessage, ServerMessage};

#[cfg(not(target_arch = "wasm32"))]
#[derive(Debug, Deserialize)]
pub struct WsQuery {
    pub role: Option<String>,
}

#[cfg(not(target_arch = "wasm32"))]
pub async fn ws_handler(
    Path(room_id): Path<String>,
    Query(query): Query<WsQuery>,
    State(manager): State<RoomManager>,
    ws: WebSocketUpgrade,
) -> impl IntoResponse {
    let role = query.role.unwrap_or_else(|| "screen".to_string());
    ws.on_upgrade(move |socket| handle_socket(socket, room_id, role, manager))
}

#[cfg(not(target_arch = "wasm32"))]
async fn handle_socket(
    socket: WebSocket,
    room_id: String,
    role: String,
    manager: RoomManager,
) {
    let (mut ws_sender, mut ws_receiver) = socket.split();

    if role == "remote" {
        let allocation = manager.join_as_remote(&room_id).await;
        let (player_id, tx, mut rx, total_players) = match allocation {
            Some(data) => data,
            None => {
                let err = ServerMessage::Error {
                    message: "Sala cheia (máximo de 4 jogadores)".to_string(),
                };
                if let Ok(json) = serde_json::to_string(&err) {
                    let _ = ws_sender.send(Message::Text(json)).await;
                }
                return;
            }
        };

        // Notify client of allocated player_id
        let joined_msg = ServerMessage::RoomJoined {
            player_id,
            room_id: room_id.clone(),
        };
        if let Ok(json) = serde_json::to_string(&joined_msg) {
            let _ = ws_sender.send(Message::Text(json)).await;
        }

        // Notify screen & others that player connected
        let _ = tx.send(ServerMessage::PlayerConnected {
            player_id,
            total_players,
        });

        // Spawn task to forward server feedback back to remote
        let send_task = tokio::spawn(async move {
            while let Ok(msg) = rx.recv().await {
                match msg {
                    ServerMessage::Pong | ServerMessage::Feedback { .. } => {
                        if let Ok(json) = serde_json::to_string(&msg)
                            && ws_sender.send(Message::Text(json)).await.is_err() {
                                break;
                            }
                    }
                    _ => {}
                }
            }
        });

        // Read incoming client messages from phone
        while let Some(Ok(msg)) = ws_receiver.next().await {
            match msg {
                Message::Text(text) => {
                    if let Ok(client_msg) = serde_json::from_str::<ClientMessage>(&text) {
                        match client_msg {
                            ClientMessage::Sample(sample) => {
                                let _ = tx.send(ServerMessage::PlayerSample {
                                    player_id,
                                    sample,
                                });
                            }
                            ClientMessage::Motion(orientation) => {
                                let _ = tx.send(ServerMessage::PlayerMotion {
                                    player_id,
                                    orientation,
                                });
                            }
                            ClientMessage::Button { button, action } => {
                                let _ = tx.send(ServerMessage::PlayerButton {
                                    player_id,
                                    button,
                                    action,
                                });
                            }
                            ClientMessage::Ping => {
                                let _ = tx.send(ServerMessage::Pong);
                            }
                            _ => {}
                        }
                    }
                }
                Message::Close(_) => break,
                _ => {}
            }
        }

        send_task.abort();

        // Handle disconnect
        let remaining = manager.leave(&room_id, player_id).await;
        let _ = tx.send(ServerMessage::PlayerDisconnected {
            player_id,
            total_players: remaining,
        });
    } else {
        // Screen (Host/Display) mode
        let (mut rx, _, _total) = manager.join_as_screen(&room_id).await;

        let send_task = tokio::spawn(async move {
            while let Ok(msg) = rx.recv().await {
                if let Ok(json) = serde_json::to_string(&msg)
                    && ws_sender.send(Message::Text(json)).await.is_err() {
                        break;
                    }
            }
        });

        while let Some(Ok(msg)) = ws_receiver.next().await {
            if let Message::Close(_) = msg {
                break;
            }
        }

        send_task.abort();
    }
}

#[cfg(not(target_arch = "wasm32"))]
pub fn get_local_ip() -> String {
    if let Ok(socket) = std::net::UdpSocket::bind("0.0.0.0:0")
        && socket.connect("1.1.1.1:80").is_ok()
            && let Ok(addr) = socket.local_addr() {
                return addr.ip().to_string();
            }
    "127.0.0.1".to_string()
}

#[cfg(not(target_arch = "wasm32"))]
pub async fn server_info_handler() -> axum::Json<serde_json::Value> {
    let local_ip = get_local_ip();
    let public_url = std::env::var("PUBLIC_URL")
        .or_else(|_| std::env::var("HOST_URL"))
        .ok();

    axum::Json(serde_json::json!({
        "local_ip": local_ip,
        "http_port": 8080,
        "https_port": 8443,
        "https_available": true,
        "public_url": public_url,
    }))
}
