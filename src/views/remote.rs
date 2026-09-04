use dioxus::prelude::*;
#[cfg(target_arch = "wasm32")]
use crate::audio::{play_click, unlock_audio};
#[cfg(target_arch = "wasm32")]
use crate::net::build_ws_url;
use crate::sensors::{haptic_medium, haptic_rumble, haptic_tap};
#[cfg(target_arch = "wasm32")]
use crate::types::ServerMessage;
use crate::types::{ButtonAction, ClientMessage, MotionData, OrientationSample, RemoteButton};

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
    let mut status_message = use_signal(|| "Toque em Ativar para conectar o RustWii Remote".to_string());
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
                    ServerMessage::Feedback { kind, combo: _ } => {
                        if kind == "slice" {
                            crate::sensors::haptic_rumble();
                            crate::audio::play_swoosh();
                        } else if kind == "bomb" {
                            crate::sensors::haptic_rumble();
                            crate::audio::play_thud();
                        }
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

    let mut sensitivity = use_signal(|| 1.0f64);
    let mut last_touch_pos = use_signal(|| Option::<(f64, f64)>::None);
    let mut virtual_angles = use_signal(|| (0.0f64, 0.0f64)); // (yaw, pitch)

    // Calibration handler
    let mut recenter_action = move || {
        haptic_medium();
        virtual_angles.set((0.0, 0.0));
        last_action.set("Centro Calibrado!".to_string());
        send_ws_message(ClientMessage::CalibrateCenter);
    };

    // Helper for button feedback and dispatch
    let mut handle_button = move |button: RemoteButton, action: ButtonAction| {
        if action == ButtonAction::Press {
            match button {
                RemoteButton::Minus => {
                    let s = (*sensitivity.read() - 0.2).max(0.4);
                    sensitivity.set(s);
                    last_action.set(format!("Velocidade Mira: {:.1}x", s));
                    haptic_tap();
                    send_ws_message(ClientMessage::Speed { factor: s });
                }
                RemoteButton::Plus => {
                    let s = (*sensitivity.read() + 0.2).min(2.5);
                    sensitivity.set(s);
                    last_action.set(format!("Velocidade Mira: {:.1}x", s));
                    haptic_tap();
                    send_ws_message(ClientMessage::Speed { factor: s });
                }
                RemoteButton::One => {
                    recenter_action();
                }
                RemoteButton::A => {
                    #[cfg(target_arch = "wasm32")]
                    play_click();
                    haptic_tap();
                }
                RemoteButton::B => haptic_rumble(),
                RemoteButton::Home => haptic_medium(),
                _ => haptic_tap(),
            }
        }
        let action_str = match action {
            ButtonAction::Press => "Pressionado",
            ButtonAction::Release => "Solto",
        };
        if button != RemoteButton::Minus && button != RemoteButton::Plus && button != RemoteButton::One {
            last_action.set(format!("{:?} ({})", button, action_str));
        }

        send_ws_message(ClientMessage::Button { button, action });
    };

    // Sensor activation handler
    let start_sensors = move |_| {
        #[cfg(target_arch = "wasm32")]
        {
            unlock_audio();
            spawn(async move {
                match crate::sensors::orientation::wasm::request_sensor_permission().await {
                    Ok(granted) => {
                        if granted {
                            status_message.set("Sensores ativos (Streaming 60Hz)!".to_string());
                            sensor_active.set(true);
                            haptic_medium();

                            let sender_holder = ws_sender.read().clone();
                            let res = crate::sensors::orientation::wasm::start_controller_streaming(move |sample| {
                                if let Some(ref sender) = *sender_holder.borrow() {
                                    let _ = sender.unbounded_send(ClientMessage::Sample(sample));
                                }
                            });

                            if let Ok(closure) = res {
                                closure.forget();
                            }
                        } else {
                            status_message.set("Permissão dos sensores negada no navegador.".to_string());
                        }
                    }
                    Err(_) => {
                        status_message.set("Aviso: No iOS/Safari os sensores exigem HTTPS.".to_string());
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

    // Virtual Touch Aiming Drag Handler
    let mut on_touch_drag_move = move |x: f64, y: f64| {
        if let Some((prev_x, prev_y)) = *last_touch_pos.read() {
            let dx = x - prev_x;
            let dy = y - prev_y;

            let (curr_yaw, curr_pitch) = *virtual_angles.read();
            let new_yaw = (curr_yaw + (dx * 0.15)).clamp(-24.0, 24.0);
            let new_pitch = (curr_pitch - (dy * 0.15)).clamp(-16.0, 16.0);
            virtual_angles.set((new_yaw, new_pitch));

            let motion_sample = OrientationSample {
                alpha: Some(new_yaw),
                beta: Some(new_pitch),
                gamma: Some(0.0),
                heading: None,
                quat: None,
                motion: Some(MotionData {
                    ax: 0.0,
                    ay: 0.0,
                    az: 0.0,
                    rx: -dy * 2.0,
                    ry: 0.0,
                    rz: dx * 2.0,
                }),
                t: 0.0,
            };
            send_ws_message(ClientMessage::Sample(motion_sample));
        }
        last_touch_pos.set(Some((x, y)));
    };

    let current_pid = *player_id.read();

    let is_secure = {
        #[cfg(target_arch = "wasm32")]
        { crate::sensors::orientation::wasm::is_secure_context() }
        #[cfg(not(target_arch = "wasm32"))]
        { true }
    };
    let mut show_perm_guide = use_signal(|| false);

    rsx! {
        div {
            class: "wiimote-page",
            header {
                class: "wiimote-top-bar",
                h2 { "RustWii Remote" }
                div {
                    class: "wiimote-top-tags",
                    span {
                        class: if *is_connected.read() { "badge-ws-online" } else { "badge-ws-offline" },
                        if *is_connected.read() { "ONLINE" } else { "OFFLINE" }
                    }
                    span { class: "badge-room", "Sala: {room_id}" }
                }
            }

            // Insecure Context Warning (if not HTTPS / localhost)
            if !is_secure {
                div {
                    class: "remote-insecure-warning",
                    p { "⚠️ " b { "Aviso HTTP:" } " Navegadores bloqueiam o giroscópio em HTTP. Acesse via " b { "HTTPS (porta 8443)" } " para liberar os sensores!" }
                }
            }

            // Sensor status & Activation bar
            div {
                class: "sensor-card",
                p { class: "status-text", "{status_message}" }
                if !*sensor_active.read() {
                    button {
                        class: "btn-sensor-activate",
                        onclick: start_sensors,
                        "📡 Ativar Giroscópio (Motion 60Hz)"
                    }
                } else {
                    div {
                        class: "telemetry-subbar",
                        span { class: "telemetry-badge-active", "● Sensores Ativos (60Hz)" }
                        button {
                            class: "btn-recenter-mini",
                            onclick: move |_| recenter_action(),
                            "🎯 Calibrar Centro (Botão 1)"
                        }
                    }
                }

                // Quick Permission Help Button
                button {
                    class: "btn-perm-help-link",
                    onclick: move |_| {
                        let curr = *show_perm_guide.read();
                        show_perm_guide.set(!curr);
                    },
                    "❓ Ajuda / Permissões do Celular"
                }
            }

            // Permission Guide Popup
            if *show_perm_guide.read() {
                div {
                    class: "remote-perm-guide-card",
                    h4 { "📱 Como liberar o Giroscópio no Celular:" }
                    ul {
                        li {
                            b { "1. Use HTTPS (Porta 8443): " }
                            "No endereço do celular, use sempre " code { "https://" } " (ex: https://192.168.x.x:8443). Se aparecer aviso de certificado, toque em Avançado -> Continuar."
                        }
                        li {
                            b { "2. No Android (Google Chrome): " }
                            "Toque no menu (3 pontinhos) -> Configurações -> Configurações do site -> " b { "Sensores de movimento" } " -> selecione " b { "Permitido" } "."
                        }
                        li {
                            b { "3. No iPhone (Safari): " }
                            "Abra Ajustes -> Safari -> role até 'Acesso a Movimento e Orientação' e ative."
                        }
                        li {
                            b { "4. Mira por Toque (Touch Aiming): " }
                            "Você também pode deslizar o dedo na área do controle abaixo para mirar diretamente!"
                        }
                    }
                    button {
                        class: "btn-perm-guide-close",
                        onclick: move |_| show_perm_guide.set(false),
                        "Entendido"
                    }
                }
            }

            // Wiimote Chassis (Also acts as virtual trackpad on drag)
            div {
                class: "wiimote-chassis",
                onpointerdown: move |evt| {
                    last_touch_pos.set(Some((evt.client_coordinates().x, evt.client_coordinates().y)));
                },
                onpointermove: move |evt| {
                    on_touch_drag_move(evt.client_coordinates().x, evt.client_coordinates().y);
                },
                onpointerup: move |_| {
                    last_touch_pos.set(None);
                },

                // Player Slot Indicator Banner
                if let Some(pid) = current_pid {
                    div { class: "player-slot-badge", "JOGADOR {pid}" }
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
