use dioxus::prelude::*;
#[cfg(target_arch = "wasm32")]
use crate::net::build_ws_url;
#[cfg(target_arch = "wasm32")]
use crate::types::ServerMessage;
use crate::types::{ButtonAction, OrientationData, RemoteButton};

#[derive(Debug, Clone, PartialEq)]
pub struct PlayerSlotState {
    pub connected: bool,
    pub orientation: OrientationData,
    pub last_button: Option<(RemoteButton, ButtonAction)>,
}

impl Default for PlayerSlotState {
    fn default() -> Self {
        Self {
            connected: false,
            orientation: OrientationData { alpha: 0.0, beta: 0.0, gamma: 0.0 },
            last_button: None,
        }
    }
}

#[component]
pub fn ScreenView() -> Element {
    let room_code = use_signal(|| "WII-001".to_string());
    #[allow(unused_mut)]
    let mut is_connected = use_signal(|| false);
    #[allow(unused_mut)]
    let mut players = use_signal(|| [
        PlayerSlotState::default(),
        PlayerSlotState::default(),
        PlayerSlotState::default(),
        PlayerSlotState::default(),
    ]);

    // Connect as screen host over WebSocket
    #[allow(unused_variables)]
    let room_code_for_hook = room_code.read().clone();
    use_effect(move || {
        #[cfg(target_arch = "wasm32")]
        {
            let url = build_ws_url(&room_code_for_hook, "screen");
            let _ = crate::net::ws_client::wasm::WsConnection::connect(
                &url,
                move |msg| match msg {
                    ServerMessage::PlayerConnected { player_id, .. } => {
                        if player_id >= 1 && player_id <= 4 {
                            players.write()[player_id - 1].connected = true;
                        }
                    }
                    ServerMessage::PlayerDisconnected { player_id, .. } => {
                        if player_id >= 1 && player_id <= 4 {
                            players.write()[player_id - 1].connected = false;
                        }
                    }
                    ServerMessage::PlayerMotion { player_id, orientation } => {
                        if player_id >= 1 && player_id <= 4 {
                            let mut p = players.write();
                            p[player_id - 1].connected = true;
                            p[player_id - 1].orientation = orientation;
                        }
                    }
                    ServerMessage::PlayerButton { player_id, button, action } => {
                        if player_id >= 1 && player_id <= 4 {
                            let mut p = players.write();
                            p[player_id - 1].connected = true;
                            p[player_id - 1].last_button = Some((button, action));
                        }
                    }
                    _ => {}
                },
                move |connected, _| {
                    is_connected.set(connected);
                },
            );
        }
    });

    let player_slots = players.read();

    rsx! {
        div {
            class: "screen-container",
            header {
                class: "screen-header",
                div {
                    class: "screen-title-area",
                    h1 { "🎮 RustWii - Console Screen" }
                    span {
                        class: if *is_connected.read() { "badge-ws-online" } else { "badge-ws-offline" },
                        if *is_connected.read() { "SERVER CONECTADO" } else { "LOCAL OFFLINE" }
                    }
                }
                p { "Sala atual: " span { class: "room-badge", "{room_code}" } }
            }

            main {
                class: "screen-content",
                div {
                    class: "pairing-info",
                    p { "Abra no navegador do celular (mesma rede/túnel):" }
                    code { "/remote/{room_code}" }
                }

                div {
                    class: "wii-grid-placeholder",
                    div { class: "wii-channel-slot", "Canal 1: RustWii Play" }
                    div { class: "wii-channel-slot", "Canal 2: Mii Studio" }
                    div { class: "wii-channel-slot", "Canal 3: Tiro ao Alvo" }
                    div { class: "wii-channel-slot", "Canal 4: Configurações" }
                }
            }

            footer {
                class: "screen-footer",
                for (idx, p) in player_slots.iter().enumerate() {
                    div {
                        class: if p.connected { "player-slot-status slot-online" } else { "player-slot-status slot-offline" },
                        div { class: "slot-header", b { "P{idx + 1}" } span { if p.connected { "Online" } else { "Desconectado" } } }
                        if p.connected {
                            div { class: "slot-orientation", "Yaw: {p.orientation.alpha:.0}° | Pitch: {p.orientation.beta:.0}°" }
                            if let Some((btn, act)) = p.last_button {
                                div { class: "slot-button", "Botão: {btn:?} ({act:?})" }
                            }
                        }
                    }
                }
            }
        }
    }
}
