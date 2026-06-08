//! Distro logos from one of several sources (`--logo`):
//! - `drawn` — a bundled distro SVG (Simple Icons, CC0) recolored to the theme accent;
//!   falls back to a generic Tux SVG for unknown distros,
//! - `ascii` — neofetch-style ASCII art rendered as text,
//! - `image` — a user-supplied image file,
//! - `off` — nothing.
//!
//! Each returns an [`RgbaImage`] on a transparent background, sized to fit a
//! `size`x`size` box, for the layout to composite.

use super::draw::Pen;
use crate::config::LogoKind;
use image::{Rgba, RgbaImage, imageops};
use resvg::{tiny_skia, usvg};
use std::path::Path;

/// Generic Tux SVG used for unknown Linux distros.
static GENERIC_SVG: &[u8] = include_bytes!("../../assets/logos/_generic.svg");

/// Produce a logo image (recolored to `accent` where applicable), or `None` for
/// `off`/failed image loads.
pub fn render(
    kind: LogoKind,
    image_path: Option<&Path>,
    distro_id: Option<&str>,
    size: u32,
    accent: Rgba<u8>,
) -> Option<RgbaImage> {
    let size = size.max(16);
    match kind {
        LogoKind::Off => None,
        LogoKind::Image => match image_path.and_then(|p| load_image(p, size)) {
            Some(img) => Some(img),
            None => {
                eprintln!("gnfetch: could not load logo image; using distro emblem instead.");
                Some(drawn(distro_id, size, accent))
            }
        },
        LogoKind::Ascii => Some(ascii(distro_id, size, accent)),
        LogoKind::Drawn => Some(drawn(distro_id, size, accent)),
    }
}

fn load_image(path: &Path, size: u32) -> Option<RgbaImage> {
    let img = image::open(path).ok()?.to_rgba8();
    let (w, h) = (img.width().max(1), img.height().max(1));
    let scale = (size as f32 / w as f32).min(size as f32 / h as f32);
    let nw = (w as f32 * scale).round().max(1.0) as u32;
    let nh = (h as f32 * scale).round().max(1.0) as u32;
    Some(imageops::resize(
        &img,
        nw,
        nh,
        imageops::FilterType::Lanczos3,
    ))
}

/// Bundled distro SVG recolored to `accent` — the specific logo when we have
/// one, else the generic Tux.
fn drawn(distro_id: Option<&str>, size: u32, accent: Rgba<u8>) -> RgbaImage {
    let svg = distro_id
        .and_then(super::svg_logos::svg_for)
        .unwrap_or(GENERIC_SVG);
    render_svg(svg, size, accent).unwrap_or_else(|| RgbaImage::new(size, size))
}

/// Rasterize a (monochrome Simple Icons) SVG and tint it with `accent`, using
/// the rendered alpha as a mask. Fits within a `size`x`size` box.
fn render_svg(data: &[u8], size: u32, accent: Rgba<u8>) -> Option<RgbaImage> {
    let tree = usvg::Tree::from_data(data, &usvg::Options::default()).ok()?;
    let ts = tree.size();
    let scale = (size as f32 / ts.width()).min(size as f32 / ts.height());
    let w = (ts.width() * scale).ceil().max(1.0) as u32;
    let h = (ts.height() * scale).ceil().max(1.0) as u32;

    let mut pixmap = tiny_skia::Pixmap::new(w, h)?;
    resvg::render(
        &tree,
        tiny_skia::Transform::from_scale(scale, scale),
        &mut pixmap.as_mut(),
    );

    let mut img = RgbaImage::new(w, h);
    for (idx, px) in pixmap.pixels().iter().enumerate() {
        let a = px.alpha();
        if a > 0 {
            let (x, y) = (idx as u32 % w, idx as u32 / w);
            img.put_pixel(x, y, Rgba([accent[0], accent[1], accent[2], a]));
        }
    }
    Some(img)
}

/// Render the distro's ASCII art (from neofetch) fit within a `size`x`size` box.
fn ascii(distro_id: Option<&str>, size: u32, accent: Rgba<u8>) -> RgbaImage {
    let art = distro_id
        .and_then(super::ascii_logos::ascii_for)
        .unwrap_or(TUX_ART);
    let lines: Vec<&str> = art.lines().collect();
    let rows = lines.len().max(1) as f32;
    let cols = lines
        .iter()
        .map(|l| l.chars().count())
        .max()
        .unwrap_or(1)
        .max(1) as f32;

    let pen = Pen::new();
    // Width per font px (monospace), to fit both width and height in the box.
    let cw_per = pen.char_w(100.0, false) / 100.0;
    const LINE_FACTOR: f32 = 1.05;
    let font = (size as f32 / (cols * cw_per))
        .min(size as f32 / (rows * LINE_FACTOR))
        .max(1.0);
    let line_h = font * LINE_FACTOR;
    let width = (pen.char_w(font, false) * cols).ceil().max(1.0) as u32;
    let height = (line_h * rows).ceil().max(1.0) as u32;

    let mut img = RgbaImage::new(width, height);
    for (row, line) in lines.iter().enumerate() {
        pen.text(
            &mut img,
            0,
            (row as f32 * line_h) as i32,
            font,
            accent,
            false,
            line,
        );
    }
    img
}

const TUX_ART: &str = r#"    .--.
   |o_o |
   |:_/ |
  //   \ \
 (|     | )
/'\_   _/`\
\___)=(___/"#;

#[cfg(test)]
mod tests {
    use super::*;

    const ACCENT: Rgba<u8> = Rgba([56, 193, 178, 255]);

    #[test]
    fn off_yields_none() {
        assert!(render(LogoKind::Off, None, Some("debian"), 100, ACCENT).is_none());
    }

    #[test]
    fn drawn_renders_for_known_and_unknown() {
        // Known distro uses its SVG; unknown falls back to the generic Tux SVG.
        for id in [
            Some("arch"),
            Some("fedora"),
            Some("nixos"),
            Some("totallyunknown"),
            None,
        ] {
            let img = render(LogoKind::Drawn, None, id, 96, ACCENT).unwrap();
            assert!(img.width() > 0 && img.height() > 0);
        }
    }

    #[test]
    fn bundled_svgs_parse_and_render() {
        for id in [
            "arch", "debian", "fedora", "ubuntu", "nixos", "gentoo", "void", "manjaro",
        ] {
            assert!(
                super::super::svg_logos::svg_for(id).is_some(),
                "{id} missing svg"
            );
            let img = render(LogoKind::Drawn, None, Some(id), 96, ACCENT).unwrap();
            assert!(img.width() > 0 && img.height() > 0);
        }
    }
}
