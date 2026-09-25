#!/usr/bin/env bash
set -uo pipefail
failed=0
trap 'rc=$?; failed=1; echo "REMAINDER_FAILURE status=$rc command=$BASH_COMMAND" >&2' ERR
repo_root=/home/bfadmin/Documents/datum-eda
cd "$repo_root"
cargo_guard=(python3 "$repo_root/scripts/run_cargo_guarded.py" --workload proof --)
# check_gui_conformance.py runs the composed-shell visual-parity gate
# (scripts/check_gui_visual_parity.py) as one of its aggregate gates: it captures
# the running app at the canonical command and fails on any regression from the
# owner-approved shell golden. It is invoked there (not a second time here) to
# avoid a duplicate cargo build+capture.
python3 scripts/check_gui_icon_assets.py
# Honest reference-capture gate: FAILS while the board-editor reference image
# docs/gui/reference/board-editor.png is missing or its *.PENDING placeholder
# exists. It is EXPECTED to be RED until the owner captures the reference on a
# machine with a working headless browser — that red is the honest signal that
# the full board-editor.html composition (split view, populated inspector) has no
# owner-approved reference yet and is gated on Phase-2, not a bug to silence.
python3 scripts/check_gui_reference_capture.py
# Decision-022 source health: hermetic checker regressions first, then the full
# tracked+untracked repository scan. CI supplies the trusted PR/push base SHA so
# legacy ceilings and policy can only ratchet downward; local runs still enforce
# current-tree discovery, normal budgets, exact ceilings, and logical includes.
python3 scripts/test_source_health_governance.py
if [[ -n "${SOURCE_HEALTH_BASE_REF:-}" ]]; then
  python3 scripts/check_source_health.py --base-ref "$SOURCE_HEALTH_BASE_REF"
else
  python3 scripts/check_source_health.py
fi
python3 scripts/check_menu_model.py
python3 scripts/menu_model_csv.py check
python3 scripts/check_erc_connectivity_parity.py
python3 scripts/check_pcb_layout_tool_matrix.py
python3 scripts/check_schematic_private_writers.py
python3 scripts/check_daemon_write_parity.py
python3 scripts/check_mcp_public_taxonomy.py
python3 -m unittest discover -s mcp-server -p 'test_context_revision_fence.py'
python3 -m unittest discover -s mcp-server -p 'test_agent_capability.py'
python3 -m unittest discover -s mcp-server -p 'test_agent_session_authority.py'
python3 -m unittest discover -s mcp-server -p 'test_workflow_catalog.py'
python3 scripts/test_agent_workflow_parity.py
python3 scripts/check_agent_workflow_parity.py
"${cargo_guard[@]}" cargo run -q -p datum-verb-registry --bin datum-verb-catalog -- --check
bash scripts/run_migration_proof_gates.sh
python3 scripts/check_cli_module_coverage.py
python3 mcp-server/server.py --self-test

exit "$failed"
