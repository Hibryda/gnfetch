//! Neofetch-style layout: logo panel on the left, info column on the right.

use super::background;
use super::content::{self, CardContent, Item};
use super::draw;
use super::layout::RenderParams;
use super::logo;
use crate::model::SystemInfo;
use image::{RgbaImage, imageops};

const PAD: f32 = 36.0;
const GAP: f32 = 44.0;
const INFO_W: f32 = 720.0;
const VALUE_COL: f32 = 168.0;
const BAR_W: f32 = 200.0;
const BAR_H: f32 = 14.0;
const DETAIL_GAP: f32 = 14.0;

const H_TITLE: f32 = 42.0;
const H_SUB: f32 = 28.0;
const H_DIV: f32 = 22.0;
const H_ROW: f32 = 30.0;
const H_GAP: f32 = 8.0;

const TITLE_SZ: f32 = 28.0;
const SUB_SZ: f32 = 18.0;
const ROW_SZ: f32 = 18.0;

/// Logo side length as a fraction of the info-block height.
const LOGO_FRAC: f32 = 0.62;

fn item_h(item: &Item) -> f32 {
    match item {
        Item::Gap => H_GAP,
        _ => H_ROW,
    }
}

fn info_height(c: &CardContent) -> f32 {
    let sub = if c.subtitle.is_some() { H_SUB } else { 0.0 };
    let body: f32 = c.items.iter().map(item_h).sum();
    H_TITLE + sub + H_DIV + body
}

fn logo_side(c: &CardContent, has_logo: bool) -> f32 {
    if has_logo {
        (info_height(c) * LOGO_FRAC).clamp(120.0, 420.0)
    } else {
        0.0
    }
}

pub fn base_size(info: &SystemInfo, has_logo: bool, fields: Option<&[String]>) -> (u32, u32) {
    let c = content::extract(info, fields);
    let info_h = info_height(&c);
    let lside = logo_side(&c, has_logo);
    let logo_block = if has_logo { lside + GAP } else { 0.0 };
    let width = PAD + logo_block + INFO_W + PAD;
    let height = PAD + info_h.max(lside) + PAD;
    (width.round() as u32, height.round() as u32)
}

pub fn render(p: &RenderParams) -> RgbaImage {
    let s = p.scale.max(0.1);
    let i = |v: f32| (v * s).round() as i32;
    let u = |v: f32| (v * s).round().max(1.0) as u32;
    let t = p.theme;

    let c = content::extract(p.info, p.fields);
    let (bw, bh) = base_size(p.info, p.has_logo(), p.fields);
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

    // Logo, vertically centered in its column.
    let mut info_x = i(PAD);
    if p.has_logo() {
        let side = u(logo_side(&c, true));
        let distro_id = p.info.distro_id.as_deref().unwrap_or("");
        if let Some(logo_img) = logo::render(p.logo, p.logo_image, Some(distro_id), side, p.brand) {
            let lx = i(PAD);
            let ly = (height as i32 - logo_img.height() as i32) / 2;
            imageops::overlay(&mut img, &logo_img, lx as i64, ly as i64);
        }
        info_x = i(PAD) + side as i32 + i(GAP);
    }

    // Info block, vertically centered.
    let head = &p.fonts.heading;
    let body = &p.fonts.body;
    let info_h_px = u(info_height(&c));
    let mut y = (height as i32 - info_h_px as i32) / 2;
    let value_x = info_x + i(VALUE_COL);
    let row_sz = ROW_SZ * s;
    let char_w = body.char_w(row_sz, false);
    let max_chars = (((width as i32 - value_x - i(PAD)) as f32) / char_w).floor() as usize;

    head.text(
        &mut img,
        info_x,
        y + i(4.0),
        TITLE_SZ * s,
        p.brand,
        true,
        &c.title,
    );
    y += i(H_TITLE);
    if let Some(sub) = &c.subtitle {
        head.text(&mut img, info_x, y + i(2.0), SUB_SZ * s, t.dim, false, sub);
        y += i(H_SUB);
    }
    let rule_w = (width as i32 - info_x - i(PAD)).max(1) as u32;
    draw::rect(&mut img, info_x, y + i(H_DIV / 2.0), rule_w, u(2.0), t.rule);
    y += i(H_DIV);

    let bar_w = u(BAR_W);
    let bar_h = u(BAR_H);
    let detail_x = value_x + bar_w as i32 + i(DETAIL_GAP);
    for item in &c.items {
        match item {
            Item::Kv { key, value } => {
                head.text(&mut img, info_x, y + i(7.0), row_sz, t.accent, true, key);
                let shown = draw::truncate(value, max_chars);
                body.text(&mut img, value_x, y + i(7.0), row_sz, t.text, false, &shown);
                y += i(H_ROW);
            }
            Item::Bar(b) => {
                head.text(&mut img, info_x, y + i(7.0), row_sz, t.accent, true, &b.key);
                let bar_y = y + (i(H_ROW) - bar_h as i32) / 2;
                draw::bar(&mut img, t, value_x, bar_y, bar_w, bar_h, b.ratio());
                body.text(
                    &mut img,
                    detail_x,
                    y + i(7.0),
                    row_sz,
                    t.text,
                    false,
                    &b.detail(true),
                );
                y += i(H_ROW);
            }
            Item::Gap => y += i(H_GAP),
        }
    }

    img
}
