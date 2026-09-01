use dioxus::prelude::*;
use qrcode::render::svg;
use qrcode::QrCode;

pub fn generate_qr_svg_string(content: &str) -> Result<String, String> {
    let code = QrCode::new(content.as_bytes()).map_err(|e| format!("Erro QR: {:?}", e))?;
    let svg_str = code
        .render::<svg::Color>()
        .min_dimensions(180, 180)
        .dark_color(svg::Color("#0f172a"))
        .light_color(svg::Color("#ffffff"))
        .build();
    Ok(svg_str)
}

#[component]
pub fn QrCodeView(url: String) -> Element {
    let svg_content = match generate_qr_svg_string(&url) {
        Ok(svg) => svg,
        Err(err) => format!("<p style='color:red;'>{}</p>", err),
    };

    rsx! {
        div {
            class: "wii-qrcode-container",
            div {
                class: "wii-qrcode-svg",
                dangerous_inner_html: "{svg_content}",
            }
            p { class: "wii-qrcode-label", "Aponte a câmera para parear" }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_qr_code_generation() {
        let svg = generate_qr_svg_string("https://rustwii.local/remote/WII-001");
        assert!(svg.is_ok());
        let content = svg.unwrap();
        assert!(content.contains("<svg"));
        assert!(content.contains("</svg>"));
    }
}
