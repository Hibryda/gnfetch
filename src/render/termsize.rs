//! Terminal dimensions, in character cells and (when reported) pixels.
//!
//! Uses the `TIOCGWINSZ` ioctl via `rustix` (no `unsafe`). Modern terminals
//! such as WezTerm, Kitty and foot report pixel dimensions too, which lets the
//! graphical renderer size the card to the available space and render it at the
//! exact display resolution for crisp output.

/// Measured size of the controlling terminal.
#[derive(Clone, Copy, Debug)]
pub struct TermSize {
    pub cols: u16,
    pub rows: u16,
    /// Width in pixels, or 0 when the terminal doesn't report it.
    pub px_w: u16,
    /// Height in pixels, or 0 when the terminal doesn't report it.
    pub px_h: u16,
}

impl TermSize {
    /// Pixel size of one character cell, if the terminal reports pixels.
    pub fn cell_px(&self) -> Option<(f32, f32)> {
        if self.px_w > 0 && self.px_h > 0 && self.cols > 0 && self.rows > 0 {
            Some((
                self.px_w as f32 / self.cols as f32,
                self.px_h as f32 / self.rows as f32,
            ))
        } else {
            None
        }
    }
}

/// Query the terminal on stdout. Returns `None` when stdout isn't a terminal
/// (e.g. piped or redirected) or the platform doesn't support the ioctl.
#[cfg(unix)]
pub fn term_size() -> Option<TermSize> {
    let ws = rustix::termios::tcgetwinsize(std::io::stdout()).ok()?;
    if ws.ws_col == 0 {
        return None;
    }
    Some(TermSize {
        cols: ws.ws_col,
        rows: ws.ws_row,
        px_w: ws.ws_xpixel,
        px_h: ws.ws_ypixel,
    })
}

#[cfg(not(unix))]
pub fn term_size() -> Option<TermSize> {
    None
}
