//! User configuration: layout, theme, logo and background.
//!
//! Defaults live here, are optionally overridden by `~/.config/gnfetch/config.toml`
//! (or `$XDG_CONFIG_HOME/gnfetch/config.toml`), and are finally overridden by CLI
//! flags. The enums are shared by `clap` (CLI parsing) and `serde` (the config file).

use clap::ValueEnum;
use serde::Deserialize;
use std::path::PathBuf;

/// Card layout style.
#[derive(Copy, Clone, Debug, PartialEq, Eq, Default, ValueEnum, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum Layout {
    /// Single column: header, key/value rows, usage bars (the default).
    #[default]
    Card,
    /// Logo panel on the left, info column on the right (classic fetch).
    Neofetch,
    /// Header on top, info packed into multiple columns.
    Columns,
    /// Short, wide horizontal band.
    Strip,
    /// Minimal: just the essentials, no bars.
    Compact,
}

/// Where the logo comes from.
#[derive(Copy, Clone, Debug, PartialEq, Eq, Default, ValueEnum, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum LogoKind {
    /// Built-in vector-style emblem drawn in code (the default).
    #[default]
    Drawn,
    /// Neofetch-style ASCII art rendered as text.
    Ascii,
    /// A user-supplied image file (see `logo_image`).
    Image,
    /// No logo.
    Off,
}

/// Card background style. Parsed from a string so `linear-<angle>` is accepted.
#[derive(Copy, Clone, Debug, PartialEq, Default)]
pub enum BackgroundMode {
    /// Solid theme background (the default).
    #[default]
    Solid,
    /// Linear gradient at `angle` degrees (0 = left→right, 90 = top→bottom).
    Linear(f32),
    /// Radial glow: brighter at the center, darker toward the edges.
    Radial,
    /// Subtle grid lines over the solid background.
    Grid,
    /// Subtle dot grid over the solid background.
    Dots,
    /// A user-supplied image file (see `background_image`), darkened for contrast.
    Image,
    /// Fully transparent background.
    Transparent,
}

impl BackgroundMode {
    /// Parse a background spec: `solid`, `gradient` (= `linear-90`), `diagonal`
    /// (= `linear-45`), `horizontal`, `radial`, `grid`, `dots`, `image`,
    /// `transparent`, or `linear-<angle>` for any angle in degrees.
    pub fn parse(s: &str) -> Option<BackgroundMode> {
        match s.trim().to_ascii_lowercase().as_str() {
            "solid" => Some(Self::Solid),
            "gradient" | "vertical" => Some(Self::Linear(90.0)),
            "diagonal" => Some(Self::Linear(45.0)),
            "horizontal" => Some(Self::Linear(0.0)),
            "radial" => Some(Self::Radial),
            "grid" => Some(Self::Grid),
            "dots" => Some(Self::Dots),
            "image" => Some(Self::Image),
            "transparent" | "none" => Some(Self::Transparent),
            other => other
                .strip_prefix("linear")
                .map(|a| a.trim_start_matches([':', '-', '=', ' ']))
                .and_then(|a| a.parse::<f32>().ok())
                .filter(|a| a.is_finite()) // reject linear-nan / linear-inf
                .map(Self::Linear),
        }
    }
}

/// How a background image is scaled to fit the card.
#[derive(Copy, Clone, Debug, PartialEq, Eq, Default, ValueEnum, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum Fit {
    /// Scale to cover the card, cropping the overflow (preserves aspect). Default.
    #[default]
    Fill,
    /// Scale to fit inside the card, letterboxing the rest (preserves aspect).
    #[value(name = "fit", alias = "contain")]
    #[serde(rename = "fit", alias = "contain")]
    Contain,
    /// Scale each axis to fill the card exactly (distorts aspect).
    Stretch,
    /// No scaling — center the image, cropping or padding as needed.
    Center,
}

/// Fully-resolved configuration.
#[derive(Clone, Debug, Deserialize)]
#[serde(default, rename_all = "kebab-case")]
pub struct Config {
    pub layout: Layout,
    /// Theme name (resolved against `render::theme::Theme::by_name`).
    pub theme: String,
    /// Optional `#rrggbb` accent override.
    pub accent: Option<String>,
    pub logo: LogoKind,
    pub logo_image: Option<PathBuf>,
    /// Background spec string (e.g. "gradient", "radial", "linear-30"); parsed via
    /// [`BackgroundMode::parse`]. `None` => solid.
    pub background: Option<String>,
    /// Background image source: a file path, an `https://` URL, or a bundled name
    /// (see `--list-backgrounds`). Implies `background = "image"` when set.
    pub background_image: Option<String>,
    /// How the background image is scaled (fill | fit | stretch | center).
    pub background_fit: Fit,
    /// Don't darken the background image (default dims it for legibility).
    pub no_darken: bool,
    /// Keep the distro brand color for the logo + title even with another theme.
    pub brand: bool,
    /// Use a light variant of the theme (light background, dark text).
    pub light: bool,
    /// Field selection / order; `None` uses the default order.
    pub fields: Option<Vec<String>>,
    /// Heading font name (sans/serif slot); `None` uses the system default sans.
    pub sans: Option<String>,
    pub serif: Option<String>,
    /// Body font name (mono slot); `None` uses the system default monospace.
    pub mono: Option<String>,
}

impl Default for Config {
    fn default() -> Self {
        Self {
            layout: Layout::default(),
            // "auto" picks a distro-branded theme, falling back to the generic default.
            theme: "auto".to_string(),
            accent: None,
            logo: LogoKind::default(),
            logo_image: None,
            background: None,
            background_image: None,
            background_fit: Fit::default(),
            no_darken: false,
            brand: false,
            light: false,
            fields: None,
            sans: None,
            serif: None,
            mono: None,
        }
    }
}

impl Config {
    /// Load config from disk, falling back to defaults. A malformed file is
    /// reported on stderr and defaults are used (never a hard failure).
    pub fn load() -> Config {
        let Some(path) = config_path() else {
            return Config::default();
        };
        match std::fs::read_to_string(&path) {
            Ok(text) => toml::from_str(&text).unwrap_or_else(|err| {
                eprintln!(
                    "gnfetch: {} is invalid ({err}); using defaults.",
                    path.display()
                );
                Config::default()
            }),
            // A missing file is normal (silent). Any other read error
            // (permissions, EISDIR, I/O) is surfaced so a user's settings don't
            // silently vanish.
            Err(err) => {
                if err.kind() != std::io::ErrorKind::NotFound {
                    eprintln!(
                        "gnfetch: cannot read {} ({err}); using defaults.",
                        path.display()
                    );
                }
                Config::default()
            }
        }
    }
}

/// `$XDG_CONFIG_HOME/gnfetch/config.toml`, else `$HOME/.config/gnfetch/config.toml`.
fn config_path() -> Option<PathBuf> {
    let base = std::env::var_os("XDG_CONFIG_HOME")
        .map(PathBuf::from)
        .filter(|p| !p.as_os_str().is_empty())
        .or_else(|| std::env::var_os("HOME").map(|h| PathBuf::from(h).join(".config")))?;
    Some(base.join("gnfetch").join("config.toml"))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn background_named_modes() {
        assert_eq!(BackgroundMode::parse("solid"), Some(BackgroundMode::Solid));
        assert_eq!(
            BackgroundMode::parse("radial"),
            Some(BackgroundMode::Radial)
        );
        assert_eq!(BackgroundMode::parse("GRID"), Some(BackgroundMode::Grid));
        assert_eq!(
            BackgroundMode::parse("  transparent "),
            Some(BackgroundMode::Transparent)
        );
    }

    #[test]
    fn background_gradient_aliases() {
        assert_eq!(
            BackgroundMode::parse("gradient"),
            Some(BackgroundMode::Linear(90.0))
        );
        assert_eq!(
            BackgroundMode::parse("diagonal"),
            Some(BackgroundMode::Linear(45.0))
        );
        assert_eq!(
            BackgroundMode::parse("horizontal"),
            Some(BackgroundMode::Linear(0.0))
        );
    }

    #[test]
    fn background_linear_angle() {
        assert_eq!(
            BackgroundMode::parse("linear-30"),
            Some(BackgroundMode::Linear(30.0))
        );
        assert_eq!(
            BackgroundMode::parse("linear-135.5"),
            Some(BackgroundMode::Linear(135.5))
        );
        // Tolerant of the separator the user types.
        assert_eq!(
            BackgroundMode::parse("linear:60"),
            Some(BackgroundMode::Linear(60.0))
        );
    }

    #[test]
    fn background_unknown_is_none() {
        assert_eq!(BackgroundMode::parse("bogus"), None);
        assert_eq!(BackgroundMode::parse("linear-"), None);
        assert_eq!(BackgroundMode::parse("linear-abc"), None);
    }

    #[test]
    fn background_rejects_nonfinite_angle() {
        // f32::parse accepts "nan"/"inf"; a NaN angle would render an all-black
        // card silently, so parse() must reject non-finite angles.
        assert_eq!(BackgroundMode::parse("linear-nan"), None);
        assert_eq!(BackgroundMode::parse("linear-inf"), None);
        assert_eq!(BackgroundMode::parse("linear--inf"), None);
    }

    #[test]
    fn fit_toml_roundtrip_including_aliases() {
        fn fit_of(s: &str) -> Fit {
            toml::from_str::<Config>(&format!("background-fit = \"{s}\""))
                .unwrap()
                .background_fit
        }
        assert_eq!(fit_of("fill"), Fit::Fill);
        assert_eq!(fit_of("fit"), Fit::Contain);
        assert_eq!(fit_of("contain"), Fit::Contain); // alias
        assert_eq!(fit_of("stretch"), Fit::Stretch);
        assert_eq!(fit_of("center"), Fit::Center);
    }
}
