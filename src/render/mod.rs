//! Rendering layer.
//!
//! Renderers consume a [`SystemInfo`] and write to stdout. [`select_renderer`]
//! picks one based on the requested [`Mode`] and the detected terminal graphics
//! capability, always with the ANSI renderer as the safe fallback. The graphical
//! path composes a themed card via the [`layout`] modules.

mod ansi;
mod ascii_logos;
mod background;
mod bg_images;
mod caps;
mod card;
mod content;
mod draw;
mod fonts;
mod format;
mod graphical;
mod layout;
mod layout_columns;
mod layout_compact;
mod layout_neofetch;
mod layout_strip;
mod logo;
mod svg_logos;
mod termsize;
mod theme;

pub use background::ImageSource;
pub use bg_images::names as background_names;
pub use caps::{GraphicsProtocol, detect_graphics};
pub use content::AVAILABLE as FIELD_KEYS;
pub use fonts::{FontChoice, Generic, bundled_names};
pub use graphical::describe as describe_plan;
pub use layout::{CardSettings, save};
pub use theme::{Theme, parse_hex};

use crate::cli::Mode;
use crate::model::SystemInfo;
use std::io;

/// Anything that can turn a [`SystemInfo`] into terminal output.
pub trait Renderer {
    fn render(&self, info: &SystemInfo) -> io::Result<()>;
}

/// Choose a renderer from the CLI mode and detected terminal capabilities.
pub fn select_renderer(mode: Mode, settings: CardSettings) -> Box<dyn Renderer> {
    let brand = settings.brand.unwrap_or(settings.theme.accent);
    match mode {
        Mode::Ansi => Box::new(ansi::AnsiRenderer::new(
            settings.theme,
            brand,
            settings.fields.clone(),
        )),
        Mode::Image => Box::new(graphical::GraphicalRenderer::new(
            detect_graphics(),
            settings,
        )),
        Mode::Auto => match detect_graphics() {
            GraphicsProtocol::None => Box::new(ansi::AnsiRenderer::new(
                settings.theme,
                brand,
                settings.fields.clone(),
            )),
            proto => Box::new(graphical::GraphicalRenderer::new(proto, settings)),
        },
    }
}
