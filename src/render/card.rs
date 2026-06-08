//! The default single-column "card" layout: accent stripe, header, then the
//! configured items (key/value rows, usage bars, spacers). Resolution-independent
//! — everything scales from `p.scale`.

use super::background;
use super::content::{self, CardContent, Item};
use super::draw;
use super::layout::RenderParams;
use super::logo;
use crate::model::SystemInfo;
use image::{RgbaImage, imageops};

/// Gap between the info block and a header logo badge.
const LOGO_GAP: f32 = 16.0;

const BASE_WIDTH: f32 = 860.0;
const PAD: f32 = 32.0;
const CONTENT_X: f32 = 40.0;
const STRIPE_W: f32 = 8.0;
const VALUE_COL: f32 = 176.0;
const BAR_W: f32 = 210.0;
const BAR_H: f32 = 14.0;
const DETAIL_GAP: f32 = 14.0;

const H_TITLE: f32 = 44.0;
const H_SUB: f32 = 28.0;
const H_DIV: f32 = 20.0;
const H_ROW: f32 = 30.0;
const H_GAP: f32 = 8.0;
const H_FOOT: f32 = 26.0;

const TITLE_SZ: f32 = 30.0;
const SUB_SZ: f32 = 18.0;
const ROW_SZ: f32 = 18.0;
const FOOT_SZ: f32 = 15.0;

/// Height of one body item at scale 1.0.
fn item_h(item: &Item) -> f32 {
    match item {
        Item::Gap => H_GAP,
        _ => H_ROW,
    }
}

/// Base (scale 1.0) pixel dimensions for `info` with the given field order.
pub fn base_size(info: &SystemInfo, fields: Option<&[String]>) -> (u32, u32) {
    let c = content::extract(info, fields);
    (BASE_WIDTH.round() as u32, base_height(&c).round() as u32)
}

fn base_height(c: &CardContent) -> f32 {
    let sub = if c.subtitle.is_some() { H_SUB } else { 0.0 };
    let body: f32 = c.items.iter().map(item_h).sum();
    PAD + H_TITLE + sub + H_DIV + body + H_GAP + H_FOOT + PAD
}

pub fn render(p: &RenderParams) -> RgbaImage {
    let s = p.scale.max(0.1);
    let i = |v: f32| (v * s).round() as i32;
    let u = |v: f32| (v * s).round().max(1.0) as u32;
    let t = p.theme;

    let c = content::extract(p.info, p.fields);
    let width = u(BASE_WIDTH);
    let height = u(base_height(&c));

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

    // Optional logo badge in the header's top-right corner. Its left edge bounds
    // the divider so the line doesn't run under it.
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

    let bar_w = u(BAR_W);
    let bar_h = u(BAR_H);
    let detail_x = value_x + bar_w as i32 + i(DETAIL_GAP);
    for item in &c.items {
        match item {
            Item::Kv { key, value } => {
                head.text(&mut img, cx, y + i(7.0), row_sz, t.accent, true, key);
                let shown = draw::truncate(value, max_chars);
                body.text(&mut img, value_x, y + i(7.0), row_sz, t.text, false, &shown);
                y += i(H_ROW);
            }
            Item::Bar(b) => {
                head.text(&mut img, cx, y + i(7.0), row_sz, t.accent, true, &b.key);
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

    body.text(
        &mut img,
        cx,
        y + i(H_GAP),
        FOOT_SZ * s,
        t.dim,
        false,
        "rendered by gnfetch",
    );

    img
}
