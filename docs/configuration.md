---
title: "Configuration"
role: how-to
parent: null
order: 2
description: "Layouts, themes, logos, backgrounds, config file and CLI flags"
---

# Configuration

gnfetch's graphical card is configurable via a config file and CLI flags. CLI
flags always override the config file, which overrides the built-in defaults.

## Config file

gnfetch reads `$XDG_CONFIG_HOME/gnfetch/config.toml` (falling back to
`~/.config/gnfetch/config.toml`). A malformed file is reported on stderr and the
defaults are used — it never blocks rendering. See
[`config.example.toml`](config.example.toml) for a fully-commented template.

## Layouts (`--layout`, `-l`)

| Layout | Description |
|--------|-------------|
| `card` | Single column: header, key/value rows, usage bars (default). |
| `neofetch` | Logo panel on the left, info column on the right. |
| `columns` | Full-width header, then info + bars in two columns (wide, short). |
| `strip` | A short, wide band — logo, title, inline stats. Good for status bars. |
| `compact` | Minimal: host, OS, uptime, CPU, memory — no bars. |

List them at runtime with `gnfetch --list-layouts`.

## Themes (`--theme`, `-t`)

The default is **`auto`**: gnfetch picks a theme that matches the detected distro's
brand colors (Debian red, Ubuntu orange, Fedora/Arch blue, Mint/Manjaro/openSUSE
green, Gentoo purple, …), falling back to the generic `default` for unknown distros. Distro themes are the brand accent over clean near-black (or near-white with `--light`) neutrals.

Aesthetic presets: `default`, `nord`, `dracula`, `gruvbox`, `catppuccin`,
`tokyonight`, `solarized`, `rose-pine`.

Distro-branded themes are also addressable directly by name (e.g. `--theme debian`,
`--theme ubuntu`). List everything with `gnfetch --list-themes`.

Override just the accent color with `--accent '#rrggbb'` (or the `accent` key); it
applies on top of whatever theme is active.

`--brand` (config key `brand`) keeps the **distro's brand color for the logo, the title,
and the accent stripe** while the rest of the card uses the chosen theme — e.g.
`gnfetch --theme dracula --brand` renders a Dracula card with a Debian-red logo/title/stripe.
(No effect for distros without a branded palette.)

## Fields (`--fields`)

Choose which info lines appear and in what order with a comma-separated list, or the
`fields` config array. Run `gnfetch --list-fields` for the keys:

`os`, `kernel`, `uptime`, `packages`, `shell`, `de`, `wm`, `cpu`, `gpu`, `memory`,
`swap`, `disk`, and `blank` (a spacer). `gpu` and `disk` expand to one line each.

```bash
gnfetch --fields cpu,memory,disk          # just CPU + memory + disks
gnfetch --fields os,kernel,blank,cpu,gpu  # with a blank spacer
```

Applies to the `card`, `neofetch`, `columns` layouts and ANSI output. The `compact` and
`strip` layouts are curated and ignore `--fields`. (Note: `os` adds an explicit OS row in
addition to the distro shown in the header.)

## Logos (`--logo`)

| Value | Description |
|-------|-------------|
| `drawn` | Bundled vector logo (default). Uses a recolored Simple Icons SVG (CC0) for 47 distros/OSes; falls back to a generic Tux SVG for unknown Linux distros. Logos are tinted to the theme accent. |
| `ascii` | Neofetch-style ASCII art rendered as text (~290 distro ids). |
| `image` | A user-supplied image — set `--logo-image <PATH>`. |
| `off` | No logo. |

`neofetch` and `strip` show the logo beside the title; `card`, `columns` and `compact`
show it as a badge in the header's top-right corner. Bundled SVGs are from
[Simple Icons](https://github.com/simple-icons/simple-icons) (CC0); see
`assets/logos/LICENSE.txt`.

Preview any distro's logo with `--distro <id>`, e.g. `gnfetch -l neofetch --distro arch`.

## Fonts (`--sans` / `--serif` / `--mono`)

The graphical card uses two font slots: a **heading** font (title, subtitle, keys)
and a **body** font (values, usage-bar details, footer).

- `--sans [FONT]` — heading font. The flag alone uses the **system default sans**;
  with a value (`--sans Inter`, `--sans lobster`) it uses that family.
- `--serif [FONT]` — heading font as a serif (system default serif, or a name).
- `--mono [FONT]` — body font. Flag alone = **system default monospace**.

Defaults (no flags): system sans heading + system monospace body. If a font isn't
found, gnfetch falls back to its bundled fonts. Config keys: `sans`, `serif`, `mono`.

Bundled fonts (always available, OFL/Apache — see `assets/fonts/OFL-*.txt`):
`poppins`, `dejavu-mono` (clean), and `pacifico`, `lobster`, `righteous`, `bungee`
(fancy). Any installed system font is settable by family name. List with
`gnfetch --list-fonts`.

```bash
gnfetch --sans lobster                 # fancy script heading, mono values
gnfetch --sans Inter --mono "Fira Code"
gnfetch --serif                        # system serif heading
```

## Backgrounds (`--background`)

| Value | Description |
|-------|-------------|
| `solid` | Solid theme background (default). |
| `gradient` | Vertical gradient (alias for `linear-90`). |
| `diagonal` | Diagonal gradient (alias for `linear-45`). |
| `linear-<angle>` | Linear gradient at any angle in degrees (`0` = left→right, `90` = top→bottom). E.g. `linear-30`, `linear-135`. |
| `horizontal` | Horizontal gradient (alias for `linear-0`). |
| `radial` | Radial glow — brighter center fading to darker edges. |
| `grid` | Subtle grid lines over the solid background. |
| `dots` | Subtle dot grid over the solid background. |
| `image` | A background image (`--background-image`), darkened for contrast. |
| `transparent` | Fully transparent background. |

Procedural backgrounds derive from the active theme's colors. The gradient/radial
endpoints are computed for a clearly visible spread: a lightened, slightly
accent-tinted "bright" end and a darkened "dark" end. Gradients are dithered
(hashed white noise) so they don't band at 8-bit color depth.

### Image backgrounds (`--background-image`, `--background-fit`)

`--background-image <FILE|URL|NAME>` sets an image background (and implies
`--background image`). The source can be:

- a **file path** — `--background-image ~/wallpaper.png`
- an **`https://` URL** — downloaded on render (timeout + size cap; `http://` is
  refused). Requires network access; failures fall back to a solid background.
- a **bundled name** — a curated set of CC0 / public-domain images ships with
  gnfetch; list them with `--list-backgrounds` (e.g. `andromeda`, `aurora`,
  `carina`, `earth`, `helix`). See `assets/backgrounds/LICENSE.txt` for credits.

`--background-fit <FIT>` controls scaling (default `fill`):

| Fit | Behavior |
|-----|----------|
| `fill` | Scale to cover the card, cropping the overflow (preserves aspect). Default. |
| `fit` | Scale to fit inside the card, letterboxing the rest (preserves aspect). |
| `stretch` | Scale each axis to fill the card exactly (distorts aspect). |
| `center` | No scaling — center the image, cropping or padding as needed. |

The image is darkened (~45% brightness) so foreground text stays legible; pass
`--no-darken` to keep it at full brightness (text legibility may suffer).

## Other flags

- `--light` — use a light theme (light background, dark text); default is dark.
- `--mode auto\|ansi\|image` — force a renderer (default `auto`).
- `--save <PATH>` / `-o` — export the card to an image file and exit.
- `--width <PX>` — target width in pixels for the saved card (`--save` only; default auto). The exact width may differ by a few pixels because layouts round at scale; the scale is clamped to keep tiny widths legible and huge ones sane.
- `--probe` — print terminal detection and the chosen card size, then exit.
- `--list-themes`, `--list-layouts`, `--list-fields`, `--list-fonts`, `--list-backgrounds`.
- `--demo` — render a gallery of example cards (dummy data) showcasing the options.

## Examples

```bash
gnfetch --layout neofetch --logo drawn --theme nord
gnfetch --layout strip --theme dracula
gnfetch --layout card --background linear-30 --accent '#ff8800'
gnfetch --background-image carina                       # bundled CC0 image
gnfetch --background-image ~/wallpaper.png --background-fit fit
gnfetch --background-image https://example.com/bg.jpg   # from a URL
gnfetch --layout neofetch --logo image --logo-image ~/me.png --save card.png
```
