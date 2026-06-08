//! Embedded CC0 / public-domain background images. DO NOT EDIT BY HAND.
//! Licensing + source per file: assets/backgrounds/LICENSE.txt.
//! Regenerate via scripts/gen_backgrounds.py.

/// Names of the bundled background images, for `--list-backgrounds`.
pub fn names() -> &'static [&'static str] {
    &["andromeda", "aurora", "carina", "earth", "helix"]
}

/// Embedded image bytes for a bundled name (case-insensitive), or `None`.
pub fn bundled(name: &str) -> Option<&'static [u8]> {
    match name.trim().to_ascii_lowercase().as_str() {
        "andromeda" => Some(include_bytes!("../../assets/backgrounds/andromeda.jpg")),
        "aurora" => Some(include_bytes!("../../assets/backgrounds/aurora.jpg")),
        "carina" => Some(include_bytes!("../../assets/backgrounds/carina.jpg")),
        "earth" => Some(include_bytes!("../../assets/backgrounds/earth.jpg")),
        "helix" => Some(include_bytes!("../../assets/backgrounds/helix.jpg")),
        _ => None,
    }
}
