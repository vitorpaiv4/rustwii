use crate::inertial::orientation::{
    axes_from_sample, clamp, wrap_deg, BodyAxes, Vec3, DEG,
};
use crate::types::OrientationSample;
use std::collections::VecDeque;

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct AxisMapping {
    pub col: usize,
    pub sign: f64,
    pub scale: f64,
}

/// Body-frame angular velocity between two attitudes (deg/s)
pub fn omega_from_attitudes(a1: &BodyAxes, a2: &BodyAxes, dt: f64) -> Vec3 {
    if dt <= 1e-6 {
        return Vec3::default();
    }
    let col = |a: &BodyAxes, k: usize| match k {
        0 => [a.x.x, a.x.y, a.x.z],
        1 => [a.y.x, a.y.y, a.y.z],
        _ => [a.z.x, a.z.y, a.z.z],
    };
    let r1 = [col(a1, 0), col(a1, 1), col(a1, 2)];
    let r2 = [col(a2, 0), col(a2, 1), col(a2, 2)];

    let mut m = [[0.0; 3]; 3];
    for i in 0..3 {
        for j in 0..3 {
            m[i][j] = r1[i][0] * r2[j][0] + r1[i][1] * r2[j][1] + r1[i][2] * r2[j][2];
        }
    }
    Vec3 {
        x: (m[2][1] - m[1][2]) / (2.0 * dt) / DEG,
        y: (m[0][2] - m[2][0]) / (2.0 * dt) / DEG,
        z: (m[1][0] - m[0][1]) / (2.0 * dt) / DEG,
    }
}

/// OpenWii Rate-Based Gyro Pointer with Learned Per-Device Axis Map
#[derive(Debug, Clone, PartialEq)]
pub struct Pointer {
    pub sensitivity: f64,
    pub deg_per_screen: f64,
    pub aspect: f64,
    pub invert_x: bool,
    pub invert_y: bool,
    pub deadzone_dps: f64,
    pub fallback_deadzone_dps: f64,

    pub pos: (f64, f64),      // (x, y) in [0.0, 1.0]
    pub rate: (f64, f64),     // screen fractions per second
    pub rate_dps: (f64, f64), // (yaw_dps, pitch_dps)

    pub display_lead: bool,
    pub ema_packet_dt: f64,
    pub lead_tau: f64,
    pub ema_rate: (f64, f64),
    pub ema_abs_dps: f64,

    // Pose anchoring
    pub yaw_off_deg: f64,
    pub pitch_off_deg: f64,
    pub overshoot: f64,
    pub ref_az: Option<f64>,
    pub ref_elev: Option<f64>,
    pub used_beam: Option<char>,
    pub fwd_beam: Option<char>,
    pub still_ms: f64,
    pub heal_tau: f64,

    // Ground truth
    pub prev_axes: Option<BodyAxes>,
    pub ema_o: Vec3,
    pub ema_tau: f64,

    // Axis-map learning
    pub ring: VecDeque<[f64; 3]>,
    pub c_matrix: [[f64; 3]; 3],
    pub abs_o: [f64; 3],
    pub abs_r: [f64; 3],
    pub axis_map: [Option<AxisMapping>; 3],
    pub map_shape: String,
    pub ema_m: Vec3,
    pub res_bad: f64,
    pub res_all: f64,
    pub gyro_trusted: bool,

    pub live: bool,
    pub last_seen: f64,
    pub last_draw: f64,
    pub has_gyro: bool,
}

impl Default for Pointer {
    fn default() -> Self {
        Self::new()
    }
}

impl Pointer {
    pub fn new() -> Self {
        Self {
            sensitivity: 1.3,
            deg_per_screen: 14.0,
            aspect: 0.6,
            invert_x: false,
            invert_y: false,
            deadzone_dps: 0.15,
            fallback_deadzone_dps: 1.0,

            pos: (0.5, 0.5),
            rate: (0.0, 0.0),
            rate_dps: (0.0, 0.0),

            display_lead: true,
            ema_packet_dt: 1.0 / 60.0,
            lead_tau: 0.02,
            ema_rate: (0.0, 0.0),
            ema_abs_dps: 0.0,

            yaw_off_deg: 0.0,
            pitch_off_deg: 0.0,
            overshoot: 0.15,
            ref_az: None,
            ref_elev: None,
            used_beam: None,
            fwd_beam: None,
            still_ms: 0.0,
            heal_tau: 1.2,

            prev_axes: None,
            ema_o: Vec3::default(),
            ema_tau: 0.08,

            ring: VecDeque::new(),
            c_matrix: [[0.0; 3]; 3],
            abs_o: [0.0; 3],
            abs_r: [0.0; 3],
            axis_map: [None, None, None],
            map_shape: String::new(),
            ema_m: Vec3::default(),
            res_bad: 0.0,
            res_all: 0.0,
            gyro_trusted: false,

            live: false,
            last_seen: 0.0,
            last_draw: 0.0,
            has_gyro: false,
        }
    }

    pub fn recentre(&mut self) {
        self.pos = (0.5, 0.5);
        self.rate = (0.0, 0.0);
        self.yaw_off_deg = 0.0;
        self.pitch_off_deg = 0.0;
        self.ref_az = None;
        self.ref_elev = None;
    }

    pub fn set_from_mouse(&mut self, nx: f64, ny: f64) {
        self.pos.0 = clamp(nx, 0.0, 1.0);
        self.pos.1 = clamp(ny, 0.0, 1.0);
        self.rate = (0.0, 0.0);
        let sx = if self.invert_x { -self.sensitivity } else { self.sensitivity };
        let sy = if self.invert_y { -self.sensitivity } else { self.sensitivity };
        let sx = if sx.abs() < 1e-6 { 1.0 } else { sx };
        let sy = if sy.abs() < 1e-6 { 1.0 } else { sy };
        self.yaw_off_deg = ((0.5 - self.pos.0) * self.deg_per_screen) / sx;
        self.pitch_off_deg = ((0.5 - self.pos.1) * self.deg_per_screen * self.aspect) / sy;
        self.ref_az = None;
        self.ref_elev = None;
    }

    pub fn apply_map(&self, r: &[f64; 3]) -> Vec3 {
        let mut out = [0.0; 3];
        for i in 0..3 {
            if let Some(ref m) = self.axis_map[i] {
                if m.col < 3 {
                    out[i] = m.sign * m.scale * r[m.col];
                }
            }
        }
        Vec3::new(out[0], out[1], out[2])
    }

    /// Feeds one sample from mobile controller. `dt` is seconds since previous sample.
    pub fn update(&mut self, sample: &OrientationSample, dt: f64, now: f64) {
        let axes = axes_from_sample(sample);
        self.live = true;
        self.last_seen = now;
        if dt > 0.0 && dt < 0.1 {
            self.ema_packet_dt += (dt - self.ema_packet_dt) * 0.1;
        }

        // 1. Ground truth body rates from attitude matrix
        if let Some(prev) = self.prev_axes {
            if dt > 0.0 && dt < 0.1 {
                let omega_true = omega_from_attitudes(&prev, &axes, dt);
                let k = clamp(dt / self.ema_tau, 0.0, 1.0);
                self.ema_o.x += (omega_true.x - self.ema_o.x) * k;
                self.ema_o.y += (omega_true.y - self.ema_o.y) * k;
                self.ema_o.z += (omega_true.z - self.ema_o.z) * k;
            }
        }
        self.prev_axes = Some(axes);

        let r = sample.motion.map(|m| [m.rx, m.ry, m.rz]);
        if let Some(r_arr) = r {
            if r_arr[0].abs() > 1e-4 || r_arr[1].abs() > 1e-4 || r_arr[2].abs() > 1e-4 {
                self.has_gyro = true;
            }
        }

        // 2. Learn the device's axis map
        if let Some(r_arr) = r {
            if self.has_gyro && dt > 0.0 && dt < 0.1 {
                self.ring.push_back(r_arr);
                let delay_slots = (0.05 / dt.max(1.0 / 240.0)).round() as usize;
                let delay_slots = delay_slots.max(1);
                while self.ring.len() > delay_slots + 1 {
                    self.ring.pop_front();
                }
                let r_del = self.ring.front().copied().unwrap_or(r_arr);

                let moving = self.ema_o.length() > 10.0;
                if moving {
                    let decay = (-dt / 30.0).exp();
                    let t = [self.ema_o.x, self.ema_o.y, self.ema_o.z];
                    for i in 0..3 {
                        self.abs_o[i] = self.abs_o[i] * decay + t[i].abs() * dt;
                        self.abs_r[i] = self.abs_r[i] * decay + r_del[i].abs() * dt;
                        for j in 0..3 {
                            self.c_matrix[i][j] = self.c_matrix[i][j] * decay + t[i] * r_del[j] * dt;
                        }
                    }

                    let max_abs_o = self.abs_o[0].max(self.abs_o[1]).max(self.abs_o[2]).max(1e-9);
                    let mut order = [0, 1, 2];
                    order.sort_by(|&a, &b| self.abs_o[b].partial_cmp(&self.abs_o[a]).unwrap());

                    let mut taken = [false; 3];
                    let mut new_map = [None, None, None];

                    for &i in &order {
                        let prev = self.axis_map[i];
                        let mut best_col = 0;
                        let mut best_val = -1.0;
                        for j in 0..3 {
                            let val = if taken[j] { 0.0 } else { self.c_matrix[i][j].abs() };
                            if val > best_val {
                                best_val = val;
                                best_col = j;
                            }
                        }

                        let mut second_val = 0.0f64;
                        for j in 0..3 {
                            if j != best_col {
                                second_val = second_val.max(self.c_matrix[i][j].abs());
                            }
                        }

                        let decisive = best_val > 2.0 * second_val && self.abs_r[best_col] > 1e-9;
                        let eligible = self.abs_o[i] >= 15.0 && self.abs_o[i] >= 0.15 * max_abs_o;

                        if let Some(p) = prev {
                            if !taken[p.col] && !(decisive && best_col != p.col && eligible) {
                                let s = self.c_matrix[i][p.col].signum();
                                let sign = if s == 0.0 { p.sign } else { s };
                                new_map[i] = Some(AxisMapping {
                                    col: p.col,
                                    sign,
                                    scale: self.abs_o[i] / self.abs_r[p.col].max(1e-9),
                                });
                                taken[p.col] = true;
                            } else if eligible && decisive {
                                new_map[i] = Some(AxisMapping {
                                    col: best_col,
                                    sign: self.c_matrix[i][best_col].signum(),
                                    scale: self.abs_o[i] / self.abs_r[best_col].max(1e-9),
                                });
                                taken[best_col] = true;
                            }
                        } else if eligible && decisive {
                            new_map[i] = Some(AxisMapping {
                                col: best_col,
                                sign: self.c_matrix[i][best_col].signum(),
                                scale: self.abs_o[i] / self.abs_r[best_col].max(1e-9),
                            });
                            taken[best_col] = true;
                        }
                    }

                    self.axis_map = new_map;

                    // Residual Gate
                    let mapped = self.apply_map(&r_del);
                    let km = clamp(dt / self.ema_tau, 0.0, 1.0);
                    self.ema_m.x += (mapped.x - self.ema_m.x) * km;
                    self.ema_m.y += (mapped.y - self.ema_m.y) * km;
                    self.ema_m.z += (mapped.z - self.ema_m.z) * km;

                    let err = ((self.ema_m.x - self.ema_o.x).powi(2)
                        + (self.ema_m.y - self.ema_o.y).powi(2)
                        + (self.ema_m.z - self.ema_o.z).powi(2))
                    .sqrt();

                    self.res_bad = self.res_bad * decay + err * dt;
                    self.res_all = self.res_all * decay + self.ema_o.length() * dt;

                    let pitch_claimed = self.axis_map[0].is_some();
                    let yaw_claimed = self.axis_map[1].is_some() || self.axis_map[2].is_some();
                    self.gyro_trusted = pitch_claimed
                        && yaw_claimed
                        && self.res_all > 1.0
                        && (self.res_bad / self.res_all) < 0.6;
                }
            }
        }

        // 3. Live body rates
        let (omega, deadzone) = if self.gyro_trusted && r.is_some() {
            (self.apply_map(&r.unwrap()), self.deadzone_dps)
        } else {
            (self.ema_o, self.fallback_deadzone_dps)
        };

        // 4. Geometry: grip-agnostic screen axes
        let omega_w = Vec3::new(
            axes.x.x * omega.x + axes.y.x * omega.y + axes.z.x * omega.z,
            axes.x.y * omega.x + axes.y.y * omega.y + axes.z.y * omega.z,
            axes.x.z * omega.x + axes.y.z * omega.y + axes.z.z * omega.z,
        );

        let beam_y = axes.y;
        let beam_z = axes.z.scale(-1.0);
        let fwd = match self.fwd_beam {
            Some('y') => {
                if beam_z.z.abs() + 0.35 < beam_y.z.abs() {
                    beam_z
                } else {
                    beam_y
                }
            }
            Some('z') => {
                if beam_y.z.abs() + 0.35 < beam_z.z.abs() {
                    beam_y
                } else {
                    beam_z
                }
            }
            _ => {
                if beam_y.z.abs() <= beam_z.z.abs() {
                    beam_y
                } else {
                    beam_z
                }
            }
        };
        self.fwd_beam = Some(if fwd == beam_y { 'y' } else { 'z' });
        let right = fwd.cross(&Vec3::new(0.0, 0.0, 1.0)).norm();

        let mut yaw_dps = omega_w.z;
        let mut pitch_dps = omega_w.dot(&right);
        if yaw_dps.abs() < deadzone {
            yaw_dps = 0.0;
        }
        if pitch_dps.abs() < deadzone {
            pitch_dps = 0.0;
        }
        self.rate_dps = (yaw_dps, pitch_dps);

        // 5. Pose anchoring & stillness healing
        let az = fwd.y.atan2(fwd.x) / DEG;
        let elev = clamp(fwd.z, -1.0, 1.0).asin() / DEG;
        let beam_id = self.fwd_beam;

        if self.ref_az.is_none() || self.used_beam != beam_id {
            self.ref_az = Some(az - self.yaw_off_deg);
            self.ref_elev = Some(elev - self.pitch_off_deg);
            self.used_beam = beam_id;
        }

        let ref_az = self.ref_az.unwrap_or(az);
        let ref_elev = self.ref_elev.unwrap_or(elev);
        let err_x = wrap_deg(az - ref_az) - self.yaw_off_deg;
        let err_y = (elev - ref_elev) - self.pitch_off_deg;
        let err = (err_x * err_x + err_y * err_y).sqrt();

        let rate_now = yaw_dps.abs().max(pitch_dps.abs());
        self.still_ms = if rate_now < 12.0 {
            self.still_ms + dt * 1000.0
        } else {
            0.0
        };

        let mut tau = if self.still_ms > 400.0 {
            self.heal_tau * (1.0 + 3.0 * (rate_now / 12.0))
        } else {
            8.0
        };
        if rate_now < 40.0 && err > (3.0 + rate_now * 0.12) {
            tau = tau.min(1.5);
        }

        let k = if rate_now >= 40.0 {
            0.0
        } else {
            clamp(dt / tau, 0.0, 1.0)
        };
        self.yaw_off_deg += err_x * k;
        self.pitch_off_deg += err_y * k;

        let sx = if self.invert_x { -self.sensitivity } else { self.sensitivity };
        let sy = if self.invert_y { -self.sensitivity } else { self.sensitivity };
        self.rate.0 = (-yaw_dps / self.deg_per_screen) * sx;
        self.rate.1 = (-pitch_dps / (self.deg_per_screen * self.aspect)) * sy;

        let k_e = 1.0 - (-dt / self.lead_tau).exp();
        self.ema_rate.0 += (self.rate.0 - self.ema_rate.0) * k_e;
        self.ema_rate.1 += (self.rate.1 - self.ema_rate.1) * k_e;
        self.ema_abs_dps += (rate_now - self.ema_abs_dps) * k_e;
    }

    /// Where to draw cursor this frame (integrates rates smoothly). `now` is milliseconds timestamp.
    pub fn sample_at(&mut self, now: f64) -> (f64, f64) {
        if self.last_draw <= 0.0 {
            self.last_draw = now;
        }
        let mut dt = (now - self.last_draw) / 1000.0;
        self.last_draw = now;
        if dt <= 0.0 || dt > 0.25 {
            dt = 0.0;
        }

        if self.live && (now - self.last_seen) < 250.0 {
            self.yaw_off_deg += self.rate_dps.0 * dt;
            self.pitch_off_deg += self.rate_dps.1 * dt;

            let sx = if self.invert_x { -self.sensitivity } else { self.sensitivity };
            let sy = if self.invert_y { -self.sensitivity } else { self.sensitivity };

            let cap_x = ((0.5 + self.overshoot) * self.deg_per_screen) / sx.abs().max(1e-6);
            let cap_y = ((0.5 + self.overshoot) * self.deg_per_screen * self.aspect) / sy.abs().max(1e-6);

            self.yaw_off_deg = clamp(self.yaw_off_deg, -cap_x, cap_x);
            self.pitch_off_deg = clamp(self.pitch_off_deg, -cap_y, cap_y);

            self.pos.0 = clamp(0.5 - (self.yaw_off_deg * sx) / self.deg_per_screen, 0.0, 1.0);
            self.pos.1 = clamp(0.5 - (self.pitch_off_deg * sy) / (self.deg_per_screen * self.aspect), 0.0, 1.0);

            // Display Lead dead reckoning
            if self.display_lead {
                let ramp = clamp((self.ema_abs_dps - 18.0) / 32.0, 0.0, 1.0);
                if ramp > 0.0 {
                    let horizon = clamp(self.ema_packet_dt + 0.008, 0.012, 0.035) + self.lead_tau;
                    let horizon = horizon.min(0.055);
                    let lead = horizon * ramp;
                    self.pos.0 = clamp(self.pos.0 + self.ema_rate.0 * lead, 0.0, 1.0);
                    self.pos.1 = clamp(self.pos.1 + self.ema_rate.1 * lead, 0.0, 1.0);
                }
            }
        }

        self.pos
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::inertial::orientation::body_axes;

    #[test]
    fn test_pointer_recentre() {
        let mut ptr = Pointer::new();
        ptr.pos = (0.8, 0.2);
        ptr.recentre();
        assert_eq!(ptr.pos, (0.5, 0.5));
    }

    #[test]
    fn test_omega_from_attitudes() {
        let a1 = body_axes(0.0, 0.0, 0.0);
        let a2 = body_axes(10.0, 0.0, 0.0); // 10 degrees yaw in 0.1s = 100 deg/s around Z
        let omega = omega_from_attitudes(&a1, &a2, 0.1);
        assert!((omega.z - 100.0).abs() < 2.0);
    }

    #[test]
    fn test_pointer_motion_update() {
        let mut ptr = Pointer::new();
        let sample1 = OrientationSample {
            alpha: Some(0.0),
            beta: Some(0.0),
            gamma: Some(0.0),
            heading: None,
            quat: None,
            motion: None,
            t: 1000.0,
        };
        ptr.update(&sample1, 0.016, 1000.0);
        assert_eq!(ptr.pos, (0.5, 0.5));

        // Move phone attitude left (positive yaw)
        let sample2 = OrientationSample {
            alpha: Some(15.0),
            beta: Some(0.0),
            gamma: Some(0.0),
            heading: None,
            quat: None,
            motion: None,
            t: 1050.0,
        };
        ptr.update(&sample2, 0.05, 1050.0);
        let pos = ptr.sample_at(1050.0);
        // Cursor should move left (< 0.5)
        assert!(pos.0 < 0.5);
    }
}
