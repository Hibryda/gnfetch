#!/usr/bin/env bash
# Produce the PUBLIC (GitHub) repo as a fresh-history copy of the core project:
# everything tracked here, minus the Claude/AI dev tooling, with a single clean
# initial commit. The full-history repo (with .claude/, CLAUDE.md) stays on Forgejo.
#
# Usage: scripts/export-public.sh [output-dir]   (default: ../gnfetch-public)
set -euo pipefail

cd "$(dirname "$0")/.."
SRC="$(pwd)"
OUT="${1:-$SRC/../gnfetch-public}"
VERSION="$(sed -n 's/^version = "\(.*\)"/\1/p' Cargo.toml | head -1)"

# Things that must be physically absent from the public repo.
EXCLUDES=(.claude CLAUDE.md)

rm -rf "$OUT"; mkdir -p "$OUT"
# git archive emits only committed (tracked) files — no target/, dist/, scratch PNGs.
git -C "$SRC" archive --format=tar HEAD | tar -x -C "$OUT"
for e in "${EXCLUDES[@]}"; do rm -rf "${OUT:?}/$e"; done

# Public .gitignore = the current one plus the dev-tooling exclusions, so those
# files can never be re-added to the public repo by accident.
cp "$SRC/.gitignore" "$OUT/.gitignore"
cat >> "$OUT/.gitignore" <<'EOF'

# === Public repo: keep AI / dev tooling out ===
.claude/
CLAUDE.md
dist/
EOF

cd "$OUT"
git init -q -b main
git add -A
git commit -q -m "gnfetch v$VERSION"

echo "Public repo prepared at: $OUT  (branch: main, version: $VERSION)"
echo "Verify it carries no Claude tooling:"
echo "    git -C \"$OUT\" ls-files | grep -iE 'claude' || echo '  (clean)'"
echo "Then add the GitHub remote and push:"
echo "    git -C \"$OUT\" remote add origin git@github.com:Hibryda/gnfetch.git"
echo "    git -C \"$OUT\" push -u origin main"
