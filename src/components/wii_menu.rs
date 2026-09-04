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
        title: "Parear RustWii Remote",
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

#[cfg(target_arch = "wasm32")]
pub fn copy_to_clipboard(text: &str) {
    if let Some(window) = web_sys::window() {
        let nav = window.navigator();
        let clipboard = nav.clipboard();
        let _ = clipboard.write_text(text);
    }
}

#[cfg(not(target_arch = "wasm32"))]
pub fn copy_to_clipboard(_text: &str) {}

#[component]
pub fn WiiMenu(
    room_code: String,
    remote_url: String,
    local_ip: String,
    public_url: Option<String>,
    cursors: Signal<[CursorState; 4]>,
    on_launch_game: EventHandler<&'static str>,
    on_update_url: EventHandler<String>,
) -> Element {
    let mut selected_channel = use_signal(|| Option::<WiiChannelData>::None);
    let mut show_pairing_modal = use_signal(|| false);
    let mut connection_mode = use_signal(|| {
        if public_url.is_some() {
            "internet".to_string()
        } else {
            "wifi".to_string()
        }
    });
    let mut custom_tunnel_input = use_signal(|| {
        public_url.clone().unwrap_or_default()
    });
    let mut is_copied = use_signal(|| false);

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

    // Calculate dynamic pairing URL according to active connection mode
    let current_mode = connection_mode.read().clone();
    let computed_url = match current_mode.as_str() {
        "internet" => {
            let val = custom_tunnel_input.read().trim().to_string();
            if !val.is_empty() {
                let base = val.trim_end_matches('/');
                if base.starts_with("http://") || base.starts_with("https://") {
                    format!("{}/remote/{}", base, room_code)
                } else {
                    format!("https://{}/remote/{}", base, room_code)
                }
            } else if let Some(ref pub_u) = public_url {
                format!("{}/remote/{}", pub_u.trim_end_matches('/'), room_code)
            } else {
                remote_url.clone()
            }
        }
        "localhost" => {
            format!("http://localhost:8080/remote/{}", room_code)
        }
        _ => {
            // Wi-Fi Local HTTPS (Native Port 8443)
            let ip = if local_ip.is_empty() || local_ip == "127.0.0.1" {
                "localhost".to_string()
            } else {
                local_ip.clone()
            };
            format!("https://{}:8443/remote/{}", ip, room_code)
        }
    };

    let copy_url_action = {
        let url_to_copy = computed_url.clone();
        move |_| {
            play_click();
            copy_to_clipboard(&url_to_copy);
            is_copied.set(true);
            spawn(async move {
                #[cfg(target_arch = "wasm32")]
                gloo_timers::future::TimeoutFuture::new(2000).await;
                is_copied.set(false);
            });
        }
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
                    span { "Parear RustWii Remote" }
                }
                a {
                    class: "btn-wii-direct-link",
                    href: "{computed_url}",
                    target: "_blank",
                    LinkIcon {}
                    span { "Abrir Controle em Nova Aba" }
                }
            }

            // Advanced QR Code & Network Modal
            if *show_pairing_modal.read() {
                div {
                    class: "wii-modal-backdrop",
                    onclick: move |_| show_pairing_modal.set(false),
                    div {
                        class: "wii-modal-card wii-modal-pairing",
                        onclick: move |e| e.stop_propagation(),

                        h2 { "🎮 Pareamento de Controles RustWii" }

                        // Mode Selector Tabs
                        div {
                            class: "wii-mode-tabs",
                            button {
                                class: if *connection_mode.read() == "wifi" { "wii-tab-btn tab-active" } else { "wii-tab-btn" },
                                onclick: move |_| connection_mode.set("wifi".to_string()),
                                WifiIcon {}
                                span { "Wi-Fi Local (HTTPS 8443)" }
                            }
                            button {
                                class: if *connection_mode.read() == "internet" { "wii-tab-btn tab-active" } else { "wii-tab-btn" },
                                onclick: move |_| connection_mode.set("internet".to_string()),
                                GlobeIcon {}
                                span { "Internet (Túnel Público)" }
                            }
                            button {
                                class: if *connection_mode.read() == "localhost" { "wii-tab-btn tab-active" } else { "wii-tab-btn" },
                                onclick: move |_| connection_mode.set("localhost".to_string()),
                                span { "💻 Localhost" }
                            }
                        }

                        // Contextual Info Banner
                        if *connection_mode.read() == "wifi" {
                            div {
                                class: "wii-mode-help",
                                p { "📱 Conecte no mesmo Wi-Fi. " b { "No 1º acesso, aceite o aviso de certificado HTTPS" } " no celular para liberar o giroscópio!" }
                            }
                        } else if *connection_mode.read() == "internet" {
                            div {
                                class: "wii-mode-help help-internet",
                                p { "🌐 " b { "Jogar de qualquer lugar (4G/5G/Internet) & Suporte a iPhone:" } }
                                p { class: "help-small", "Acesse via HTTPS (túnel público) para liberar o giroscópio no Safari/iOS e permitir amigos jogarem à distância!" }
                                
                                div {
                                    class: "wii-tunnel-input-row",
                                    input {
                                        class: "wii-input-url",
                                        r#type: "text",
                                        placeholder: "https://meu-tunel.trycloudflare.com",
                                        value: "{custom_tunnel_input}",
                                        oninput: move |evt| {
                                            custom_tunnel_input.set(evt.value());
                                        }
                                    }
                                }
                                div {
                                    class: "wii-tunnel-tip-box",
                                    span { class: "tip-title", "💡 Como gerar um link público grátis em 5 segundos:" }
                                    code { "cloudflared tunnel --url http://localhost:8080" }
                                    span { class: "tip-or", "ou" }
                                    code { "npx localtunnel --port 8080" }
                                }
                            }
                        } else {
                            div {
                                class: "wii-mode-help help-warn",
                                p { "⚠️ " b { "Aviso:" } " O endereço " code { "localhost" } " só funciona no próprio computador (abra em nova aba)." }
                            }
                        }

                        // QR Code View
                        QrCodeView { url: computed_url.clone() }

                        // Direct URL & Copy Action Bar
                        div {
                            class: "wii-url-action-bar",
                            a {
                                class: "wii-pairing-url-link",
                                href: "{computed_url}",
                                target: "_blank",
                                "{computed_url}"
                            }
                            button {
                                class: if *is_copied.read() { "btn-wii-copy btn-copied" } else { "btn-wii-copy" },
                                onclick: copy_url_action,
                                if *is_copied.read() {
                                    CheckIcon {}
                                    span { "Copiado!" }
                                } else {
                                    CopyIcon {}
                                    span { "Copiar" }
                                }
                            }
                        }

                        button {
                            class: "btn-wii-dialog-back",
                            style: "margin-top: 8px; width: 100%;",
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
