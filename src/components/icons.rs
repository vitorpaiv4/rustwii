use dioxus::prelude::*;

#[component]
pub fn DiscIcon() -> Element {
    rsx! {
        svg {
            view_box: "0 0 48 48",
            width: "38",
            height: "38",
            fill: "none",
            stroke: "currentColor",
            stroke_width: "2.5",
            stroke_linecap: "round",
            stroke_linejoin: "round",
            circle { cx: "24", cy: "24", r: "20" }
            circle { cx: "24", cy: "24", r: "6" }
            path { d: "M24 4a20 20 0 0 1 14.14 5.86" }
            path { d: "M24 44a20 20 0 0 1-14.14-5.86" }
        }
    }
}

#[component]
pub fn TargetIcon() -> Element {
    rsx! {
        svg {
            view_box: "0 0 48 48",
            width: "38",
            height: "38",
            fill: "none",
            stroke: "currentColor",
            stroke_width: "2.5",
            stroke_linecap: "round",
            stroke_linejoin: "round",
            circle { cx: "24", cy: "24", r: "18" }
            circle { cx: "24", cy: "24", r: "11" }
            circle { cx: "24", cy: "24", r: "4", fill: "currentColor" }
            line { x1: "24", y1: "2", x2: "24", y2: "9" }
            line { x1: "24", y1: "39", x2: "24", y2: "46" }
            line { x1: "2", y1: "24", x2: "9", y2: "24" }
            line { x1: "39", y1: "24", x2: "46", y2: "24" }
        }
    }
}

#[component]
pub fn RemoteIcon() -> Element {
    rsx! {
        svg {
            view_box: "0 0 48 48",
            width: "36",
            height: "36",
            fill: "none",
            stroke: "currentColor",
            stroke_width: "2.5",
            stroke_linecap: "round",
            stroke_linejoin: "round",
            rect { x: "14", y: "4", width: "20", height: "40", rx: "6" }
            circle { cx: "24", cy: "14", r: "4" }
            circle { cx: "24", cy: "26", r: "2" }
            circle { cx: "24", cy: "34", r: "2" }
            line { x1: "20", y1: "40", x2: "28", y2: "40" }
        }
    }
}

#[component]
pub fn MiiIcon() -> Element {
    rsx! {
        svg {
            view_box: "0 0 48 48",
            width: "38",
            height: "38",
            fill: "none",
            stroke: "currentColor",
            stroke_width: "2.5",
            stroke_linecap: "round",
            stroke_linejoin: "round",
            circle { cx: "24", cy: "16", r: "10" }
            path { d: "M8 42c0-8.837 7.163-16 16-16s16 7.163 16 16" }
        }
    }
}

#[component]
pub fn WeatherIcon() -> Element {
    rsx! {
        svg {
            view_box: "0 0 48 48",
            width: "38",
            height: "38",
            fill: "none",
            stroke: "currentColor",
            stroke_width: "2.5",
            stroke_linecap: "round",
            stroke_linejoin: "round",
            circle { cx: "24", cy: "24", r: "10" }
            line { x1: "24", y1: "4", x2: "24", y2: "8" }
            line { x1: "24", y1: "40", x2: "24", y2: "44" }
            line { x1: "4", y1: "24", x2: "8", y2: "24" }
            line { x1: "40", y1: "24", x2: "44", y2: "24" }
            line { x1: "9.86", y1: "9.86", x2: "12.69", y2: "12.69" }
            line { x1: "35.31", y1: "35.31", x2: "38.14", y2: "38.14" }
            line { x1: "9.86", y1: "38.14", x2: "12.69", y2: "35.31" }
            line { x1: "35.31", y1: "12.69", x2: "38.14", y2: "9.86" }
        }
    }
}

#[component]
pub fn NewsIcon() -> Element {
    rsx! {
        svg {
            view_box: "0 0 48 48",
            width: "38",
            height: "38",
            fill: "none",
            stroke: "currentColor",
            stroke_width: "2.5",
            stroke_linecap: "round",
            stroke_linejoin: "round",
            rect { x: "8", y: "8", width: "32", height: "32", rx: "4" }
            line { x1: "14", y1: "16", x2: "24", y2: "16" }
            line { x1: "14", y1: "24", x2: "34", y2: "24" }
            line { x1: "14", y1: "32", x2: "34", y2: "32" }
        }
    }
}

#[component]
pub fn EmptyChannelIcon() -> Element {
    rsx! {
        svg {
            view_box: "0 0 48 48",
            width: "32",
            height: "32",
            fill: "none",
            stroke: "currentColor",
            stroke_width: "2",
            stroke_dasharray: "4 4",
            stroke_linecap: "round",
            stroke_linejoin: "round",
            rect { x: "8", y: "10", width: "32", height: "28", rx: "4" }
        }
    }
}

#[component]
pub fn MailIcon() -> Element {
    rsx! {
        svg {
            view_box: "0 0 24 24",
            width: "20",
            height: "20",
            fill: "none",
            stroke: "currentColor",
            stroke_width: "2",
            stroke_linecap: "round",
            stroke_linejoin: "round",
            path { d: "M4 4h16c1.1 0 2 .9 2 2v12c0 1.1-.9 2-2 2H4c-1.1 0-2-.9-2-2V6c0-1.1.9-2 2-2z" }
            polyline { points: "22,6 12,13 2,6" }
        }
    }
}

#[component]
pub fn QrIcon() -> Element {
    rsx! {
        svg {
            view_box: "0 0 24 24",
            width: "18",
            height: "18",
            fill: "none",
            stroke: "currentColor",
            stroke_width: "2",
            stroke_linecap: "round",
            stroke_linejoin: "round",
            rect { x: "3", y: "3", width: "7", height: "7" }
            rect { x: "14", y: "3", width: "7", height: "7" }
            rect { x: "3", y: "14", width: "7", height: "7" }
            line { x1: "14", y1: "14", x2: "14", y2: "14.01" }
            line { x1: "17", y1: "17", x2: "17", y2: "17.01" }
            line { x1: "21", y1: "21", x2: "21", y2: "21.01" }
        }
    }
}

#[component]
pub fn LinkIcon() -> Element {
    rsx! {
        svg {
            view_box: "0 0 24 24",
            width: "16",
            height: "16",
            fill: "none",
            stroke: "currentColor",
            stroke_width: "2",
            stroke_linecap: "round",
            stroke_linejoin: "round",
            path { d: "M18 13v6a2 2 0 0 1-2 2H5a2 2 0 0 1-2-2V8a2 2 0 0 1 2-2h6" }
            polyline { points: "15 3 21 3 21 9" }
            line { x1: "10", y1: "14", x2: "21", y2: "3" }
        }
    }
}
