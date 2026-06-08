//! Color themes for the graphical card.
//!
//! A [`Theme`] is a flat palette consumed by every layout. Built-in presets are
//! listed in [`Theme::ALL`]; [`Theme::by_name`] resolves a name (case-insensitive)
//! and [`Theme::with_accent`] applies a user accent override.

use image::Rgba;

/// A resolved color palette.
#[derive(Clone, Copy, Debug)]
pub struct Theme {
    pub name: &'static str,
    /// Primary background.
    pub bg: Rgba<u8>,
    /// Accent — titles, keys, logo, normal bar fill.
    pub accent: Rgba<u8>,
    /// Body text.
    pub text: Rgba<u8>,
    /// De-emphasized text (subtitle, separators-as-text).
    pub dim: Rgba<u8>,
    /// Divider rule.
    pub rule: Rgba<u8>,
    /// Empty portion of a usage bar.
    pub bar_bg: Rgba<u8>,
    /// Bar fill once usage is high (>=75%).
    pub warn: Rgba<u8>,
    /// Bar fill once usage is critical (>=90%).
    pub crit: Rgba<u8>,
}

const fn rgb(r: u8, g: u8, b: u8) -> Rgba<u8> {
    Rgba([r, g, b, 255])
}

// Shared neutral palettes. Distro themes pair a brand accent with the dark set;
// `--light` swaps any theme onto the light set.
const DARK_BG: Rgba<u8> = rgb(20, 21, 24);
const DARK_TEXT: Rgba<u8> = rgb(228, 230, 234);
const DARK_DIM: Rgba<u8> = rgb(140, 147, 158);
const DARK_RULE: Rgba<u8> = rgb(48, 52, 60);
const DARK_BARBG: Rgba<u8> = rgb(40, 44, 52);

const LIGHT_BG: Rgba<u8> = rgb(247, 248, 250);
const LIGHT_TEXT: Rgba<u8> = rgb(34, 36, 42);
const LIGHT_DIM: Rgba<u8> = rgb(108, 116, 128);
const LIGHT_RULE: Rgba<u8> = rgb(206, 211, 219);
const LIGHT_BARBG: Rgba<u8> = rgb(220, 224, 231);

const WARN: Rgba<u8> = rgb(224, 176, 92);
const CRIT: Rgba<u8> = rgb(224, 108, 108);

/// A distro-branded theme: the brand `accent` over clean near-black neutrals.
const fn branded(name: &'static str, accent: (u8, u8, u8)) -> Theme {
    let (ar, ag, ab) = accent;
    Theme {
        name,
        bg: DARK_BG,
        accent: rgb(ar, ag, ab),
        text: DARK_TEXT,
        dim: DARK_DIM,
        rule: DARK_RULE,
        bar_bg: DARK_BARBG,
        warn: WARN,
        crit: CRIT,
    }
}

/// Distro-branded palettes, keyed by os-release id. Used by `--theme auto` and
/// addressable by name (e.g. `--theme debian`).
#[rustfmt::skip]
const DISTRO: &[(&str, Theme)] = &[
    ("debian",      branded("debian",      (215, 10, 83))),
    ("ubuntu",      branded("ubuntu",      (233, 84, 32))),
    ("fedora",      branded("fedora",      (60, 110, 180))),
    ("arch",        branded("arch",        (23, 147, 209))),
    ("linuxmint",   branded("mint",        (135, 207, 62))),
    ("manjaro",     branded("manjaro",     (53, 191, 92))),
    ("opensuse",    branded("opensuse",    (115, 186, 37))),
    ("gentoo",      branded("gentoo",      (154, 134, 196))),
    ("kali",        branded("kali",        (54, 123, 240))),
    ("popos",       branded("pop",         (72, 185, 199))),
    ("nixos",       branded("nixos",       (82, 119, 195))),
    ("redhat",      branded("redhat",      (238, 0, 0))),
    ("centos",      branded("centos",      (147, 34, 121))),
    ("alpine",      branded("alpine",      (13, 120, 180))),
    ("void",        branded("void",        (71, 128, 97))),
    ("elementary",  branded("elementary",  (100, 186, 255))),
    ("zorin",       branded("zorin",       (21, 166, 240))),
    ("endeavouros", branded("endeavouros", (138, 80, 200))),
    ("raspbian",    branded("raspberrypi", (197, 26, 74))),
    ("freebsd",     branded("freebsd",     (171, 43, 40))),
    ("artix",       branded("artix",       (16, 160, 204))),
];

impl Theme {
    /// Every built-in theme, in listing order. The first is the default.
    pub const ALL: &'static [Theme] = &[
        DEFAULT,
        NORD,
        DRACULA,
        GRUVBOX,
        CATPPUCCIN,
        TOKYO_NIGHT,
        SOLARIZED,
        ROSE_PINE,
    ];

    /// The default theme (teal on deep slate).
    pub fn default_theme() -> Theme {
        DEFAULT
    }

    /// Resolve a theme by name (case-insensitive): an aesthetic preset, or a
    /// distro brand name / key. Returns `None` if unknown.
    pub fn by_name(name: &str) -> Option<Theme> {
        Theme::ALL
            .iter()
            .find(|t| t.name.eq_ignore_ascii_case(name))
            .copied()
            .or_else(|| {
                DISTRO
                    .iter()
                    .find(|(key, t)| {
                        key.eq_ignore_ascii_case(name) || t.name.eq_ignore_ascii_case(name)
                    })
                    .map(|(_, t)| *t)
            })
    }

    /// A distro-branded theme for `id` (os-release id), if one exists.
    pub fn for_distro(id: Option<&str>) -> Option<Theme> {
        let id = id?;
        let mut key: String = id
            .chars()
            .filter(|c| c.is_ascii_alphanumeric())
            .flat_map(|c| c.to_lowercase())
            .collect();
        key = match key.as_str() {
            "archlinux" => "arch".into(),
            "pop" => "popos".into(),
            "rhel" => "redhat".into(),
            "kalilinux" => "kali".into(),
            "alpinelinux" => "alpine".into(),
            "voidlinux" => "void".into(),
            "raspberrypi" => "raspbian".into(),
            "elementaryos" => "elementary".into(),
            "manjarolinux" => "manjaro".into(),
            "opensuseleap" | "opensusetumbleweed" | "suse" => "opensuse".into(),
            _ => key,
        };
        DISTRO.iter().find(|(k, _)| *k == key).map(|(_, t)| *t)
    }

    /// Names of the distro-branded themes, for `--list-themes`.
    pub fn distro_names() -> Vec<&'static str> {
        DISTRO.iter().map(|(_, t)| t.name).collect()
    }

    /// Return a copy with the accent overridden.
    pub fn with_accent(mut self, accent: Rgba<u8>) -> Theme {
        self.accent = accent;
        self
    }

    /// Return a light variant: the same accent/warn/crit over light neutrals.
    pub fn light(self) -> Theme {
        Theme {
            bg: LIGHT_BG,
            text: LIGHT_TEXT,
            dim: LIGHT_DIM,
            rule: LIGHT_RULE,
            bar_bg: LIGHT_BARBG,
            ..self
        }
    }

    /// Names of all built-in themes, for `--list-themes`.
    pub fn names() -> Vec<&'static str> {
        Theme::ALL.iter().map(|t| t.name).collect()
    }
}

/// Parse a `#rrggbb` / `rrggbb` hex color. Returns `None` on malformed input.
pub fn parse_hex(s: &str) -> Option<Rgba<u8>> {
    let h = s.strip_prefix('#').unwrap_or(s);
    if h.len() != 6 || !h.bytes().all(|b| b.is_ascii_hexdigit()) {
        return None;
    }
    let r = u8::from_str_radix(&h[0..2], 16).ok()?;
    let g = u8::from_str_radix(&h[2..4], 16).ok()?;
    let b = u8::from_str_radix(&h[4..6], 16).ok()?;
    Some(Rgba([r, g, b, 255]))
}

const DEFAULT: Theme = Theme {
    name: "default",
    bg: rgb(22, 33, 38),
    accent: rgb(56, 193, 178),
    text: rgb(223, 230, 233),
    dim: rgb(138, 154, 160),
    rule: rgb(54, 74, 82),
    bar_bg: rgb(40, 56, 63),
    warn: rgb(224, 176, 92),
    crit: rgb(224, 108, 108),
};

const NORD: Theme = Theme {
    name: "nord",
    bg: rgb(46, 52, 64),
    accent: rgb(136, 192, 208),
    text: rgb(229, 233, 240),
    dim: rgb(143, 152, 169),
    rule: rgb(67, 76, 94),
    bar_bg: rgb(59, 66, 82),
    warn: rgb(235, 203, 139),
    crit: rgb(191, 97, 106),
};

const DRACULA: Theme = Theme {
    name: "dracula",
    bg: rgb(40, 42, 54),
    accent: rgb(189, 147, 249),
    text: rgb(248, 248, 242),
    dim: rgb(149, 152, 178),
    rule: rgb(68, 71, 90),
    bar_bg: rgb(56, 58, 75),
    warn: rgb(241, 250, 140),
    crit: rgb(255, 85, 85),
};

const GRUVBOX: Theme = Theme {
    name: "gruvbox",
    bg: rgb(40, 40, 40),
    accent: rgb(184, 187, 38),
    text: rgb(235, 219, 178),
    dim: rgb(168, 153, 132),
    rule: rgb(80, 73, 69),
    bar_bg: rgb(60, 56, 54),
    warn: rgb(250, 189, 47),
    crit: rgb(251, 73, 52),
};

const CATPPUCCIN: Theme = Theme {
    name: "catppuccin",
    bg: rgb(30, 30, 46),
    accent: rgb(137, 180, 250),
    text: rgb(205, 214, 244),
    dim: rgb(147, 153, 178),
    rule: rgb(69, 71, 90),
    bar_bg: rgb(49, 50, 68),
    warn: rgb(249, 226, 175),
    crit: rgb(243, 139, 168),
};

const TOKYO_NIGHT: Theme = Theme {
    name: "tokyonight",
    bg: rgb(26, 27, 38),
    accent: rgb(122, 162, 247),
    text: rgb(192, 202, 245),
    dim: rgb(125, 133, 172),
    rule: rgb(54, 58, 79),
    bar_bg: rgb(41, 46, 66),
    warn: rgb(224, 175, 104),
    crit: rgb(247, 118, 142),
};

const SOLARIZED: Theme = Theme {
    name: "solarized",
    bg: rgb(0, 43, 54),
    accent: rgb(42, 161, 152),
    text: rgb(238, 232, 213),
    dim: rgb(131, 148, 150),
    rule: rgb(7, 54, 66),
    bar_bg: rgb(7, 54, 66),
    warn: rgb(181, 137, 0),
    crit: rgb(220, 50, 47),
};

const ROSE_PINE: Theme = Theme {
    name: "rose-pine",
    bg: rgb(25, 23, 36),
    accent: rgb(196, 167, 231),
    text: rgb(224, 222, 244),
    dim: rgb(144, 140, 170),
    rule: rgb(38, 35, 58),
    bar_bg: rgb(38, 35, 58),
    warn: rgb(246, 193, 119),
    crit: rgb(235, 111, 146),
};

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn hex_parses_with_and_without_hash() {
        assert_eq!(parse_hex("#38c1b2"), Some(Rgba([56, 193, 178, 255])));
        assert_eq!(parse_hex("ffffff"), Some(Rgba([255, 255, 255, 255])));
        assert_eq!(parse_hex("#fff"), None);
        assert_eq!(parse_hex("nothex"), None);
    }

    #[test]
    fn by_name_is_case_insensitive_and_total() {
        assert!(Theme::by_name("Nord").is_some());
        assert!(Theme::by_name("DRACULA").is_some());
        assert!(Theme::by_name("nope").is_none());
        // every advertised name resolves
        for n in Theme::names() {
            assert!(Theme::by_name(n).is_some(), "{n} did not resolve");
        }
    }

    #[test]
    fn accent_override_applies() {
        let t = DEFAULT.with_accent(Rgba([1, 2, 3, 255]));
        assert_eq!(t.accent, Rgba([1, 2, 3, 255]));
        assert_eq!(t.bg, DEFAULT.bg); // unrelated fields preserved
    }

    #[test]
    fn light_keeps_accent_swaps_neutrals() {
        let dark = Theme::by_name("debian").unwrap();
        let light = dark.light();
        assert_eq!(light.accent, dark.accent); // brand accent preserved
        assert_eq!(light.bg, LIGHT_BG); // light background
        assert!(light.bg[0] > dark.bg[0]); // genuinely lighter
        assert!(light.text[0] < light.bg[0]); // dark text on light bg
    }

    #[test]
    fn distro_themes_resolve_with_aliases() {
        assert!(Theme::for_distro(Some("debian")).is_some());
        assert!(Theme::for_distro(Some("arch")).is_some());
        assert!(Theme::for_distro(Some("pop")).is_some()); // alias -> popos
        assert!(Theme::for_distro(Some("opensuse-tumbleweed")).is_some());
        assert!(Theme::for_distro(Some("totallyunknown")).is_none());
        assert!(Theme::for_distro(None).is_none());
        // distro themes are also addressable by name
        assert!(Theme::by_name("debian").is_some());
        // every distro theme resolves by its display name
        for n in Theme::distro_names() {
            assert!(Theme::by_name(n).is_some(), "{n} did not resolve");
        }
    }
}
