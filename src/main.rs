#[cfg(target_arch = "wasm32")]
use rustwii::App;

fn main() {
    #[cfg(target_arch = "wasm32")]
    {
        dioxus::launch(App);
    }

    #[cfg(not(target_arch = "wasm32"))]
    {
        tokio::runtime::Runtime::new()
            .unwrap()
            .block_on(async move {
                use axum::routing::get;
                use rustwii::server::{ws_handler, RoomManager};
                use tower_http::services::{ServeDir, ServeFile};

                let room_manager = RoomManager::new();
                let address = std::net::SocketAddr::from(([0, 0, 0, 0], 8080));

                println!("🎮 Servidor RustWii iniciado com sucesso!");
                println!("🖥️  Tela Principal (Wii Screen):  http://localhost:8080/");
                println!("📱 Controle Remoto (Smartphone): http://localhost:8080/remote/WII-001");
                println!("🌐 Rede Local:                   http://192.168.30.238:8080/");

                let listener = tokio::net::TcpListener::bind(address)
                    .await
                    .expect("Falha ao abrir porta TCP");

                let static_dir = "target/dx/rustwii/debug/web/public";
                let index_file = format!("{}/index.html", static_dir);

                let router = axum::Router::new()
                    .route("/ws/:room_id", get(ws_handler))
                    .route("/api/server-info", get(rustwii::server::server_info_handler))
                    .with_state(room_manager)
                    .fallback_service(
                        ServeDir::new(static_dir)
                            .not_found_service(ServeFile::new(index_file))
                    );

                axum::serve(listener, router.into_make_service())
                    .await
                    .expect("Falha no servidor Axum");
            });
    }
}
