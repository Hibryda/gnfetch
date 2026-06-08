//! Terminal inline-graphics capability detection.
//!
//! Detection here is heuristic, based on environment variables and `$TERM` —
//! the same approach tools like `viuer` and `timg` use. A more rigorous probe
//! (sending a query escape and reading the reply) can replace this later
//! without changing the renderer interface.

use std::env;

#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub enum GraphicsProtocol {
    /// Kitty graphics protocol (Kitty, WezTerm, Konsole, ...).
    Kitty,
    /// iTerm2 inline images (OSC 1337).
    ITerm2,
    /// Sixel raster graphics.
    Sixel,
    /// No inline-graphics support detected.
    None,
}

impl GraphicsProtocol {
    pub fn label(self) -> &'static str {
        match self {
            GraphicsProtocol::Kitty => "Kitty graphics protocol",
            GraphicsProtocol::ITerm2 => "iTerm2 inline images",
            GraphicsProtocol::Sixel => "Sixel",
            GraphicsProtocol::None => "none",
        }
    }
}

/// Best-effort detection of the terminal's inline-graphics protocol.
pub fn detect_graphics() -> GraphicsProtocol {
    let term = env::var("TERM").unwrap_or_default();
    let term_program = env::var("TERM_PROGRAM").unwrap_or_default();

    // Kitty graphics protocol.
    if env::var_os("KITTY_WINDOW_ID").is_some() || term.contains("kitty") {
        return GraphicsProtocol::Kitty;
    }
    // WezTerm implements the Kitty protocol.
    if term_program == "WezTerm" {
        return GraphicsProtocol::Kitty;
    }

    // iTerm2 inline images.
    if term_program == "iTerm.app" || env::var_os("ITERM_SESSION_ID").is_some() {
        return GraphicsProtocol::ITerm2;
    }

    // Terminals commonly built with Sixel support.
    if term.contains("sixel")
        || term.contains("foot")
        || term.contains("mlterm")
        || term.contains("yaft")
    {
        return GraphicsProtocol::Sixel;
    }

    GraphicsProtocol::None
}
