# Building from Source

## Toolchain

A stable Rust toolchain **1.95+** (the floor comes from `sysinfo`), via
[rustup](https://rustup.rs). Linux is the primary target.

```bash
git clone https://github.com/Hibryda/gnfetch && cd gnfetch
cargo build --release       # ./target/release/gnfetch
cargo install --path .      # into ~/.cargo/bin
```

## Checks

```bash
cargo test
cargo clippy --all-targets -- -D warnings
cargo fmt --check
```

## Generated modules

Three source modules are generated from assets and committed:

| Module | Generator | Source |
|--------|-----------|--------|
| `src/render/ascii_logos.rs` | `scripts/gen_ascii.py` | neofetch ASCII art (MIT) |
| `src/render/svg_logos.rs` | `scripts/gen_svg.py` | `assets/logos/*.svg` (Simple Icons, CC0) |
| `src/render/bg_images.rs` | `scripts/gen_backgrounds.py` | `assets/backgrounds/*.jpg` (NASA PD) |

Re-run the relevant generator after changing the underlying assets, then `cargo fmt`.

## Release artifacts

`packaging/build-release.sh` produces tarballs (x86_64/aarch64, glibc + musl), a `.deb`,
a `.rpm`, and an AppImage into `./dist`. Each step is skipped with a warning if its tool is
absent. One-time setup:

```bash
rustup target add x86_64-unknown-linux-musl aarch64-unknown-linux-gnu aarch64-unknown-linux-musl
cargo install cargo-deb cargo-generate-rpm cargo-zigbuild   # zigbuild needs `zig`
# appimagetool from https://github.com/AppImage/appimagetool
```

## Architecture

One-way pipeline: **Collectors → `SystemInfo` → Renderer → stdout**. Collectors never touch
rendering; renderers read only from `SystemInfo`. A `TerminalCaps` probe selects the graphical
or ANSI renderer, always falling back to ANSI.
