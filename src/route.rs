use dioxus::prelude::*;
use crate::views::{RemoteView, ScreenView};

#[derive(Routable, Clone, PartialEq, Debug)]
pub enum Route {
    #[route("/")]
    ScreenView {},

    #[route("/remote/:room_id")]
    RemoteView { room_id: String },
}
