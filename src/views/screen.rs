use dioxus::prelude::*;
use crate::components::{WiiCursor, WiiMenu};
use crate::inertial::CursorState;
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

    #[allow(unused_mut)]
    let mut cursors = use_signal(|| [
        CursorState::new(1),
        CursorState::new(2),
        CursorState::new(3),
        CursorState::new(4),
    ]);

    let mut current_view = use_signal(|| "menu");

    // Dynamic pairing URL computation
    let remote_url = {
        #[cfg(target_arch = "wasm32")]
        {
            if let Some(window) = web_sys::window() {
                let location = window.location();
                let host = location.host().unwrap_or_else(|_| "localhost:8080".to_string());
                let protocol = location.protocol().unwrap_or_else(|_| "http:".to_string());
                format!("{}//{}/remote/{}", protocol, host, room_code.read())
            } else {
                format!("http://localhost:8080/remote/{}", room_code.read())
            }
        }
        #[cfg(not(target_arch = "wasm32"))]
        {
            format!("http://localhost:8080/remote/{}", room_code.read())
        }
    };

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
                            cursors.write()[player_id - 1].set_active(true);
                        }
                    }
                    ServerMessage::PlayerDisconnected { player_id, .. } => {
                        if player_id >= 1 && player_id <= 4 {
                            players.write()[player_id - 1].connected = false;
                            cursors.write()[player_id - 1].set_active(false);
                        }
                    }
                    ServerMessage::PlayerMotion { player_id, orientation } => {
                        if player_id >= 1 && player_id <= 4 {
                            players.write()[player_id - 1].connected = true;
                            players.write()[player_id - 1].orientation = orientation;
                            cursors.write()[player_id - 1].update_orientation(&orientation);
                        }
                    }
                    ServerMessage::PlayerButton { player_id, button, action } => {
                        if player_id >= 1 && player_id <= 4 {
                            let is_press = action == ButtonAction::Press;
                            players.write()[player_id - 1].connected = true;
                            players.write()[player_id - 1].last_button = Some((button, action));

                            match button {
                                RemoteButton::A => cursors.write()[player_id - 1].set_click(is_press),
                                RemoteButton::B => cursors.write()[player_id - 1].set_trigger(is_press),
                                _ => {}
                            }
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
    let current_cursors = cursors.read();

    rsx! {
        div {
            class: "screen-container",

            // Render Multiplayer Wii Cursors Overlay
            for cursor in current_cursors.iter() {
                WiiCursor { cursor: cursor.clone() }
            }

            // Wii Top System Bar
            header {
                class: "wii-system-top-bar",
                div {
                    class: "wii-brand-area",
                    span { class: "wii-logo-icon", "Wii" }
                    span {
                        class: if *is_connected.read() { "badge-ws-online" } else { "badge-ws-offline" },
                        if *is_connected.read() { "CONECTADO" } else { "OFFLINE" }
                    }
                }

                div {
                    class: "wii-system-clock",
                    span { class: "clock-date", "Menu Principal" }
                }

                div {
                    class: "wii-room-tag",
                    span { "Sala: " }
                    b { "{room_code}" }
                }
            }

            // Wii Main Content
            main {
                class: "wii-main-area",
                if *current_view.read() == "menu" {
                    WiiMenu {
                        room_code: room_code.read().clone(),
                        remote_url: remote_url.clone(),
                        cursors: cursors,
                        on_launch_game: move |game_id: &'static str| {
                            current_view.set(game_id);
                        }
                    }
                } else {
                    div {
                        class: "wii-minigame-placeholder",
                        h2 { "🎮 Carregando Mini-game: {current_view}" }
                        p { "Mini-game será renderizado no Canvas (Fase 6)" }
                        button {
                            class: "btn-wii-dialog-back",
                            onclick: move |_| current_view.set("menu"),
                            "◀ Retornar ao Menu Wii"
                        }
                    }
                }
            }

            // Wii Bottom System Bar
            footer {
                class: "wii-system-bottom-bar",
                div { class: "wii-circle-btn", "Wii" }

                div {
                    class: "wii-players-status-row",
                    for (idx, p) in player_slots.iter().enumerate() {
                        div {
                            class: if p.connected { "player-pill pill-online" } else { "player-pill pill-offline" },
                            span { class: "pill-id", "P{idx + 1}" }
                            span { class: "pill-state", if p.connected { "Conectado" } else { "-" } }
                        }
                    }
                }

                div { class: "wii-circle-btn", "✉" }
            }
        }
    }
}
