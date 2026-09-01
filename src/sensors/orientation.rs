use crate::types::OrientationData;

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct CalibrationOffset {
    pub alpha_offset: f64,
    pub beta_offset: f64,
    pub gamma_offset: f64,
}

impl Default for CalibrationOffset {
    fn default() -> Self {
        Self {
            alpha_offset: 0.0,
            beta_offset: 0.0,
            gamma_offset: 0.0,
        }
    }
}

impl CalibrationOffset {
    /// Sets the offset from the current orientation reading
    pub fn calibrate_from(&mut self, current: OrientationData) {
        self.alpha_offset = current.alpha;
        self.beta_offset = current.beta;
        self.gamma_offset = current.gamma;
    }

    /// Normalizes raw orientation with calibrated offsets
    pub fn apply(&self, raw: OrientationData) -> OrientationData {
        // Delta alpha: wrap within [-180, 180]
        let mut diff_alpha = raw.alpha - self.alpha_offset;
        while diff_alpha > 180.0 {
            diff_alpha -= 360.0;
        }
        while diff_alpha < -180.0 {
            diff_alpha += 360.0;
        }

        let diff_beta = raw.beta - self.beta_offset;
        let diff_gamma = raw.gamma - self.gamma_offset;

        OrientationData {
            alpha: diff_alpha,
            beta: diff_beta,
            gamma: diff_gamma,
        }
    }
}

#[cfg(target_arch = "wasm32")]
pub mod wasm {
    use super::*;
    use wasm_bindgen::prelude::*;
    use wasm_bindgen::JsCast;
    use web_sys::{DeviceOrientationEvent, Event};

    /// Requests permission on iOS 13+ if necessary, otherwise resolves immediately
    pub async fn request_sensor_permission() -> Result<bool, JsValue> {
        let window = web_sys::window().ok_or_else(|| JsValue::from_str("No window found"))?;

        // Check if DeviceOrientationEvent has requestPermission method (iOS 13+)
        let device_orientation_constructor = js_sys::Reflect::get(&window, &JsValue::from_str("DeviceOrientationEvent"))?;
        if !device_orientation_constructor.is_undefined() {
            let request_permission_fn = js_sys::Reflect::get(&device_orientation_constructor, &JsValue::from_str("requestPermission"))?;
            if request_permission_fn.is_function() {
                let func = request_permission_fn.dyn_into::<js_sys::Function>()?;
                let promise = func.call0(&device_orientation_constructor)?.dyn_into::<js_sys::Promise>()?;
                let result = wasm_bindgen_futures::JsFuture::from(promise).await?;
                if let Some(res_str) = result.as_string() {
                    return Ok(res_str == "granted");
                }
            }
        }

        // Android / desktop doesn't require explicit requestPermission
        Ok(true)
    }

    /// Attaches the `deviceorientation` listener to window and calls `on_data` for each sensor update
    pub fn start_orientation_listener<F>(mut on_data: F) -> Result<Closure<dyn FnMut(Event)>, JsValue>
    where
        F: FnMut(OrientationData) + 'static,
    {
        let window = web_sys::window().ok_or_else(|| JsValue::from_str("No window found"))?;

        let closure = Closure::wrap(Box::new(move |event: Event| {
            if let Ok(orientation_event) = event.dyn_into::<DeviceOrientationEvent>() {
                let alpha = orientation_event.alpha().unwrap_or(0.0);
                let beta = orientation_event.beta().unwrap_or(0.0);
                let gamma = orientation_event.gamma().unwrap_or(0.0);

                let data = OrientationData { alpha, beta, gamma };
                on_data(data);
            }
        }) as Box<dyn FnMut(Event)>);

        window.add_event_listener_with_callback("deviceorientation", closure.as_ref().unchecked_ref())?;

        Ok(closure)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_calibration_offset_application() {
        let mut offset = CalibrationOffset::default();
        let initial = OrientationData {
            alpha: 100.0,
            beta: 45.0,
            gamma: 10.0,
        };

        offset.calibrate_from(initial);
        let calibrated = offset.apply(initial);

        assert_eq!(calibrated.alpha, 0.0);
        assert_eq!(calibrated.beta, 0.0);
        assert_eq!(calibrated.gamma, 0.0);
    }

    #[test]
    fn test_calibration_wrapping() {
        let mut offset = CalibrationOffset::default();
        offset.calibrate_from(OrientationData {
            alpha: 350.0,
            beta: 0.0,
            gamma: 0.0,
        });

        let raw = OrientationData {
            alpha: 10.0,
            beta: 0.0,
            gamma: 0.0,
        };

        let calibrated = offset.apply(raw);
        // 10.0 - 350.0 = -340.0 -> wrapped is +20.0
        assert_eq!(calibrated.alpha, 20.0);
    }
}
