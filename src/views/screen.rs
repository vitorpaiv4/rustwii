use dioxus::prelude::*;

#[component]
pub fn ScreenView() -> Element {
    let room_code = use_signal(|| "WII-001".to_string());

    rsx! {
        div {
            class: "screen-container",
            header {
                class: "screen-header",
                h1 { "🎮 RustWii - Console Screen" }
                p { "Sala atual: " span { class: "room-badge", "{room_code}" } }
            }

            main {
                class: "screen-content",
                div {
                    class: "pairing-info",
                    p { "Aponte a câmera do celular para o QR Code ou acesse:" }
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
                span { "P1: Desconectado" }
                span { "P2: Desconectado" }
                span { "P3: Desconectado" }
                span { "P4: Desconectado" }
            }
        }
    }
}
