# M1 query and M2 closeout evidence

> Detailed evidence relocated from `specs/PROGRESS.md`. The tracker remains the implementation-status entry point; the generated Active Frontier alone selects work. Dated history below does not grant execution or establish a fresh PASS.

### PROGRAM_SPEC.md — M1 Query API

| Method | Engine | Daemon | MCP | CLI |
|--------|--------|--------|-----|-----|
| get_netlist | [x] | [x] | [x] | [x] |
| get_components | [x] | [x] | [x] | [x] |
| get_nets / get_net_info | [x] | [x] | [x] | [x] |
| get_schematic_net_info | [x] | [x] | [x] | [x] |
| get_board_summary | [x] | [x] | [x] | [x] |
| get_schematic_summary | [x] | [x] | [x] | [x] |
| get_sheets | [x] | [x] | [x] | [x] |
| get_symbols | [x] | [x] | [x] | [x] |
| get_ports | [x] | [x] | [x] | [x] |
| get_labels | [x] | [x] | [x] | [x] |
| get_buses | [x] | [x] | [x] | [x] |
| get_bus_entries | [x] | [x] | [x] | [x] |
| get_noconnects | [x] | [x] | [x] | [x] |
| get_hierarchy | [x] | [x] | [x] | [x] |
| get_connectivity_diagnostics | [x] | [x] | [x] | [x] |
| get_unrouted | [x] | [x] | [x] | [x] |
| search_pool | [x] | [x] | [x] | [x] |

**M1 overall**: [~] Query-surface parity is now complete across engine/daemon/MCP/CLI for the current imported-design read slice; import fidelity, corpus, and broader golden coverage are still open

M1 imported-query reliability note (2026-03-30):
- Checked-in CLI query goldens now cover the current imported-design read
  surfaces for `simple-demo.kicad_sch`, `bus-demo.kicad_sch`,
  `hierarchy-mismatch-demo.kicad_sch`, `erc-coverage-demo.kicad_sch`,
  `simple-demo.kicad_pcb`, and `airwire-demo.kicad_pcb`:
  - `summary`
  - `netlist`
  - `nets`
  - `schematic-nets`
  - `sheets`
  - `symbols`
  - `buses`
  - `bus-entries`
  - `noconnects`
  - `labels`
  - `ports`
  - `hierarchy`
  - `diagnostics`
  - `unrouted`
- Those fixtures live under `crates/cli/testdata/golden/query` and are enforced
  by `main_tests_query_goldens`, with `UPDATE_GOLDENS=1` as the explicit
  regeneration path.
- Checked-in CLI check goldens now cover the imported schematic fixtures already
  used by the current M1/M2 corpus:
  - `simple-demo.kicad_sch`
  - `bus-demo.kicad_sch`
  - `analog-input-demo.kicad_sch`
  - `analog-input-bias-demo.kicad_sch`
  - `erc-coverage-demo.kicad_sch`
  - `hierarchy-mismatch-demo.kicad_sch`
- Those fixtures live under `crates/cli/testdata/golden/check` and are enforced
  by `main_tests_check_goldens`, again with `UPDATE_GOLDENS=1` for intentional
  regeneration.
- Imported KiCad schematic hierarchy now follows the current supported child-sheet
  subset:
  - `simple-demo.kicad_sch` now resolves its checked-in `sub.kicad_sch`
    child file during import
  - hierarchical net propagation now merges the parent sheet pin and
    hierarchical label with the child-sheet hierarchical label target
  - existing query/check surfaces now expose the resulting extra child sheet,
    hierarchy link, and propagated `SUB_IN` pin attachment without adding new
    endpoints
  - resolver diagnostics now report missing or multiply-mapped hierarchical
    child targets deterministically when the imported subset cannot link them
- Imported KiCad bus/member expansion now covers the current supported subset:
  - scalar member labels like `DATA[0]` resolve to deterministic scalar net
    names like `DATA0` on existing net/query/check surfaces
  - simple bus-range labels like `DATA[0..1]` populate imported bus members
  - imported `bus_entry` forms now associate to one bus and one wire when the
    current geometric subset is unambiguous
  - the checked-in `bus-demo.kicad_sch` fixture plus CLI goldens now exercise
    `schematic-nets`, `buses`, `bus-entries`, and `diagnostics` for this path
- The current KiCad import/query corpus now has a repo-native determinism gate:
  - manifest: `crates/test-harness/testdata/quality/import_query_determinism_manifest_v1.json`
  - gate: `python3 scripts/check_import_query_determinism.py`
  - CI: `.github/workflows/alignment.yml`
  - scope: repeated `query` and `check` runs across the checked-in KiCad board
    and schematic fixture corpus used by the current M1 goldens
  - current result: 36/36 cases stable across 3 repeated runs for the current
    supported subset

---

### M2 Pre-Freeze Closeout Checklist (Ordered)

This is the execution order to close meaningful M2 gaps before full integrated
program specification freeze.

| Order | Work Item | Exit Condition | Status |
|------:|-----------|----------------|--------|
| 1 | DRC foundation + first runnable checks | `run_drc` returns structured violations for at least connectivity + clearance on fixture boards | [x] |
| 2 | CLI DRC integration | `tool drc <board.kicad_pcb>` works with pass/fail exit behavior and JSON output | [x] |
| 3 | MCP/daemon DRC path | MCP `run_drc` round-trips through daemon with stable payload shape | [x] |
| 4 | Remaining high-value MCP query parity | `get_sheets`, `get_netlist`, `get_design_rules` implemented or explicitly deferred with gate note | [x] |
| 5 | ERC corpus hardening | Corpus-backed ERC goldens exist and cover required M2 codes in current implementation slice | [x] |
| 6 | DRC corpus hardening | Corpus-backed DRC goldens exist for implemented DRC checks | [x] |
| 7 | Performance gate harness | Reproducible timing harness for ERC/DRC on DOA2526 with recorded baseline | [x] |
| 8 | M2 gate reconciliation pass | `PROGRAM_SPEC.md`, `MCP_API_SPEC.md`, `ENGINE_SPEC.md`, `PLAN.md`, `PROGRESS.md` mutually consistent for remaining open items | [x] |

Item 4 defer notes (2026-03-25):
- `get_sheets`: implemented in engine/daemon/MCP.
- `get_design_rules`: implemented in engine/daemon/MCP using the current rule
  evaluator subset payload.
- `get_netlist`: implemented in engine/daemon/MCP using a canonical net
  inventory payload with board-vs-schematic field parity.

Item 5 current artifacts (2026-03-25):
- ERC golden fixtures added:
  - `crates/engine/testdata/golden/erc/simple-demo.kicad_sch.json`
  - `crates/engine/testdata/golden/erc/analog-input-demo.kicad_sch.json`
  - `crates/engine/testdata/golden/erc/analog-input-bias-demo.kicad_sch.json`
  - `crates/engine/testdata/golden/erc/erc-coverage-demo.kicad_sch.json`
  - `crates/engine/testdata/golden/erc/hierarchy-mismatch-demo.kicad_sch.json`
- Golden validation tests:
  - `api::tests::erc_golden_simple_demo_matches_checked_in_fixture`
  - `api::tests::erc_golden_analog_input_demo_matches_checked_in_fixture`
  - `api::tests::erc_golden_analog_input_bias_demo_matches_checked_in_fixture`
  - `api::tests::erc_golden_coverage_demo_matches_checked_in_fixture`
  - `api::tests::erc_golden_hierarchy_mismatch_demo_matches_checked_in_fixture`
  - `api::tests::erc_golden_corpus_covers_required_m2_codes_for_current_implementation_slice`
- Golden contract is normalized to stable semantic fields (code/severity/message/
  net/component/pin/objects/waived) to avoid false churn from volatile UUIDs.
- Added implementation slice coverage for required ERC codes:
  - `output_to_output_conflict`
  - `undriven_input_pin`
  - `input_without_explicit_driver`
  - `power_in_without_source`
  - `unconnected_component_pin`
  - `undriven_power_net`
  - `noconnect_connected`
  - `hierarchical_connectivity_mismatch`
- Note: current implementation is sheet-level interface name consistency. Full
  instance-aware hierarchical net resolution remains tracked under schematic
  connectivity maturity (M1/M3+), not as an open M2 rule omission.

Item 6 current artifacts (2026-03-25):
- DRC golden fixtures added:
  - `crates/engine/testdata/golden/drc/simple-demo.kicad_pcb.json`
  - `crates/engine/testdata/golden/drc/partial-route-demo.kicad_pcb.json`
  - `crates/engine/testdata/golden/drc/airwire-demo.kicad_pcb.json`
  - `crates/engine/testdata/golden/drc/clearance-violation-demo.kicad_pcb.json`
  - `crates/engine/testdata/golden/drc/drc-coverage-demo.kicad_pcb.json`
  - `crates/engine/testdata/golden/drc/silk-clearance-demo.kicad_pcb.json`
- Golden validation tests:
  - `api::tests::drc_golden_simple_demo_matches_checked_in_fixture`
  - `api::tests::drc_golden_partial_route_demo_matches_checked_in_fixture`
  - `api::tests::drc_golden_airwire_demo_matches_checked_in_fixture`
  - `api::tests::drc_golden_clearance_violation_demo_matches_checked_in_fixture`
  - `api::tests::drc_golden_coverage_demo_matches_checked_in_fixture`
  - `api::tests::drc_golden_silk_clearance_demo_matches_checked_in_fixture`
- Golden contract is normalized to stable semantic fields (pass/fail, summary,
  code/rule/severity/message/location/objects) and excludes volatile violation IDs.
- Coverage for implemented DRC checks now includes:
  - connectivity (`connectivity_unrouted_net`)
  - clearance (`clearance_copper`)
  - connectivity unconnected pin (`connectivity_unconnected_pin`)
  - track width (`track_width_below_min`)
  - via hole (`via_hole_out_of_range`)
  - via annular ring (`via_annular_below_min`)
  - silk clearance (`silk_clearance_copper`)
- Remaining gap: broaden corpus and validation metrics against external reference
  behavior for quality-rate gates.
- Corpus-size gate is now met by checked-in golden fixtures:
  - total unique designs: 10
  - DRC additions in this pass include `airwire-demo.kicad_pcb`.
  - ERC additions in this pass include `analog-input-bias-demo.kicad_sch`.

Item 7 current artifacts (2026-03-25):
- Harness binary:
  - `crates/test-harness/src/bin/m2_perf.rs`
- Baseline artifact:
  - `crates/test-harness/testdata/perf/m2_doa2526_baseline.json`
- Commands:
  - Baseline write:
    - `cargo run -p eda-test-harness --bin m2_perf -- --iterations 3 --write-baseline crates/test-harness/testdata/perf/m2_doa2526_baseline.json`
  - Baseline compare:
    - `cargo run -p eda-test-harness --bin m2_perf -- --iterations 3 --compare-baseline crates/test-harness/testdata/perf/m2_doa2526_baseline.json`
- Current measured medians:
  - Baseline run: import_board=1200ms, import_schematic=1759ms, ERC=2ms, DRC=18ms
  - Compare run: import_board=1285ms, import_schematic=1838ms, ERC=2ms, DRC=20ms

Item 9 quality-rate gate artifacts (2026-03-25):
- Harness binary:
  - `crates/test-harness/src/bin/m2_quality.rs`
- Manifest artifact:
  - `crates/test-harness/testdata/quality/m2_quality_manifest.json`
- Command:
  - `cargo run -p eda-test-harness --bin m2_quality -- --json`
- Current measured output:
  - `erc false_positive_rate_pct=0.0`
  - `erc false_negative_rate_pct=0.0`
  - `drc false_positive_rate_pct=0.0`
  - `drc false_negative_rate_pct=0.0`
  - `unique_designs=11`, `required_min_designs=10`
  - `pass=true`

Item 8 reconciliation notes (2026-03-25):
- Performance harness status synced across:
  - `specs/PROGRESS.md` M2 requirement table
  - `specs/PROGRESS.md` closeout checklist
  - `PLAN.md` M2 progress section
- Deferred MCP query language remains aligned across:
  - `specs/PROGRAM_SPEC.md`
  - `specs/MCP_API_SPEC.md`
- Stale wording corrected:
  - `Waiver matching in DRC` now states DRC exists and waiver matching is the remaining gap.

Checking follow-up note (2026-03-30):
- Native-project authored waivers are now honored by the existing `project query erc`
  and `project query check` load path.
- DRC-domain waiver matching is now honored inside `run_drc`; waived DRC
  violations remain visible and waived-only DRC runs do not fail.
- Native authored board state now exposes the same waiver-aware DRC report through
  `project query <dir> drc`.
- Native `project query <dir> check` now returns a combined report with distinct
  `erc` and `drc` sections while leaving `project query <dir> board-check`
  unchanged.
- Native-project structural validation is now exposed through
  `project validate <dir>`, covering required native files, schema-version
  compatibility, duplicate UUID consistency within authored object types, and
  non-dangling persisted schematic/board references.
- The same native-project validation contract is now available through MCP as
  `validate_project`, preserving the CLI report shape and valid/invalid result
  semantics.
- MCP tool registration now also uses one shared spec table for `tools/list`
  export and `tools/call` dispatch, with parity tests checking runtime and
  fake-daemon coverage for each registered tool.
- Checked-in native fixture projects are now exercised continuously through
  `scripts/check_native_project_fixtures.py`, driven by
  `crates/test-harness/testdata/quality/native_project_validation_manifest_v1.json`
  and the existing `project validate` contract in CI.
- That manifest now covers both real route-strategy native fixtures and a
  dedicated checked-in invalid-case suite for duplicate UUID, missing sheet,
  and unsupported schema-version failures, and it currently exhausts all
  checked-in native project roots under `crates/test-harness/testdata/quality`.

### 2026-03-25 Contract Alignment Pass

| Item | Status | Notes |
|------|--------|-------|
| Integrated spec scaffold added | [x] | `specs/INTEGRATED_PROGRAM_SPEC.md` |
| Integrated contract verification matrix added | [x] | `specs/INTEGRATED_PROGRAM_SPEC.md` §8 |
| Integrated M3/M4 boundary contracts drafted | [x] | `specs/INTEGRATED_PROGRAM_SPEC.md` §§9-11 |
| Integrated M3/M4 acceptance tables drafted | [x] | `specs/INTEGRATED_PROGRAM_SPEC.md` §§12-13 |
| M3 determinism evidence hook behavioral | [x] | `crates/test-harness/src/bin/m3_op_determinism.rs` and `crates/test-harness/src/bin/m3_replacement_op_determinism.rs` together pass for the full current save-backed M3 mutation slice: `move_component`, `delete_track`, `delete_via`, `delete_component`, `rotate_component`, `set_value`, `set_reference`, `set_design_rule`, `assign_part`, `set_package`, `set_package_with_part`, `replace_component`, `replace_components`, `apply_component_replacement_plan`, `apply_component_replacement_policy`, `apply_scoped_component_replacement_policy`, `apply_scoped_component_replacement_plan`, and `set_net_class` |
| M3 undo/redo evidence hook behavioral | [x] | `crates/test-harness/src/bin/m3_undo_redo_roundtrip.rs` passes for current `delete_track`, `delete_via`, `delete_component`, `move_component`, `rotate_component`, `set_value`, `set_reference`, `set_design_rule`, `assign_part`, `set_package`, and `set_net_class` undo/redo slice |
| M3 replacement undo/redo evidence hook behavioral | [x] | `crates/test-harness/src/bin/m3_replacement_undo_redo_roundtrip.rs` passes for current `set_package_with_part`, `replace_component`, `replace_components`, `apply_component_replacement_plan`, `apply_component_replacement_policy`, `apply_scoped_component_replacement_policy`, and `apply_scoped_component_replacement_plan` undo/redo slice |
| M3 board round-trip fidelity hook behavioral | [x] | `crates/test-harness/src/bin/m3_board_roundtrip_fidelity.rs` passes for unmodified KiCad-board identity plus current `delete_track`, `delete_via`, `delete_component`, `move_component`, `rotate_component`, `set_value`, and `set_reference` save→reimport→save artifact stability |
| M3 sidecar round-trip fidelity hook behavioral | [x] | `crates/test-harness/src/bin/m3_sidecar_roundtrip_fidelity.rs` passes for current `set_design_rule`, `assign_part`, `set_package`, `set_package_with_part`, `replace_component`, `replace_components`, `apply_component_replacement_plan`, `apply_component_replacement_policy`, `apply_scoped_component_replacement_policy`, `apply_scoped_component_replacement_plan`, and `set_net_class` save→reimport→save slice: rules/net-class remain sidecar-only byte-stable, while part/package replacement flows verify semantic sidecar replay and reimported board state after intentional footprint/body materialization |
| M3 write-surface parity hook behavioral | [x] | `crates/test-harness/src/bin/m3_write_surface_parity.rs` passes for current engine/daemon/MCP/CLI `move_component`/`rotate_component`/`set_value`/`set_reference`/`assign_part`/`set_package`/`set_package_with_part`/`set_net_class`/`delete_component`/`delete_track`/`delete_via`/`set_design_rule`/`undo`/`redo`/`save` slice, including follow-up derived-state checks |
| M3 aggregate acceptance gate behavioral | [x] | `crates/test-harness/src/bin/m3_acceptance_gate.rs` passes and composes base determinism, replacement determinism, base undo/redo, replacement undo/redo, board round-trip fidelity, sidecar round-trip fidelity, and write-surface parity into one milestone checkpoint |
| PLAN progress ticks added for M0/M1/M2/R1 | [x] | `PLAN.md` now has dated progress blocks |
| MCP contract split (current vs target) | [x] | `specs/MCP_API_SPEC.md` |
| PROGRAM spec references MCP contract split | [x] | `specs/PROGRAM_SPEC.md` M2 status section |
| ENGINE spec API split (current vs target) | [x] | `specs/ENGINE_SPEC.md` §5 |
| MCP design doc hardened to target/current labels | [x] | `docs/MCP_DESIGN.md` |
| User workflows aligned to current MCP/CLI surfaces | [x] | `docs/USER_WORKFLOWS.md` |
| Engine design doc marked target-state vs live API | [x] | `docs/ENGINE_DESIGN.md` |

---
