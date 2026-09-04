use crate::inertial::orientation::{
    axes_from_sample, clamp, BodyAxes, Vec3, DEG,
};
use crate::types::OrientationSample;

pub const STEADY_MS: f64 = 1200.0;
pub const STEADY_KEEP_MS: f64 = STEADY_MS * 1.6;
pub const STEADY_GIVE_UP_MS: f64 = 6000.0;

/// Reference frame built around player's neutral grip
#[derive(Debug, Clone, PartialEq)]
pub struct CalibFrame {
    pub f: Vec3,
    pub u: Vec3,
    pub r: Vec3,
    pub axis: char, // 'y' (top edge) or 'z' (back)
    pub yaw0: f64,
    pub pitch0: f64,
}

/// Builds an orthonormal frame around whichever axis is closest to horizontal
pub fn build_frame(axes: &BodyAxes) -> Option<CalibFrame> {
    let use_top_edge = axes.y.z.abs() <= axes.z.z.abs();
    let f = if use_top_edge {
        axes.y
    } else {
        axes.z.scale(-1.0)
    };

    let world_up = Vec3::new(0.0, 0.0, 1.0);
    let u_raw = world_up.sub(&f.scale(f.z));
    let u_len = u_raw.length();
    if u_len < 1e-3 {
        return None; // Pointing vertically
    }
    let u = u_raw.scale(1.0 / u_len);
    let r = f.cross(&u);

    Some(CalibFrame {
        f,
        u,
        r,
        axis: if use_top_edge { 'y' } else { 'z' },
        yaw0: 0.0,
        pitch0: 0.0,
    })
}

pub fn forward_of(frame: &CalibFrame, axes: &BodyAxes) -> Vec3 {
    if frame.axis == 'y' {
        axes.y
    } else {
        axes.z.scale(-1.0)
    }
}

/// Yaw/pitch of the phone measured inside a calibrated frame (degrees)
pub fn angles_in(frame: &CalibFrame, fwd: &Vec3) -> (f64, f64) {
    let yaw = fwd.dot(&frame.r).atan2(fwd.dot(&frame.f)) / DEG - frame.yaw0;
    let pitch = clamp(fwd.dot(&frame.u), -1.0, 1.0).asin() / DEG - frame.pitch0;
    (yaw, pitch)
}

#[derive(Debug, Clone, PartialEq)]
pub struct CalibrationResult {
    pub deg_per_screen_x: f64,
    pub deg_per_screen_y: f64,
    pub grip: char,
    pub noise_deg: f64,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CalibStep {
    Signal,
    Steady,
    Range,
    Done,
}

#[derive(Debug, Clone, PartialEq)]
pub struct Calibration {
    pub active: bool,
    pub done: bool,
    pub step: CalibStep,
    pub step_since: f64,
    pub steady_buf: Vec<(f64, Vec3, Vec3)>, // (t, y, z)
    pub frame: Option<CalibFrame>,
    pub yaw_min: f64,
    pub yaw_max: f64,
    pub pitch_min: f64,
    pub pitch_max: f64,
    pub noise_deg: f64,
    pub last_span: f64,
    pub span_still_since: f64,
    pub result: Option<CalibrationResult>,
}

impl Default for Calibration {
    fn default() -> Self {
        Self::new()
    }
}

impl Calibration {
    pub fn new() -> Self {
        Self {
            active: false,
            done: false,
            step: CalibStep::Signal,
            step_since: 0.0,
            steady_buf: Vec::new(),
            frame: None,
            yaw_min: 0.0,
            yaw_max: 0.0,
            pitch_min: 0.0,
            pitch_max: 0.0,
            noise_deg: 0.0,
            last_span: 0.0,
            span_still_since: 0.0,
            result: None,
        }
    }

    pub fn start(&mut self, now: f64) {
        self.active = true;
        self.done = false;
        self.frame = None;
        self.steady_buf.clear();
        self.step = CalibStep::Signal;
        self.step_since = now;
    }

    pub fn advance(&mut self, sample: &OrientationSample, now: f64) -> Option<&CalibFrame> {
        let axes = axes_from_sample(sample);
        if !self.active {
            return self.frame.as_ref();
        }

        match self.step {
            CalibStep::Signal => {
                self.step = CalibStep::Steady;
                self.step_since = now;
            }
            CalibStep::Steady => {
                self.steady_buf.push((now, axes.y, axes.z));
                while self.steady_buf.len() > 1 && (now - self.steady_buf[0].0) > STEADY_KEEP_MS {
                    self.steady_buf.remove(0);
                }

                let cos_steady = (5.0 * DEG).cos();
                let spans = self.steady_buf.len() >= 2 && (now - self.steady_buf[0].0) >= STEADY_MS;
                let agrees = self.steady_buf.iter().all(|(_, y, z)| {
                    y.dot(&axes.y) > cos_steady && z.dot(&axes.z) > cos_steady
                });
                let given_up = (now - self.step_since) > STEADY_GIVE_UP_MS;

                if (spans && agrees) || given_up {
                    if let Some(built) = build_frame(&axes) {
                        self.frame = Some(built);
                        self.noise_deg = 0.5; // measured noise floor
                        self.yaw_min = 0.0;
                        self.yaw_max = 0.0;
                        self.pitch_min = 0.0;
                        self.pitch_max = 0.0;
                        self.last_span = 0.0;
                        self.span_still_since = now;
                        self.step = CalibStep::Range;
                        self.step_since = now;
                    } else if given_up {
                        self.step_since = now;
                    }
                }
            }
            CalibStep::Range => {
                if let Some(ref mut frame) = self.frame {
                    let fwd = forward_of(frame, &axes);
                    let (yaw, pitch) = angles_in(frame, &fwd);
                    self.yaw_min = self.yaw_min.min(yaw);
                    self.yaw_max = self.yaw_max.max(yaw);
                    self.pitch_min = self.pitch_min.min(pitch);
                    self.pitch_max = self.pitch_max.max(pitch);

                    let both_yaw = self.yaw_min < -12.0 && self.yaw_max > 12.0;
                    let both_pitch = self.pitch_min < -7.0 && self.pitch_max > 7.0;
                    let span = (self.yaw_max - self.yaw_min) + (self.pitch_max - self.pitch_min);
                    if span > self.last_span + 0.5 {
                        self.last_span = span;
                        self.span_still_since = now;
                    }
                    let settled = (now - self.span_still_since) > 900.0;
                    let elapsed = now - self.step_since;
                    if (both_yaw && both_pitch && settled && elapsed > 3000.0) || elapsed > 14000.0 {
                        self.finish();
                    }
                }
            }
            CalibStep::Done => {}
        }

        self.frame.as_ref()
    }

    pub fn finish(&mut self) {
        let noise_floor = clamp(self.noise_deg * 55.0, 20.0, 50.0);
        let hi_x = noise_floor.max(60.0);
        let hi_y = (noise_floor * 0.62).max(40.0);
        let span_x = clamp(self.yaw_max - self.yaw_min, noise_floor, hi_x);
        let span_y = clamp(self.pitch_max - self.pitch_min, noise_floor * 0.62, hi_y);

        if let Some(ref mut frame) = self.frame {
            frame.yaw0 += (self.yaw_min + self.yaw_max) / 2.0;
            frame.pitch0 += (self.pitch_min + self.pitch_max) / 2.0;
            self.result = Some(CalibrationResult {
                deg_per_screen_x: span_x * 0.6,
                deg_per_screen_y: span_y * 0.6,
                grip: frame.axis,
                noise_deg: self.noise_deg,
            });
        }

        self.active = false;
        self.done = true;
        self.step = CalibStep::Done;
    }
}
