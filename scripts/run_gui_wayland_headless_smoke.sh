#!/usr/bin/env bash
set -euo pipefail

ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
cd "$ROOT"

if ! command -v weston >/dev/null 2>&1; then
  echo "weston is required for the headless Wayland smoke test" >&2
  echo "install weston, then rerun: bash scripts/run_gui_wayland_headless_smoke.sh" >&2
  exit 127
fi

export XDG_RUNTIME_DIR="${XDG_RUNTIME_DIR:-/tmp/datum-wl}"
mkdir -p "$XDG_RUNTIME_DIR"
chmod 700 "$XDG_RUNTIME_DIR"

SOCKET="${DATUM_WAYLAND_SOCKET:-wayland-datum}"
WESTON_LOG="${DATUM_WESTON_LOG:-/tmp/datum-weston-headless.log}"
WIDTH="${DATUM_GUI_SMOKE_WIDTH:-1280}"
HEIGHT="${DATUM_GUI_SMOKE_HEIGHT:-768}"

weston \
  --backend=headless-backend.so \
  --socket="$SOCKET" \
  --width="$WIDTH" \
  --height="$HEIGHT" \
  --idle-time=0 \
  --log="$WESTON_LOG" &
WPID=$!

cleanup() {
  kill "$WPID" >/dev/null 2>&1 || true
  wait "$WPID" >/dev/null 2>&1 || true
}
trap cleanup EXIT

sleep "${DATUM_WESTON_STARTUP_DELAY:-1}"

export WAYLAND_DISPLAY="$SOCKET"
unset DISPLAY

if [[ -z "${VK_ICD_FILENAMES:-}" ]]; then
  if compgen -G "/usr/share/vulkan/icd.d/lvp_icd*.json" >/dev/null; then
    export VK_ICD_FILENAMES="$(ls /usr/share/vulkan/icd.d/lvp_icd*.json | head -1)"
  fi
fi
export LIBGL_ALWAYS_SOFTWARE="${LIBGL_ALWAYS_SOFTWARE:-1}"
export GALLIUM_DRIVER="${GALLIUM_DRIVER:-llvmpipe}"

bash scripts/run_gui_compositor_smoke.sh
