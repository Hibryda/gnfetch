#!/usr/bin/env bash
# Build all release artifacts locally into ./dist.
#
# Produces, when the relevant toolchain is present (each step is skipped with a
# warning if its tool is missing):
#   - tarballs for x86_64/aarch64, glibc + static musl
#   - a Debian .deb           (cargo-deb)
#   - a Fedora/openSUSE .rpm  (cargo-generate-rpm)
#   - an AppImage             (appimagetool)
#
# One-time setup:
#   rustup target add x86_64-unknown-linux-musl aarch64-unknown-linux-gnu aarch64-unknown-linux-musl
#   cargo install cargo-deb cargo-generate-rpm cargo-zigbuild   # zigbuild needs `zig`
#   # appimagetool: https://github.com/AppImage/appimagetool
set -euo pipefail

cd "$(dirname "$0")/.."
ROOT="$(pwd)"
DIST="$ROOT/dist"
NAME="gnfetch"
VERSION="$(sed -n 's/^version = "\(.*\)"/\1/p' Cargo.toml | head -1)"

have() { command -v "$1" >/dev/null 2>&1; }
log()  { printf '\033[1;36m==>\033[0m %s\n' "$*"; }
warn() { printf '\033[1;33mskip:\033[0m %s\n' "$*" >&2; }

rm -rf "$DIST"; mkdir -p "$DIST"
log "gnfetch $VERSION -> $DIST"

# How to build a given target: prefer cargo-zigbuild (easy cross), else plain cargo.
build_target() {
    local target="$1"
    rustup target add "$target" >/dev/null 2>&1 || true
    if have cargo-zigbuild; then
        cargo zigbuild --release --target "$target"
    elif [ "$target" = "$(rustc -vV | sed -n 's/host: //p')" ]; then
        cargo build --release --target "$target"
    else
        warn "no cargo-zigbuild and $target is not the host; skipping"
        return 1
    fi
}

tarball() {
    local target="$1" bin="target/$target/release/$NAME"
    [ -f "$bin" ] || { warn "no binary for $target"; return 1; }
    local stage="$DIST/$NAME-$VERSION-$target"
    mkdir -p "$stage"
    cp "$bin" "$stage/"; cp README.md LICENSE "$stage/"
    tar -C "$DIST" -czf "$DIST/$NAME-$target.tar.gz" "$(basename "$stage")"
    rm -rf "$stage"
    log "tarball: $NAME-$target.tar.gz"
}

# --- portable tarballs ---
for t in x86_64-unknown-linux-gnu x86_64-unknown-linux-musl \
         aarch64-unknown-linux-gnu aarch64-unknown-linux-musl; do
    log "building $t"
    if build_target "$t"; then tarball "$t"; fi
done

# Ensure a native release build exists for the distro packages below.
[ -f target/release/$NAME ] || cargo build --release

# --- .deb ---
if have cargo-deb; then
    log "building .deb"
    cargo deb --no-build -o "$DIST/" 2>/dev/null || cargo deb -o "$DIST/"
else warn "cargo-deb not installed; skipping .deb"; fi

# --- .rpm ---
if have cargo-generate-rpm; then
    log "building .rpm"
    cargo generate-rpm -o "$DIST/" || warn ".rpm build failed"
else warn "cargo-generate-rpm not installed; skipping .rpm"; fi

# --- AppImage ---
if have appimagetool; then
    log "building AppImage"
    APPDIR="$DIST/$NAME.AppDir"
    mkdir -p "$APPDIR/usr/bin" "$APPDIR/usr/share/applications" \
             "$APPDIR/usr/share/icons/hicolor/256x256/apps"
    install -Dm755 target/release/$NAME "$APPDIR/usr/bin/$NAME"
    install -Dm644 packaging/$NAME.desktop "$APPDIR/usr/share/applications/$NAME.desktop"
    cp packaging/$NAME.desktop "$APPDIR/$NAME.desktop"
    install -Dm644 packaging/$NAME.png "$APPDIR/usr/share/icons/hicolor/256x256/apps/$NAME.png"
    cp packaging/$NAME.png "$APPDIR/$NAME.png"
    printf '#!/bin/sh\nexec "$(dirname "$0")/usr/bin/%s" "$@"\n' "$NAME" > "$APPDIR/AppRun"
    chmod +x "$APPDIR/AppRun"
    ARCH=x86_64 appimagetool "$APPDIR" "$DIST/$NAME-x86_64.AppImage" >/dev/null 2>&1 \
        && log "AppImage: $NAME-x86_64.AppImage" || warn "appimagetool failed"
    rm -rf "$APPDIR"
else warn "appimagetool not installed; skipping AppImage"; fi

log "done. artifacts in $DIST:"
ls -1 "$DIST"
