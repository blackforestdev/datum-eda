#!/usr/bin/env bash
set -euo pipefail

repo_root="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
proof_target="${DATUM_AGENT_PROOF_TARGET:-/tmp/datum-agent-launch-proof-target}"

cd "$repo_root"
CARGO_TARGET_DIR="$proof_target" cargo build -p datum-eda-cli --locked --offline
DATUM_AGENT_CLI_PROOF_BIN="$proof_target/debug/datum-eda" \
  CARGO_TARGET_DIR="$proof_target" \
  cargo test -p datum-gui-app \
    terminal_agent_launch_tests::governed_agents_complete_production_workflow_through_owned_pty \
    --locked --offline -- --ignored --nocapture

CARGO_TARGET_DIR="$proof_target" \
  cargo test -p datum-eda-cli commands::agent --locked --offline

PYTHONPATH="$repo_root/mcp-server" python3 -m unittest \
  mcp-server/test_agent_session_authority.py \
  mcp-server/test_context_revision_fence.py \
  mcp-server/test_stdio_broker.py
