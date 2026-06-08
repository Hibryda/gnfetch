#!/usr/bin/env python3
"""Generate src/render/svg_logos.rs from the SVGs in assets/logos/.

Each <key>.svg becomes an include_bytes! entry keyed by os-release id. A small
alias table maps differing ids onto the available files. Regenerate with:

    python3 scripts/gen_svg.py
"""
import os

LOGO_DIR = "assets/logos"
OUT = "src/render/svg_logos.rs"

# os-release id (normalized) -> available file key
ALIASES = {
    "archlinux": "arch",
    "pop": "popos",
    "rhel": "redhat",
    "coreos": "fedora",
    "raspberrypi": "raspbian",
    "alpinelinux": "alpine",
    "kalilinux": "kali",
    "opensuseleap": "opensuse",
    "opensusetumbleweed": "opensuse",
    "suse": "opensuse",
    "elementaryos": "elementary",
    "artixlinux": "artix",
    "voidlinux": "void",
    "rockylinux": "rocky",
    "garudalinux": "garuda",
    "kdeneon": "neon",
    "mxlinux": "mx",
    "nobaralinux": "nobara",
    "qubesos": "qubes",
    "pve": "proxmox",
    "fedoraasahiremix": "asahi",
    "manjarolinux": "manjaro",
    "ubuntumatelinux": "ubuntumate",
}


def main():
    # Skip underscore-prefixed files (e.g. _generic.svg, handled directly in logo.rs).
    keys = sorted(
        f[:-4]
        for f in os.listdir(LOGO_DIR)
        if f.endswith(".svg") and not f.startswith("_")
    )
    lines = [
        "//! Embedded distro logo SVGs from Simple Icons (CC0). DO NOT EDIT BY HAND.",
        "//! Source: https://github.com/simple-icons/simple-icons — see assets/logos/LICENSE.txt.",
        "//! Regenerate via scripts/gen_svg.py.",
        "",
        "/// Embedded SVG bytes for a distro id (any case / punctuation), or `None`.",
        "pub fn svg_for(id: &str) -> Option<&'static [u8]> {",
        "    let mut key: String = id",
        "        .chars()",
        "        .filter(|c| c.is_ascii_alphanumeric())",
        "        .flat_map(|c| c.to_lowercase())",
        "        .collect();",
        "    key = match key.as_str() {",
    ]
    for alias, target in sorted(ALIASES.items()):
        if target in keys:
            lines.append(f'        "{alias}" => "{target}".to_string(),')
    lines += [
        "        _ => key,",
        "    };",
        "    match key.as_str() {",
    ]
    for k in keys:
        lines.append(
            f'        "{k}" => Some(include_bytes!("../../{LOGO_DIR}/{k}.svg")),'
        )
    lines += [
        "        _ => None,",
        "    }",
        "}",
        "",
    ]
    open(OUT, "w", encoding="utf-8").write("\n".join(lines))
    print(f"wrote {OUT}: {len(keys)} svg logos")


if __name__ == "__main__":
    main()
