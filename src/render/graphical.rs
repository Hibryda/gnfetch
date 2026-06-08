//! Graphical "system visiting card" renderer.
//!
//! Measures the terminal, sizes the card to a generous fraction of the width
//! (height unconstrained — a tall card scrolls), renders the configured layout
//! at the matching pixel resolution, and emits it inline via `viuer` (Kitty
//! graphics protocol, iTerm2 inline images, or Sixel; viuer degrades to Unicode
//! half-blocks). On any failure it falls back to the ANSI renderer.

use super::Renderer;
use super::ansi::AnsiRenderer;
use super::caps::GraphicsProtocol;
use super::layout::{self, CardSettings, RenderParams};
use super::termsize::{TermSize, term_size};
use crate::model::SystemInfo;
use image::DynamicImage;
use std::io;

/// Fraction of the terminal width the card occupies. The card is sized by width
/// and may be taller than the viewport (it scrolls) — the chosen behaviour for
/// wide, short terminals where a height-fit card would look tiny.
const WIDTH_FRACTION: f32 = 0.72;
/// Assumed cell width in px when the terminal doesn't report pixel size.
const ASSUMED_CELL_PX_W: f32 = 9.0;
/// Assumed cell height:width ratio when the terminal doesn't report pixels.
const ASSUMED_CELL_ASPECT: f32 = 2.0;
/// Card width in cells when the terminal size is unknown (piped output).
const FALLBACK_CELLS: u32 = 80;
/// Render scale when the terminal size is unknown (piped output).
const FALLBACK_SCALE: f32 = 2.0;
/// Clamp for the chosen display width, in cells.
const MIN_CELLS: f32 = 40.0;
const MAX_CELLS: f32 = 240.0;

pub struct GraphicalRenderer {
    protocol: GraphicsProtocol,
    settings: CardSettings,
}

impl GraphicalRenderer {
    pub fn new(protocol: GraphicsProtocol, settings: CardSettings) -> Self {
        Self { protocol, settings }
    }
}

impl Renderer for GraphicalRenderer {
    fn render(&self, info: &SystemInfo) -> io::Result<()> {
        let fonts = self.settings.fonts();
        let (scale, width_cells, height_cells) = plan(info, &self.settings, &fonts, term_size());
        let params = RenderParams::new(info, &self.settings, &fonts, scale);
        let card = DynamicImage::ImageRgba8(layout::render(self.settings.layout, &params));

        let config = viuer::Config {
            absolute_offset: false,
            width: Some(width_cells),
            height: height_cells,
            ..Default::default()
        };

        match viuer::print(&card, &config) {
            Ok(_) => Ok(()),
            Err(err) => {
                eprintln!(
                    "gnfetch: inline graphics failed via {} ({err}); falling back to ANSI.",
                    self.protocol.label()
                );
                let brand = self.settings.brand.unwrap_or(self.settings.theme.accent);
                AnsiRenderer::new(self.settings.theme, brand, self.settings.fields.clone())
                    .render(info)
            }
        }
    }
}

/// Human-readable diagnostics for the `--probe` flag.
pub fn describe(info: &SystemInfo, settings: &CardSettings) -> String {
    let term = term_size();
    let fonts = settings.fonts();
    let has_logo = settings.logo != crate::config::LogoKind::Off;
    let (base_w, base_h) = layout::base_size(
        settings.layout,
        info,
        has_logo,
        settings.fields.as_deref(),
        &fonts,
    );
    let (scale, width_cells, height_cells) = plan(info, settings, &fonts, term);
    let term_str = match term {
        Some(t) => format!(
            "cols={} rows={} px={}x{} cell_px={}",
            t.cols,
            t.rows,
            t.px_w,
            t.px_h,
            match t.cell_px() {
                Some((w, h)) => format!("{w:.1}x{h:.1}"),
                None => "unreported".to_string(),
            }
        ),
        None => "not a terminal (stdout is piped/redirected)".to_string(),
    };
    let height_str = match height_cells {
        Some(h) => h.to_string(),
        None => "auto".to_string(),
    };
    format!(
        "layout: {:?}  theme: {}  logo: {:?}\nterminal: {term_str}\ncard base: {base_w}x{base_h}px\nchosen: {width_cells}x{height_str} cells, render scale={scale:.2} (image {}x{}px)",
        settings.layout,
        settings.theme.name,
        settings.logo,
        (base_w as f32 * scale).round() as u32,
        (base_h as f32 * scale).round() as u32,
    )
}

/// Decide the render scale and display size (width, height) in cells.
///
/// Crucially, the height is computed from the terminal's *real* cell aspect
/// (cell_w / cell_h) so the card keeps its true proportions. Passing both width
/// and height to viuer bypasses its hardcoded 2:1 cell assumption, which would
/// otherwise squeeze the image horizontally on terminals with taller cells.
fn plan(
    info: &SystemInfo,
    settings: &CardSettings,
    fonts: &super::fonts::Fonts,
    term: Option<TermSize>,
) -> (f32, u32, Option<u32>) {
    let has_logo = settings.logo != crate::config::LogoKind::Off;
    let (base_w, base_h) = layout::base_size(
        settings.layout,
        info,
        has_logo,
        settings.fields.as_deref(),
        fonts,
    );

    let Some(term) = term.filter(|t| t.cols > 0) else {
        // Piped: let viuer compute the height with its default assumption.
        return (FALLBACK_SCALE, FALLBACK_CELLS, None);
    };

    let (cell_w, cell_h) = term
        .cell_px()
        .unwrap_or((ASSUMED_CELL_PX_W, ASSUMED_CELL_PX_W * ASSUMED_CELL_ASPECT));

    let width_cells = (term.cols as f32 * WIDTH_FRACTION)
        .clamp(MIN_CELLS, MAX_CELLS)
        .min(term.cols as f32);

    let scale = (width_cells * cell_w / base_w as f32)
        .clamp(layout::RENDER_SCALE.0, layout::RENDER_SCALE.1);

    // Height in cells that preserves the card's pixel aspect at this cell aspect:
    //   width_px / height_px = base_w / base_h, with px = cells * cell size.
    // Round up so short cards (e.g. the strip) are never under-sized and clipped.
    let height_cells = (width_cells * (cell_w / cell_h) * (base_h as f32 / base_w.max(1) as f32))
        .ceil()
        .max(1.0) as u32;

    (scale, width_cells.round() as u32, Some(height_cells))
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::config::{BackgroundMode, Fit, Layout, LogoKind};
    use crate::render::fonts::{FontChoice, Fonts, Generic};
    use crate::render::theme::Theme;

    fn settings() -> CardSettings {
        CardSettings {
            layout: Layout::Card,
            theme: Theme::default_theme(),
            brand: None,
            fields: None,
            // Bundled fonts so tests never scan system fonts.
            heading_font: FontChoice {
                name: Some("poppins".to_string()),
                generic: Generic::Sans,
            },
            body_font: FontChoice {
                name: Some("dejavu-mono".to_string()),
                generic: Generic::Mono,
            },
            logo: LogoKind::Off,
            logo_image: None,
            background: BackgroundMode::Solid,
            background_image: None,
            background_fit: Fit::Fill,
            darken_image: true,
        }
    }

    #[test]
    fn piped_output_uses_fallback() {
        let s = settings();
        let f = Fonts::resolve(&s.heading_font, &s.body_font);
        let (scale, cells, height) = plan(&SystemInfo::default(), &s, &f, None);
        assert_eq!(cells, FALLBACK_CELLS);
        assert!(scale >= 1.0);
        assert_eq!(height, None); // let viuer compute the height when piped
    }

    #[test]
    fn wide_terminal_grows_within_clamp() {
        let term = TermSize {
            cols: 200,
            rows: 50,
            px_w: 2000,
            px_h: 1200,
        };
        let s = settings();
        let f = Fonts::resolve(&s.heading_font, &s.body_font);
        let (_scale, cells, height) = plan(&SystemInfo::default(), &s, &f, Some(term));
        assert!((MIN_CELLS as u32..=MAX_CELLS as u32).contains(&cells));
        assert!(cells > FALLBACK_CELLS);
        assert!(height.is_some_and(|h| h > 0));
    }

    #[test]
    fn narrow_terminal_never_exceeds_columns() {
        let term = TermSize {
            cols: 60,
            rows: 50,
            px_w: 540,
            px_h: 1000,
        };
        let s = settings();
        let f = Fonts::resolve(&s.heading_font, &s.body_font);
        let (_scale, cells, _height) = plan(&SystemInfo::default(), &s, &f, Some(term));
        assert!(cells <= 60);
    }

    #[test]
    fn taller_cells_yield_fewer_rows() {
        // A terminal with taller cells (2.4:1) should reserve fewer rows than a
        // 2:1 terminal for the same width — that's what fixes the squeeze.
        let base = TermSize {
            cols: 200,
            rows: 50,
            px_w: 2000,
            px_h: 2000,
        }; // cell 10x40 (4:1)
        let square = TermSize {
            cols: 200,
            rows: 50,
            px_w: 2000,
            px_h: 1000,
        }; // cell 10x20 (2:1)
        let s = settings();
        let f = Fonts::resolve(&s.heading_font, &s.body_font);
        let (_, _, h_tall) = plan(&SystemInfo::default(), &s, &f, Some(base));
        let (_, _, h_norm) = plan(&SystemInfo::default(), &s, &f, Some(square));
        assert!(h_tall.unwrap() < h_norm.unwrap());
    }
}
