use dioxus::prelude::*;

#[component]
pub fn RemoteView(room_id: String) -> Element {
    let mut status = use_signal(|| "Aguardando conexão com a sala...".to_string());

    rsx! {
        div {
            class: "remote-container",
            header {
                class: "remote-header",
                h2 { "🎮 RustWii Remote" }
                span { class: "room-tag", "Sala: {room_id}" }
            }

            main {
                class: "remote-body",
                div {
                    class: "sensor-status",
                    p { "{status}" }
                    button {
                        class: "btn-primary",
                        onclick: move |_| {
                            status.set("Calibração / Sensores acionados".to_string());
                        },
                        "Ativar Giroscópio / Calibrar"
                    }
                }

                div {
                    class: "remote-dpad-section",
                    div { class: "dpad-btn", "▲" }
                    div { class: "dpad-row",
                        div { class: "dpad-btn", "◀" }
                        div { class: "dpad-center", "●" }
                        div { class: "dpad-btn", "▶" }
                    }
                    div { class: "dpad-btn", "▼" }
                }

                div {
                    class: "remote-action-buttons",
                    button { class: "btn-wii-a", "A" }
                    button { class: "btn-wii-b", "B (Gatilho)" }
                }

                div {
                    class: "remote-meta-buttons",
                    button { class: "btn-wii-small", "-" }
                    button { class: "btn-wii-home", "⌂" }
                    button { class: "btn-wii-small", "+" }
                }

                div {
                    class: "remote-numpad-buttons",
                    button { class: "btn-wii-num", "1" }
                    button { class: "btn-wii-num", "2" }
                }
            }
        }
    }
}
