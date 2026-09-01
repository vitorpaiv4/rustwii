use dioxus::prelude::*;
use crate::sensors::{haptic_medium, haptic_rumble, haptic_tap, CalibrationOffset};
use crate::types::{ButtonAction, OrientationData, RemoteButton};

#[component]
pub fn RemoteView(room_id: String) -> Element {
    let mut sensor_active = use_signal(|| false);
    let mut status_message = use_signal(|| "Toque para ativar os sensores de movimento".to_string());
    #[allow(unused_mut)]
    let mut raw_orientation = use_signal(|| OrientationData { alpha: 0.0, beta: 0.0, gamma: 0.0 });
    let mut calibrated_orientation = use_signal(|| OrientationData { alpha: 0.0, beta: 0.0, gamma: 0.0 });
    let mut calibration = use_signal(CalibrationOffset::default);
    let mut last_action = use_signal(|| "-".to_string());

    // Helper for button feedback
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
    };

    // Sensor activation handler
    let start_sensors = move |_| {
        #[cfg(target_arch = "wasm32")]
        {
            spawn(async move {
                match crate::sensors::orientation::wasm::request_sensor_permission().await {
                    Ok(granted) => {
                        if granted {
                            status_message.set("Sensores ativados com sucesso!".to_string());
                            sensor_active.set(true);

                            let res = crate::sensors::orientation::wasm::start_orientation_listener(move |data| {
                                raw_orientation.set(data);
                                let cal = calibration.read().apply(data);
                                calibrated_orientation.set(cal);
                            });

                            if let Ok(closure) = res {
                                closure.forget(); // Keep listener alive in JS runtime
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
            status_message.set("Modo Simulação (Ambiente Desktop/Server)".to_string());
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
    };

    let cal = *calibrated_orientation.read();

    rsx! {
        div {
            class: "wiimote-page",
            header {
                class: "wiimote-top-bar",
                h2 { "Wii Remote" }
                span { class: "badge-room", "Sala: {room_id}" }
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
                    div { class: "led-indicator led-active" }
                    div { class: "led-indicator" }
                    div { class: "led-indicator" }
                    div { class: "led-indicator" }
                }
            }

            footer {
                class: "wiimote-footer-status",
                span { "Última Ação: " b { "{last_action}" } }
            }
        }
    }
}
