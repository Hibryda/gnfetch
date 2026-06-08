#!/usr/bin/env python3
"""Generate src/render/bg_images.rs from the images in assets/backgrounds/.

Each <name>.{jpg,jpeg,png} becomes an include_bytes! entry keyed by its stem
(lowercased). Licensing/attribution for every file lives in
assets/backgrounds/LICENSE.txt. Regenerate with:

    python3 scripts/gen_backgrounds.py
"""
import os

BG_DIR = "assets/backgrounds"
OUT = "src/render/bg_images.rs"
EXTS = (".jpg", ".jpeg", ".png")


def main():
    files = sorted(
        f
        for f in os.listdir(BG_DIR)
        if f.lower().endswith(EXTS) and not f.startswith("_")
    )
    entries = [(os.path.splitext(f)[0].lower(), f) for f in files]

    lines = [
        "//! Embedded CC0 / public-domain background images. DO NOT EDIT BY HAND.",
        "//! Licensing + source per file: assets/backgrounds/LICENSE.txt.",
        "//! Regenerate via scripts/gen_backgrounds.py.",
        "",
        "/// Names of the bundled background images, for `--list-backgrounds`.",
        "pub fn names() -> &'static [&'static str] {",
        "    &[" + ", ".join(f'"{name}"' for name, _ in entries) + "]",
        "}",
        "",
        "/// Embedded image bytes for a bundled name (case-insensitive), or `None`.",
        "pub fn bundled(name: &str) -> Option<&'static [u8]> {",
        "    match name.trim().to_ascii_lowercase().as_str() {",
    ]
    for name, fname in entries:
        lines.append(
            f'        "{name}" => Some(include_bytes!("../../{BG_DIR}/{fname}")),'
        )
    lines += [
        "        _ => None,",
        "    }",
        "}",
        "",
    ]
    os.makedirs(os.path.dirname(OUT), exist_ok=True)
    open(OUT, "w", encoding="utf-8").write("\n".join(lines))
    print(f"wrote {OUT}: {len(entries)} background images")


if __name__ == "__main__":
    main()
