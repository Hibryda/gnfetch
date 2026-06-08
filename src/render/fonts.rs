//! Font resolution for the graphical card.
//!
//! The card uses two slots: a **heading** font (title, subtitle, keys) and a
//! **body** font (values, bar details, footer). Each can be a bundled font, a
//! named system font, or the system default of a generic category (sans / serif
//! / mono). System discovery uses `fontdb` (fontconfig on Linux); everything
//! falls back to the embedded bundled fonts so rendering never fails.

use super::draw::Pen;
use ab_glyph::FontVec;
use fontdb::{Database, Family, Query, Weight};

static DEJAVU_R: &[u8] = include_bytes!("../../assets/fonts/DejaVuSansMono.ttf");
static DEJAVU_B: &[u8] = include_bytes!("../../assets/fonts/DejaVuSansMono-Bold.ttf");
static POPPINS_R: &[u8] = include_bytes!("../../assets/fonts/poppins-regular.ttf");
static POPPINS_B: &[u8] = include_bytes!("../../assets/fonts/poppins-bold.ttf");
static PACIFICO: &[u8] = include_bytes!("../../assets/fonts/pacifico.ttf");
static LOBSTER: &[u8] = include_bytes!("../../assets/fonts/lobster.ttf");
static RIGHTEOUS: &[u8] = include_bytes!("../../assets/fonts/righteous.ttf");
static BUNGEE: &[u8] = include_bytes!("../../assets/fonts/bungee.ttf");

/// A bundled font entry: `(name, regular bytes, optional bold bytes)`.
type BundledFont = (&'static str, &'static [u8], Option<&'static [u8]>);

/// Bundled fonts, addressable by name.
const BUNDLED: &[BundledFont] = &[
    ("dejavu-mono", DEJAVU_R, Some(DEJAVU_B)),
    ("poppins", POPPINS_R, Some(POPPINS_B)),
    ("pacifico", PACIFICO, None),
    ("lobster", LOBSTER, None),
    ("righteous", RIGHTEOUS, None),
    ("bungee", BUNGEE, None),
];

/// Names of the bundled fonts, for `--list-fonts`.
pub fn bundled_names() -> Vec<&'static str> {
    BUNDLED.iter().map(|(n, _, _)| *n).collect()
}

fn bundled(name: &str) -> Option<(&'static [u8], Option<&'static [u8]>)> {
    BUNDLED
        .iter()
        .find(|(n, _, _)| n.eq_ignore_ascii_case(name))
        .map(|(_, r, b)| (*r, *b))
}

/// Generic font category, used for system defaults and as a fallback.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Generic {
    Sans,
    Serif,
    Mono,
}

/// A font selection for one slot.
#[derive(Clone, Debug)]
pub struct FontChoice {
    /// `None`/empty => the system default of `generic`. Otherwise a bundled or
    /// system font family name.
    pub name: Option<String>,
    pub generic: Generic,
}

impl FontChoice {
    pub fn system_default(generic: Generic) -> Self {
        Self {
            name: None,
            generic,
        }
    }

    fn wants_named(&self) -> Option<&str> {
        self.name.as_deref().filter(|n| !n.trim().is_empty())
    }

    /// Whether resolving this choice needs the system font database.
    fn needs_system(&self) -> bool {
        match self.wants_named() {
            Some(name) => bundled(name).is_none(),
            None => true, // system default
        }
    }
}

/// Resolved fonts for both card slots.
pub struct Fonts {
    pub heading: Pen,
    pub body: Pen,
}

impl Fonts {
    /// Resolve both slots, loading system fonts only if needed.
    pub fn resolve(heading: &FontChoice, body: &FontChoice) -> Fonts {
        let db = (heading.needs_system() || body.needs_system()).then(|| {
            let mut db = Database::new();
            db.load_system_fonts();
            db
        });
        Fonts {
            heading: resolve_pen(heading, db.as_ref()),
            body: resolve_pen(body, db.as_ref()),
        }
    }
}

fn resolve_pen(choice: &FontChoice, db: Option<&Database>) -> Pen {
    if let Some(name) = choice.wants_named() {
        if let Some((r, b)) = bundled(name) {
            return pen_from_bytes(r, b);
        }
        if let Some(db) = db
            && let Some(pen) = system_pen(db, Family::Name(name))
        {
            return pen;
        }
        eprintln!(
            "gnfetch: font '{name}' not found; using {:?} default.",
            choice.generic
        );
    } else if let Some(db) = db
        && let Some(pen) = system_pen(db, generic_family(choice.generic))
    {
        return pen;
    }
    fallback_pen(choice.generic)
}

fn generic_family(g: Generic) -> Family<'static> {
    match g {
        Generic::Sans => Family::SansSerif,
        Generic::Serif => Family::Serif,
        Generic::Mono => Family::Monospace,
    }
}

/// Bundled fallback when system lookup fails.
fn fallback_pen(g: Generic) -> Pen {
    match g {
        Generic::Mono => pen_from_bytes(DEJAVU_R, Some(DEJAVU_B)),
        // Poppins is our bundled sans; use it for serif too rather than fail.
        Generic::Sans | Generic::Serif => pen_from_bytes(POPPINS_R, Some(POPPINS_B)),
    }
}

fn pen_from_bytes(regular: &[u8], bold: Option<&[u8]>) -> Pen {
    let r = FontVec::try_from_vec(regular.to_vec()).expect("valid embedded font");
    let b = bold
        .and_then(|bytes| FontVec::try_from_vec(bytes.to_vec()).ok())
        // No bold face: reuse regular (synthetic bolding isn't worth it here).
        .unwrap_or_else(|| FontVec::try_from_vec(regular.to_vec()).expect("valid embedded font"));
    Pen::from_faces(r, b)
}

/// Load a regular + bold face for `family` from the system database.
fn system_pen(db: &Database, family: Family) -> Option<Pen> {
    let regular = load_face(db, family, Weight::NORMAL)?;
    let bold = load_face(db, family, Weight::BOLD)
        .unwrap_or_else(|| load_face(db, family, Weight::NORMAL).expect("regular re-loads"));
    Some(Pen::from_faces(regular, bold))
}

fn load_face(db: &Database, family: Family, weight: Weight) -> Option<FontVec> {
    let id = db.query(&Query {
        families: &[family],
        weight,
        ..Query::default()
    })?;
    db.with_face_data(id, |data, index| {
        FontVec::try_from_vec_and_index(data.to_vec(), index).ok()
    })
    .flatten()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn bundled_lookup_is_case_insensitive() {
        assert!(bundled("Poppins").is_some());
        assert!(bundled("DEJAVU-MONO").is_some());
        assert!(bundled("nope").is_none());
        assert!(bundled_names().contains(&"lobster"));
    }

    #[test]
    fn needs_system_only_for_unbundled_or_default() {
        let bundled_choice = FontChoice {
            name: Some("lobster".into()),
            generic: Generic::Sans,
        };
        assert!(!bundled_choice.needs_system());
        let named = FontChoice {
            name: Some("Some System Font".into()),
            generic: Generic::Sans,
        };
        assert!(named.needs_system());
        assert!(FontChoice::system_default(Generic::Mono).needs_system());
    }

    #[test]
    fn bundled_only_resolves_without_system_scan() {
        // Both bundled => no Database is loaded; pens build from embedded bytes.
        let heading = FontChoice {
            name: Some("pacifico".into()),
            generic: Generic::Sans,
        };
        let body = FontChoice {
            name: Some("dejavu-mono".into()),
            generic: Generic::Mono,
        };
        let fonts = Fonts::resolve(&heading, &body);
        // A real width measurement proves the faces parsed.
        assert!(fonts.heading.text_w(20.0, false, "abc") > 0.0);
        assert!(fonts.body.text_w(20.0, false, "abc") > 0.0);
    }
}
