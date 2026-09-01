use dioxus::prelude::*;
#[cfg(target_arch = "wasm32")]
use crate::net::build_ws_url;
use crate::sensors::{haptic_medium, haptic_rumble, haptic_tap, CalibrationOffset};
#[cfg(target_arch = "wasm32")]
use crate::types::ServerMessage;
use crate::types::{ButtonAction, ClientMessage, OrientationData, RemoteButton};

#[cfg(target_arch = "wasm32")]
use futures_channel::mpsc;
#[cfg(target_arch = "wasm32")]
use std::cell::RefCell;
#[cfg(target_arch = "wasm32")]
use std::rc::Rc;

#[component]
pub fn RemoteView(room_id: String) -> Element {
    let mut sensor_active = use_signal(|| false);
    #[allow(unused_mut)]
    let mut is_connected = use_signal(|| false);
    #[allow(unused_mut)]
    let mut player_id = use_signal(|| Option::<usize>::None);
    let mut status_message = use_signal(|| "Toque para ativar os sensores de movimento".to_string());
    #[allow(unused_mut)]
    let mut raw_orientation = use_signal(|| OrientationData { alpha: 0.0, beta: 0.0, gamma: 0.0 });
    let mut calibrated_orientation = use_signal(|| OrientationData { alpha: 0.0, beta: 0.0, gamma: 0.0 });
    let mut calibration = use_signal(CalibrationOffset::default);
    let mut last_action = use_signal(|| "-".to_string());

    #[cfg(target_arch = "wasm32")]
    let ws_sender = use_signal(|| Rc::new(RefCell::new(Option::<mpsc::UnboundedSender<ClientMessage>>::None)));

    // Connect to WebSocket on startup (WASM)
    #[allow(unused_variables)]
    let room_id_for_hook = room_id.clone();
    use_effect(move || {
        #[cfg(target_arch = "wasm32")]
        {
            let url = build_ws_url(&room_id_for_hook, "remote");
            let sender_holder = ws_sender.read().clone();

            let res = crate::net::ws_client::wasm::WsConnection::connect(
                &url,
                move |msg| match msg {
                    ServerMessage::RoomJoined { player_id: pid, .. } => {
                        player_id.set(Some(pid));
                        status_message.set(format!("Conectado como Jogador P{}!", pid));
                        haptic_medium();
                    }
                    ServerMessage::Error { message } => {
                        status_message.set(format!("Erro: {}", message));
                    }
                    _ => {}
                },
                move |connected, status| {
                    is_connected.set(connected);
                    if !connected {
                        player_id.set(None);
                    }
                    status_message.set(status);
                },
            );

            if let Ok(conn) = res {
                *sender_holder.borrow_mut() = Some(conn.sender);
            }
        }
    });

    // Helper to send messages over WebSocket
    let send_ws_message = move |msg: ClientMessage| {
        #[cfg(target_arch = "wasm32")]
        {
            if let Some(ref sender) = *ws_sender.read().borrow() {
                let _ = sender.unbounded_send(msg);
            }
        }
        #[cfg(not(target_arch = "wasm32"))]
        {
            let _ = msg;
        }
    };

    // Helper for button feedback and dispatch
    let mut handle_button = move |button: RemoteButton, action: ButtonAction| {
        match button {
            RemoteButton::B => haptic_rumble(),
            RemoteButton::Home => haptic_medium(),
            _ => haptic_tap(),
        }
        let action_str = match action {
            ButtonAction::Press => "Pressionado",
            ButtonAction::Release => "Solto",
        };
        last_action.set(format!("{:?} ({})", button, action_str));

        send_ws_message(ClientMessage::Button { button, action });
    };

    // Sensor activation handler
    let start_sensors = move |_| {
        #[cfg(target_arch = "wasm32")]
        {
            spawn(async move {
                match crate::sensors::orientation::wasm::request_sensor_permission().await {
                    Ok(granted) => {
                        if granted {
                            status_message.set("Sensores ativos e calibrados!".to_string());
                            sensor_active.set(true);

                            let sender_holder = ws_sender.read().clone();
                            let res = crate::sensors::orientation::wasm::start_orientation_listener(move |data| {
                                raw_orientation.set(data);
                                let cal = calibration.read().apply(data);
                                calibrated_orientation.set(cal);

                                // Send motion to server
                                if let Some(ref sender) = *sender_holder.borrow() {
                                    let _ = sender.unbounded_send(ClientMessage::Motion(cal));
                                }
                            });

                            if let Ok(closure) = res {
                                closure.forget();
                            }
                        } else {
                            status_message.set("Permissão dos sensores foi negada.".to_string());
                        }
                    }
                    Err(err) => {
                        status_message.set(format!("Erro ao ativar sensores: {:?}", err));
                    }
                }
            });
        }
        #[cfg(not(target_arch = "wasm32"))]
        {
            sensor_active.set(true);
            status_message.set("Modo Simulação Desktop".to_string());
        }
    };

    // Calibration handler
    let recenter = move |_| {
        haptic_medium();
        let current_raw = *raw_orientation.read();
        calibration.write().calibrate_from(current_raw);
        let cal = calibration.read().apply(current_raw);
        calibrated_orientation.set(cal);
        last_action.set("Centro Calibrado!".to_string());
        send_ws_message(ClientMessage::CalibrateCenter);
    };

    let cal = *calibrated_orientation.read();
    let current_pid = *player_id.read();

    rsx! {
        div {
            class: "wiimote-page",
            header {
                class: "wiimote-top-bar",
                h2 { "Wii Remote" }
                div {
                    class: "wiimote-top-tags",
                    span {
                        class: if *is_connected.read() { "badge-ws-online" } else { "badge-ws-offline" },
                        if *is_connected.read() { "ONLINE" } else { "OFFLINE" }
                    }
                    span { class: "badge-room", "Sala: {room_id}" }
                }
            }

            // Sensor status & Calibration bar
            div {
                class: "sensor-card",
                p { class: "status-text", "{status_message}" }
                if !*sensor_active.read() {
                    button {
                        class: "btn-sensor-activate",
                        onclick: start_sensors,
                        "📡 Ativar Sensores (Giroscópio)"
                    }
                } else {
                    div {
                        class: "telemetry-panel",
                        div { class: "telemetry-item", span { "Yaw (Z):" } b { "{cal.alpha:.1}°" } }
                        div { class: "telemetry-item", span { "Pitch (X):" } b { "{cal.beta:.1}°" } }
                        div { class: "telemetry-item", span { "Roll (Y):" } b { "{cal.gamma:.1}°" } }
                    }
                    button {
                        class: "btn-recenter",
                        onclick: recenter,
                        "🎯 Recentralizar / Calibrar"
                    }
                }
            }

            // Wiimote Chassis
            div {
                class: "wiimote-chassis",

                // Player Slot Indicator Banner
                if let Some(pid) = current_pid {
                    div { class: "player-slot-badge", "🎮 JOGADOR {pid}" }
                }

                // D-Pad Section
                div {
                    class: "wiimote-dpad",
                    button {
                        class: "dpad-btn dpad-up",
                        onpointerdown: move |_| handle_button(RemoteButton::DpadUp, ButtonAction::Press),
                        onpointerup: move |_| handle_button(RemoteButton::DpadUp, ButtonAction::Release),
                        "▲"
                    }
                    div {
                        class: "dpad-middle-row",
                        button {
                            class: "dpad-btn dpad-left",
                            onpointerdown: move |_| handle_button(RemoteButton::DpadLeft, ButtonAction::Press),
                            onpointerup: move |_| handle_button(RemoteButton::DpadLeft, ButtonAction::Release),
                            "◀"
                        }
                        div { class: "dpad-core" }
                        button {
                            class: "dpad-btn dpad-right",
                            onpointerdown: move |_| handle_button(RemoteButton::DpadRight, ButtonAction::Press),
                            onpointerup: move |_| handle_button(RemoteButton::DpadRight, ButtonAction::Release),
                            "▶"
                        }
                    }
                    button {
                        class: "dpad-btn dpad-down",
                        onpointerdown: move |_| handle_button(RemoteButton::DpadDown, ButtonAction::Press),
                        onpointerup: move |_| handle_button(RemoteButton::DpadDown, ButtonAction::Release),
                        "▼"
                    }
                }

                // Action Buttons (A & B)
                div {
                    class: "wiimote-actions",
                    button {
                        class: "btn-wii-a-tactile",
                        onpointerdown: move |_| handle_button(RemoteButton::A, ButtonAction::Press),
                        onpointerup: move |_| handle_button(RemoteButton::A, ButtonAction::Release),
                        "A"
                    }
                    button {
                        class: "btn-wii-b-tactile",
                        onpointerdown: move |_| handle_button(RemoteButton::B, ButtonAction::Press),
                        onpointerup: move |_| handle_button(RemoteButton::B, ButtonAction::Release),
                        "B (Gatilho)"
                    }
                }

                // Middle Buttons (-, Home, +)
                div {
                    class: "wiimote-middle-row",
                    button {
                        class: "btn-wii-symbol",
                        onpointerdown: move |_| handle_button(RemoteButton::Minus, ButtonAction::Press),
                        onpointerup: move |_| handle_button(RemoteButton::Minus, ButtonAction::Release),
                        "-"
                    }
                    button {
                        class: "btn-wii-home-tactile",
                        onpointerdown: move |_| handle_button(RemoteButton::Home, ButtonAction::Press),
                        onpointerup: move |_| handle_button(RemoteButton::Home, ButtonAction::Release),
                        "⌂"
                    }
                    button {
                        class: "btn-wii-symbol",
                        onpointerdown: move |_| handle_button(RemoteButton::Plus, ButtonAction::Press),
                        onpointerup: move |_| handle_button(RemoteButton::Plus, ButtonAction::Release),
                        "+"
                    }
                }

                // Bottom Buttons (1, 2)
                div {
                    class: "wiimote-bottom-row",
                    button {
                        class: "btn-wii-num-tactile",
                        onpointerdown: move |_| handle_button(RemoteButton::One, ButtonAction::Press),
                        onpointerup: move |_| handle_button(RemoteButton::One, ButtonAction::Release),
                        "1"
                    }
                    button {
                        class: "btn-wii-num-tactile",
                        onpointerdown: move |_| handle_button(RemoteButton::Two, ButtonAction::Press),
                        onpointerup: move |_| handle_button(RemoteButton::Two, ButtonAction::Release),
                        "2"
                    }
                }

                // LED Indicators (P1 - P4)
                div {
                    class: "wiimote-led-row",
                    for i in 1..=4 {
                        div {
                            class: if current_pid == Some(i) { "led-indicator led-active" } else { "led-indicator" }
                        }
                    }
                }
            }

            footer {
                class: "wiimote-footer-status",
                span { "Última Ação: " b { "{last_action}" } }
            }
        }
    }
}
