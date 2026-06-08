# Packaging

How gnfetch is packaged for distribution.

## Files

| File | Purpose |
|------|---------|
| `PKGBUILD` | AUR **`gnfetch`** — builds from source. |
| `PKGBUILD-bin` | AUR **`gnfetch-bin`** — installs the prebuilt static (musl) release binary. |
| `build-release.sh` | Builds all release artifacts (tarballs, `.deb`, `.rpm`, AppImage) into `dist/`. |
| `gnfetch.desktop`, `gnfetch.png` | Desktop entry + icon for the AppImage. |

The `.deb` / `.rpm` metadata lives in `Cargo.toml` (`[package.metadata.deb]`,
`[package.metadata.generate-rpm]`), consumed by `cargo deb` / `cargo generate-rpm`.

## ⚠️ The source PKGBUILD needs `options=('!lto')`

Without it the from-source build fails to link with `undefined symbol: ring_core_*`.
**It is not a linker, binutils, or LTO-in-Rust problem** — plain `cargo build` links fine.
makepkg's default `lto` option injects `-flto` into `CFLAGS`, which makes `ring`'s build
script compile its hand-written assembly into LTO objects that carry no real symbols, so the
final link can't resolve them. `options=('!lto')` disables makepkg's LTO and fixes it. (Once
gnfetch moves to a `ring` that tolerates `-flto`, this can be revisited.)

## Releasing a new version

1. Bump `version` in `Cargo.toml`, update `CHANGELOG.md`, tag `vX.Y.Z`, push, and create the
   GitHub release (attach the `build-release.sh` artifacts).
2. Update the AUR packages (one repo each, push to branch `master`):
   - **gnfetch**: bump `pkgver` and the source `sha256sums` (the GitHub tag tarball), then
     regenerate `.SRCINFO` with `makepkg --printsrcinfo`.
   - **gnfetch-bin**: bump `pkgver` and `sha256sums_x86_64` / `sha256sums_aarch64` (the musl
     release tarballs).
3. Validate any PKGBUILD without an Arch machine:
   `docker run --rm -v "$PWD":/p archlinux bash -c 'pacman -Sy --noconfirm base-devel rust git;
   useradd -m b; cp /p/PKGBUILD /home/b; chown -R b /home/b; su b -c "cd /home/b && makepkg -f"'`
