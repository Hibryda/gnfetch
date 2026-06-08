//! Desktop environment and window manager / compositor.

use std::env;

/// Desktop environment from the standard XDG hints.
pub fn desktop() -> Option<String> {
    env::var("XDG_CURRENT_DESKTOP")
        .ok()
        .or_else(|| env::var("DESKTOP_SESSION").ok())
        .filter(|s| !s.is_empty())
}

/// Window manager / compositor.
///
/// Reliable WM detection requires querying the X11/Wayland server (e.g. via
/// `_NET_SUPPORTING_WM_CHECK`), which we don't do yet. For now we report the
/// session type as a useful approximation; `None` when unknown.
pub fn wm() -> Option<String> {
    match env::var("XDG_SESSION_TYPE").ok()?.as_str() {
        "wayland" => Some("Wayland".to_string()),
        "x11" => Some("X11".to_string()),
        other if !other.is_empty() => Some(other.to_string()),
        _ => None,
    }
}
