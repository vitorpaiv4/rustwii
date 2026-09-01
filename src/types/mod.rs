use serde::{Deserialize, Serialize};

/// Sensor orientation angles captured from mobile device (Euler angles in degrees)
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub struct OrientationData {
    pub alpha: f64, // Z axis (0 to 360)
    pub beta: f64,  // X axis (-180 to 180)
    pub gamma: f64, // Y axis (-90 to 90)
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
    Motion(OrientationData),
    Button { button: RemoteButton, action: ButtonAction },
    CalibrateCenter,
    Ping,
}

/// Messages broadcasted from Server to Screen (PC/Console) or back to Remote
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(tag = "type", content = "data")]
pub enum ServerMessage {
    RoomJoined { player_id: usize, room_id: String },
    PlayerConnected { player_id: usize, total_players: usize },
    PlayerDisconnected { player_id: usize, total_players: usize },
    PlayerMotion { player_id: usize, orientation: OrientationData },
    PlayerButton { player_id: usize, button: RemoteButton, action: ButtonAction },
    Pong,
    Error { message: String },
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_client_message_serde() {
        let msg = ClientMessage::Motion(OrientationData {
            alpha: 180.5,
            beta: 12.0,
            gamma: -4.5,
        });

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

