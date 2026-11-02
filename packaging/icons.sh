#!/usr/bin/env bash
# Regenerate every app icon from assets/app-icon/localpaint.svg (the canonical master: the
# project owner's source SVG, on a 1024-unit canvas).
#
# Needs: resvg (brew install resvg / cargo install resvg); the .ico is packed by `cargo xtask ico`.
# On macOS, iconutil also writes the .icns. The outputs are committed, so packaging never needs
# these tools.
#
#   packaging/icons.sh
set -euo pipefail
ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
DIR="$ROOT/assets/app-icon"
SVG="$DIR/localpaint.svg"
TMP="$(mktemp -d)"
trap 'rm -rf "$TMP"' EXIT

command -v resvg >/dev/null || { echo "error: resvg not found (brew install resvg)" >&2; exit 1; }

# The master is a full-bleed rounded tile inset 64 units of 1024 (rx=200), which already leaves
# close to Apple's margin, so the macOS renders use it as it is. Windows and Linux use the whole
# tile, as the other Local apps' icons do.
grep -q 'viewBox="0 0 1024 1024"' "$SVG" || { echo "error: expected viewBox=\"0 0 1024 1024\" in $SVG" >&2; exit 1; }

render() { resvg -w "$2" -h "$2" "$1" "$3" </dev/null; }

# 1024 px PNG (also the runtime Dock icon on macOS, see apps/localpaint/src/app_icon.rs).
render "$SVG" 1024 "$DIR/localpaint-1024.png"

# Linux hicolor theme.
for s in 16 24 32 48 64 128 256 512; do
  mkdir -p "$DIR/hicolor/${s}x${s}/apps"
  render "$SVG" "$s" "$DIR/hicolor/${s}x${s}/apps/com.localpaint.app.png"
done
mkdir -p "$DIR/hicolor/scalable/apps"
# localpaint-small.svg is the same master under a name the READMEs can link; the scalable theme
# icon is that copy.
cp "$DIR/localpaint-small.svg" "$DIR/hicolor/scalable/apps/com.localpaint.app.svg"

# Windows .ico.
ICO_PNGS=()
for s in 16 20 24 32 40 48 64 128 256; do
  render "$SVG" "$s" "$TMP/ico-$s.png"
  ICO_PNGS+=("$TMP/ico-$s.png")
done
(cd "$ROOT" && cargo run -q -p xtask -- ico "$DIR/localpaint.ico" "${ICO_PNGS[@]}")

# macOS .icns.
if command -v iconutil >/dev/null; then
  SET="$TMP/localpaint.iconset"
  mkdir -p "$SET"
  for s in 16 32 128 256 512; do
    render "$SVG" "$s" "$SET/icon_${s}x${s}.png"
    render "$SVG" $((s * 2)) "$SET/icon_${s}x${s}@2x.png"
  done
  iconutil -c icns -o "$DIR/localpaint.icns" "$SET"
else
  echo "warning: iconutil not found (macOS only); localpaint.icns not regenerated" >&2
fi
echo "icons written to $DIR"
