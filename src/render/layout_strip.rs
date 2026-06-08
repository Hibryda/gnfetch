//! Horizontal strip: a short, wide band — logo, title, and an inline stats line.
//! Good for status bars / tmux panes.

use super::background;
use super::content::{self, CardContent};
use super::draw;
use super::fonts::Fonts;
use super::format::percent;
use super::layout::RenderParams;
use super::logo;
use crate::model::SystemInfo;
use image::{RgbaImage, imageops};

const PAD: f32 = 32.0;
const GAP: f32 = 30.0;
const H_TITLE: f32 = 42.0;
const H_STATS: f32 = 38.0;
const TITLE_SZ: f32 = 28.0;
const STATS_SZ: f32 = 18.0;
const MIN_TEXT_W: f32 = 360.0;

fn inner_h() -> f32 {
    H_TITLE + H_STATS
}

fn stats_line(c: &CardContent) -> String {
    let mut parts: Vec<String> = Vec::new();
    if let Some(sub) = &c.subtitle {
        parts.push(sub.clone());
    }
    if let Some(up) = c.kv("Uptime") {
        parts.push(format!("up {up}"));
    }
    if let Some(mem) = c.bar("Memory") {
        parts.push(format!("ram {}%", percent(mem.used, mem.total)));
    }
    if let Some(cpu) = c.kv("CPU") {
        parts.push(draw::truncate(cpu, 36));
    }
    if let Some(pkgs) = c.kv("Packages") {
        parts.push(pkgs.to_string());
    }
    parts.join("   ·   ")
}

pub fn base_size(info: &SystemInfo, has_logo: bool, fonts: &Fonts) -> (u32, u32) {
    let c = content::extract(info, None);
    let text_w = fonts
        .heading
        .text_w(TITLE_SZ, true, &c.title)
        .max(fonts.body.text_w(STATS_SZ, false, &stats_line(&c)))
        .max(MIN_TEXT_W);
    let logo_block = if has_logo { inner_h() + GAP } else { 0.0 };
    let width = PAD + logo_block + text_w + PAD;
    let height = PAD + inner_h() + PAD;
    (width.round() as u32, height.round() as u32)
}

pub fn render(p: &RenderParams) -> RgbaImage {
    let s = p.scale.max(0.1);
    let i = |v: f32| (v * s).round() as i32;
    let u = |v: f32| (v * s).round().max(1.0) as u32;
    let t = p.theme;

    let c = content::extract(p.info, None);
    let (bw, bh) = base_size(p.info, p.has_logo(), p.fonts);
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

    let mut text_x = i(PAD);
    if p.has_logo() {
        let side = u(inner_h());
        let distro = p.info.distro_id.as_deref().unwrap_or("");
        if let Some(logo_img) = logo::render(p.logo, p.logo_image, Some(distro), side, p.brand) {
            let ly = (height as i32 - logo_img.height() as i32) / 2;
            imageops::overlay(&mut img, &logo_img, i(PAD) as i64, ly as i64);
        }
        text_x = i(PAD) + side as i32 + i(GAP);
    }

    let mut y = i(PAD);
    p.fonts.heading.text(
        &mut img,
        text_x,
        y + i(4.0),
        TITLE_SZ * s,
        p.brand,
        true,
        &c.title,
    );
    y += i(H_TITLE);
    p.fonts.body.text(
        &mut img,
        text_x,
        y + i(4.0),
        STATS_SZ * s,
        t.dim,
        false,
        &stats_line(&c),
    );

    img
}
