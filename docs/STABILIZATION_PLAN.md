# Stabilization Plan (Structural Debt Retirement)

Date: 2026-03-26
Scope: eliminate monolithic runtime files and large inline test blocks without
changing runtime behavior.

## Objectives

- Keep development velocity while reducing structural failure risk.
- Move test code out of production entrypoint files.
- Enforce CI guardrails so this pattern cannot regress.

## Workstreams

### 1) Guardrail Closure (First)

Status: Completed

- Expand test size gates to cover Rust dedicated test modules and MCP tests.
- Add inline Rust test-tail limits (`#[cfg(test)]` to EOF).
- Freeze known legacy inline-test hotspots with per-file ceilings so debt
  cannot grow while decomposition is underway.
- Keep all checks in `scripts/run_drift_gates.sh`.
- Added non-growth freezes for current large inline tails in:
  - `crates/engine/src/import/eagle/mod.rs`
  - `crates/cli/src/main.rs`
- Retired inline-test bulk from `crates/engine/src/import/eagle/mod.rs`
  into `crates/engine/src/import/eagle/tests/mod_tests_import_eagle.rs`
  and tightened freeze from `262` → `39`.
- Moved CLI test support wiring from `crates/cli/src/main.rs`
  into `crates/cli/src/main_tests.rs` and tightened freeze from `169` → `2`.

Exit criteria:
- CI fails on any new oversized test module.
- CI fails if a known hotspot inline-test tail grows.

### 2) High-Risk Inline-Test Extraction (Priority Order)

Status: Completed

1. `crates/engine-daemon/src/main.rs` ✅ completed (2026-03-26)
2. `crates/engine/src/erc/mod.rs` ✅ completed (2026-03-26)
3. `crates/engine/src/board/mod.rs` ✅ completed (2026-03-26)
4. `crates/engine/src/connectivity/mod.rs` ✅ completed (2026-03-26)
5. `crates/engine/src/import/kicad/mod.rs` ✅ completed (2026-03-26)
6. `crates/engine/src/drc/mod.rs` ✅ completed (2026-03-26)

Method:
- Keep runtime behavior unchanged.
- Convert each file to a thin runtime/module hub.
- Move tests into focused sibling shards.
- Run full drift gates after each extraction.

Exit criteria:
- No high-risk inline-test monolith remains.
- Each new test shard stays within test size budgets.

### 3) Production Monolith Decomposition

Status: Completed

Targets:
- `crates/engine/src/api/write_ops.rs` ✅ completed (2026-03-26)
- `crates/engine/src/import/kicad/mod.rs` ✅ completed (2026-03-26)
  - ✅ follow-up decomposition (2026-03-26):
    `import/kicad/mod.rs` (681 → 231) +
    `import/kicad/skeleton.rs` (461), preserving import/report surface behavior
- `crates/engine/src/import/eagle/mod.rs` ✅ completed (2026-03-26)
- Follow-up bounded split:
  `crates/engine/src/import/eagle/mod.rs` (518 → 85) +
  `crates/engine/src/import/eagle/parser.rs` (444) ✅ completed (2026-03-26)

Method:
- Split by operation/parser domain.
- Preserve public API surface and behavior.
- Do not mix functional changes with structural changes.

Exit criteria:
- Runtime entrypoints remain thin.
- Large modules split into stable, bounded units.

### 4) Prevention and Policy Hardening

Status: Completed

- Keep and tighten line budgets over time (only downward movement).
- Require test modules in dedicated files for new feature work.
- Disallow large inline test blocks in runtime-critical files.
- Keep PR review checklist aligned with these rules.
- Added explicit source budgets for decomposed production modules:
  - `crates/engine/src/api/write_ops.rs`
  - `crates/engine/src/import/kicad/*`
  - `crates/engine/src/import/eagle/*`

Exit criteria:
- Guardrails enforce structure automatically.
- No manual policing required to prevent recurrence.

### 5) Phase 2: Audit-Discovered Hotspots (Repo-Wide Closure)

Status: In Progress

Scope expansion rule:
- Every monolithic or inline-test hotspot discovered by audit is tracked here
  until decomposed or intentionally accepted with explicit rationale.

Priority order (from current audit):
1. `crates/engine/src/schematic/mod.rs` ✅ completed (2026-03-26; inline tail 217 → 7)
2. `crates/engine/src/pool/mod.rs` ✅ completed (2026-03-26; inline tail 199 → 3)
3. `crates/engine/src/import/mod.rs` ✅ completed (2026-03-26; inline tail 105 → 3)
4. `crates/engine/src/import/ids_sidecar.rs` ✅ completed (2026-03-26; inline tail 92 → 3)
5. `crates/engine/src/api/mod.rs` ✅ completed (2026-03-26; total 838 → 574 pre-test 549)
   - ✅ follow-up decomposition (2026-03-26):
     `api/mod.rs` (574 → 200) +
     `api/api_types.rs` (383), preserving `eda_engine::api::*` via re-export
6. `crates/test-harness/src/bin/m3_write_surface_parity.rs` ✅ completed (2026-03-26; total 2474 split into parity + engine/daemon/mcp/cli/common shards)
7. `crates/engine/src/api/project_surface.rs` ✅ completed (2026-03-26; total 736 split into `project_surface.rs` + `project_surface/project_surface_replacements.rs`)
8. `crates/engine/src/api/save_kicad.rs` ✅ completed (2026-03-26; total 720 split into `save_kicad.rs` + `save_kicad/transaction_state.rs` + `save_kicad/kicad_text.rs`)
9. `crates/engine/src/api/write_ops/undo_redo.rs` ✅ completed (2026-03-26; total 722 split into `write_ops/undo_redo.rs` + `write_ops/undo_redo/undo.rs` + `write_ops/undo_redo/redo.rs`)
10. `crates/engine/src/api/write_ops.rs` ✅ completed (2026-03-26; total 661 split into `write_ops.rs` + `write_ops/basic_mutations.rs` + `write_ops/assign_package_rule.rs`)

Secondary queue (continue if still high after phase-2 primary):
- Large dedicated test shards near current cap in `crates/engine-daemon/src/tests/`
  - ✅ `main_tests_component_mutation_core.rs` split into:
    `main_tests_component_mutation_core.rs` (550 → 6) +
    `main_tests_component_mutation_core/main_tests_component_mutation_core_updates.rs` (281) +
    `main_tests_component_mutation_core/main_tests_component_mutation_core_assign_part.rs` (270) on 2026-03-26
  - ✅ `main_tests_query_check.rs` split into:
    `main_tests_query_check.rs` (533 → 6) +
    `main_tests_query_check/main_tests_query_check_board.rs` (92) +
    `main_tests_query_check/main_tests_query_check_schematic.rs` (442) on 2026-03-26
  - ✅ `main_tests_session_pool.rs` split into:
    `main_tests_session_pool.rs` (693 → 295) +
    `main_tests_session_pool_replacements.rs` (399) on 2026-03-26
  - ✅ `main_tests_package_replacements.rs` split into:
    `main_tests_package_replacements.rs` (685 → 340) +
    `main_tests_package_replacements_apply.rs` (346) on 2026-03-26
  - ✅ `main_tests_query_check.rs` split into:
    `main_tests_query_check.rs` (666 → 533) +
    `main_tests_query_check_runs.rs` (134) on 2026-03-26
- Large dedicated CLI test shards in `crates/cli/src/main_tests_*`
  - ✅ `main_tests_modify_advanced.rs` split into:
    `main_tests_modify_advanced.rs` (604 → 339) +
    `main_tests_modify_advanced_plan.rs` (266) on 2026-03-26
  - ✅ `main_tests_modify_basic.rs` split into:
    `main_tests_modify_basic.rs` (574 → 6) +
    `main_tests_modify_basic/main_tests_modify_basic_core.rs` (307) +
    `main_tests_modify_basic/main_tests_modify_basic_pool.rs` (268) on 2026-03-26
- Large CLI modify command surface in `crates/cli/src/`
  - ✅ `command_modify.rs` split into:
    `command_modify/mod.rs` (12) +
    `command_modify/modify_ops.rs` (307) +
    `command_modify/parse_args.rs` (211) on 2026-03-26
- Large dedicated ERC test shards in `crates/engine/src/erc/tests/`
  - ✅ `mod_tests_prechecks_rules.rs` split into:
    `mod_tests_prechecks_rules.rs` (604 → 347) +
    `mod_tests_prechecks_rules_edge.rs` (267) on 2026-03-26
  - ✅ `mod_tests_prechecks_core.rs` split into:
    `mod_tests_prechecks_core.rs` (571 → 330) +
    `mod_tests_prechecks_core_waivers.rs` (256) on 2026-03-26
- Large dedicated API write-op test shard in `crates/engine/src/api/tests/`
  - ✅ `write_ops.rs` split into:
    `write_ops.rs` (1511 → 5 module lines) +
    `write_ops/write_ops_smoke_and_delete.rs` +
    `write_ops/write_ops_value_reference_rotate.rs` +
    `write_ops/write_ops_assign_part.rs` +
    `write_ops/write_ops_package_replace.rs` +
    `write_ops/write_ops_netclass_move.rs` on 2026-03-26
- MCP support test-fixture monolith in `mcp-server/`
  - ✅ `test_support.py` split into:
    `test_support.py` (670 → 16) +
    `fake_daemon_support_base.py` +
    `fake_daemon_support_mutations.py` +
    `fake_daemon_support_queries.py` +
    `fake_daemon_support_replacements.py` on 2026-03-26
- MCP dispatch core test monolith in `mcp-server/`
  - ✅ `test_dispatch_core.py` split into:
    `test_dispatch_core.py` (553 → 11) +
    `dispatch_core_project_write_tests.py` +
    `dispatch_core_replacement_tests.py` on 2026-03-26
- MCP daemon client test monolith in `mcp-server/`
  - ✅ `test_daemon_client.py` split into:
    `test_daemon_client.py` (549 → 11) +
    `daemon_client_request_tests.py` +
    `daemon_client_transport_tests.py` on 2026-03-26
- MCP read-surface dispatch test monolith in `mcp-server/`
  - ✅ `test_dispatch_read_surface.py` split into:
    `test_dispatch_read_surface.py` (505 → 13) +
    `dispatch_read_surface_pool_candidates_tests.py` +
    `dispatch_read_surface_query_tests.py` on 2026-03-26
- MCP write-basics dispatch test monolith in `mcp-server/`
  - ✅ `test_dispatch_write_basics.py` split into:
    `test_dispatch_write_basics.py` (490 → 11) +
    `dispatch_write_basics_command_tests.py` +
    `dispatch_write_basics_stateful_tests.py` on 2026-03-26
- Rust API read-surface test monolith in `crates/engine/src/api/tests/`
  - ✅ `read_surface.rs` split into:
    `read_surface.rs` (889 → 3) +
    `read_surface/read_surface_summary_and_symbols.rs` +
    `read_surface/read_surface_import_and_checks.rs` +
    `read_surface/read_surface_pool_and_replacements.rs` on 2026-03-26
  - ✅ Retired all temporary oversized Rust test-file freezes in
    `scripts/check_test_file_sizes.py` (now `0` freeze entries) on 2026-03-26
- Test-harness CLI write-surface monolith in `crates/test-harness/src/bin/`
  - ✅ `m3_write_surface_cli.rs` split into:
    `m3_write_surface_cli.rs` (1197 → 65) +
    `m3_write_surface_cli/m3_write_surface_cli_core.rs` +
    `m3_write_surface_cli/m3_write_surface_cli_assign_package.rs` +
    `m3_write_surface_cli/m3_write_surface_cli_motion_netclass.rs` on 2026-03-26
  - ✅ `m3_write_surface_cli/m3_write_surface_cli_assign_package.rs` split into:
    `m3_write_surface_cli/m3_write_surface_cli_assign_package/mod.rs` (12) +
    `m3_write_surface_cli/m3_write_surface_cli_assign_package/assign_part.rs` (203) +
    `m3_write_surface_cli/m3_write_surface_cli_assign_package/set_package.rs` (321) on 2026-03-26
  - ✅ Parity binary validation rerun:
    `cargo run -q -p eda-test-harness --bin m3_write_surface_parity -- --json --allow-deferred`
    returned `overall_status=passed` on 2026-03-26
- Test-harness M2 perf monolith in `crates/test-harness/src/bin/`
  - ✅ `m2_perf.rs` split into:
    `m2_perf.rs` (500 → 206) +
    `m2_perf_helpers.rs` (306) on 2026-03-26
  - ✅ Validation rerun:
    `cargo check -q -p eda-test-harness --bin m2_perf` and
    `cargo test -q -p eda-test-harness --bin m2_perf` passed on 2026-03-26
- DRC module decomposition in `crates/engine/src/drc/`
  - ✅ `drc/mod.rs` split into:
    `drc/mod.rs` (524 → 144) +
    `drc/checks/mod.rs` (391) on 2026-03-26
  - ✅ Validation rerun:
    `cargo test -q -p eda-engine` and `scripts/run_drift_gates.sh` passed on 2026-03-26
- Test-harness engine write-surface monolith in `crates/test-harness/src/bin/`
  - ✅ `m3_write_surface_engine.rs` split into:
    `m3_write_surface_engine.rs` (558 → 23) +
    `m3_write_surface_engine/m3_write_surface_engine_basic.rs` +
    `m3_write_surface_engine/m3_write_surface_engine_replacements.rs` on 2026-03-26
  - ✅ Parity binary validation rerun:
    `cargo run -q -p eda-test-harness --bin m3_write_surface_parity -- --json --allow-deferred`
    returned `overall_status=passed` on 2026-03-26

Method:
- Structural-only changes; no behavior deltas in same changeset.
- One hotspot per changeset.
- Full drift-gate pass after each hotspot.
- Tighten freeze/budget values downward immediately after each extraction.

Exit criteria:
- Primary hotspot queue fully decomposed into bounded modules/test shards.
- No inline test tails above working thresholds in runtime-critical files.
- Remaining large files are either split or explicitly budgeted with written rationale.
- Plan is retired after closure and guardrails become the only enforcement mechanism.

Execution note (2026-03-26):
- During item 6 execution, the `m3_write_surface_cli` shard was temporarily missing due to an interrupted/partial structural extraction sequence.
- This was not a roadmap or feature-scope change; it was refactor completion drift within the approved stabilization scope.
- Corrective action completed in the same stabilization pass: shard restored, entrypoint thinned, and full drift gates re-run green.

## Change Control Rules During Stabilization

- One hotspot per change-set.
- Structural refactor only (no feature deltas in same change-set).
- Full gate pass required after each step.
- If a gate requires temporary allowance, it must be explicit and non-growing.
