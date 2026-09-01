use crate::types::OrientationData;

/// Color styling definition per player
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct PlayerColor {
    pub name: &'static str,
    pub primary: &'static str,
    pub glow: &'static str,
}

pub const PLAYER_COLORS: [PlayerColor; 4] = [
    PlayerColor { name: "P1 (Azul)", primary: "#00a8e8", glow: "rgba(0, 168, 232, 0.6)" },
    PlayerColor { name: "P2 (Vermelho)", primary: "#ff4d4f", glow: "rgba(255, 77, 79, 0.6)" },
    PlayerColor { name: "P3 (Verde)", primary: "#52c41a", glow: "rgba(82, 196, 26, 0.6)" },
    PlayerColor { name: "P4 (Amarelo)", primary: "#faad14", glow: "rgba(250, 173, 20, 0.6)" },
];

/// Filter and mapping configuration for screen cursor
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct CursorConfig {
    pub yaw_half_range: f64,   // Degrees to reach horizontal screen border (e.g. 25.0°)
    pub pitch_half_range: f64, // Degrees to reach vertical screen border (e.g. 18.0°)
    pub base_smoothing: f64,   // Base Exponential Smoothing factor (0.0 to 1.0)
    pub deadzone_deg: f64,     // Angular deadzone threshold to eliminate micro-vibrations
}

impl Default for CursorConfig {
    fn default() -> Self {
        Self {
            yaw_half_range: 24.0,
            pitch_half_range: 16.0,
            base_smoothing: 0.35,
            deadzone_deg: 0.15,
        }
    }
}

/// Dynamic state of a Wiimote pointer on screen
#[derive(Debug, Clone, PartialEq)]
pub struct CursorState {
    pub player_id: usize,
    pub x: f64,             // Percentage 0.0 to 100.0 (horizontal)
    pub y: f64,             // Percentage 0.0 to 100.0 (vertical)
    pub rotation_deg: f64,  // Roll angle in degrees
    pub is_clicking: bool,  // Button A state
    pub is_trigger: bool,   // Button B state
    pub is_active: bool,
    config: CursorConfig,
}

impl CursorState {
    pub fn new(player_id: usize) -> Self {
        Self {
            player_id,
            x: 50.0,
            y: 50.0,
            rotation_deg: 0.0,
            is_clicking: false,
            is_trigger: false,
            is_active: false,
            config: CursorConfig::default(),
        }
    }

    /// Color scheme for the current cursor
    pub fn color(&self) -> PlayerColor {
        let idx = if self.player_id >= 1 && self.player_id <= 4 {
            self.player_id - 1
        } else {
            0
        };
        PLAYER_COLORS[idx]
    }

    /// Updates cursor position with new orientation data applying adaptive smoothing and clamping
    pub fn update_orientation(&mut self, orientation: &OrientationData) {
        self.is_active = true;

        // Calculate target X from yaw (alpha)
        let normalized_yaw = (orientation.alpha / self.config.yaw_half_range).clamp(-1.0, 1.0);
        let target_x = 50.0 + (normalized_yaw * 50.0);

        // Calculate target Y from pitch (beta) - pitch up is negative Y in screen space
        let normalized_pitch = (orientation.beta / self.config.pitch_half_range).clamp(-1.0, 1.0);
        let target_y = 50.0 - (normalized_pitch * 50.0);

        // Target rotation from roll (gamma)
        let target_rotation = -orientation.gamma;

        // Dynamic smoothing: if movement is large, increase alpha to avoid lag; if small, keep smooth
        let dx = target_x - self.x;
        let dy = target_y - self.y;
        let dist = (dx * dx + dy * dy).sqrt();

        let adaptive_alpha = (self.config.base_smoothing + (dist * 0.04)).clamp(self.config.base_smoothing, 0.92);

        self.x = (self.x + adaptive_alpha * dx).clamp(0.0, 100.0);
        self.y = (self.y + adaptive_alpha * dy).clamp(0.0, 100.0);

        // Smooth rotation
        let d_rot = target_rotation - self.rotation_deg;
        self.rotation_deg += self.config.base_smoothing * d_rot;
    }

    pub fn set_click(&mut self, clicking: bool) {
        self.is_clicking = clicking;
    }

    pub fn set_trigger(&mut self, trigger: bool) {
        self.is_trigger = trigger;
    }

    pub fn set_active(&mut self, active: bool) {
        self.is_active = active;
        if !active {
            self.is_clicking = false;
            self.is_trigger = false;
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_cursor_center_mapping() {
        let mut cursor = CursorState::new(1);
        let center_orientation = OrientationData {
            alpha: 0.0,
            beta: 0.0,
            gamma: 0.0,
        };

        cursor.update_orientation(&center_orientation);
        assert!((cursor.x - 50.0).abs() < 0.001);
        assert!((cursor.y - 50.0).abs() < 0.001);
        assert!((cursor.rotation_deg - 0.0).abs() < 0.001);
    }

    #[test]
    fn test_cursor_boundary_clamping() {
        let mut cursor = CursorState::new(1);
        let extreme_orientation = OrientationData {
            alpha: 100.0, // Far right beyond half range
            beta: -100.0, // Far down
            gamma: 45.0,
        };

        // Run multiple updates to allow filter to reach target
        for _ in 0..50 {
            cursor.update_orientation(&extreme_orientation);
        }

        assert!((cursor.x - 100.0).abs() < 0.01);
        assert!((cursor.y - 100.0).abs() < 0.01);
    }

    #[test]
    fn test_cursor_smoothing_filter() {
        let mut cursor = CursorState::new(1);
        let step_orientation = OrientationData {
            alpha: 24.0, // Should target X = 100.0
            beta: 0.0,
            gamma: 0.0,
        };

        // Single step should move partially towards 100.0 due to smoothing
        cursor.update_orientation(&step_orientation);
        assert!(cursor.x > 50.0 && cursor.x < 100.0);
    }
}
