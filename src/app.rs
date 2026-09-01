use dioxus::prelude::*;
use crate::route::Route;

const MAIN_CSS: &str = r#"
* {
    box-sizing: border-box;
    margin: 0;
    padding: 0;
    font-family: -apple-system, BlinkMacSystemFont, 'Segoe UI', Roboto, Helvetica, Arial, sans-serif;
    user-select: none;
    -webkit-user-select: none;
    -webkit-touch-callout: none;
}

body {
    background-color: #e9ecef;
    color: #333333;
    overflow-x: hidden;
    min-height: 100vh;
}

/* ========================================================= */
/* Wii System Screen View Styles                             */
/* ========================================================= */
.screen-container {
    display: flex;
    flex-direction: column;
    height: 100vh;
    justify-content: space-between;
    padding: 16px 24px;
    background: radial-gradient(circle at 50% 40%, #ffffff 0%, #d5e1eb 80%, #b8cad8 100%);
    overflow: hidden;
    position: relative;
}

.wii-system-top-bar {
    display: flex;
    justify-content: space-between;
    align-items: center;
    padding: 8px 16px;
    background: rgba(255, 255, 255, 0.75);
    backdrop-filter: blur(8px);
    border-radius: 16px;
    border-bottom: 2px solid #cbd5e1;
}

.wii-brand-area {
    display: flex;
    align-items: center;
    gap: 12px;
}

.wii-logo-icon {
    font-size: 1.6rem;
    font-weight: 900;
    color: #475569;
    letter-spacing: -1px;
}

.wii-system-clock {
    font-size: 1.1rem;
    font-weight: 700;
    color: #334155;
}

.wii-room-tag {
    background: #0284c7;
    color: white;
    padding: 4px 12px;
    border-radius: 20px;
    font-size: 0.9rem;
}

.wii-main-area {
    flex: 1;
    display: flex;
    align-items: center;
    justify-content: center;
    padding: 12px 0;
}

.wii-menu-wrapper {
    display: flex;
    flex-direction: column;
    align-items: center;
    width: 100%;
    max-width: 1100px;
    gap: 12px;
}

/* 4 Columns x 3 Rows Channels Grid */
.wii-channels-grid-4x3 {
    display: grid;
    grid-template-columns: repeat(4, 1fr);
    grid-template-rows: repeat(3, 115px);
    gap: 14px;
    width: 100%;
}

.wii-channel-card {
    position: relative;
    background: linear-gradient(180deg, #ffffff 0%, #e2e8f0 100%);
    border: 3px solid #b0bec5;
    border-radius: 18px;
    box-shadow: 0 4px 10px rgba(0, 0, 0, 0.08), inset 0 2px 4px rgba(255, 255, 255, 0.9);
    cursor: pointer;
    overflow: hidden;
    transition: transform 0.15s ease, border-color 0.15s ease, box-shadow 0.15s ease;
    display: flex;
    align-items: center;
    justify-content: center;
}

.wii-channel-card:hover {
    transform: scale(1.04);
    border-color: #00a8e8;
    box-shadow: 0 8px 24px rgba(0, 168, 232, 0.35), inset 0 2px 4px rgba(255, 255, 255, 1);
}

.channel-playable {
    border-color: #7dd3fc;
}

.channel-card-screen {
    display: flex;
    flex-direction: column;
    align-items: center;
    justify-content: center;
    gap: 4px;
    z-index: 2;
    text-align: center;
    padding: 8px;
}

.channel-card-icon {
    font-size: 1.8rem;
}

.channel-card-title {
    font-size: 0.85rem;
    color: #1e293b;
    font-weight: 700;
}

.badge-playable {
    background: #0284c7;
    color: white;
    font-size: 0.65rem;
    font-weight: 900;
    padding: 2px 6px;
    border-radius: 8px;
    letter-spacing: 0.5px;
}

.channel-card-gloss {
    position: absolute;
    top: 0;
    left: 0;
    right: 0;
    height: 45%;
    background: linear-gradient(180deg, rgba(255,255,255,0.7) 0%, rgba(255,255,255,0) 100%);
    pointer-events: none;
    border-top-left-radius: 15px;
    border-top-right-radius: 15px;
}

.wii-pairing-float-bar {
    margin-top: 4px;
}

.btn-wii-pair {
    background: linear-gradient(135deg, #0284c7, #0369a1);
    color: white;
    font-weight: bold;
    font-size: 0.95rem;
    padding: 8px 20px;
    border: none;
    border-radius: 20px;
    cursor: pointer;
    box-shadow: 0 4px 12px rgba(2, 132, 199, 0.3);
    transition: transform 0.15s ease;
}

.btn-wii-pair:hover {
    transform: scale(1.03);
}

/* Wii System Bottom Bar */
.wii-system-bottom-bar {
    display: flex;
    justify-content: space-between;
    align-items: center;
    padding: 8px 16px;
    background: rgba(255, 255, 255, 0.8);
    backdrop-filter: blur(8px);
    border-radius: 18px;
    border-top: 2px solid #cbd5e1;
}

.wii-circle-btn {
    width: 44px;
    height: 44px;
    border-radius: 50%;
    background: linear-gradient(180deg, #ffffff 0%, #cbd5e1 100%);
    border: 2px solid #94a3b8;
    color: #475569;
    font-weight: 900;
    font-size: 1.1rem;
    display: flex;
    align-items: center;
    justify-content: center;
    cursor: pointer;
    box-shadow: 0 2px 6px rgba(0,0,0,0.1);
}

.wii-players-status-row {
    display: flex;
    gap: 12px;
}

.player-pill {
    display: flex;
    align-items: center;
    gap: 8px;
    padding: 6px 14px;
    border-radius: 12px;
    font-size: 0.85rem;
    font-weight: 700;
    border: 2px solid;
}

.pill-online {
    background: #f0fdf4;
    border-color: #86efac;
    color: #16a34a;
}

.pill-offline {
    background: #f8fafc;
    border-color: #e2e8f0;
    color: #94a3b8;
}

/* Modals */
.wii-modal-backdrop {
    position: fixed;
    top: 0;
    left: 0;
    right: 0;
    bottom: 0;
    background: rgba(15, 23, 42, 0.6);
    backdrop-filter: blur(4px);
    display: flex;
    align-items: center;
    justify-content: center;
    z-index: 10000;
}

.wii-modal-card, .wii-channel-banner-card {
    background: white;
    padding: 24px 32px;
    border-radius: 24px;
    text-align: center;
    max-width: 480px;
    width: 90%;
    box-shadow: 0 16px 36px rgba(0,0,0,0.25);
    display: flex;
    flex-direction: column;
    align-items: center;
    gap: 14px;
}

.wii-qrcode-container {
    display: flex;
    flex-direction: column;
    align-items: center;
    gap: 8px;
}

.wii-qrcode-svg svg {
    border-radius: 12px;
    border: 3px solid #e2e8f0;
    padding: 8px;
}

.wii-qrcode-label {
    font-size: 0.85rem;
    color: #64748b;
}

.banner-header {
    display: flex;
    align-items: center;
    gap: 12px;
}

.banner-icon {
    font-size: 2.2rem;
}

.banner-subtitle {
    color: #64748b;
    font-size: 0.95rem;
}

.banner-action-row {
    display: flex;
    gap: 16px;
    width: 100%;
    margin-top: 8px;
}

.btn-wii-dialog-back {
    flex: 1;
    background: #f1f5f9;
    border: 2px solid #cbd5e1;
    color: #475569;
    font-weight: bold;
    padding: 10px;
    border-radius: 12px;
    cursor: pointer;
}

.btn-wii-dialog-start {
    flex: 1;
    background: linear-gradient(135deg, #0284c7, #0369a1);
    color: white;
    font-weight: bold;
    padding: 10px;
    border-radius: 12px;
    border: none;
    cursor: pointer;
    box-shadow: 0 4px 12px rgba(2, 132, 199, 0.3);
}

.wii-minigame-placeholder {
    background: white;
    padding: 40px;
    border-radius: 24px;
    text-align: center;
    box-shadow: 0 10px 25px rgba(0,0,0,0.1);
    display: flex;
    flex-direction: column;
    gap: 16px;
    align-items: center;
}

/* ========================================================= */
/* Wii Cursor Styles                                          */
/* ========================================================= */
.wii-cursor-wrapper {
    position: fixed;
    pointer-events: none;
    z-index: 9999;
    transition: transform 0.05s linear;
    display: flex;
    align-items: center;
    justify-content: center;
}

.wii-cursor-clicking {
    filter: brightness(1.15);
}

.wii-cursor-ripple {
    position: absolute;
    width: 60px;
    height: 60px;
    border: 3px solid;
    border-radius: 50%;
    animation: ripple-pulse 0.4s ease-out infinite;
}

@keyframes ripple-pulse {
    0% {
        transform: scale(0.5);
        opacity: 1;
    }
    100% {
        transform: scale(1.4);
        opacity: 0;
    }
}

.screen-footer {
    display: grid;
    grid-template-columns: repeat(4, 1fr);
    gap: 16px;
    padding: 12px;
    background: #ffffff;
    border-radius: 16px;
    box-shadow: 0 2px 10px rgba(0,0,0,0.04);
}

.player-slot-status {
    padding: 10px 14px;
    border-radius: 12px;
    border: 2px solid #e2e8f0;
    display: flex;
    flex-direction: column;
    gap: 4px;
    font-size: 0.85rem;
}

.slot-online {
    background: #f0fdf4;
    border-color: #86efac;
    color: #15803d;
}

.slot-offline {
    background: #f8fafc;
    border-color: #e2e8f0;
    color: #94a3b8;
}

.slot-header {
    display: flex;
    justify-content: space-between;
    align-items: center;
}

.slot-orientation, .slot-button {
    font-size: 0.75rem;
    color: #334155;
    font-weight: 500;
}

.player-slot-badge {
    background: linear-gradient(135deg, #0284c7, #0369a1);
    color: white;
    font-weight: 900;
    font-size: 1rem;
    padding: 6px 16px;
    border-radius: 20px;
    box-shadow: 0 3px 8px rgba(2,132,199,0.3);
    letter-spacing: 1px;
}

.wiimote-top-tags {
    display: flex;
    align-items: center;
    gap: 8px;
}

/* ========================================================= */
/* Mobile Remote (Wiimote) Styles                             */
/* ========================================================= */
.wiimote-page {
    display: flex;
    flex-direction: column;
    min-height: 100vh;
    max-width: 420px;
    margin: 0 auto;
    background: #f1f5f9;
    padding: 12px;
    gap: 12px;
}

.wiimote-top-bar {
    display: flex;
    justify-content: space-between;
    align-items: center;
    background: white;
    padding: 10px 16px;
    border-radius: 12px;
    box-shadow: 0 2px 8px rgba(0,0,0,0.04);
}

.badge-room {
    background: #0284c7;
    color: white;
    font-weight: bold;
    font-size: 0.85rem;
    padding: 4px 10px;
    border-radius: 20px;
}

.sensor-card {
    background: white;
    border-radius: 16px;
    padding: 12px 16px;
    text-align: center;
    box-shadow: 0 2px 8px rgba(0,0,0,0.05);
    display: flex;
    flex-direction: column;
    gap: 8px;
}

.status-text {
    font-size: 0.85rem;
    color: #475569;
}

.btn-sensor-activate {
    background: linear-gradient(135deg, #0284c7, #0369a1);
    color: white;
    border: none;
    padding: 12px;
    border-radius: 10px;
    font-weight: bold;
    font-size: 0.95rem;
    cursor: pointer;
    box-shadow: 0 4px 10px rgba(2,132,199,0.3);
}

.btn-sensor-activate:active {
    transform: scale(0.98);
}

.telemetry-panel {
    display: flex;
    justify-content: space-around;
    background: #f8fafc;
    border: 1px solid #e2e8f0;
    border-radius: 8px;
    padding: 6px;
    font-size: 0.8rem;
}

.telemetry-item b {
    color: #0284c7;
    margin-left: 4px;
}

.btn-recenter {
    background: #f8fafc;
    border: 2px solid #0284c7;
    color: #0284c7;
    padding: 8px;
    border-radius: 8px;
    font-weight: bold;
    font-size: 0.9rem;
    cursor: pointer;
}

.btn-recenter:active {
    background: #0284c7;
    color: white;
}

/* Wiimote Physical Chassis */
.wiimote-chassis {
    background: linear-gradient(180deg, #ffffff 0%, #e2e8f0 100%);
    border: 4px solid #cbd5e1;
    border-radius: 36px;
    padding: 24px 16px;
    display: flex;
    flex-direction: column;
    align-items: center;
    gap: 24px;
    box-shadow: inset 0 2px 4px rgba(255,255,255,0.8), 0 10px 25px rgba(0,0,0,0.1);
}

/* D-Pad */
.wiimote-dpad {
    display: flex;
    flex-direction: column;
    align-items: center;
    gap: 2px;
}

.dpad-middle-row {
    display: flex;
    align-items: center;
    gap: 2px;
}

.dpad-btn {
    width: 52px;
    height: 52px;
    background: #334155;
    color: #94a3b8;
    border: none;
    font-size: 1.2rem;
    display: flex;
    align-items: center;
    justify-content: center;
    cursor: pointer;
    touch-action: none;
    box-shadow: inset 0 2px 3px rgba(255,255,255,0.15), 0 3px 6px rgba(0,0,0,0.3);
}

.dpad-up { border-radius: 10px 10px 0 0; }
.dpad-down { border-radius: 0 0 10px 10px; }
.dpad-left { border-radius: 10px 0 0 10px; }
.dpad-right { border-radius: 0 10px 10px 0; }

.dpad-core {
    width: 52px;
    height: 52px;
    background: #334155;
}

.dpad-btn:active {
    background: #0f172a;
    color: #38bdf8;
    box-shadow: inset 0 3px 6px rgba(0,0,0,0.6);
}

/* A & B Action Buttons */
.wiimote-actions {
    display: flex;
    align-items: center;
    gap: 20px;
}

.btn-wii-a-tactile {
    width: 80px;
    height: 80px;
    border-radius: 50%;
    background: radial-gradient(circle at 35% 35%, #ffffff 0%, #cbd5e1 70%, #94a3b8 100%);
    border: 3px solid #94a3b8;
    color: #334155;
    font-size: 1.8rem;
    font-weight: 900;
    cursor: pointer;
    box-shadow: 0 6px 12px rgba(0,0,0,0.2), inset 0 2px 4px rgba(255,255,255,0.9);
    touch-action: none;
}

.btn-wii-a-tactile:active {
    transform: scale(0.94);
    background: #94a3b8;
    color: #ffffff;
    box-shadow: inset 0 4px 8px rgba(0,0,0,0.4);
}

.btn-wii-b-tactile {
    padding: 0 20px;
    height: 60px;
    border-radius: 20px;
    background: linear-gradient(180deg, #64748b 0%, #475569 100%);
    border: 2px solid #334155;
    color: white;
    font-size: 1.1rem;
    font-weight: bold;
    cursor: pointer;
    box-shadow: 0 6px 12px rgba(0,0,0,0.2);
    touch-action: none;
}

.btn-wii-b-tactile:active {
    transform: scale(0.94);
    background: #1e293b;
    box-shadow: inset 0 4px 8px rgba(0,0,0,0.5);
}

/* Middle Row (-, Home, +) */
.wiimote-middle-row {
    display: flex;
    align-items: center;
    gap: 18px;
}

.btn-wii-symbol {
    width: 38px;
    height: 38px;
    border-radius: 50%;
    background: #f1f5f9;
    border: 2px solid #cbd5e1;
    color: #64748b;
    font-size: 1.3rem;
    font-weight: 900;
    cursor: pointer;
    box-shadow: 0 3px 6px rgba(0,0,0,0.1);
    touch-action: none;
}

.btn-wii-symbol:active {
    background: #cbd5e1;
    transform: scale(0.92);
}

.btn-wii-home-tactile {
    width: 48px;
    height: 48px;
    border-radius: 50%;
    background: #0284c7;
    border: 2px solid #0369a1;
    color: white;
    font-size: 1.4rem;
    cursor: pointer;
    box-shadow: 0 4px 8px rgba(2,132,199,0.3);
    touch-action: none;
}

.btn-wii-home-tactile:active {
    background: #075985;
    transform: scale(0.92);
}

/* 1 and 2 Buttons */
.wiimote-bottom-row {
    display: flex;
    gap: 20px;
}

.btn-wii-num-tactile {
    width: 54px;
    height: 54px;
    border-radius: 14px;
    background: #f8fafc;
    border: 2px solid #cbd5e1;
    color: #475569;
    font-size: 1.3rem;
    font-weight: bold;
    cursor: pointer;
    box-shadow: 0 4px 8px rgba(0,0,0,0.1);
    touch-action: none;
}

.btn-wii-num-tactile:active {
    background: #cbd5e1;
    transform: scale(0.92);
}

/* LEDs */
.wiimote-led-row {
    display: flex;
    gap: 12px;
    margin-top: 8px;
}

.led-indicator {
    width: 10px;
    height: 10px;
    border-radius: 50%;
    background: #94a3b8;
    box-shadow: inset 0 1px 2px rgba(0,0,0,0.3);
}

.led-active {
    background: #38bdf8;
    box-shadow: 0 0 8px #38bdf8, inset 0 1px 2px rgba(255,255,255,0.8);
}

.wiimote-footer-status {
    text-align: center;
    font-size: 0.85rem;
    color: #64748b;
    background: white;
    padding: 8px;
    border-radius: 8px;
}
"#;

#[component]
pub fn App() -> Element {
    rsx! {
        style { "{MAIN_CSS}" }
        Router::<Route> {}
    }
}
