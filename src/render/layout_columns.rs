//! Multi-column layout: full-width header, then fields and bars packed into two
//! columns. Wide and short — good when the terminal has width to spare.

use super::background;
use super::content::{self, Item};
use super::draw;
use super::layout::RenderParams;
use super::logo;
use crate::model::SystemInfo;
use image::{RgbaImage, imageops};

/// Gap between the header and a logo badge.
const LOGO_GAP: f32 = 16.0;

const COLS: usize = 2;
const PAD: f32 = 34.0;
const STRIPE_W: f32 = 8.0;
const COL_W: f32 = 600.0;
const COL_GAP: f32 = 52.0;
const KEY_W: f32 = 158.0;
const BAR_W: f32 = 120.0;
const BAR_H: f32 = 14.0;
const DETAIL_GAP: f32 = 12.0;

const H_TITLE: f32 = 42.0;
const H_SUB: f32 = 28.0;
const H_DIV: f32 = 22.0;
const H_ROW: f32 = 30.0;

const TITLE_SZ: f32 = 28.0;
const SUB_SZ: f32 = 18.0;
const ROW_SZ: f32 = 18.0;

fn rows_per_col(n: usize) -> usize {
    n.div_ceil(COLS)
}

/// Body item count, excluding spacers (gaps don't apply to a column grid).
fn body_count(info: &SystemInfo, fields: Option<&[String]>) -> usize {
    content::extract(info, fields)
        .items
        .iter()
        .filter(|it| !matches!(it, Item::Gap))
        .count()
}

pub fn base_size(info: &SystemInfo, fields: Option<&[String]>) -> (u32, u32) {
    let has_sub = info.distro.is_some();
    let n = body_count(info, fields);
    let header = H_TITLE + if has_sub { H_SUB } else { 0.0 } + H_DIV;
    let height = PAD + header + rows_per_col(n) as f32 * H_ROW + PAD;
    let width = PAD + COLS as f32 * COL_W + (COLS as f32 - 1.0) * COL_GAP + PAD;
    (width.round() as u32, height.round() as u32)
}

pub fn render(p: &RenderParams) -> RgbaImage {
    let s = p.scale.max(0.1);
    let i = |v: f32| (v * s).round() as i32;
    let u = |v: f32| (v * s).round().max(1.0) as u32;
    let t = p.theme;

    let c = content::extract(p.info, p.fields);
    let (bw, bh) = base_size(p.info, p.fields);
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

    let cx = i(PAD);
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
    let row_sz = ROW_SZ * s;
    let char_w = body.char_w(row_sz, false);

    // Header (full width).
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
    let body_y = y;

    // Gaps don't fit a column grid; drop them.
    let all: Vec<&Item> = c
        .items
        .iter()
        .filter(|it| !matches!(it, Item::Gap))
        .collect();
    let per = rows_per_col(all.len()).max(1);
    let key_w = i(KEY_W);
    let bar_w = u(BAR_W);
    let bar_h = u(BAR_H);

    for (idx, item) in all.iter().enumerate() {
        let col = idx / per;
        let row = idx % per;
        let colx = cx + col as i32 * i(COL_W + COL_GAP);
        let ry = body_y + row as i32 * i(H_ROW);
        let value_x = colx + key_w;

        match item {
            Item::Kv { key, value } => {
                head.text(&mut img, colx, ry + i(7.0), row_sz, t.accent, true, key);
                let avail = (((colx + i(COL_W)) - value_x) as f32 / char_w).floor() as usize;
                body.text(
                    &mut img,
                    value_x,
                    ry + i(7.0),
                    row_sz,
                    t.text,
                    false,
                    &draw::truncate(value, avail),
                );
            }
            Item::Bar(b) => {
                head.text(&mut img, colx, ry + i(7.0), row_sz, t.accent, true, &b.key);
                let bar_y = ry + (i(H_ROW) - bar_h as i32) / 2;
                draw::bar(&mut img, t, value_x, bar_y, bar_w, bar_h, b.ratio());
                let detail_x = value_x + bar_w as i32 + i(DETAIL_GAP);
                let avail = (((colx + i(COL_W)) - detail_x) as f32 / char_w).floor() as usize;
                body.text(
                    &mut img,
                    detail_x,
                    ry + i(7.0),
                    row_sz,
                    t.text,
                    false,
                    &draw::truncate(&b.detail(false), avail),
                );
            }
            Item::Gap => {}
        }
    }

    img
}
