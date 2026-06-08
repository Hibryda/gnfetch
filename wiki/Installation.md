# Installation

Prebuilt artifacts for Linux x86_64 and aarch64 are attached to each
[release](https://github.com/Hibryda/gnfetch/releases).

## Debian / Ubuntu

```bash
curl -LO https://github.com/Hibryda/gnfetch/releases/latest/download/gnfetch_amd64.deb
sudo dpkg -i gnfetch_amd64.deb
```

## Fedora / openSUSE

```bash
sudo rpm -i https://github.com/Hibryda/gnfetch/releases/latest/download/gnfetch.x86_64.rpm
```

## Arch Linux

```bash
yay -S gnfetch-bin   # or paru -S gnfetch-bin
```

## AppImage (any distro)

```bash
curl -LO https://github.com/Hibryda/gnfetch/releases/latest/download/gnfetch-x86_64.AppImage
chmod +x gnfetch-x86_64.AppImage
./gnfetch-x86_64.AppImage
```

## Prebuilt binary

The `*-musl` tarball is a static binary that runs on any Linux:

```bash
curl -L https://github.com/Hibryda/gnfetch/releases/latest/download/gnfetch-x86_64-unknown-linux-musl.tar.gz | tar xz
sudo install gnfetch /usr/local/bin/
```

## From source

Requires a Rust toolchain **1.95+** ([rustup](https://rustup.rs)):

```bash
cargo install --git https://github.com/Hibryda/gnfetch
```

See **[[Building from Source]]** for development builds and packaging.

## Terminal support for the graphical card

The inline card needs a terminal speaking the Kitty graphics protocol, iTerm2 inline images,
or Sixel — e.g. **Kitty, WezTerm, iTerm2, Konsole, foot**. Elsewhere gnfetch prints ANSI.
Run `gnfetch --probe` to see what was detected.
