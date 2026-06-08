//! Layout dispatch: resolved render settings and the entry points every layout
//! shares. Each [`Layout`](crate::config::Layout) variant maps to a builder in a
//! sibling module; this file routes to them and owns the `--save` path.

use super::background::ImageSource;
use super::fonts::{FontChoice, Fonts};
use super::theme::Theme;
use crate::config::{BackgroundMode, Fit, Layout, LogoKind};
use crate::model::SystemInfo;
use image::{Rgba, RgbaImage};
use std::path::{Path, PathBuf};

/// Scale used for `--save` PNG export — high enough to look good anywhere.
pub const SAVE_SCALE: f32 = 2.5;

// Render-scale clamps. The interactive path auto-sizes to the terminal and stays
// in a conservative band (never below 1× — viuer downscales anyway, and a
// sub-1× card would be needlessly soft). `--save --width` is an explicit user
// request, so it allows a wider range for thumbnails and large exports.
/// Min/max render scale for the auto-sized terminal path.
pub const RENDER_SCALE: (f32, f32) = (1.0, 6.0);
/// Min/max render scale for an explicit `--save --width`.
pub const SAVE_SCALE_RANGE: (f32, f32) = (0.5, 12.0);

/// Fully-resolved render settings (config + CLI overrides, theme materialized).
#[derive(Clone, Debug)]
pub struct CardSettings {
    pub layout: Layout,
    pub theme: Theme,
    /// Brand color for the logo + title (distro brand when `--brand`), else `None`
    /// to use the theme accent.
    pub brand: Option<Rgba<u8>>,
    /// Field selection / order; `None` uses the default order.
    pub fields: Option<Vec<String>>,
    /// Heading font (title, subtitle, keys) and body font (values, bars, footer).
    pub heading_font: FontChoice,
    pub body_font: FontChoice,
    pub logo: LogoKind,
    pub logo_image: Option<PathBuf>,
    pub background: BackgroundMode,
    /// Background image source, classified at resolution time.
    pub background_image: Option<ImageSource>,
    pub background_fit: Fit,
    /// Darken a background image for text legibility (false = `--no-darken`).
    pub darken_image: bool,
}

impl CardSettings {
    /// Resolve the heading + body fonts (loads system fonts if needed).
    pub fn fonts(&self) -> Fonts {
        Fonts::resolve(&self.heading_font, &self.body_font)
    }
}

/// Everything a layout needs to compose one image.
pub struct RenderParams<'a> {
    pub info: &'a SystemInfo,
    pub theme: &'a Theme,
    /// Color for the logo and title — the distro brand when `--brand`, else the
    /// theme accent. Everything else uses `theme.accent`.
    pub brand: Rgba<u8>,
    /// Field selection / order; `None` uses the default order.
    pub fields: Option<&'a [String]>,
    /// Resolved heading + body fonts.
    pub fonts: &'a Fonts,
    pub scale: f32,
    pub logo: LogoKind,
    pub logo_image: Option<&'a Path>,
    pub background: BackgroundMode,
    pub background_image: Option<&'a ImageSource>,
    pub background_fit: Fit,
    pub darken_image: bool,
}

impl<'a> RenderParams<'a> {
    /// Build params for `info` at `scale` from resolved settings + fonts.
    pub fn new(
        info: &'a SystemInfo,
        settings: &'a CardSettings,
        fonts: &'a Fonts,
        scale: f32,
    ) -> Self {
        Self {
            info,
            theme: &settings.theme,
            brand: settings.brand.unwrap_or(settings.theme.accent),
            fields: settings.fields.as_deref(),
            fonts,
            scale,
            logo: settings.logo,
            logo_image: settings.logo_image.as_deref(),
            background: settings.background,
            background_image: settings.background_image.as_ref(),
            background_fit: settings.background_fit,
            darken_image: settings.darken_image,
        }
    }

    /// Whether a logo will actually be drawn (affects layout sizing).
    pub fn has_logo(&self) -> bool {
        self.logo != LogoKind::Off
    }
}

/// Base (scale 1.0) pixel size for a layout + data, used to choose a scale.
pub fn base_size(
    layout: Layout,
    info: &SystemInfo,
    has_logo: bool,
    fields: Option<&[String]>,
    fonts: &Fonts,
) -> (u32, u32) {
    match layout {
        Layout::Card => super::card::base_size(info, fields),
        Layout::Compact => super::layout_compact::base_size(info),
        Layout::Neofetch => super::layout_neofetch::base_size(info, has_logo, fields),
        Layout::Columns => super::layout_columns::base_size(info, fields),
        Layout::Strip => super::layout_strip::base_size(info, has_logo, fonts),
    }
}

/// Compose the card for the requested layout.
pub fn render(layout: Layout, p: &RenderParams) -> RgbaImage {
    match layout {
        Layout::Card => super::card::render(p),
        Layout::Compact => super::layout_compact::render(p),
        Layout::Neofetch => super::layout_neofetch::render(p),
        Layout::Columns => super::layout_columns::render(p),
        Layout::Strip => super::layout_strip::render(p),
    }
}

/// Render the card and write it to `path` (format from the extension).
///
/// `width` sets the target output width in pixels; `None` uses [`SAVE_SCALE`].
/// The exact pixel width may differ by a few px because layouts round at scale.
pub fn save(
    info: &SystemInfo,
    settings: &CardSettings,
    path: &Path,
    width: Option<u32>,
) -> image::ImageResult<()> {
    let fonts = settings.fonts();
    let scale = match width {
        Some(w) => {
            let (base_w, _) = base_size(
                settings.layout,
                info,
                settings.logo != LogoKind::Off,
                settings.fields.as_deref(),
                &fonts,
            );
            // Clamp so a tiny width stays legible and a huge one stays sane.
            (w as f32 / base_w.max(1) as f32).clamp(SAVE_SCALE_RANGE.0, SAVE_SCALE_RANGE.1)
        }
        None => SAVE_SCALE,
    };
    let p = RenderParams::new(info, settings, &fonts, scale);
    render(settings.layout, &p).save(path)
}
