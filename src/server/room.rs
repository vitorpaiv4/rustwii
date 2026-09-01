#[cfg(not(target_arch = "wasm32"))]
use std::collections::{HashMap, HashSet};
#[cfg(not(target_arch = "wasm32"))]
use std::sync::Arc;
#[cfg(not(target_arch = "wasm32"))]
use tokio::sync::{broadcast, RwLock};
#[cfg(not(target_arch = "wasm32"))]
use crate::types::ServerMessage;

#[cfg(not(target_arch = "wasm32"))]
#[derive(Debug, Clone)]
pub struct Room {
    pub room_id: String,
    pub tx: broadcast::Sender<ServerMessage>,
    pub active_players: HashSet<usize>,
}

#[cfg(not(target_arch = "wasm32"))]
#[derive(Debug, Clone, Default)]
pub struct RoomManager {
    rooms: Arc<RwLock<HashMap<String, Room>>>,
}

#[cfg(not(target_arch = "wasm32"))]
impl RoomManager {
    pub fn new() -> Self {
        Self {
            rooms: Arc::new(RwLock::new(HashMap::new())),
        }
    }

    /// Registers a Screen (host) in the room and returns a broadcast receiver and sender
    pub async fn join_as_screen(
        &self,
        room_id: &str,
    ) -> (broadcast::Receiver<ServerMessage>, broadcast::Sender<ServerMessage>, usize) {
        let mut rooms = self.rooms.write().await;
        let room = rooms.entry(room_id.to_string()).or_insert_with(|| {
            let (tx, _) = broadcast::channel(256);
            Room {
                room_id: room_id.to_string(),
                tx,
                active_players: HashSet::new(),
            }
        });
        (room.tx.subscribe(), room.tx.clone(), room.active_players.len())
    }

    /// Registers a Remote in the room and allocates a player ID (1 to 4)
    pub async fn join_as_remote(
        &self,
        room_id: &str,
    ) -> Option<(usize, broadcast::Sender<ServerMessage>, broadcast::Receiver<ServerMessage>, usize)> {
        let mut rooms = self.rooms.write().await;
        let room = rooms.entry(room_id.to_string()).or_insert_with(|| {
            let (tx, _) = broadcast::channel(256);
            Room {
                room_id: room_id.to_string(),
                tx,
                active_players: HashSet::new(),
            }
        });

        // Max 4 players
        let mut allocated_id = None;
        for id in 1..=4 {
            if !room.active_players.contains(&id) {
                allocated_id = Some(id);
                break;
            }
        }

        let player_id = allocated_id?;
        room.active_players.insert(player_id);
        let total = room.active_players.len();

        Some((player_id, room.tx.clone(), room.tx.subscribe(), total))
    }

    /// Removes a player from the room and returns the updated count of active players
    pub async fn leave(&self, room_id: &str, player_id: usize) -> usize {
        let mut rooms = self.rooms.write().await;
        if let Some(room) = rooms.get_mut(room_id) {
            room.active_players.remove(&player_id);
            let remaining = room.active_players.len();
            if remaining == 0 && room.tx.receiver_count() == 0 {
                rooms.remove(room_id);
            }
            remaining
        } else {
            0
        }
    }
}

#[cfg(all(test, not(target_arch = "wasm32")))]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_room_player_allocation() {
        let manager = RoomManager::new();
        let room_id = "TEST-ROOM";

        let (p1, _tx, _, count1) = manager.join_as_remote(room_id).await.unwrap();
        assert_eq!(p1, 1);
        assert_eq!(count1, 1);

        let (p2, _, _, count2) = manager.join_as_remote(room_id).await.unwrap();
        assert_eq!(p2, 2);
        assert_eq!(count2, 2);

        let (p3, _, _, count3) = manager.join_as_remote(room_id).await.unwrap();
        assert_eq!(p3, 3);
        assert_eq!(count3, 3);

        let (p4, _, _, count4) = manager.join_as_remote(room_id).await.unwrap();
        assert_eq!(p4, 4);
        assert_eq!(count4, 4);

        // 5th player should fail as maximum is 4
        assert!(manager.join_as_remote(room_id).await.is_none());

        // Leave p2 and allocate again -> should reuse p2 ID
        let remaining = manager.leave(room_id, 2).await;
        assert_eq!(remaining, 3);

        let (reassigned_p2, _, _, new_count) = manager.join_as_remote(room_id).await.unwrap();
        assert_eq!(reassigned_p2, 2);
        assert_eq!(new_count, 4);
    }
}
