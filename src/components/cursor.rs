use dioxus::prelude::*;
use crate::inertial::CursorState;

#[component]
pub fn WiiCursor(cursor: CursorState) -> Element {
    if !cursor.is_active {
        return rsx! {};
    }

    let color = cursor.color();
    let is_clicking = cursor.is_clicking;
    let scale = if is_clicking { 0.88 } else { 1.0 };

    let style = format!(
        "left: {:.2}%; top: {:.2}%; transform: translate(-30%, -20%) rotate({:.1}deg) scale({:.2});",
        cursor.x, cursor.y, cursor.rotation_deg, scale
    );

    let click_class = if is_clicking { "wii-cursor-clicking" } else { "" };

    rsx! {
        div {
            class: "wii-cursor-wrapper {click_class}",
            style: "{style}",

            // Wii Hand Pointer SVG
            svg {
                class: "wii-hand-svg",
                view_box: "0 0 100 120",
                width: "48",
                height: "58",
                filter: "drop-shadow(0 4px 8px rgba(0,0,0,0.35))",

                // Glove Body Outline & Fill
                path {
                    d: "M 32 15 C 32 5, 48 5, 48 15 L 48 48 C 52 45, 62 45, 64 52 C 67 48, 77 49, 78 57 C 81 54, 89 57, 88 66 C 87 78, 80 95, 65 105 C 50 115, 30 110, 24 100 C 18 90, 16 75, 20 65 L 20 62 C 16 62, 10 55, 14 45 C 18 35, 32 40, 32 48 Z",
                    fill: "#ffffff",
                    stroke: color.primary,
                    stroke_width: "5",
                    stroke_linejoin: "round",
                }

                // Inner Shadow / Accent
                path {
                    d: "M 48 15 L 48 48 M 64 52 L 64 68 M 78 57 L 78 72",
                    stroke: color.primary,
                    stroke_width: "3",
                    stroke_linecap: "round",
                }

                // Player Badge on Palm
                circle {
                    cx: "48",
                    cy: "82",
                    r: "14",
                    fill: color.primary,
                }

                // Player Text (P1, P2, P3, P4)
                text {
                    x: "48",
                    y: "87",
                    text_anchor: "middle",
                    font_family: "system-ui, sans-serif",
                    font_size: "14",
                    font_weight: "900",
                    fill: "#ffffff",
                    "P{cursor.player_id}"
                }
            }

            // Click ripple ring when pressing A
            if is_clicking {
                div {
                    class: "wii-cursor-ripple",
                    style: "border-color: {color.primary}; box-shadow: 0 0 12px {color.glow};"
                }
            }
        }
    }
}
