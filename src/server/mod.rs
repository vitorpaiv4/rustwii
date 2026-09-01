pub mod room;
pub mod ws;

#[cfg(not(target_arch = "wasm32"))]
pub use room::RoomManager;
#[cfg(not(target_arch = "wasm32"))]
pub use ws::{get_local_ip, server_info_handler, ws_handler};
