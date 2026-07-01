#!/usr/bin/env bash
set -euo pipefail

python3 scripts/check_progress_coverage.py
python3 scripts/check_spec_parity.py
python3 scripts/check_alignment.py
python3 scripts/check_alignment.py --run-gates
python3 scripts/check_product_north_star.py
python3 scripts/check_library_foundation_contract.py
python3 scripts/check_spec_governance.py
python3 scripts/check_resolver_raw_loads.py
python3 scripts/check_gui_agent_terminal_convergence.py
python3 scripts/check_gui_design_tokens.py
python3 scripts/check_gui_icon_assets.py
python3 scripts/check_schematic_private_writers.py
python3 scripts/check_daemon_write_parity.py
python3 scripts/check_mcp_public_taxonomy.py
bash scripts/run_migration_proof_gates.sh
python3 scripts/check_file_size_budgets.py
python3 scripts/check_decomposition_coverage.py
python3 scripts/check_touched_monolith_growth.py
python3 scripts/check_test_file_sizes.py --max-lines 700
python3 mcp-server/server.py --self-test
