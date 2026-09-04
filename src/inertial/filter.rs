use crate::inertial::orientation::TAU;

/// One Euro filter — adaptive low-pass filter
/// Heavy smoothing when nearly still (kills IMU jitter), almost none when moving fast (keeps swings crisp).
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct OneEuro {
    pub min_cutoff: f64,
    pub beta: f64,
    pub d_cutoff: f64,
    pub x: Option<f64>,
    pub dx: f64,
}

impl Default for OneEuro {
    fn default() -> Self {
        Self::new(1.4, 0.05, 1.0)
    }
}

impl OneEuro {
    pub fn new(min_cutoff: f64, beta: f64, d_cutoff: f64) -> Self {
        Self {
            min_cutoff,
            beta,
            d_cutoff,
            x: None,
            dx: 0.0,
        }
    }

    fn alpha(cutoff: f64, dt: f64) -> f64 {
        let tau = 1.0 / (TAU * cutoff);
        1.0 / (1.0 + tau / dt)
    }

    pub fn filter(&mut self, value: f64, dt: f64) -> f64 {
        if dt <= 0.0 {
            return self.x.unwrap_or(value);
        }
        let prev_x = match self.x {
            Some(x) => x,
            None => {
                self.x = Some(value);
                return value;
            }
        };

        let d_raw = (value - prev_x) / dt;
        self.dx += Self::alpha(self.d_cutoff, dt) * (d_raw - self.dx);
        let cutoff = self.min_cutoff + self.beta * self.dx.abs();
        let filtered_x = prev_x + Self::alpha(cutoff, dt) * (value - prev_x);
        self.x = Some(filtered_x);
        filtered_x
    }

    pub fn reset(&mut self) {
        self.x = None;
        self.dx = 0.0;
    }
}
