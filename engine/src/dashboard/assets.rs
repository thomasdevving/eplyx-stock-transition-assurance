//! Dashboard browser assets, embedded at compile time so `eplyx dashboard`
//! needs no Node runtime, network fonts or files outside the binary. Shared
//! modules come straight from the main frontend to keep one visual identity.
macro_rules! frontend {
    ($path:literal) => {
        include_str!(concat!(env!("CARGO_MANIFEST_DIR"), "/../frontend/", $path))
    };
}

pub const INDEX: &str = frontend!("dashboard/index.html");

const ASSETS: &[(&str, &str, &str)] = &[
    (
        "dashboard.css",
        "text/css; charset=utf-8",
        frontend!("dashboard/dashboard.css"),
    ),
    (
        "dashboard.js",
        "text/javascript; charset=utf-8",
        frontend!("dashboard/dashboard.js"),
    ),
    (
        "pages.js",
        "text/javascript; charset=utf-8",
        frontend!("dashboard/pages.js"),
    ),
    (
        "ui.js",
        "text/javascript; charset=utf-8",
        frontend!("dashboard/ui.js"),
    ),
    (
        "mode.js",
        "text/javascript; charset=utf-8",
        frontend!("src/mode.js"),
    ),
    (
        "format.js",
        "text/javascript; charset=utf-8",
        frontend!("src/format.js"),
    ),
    (
        "brand.js",
        "text/javascript; charset=utf-8",
        frontend!("src/brand.js"),
    ),
    ("logo.svg", "image/svg+xml", frontend!("public/logo.svg")),
];

pub fn get(name: &str) -> Option<(&'static str, &'static [u8])> {
    ASSETS
        .iter()
        .find(|(asset, ..)| *asset == name)
        .map(|(_, kind, body)| (*kind, body.as_bytes()))
}
