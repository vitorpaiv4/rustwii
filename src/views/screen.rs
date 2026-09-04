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
    let mut local_ip = use_signal(String::new);
    #[allow(unused_mut)]
    let mut public_url = use_signal(|| Option::<String>::None);
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

    // Default pairing URL computation
    #[allow(unused_mut)]
    let mut pairing_url = use_signal(|| {
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
    });

    // Auto-discover local network IP and public URL from server
    #[cfg(target_arch = "wasm32")]
    {
        let room_code_for_ip = room_code.read().clone();
        use_effect(move || {
            let rcode = room_code_for_ip.clone();
            spawn(async move {
                if let Ok(resp) = gloo_net::http::Request::get("/api/server-info").send().await
                    && let Ok(data) = resp.json::<serde_json::Value>().await {
                        if let Some(ip) = data["local_ip"].as_str()
                            && !ip.is_empty() && ip != "127.0.0.1" {
                                local_ip.set(ip.to_string());
                                let port = data["https_port"].as_u64().unwrap_or(8443);
                                pairing_url.set(format!("https://{}:{}/remote/{}", ip, port, rcode));
                            }
                        if let Some(pub_u) = data["public_url"].as_str()
                            && !pub_u.is_empty() {
                                public_url.set(Some(pub_u.to_string()));
                                pairing_url.set(format!("{}/remote/{}", pub_u.trim_end_matches('/'), rcode));
                            }
                    }
            });
        });
    }

    // Connect as screen host over WebSocket
    #[allow(unused_variables)]
    let room_code_for_hook = room_code.read().clone();
    use_effect(move || {
        #[cfg(target_arch = "wasm32")]
        {
            let url = build_ws_url(&room_code_for_hook, "screen");
            let mut last_sample_times = [0.0f64; 4];

            fn now_perf_ms() -> f64 {
                if let Some(w) = web_sys::window() {
                    if let Some(p) = w.performance() {
                        return p.now();
                    }
                }
                js_sys::Date::now()
            }

            let _ = crate::net::ws_client::wasm::WsConnection::connect(
                &url,
                move |msg| match msg {
                    ServerMessage::PlayerConnected { player_id, .. } if (1..=4).contains(&player_id) => {
                        players.write()[player_id - 1].connected = true;
                        cursors.write()[player_id - 1].set_active(true);
                        cursors.write()[player_id - 1].recentre();
                    }
                    ServerMessage::PlayerDisconnected { player_id, .. } if (1..=4).contains(&player_id) => {
                        players.write()[player_id - 1].connected = false;
                        cursors.write()[player_id - 1].set_active(false);
                    }
                    ServerMessage::PlayerSample { player_id, sample } if (1..=4).contains(&player_id) => {
                        let idx = player_id - 1;
                        let now = now_perf_ms();
                        let last_t = last_sample_times[idx];
                        let dt = if last_t > 0.0 && now > last_t {
                            ((now - last_t) / 1000.0).clamp(0.001, 0.1)
                        } else {
                            1.0 / 60.0
                        };
                        last_sample_times[idx] = now;

                        players.write()[idx].connected = true;
                        if let (Some(a), Some(b), Some(g)) = (sample.alpha, sample.beta, sample.gamma) {
                            players.write()[idx].orientation = OrientationData { alpha: a, beta: b, gamma: g };
                        }

                        cursors.write()[idx].update_sample(&sample, dt, now);
                    }
                    ServerMessage::PlayerMotion { player_id, orientation } if (1..=4).contains(&player_id) => {
                        let idx = player_id - 1;
                        players.write()[idx].connected = true;
                        players.write()[idx].orientation = orientation;
                        cursors.write()[idx].update_orientation(&orientation);
                    }
                    ServerMessage::PlayerButton { player_id, button, action } if (1..=4).contains(&player_id) => {
                        let idx = player_id - 1;
                        let is_press = action == ButtonAction::Press;
                        players.write()[idx].connected = true;
                        players.write()[idx].last_button = Some((button, action));

                        match button {
                            RemoteButton::A => cursors.write()[idx].set_click(is_press),
                            RemoteButton::B => cursors.write()[idx].set_trigger(is_press),
                            RemoteButton::One if is_press => {
                                cursors.write()[idx].recentre();
                            }
                            RemoteButton::Plus if is_press => {
                                let s = (cursors.read()[idx].pointer.sensitivity + 0.2).min(3.5);
                                cursors.write()[idx].pointer.sensitivity = s;
                            }
                            RemoteButton::Minus if is_press => {
                                let s = (cursors.read()[idx].pointer.sensitivity - 0.2).max(0.4);
                                cursors.write()[idx].pointer.sensitivity = s;
                            }
                            RemoteButton::Home if is_press => {
                                current_view.set("menu");
                            }
                            _ => {}
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

    // 60FPS Cursor extrapolation ticker loop
    use_effect(move || {
        #[cfg(target_arch = "wasm32")]
        {
            spawn(async move {
                loop {
                    gloo_timers::future::TimeoutFuture::new(16).await;
                    let now = if let Some(w) = web_sys::window() {
                        w.performance().map(|p| p.now()).unwrap_or_else(|| js_sys::Date::now())
                    } else {
                        js_sys::Date::now()
                    };
                    let mut c = cursors.write();
                    for cur in c.iter_mut() {
                        if cur.is_active {
                            cur.tick(now);
                        }
                    }
                }
            });
        }
    });

    let player_slots = players.read();
    let current_cursors = cursors.read();

    rsx! {
        div {
            class: "screen-container",
            onmousemove: move |_evt| {
                if !players.read()[0].connected {
                    #[cfg(target_arch = "wasm32")]
                    {
                        if let Some(window) = web_sys::window() {
                            let inner_w = window.inner_width().ok().and_then(|v| v.as_f64()).unwrap_or(1920.0);
                            let inner_h = window.inner_height().ok().and_then(|v| v.as_f64()).unwrap_or(1080.0);
                            let x = (_evt.client_coordinates().x / inner_w * 100.0).clamp(0.0, 100.0);
                            let y = (_evt.client_coordinates().y / inner_h * 100.0).clamp(0.0, 100.0);
                            cursors.write()[0].set_mouse_pos(x, y);
                        }
                    }
                }
            },
            onmousedown: move |_| {
                if !players.read()[0].connected {
                    cursors.write()[0].set_click(true);
                }
            },
            onmouseup: move |_| {
                if !players.read()[0].connected {
                    cursors.write()[0].set_click(false);
                }
            },

            // Render Multiplayer Wii Cursors Overlay
            for cursor in current_cursors.iter() {
                WiiCursor { cursor: cursor.clone() }
            }

            // Wii Top System Bar
            header {
                class: "wii-system-top-bar",
                div {
                    class: "wii-brand-area",
                    span { class: "wii-logo-icon", "RustWii" }
                    span {
                        class: if *is_connected.read() { "badge-ws-online" } else { "badge-ws-offline" },
                        if *is_connected.read() { "CONECTADO" } else { "OFFLINE" }
                    }
                }

                div {
                    class: "wii-system-clock",
                    span { class: "clock-date", "Menu Principal RustWii" }
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
                        remote_url: pairing_url.read().clone(),
                        local_ip: local_ip.read().clone(),
                        public_url: public_url.read().clone(),
                        cursors: cursors,
                        on_launch_game: move |game_id: &'static str| {
                            current_view.set(game_id);
                        },
                        on_update_url: move |new_url: String| {
                            pairing_url.set(new_url);
                        }
                    }
                } else {
                    crate::game::TargetShootingGame {
                        cursors: cursors,
                        on_exit: move |_| current_view.set("menu"),
                    }
                }
            }

            // Wii Bottom System Bar
            footer {
                class: "wii-system-bottom-bar",
                div { class: "wii-circle-btn", style: "font-size: 0.75rem; width: 62px; border-radius: 14px;", "RustWii" }

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

                div { class: "wii-circle-btn", crate::components::MailIcon {} }
            }
        }
    }
}
