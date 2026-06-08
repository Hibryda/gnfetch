# Configuration

gnfetch reads `~/.config/gnfetch/config.toml` (or `$XDG_CONFIG_HOME/gnfetch/config.toml`) and
lets CLI flags override any value. A missing or invalid file is reported and ignored — gnfetch
always runs with sensible defaults.

The complete, always-current reference lives in the repository at
[`docs/configuration.md`](https://github.com/Hibryda/gnfetch/blob/main/docs/configuration.md).
This page is a quick orientation.

## Example config

```toml
layout = "neofetch"      # card | neofetch | columns | strip | compact
theme  = "auto"          # auto | a preset | a distro name
# accent = "#ff8800"
light  = false
logo   = "drawn"         # drawn | ascii | image | off
background = "linear-30" # solid | gradient | diagonal | radial | grid | dots | linear-<angle> | image | transparent
# background-image = "carina"        # a file, an https URL, or a bundled name
background-fit = "fill"  # fill | fit | stretch | center
no-darken = false
fields = ["cpu", "gpu", "memory", "swap", "disk"]
# sans = "Inter"         # heading font (family name or a bundled font)
# mono = "JetBrains Mono" # body font
```

## Discovering values

```bash
gnfetch --list-themes
gnfetch --list-layouts
gnfetch --list-fields
gnfetch --list-fonts
gnfetch --list-backgrounds
```

## Precedence

`CLI flag` > `config.toml` > built-in default. `--save <PATH>` exports the card to a PNG from
any terminal; `--width <PX>` sets the exported width.

See **[[Themes Layouts and Backgrounds]]** for visual examples.
