#!/usr/bin/env bash
set -euo pipefail

repo_root="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
cargo_guard=(python3 "$repo_root/scripts/run_cargo_guarded.py" --workload proof --)
cd "$repo_root"

if [[ "$(uname -s)" != Linux ]]; then
  echo "DTC-P28 production witness requires Linux" >&2
  exit 1
fi

tool_root="${DATUM_P28_TOOL_ROOT:-}"
resolve_tool() {
  local name="$1"
  if [[ -n "$tool_root" && -x "$tool_root/usr/bin/$name" ]]; then
    printf '%s\n' "$tool_root/usr/bin/$name"
  else
    command -v "$name"
  fi
}

for tool in bash zsh fish ssh tmux less vim.tiny nvim htop btop python3 git cargo; do
  if ! resolve_tool "$tool" >/dev/null 2>&1; then
    echo "DTC-P28 missing required real-program witness: $tool" >&2
    echo "Install it as a host QA tool or set DATUM_P28_TOOL_ROOT to an extracted package root." >&2
    exit 1
  fi
done

python3 scripts/check_terminal_compatibility_matrix.py
revision="$(git rev-parse HEAD)"
evidence_dir="${DATUM_P28_EVIDENCE_DIR:-target/p28-evidence}"
if [[ "$evidence_dir" != /* ]]; then
  evidence_dir="$repo_root/$evidence_dir"
fi
mkdir -p "$evidence_dir"
evidence="$evidence_dir/${revision}-linux-x86_64.json"

DATUM_P28_TOOL_ROOT="$tool_root" \
DATUM_P28_REVISION="$revision" \
DATUM_P28_EVIDENCE="$evidence" \
"${cargo_guard[@]}" cargo test -p datum-gui-app --locked --offline \
  terminal_profile::compatibility_tests::production_pty_proves_named_shell_tui_and_tool_compatibility \
  -- --ignored --exact --nocapture --test-threads=1

python3 - "$evidence" <<'PY'
import json
import pathlib
import sys

path = pathlib.Path(sys.argv[1])
payload = json.loads(path.read_text(encoding="utf-8"))
names = {entry["name"] for entry in payload["results"] if entry["status"] == "passed"}
required = {"bash", "zsh", "fish", "ssh", "tmux", "less", "vim", "neovim",
            "htop", "btop", "python", "git", "cargo"}
if names != required or payload.get("failures"):
    raise SystemExit(f"DTC-P28 evidence incomplete: passed={sorted(names)} failures={payload.get('failures')}")
print(f"DTC-P28 production compatibility evidence verified: {path}")
PY
