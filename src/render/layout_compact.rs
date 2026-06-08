//! Compact layout: just the essentials (host, OS, CPU, memory, uptime) — no bars.

use super::background;
use super::content;
use super::draw;
use super::layout::RenderParams;
use super::logo;
use crate::model::SystemInfo;
use image::{RgbaImage, imageops};

/// Gap between the header and a logo badge.
const LOGO_GAP: f32 = 14.0;

const BASE_WIDTH: f32 = 560.0;
const PAD: f32 = 28.0;
const CONTENT_X: f32 = 34.0;
const STRIPE_W: f32 = 7.0;
const VALUE_COL: f32 = 130.0;
const H_TITLE: f32 = 40.0;
const H_SUB: f32 = 26.0;
const H_DIV: f32 = 18.0;
const H_ROW: f32 = 28.0;
const TITLE_SZ: f32 = 27.0;
const SUB_SZ: f32 = 16.0;
const ROW_SZ: f32 = 17.0;

/// Essential lines, in order: Uptime, CPU, Memory (as text). Compact is a curated
/// view and ignores the `--fields` order.
fn lines(info: &SystemInfo) -> Vec<(String, String)> {
    let c = content::extract(info, None);
    let mut out = Vec::new();
    if let Some(v) = c.kv("Uptime") {
        out.push(("Uptime".to_string(), v.to_string()));
    }
    if let Some(v) = c.kv("CPU") {
        out.push(("CPU".to_string(), v.to_string()));
    }
    if let Some(mem) = c.bar("Memory") {
        out.push(("Memory".to_string(), mem.detail(false)));
    }
    out
}

pub fn base_size(info: &SystemInfo) -> (u32, u32) {
    let n = lines(info).len();
    let has_sub = info.distro.is_some();
    let h = PAD + H_TITLE + if has_sub { H_SUB } else { 0.0 } + H_DIV + n as f32 * H_ROW + PAD;
    (BASE_WIDTH.round() as u32, h.round() as u32)
}

pub fn render(p: &RenderParams) -> RgbaImage {
    let s = p.scale.max(0.1);
    let i = |v: f32| (v * s).round() as i32;
    let u = |v: f32| (v * s).round().max(1.0) as u32;
    let t = p.theme;

    let (bw, bh) = base_size(p.info);
    let (width, height) = (u(bw as f32), u(bh as f32));
    let mut img = RgbaImage::new(width, height);
    background::fill(
        &mut img,
        p.background,
        p.background_image,
        p.background_fit,
        p.darken_image,
        t,
    );
    draw::rect(&mut img, 0, 0, u(STRIPE_W), height, p.brand);

    let logo_left = if p.has_logo() {
        let side = u(H_TITLE + H_SUB + H_DIV);
        let distro = p.info.distro_id.as_deref().unwrap_or("");
        logo::render(p.logo, p.logo_image, Some(distro), side, p.brand).map(|lg| {
            let lx = width as i32 - i(PAD) - lg.width() as i32;
            imageops::overlay(&mut img, &lg, lx as i64, i(PAD) as i64);
            lx
        })
    } else {
        None
    };

    let head = &p.fonts.heading;
    let body = &p.fonts.body;
    let c = content::extract(p.info, None);
    let cx = i(CONTENT_X);
    let value_x = cx + i(VALUE_COL);
    let row_sz = ROW_SZ * s;
    let char_w = body.char_w(row_sz, false);
    let max_chars = (((width as i32 - value_x - i(PAD)) as f32) / char_w).floor() as usize;

    let mut y = i(PAD);
    head.text(
        &mut img,
        cx,
        y + i(4.0),
        TITLE_SZ * s,
        p.brand,
        true,
        &c.title,
    );
    y += i(H_TITLE);
    if let Some(sub) = &c.subtitle {
        head.text(&mut img, cx, y + i(2.0), SUB_SZ * s, t.dim, false, sub);
        y += i(H_SUB);
    }
    let rule_right = match logo_left {
        Some(lx) => (lx - i(LOGO_GAP)).max(cx + 1),
        None => width as i32 - i(PAD),
    };
    let rule_w = (rule_right - cx).max(1) as u32;
    draw::rect(&mut img, cx, y + i(H_DIV / 2.0), rule_w, u(2.0), t.rule);
    y += i(H_DIV);

    for (key, value) in lines(p.info) {
        head.text(&mut img, cx, y + i(6.0), row_sz, t.accent, true, &key);
        let shown = draw::truncate(&value, max_chars);
        body.text(&mut img, value_x, y + i(6.0), row_sz, t.text, false, &shown);
        y += i(H_ROW);
    }

    img
}
