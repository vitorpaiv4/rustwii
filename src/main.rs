use rustwii::App;

fn main() {
    #[cfg(feature = "server")]
    {
        tokio::runtime::Runtime::new().unwrap().block_on(async {
            use axum::routing::get;
            use rustwii::server::{ws_handler, RoomManager};

            let room_manager = RoomManager::new();
            let addr = std::net::SocketAddr::from(([0, 0, 0, 0], 8080));

            println!("🎮 Servidor RustWii iniciado em http://{}", addr);

            let listener = tokio::net::TcpListener::bind(&addr).await.unwrap();
            let app = axum::Router::new()
                .route("/ws/:room_id", get(ws_handler))
                .with_state(room_manager);

            axum::serve(listener, app).await.unwrap();
        });
    }

    #[cfg(not(feature = "server"))]
    {
        dioxus::launch(App);
    }
}
