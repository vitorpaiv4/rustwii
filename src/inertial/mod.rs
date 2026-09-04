pub mod calibration;
pub mod cursor;
pub mod filter;
pub mod gesture;
pub mod orientation;
pub mod pointer;

pub use calibration::{build_frame, CalibFrame, Calibration, CalibrationResult};
pub use cursor::{CursorState, PlayerColor, PLAYER_COLORS};
pub use filter::OneEuro;
pub use gesture::{angle_diff, SwingDetector, SwingResult};
pub use orientation::{
    axes_from_sample, body_axes, body_axes_from_quat, clamp, grip_tilt, wrap_deg, BodyAxes,
    GripTilt, Vec3, DEG, TAU,
};
pub use pointer::Pointer;
