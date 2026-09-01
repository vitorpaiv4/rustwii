use dioxus::prelude::*;
use crate::route::Route;

const MAIN_CSS: &str = r#"
* {
    box-sizing: border-box;
    margin: 0;
    padding: 0;
    font-family: system-ui, -apple-system, BlinkMacSystemFont, 'Segoe UI', Roboto, Oxygen, Ubuntu, Cantarell, sans-serif;
    user-select: none;
    -webkit-user-select: none;
}

body {
    background-color: #e6ecf0;
    color: #333333;
    overflow-x: hidden;
    min-height: 100vh;
}

/* Screen View (PC/TV) Styles */
.screen-container {
    display: flex;
    flex-direction: column;
    height: 100vh;
    justify-content: space-between;
    padding: 24px;
    background: radial-gradient(circle at center, #ffffff 0%, #dfe7ed 100%);
}

.screen-header {
    display: flex;
    justify-content: space-between;
    align-items: center;
    border-bottom: 2px solid #00a8e8;
    padding-bottom: 12px;
}

.room-badge {
    background: #00a8e8;
    color: #ffffff;
    padding: 4px 10px;
    border-radius: 12px;
    font-weight: bold;
}

.screen-content {
    display: flex;
    flex-direction: column;
    align-items: center;
    gap: 20px;
}

.pairing-info {
    text-align: center;
    background: white;
    padding: 16px 24px;
    border-radius: 16px;
    box-shadow: 0 4px 12px rgba(0,0,0,0.08);
}

.pairing-info code {
    display: inline-block;
    margin-top: 8px;
    font-size: 1.2rem;
    font-weight: bold;
    color: #0077b6;
    background: #edf2f7;
    padding: 6px 12px;
    border-radius: 8px;
}

.wii-grid-placeholder {
    display: grid;
    grid-template-columns: repeat(2, 200px);
    gap: 16px;
}

.wii-channel-slot {
    background: #ffffff;
    border: 2px solid #b0bec5;
    border-radius: 16px;
    height: 120px;
    display: flex;
    align-items: center;
    justify-content: center;
    text-align: center;
    padding: 12px;
    font-weight: bold;
    box-shadow: 0 2px 6px rgba(0,0,0,0.05);
    transition: transform 0.2s ease, border-color 0.2s ease;
}

.wii-channel-slot:hover {
    border-color: #00a8e8;
    transform: scale(1.03);
}

.screen-footer {
    display: flex;
    justify-content: space-around;
    padding: 12px;
    background: #ffffff;
    border-radius: 12px;
    font-size: 0.9rem;
    color: #64748b;
    box-shadow: 0 2px 8px rgba(0,0,0,0.04);
}

/* Remote View (Smartphone) Styles */
.remote-container {
    display: flex;
    flex-direction: column;
    min-height: 100vh;
    max-width: 400px;
    margin: 0 auto;
    background: #f8fafc;
    padding: 16px;
    touch-action: manipulation;
}

.remote-header {
    text-align: center;
    margin-bottom: 16px;
}

.room-tag {
    display: inline-block;
    background: #e2e8f0;
    color: #475569;
    padding: 2px 8px;
    border-radius: 6px;
    font-size: 0.85rem;
    margin-top: 4px;
}

.remote-body {
    display: flex;
    flex-direction: column;
    align-items: center;
    gap: 20px;
}

.sensor-status {
    width: 100%;
    text-align: center;
    background: #ffffff;
    padding: 12px;
    border-radius: 12px;
    box-shadow: 0 2px 6px rgba(0,0,0,0.06);
}

.btn-primary {
    margin-top: 8px;
    background: #00a8e8;
    color: white;
    border: none;
    padding: 10px 16px;
    border-radius: 8px;
    font-weight: bold;
    cursor: pointer;
    width: 100%;
}

.remote-dpad-section {
    display: flex;
    flex-direction: column;
    align-items: center;
    gap: 4px;
}

.dpad-row {
    display: flex;
    align-items: center;
    gap: 4px;
}

.dpad-btn, .dpad-center {
    width: 48px;
    height: 48px;
    background: #334155;
    color: white;
    display: flex;
    align-items: center;
    justify-content: center;
    border-radius: 8px;
    font-size: 1.2rem;
}

.dpad-center {
    background: #1e293b;
}

.remote-action-buttons {
    display: flex;
    gap: 16px;
    width: 100%;
    justify-content: center;
}

.btn-wii-a {
    width: 70px;
    height: 70px;
    border-radius: 50%;
    background: #3b82f6;
    color: white;
    font-size: 1.4rem;
    font-weight: bold;
    border: 3px solid #1d4ed8;
    box-shadow: 0 4px 6px rgba(0,0,0,0.15);
}

.btn-wii-b {
    padding: 0 16px;
    height: 70px;
    border-radius: 16px;
    background: #64748b;
    color: white;
    font-size: 1rem;
    font-weight: bold;
    border: none;
    box-shadow: 0 4px 6px rgba(0,0,0,0.15);
}

.remote-meta-buttons {
    display: flex;
    align-items: center;
    gap: 12px;
}

.btn-wii-small {
    width: 40px;
    height: 40px;
    border-radius: 50%;
    background: #e2e8f0;
    border: 1px solid #cbd5e1;
    font-size: 1.2rem;
    font-weight: bold;
}

.btn-wii-home {
    width: 48px;
    height: 48px;
    border-radius: 50%;
    background: #38bdf8;
    color: white;
    border: none;
    font-size: 1.3rem;
}

.remote-numpad-buttons {
    display: flex;
    gap: 16px;
}

.btn-wii-num {
    width: 50px;
    height: 50px;
    border-radius: 12px;
    background: #e2e8f0;
    border: 2px solid #cbd5e1;
    font-size: 1.2rem;
    font-weight: bold;
}
"#;

#[component]
pub fn App() -> Element {
    rsx! {
        style { "{MAIN_CSS}" }
        Router::<Route> {}
    }
}
