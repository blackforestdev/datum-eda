#!/usr/bin/env bash
# Capture an HTML file (or URL) to a PNG using headless Firefox.
#
# Why Firefox and not Chromium: in this sandboxed dev environment Chromium
# SIGTRAPs at startup — its multi-process/zygote sandbox needs the namespace and
# seccomp syscalls the outer sandbox blocks, even with --no-sandbox. Firefox/Gecko
# starts fine; the only catch is that a desktop Firefox already running locks the
# default profile, so we always spin up a throwaway isolated profile with
# --no-remote. (The wgpu app itself screenshots via its own --visual-test flag; this
# script is for HTML prototypes/artifacts under docs/gui/prototypes/.)
#
# Usage: scripts/shot_html.sh <file.html|url> [out.png] [WIDTHxHEIGHT]
#   scripts/shot_html.sh docs/gui/prototypes/schematic-editor.html shot.png 1680x1050
set -euo pipefail

src="${1:?usage: shot_html.sh <file.html|url> [out.png] [WIDTHxHEIGHT]}"
out="${2:-"$(basename "${src%.html}").png"}"
size="${3:-1680x1050}"
firefox_bin="${FIREFOX:-firefox}"

# Local path -> absolute file:// URL.
case "$src" in
  http://*|https://*|file://*) url="$src" ;;
  *) url="file://$(cd "$(dirname "$src")" && pwd)/$(basename "$src")" ;;
esac

command -v "$firefox_bin" >/dev/null \
  || { echo "shot_html: '$firefox_bin' not found (set FIREFOX=...)" >&2; exit 1; }

profile="$(mktemp -d "${TMPDIR:-/tmp}/ff-shot.XXXXXX")"
trap 'rm -rf "$profile"' EXIT

"$firefox_bin" --headless --no-remote --profile "$profile" \
  --window-size="${size/x/,}" --screenshot "$out" "$url" >/dev/null 2>&1 || true

if [ -s "$out" ]; then
  dim="$(command -v identify >/dev/null && identify -format '%wx%h' "$out" 2>/dev/null || echo "$(wc -c <"$out") bytes")"
  echo "shot_html: wrote $out ($dim)"
else
  echo "shot_html: FAILED to capture $url -> $out" >&2
  exit 1
fi
