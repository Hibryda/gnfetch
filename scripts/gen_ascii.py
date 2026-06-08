#!/usr/bin/env python3
"""Generate src/render/ascii_logos.rs from the neofetch script (MIT).

neofetch embeds ~270 distro ASCII logos as heredoc blocks inside
get_distro_ascii(). We parse each block, strip the ${cN} color placeholders,
and emit a Rust lookup module. Regenerate with:

    curl -s https://raw.githubusercontent.com/dylanaraps/neofetch/master/neofetch -o /tmp/neofetch.sh
    python3 scripts/gen_ascii.py /tmp/neofetch.sh src/render/ascii_logos.rs
"""
import re
import sys


def parse(src: str):
    art_re = re.compile(r"read -rd '' ascii_data <<'EOF'\n(.*?)\nEOF", re.DOTALL)
    entries = []
    for m in art_re.finditer(src):
        art = m.group(1)
        pre = src[: m.start()]
        pat_line = None
        for ln in reversed(pre.splitlines()):
            s = ln.strip()
            if s.endswith(")") and ('"' in s or "'" in s) and "EOF" not in s:
                pat_line = s
                break
        if not pat_line:
            continue
        names = re.findall(r'"([^"]+)"', pat_line) + re.findall(r"'([^']+)'", pat_line)
        keys = []
        for n in names:
            k = re.sub(r"[^a-z0-9]", "", n.lower())
            if k and k != "default":
                keys.append(k)
        if not keys:
            continue
        art = re.sub(r"\$\{c[0-9]+\}", "", art)
        lines = [l.rstrip() for l in art.split("\n")]
        while lines and lines[-1] == "":
            lines.pop()
        while lines and lines[0] == "":
            lines.pop(0)
        if lines:
            entries.append((keys, "\n".join(lines)))
    return entries


def escape(s: str) -> str:
    return s.replace("\\", "\\\\").replace('"', '\\"').replace("\n", "\\n")


def main():
    src = open(sys.argv[1], encoding="utf-8", errors="replace").read()
    out_path = sys.argv[2]
    entries = parse(src)

    # Map each key to its art (first claim wins), then keep only reachable arts.
    key_to_art = {}
    for keys, art in entries:
        for k in keys:
            key_to_art.setdefault(k, art)
    arts = []
    seen = set()
    for art in key_to_art.values():
        if art not in seen:
            seen.add(art)
            arts.append(art)
    art_idx = {a: i for i, a in enumerate(arts)}
    key_to_idx = {k: art_idx[a] for k, a in key_to_art.items()}

    lines = [
        "//! ASCII distro logos generated from neofetch (MIT). DO NOT EDIT BY HAND.",
        "//! Source: https://github.com/dylanaraps/neofetch — regenerate via scripts/gen_ascii.py",
        "",
        "/// ASCII art for a distro id (any case / punctuation), or `None` if unknown.",
        "pub fn ascii_for(id: &str) -> Option<&'static str> {",
        "    let mut key: String = id",
        "        .chars()",
        "        .filter(|c| c.is_ascii_alphanumeric())",
        "        .flat_map(|c| c.to_lowercase())",
        "        .collect();",
        "    // A few os-release ids differ from neofetch's display-name keys.",
        "    key = match key.as_str() {",
        '        "pop" => "popos".to_string(),',
        '        "rhel" => "redhat".to_string(),',
        "        _ => key,",
        "    };",
        "    match key.as_str() {",
    ]

    # Group keys by index for compact arms.
    idx_keys = {}
    for k, idx in sorted(key_to_idx.items()):
        idx_keys.setdefault(idx, []).append(k)
    for idx in sorted(idx_keys):
        pats = " | ".join(f'"{k}"' for k in idx_keys[idx])
        lines.append(f"        {pats} => Some(L{idx}),")
    lines.append("        _ => None,")
    lines.append("    }")
    lines.append("}")
    lines.append("")
    for idx, art in enumerate(arts):
        lines.append(f'static L{idx}: &str = "{escape(art)}";')
    lines.append("")

    open(out_path, "w", encoding="utf-8").write("\n".join(lines))
    print(f"wrote {out_path}: {len(arts)} arts, {len(key_to_idx)} keys")


if __name__ == "__main__":
    main()
