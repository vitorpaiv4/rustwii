#[cfg(target_arch = "wasm32")]
pub fn trigger_haptic(duration_ms: u32) {
    if let Some(window) = web_sys::window() {
        let nav = window.navigator();
        let _ = nav.vibrate_with_duration(duration_ms);
    }
}

#[cfg(not(target_arch = "wasm32"))]
pub fn trigger_haptic(_duration_ms: u32) {
    // No-op for non-WASM targets
}

/// Short haptic pulse for standard button tap (20ms)
pub fn haptic_tap() {
    trigger_haptic(20);
}

/// Medium haptic feedback for special buttons (Home, Calibration, etc - 45ms)
pub fn haptic_medium() {
    trigger_haptic(45);
}

/// Stronger rumble feedback (e.g. B trigger or shake - 90ms)
pub fn haptic_rumble() {
    trigger_haptic(90);
}
