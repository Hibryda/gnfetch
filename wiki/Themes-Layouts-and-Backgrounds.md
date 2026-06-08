# Themes, Layouts, and Backgrounds

Everything here works on both the graphical card and (where applicable) the ANSI output. Run
`gnfetch --demo` to see all of it at once.

## Layouts

| Layout | Description |
|--------|-------------|
| `card` | Single column: header, key/value rows, usage bars (default). |
| `neofetch` | Logo panel on the left, info column on the right. |
| `columns` | Full-width header, info packed into two columns. |
| `strip` | Short, wide status band. |
| `compact` | Just the essentials, no bars. |

```bash
gnfetch --layout neofetch
gnfetch --layout strip
```

![strip layout](https://raw.githubusercontent.com/Hibryda/gnfetch/main/assets/screenshots/strip.png)

## Themes

- `auto` (default) matches your distro's brand colours over clean neutrals.
- 8 aesthetic presets: `nord`, `dracula`, `gruvbox`, `catppuccin`, `tokyonight`, `solarized`,
  `rose-pine`, `default`.
- 21 distro palettes, addressable by name (`--theme debian`).
- `--light` for a light variant, `--accent '#rrggbb'` to override, `--brand` to keep the
  distro colour while using another theme.

```bash
gnfetch --theme dracula
gnfetch --theme gruvbox --light
gnfetch --accent '#ff8800'
```

![themes](https://raw.githubusercontent.com/Hibryda/gnfetch/main/assets/screenshots/gallery.png)

## Logos

```bash
gnfetch --logo drawn      # recoloured distro SVG (default)
gnfetch --logo ascii      # neofetch-style ASCII art
gnfetch --logo image --logo-image ~/me.png
gnfetch --logo off
gnfetch --distro arch     # preview another distro's logo/theme
```

## Backgrounds

```bash
gnfetch --background linear-30          # gradient at any angle
gnfetch --background radial
gnfetch --background dots
gnfetch --background-image carina       # a bundled CC0 image
gnfetch --background-image ~/wall.png --background-fit fit
gnfetch --background-image https://example.com/bg.jpg
```

Bundled CC0 images (`--list-backgrounds`): `andromeda`, `aurora`, `carina`, `earth`, `helix`
(NASA public domain). Image backgrounds are dimmed for text legibility; pass `--no-darken` to
keep full brightness. Gradients are dithered so they never band.
