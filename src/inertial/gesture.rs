use std::f64::consts::PI;

/// Swing / Slash detection
#[derive(Debug, Clone, PartialEq)]
pub struct SwingDetector {
    pub on_threshold: f64,  // deg/s to arm a swing (default 150)
    pub off_threshold: f64, // deg/s at which the swing has ended (default 60)
    pub min_travel: f64,    // degrees of angular travel to count at all (default 25)
    pub max_ms: f64,        // max duration (default 500ms)
    pub active: bool,
    pub started_at: f64,
    pub travel: f64,
    pub peak: f64,
    pub vx: f64,
    pub vy: f64,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct SwingResult {
    pub angle: f64, // atan2(vy, vx) in radians
    pub peak: f64,
    pub travel: f64,
    pub duration_ms: f64,
}

impl Default for SwingDetector {
    fn default() -> Self {
        Self::new(150.0, 60.0, 25.0, 500.0)
    }
}

impl SwingDetector {
    pub fn new(on_threshold: f64, off_threshold: f64, min_travel: f64, max_ms: f64) -> Self {
        Self {
            on_threshold,
            off_threshold,
            min_travel,
            max_ms,
            active: false,
            started_at: 0.0,
            travel: 0.0,
            peak: 0.0,
            vx: 0.0,
            vy: 0.0,
        }
    }

    /// Feeds angular rates (yaw and pitch in deg/s). Returns SwingResult when completed.
    pub fn update(&mut self, yaw: f64, pitch: f64, dt: f64, now_ms: f64) -> Option<SwingResult> {
        let mag = (yaw * yaw + pitch * pitch).sqrt();
        let cx = -yaw;
        let cy = -pitch;

        if !self.active {
            if mag >= self.on_threshold {
                self.active = true;
                self.started_at = now_ms;
                self.travel = mag * dt;
                self.peak = mag;
                self.vx = cx * dt;
                self.vy = cy * dt;
            }
            return None;
        }

        self.travel += mag * dt;
        self.peak = self.peak.max(mag);
        self.vx += cx * dt;
        self.vy += cy * dt;

        let too_long = (now_ms - self.started_at) > self.max_ms;
        if mag < self.off_threshold || too_long {
            self.active = false;
            if self.travel >= self.min_travel && !too_long {
                return Some(SwingResult {
                    angle: self.vy.atan2(self.vx),
                    peak: self.peak,
                    travel: self.travel,
                    duration_ms: now_ms - self.started_at,
                });
            }
        }
        None
    }
}

/// Shortest difference between two angles in radians (-PI, PI]
pub fn angle_diff(a: f64, b: f64) -> f64 {
    let mut d = a - b;
    while d > PI {
        d -= 2.0 * PI;
    }
    while d <= -PI {
        d += 2.0 * PI;
    }
    d
}
