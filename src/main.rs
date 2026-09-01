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
                use dioxus::fullstack::prelude::*;
                use rustwii::server::{ws_handler, RoomManager};

                let room_manager = RoomManager::new();
                let address = std::net::SocketAddr::from(([0, 0, 0, 0], 8080));

                println!("🎮 Servidor RustWii iniciado com sucesso!");
                println!("🖥️  Tela Principal (Wii Screen):  http://localhost:8080/");
                println!("📱 Controle Remoto (Smartphone): http://localhost:8080/remote/WII-001");
                println!("🌐 Rede Local:                   http://192.168.30.238:8080/");

                let listener = tokio::net::TcpListener::bind(address)
                    .await
                    .expect("Falha ao abrir porta TCP");

                let index_template = r#"<!DOCTYPE html>
<html lang="pt-BR">
<head>
    <meta charset="utf-8"/>
    <meta name="viewport" content="width=device-width, initial-scale=1.0, maximum-scale=1.0, user-scalable=no"/>
    <title>RustWii - Nintendo Wii Experience in Pure Rust</title>
</head>
<body>
    <div id="main"></div>
</body>
</html>"#;

                let cfg = ServeConfigBuilder::new().index_html(index_template.to_string());

                let router = axum::Router::new()
                    .route("/ws/:room_id", get(ws_handler))
                    .with_state(room_manager)
                    .serve_dioxus_application(cfg, App);

                axum::serve(listener, router.into_make_service())
                    .await
                    .expect("Falha no servidor Axum");
            });
    }
}
