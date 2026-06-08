# gnfetch wiki

**gnfetch** renders your system information as classic ANSI output *and* as a rich graphical
"system visiting card" inline in terminals that support graphics (Kitty / iTerm2 / Sixel).

![gnfetch](https://raw.githubusercontent.com/Hibryda/gnfetch/main/assets/screenshots/hero.png)

## Pages

- **[[Installation]]** — packages, prebuilt binaries, and building from source.
- **[[Configuration]]** — the config file and every flag.
- **[[Themes Layouts and Backgrounds]]** — making it look the way you want.
- **[[Building from Source]]** — toolchain, generated modules, packaging.
- **[[FAQ]]** — terminal support, fonts, performance, troubleshooting.

## Quick start

```bash
gnfetch              # auto-detect the terminal and render
gnfetch --demo       # a captioned tour of every option
gnfetch --help       # all flags
```

If your terminal can't show inline graphics, gnfetch falls back to ANSI automatically. The
graphical card always works via `--save card.png` regardless of terminal.
