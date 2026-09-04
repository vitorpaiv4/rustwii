use serde::{Deserialize, Serialize};

/// 3D Linear acceleration and angular rotation rate from DeviceMotion
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize, Default)]
pub struct MotionData {
    pub ax: f64,
    pub ay: f64,
    pub az: f64,
    pub rx: f64, // rotationRate.beta (pitch rate in deg/s)
    pub ry: f64, // rotationRate.gamma (roll rate in deg/s)
    pub rz: f64, // rotationRate.alpha (yaw rate in deg/s)
}

/// Raw orientation Euler angles (degrees)
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize, Default)]
pub struct OrientationData {
    pub alpha: f64, // Z axis (0 to 360)
    pub beta: f64,  // X axis (-180 to 180)
    pub gamma: f64, // Y axis (-90 to 90)
}

/// Rich orientation sample sent from smartphone at ~60Hz
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, Default)]
pub struct OrientationSample {
    pub alpha: Option<f64>,
    pub beta: Option<f64>,
    pub gamma: Option<f64>,
    pub heading: Option<f64>,
    pub quat: Option<[f64; 4]>, // [x, y, z, w] from Absolute/RelativeOrientationSensor
    pub motion: Option<MotionData>,
    pub t: f64, // timestamp in milliseconds
}

/// Wii Remote button actions
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum RemoteButton {
    A,
    B,
    Home,
    Plus,
    Minus,
    One,
    Two,
    DpadUp,
    DpadDown,
    DpadLeft,
    DpadRight,
}

/// Action state for button press / release
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum ButtonAction {
    Press,
    Release,
}

/// Messages sent from Remote (Smartphone) to Server
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(tag = "type", content = "data")]
pub enum ClientMessage {
    JoinRoom { room_id: String, player_name: Option<String> },
    Sample(OrientationSample),
    Motion(OrientationData), // Fallback for simple packets
    Button { button: RemoteButton, action: ButtonAction },
    CalibrateCenter,
    Speed { factor: f64 },
    Ping,
}

/// Messages broadcasted from Server to Screen (PC/Console) or back to Remote
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(tag = "type", content = "data")]
pub enum ServerMessage {
    RoomJoined { player_id: usize, room_id: String },
    PlayerConnected { player_id: usize, total_players: usize },
    PlayerDisconnected { player_id: usize, total_players: usize },
    PlayerSample { player_id: usize, sample: OrientationSample },
    PlayerMotion { player_id: usize, orientation: OrientationData },
    PlayerButton { player_id: usize, button: RemoteButton, action: ButtonAction },
    Feedback { kind: String, combo: usize },
    Pong,
    Error { message: String },
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_sample_message_serde() {
        let sample = OrientationSample {
            alpha: Some(180.5),
            beta: Some(12.0),
            gamma: Some(-4.5),
            heading: Some(90.0),
            quat: Some([0.0, 0.707, 0.0, 0.707]),
            motion: Some(MotionData {
                ax: 0.1,
                ay: 9.8,
                az: 0.2,
                rx: 15.0,
                ry: 2.0,
                rz: -8.5,
            }),
            t: 123456.78,
        };

        let msg = ClientMessage::Sample(sample.clone());
        let json = serde_json::to_string(&msg).expect("serialization failed");
        let deserialized: ClientMessage = serde_json::from_str(&json).expect("deserialization failed");
        assert_eq!(msg, deserialized);
    }

    #[test]
    fn test_button_message_serde() {
        let msg = ClientMessage::Button {
            button: RemoteButton::A,
            action: ButtonAction::Press,
        };

        let json = serde_json::to_string(&msg).expect("serialization failed");
        let deserialized: ClientMessage = serde_json::from_str(&json).expect("deserialization failed");
        assert_eq!(msg, deserialized);
    }
}

