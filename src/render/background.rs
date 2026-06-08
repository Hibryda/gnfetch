//! Card background fills: solid, gradients, radial glow, grid/dot patterns,
//! image, or transparent. Procedural fills derive from the theme colors. Image
//! backgrounds come from a file, an `https://` URL, or a bundled CC0 image
//! (see [`super::bg_images`]), scaled per [`Fit`].

use super::bg_images;
use super::theme::Theme;
use crate::config::{BackgroundMode, Fit};
use image::{Rgba, RgbaImage};
use std::path::{Path, PathBuf};
use std::time::Duration;

/// Max bytes accepted for a downloaded/loaded background image (32 MiB).
const MAX_IMAGE_BYTES: u64 = 32 * 1024 * 1024;

/// A background-image source, classified once (at settings resolution) so the
/// renderer never re-sniffs the spec string.
#[derive(Clone, Debug)]
pub enum ImageSource {
    /// An `http(s)://` URL, downloaded on render.
    Url(String),
    /// A bundled CC0 image name (see [`bg_images`]).
    Bundled(String),
    /// A local file path.
    File(PathBuf),
}

impl ImageSource {
    /// Classify a spec: a URL (`http(s)://`) beats a bundled name beats a file path.
    pub fn classify(spec: &str) -> ImageSource {
        let s = spec.trim();
        if s.starts_with("https://") || s.starts_with("http://") {
            ImageSource::Url(s.to_string())
        } else if bg_images::bundled(s).is_some() {
            ImageSource::Bundled(s.to_string())
        } else {
            ImageSource::File(PathBuf::from(s))
        }
    }

    /// Load and decode this source to pixels.
    fn load(&self) -> Result<RgbaImage, String> {
        match self {
            ImageSource::Url(url) => fetch_url(url),
            ImageSource::Bundled(name) => match bg_images::bundled(name) {
                Some(bytes) => {
                    decode_bytes(bytes).map_err(|e| format!("bundled image '{name}': {e}"))
                }
                None => Err(format!("bundled image '{name}' not found")),
            },
            ImageSource::File(path) => decode_file(path)
                .map_err(|e| format!("could not load file '{}' ({e})", path.display())),
        }
    }
}

/// Fill `canvas` with the requested background. Image backgrounds that fail to
/// load fall back to a solid fill so rendering never aborts.
pub fn fill(
    canvas: &mut RgbaImage,
    mode: BackgroundMode,
    image: Option<&ImageSource>,
    fit: Fit,
    darken: bool,
    theme: &Theme,
) {
    match mode {
        BackgroundMode::Solid => solid(canvas, theme.bg),
        BackgroundMode::Transparent => solid(canvas, Rgba([0, 0, 0, 0])),
        BackgroundMode::Linear(angle) => {
            let (a, b) = ends(theme);
            linear(canvas, a, b, angle);
        }
        BackgroundMode::Radial => {
            let (a, b) = ends(theme);
            radial(canvas, a, b);
        }
        BackgroundMode::Grid => pattern(canvas, theme, false),
        BackgroundMode::Dots => pattern(canvas, theme, true),
        BackgroundMode::Image => match image.map(ImageSource::load) {
            Some(Ok(src)) => place_image(canvas, &src, fit, darken, theme),
            Some(Err(err)) => {
                eprintln!("gnfetch: background image: {err}; using solid.");
                solid(canvas, theme.bg);
            }
            None => {
                eprintln!("gnfetch: --background image needs --background-image; using solid.");
                solid(canvas, theme.bg);
            }
        },
    }
}

/// Download an image over HTTPS (timeout + size cap; `http://` and http-downgrade
/// redirects are refused; redirects are bounded).
fn fetch_url(url: &str) -> Result<RgbaImage, String> {
    if !url.starts_with("https://") {
        return Err(format!("refusing non-https URL '{url}'"));
    }
    let agent = ureq::Agent::config_builder()
        .timeout_global(Some(Duration::from_secs(20)))
        // https_only rejects a redirect that downgrades to http or points at an
        // http host — the initial-URL check above only covers the first hop.
        .https_only(true)
        .max_redirects(4)
        .user_agent(concat!("gnfetch/", env!("CARGO_PKG_VERSION")))
        .build()
        .new_agent();
    let mut resp = agent
        .get(url)
        .call()
        .map_err(|e| format!("fetch '{url}' failed ({e})"))?;
    let bytes = resp
        .body_mut()
        .with_config()
        .limit(MAX_IMAGE_BYTES)
        .read_to_vec()
        .map_err(|e| format!("read '{url}' failed ({e})"))?;
    decode_bytes(&bytes).map_err(|e| format!("decode '{url}' failed ({e})"))
}

/// Decode limits that bound a pixel/decompression bomb: a small file can declare
/// gigapixel dimensions, and the byte cap only bounds the *compressed* payload.
/// (image's default already caps `max_alloc` at 512 MiB; we tighten further.)
fn decode_limits() -> image::Limits {
    let mut limits = image::Limits::default();
    limits.max_image_width = Some(16_384);
    limits.max_image_height = Some(16_384);
    limits.max_alloc = Some(256 * 1024 * 1024);
    limits
}

/// Decode in-memory image bytes (URL / bundled) under [`decode_limits`].
fn decode_bytes(bytes: &[u8]) -> Result<RgbaImage, String> {
    let mut reader = image::ImageReader::new(std::io::Cursor::new(bytes))
        .with_guessed_format()
        .map_err(|e| e.to_string())?;
    reader.limits(decode_limits());
    reader
        .decode()
        .map(|i| i.to_rgba8())
        .map_err(|e| e.to_string())
}

/// Decode an image file under [`decode_limits`].
fn decode_file(path: &Path) -> Result<RgbaImage, String> {
    let mut reader = image::ImageReader::open(path).map_err(|e| e.to_string())?;
    reader.limits(decode_limits());
    reader
        .decode()
        .map(|i| i.to_rgba8())
        .map_err(|e| e.to_string())
}

fn solid(canvas: &mut RgbaImage, color: Rgba<u8>) {
    for px in canvas.pixels_mut() {
        *px = color;
    }
}

/// Gradient endpoints with a clearly visible spread, derived from the theme:
/// a lightened, slightly accent-tinted "bright" end and a darkened "dark" end.
/// (The theme's `bg`/`bg2` are nearly identical, so we don't use them directly.)
fn ends(theme: &Theme) -> (Rgba<u8>, Rgba<u8>) {
    let bright = tint(lighten(theme.bg, 0.16), theme.accent, 0.10);
    let dark = darken(theme.bg, 0.40);
    (bright, dark)
}

/// Linear gradient from `from` to `to` at `angle` degrees (0 = →, 90 = ↓).
fn linear(canvas: &mut RgbaImage, from: Rgba<u8>, to: Rgba<u8>, angle: f32) {
    let a = angle.to_radians();
    let (dx, dy) = (a.cos(), a.sin());
    let (w, h) = (canvas.width(), canvas.height());
    // Normalize over the projection of the four corners onto the gradient axis.
    let corners = [
        (0.0, 0.0),
        (w as f32, 0.0),
        (0.0, h as f32),
        (w as f32, h as f32),
    ];
    let proj = |x: f32, y: f32| x * dx + y * dy;
    let pmin = corners
        .iter()
        .map(|&(x, y)| proj(x, y))
        .fold(f32::INFINITY, f32::min);
    let pmax = corners
        .iter()
        .map(|&(x, y)| proj(x, y))
        .fold(f32::NEG_INFINITY, f32::max);
    let range = (pmax - pmin).max(1.0);
    for y in 0..h {
        for x in 0..w {
            let t = ((proj(x as f32, y as f32) - pmin) / range).clamp(0.0, 1.0);
            canvas.put_pixel(x, y, mix_dithered(from, to, t, x, y));
        }
    }
}

/// Radial glow: `center` at the middle fading to `edge` at the corners.
fn radial(canvas: &mut RgbaImage, center: Rgba<u8>, edge: Rgba<u8>) {
    let (w, h) = (canvas.width(), canvas.height());
    let (cx, cy) = (w as f32 / 2.0, h as f32 / 2.0);
    let max_d = (cx * cx + cy * cy).sqrt().max(1.0);
    for y in 0..h {
        for x in 0..w {
            let (dx, dy) = (x as f32 - cx, y as f32 - cy);
            let t = ((dx * dx + dy * dy).sqrt() / max_d).clamp(0.0, 1.0);
            canvas.put_pixel(x, y, mix_dithered(center, edge, t, x, y));
        }
    }
}

/// Solid background overlaid with a subtle grid or dot pattern in `rule` color.
fn pattern(canvas: &mut RgbaImage, theme: &Theme, dots: bool) {
    solid(canvas, theme.bg);
    let (w, h) = (canvas.width(), canvas.height());
    // Spacing/thickness scale with the canvas so they look consistent at any size.
    let spacing = (h / 22).max(18);
    let thick = (h / 360).max(1);
    let line = mix(theme.bg, theme.rule, 0.6);
    for y in 0..h {
        let on_h = y % spacing < thick;
        for x in 0..w {
            let on_v = x % spacing < thick;
            let draw = if dots { on_h && on_v } else { on_h || on_v };
            if draw {
                canvas.put_pixel(x, y, line);
            }
        }
    }
}

/// Place `src` onto the canvas using `fit`. When `darken`, the image is dimmed
/// so foreground text stays legible (off with `--no-darken`). Uncovered area
/// (letterbox/pad) is filled with `theme.bg`.
fn place_image(canvas: &mut RgbaImage, src: &RgbaImage, fit: Fit, darken: bool, theme: &Theme) {
    let (cw, ch) = (canvas.width(), canvas.height());
    let (sw, sh) = (src.width().max(1), src.height().max(1));
    solid(canvas, theme.bg);
    let shade = |p: &Rgba<u8>| if darken { darken_px(p) } else { *p };

    match fit {
        Fit::Stretch => {
            let r = image::imageops::resize(src, cw, ch, image::imageops::FilterType::Triangle);
            for y in 0..ch {
                for x in 0..cw {
                    canvas.put_pixel(x, y, shade(r.get_pixel(x, y)));
                }
            }
        }
        Fit::Fill => {
            // Cover: scale up to fill both axes, center-crop the overflow.
            let scale = (cw as f32 / sw as f32).max(ch as f32 / sh as f32);
            let rw = ((sw as f32 * scale).ceil() as u32).max(cw);
            let rh = ((sh as f32 * scale).ceil() as u32).max(ch);
            let r = image::imageops::resize(src, rw, rh, image::imageops::FilterType::Triangle);
            let (ox, oy) = ((rw - cw) / 2, (rh - ch) / 2);
            for y in 0..ch {
                for x in 0..cw {
                    canvas.put_pixel(x, y, shade(r.get_pixel(x + ox, y + oy)));
                }
            }
        }
        Fit::Contain => {
            // Contain: scale down to fit inside, centered, letterboxed.
            let scale = (cw as f32 / sw as f32).min(ch as f32 / sh as f32);
            let rw = ((sw as f32 * scale).round() as u32).clamp(1, cw);
            let rh = ((sh as f32 * scale).round() as u32).clamp(1, ch);
            let r = image::imageops::resize(src, rw, rh, image::imageops::FilterType::Triangle);
            let (ox, oy) = ((cw - rw) / 2, (ch - rh) / 2);
            for y in 0..rh {
                for x in 0..rw {
                    canvas.put_pixel(x + ox, y + oy, shade(r.get_pixel(x, y)));
                }
            }
        }
        Fit::Center => {
            // No scaling: center the image, cropping if larger, padding if smaller.
            let ox = (cw as i64 - sw as i64) / 2;
            let oy = (ch as i64 - sh as i64) / 2;
            for sy in 0..sh {
                let cy = sy as i64 + oy;
                if cy < 0 || cy >= ch as i64 {
                    continue;
                }
                for sx in 0..sw {
                    let cx = sx as i64 + ox;
                    if cx < 0 || cx >= cw as i64 {
                        continue;
                    }
                    canvas.put_pixel(cx as u32, cy as u32, shade(src.get_pixel(sx, sy)));
                }
            }
        }
    }
}

/// Darken a pixel to ~45% brightness so light foreground text stays legible.
fn darken_px(p: &Rgba<u8>) -> Rgba<u8> {
    Rgba([
        (p[0] as u32 * 45 / 100) as u8,
        (p[1] as u32 * 45 / 100) as u8,
        (p[2] as u32 * 45 / 100) as u8,
        255,
    ])
}

const WHITE: Rgba<u8> = Rgba([255, 255, 255, 255]);
const BLACK: Rgba<u8> = Rgba([0, 0, 0, 255]);

/// Blend `c` toward white by `amt`.
fn lighten(c: Rgba<u8>, amt: f32) -> Rgba<u8> {
    mix(c, WHITE, amt)
}

/// Blend `c` toward black by `amt`.
fn darken(c: Rgba<u8>, amt: f32) -> Rgba<u8> {
    mix(c, BLACK, amt)
}

/// Blend `c` toward `accent` by `amt`.
fn tint(c: Rgba<u8>, accent: Rgba<u8>, amt: f32) -> Rgba<u8> {
    mix(c, accent, amt)
}

/// Linearly interpolate two colors (RGB; result is opaque).
fn mix(a: Rgba<u8>, b: Rgba<u8>, t: f32) -> Rgba<u8> {
    Rgba([
        lerp(a[0], b[0], t),
        lerp(a[1], b[1], t),
        lerp(a[2], b[2], t),
        255,
    ])
}

fn lerp(a: u8, b: u8, t: f32) -> u8 {
    (a as f32 + (b as f32 - a as f32) * t)
        .round()
        .clamp(0.0, 255.0) as u8
}

/// Dither offset in `[-0.5, +0.5)` for pixel `(x, y)`.
///
/// Smooth gradients band because each 1/255 step covers many pixels at 8-bit
/// depth. Nudging each pixel by up to ±0.5 LSB before rounding scatters the band
/// edges into noise the eye averages out — and which vanishes entirely once the
/// supersampled card is downscaled to the terminal.
///
/// Uses hashed **white noise** rather than an ordered (Bayer) matrix or
/// interleaved gradient noise: those are deterministic but *periodic*, leaving a
/// faint diagonal cross-hatch in flat mid-tones. White noise has no structure,
/// so what's left is imperceptible random grain instead of a visible weave.
fn dither(x: u32, y: u32) -> f32 {
    // Integer hash -> uniform value in [0, 1).
    let mut h = x.wrapping_mul(73_856_093) ^ y.wrapping_mul(19_349_663);
    h = h.wrapping_mul(0x85eb_ca6b);
    h ^= h >> 13;
    h = h.wrapping_mul(0xc2b2_ae35);
    h ^= h >> 16;
    (h >> 8) as f32 / 16_777_216.0 - 0.5
}

/// Like [`mix`], but applies dithering to defeat gradient banding.
fn mix_dithered(a: Rgba<u8>, b: Rgba<u8>, t: f32, x: u32, y: u32) -> Rgba<u8> {
    let d = dither(x, y);
    Rgba([
        lerp_dithered(a[0], b[0], t, d),
        lerp_dithered(a[1], b[1], t, d),
        lerp_dithered(a[2], b[2], t, d),
        255,
    ])
}

fn lerp_dithered(a: u8, b: u8, t: f32, d: f32) -> u8 {
    (a as f32 + (b as f32 - a as f32) * t + d)
        .round()
        .clamp(0.0, 255.0) as u8
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn bundled_image_loads_case_insensitive() {
        let img = ImageSource::classify("carina")
            .load()
            .expect("bundled 'carina' should decode");
        assert!(img.width() > 0 && img.height() > 0);
        assert!(ImageSource::classify("CARINA").load().is_ok());
    }

    #[test]
    fn missing_file_source_errors() {
        assert!(
            ImageSource::classify("definitely-not-a-real-file-9z9z.png")
                .load()
                .is_err()
        );
    }

    #[test]
    fn non_https_url_is_rejected() {
        // Rejected before any network access, so this never touches the wire.
        assert!(fetch_url("http://example.com/x.png").is_err());
    }

    #[test]
    fn place_image_preserves_canvas_size() {
        let theme = Theme::default_theme();
        let src = RgbaImage::from_pixel(4, 3, Rgba([200, 100, 50, 255]));
        for fit in [Fit::Fill, Fit::Contain, Fit::Stretch, Fit::Center] {
            let mut canvas = RgbaImage::new(20, 16);
            place_image(&mut canvas, &src, fit, true, &theme);
            assert_eq!((canvas.width(), canvas.height()), (20, 16));
        }
    }

    #[test]
    fn center_fit_places_at_correct_offset_and_pads() {
        let theme = Theme::default_theme();
        let px = Rgba([200, 100, 50, 255]);
        let src = RgbaImage::from_pixel(4, 3, px);
        let mut canvas = RgbaImage::new(20, 16);
        place_image(&mut canvas, &src, Fit::Center, true, &theme);
        // src (4x3) centered in (20x16): offset (8, 6).
        assert_eq!(*canvas.get_pixel(8, 6), darken_px(&px));
        // Padding outside the image is the opaque theme background.
        assert_eq!(*canvas.get_pixel(0, 0), theme.bg);
        assert_eq!(canvas.get_pixel(0, 0)[3], 255);
    }

    #[test]
    fn center_fit_source_larger_than_canvas_crops_without_panic() {
        let theme = Theme::default_theme();
        let src = RgbaImage::from_pixel(30, 30, Rgba([10, 220, 30, 255]));
        let mut canvas = RgbaImage::new(20, 16);
        place_image(&mut canvas, &src, Fit::Center, true, &theme); // must not panic
        // Every pixel is painted (opaque); the negative-offset crop branch ran.
        assert!(canvas.pixels().all(|p| p[3] == 255));
        assert_eq!(
            *canvas.get_pixel(10, 8),
            darken_px(&Rgba([10, 220, 30, 255]))
        );
    }

    #[test]
    fn linear_gradient_runs_from_left_to_right_at_zero_degrees() {
        let from = Rgba([255, 0, 0, 255]);
        let to = Rgba([0, 0, 255, 255]);
        let mut canvas = RgbaImage::new(64, 8);
        linear(&mut canvas, from, to, 0.0);
        // angle 0 = left->right: left edge ~from (red), right edge ~to (blue).
        assert!(canvas.get_pixel(0, 4)[0] > 200 && canvas.get_pixel(0, 4)[2] < 40);
        assert!(canvas.get_pixel(63, 4)[2] > 200 && canvas.get_pixel(63, 4)[0] < 40);
    }

    #[test]
    fn radial_is_bright_at_center_dark_at_corner() {
        let center = Rgba([255, 255, 255, 255]);
        let edge = Rgba([0, 0, 0, 255]);
        let mut canvas = RgbaImage::new(33, 33);
        radial(&mut canvas, center, edge);
        assert!(canvas.get_pixel(16, 16)[0] > 230); // near-white center
        assert!(canvas.get_pixel(0, 0)[0] < 25); // near-black corner
    }

    #[test]
    fn no_darken_leaves_image_pixels_unmodified() {
        let theme = Theme::default_theme();
        let px = Rgba([200, 100, 50, 255]);
        let src = RgbaImage::from_pixel(4, 3, px);
        let mut canvas = RgbaImage::new(8, 6);
        // darken = false: a uniform image stays exactly its own color (no dimming).
        place_image(&mut canvas, &src, Fit::Stretch, false, &theme);
        assert_eq!(*canvas.get_pixel(4, 3), px);
        // For contrast, darken = true dims it.
        place_image(&mut canvas, &src, Fit::Stretch, true, &theme);
        assert_eq!(*canvas.get_pixel(4, 3), darken_px(&px));
    }

    #[test]
    fn image_source_classifies_url_bundled_file() {
        assert!(matches!(
            ImageSource::classify("https://example.com/x.png"),
            ImageSource::Url(_)
        ));
        assert!(matches!(
            ImageSource::classify("carina"),
            ImageSource::Bundled(_)
        ));
        assert!(matches!(
            ImageSource::classify("/some/wallpaper.png"),
            ImageSource::File(_)
        ));
    }
}
