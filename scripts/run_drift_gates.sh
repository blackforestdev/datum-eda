#!/usr/bin/env bash
set -euo pipefail

python3 scripts/check_progress_coverage.py
python3 scripts/check_spec_parity.py
python3 scripts/check_alignment.py --run-gates
python3 scripts/check_spec_governance.py
python3 scripts/check_resolver_raw_loads.py
python3 scripts/check_gui_agent_terminal_convergence.py
python3 scripts/check_gui_design_tokens.py
python3 scripts/check_gui_conformance.py
# check_gui_conformance.py runs the composed-shell visual-parity gate
# (scripts/check_gui_visual_parity.py) as one of its aggregate gates: it captures
# the running app at the canonical command and fails on any regression from the
# owner-approved shell golden. It is invoked there (not a second time here) to
# avoid a duplicate cargo build+capture.
python3 scripts/check_gui_icon_assets.py
# Source-module size governance flag: FLAGS oversized modules as
# decomposition-pending (governance-triggered/organic decomp, never scheduled),
# fails on NEW unregistered oversized modules or ledger drift. Does not decompose.
python3 scripts/check_source_module_size.py
python3 scripts/check_menu_model.py
python3 scripts/menu_model_csv.py check
python3 scripts/check_erc_connectivity_parity.py
python3 scripts/check_pcb_layout_tool_matrix.py
python3 scripts/check_schematic_private_writers.py
python3 scripts/check_daemon_write_parity.py
python3 scripts/check_mcp_public_taxonomy.py
cargo run -q -p datum-verb-registry --bin datum-verb-catalog -- --check
bash scripts/run_migration_proof_gates.sh
python3 scripts/check_cli_module_coverage.py
python3 mcp-server/server.py --self-test
