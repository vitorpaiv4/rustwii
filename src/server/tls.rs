#[cfg(not(target_arch = "wasm32"))]
use axum_server::tls_rustls::RustlsConfig;
#[cfg(not(target_arch = "wasm32"))]
use rcgen::generate_simple_self_signed;

#[cfg(not(target_arch = "wasm32"))]
pub async fn create_self_signed_rustls_config(
    local_ip: &str,
) -> Result<RustlsConfig, Box<dyn std::error::Error + Send + Sync>> {
    let subject_alt_names = vec![
        "localhost".to_string(),
        "127.0.0.1".to_string(),
        local_ip.to_string(),
    ];
    let cert = generate_simple_self_signed(subject_alt_names)?;
    let cert_pem = cert.cert.pem();
    let key_pem = cert.key_pair.serialize_pem();

    let config = RustlsConfig::from_pem(cert_pem.into_bytes(), key_pem.into_bytes()).await?;
    Ok(config)
}
