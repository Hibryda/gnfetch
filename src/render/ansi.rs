//! Classic ASCII/ANSI renderer: a colored ASCII logo on the left, an aligned
//! key/value info block on the right — the familiar neofetch layout. Colors come
//! from the active [`Theme`] (24-bit truecolor).

use super::Renderer;
use super::content;
use super::theme::Theme;
use crate::model::SystemInfo;
use image::Rgba;
use std::io::{self, Write};

const RESET: &str = "\x1b[0m";
const BOLD: &str = "\x1b[1m";
/// Gap printed between the logo column and the info column.
const COLUMN_GAP: &str = "   ";

pub struct AnsiRenderer {
    theme: Theme,
    /// Color for the logo + title (distro brand with `--brand`, else theme accent).
    brand: Rgba<u8>,
    /// Field selection / order; `None` uses the default order.
    fields: Option<Vec<String>>,
}

impl AnsiRenderer {
    pub fn new(theme: Theme, brand: Rgba<u8>, fields: Option<Vec<String>>) -> Self {
        Self {
            theme,
            brand,
            fields,
        }
    }
}

/// 24-bit foreground escape for an RGB color.
fn fg(c: Rgba<u8>) -> String {
    format!("\x1b[38;2;{};{};{}m", c[0], c[1], c[2])
}

impl Renderer for AnsiRenderer {
    fn render(&self, info: &SystemInfo) -> io::Result<()> {
        let accent = fg(self.theme.accent);
        let brand = fg(self.brand);
        let dim = fg(self.theme.dim);
        let text = fg(self.theme.text);

        let logo = logo_lines(info.distro_id.as_deref());
        let logo_w = logo.iter().map(|l| l.chars().count()).max().unwrap_or(0);
        let fields = build_fields(info, &brand, &dim, self.fields.as_deref());

        let stdout = io::stdout();
        let mut out = stdout.lock();

        writeln!(out)?;
        let rows = logo.len().max(fields.len());
        for i in 0..rows {
            let logo_line = logo.get(i).map(String::as_str).unwrap_or("");
            let pad = " ".repeat(logo_w - logo_line.chars().count());
            write!(out, "  {brand}{logo_line}{RESET}{pad}{COLUMN_GAP}")?;

            match fields.get(i) {
                // Empty key => a pre-formatted line (title / separator).
                Some((key, value)) if key.is_empty() => writeln!(out, "{value}")?,
                Some((key, value)) => writeln!(
                    out,
                    "{BOLD}{accent}{key}{RESET}{dim}:{RESET} {text}{value}{RESET}"
                )?,
                None => writeln!(out)?,
            }
        }
        writeln!(out)?;
        out.flush()
    }
}

/// Right-hand info block as `(key, value)` rows. An empty key marks a line that
/// is already fully formatted (the title and its separator).
fn build_fields(
    info: &SystemInfo,
    brand: &str,
    dim: &str,
    order: Option<&[String]>,
) -> Vec<(String, String)> {
    use content::Item;
    let c = content::extract(info, order);
    let mut f: Vec<(String, String)> = Vec::new();

    f.push((String::new(), format!("{BOLD}{brand}{}{RESET}", c.title)));
    if let Some(sub) = &c.subtitle {
        f.push((String::new(), format!("{dim}{sub}{RESET}")));
    }
    let rule = "-".repeat(c.title.chars().count());
    f.push((String::new(), format!("{dim}{rule}{RESET}")));
    for item in &c.items {
        match item {
            Item::Kv { key, value } => f.push((key.clone(), value.clone())),
            Item::Bar(b) => f.push((b.key.clone(), b.detail(true))),
            // Empty key + empty value => a blank spacer line.
            Item::Gap => f.push((String::new(), String::new())),
        }
    }
    f
}

/// ASCII logo lines for the distro (neofetch set), with a generic fallback.
fn logo_lines(distro_id: Option<&str>) -> Vec<String> {
    let art = distro_id
        .and_then(super::ascii_logos::ascii_for)
        .unwrap_or(GENERIC);
    art.lines().map(str::to_string).collect()
}

const GENERIC: &str = r#"    .--.
   |o_o |
   |:_/ |
  //   \ \
 (|     | )
/'\_   _/`\
\___)=(___/"#;
