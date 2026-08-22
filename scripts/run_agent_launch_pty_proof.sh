#!/usr/bin/env bash
set -euo pipefail

repo_root="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
proof_target="${DATUM_AGENT_PROOF_TARGET:-/tmp/datum-agent-launch-proof-target}"

cd "$repo_root"
CARGO_TARGET_DIR="$proof_target" cargo build -p datum-eda-cli --locked --offline
DATUM_AGENT_CLI_PROOF_BIN="$proof_target/debug/datum-eda" \
  CARGO_TARGET_DIR="$proof_target" \
  cargo test -p datum-gui-app \
    terminal_agent_launch_tests::governed_agents_launch_through_owned_pty_with_context_intact \
    --locked --offline -- --ignored --nocapture
