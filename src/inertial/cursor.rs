use crate::inertial::pointer::Pointer;
use crate::types::{OrientationData, OrientationSample};

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

/// Dynamic state of a Wiimote pointer on screen
#[derive(Debug, Clone, PartialEq)]
pub struct CursorState {
    pub player_id: usize,
    pub x: f64,            // Percentage 0.0 to 100.0 (horizontal)
    pub y: f64,            // Percentage 0.0 to 100.0 (vertical)
    pub rotation_deg: f64, // Roll angle in degrees
    pub is_clicking: bool, // Button A state
    pub is_trigger: bool,  // Button B state
    pub is_active: bool,
    pub pointer: Pointer,
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
            pointer: Pointer::new(),
        }
    }

    /// Color scheme for the current cursor
    pub fn color(&self) -> PlayerColor {
        let idx = if (1..=4).contains(&self.player_id) {
            self.player_id - 1
        } else {
            0
        };
        PLAYER_COLORS[idx]
    }

    /// Feeds rich orientation sample and updates pointer engine
    pub fn update_sample(&mut self, sample: &OrientationSample, dt: f64, now_ms: f64) {
        self.is_active = true;
        self.pointer.update(sample, dt, now_ms);
        let (nx, ny) = self.pointer.sample_at(now_ms);
        self.x = nx * 100.0;
        self.y = ny * 100.0;
        if let Some(g) = sample.gamma {
            self.rotation_deg = -g;
        }
    }

    /// Fallback for simple orientation data
    pub fn update_orientation(&mut self, orientation: &OrientationData) {
        let sample = OrientationSample {
            alpha: Some(orientation.alpha),
            beta: Some(orientation.beta),
            gamma: Some(orientation.gamma),
            heading: None,
            quat: None,
            motion: None,
            t: 0.0,
        };
        self.update_sample(&sample, 0.016, 0.0);
    }

    /// Animates and samples current cursor position for the given timestamp (now_ms)
    pub fn tick(&mut self, now_ms: f64) {
        if self.is_active {
            let (nx, ny) = self.pointer.sample_at(now_ms);
            self.x = nx * 100.0;
            self.y = ny * 100.0;
        }
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

    pub fn recentre(&mut self) {
        self.pointer.recentre();
        self.x = 50.0;
        self.y = 50.0;
    }

    pub fn set_mouse_pos(&mut self, pct_x: f64, pct_y: f64) {
        self.is_active = true;
        self.x = pct_x.clamp(0.0, 100.0);
        self.y = pct_y.clamp(0.0, 100.0);
        self.pointer.set_from_mouse(self.x / 100.0, self.y / 100.0);
    }
}
