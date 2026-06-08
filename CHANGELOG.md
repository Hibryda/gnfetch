# Changelog

All notable changes to gnfetch are documented here. The format follows
[Keep a Changelog](https://keepachangelog.com/en/1.1.0/), and the project aims to
follow [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [Unreleased]

## [0.1.0] - 2026-06-08

First public release.

### Added

- **Two render paths** — classic ANSI (logo + aligned key/value block, truecolor) and a
  graphical "system visiting card" emitted inline via the Kitty graphics protocol, iTerm2
  inline images, or Sixel, with capability detection and an ANSI fallback.
- **Collectors** for OS, kernel, CPU, GPU, memory, swap, disk, uptime, packages, shell, and
  desktop environment / window manager, each degrading gracefully to `None`.
- **5 layouts** — `card`, `neofetch`, `columns`, `strip`, `compact`.
- **Themes** — `auto` distro-branded palettes (21 distros) over clean neutrals, 8 aesthetic
  presets, `--light`, `--accent`, and `--brand`; applied to both the card and ANSI output.
- **Logos** — `drawn` (47 recoloured Simple Icons SVGs + generic fallback), `ascii` (262
  neofetch arts), `image`, or `off`.
- **Backgrounds** — solid, gradient/diagonal/radial/grid/dots, any-angle `linear-<angle>`,
  and image backgrounds from a file, an `https://` URL, or a bundled CC0 image (5 NASA
  public-domain images). `--background-fit fill|fit|stretch|center` and `--no-darken`.
  Gradients are dithered to avoid 8-bit banding.
- **Fonts** — two-slot heading/body system using system fonts by default, with `--sans`,
  `--serif`, `--mono` (any installed family or a bundled font).
- **Configurable fields** — choose which info lines appear and their order.
- **Config file** — `~/.config/gnfetch/config.toml`, overridden by CLI flags.
- **Utilities** — `--save <PATH>` (PNG export), `--width`, `--demo`, `--probe`, and
  `--list-themes`/`-layouts`/`-fields`/`-fonts`/`-backgrounds`.

### Security

- URL background fetches are HTTPS-only with a download timeout, a size cap, bounded
  redirects, and explicit image-decode limits (pixel-bomb guard).

[Unreleased]: https://github.com/Hibryda/gnfetch/compare/v0.1.0...HEAD
[0.1.0]: https://github.com/Hibryda/gnfetch/releases/tag/v0.1.0
