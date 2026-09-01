use dioxus::prelude::*;
use crate::audio::{play_back, play_click, play_hover, play_start_chime};
use crate::components::icons::*;
use crate::components::qrcode_view::QrCodeView;
use crate::inertial::CursorState;

#[derive(Debug, Clone, PartialEq)]
pub struct WiiChannelData {
    pub id: &'static str,
    pub title: &'static str,
    pub subtitle: &'static str,
    pub is_playable: bool,
}

pub const CHANNELS: [WiiChannelData; 12] = [
    WiiChannelData {
        id: "wii_play",
        title: "RustWii Play",
        subtitle: "Mini-games multiplayer com controle de movimento",
        is_playable: true,
    },
    WiiChannelData {
        id: "target_shoot",
        title: "Tiro ao Alvo",
        subtitle: "Treine sua mira disparando nos alvos em movimento",
        is_playable: true,
    },
    WiiChannelData {
        id: "pairing_channel",
        title: "Parear Wiimote",
        subtitle: "Conecte seu smartphone via QR Code para usar como controle",
        is_playable: false,
    },
    WiiChannelData {
        id: "mii_channel",
        title: "Canal Mii",
        subtitle: "Crie e personalize avatares dos jogadores",
        is_playable: false,
    },
    WiiChannelData {
        id: "forecast",
        title: "Canal Tempo",
        subtitle: "Previsão do tempo global no globo 3D",
        is_playable: false,
    },
    WiiChannelData {
        id: "news",
        title: "Canal Notícias",
        subtitle: "Principais manchetes e acontecimentos do mundo",
        is_playable: false,
    },
    WiiChannelData { id: "empty_7", title: "Canal Vazio", subtitle: "Espaço disponível", is_playable: false },
    WiiChannelData { id: "empty_8", title: "Canal Vazio", subtitle: "Espaço disponível", is_playable: false },
    WiiChannelData { id: "empty_9", title: "Canal Vazio", subtitle: "Espaço disponível", is_playable: false },
    WiiChannelData { id: "empty_10", title: "Canal Vazio", subtitle: "Espaço disponível", is_playable: false },
    WiiChannelData { id: "empty_11", title: "Canal Vazio", subtitle: "Espaço disponível", is_playable: false },
    WiiChannelData { id: "empty_12", title: "Canal Vazio", subtitle: "Espaço disponível", is_playable: false },
];

#[component]
fn ChannelIcon(id: &'static str) -> Element {
    match id {
        "wii_play" => rsx! { DiscIcon {} },
        "target_shoot" => rsx! { TargetIcon {} },
        "pairing_channel" => rsx! { RemoteIcon {} },
        "mii_channel" => rsx! { MiiIcon {} },
        "forecast" => rsx! { WeatherIcon {} },
        "news" => rsx! { NewsIcon {} },
        _ => rsx! { EmptyChannelIcon {} },
    }
}

#[component]
pub fn WiiMenu(
    room_code: String,
    remote_url: String,
    cursors: Signal<[CursorState; 4]>,
    on_launch_game: EventHandler<&'static str>,
) -> Element {
    let mut selected_channel = use_signal(|| Option::<WiiChannelData>::None);
    let mut show_pairing_modal = use_signal(|| false);

    let mut select_channel = move |channel: WiiChannelData| {
        play_click();
        if channel.id == "pairing_channel" {
            show_pairing_modal.set(true);
        } else {
            selected_channel.set(Some(channel));
        }
    };

    let close_modal = move |_| {
        play_back();
        selected_channel.set(None);
    };

    let mut start_game = move |channel_id: &'static str| {
        play_start_chime();
        selected_channel.set(None);
        on_launch_game.call(channel_id);
    };

    rsx! {
        div {
            class: "wii-menu-wrapper",

            // Main Channel Grid (4x3)
            div {
                class: "wii-channels-grid-4x3",
                for channel in CHANNELS.iter() {
                    div {
                        class: if channel.is_playable { "wii-channel-card channel-playable" } else { "wii-channel-card" },
                        onmouseenter: move |_| play_hover(),
                        onclick: {
                            let ch = channel.clone();
                            move |_| select_channel(ch.clone())
                        },

                        div { class: "channel-card-screen",
                            div { class: "channel-card-icon", ChannelIcon { id: channel.id } }
                            b { class: "channel-card-title", "{channel.title}" }
                            if channel.is_playable {
                                span { class: "badge-playable", "JOGÁVEL" }
                            }
                        }
                        div { class: "channel-card-gloss" }
                    }
                }
            }

            // Floating QR Code Pairing Toggle Button & Direct Link
            div {
                class: "wii-pairing-float-bar",
                button {
                    class: "btn-wii-pair",
                    onclick: move |_| {
                        play_click();
                        let curr = *show_pairing_modal.read();
                        show_pairing_modal.set(!curr);
                    },
                    QrIcon {}
                    span { "Parear Smartphone" }
                }
                a {
                    class: "btn-wii-direct-link",
                    href: "{remote_url}",
                    target: "_blank",
                    LinkIcon {}
                    span { "Abrir Controle em Nova Aba" }
                }
            }

            // QR Code Modal
            if *show_pairing_modal.read() {
                div {
                    class: "wii-modal-backdrop",
                    onclick: move |_| show_pairing_modal.set(false),
                    div {
                        class: "wii-modal-card",
                        onclick: move |e| e.stop_propagation(),
                        h2 { "Pareamento de Controles" }
                        p { "Aponte a câmera do celular (na mesma rede Wi-Fi) para conectar:" }
                        QrCodeView { url: remote_url.clone() }
                        a {
                            class: "wii-pairing-url-link",
                            href: "{remote_url}",
                            target: "_blank",
                            "{remote_url}"
                        }
                        p {
                            style: "font-size: 0.8rem; color: #64748b; margin-top: 6px;",
                            "Certifique-se de que o celular e o computador estão na mesma rede Wi-Fi."
                        }
                        button {
                            class: "btn-wii-dialog-back",
                            onclick: move |_| show_pairing_modal.set(false),
                            "Fechar"
                        }
                    }
                }
            }

            // Channel Preview / Launch Dialog Modal
            if let Some(ref ch) = *selected_channel.read() {
                div {
                    class: "wii-modal-backdrop",
                    onclick: close_modal,
                    div {
                        class: "wii-channel-banner-card",
                        onclick: move |e| e.stop_propagation(),

                        div { class: "banner-header",
                            div { class: "banner-icon", ChannelIcon { id: ch.id } }
                            h2 { "{ch.title}" }
                        }
                        p { class: "banner-subtitle", "{ch.subtitle}" }

                        div {
                            class: "banner-action-row",
                            button {
                                class: "btn-wii-dialog-back",
                                onclick: close_modal,
                                "Voltar ao Menu"
                            }
                            if ch.is_playable {
                                button {
                                    class: "btn-wii-dialog-start",
                                    onclick: {
                                        let id = ch.id;
                                        move |_| start_game(id)
                                    },
                                    "Começar"
                                }
                            }
                        }
                    }
                }
            }
        }
    }
}
