//! Shared drawing toolkit used by every layout: a [`Pen`] for text and helpers
//! for usage bars and text truncation. A `Pen` owns a regular + bold face, so it
//! works for any font (bundled or system) — not just the embedded monospace.

use super::theme::Theme;
use ab_glyph::{Font, FontVec, PxScale, ScaleFont};
use image::{Rgba, RgbaImage};
use imageproc::drawing::{draw_filled_rect_mut, draw_text_mut};
use imageproc::rect::Rect;

static FONT_REGULAR: &[u8] = include_bytes!("../../assets/fonts/DejaVuSansMono.ttf");
static FONT_BOLD: &[u8] = include_bytes!("../../assets/fonts/DejaVuSansMono-Bold.ttf");

/// Text renderer with an owned regular and bold face.
pub struct Pen {
    regular: FontVec,
    bold: FontVec,
}

impl Default for Pen {
    fn default() -> Self {
        Self::new()
    }
}

impl Pen {
    /// The embedded DejaVu Sans Mono default (used for ASCII logos and as a fallback).
    pub fn new() -> Self {
        Self {
            regular: FontVec::try_from_vec(FONT_REGULAR.to_vec()).expect("valid embedded font"),
            bold: FontVec::try_from_vec(FONT_BOLD.to_vec()).expect("valid embedded font"),
        }
    }

    /// Build a pen from two owned faces.
    pub fn from_faces(regular: FontVec, bold: FontVec) -> Self {
        Self { regular, bold }
    }

    fn face(&self, bold: bool) -> &FontVec {
        if bold { &self.bold } else { &self.regular }
    }

    /// Draw `text` at `(x, y)` (top-left), in `size` px.
    // A low-level drawing primitive; the positional args read clearly at call sites.
    #[allow(clippy::too_many_arguments)]
    pub fn text(
        &self,
        img: &mut RgbaImage,
        x: i32,
        y: i32,
        size: f32,
        color: Rgba<u8>,
        bold: bool,
        text: &str,
    ) {
        draw_text_mut(img, color, x, y, PxScale::from(size), self.face(bold), text);
    }

    /// Advance width of `M` at `size` (a good per-char estimate for monospace).
    pub fn char_w(&self, size: f32, bold: bool) -> f32 {
        let f = self.face(bold);
        f.as_scaled(PxScale::from(size)).h_advance(f.glyph_id('M'))
    }

    /// Actual pixel width of `text` at `size`, summing real glyph advances (so it
    /// is correct for proportional fonts, not just monospace).
    pub fn text_w(&self, size: f32, bold: bool, text: &str) -> f32 {
        let f = self.face(bold);
        let scaled = f.as_scaled(PxScale::from(size));
        text.chars().map(|c| scaled.h_advance(f.glyph_id(c))).sum()
    }
}

/// Draw a usage bar: empty track plus a fill whose color escalates with `ratio`.
pub fn bar(img: &mut RgbaImage, theme: &Theme, x: i32, y: i32, w: u32, h: u32, ratio: f32) {
    draw_filled_rect_mut(img, Rect::at(x, y).of_size(w, h), theme.bar_bg);
    let r = ratio.clamp(0.0, 1.0);
    let fill = (w as f32 * r).round() as u32;
    if fill > 0 {
        let color = if r >= 0.90 {
            theme.crit
        } else if r >= 0.75 {
            theme.warn
        } else {
            theme.accent
        };
        draw_filled_rect_mut(img, Rect::at(x, y).of_size(fill, h), color);
    }
}

/// Filled rectangle convenience.
pub fn rect(img: &mut RgbaImage, x: i32, y: i32, w: u32, h: u32, color: Rgba<u8>) {
    draw_filled_rect_mut(img, Rect::at(x, y).of_size(w, h), color);
}

/// Truncate to `max` chars with a trailing ellipsis when it doesn't fit.
pub fn truncate(s: &str, max: usize) -> String {
    let count = s.chars().count();
    if max == 0 {
        return String::new();
    }
    if count <= max {
        return s.to_string();
    }
    if max == 1 {
        return "…".to_string();
    }
    let kept: String = s.chars().take(max - 1).collect();
    format!("{kept}…")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn truncate_adds_ellipsis_only_when_needed() {
        assert_eq!(truncate("abc", 5), "abc");
        assert_eq!(truncate("abcdef", 4), "abc…");
        assert_eq!(truncate("abc", 0), "");
    }

    #[test]
    fn monospace_width_is_linear() {
        let pen = Pen::new();
        let one = pen.char_w(20.0, false);
        assert!((pen.text_w(20.0, false, "abcd") - one * 4.0).abs() < 0.01);
    }
}
