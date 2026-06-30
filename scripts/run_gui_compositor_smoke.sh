#!/usr/bin/env bash
set -euo pipefail

ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
cd "$ROOT"

if [[ -z "${WAYLAND_DISPLAY:-}" && -z "${DISPLAY:-}" ]]; then
  echo "datum-gui compositor smoke requires WAYLAND_DISPLAY or DISPLAY" >&2
  exit 2
fi

SCREENSHOT_OUT="${DATUM_GUI_SMOKE_SCREENSHOT:-/tmp/datum-gui-compositor-smoke.png}"
LOG_OUT="${DATUM_GUI_LOG:-/tmp/datum-gui-compositor-smoke.log}"
PROJECT_ROOT="${DATUM_GUI_SMOKE_PROJECT_ROOT:-crates/engine/testdata/golden/text/native/text-fidelity-repro}"

export DATUM_GUI_LOG="$LOG_OUT"
export DATUM_GUI_VERBOSE_LOG="${DATUM_GUI_VERBOSE_LOG:-1}"
export DATUM_TRACE_TIMING="${DATUM_TRACE_TIMING:-1}"

cargo run -p datum-gui-app --features visual --bin datum-gui -- \
  --project-root "$PROJECT_ROOT" \
  --window-size 1280x768 \
  --visual-scale-factor "${DATUM_GUI_SMOKE_SCALE:-1}" \
  --visual-test \
  --window-visual-test \
  --screenshot-out "$SCREENSHOT_OUT" \
  --exit-after-screenshot \
  --interaction-smoke \
  --resize-torture-smoke

cargo run -p datum-gui-app --features visual --bin datum-gui -- \
  --project-root "$PROJECT_ROOT" \
  --window-size 1280x768 \
  --visual-scale-factor "${DATUM_GUI_SMOKE_SCALE:-1}" \
  --kwin-lifecycle-smoke
