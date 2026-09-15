#!/usr/bin/env bash
set -euo pipefail

repo_root="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
cargo_guard=(python3 "$repo_root/scripts/run_cargo_guarded.py" --workload proof --)
cleanup_target=false
if [[ -n "${DATUM_AGENT_PROOF_TARGET:-}" ]]; then
  proof_target="$DATUM_AGENT_PROOF_TARGET"
else
  proof_root="$repo_root/target/proofs"
  mkdir -p "$proof_root"
  proof_target="$(mktemp -d "$proof_root/agent-launch.XXXXXX")"
  cleanup_target=true
fi

cleanup() {
  if [[ "$cleanup_target" == true ]]; then
    rm -rf "$proof_target"
  fi
}
trap cleanup EXIT

cd "$repo_root"
CARGO_TARGET_DIR="$proof_target" \
  "${cargo_guard[@]}" cargo build -p datum-eda-cli --locked --offline
DATUM_AGENT_CLI_PROOF_BIN="$proof_target/debug/datum-eda" \
  CARGO_TARGET_DIR="$proof_target" \
  "${cargo_guard[@]}" cargo test -p datum-gui-app \
    terminal_agent_launch_tests::governed_agents_complete_production_workflow_through_owned_pty \
    --locked --offline -- --ignored --nocapture

CARGO_TARGET_DIR="$proof_target" \
  "${cargo_guard[@]}" cargo test -p datum-eda-cli commands::agent --locked --offline

PYTHONPATH="$repo_root/mcp-server" python3 -m unittest \
  mcp-server/test_agent_session_authority.py \
  mcp-server/test_context_revision_fence.py \
  mcp-server/test_stdio_broker.py
