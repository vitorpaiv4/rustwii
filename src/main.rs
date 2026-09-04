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
                let http_addr = std::net::SocketAddr::from(([0, 0, 0, 0], 8080));
                let https_addr = std::net::SocketAddr::from(([0, 0, 0, 0], 8443));
                let local_ip = rustwii::server::get_local_ip();

                println!("============================================================");
                println!("🎮 Servidor RustWii (100% Pure Rust) iniciado com sucesso!");
                println!("🔒 HTTPS Nativo (Porta 8443 - Recomendado para Celulares):");
                println!("   🖥️  Tela Principal:  https://localhost:8443/");
                println!("   📱 Smartphone Wi-Fi: https://{}:8443/remote/WII-001", local_ip);
                println!("🌐 HTTP Padrão (Porta 8080):");
                println!("   🖥️  Tela Principal:  http://localhost:8080/");
                println!("   📱 Smartphone Wi-Fi: http://{}:8080/remote/WII-001", local_ip);
                if let Ok(pub_url) = std::env::var("PUBLIC_URL").or_else(|_| std::env::var("HOST_URL")) {
                    println!("🌍 URL Pública / Túnel:     {}/remote/WII-001", pub_url.trim_end_matches('/'));
                }
                println!("============================================================");

                fn make_app_router(room_manager: RoomManager) -> axum::Router {
                    let static_dir = if std::path::Path::new("target/dx/rustwii/release/web/public").exists() {
                        "target/dx/rustwii/release/web/public"
                    } else if std::path::Path::new("target/dx/rustwii/debug/web/public").exists() {
                        "target/dx/rustwii/debug/web/public"
                    } else {
                        "dist"
                    };
                    let index_file = format!("{}/index.html", static_dir);
                    println!("📦 Servindo arquivos web a partir de: {}", static_dir);
                    axum::Router::new()
                        .route("/ws/:room_id", get(ws_handler))
                        .route("/api/server-info", get(rustwii::server::server_info_handler))
                        .layer(tower_http::cors::CorsLayer::permissive())
                        .with_state(room_manager)
                        .fallback_service(
                            ServeDir::new(static_dir)
                                .not_found_service(ServeFile::new(index_file))
                        )
                }

                // Spawn HTTP server on 8080
                let http_listener = tokio::net::TcpListener::bind(http_addr)
                    .await
                    .expect("Falha ao abrir porta HTTP 8080");

                let rm_http = room_manager.clone();
                let http_task = tokio::spawn(async move {
                    let _ = axum::serve(http_listener, make_app_router(rm_http).into_make_service()).await;
                });

                // Spawn Native HTTPS server on 8443 with self-signed certificate
                let tls_config_res = rustwii::server::create_self_signed_rustls_config(&local_ip).await;
                let rm_https = room_manager.clone();
                match tls_config_res {
                    Ok(tls_config) => {
                        let https_task = tokio::spawn(async move {
                            let _ = axum_server::bind_rustls(https_addr, tls_config)
                                .serve(make_app_router(rm_https).into_make_service())
                                .await;
                        });
                        let _ = tokio::join!(http_task, https_task);
                    }
                    Err(err) => {
                        eprintln!("⚠️ Não foi possível iniciar HTTPS nativo: {:?}. Rodando em modo HTTP.", err);
                        let _ = http_task.await;
                    }
                }
            });
    }
}
