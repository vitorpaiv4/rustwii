#[cfg(target_arch = "wasm32")]
pub mod wasm {
    use wasm_bindgen::JsValue;
    use web_sys::{AudioContext, OscillatorType};

    thread_local! {
        static AUDIO_CTX: Result<AudioContext, JsValue> = AudioContext::new();
    }

    pub fn unlock_audio() {
        AUDIO_CTX.with(|ctx_res| {
            if let Ok(ctx) = ctx_res {
                let _ = ctx.resume();
            }
        });
    }

    pub fn play_tone(freq_start: f32, freq_end: f32, duration_secs: f64, gain_vol: f32) {
        AUDIO_CTX.with(|ctx_res| {
            if let Ok(ctx) = ctx_res
                && let (Ok(osc), Ok(gain)) = (ctx.create_oscillator(), ctx.create_gain()) {
                    let now = ctx.current_time();

                    osc.set_type(OscillatorType::Sine);
                    let _ = osc.frequency().set_value_at_time(freq_start, now);
                    let _ = osc.frequency().exponential_ramp_to_value_at_time(freq_end, now + duration_secs);

                    let _ = gain.gain().set_value_at_time(gain_vol, now);
                    let _ = gain.gain().exponential_ramp_to_value_at_time(0.0001, now + duration_secs);

                    let _ = osc.connect_with_audio_node(&gain);
                    let _ = gain.connect_with_audio_node(&ctx.destination());

                    let _ = osc.start();
                    let _ = osc.stop_with_when(now + duration_secs);
                }
        });
    }

    pub fn play_swoosh() {
        AUDIO_CTX.with(|ctx_res| {
            if let Ok(ctx) = ctx_res
                && let (Ok(osc), Ok(gain)) = (ctx.create_oscillator(), ctx.create_gain()) {
                    let now = ctx.current_time();
                    osc.set_type(OscillatorType::Triangle);
                    let _ = osc.frequency().set_value_at_time(1200.0, now);
                    let _ = osc.frequency().exponential_ramp_to_value_at_time(300.0, now + 0.12);

                    let _ = gain.gain().set_value_at_time(0.15, now);
                    let _ = gain.gain().exponential_ramp_to_value_at_time(0.0001, now + 0.12);

                    let _ = osc.connect_with_audio_node(&gain);
                    let _ = gain.connect_with_audio_node(&ctx.destination());

                    let _ = osc.start();
                    let _ = osc.stop_with_when(now + 0.13);
                }
        });
    }

    pub fn play_thud() {
        play_tone(160.0, 50.0, 0.25, 0.25);
    }
}

pub fn unlock_audio() {
    #[cfg(target_arch = "wasm32")]
    wasm::unlock_audio();
}

/// Plays a soft pop when hovering over a channel
pub fn play_hover() {
    #[cfg(target_arch = "wasm32")]
    wasm::play_tone(580.0, 720.0, 0.05, 0.08);
}

/// Plays a distinct click sound when pressing A
pub fn play_click() {
    #[cfg(target_arch = "wasm32")]
    wasm::play_tone(880.0, 1174.0, 0.09, 0.15);
}

/// Plays a cancellation / back sound
pub fn play_back() {
    #[cfg(target_arch = "wasm32")]
    wasm::play_tone(600.0, 420.0, 0.08, 0.12);
}

/// Plays a game start chime
pub fn play_start_chime() {
    #[cfg(target_arch = "wasm32")]
    wasm::play_tone(523.25, 659.25, 0.12, 0.12);
}

/// Plays a sword / slice swoosh sound
pub fn play_swoosh() {
    #[cfg(target_arch = "wasm32")]
    wasm::play_swoosh();
}

/// Plays a bomb thud sound
pub fn play_thud() {
    #[cfg(target_arch = "wasm32")]
    wasm::play_thud();
}
