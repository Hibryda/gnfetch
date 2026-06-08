# FAQ

### The graphical card doesn't show — I only get text.

Your terminal probably doesn't support inline graphics. gnfetch needs the Kitty graphics
protocol, iTerm2 inline images, or Sixel (Kitty, WezTerm, iTerm2, Konsole, foot, …). Run
`gnfetch --probe` to see what was detected. You can always export with `gnfetch --save card.png`.

### The card looks squeezed or tiny.

gnfetch sizes the card from the terminal's reported cell pixel size. If your terminal doesn't
report it, it falls back to assumptions. `--probe` shows the chosen dimensions; `--save --width`
gives you exact control for files.

### Which fonts does it use?

By default, your system's sans and monospace fonts. Override per slot with `--sans`, `--serif`,
`--mono` (an installed family name, or a bundled font from `--list-fonts`). The first run after
choosing a *system* font scans the font database once.

### Is it fast?

Yes — it's a one-shot program like neofetch, no daemon. GPU and package counts shell out to the
usual tools (`lspci`, `dpkg-query`/`rpm`/`pacman`/`flatpak`); everything else is direct syscalls
or `/proc` reads.

### Does piping work? (`gnfetch | head`, scripts)

Yes. On a closed pipe gnfetch exits quietly like any Unix tool. For machine-readable output use
`--mode ansi` or export an image with `--save`.

### Can I use my own wallpaper / a URL as the background?

Yes: `--background-image ~/wall.png` (a file), `--background-image https://…/bg.jpg` (HTTPS
only, downloaded on render), or a bundled name (`--list-backgrounds`). Control scaling with
`--background-fit` and brightness with `--no-darken`.

### Windows / macOS / BSD?

Linux is the primary, tested target. macOS and the BSDs are stretch goals — collectors are
behind `#[cfg(...)]` boundaries, but they aren't validated yet.

### Where do the bundled images and logos come from?

Distro logos: [Simple Icons](https://simpleicons.org) (CC0). ASCII art:
[neofetch](https://github.com/dylanaraps/neofetch) (MIT). Background images: NASA public-domain
imagery. See the licenses in `assets/` in the repo.
