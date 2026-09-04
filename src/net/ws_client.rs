#[cfg(target_arch = "wasm32")]
use crate::types::{ClientMessage, ServerMessage};

/// Constructs the appropriate WebSocket URL according to current window protocol and host
pub fn build_ws_url(room_id: &str, role: &str) -> String {
    #[cfg(target_arch = "wasm32")]
    {
        if let Some(window) = web_sys::window() {
            let location = window.location();
            let host = location.host().unwrap_or_else(|_| "localhost:8080".to_string());
            let protocol = location.protocol().unwrap_or_else(|_| "http:".to_string());
            let ws_protocol = if protocol == "https:" { "wss:" } else { "ws:" };
            return format!("{}//{}/ws/{}?role={}", ws_protocol, host, room_id, role);
        }
    }
    format!("ws://localhost:8080/ws/{}?role={}", room_id, role)
}

#[cfg(target_arch = "wasm32")]
pub mod wasm {
    use super::*;
    use futures_channel::mpsc;
    use futures_util::{SinkExt, StreamExt};
    use gloo_net::websocket::futures::WebSocket;
    use gloo_net::websocket::Message;
    use std::cell::RefCell;
    use std::rc::Rc;

    pub struct WsConnection {
        pub sender: mpsc::UnboundedSender<ClientMessage>,
    }

    impl WsConnection {
        pub fn connect<F, S>(
            url: &str,
            mut on_message: F,
            on_status: S,
        ) -> Result<Self, String>
        where
            F: FnMut(ServerMessage) + 'static,
            S: FnMut(bool, String) + 'static,
        {
            let ws = WebSocket::open(url).map_err(|e| format!("Falha ao conectar WS: {:?}", e))?;
            let (mut write_sink, mut read_stream) = ws.split();

            let (tx, mut rx) = mpsc::unbounded::<ClientMessage>();

            // Task to send messages from channel to WebSocket
            wasm_bindgen_futures::spawn_local(async move {
                while let Some(msg) = rx.next().await {
                    if let Ok(json) = serde_json::to_string(&msg)
                        && write_sink.send(Message::Text(json)).await.is_err() {
                            break;
                        }
                }
            });

            // Task to read incoming messages from WebSocket
            let on_status_cell = Rc::new(RefCell::new(on_status));
            let on_status_clone = on_status_cell.clone();

            (on_status_cell.borrow_mut())(true, "Conectado ao servidor".to_string());

            wasm_bindgen_futures::spawn_local(async move {
                while let Some(msg_res) = read_stream.next().await {
                    match msg_res {
                        Ok(Message::Text(text)) => {
                            if let Ok(server_msg) = serde_json::from_str::<ServerMessage>(&text) {
                                on_message(server_msg);
                            }
                        }
                        Ok(Message::Bytes(_)) => {}
                        Err(err) => {
                            (on_status_clone.borrow_mut())(false, format!("Erro na conexão WS: {:?}", err));
                            break;
                        }
                    }
                }
                (on_status_clone.borrow_mut())(false, "Desconectado do servidor".to_string());
            });

            Ok(Self { sender: tx })
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_build_ws_url_format() {
        let url = build_ws_url("ROOM123", "remote");
        assert!(url.contains("/ws/ROOM123?role=remote"));
    }
}
