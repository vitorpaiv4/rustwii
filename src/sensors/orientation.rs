#[cfg(target_arch = "wasm32")]
use crate::types::OrientationSample;

#[cfg(target_arch = "wasm32")]
pub mod wasm {
    use super::*;
    use wasm_bindgen::prelude::*;
    use wasm_bindgen::JsCast;

    #[wasm_bindgen(inline_js = r#"
    export function init_openwii_controller_sensors(onSampleCallback) {
        console.log("[OpenWii-Rust] Initializing high-frequency controller sensors...");

        var streaming = false;
        var rawEvents = 0;
        var sensorEvents = 0;
        var orientationSource = null;
        var latestOrientation = null;
        var latestMotion = null;
        var lastEmit = 0;
        var MIN_EMIT_MS = 6; // ~160Hz ceiling
        var genericSensor = null;
        var usingGenericSensor = false;

        function flush() {
            if (!streaming || (!latestOrientation && !latestMotion)) return;
            var now = performance.now();
            if (now - lastEmit < MIN_EMIT_MS) return;
            lastEmit = now;

            var payload = {
                alpha: null,
                beta: null,
                gamma: null,
                heading: null,
                quat: null,
                motion: null,
                t: now
            };

            if (latestOrientation) {
                if (latestOrientation.quat) {
                    payload.quat = latestOrientation.quat;
                } else {
                    payload.alpha = latestOrientation.alpha;
                    payload.beta = latestOrientation.beta;
                    payload.gamma = latestOrientation.gamma;
                    payload.heading = latestOrientation.heading;
                }
            }

            if (latestMotion) {
                payload.motion = {
                    ax: latestMotion.ax || 0,
                    ay: latestMotion.ay || 0,
                    az: latestMotion.az || 0,
                    rx: latestMotion.rx || 0,
                    ry: latestMotion.ry || 0,
                    rz: latestMotion.rz || 0
                };
            }

            try {
                onSampleCallback(JSON.stringify(payload));
            } catch (err) {
                console.error("[OpenWii-Rust] onSampleCallback error:", err);
            }
        }

        function onOrientation(e) {
            rawEvents++;
            if (e.alpha === null && e.beta === null && e.gamma === null) return;

            // Interleaving prevention: prefer absolute if available
            if (e.type === 'deviceorientationabsolute') orientationSource = e.type;
            else if (orientationSource === null) orientationSource = e.type;
            if (e.type !== orientationSource) return;

            sensorEvents++;
            latestOrientation = {
                alpha: e.alpha || 0,
                beta: e.beta || 0,
                gamma: e.gamma || 0,
                heading: typeof e.webkitCompassHeading === 'number' ? e.webkitCompassHeading : null
            };
            flush();
        }

        function onMotion(e) {
            var a = e.acceleration || e.accelerationIncludingGravity;
            var r = e.rotationRate;
            if (!a && !r) return;

            latestMotion = {
                ax: a ? a.x || 0 : 0,
                ay: a ? a.y || 0 : 0,
                az: a ? a.z || 0 : 0,
                rz: r ? r.alpha || 0 : 0,
                rx: r ? r.beta || 0 : 0,
                ry: r ? r.gamma || 0 : 0,
            };
            flush();
        }

        function startGenericSensor() {
            var Ctor = window.AbsoluteOrientationSensor || window.RelativeOrientationSensor;
            if (!Ctor) return false;
            try {
                genericSensor = new Ctor({ frequency: 120, referenceFrame: 'device' });
                genericSensor.addEventListener('reading', function() {
                    var q = genericSensor.quaternion;
                    if (!q) return;
                    rawEvents++;
                    sensorEvents++;
                    usingGenericSensor = true;
                    latestOrientation = { quat: [q[0], q[1], q[2], q[3]] };
                    flush();
                });
                genericSensor.addEventListener('error', function(ev) {
                    console.warn("[OpenWii-Rust] Generic sensor error:", ev);
                });
                genericSensor.start();
                return true;
            } catch (err) {
                console.warn("[OpenWii-Rust] GenericSensor init error:", err);
                return false;
            }
        }

        // Attach DOM listeners with capture
        window.addEventListener('deviceorientation', onOrientation, true);
        window.addEventListener('deviceorientationabsolute', onOrientation, true);
        window.addEventListener('devicemotion', onMotion, true);

        // Fallback watchdog after 1500ms if no events arrived
        setTimeout(function() {
            if (rawEvents === 0) {
                startGenericSensor();
            }
        }, 1500);

        // WakeLock to keep screen on while swinging
        try {
            if ('wakeLock' in navigator) {
                navigator.wakeLock.request('screen').catch(function() {});
            }
        } catch (e) {}

        // Screen orientation lock
        try {
            if (screen.orientation && screen.orientation.lock) {
                screen.orientation.lock('portrait').catch(function() {});
            }
        } catch (e) {}

        streaming = true;
        console.log("[OpenWii-Rust] Sensor streaming active!");
    }
    "#)]
    extern "C" {
        fn init_openwii_controller_sensors(on_sample: &js_sys::Function);
    }

    /// Checks if running under a Secure Context (HTTPS or localhost)
    pub fn is_secure_context() -> bool {
        if let Some(window) = web_sys::window() {
            return window.is_secure_context();
        }
        false
    }

    /// Requests permission on iOS 13+ if necessary, otherwise resolves immediately
    pub async fn request_sensor_permission() -> Result<bool, JsValue> {
        let window = web_sys::window().ok_or_else(|| JsValue::from_str("No window found"))?;

        // 1. Check DeviceOrientationEvent.requestPermission (iOS 13+ Safari)
        if let Ok(constructor) = js_sys::Reflect::get(&window, &JsValue::from_str("DeviceOrientationEvent")) {
            if !constructor.is_undefined() && !constructor.is_null() {
                if let Ok(request_permission_fn) = js_sys::Reflect::get(&constructor, &JsValue::from_str("requestPermission")) {
                    if request_permission_fn.is_function() {
                        let func = request_permission_fn.dyn_into::<js_sys::Function>()?;
                        let promise = func.call0(&constructor)?.dyn_into::<js_sys::Promise>()?;
                        let result = wasm_bindgen_futures::JsFuture::from(promise).await?;
                        if let Some(res_str) = result.as_string() {
                            if res_str != "granted" {
                                return Ok(false);
                            }
                        }
                    }
                }
            }
        }

        // 2. Check DeviceMotionEvent.requestPermission (iOS 13+ Safari)
        if let Ok(constructor) = js_sys::Reflect::get(&window, &JsValue::from_str("DeviceMotionEvent")) {
            if !constructor.is_undefined() && !constructor.is_null() {
                if let Ok(request_permission_fn) = js_sys::Reflect::get(&constructor, &JsValue::from_str("requestPermission")) {
                    if request_permission_fn.is_function() {
                        let func = request_permission_fn.dyn_into::<js_sys::Function>()?;
                        let promise = func.call0(&constructor)?.dyn_into::<js_sys::Promise>()?;
                        let _ = wasm_bindgen_futures::JsFuture::from(promise).await;
                    }
                }
            }
        }

        Ok(true)
    }

    /// Starts streaming controller samples
    pub fn start_controller_streaming<F>(mut on_sample: F) -> Result<Closure<dyn FnMut(String)>, JsValue>
    where
        F: FnMut(OrientationSample) + 'static,
    {
        let closure = Closure::wrap(Box::new(move |json_str: String| {
            if let Ok(sample) = serde_json::from_str::<OrientationSample>(&json_str) {
                on_sample(sample);
            }
        }) as Box<dyn FnMut(String)>);

        init_openwii_controller_sensors(closure.as_ref().unchecked_ref());

        Ok(closure)
    }
}
