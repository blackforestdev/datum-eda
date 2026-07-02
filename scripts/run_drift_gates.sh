#!/usr/bin/env bash
set -euo pipefail

python3 scripts/check_progress_coverage.py
python3 scripts/check_spec_parity.py
python3 scripts/check_alignment.py --run-gates
python3 scripts/check_spec_governance.py
python3 scripts/check_resolver_raw_loads.py
python3 scripts/check_gui_agent_terminal_convergence.py
python3 scripts/check_gui_design_tokens.py
python3 scripts/check_gui_icon_assets.py
python3 scripts/check_schematic_private_writers.py
python3 scripts/check_daemon_write_parity.py
python3 scripts/check_mcp_public_taxonomy.py
cargo run -q -p datum-verb-registry --bin datum-verb-catalog -- --check
bash scripts/run_migration_proof_gates.sh
python3 mcp-server/server.py --self-test
