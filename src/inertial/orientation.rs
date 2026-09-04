use crate::types::OrientationSample;
use std::f64::consts::PI;

pub const DEG: f64 = PI / 180.0;
pub const TAU: f64 = PI * 2.0;

#[inline]
pub fn clamp(v: f64, min: f64, max: f64) -> f64 {
    if v < min {
        min
    } else if v > max {
        max
    } else {
        v
    }
}

/// 3D vector and operations
#[derive(Debug, Clone, Copy, PartialEq, Default)]
pub struct Vec3 {
    pub x: f64,
    pub y: f64,
    pub z: f64,
}

impl Vec3 {
    pub const fn new(x: f64, y: f64, z: f64) -> Self {
        Self { x, y, z }
    }

    pub fn dot(&self, other: &Self) -> f64 {
        self.x * other.x + self.y * other.y + self.z * other.z
    }

    pub fn cross(&self, other: &Self) -> Self {
        Self {
            x: self.y * other.z - self.z * other.y,
            y: self.z * other.x - self.x * other.z,
            z: self.x * other.y - self.y * other.x,
        }
    }

    pub fn scale(&self, k: f64) -> Self {
        Self {
            x: self.x * k,
            y: self.y * k,
            z: self.z * k,
        }
    }

    pub fn sub(&self, other: &Self) -> Self {
        Self {
            x: self.x - other.x,
            y: self.y - other.y,
            z: self.z - other.z,
        }
    }

    pub fn add(&self, other: &Self) -> Self {
        Self {
            x: self.x + other.x,
            y: self.y + other.y,
            z: self.z + other.z,
        }
    }

    pub fn length(&self) -> f64 {
        (self.x * self.x + self.y * self.y + self.z * self.z).sqrt()
    }

    pub fn norm(&self) -> Self {
        let l = self.length();
        if l > 1e-9 {
            self.scale(1.0 / l)
        } else {
            *self
        }
    }
}

/// The phone's three body axes in world coordinates.
/// x -> right edge, y -> top edge, z -> out of the screen
#[derive(Debug, Clone, Copy, PartialEq, Default)]
pub struct BodyAxes {
    pub x: Vec3,
    pub y: Vec3,
    pub z: Vec3,
}

/// Body axes from Euler angles (alpha: yaw Z, beta: pitch X, gamma: roll Y)
pub fn body_axes(alpha: f64, beta: f64, gamma: f64) -> BodyAxes {
    let z = alpha * DEG;
    let x = beta * DEG;
    let y = gamma * DEG;
    let (c_z, s_z) = (z.cos(), z.sin());
    let (c_x, s_x) = (x.cos(), x.sin());
    let (c_y, s_y) = (y.cos(), y.sin());
    BodyAxes {
        x: Vec3::new(
            c_z * c_y - s_z * s_x * s_y,
            s_z * c_y + c_z * s_x * s_y,
            -c_x * s_y,
        ),
        y: Vec3::new(
            -s_z * c_x,
            c_z * c_x,
            s_x,
        ),
        z: Vec3::new(
            c_z * s_y + s_z * s_x * c_y,
            s_z * s_y - c_z * s_x * c_y,
            c_x * c_y,
        ),
    }
}

/// Body axes directly from quaternion [x, y, z, w]
pub fn body_axes_from_quat(q: [f64; 4]) -> BodyAxes {
    let [x, y, z, w] = q;
    BodyAxes {
        x: Vec3::new(
            1.0 - 2.0 * (y * y + z * z),
            2.0 * (x * y + z * w),
            2.0 * (x * z - y * w),
        ),
        y: Vec3::new(
            2.0 * (x * y - z * w),
            1.0 - 2.0 * (x * x + z * z),
            2.0 * (y * z + x * w),
        ),
        z: Vec3::new(
            2.0 * (x * z + y * w),
            2.0 * (y * z - x * w),
            1.0 - 2.0 * (x * x + y * y),
        ),
    }
}

/// Decodes sample to body axes
pub fn axes_from_sample(sample: &OrientationSample) -> BodyAxes {
    if let Some(q) = sample.quat {
        body_axes_from_quat(q)
    } else {
        body_axes(
            sample.alpha.unwrap_or(0.0),
            sample.beta.unwrap_or(0.0),
            sample.gamma.unwrap_or(0.0),
        )
    }
}

/// Shortest signed difference a - b, wrapped to (-180, 180]
pub fn angle_delta(a: f64, b: f64) -> f64 {
    let mut d = a - b;
    while d > 180.0 {
        d -= 360.0;
    }
    while d <= -180.0 {
        d += 360.0;
    }
    d
}

/// Wrap angle to (-180, 180]
pub fn wrap_deg(mut a: f64) -> f64 {
    while a > 180.0 {
        a -= 360.0;
    }
    while a <= -180.0 {
        a += 360.0;
    }
    a
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct GripTilt {
    pub bank: f64,
    pub pitch: f64,
}

/// Tilt of the phone relative to horizon (degrees)
pub fn grip_tilt(axes: &BodyAxes) -> GripTilt {
    let beam_y = axes.y;
    let beam_z = axes.z.scale(-1.0);
    let fwd = if beam_y.z.abs() <= beam_z.z.abs() {
        beam_y
    } else {
        beam_z
    };
    GripTilt {
        bank: -clamp(axes.x.z, -1.0, 1.0).asin() / DEG,
        pitch: clamp(fwd.z, -1.0, 1.0).asin() / DEG,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_body_axes_orthonormality() {
        let axes = body_axes(45.0, 30.0, -15.0);
        assert!((axes.x.length() - 1.0).abs() < 1e-6);
        assert!((axes.y.length() - 1.0).abs() < 1e-6);
        assert!((axes.z.length() - 1.0).abs() < 1e-6);
        assert!(axes.x.dot(&axes.y).abs() < 1e-6);
        assert!(axes.y.dot(&axes.z).abs() < 1e-6);
        assert!(axes.z.dot(&axes.x).abs() < 1e-6);
    }
}
