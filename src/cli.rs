//! Command-line interface definition. Flags override `~/.config/gnfetch/config.toml`.

use crate::config::{Fit, Layout, LogoKind};
use clap::{Parser, ValueEnum};
use std::path::PathBuf;

/// A neofetch/fastfetch alternative with TUI and inline-graphics system cards.
#[derive(Parser, Debug)]
#[command(name = "gnfetch", version, about, long_about = None)]
pub struct Cli {
    /// Output mode: auto-detect, classic ANSI, or graphical card.
    #[arg(short, long, value_enum, default_value_t = Mode::Auto)]
    pub mode: Mode,

    /// Card layout (overrides config).
    #[arg(short, long, value_enum)]
    pub layout: Option<Layout>,

    /// Theme name (overrides config); see --list-themes.
    #[arg(short, long)]
    pub theme: Option<String>,

    /// Accent color override as #rrggbb.
    #[arg(long, value_name = "HEX")]
    pub accent: Option<String>,

    /// Keep the distro's brand color for the logo and title, even with another theme.
    #[arg(long)]
    pub brand: bool,

    /// Use a light theme (light background, dark text); default is dark.
    #[arg(long)]
    pub light: bool,

    /// Heading font (title/keys): a font name, or the flag alone for the system sans.
    #[arg(long, num_args = 0..=1, default_missing_value = "", value_name = "FONT")]
    pub sans: Option<String>,

    /// Heading font as a serif: a font name, or the flag alone for the system serif.
    #[arg(long, num_args = 0..=1, default_missing_value = "", value_name = "FONT")]
    pub serif: Option<String>,

    /// Body font (values/bars): a font name, or the flag alone for the system monospace.
    #[arg(long, num_args = 0..=1, default_missing_value = "", value_name = "FONT")]
    pub mono: Option<String>,

    /// List bundled fonts and exit.
    #[arg(long)]
    pub list_fonts: bool,

    /// Render a gallery of example cards (dummy data) showcasing the options, then exit.
    #[arg(long)]
    pub demo: bool,

    /// Fields to show, in order (comma-separated); see --list-fields.
    #[arg(long, value_name = "LIST", value_delimiter = ',')]
    pub fields: Option<Vec<String>>,

    /// List the available field keys and exit.
    #[arg(long)]
    pub list_fields: bool,

    /// Logo source: drawn | ascii | image | off.
    #[arg(long, value_enum)]
    pub logo: Option<LogoKind>,

    /// Logo image file (use with --logo image).
    #[arg(long, value_name = "PATH")]
    pub logo_image: Option<PathBuf>,

    /// Override the detected distro id (preview another distro's logo).
    #[arg(long, value_name = "ID")]
    pub distro: Option<String>,

    /// Render a fictional example system instead of this machine (for
    /// screenshots/testing); optionally pick a distro, e.g. --mock arch.
    #[arg(long, num_args = 0..=1, default_missing_value = "", value_name = "DISTRO")]
    pub mock: Option<String>,

    /// Background: solid | gradient | diagonal | radial | grid | dots | image |
    /// transparent | linear-<angle> (e.g. linear-30).
    #[arg(long, value_name = "SPEC")]
    pub background: Option<String>,

    /// Background image: a file path, an https:// URL, or a bundled name (see
    /// --list-backgrounds). Setting this implies `--background image`.
    #[arg(long, value_name = "FILE|URL|NAME")]
    pub background_image: Option<String>,

    /// How a background image is scaled: fill | fit | stretch | center (default fill).
    #[arg(long, value_enum, value_name = "FIT")]
    pub background_fit: Option<Fit>,

    /// Don't darken the background image (default dims it for text legibility).
    #[arg(long)]
    pub no_darken: bool,

    /// List the bundled background image names, then exit.
    #[arg(long)]
    pub list_backgrounds: bool,

    /// Write the graphical card to an image file (e.g. card.png) and exit.
    #[arg(short = 'o', long, value_name = "PATH")]
    pub save: Option<PathBuf>,

    /// Target width in pixels for the saved card (`--save` only; default auto).
    #[arg(long, value_name = "PX", requires = "save")]
    pub width: Option<u32>,

    /// Print terminal detection and the chosen card dimensions, then exit.
    #[arg(long)]
    pub probe: bool,

    /// List available themes and exit.
    #[arg(long)]
    pub list_themes: bool,

    /// List available layouts and exit.
    #[arg(long)]
    pub list_layouts: bool,
}

#[derive(Copy, Clone, Debug, PartialEq, Eq, ValueEnum)]
pub enum Mode {
    /// Detect terminal capabilities and choose the best renderer.
    Auto,
    /// Classic ASCII/ANSI output (the neofetch-style path).
    Ansi,
    /// Rich graphical system card (requires a capable terminal).
    Image,
}
