# Specification Progress Tracker

> **Purpose**: Maps every requirement from the controlling specs to its
> implementation status. Updated when code changes.
>
> Legend: `[x]` done, `[~]` partial, `[ ]` not started, `[—]` deferred/N/A

Last updated: 2026-03-28

---

## Current Repo Health

Current repo health status (2026-03-25 audit):
- `cargo test -q` currently passes.
- The current `M2` implementation slice is backed by `m2_quality`,
  `m2_perf`, CLI tests, daemon tests, and MCP self-test.
- Milestone completion status and workspace health are currently reconciled.
- API/daemon/MCP test-support monolith risk has been reduced via module splits,
  and file-size budgets are now enforced in CI.

### Drift RCA + Prevention (2026-03-25)

Drift cause (root):
- Prior alignment checks were primarily static text-presence checks.
- They did not verify factual claims against live repo state (for example
  daemon method count, git/CI status, and current MCP method catalog parity).
- `specs/MCP_API_SPEC.md` mixed historical deferral wording with current write
  support, creating internal contradiction.

Corrections landed:
- Added `scripts/check_progress_coverage.py` to enforce:
  - `PLAN.md` progress-block coverage for active milestones (`M0`-`M4`, `R1`)
  - `specs/PROGRESS.md` section presence and infrastructure-fact parity
  - MCP current-method parity (`dispatch.rs` ↔ `tools_catalog.py` ↔
    `specs/MCP_API_SPEC.md` list/headings)
  - stale deferral text rejection for write/save support
- Wired `check_progress_coverage.py` into CI (`.github/workflows/alignment.yml`).

---

## PROGRAM_SPEC.md — M0 Exit Criteria

| Criterion | Status | Notes |
|-----------|--------|-------|
| Pool types: Unit, Pin, Entity, Gate, Package, Pad, Padstack, Part, Symbol | [x] | `pool/mod.rs` |
| Pool serialization round-trip (100% of types) | [x] | serde derive + golden tests |
| Pool SQLite index: create, insert, keyword search, parametric search | [x] | `pool/mod.rs` PoolIndex |
| Deterministic serialization (byte-identical 3 runs) | [x] | `ir/serialization.rs` |
| Eagle .lbr import (20 libraries, 0 errors) | [x] | `import/eagle/mod.rs` |
| Eagle canonicalization round-trip | [x] | import → pool JSON → deserialize |
| Eagle deterministic re-import (identical UUIDs) | [x] | UUID v5 with eagle namespace |
| RuleScope IR compiles, serializes, leaf evaluator works | [x] | `rules/ast.rs`, `rules/eval.rs` |
| UUID v5 import identity | [x] | `ir/ids.rs` |
| .ids.json sidecar (write on import, restore on re-import) | [x] | `import/ids_sidecar.rs` |
| **No .lbr write-back** | [x] | import-only, confirmed |

**M0 overall**: [x] Complete (closed by architect)

---

## PROGRAM_SPEC.md — M1 Exit Criteria

| Criterion | Status | Notes |
|-----------|--------|-------|
| KiCad .kicad_pcb import: DOA2526 + 4 designs, 0 errors | [~] | Skeleton parser exists; warns "full geometry not implemented" |
| KiCad .kicad_sch import: DOA2526 + 2 schematics, 0 errors | [~] | Skeleton parser exists; warns "full symbol connectivity not implemented" |
| Eagle .brd/.sch import: 3 designs, 0 errors | [ ] | Returns "not implemented" error |
| Schematic connectivity: hierarchy, labels, power symbols | [~] | Union-find solver in `connectivity/mod.rs`; hierarchy links partial |
| Board connectivity: net-to-pin resolution | [x] | `board/mod.rs` net_pins(), net_pad_points() |
| Airwire computation: matches KiCad on DOA2526 | [~] | Algorithm implemented; DOA2526 validation not confirmed |
| Board diagnostics: net_without_copper, via_only, partially_routed | [x] | `board/mod.rs` diagnostics() |
| Golden tests: 8+ designs with checked-in golden files | [ ] | `tests/corpus/` is empty |
| Deterministic import: byte-identical on 3 runs | [~] | Serialization is deterministic; no corpus to verify against |
| Import fidelity KiCad ≥ 90% | [ ] | No fidelity matrix exists |
| Import fidelity Eagle ≥ 85% | [ ] | Eagle design import not implemented |

### PROGRAM_SPEC.md — M1 Query API

| Method | Engine | Daemon | MCP | CLI |
|--------|--------|--------|-----|-----|
| get_netlist | [x] | [x] | [x] | [ ] |
| get_components | [x] | [x] | [x] | [x] |
| get_nets / get_net_info | [x] | [x] | [x] | [x] |
| get_schematic_net_info | [x] | [x] | [x] | [ ] |
| get_board_summary | [x] | [x] | [x] | [x] |
| get_schematic_summary | [x] | [x] | [x] | [x] |
| get_sheets | [x] | [x] | [x] | [ ] |
| get_symbols | [x] | [x] | [x] | [ ] |
| get_ports | [x] | [x] | [x] | [x] |
| get_labels | [x] | [x] | [x] | [x] |
| get_buses | [x] | [x] | [x] | [ ] |
| get_bus_entries | [x] | [x] | [x] | [ ] |
| get_noconnects | [x] | [x] | [x] | [ ] |
| get_hierarchy | [x] | [x] | [x] | [x] |
| get_connectivity_diagnostics | [x] | [x] | [x] | [x] |
| get_unrouted | [x] | [x] | [x] | [x] |
| search_pool | [x] | [x] | [x] | [x] |

**M1 overall**: [~] ~50% — data models and query surface implemented; import fidelity, corpus, and golden tests not done

---

## PROGRAM_SPEC.md — M2 Exit Criteria

### ERC Rules (specs/ERC_SPEC.md §4)

| Check | Status | Notes |
|-------|--------|-------|
| output_to_output_conflict | [x] | `erc/mod.rs` |
| undriven_input | [x] | `erc/mod.rs` (undriven_input_pin + input_without_explicit_driver) |
| power_without_source | [x] | `erc/mod.rs` (power_in_without_source) |
| noconnect_connected | [x] | `erc/mod.rs` |
| unconnected_required_pin | [x] | `erc/mod.rs` (unconnected_component_pin) |
| passive_only_net | [x] | `erc/mod.rs` |
| hierarchical_connectivity_mismatch | [x] | Implemented as sheet-level hierarchical-label vs port mismatch check in `erc/mod.rs` |

ERC correctness: [x] `m2_quality` harness reports 0.0% FP / 0.0% FN on current ERC fixtures (2026-03-25)

### DRC Rules

| Check | Status | Notes |
|-------|--------|-------|
| clearance_copper | [x] | Implemented in `drc/mod.rs` (track-track same-layer clearance) |
| track_width | [x] | Implemented in `drc/mod.rs` (`track_width_below_min`) |
| via_hole | [x] | Implemented in `drc/mod.rs` (`via_hole_out_of_range`) |
| via_annular | [x] | Implemented in `drc/mod.rs` (`via_annular_below_min`) |
| connectivity | [x] | Implemented in `drc/mod.rs` (no-copper + unrouted-net checks) |
| unconnected_pins | [x] | Implemented in `drc/mod.rs` (`connectivity_unconnected_pin`) |
| silk_clearance | [x] | Implemented in `drc/mod.rs` (`silk_clearance_copper`) |

DRC correctness: [x] `m2_quality` harness reports 0.0% FP / 0.0% FN on current DRC fixtures (2026-03-25)

### MCP Tools (specs/MCP_API_SPEC.md — M2)

| Tool | Status | Notes |
|------|--------|-------|
| open_project | [x] | Current implementation contract |
| close_project | [x] | Current implementation contract |
| search_pool | [x] | Current implementation contract |
| get_part | [x] | Current implementation contract |
| get_package | [x] | Current implementation contract |
| get_package_change_candidates | [x] | Current implementation contract |
| get_board_summary | [x] | Current implementation contract |
| get_schematic_summary | [x] | Current implementation contract |
| get_sheets | [x] | Current implementation contract |
| get_symbols | [x] | Current (all-sheets only) |
| get_ports | [x] | Current (all-sheets only) |
| get_labels | [x] | Current (all-sheets only) |
| get_buses | [x] | Current (all-sheets only) |
| get_bus_entries | [x] | Current implementation contract |
| get_noconnects | [x] | Current (all-sheets only) |
| get_symbol_fields | [x] | Current implementation contract |
| get_hierarchy | [x] | Current implementation contract |
| get_netlist | [x] | Current implementation contract |
| get_components | [x] | Current implementation contract |
| get_net_info | [x] | Current (full inventory, not single-net selector) |
| get_schematic_net_info | [x] | Current (full inventory, not single-net selector) |
| get_connectivity_diagnostics | [x] | Current implementation contract |
| get_design_rules | [x] | Current implementation contract |
| get_unrouted | [x] | Current implementation contract |
| get_check_report | [x] | Current implementation contract (not in M2 exit list but implemented) |
| run_erc | [x] | Current implementation contract |
| run_drc | [x] | Current implementation contract |
| explain_violation | [x] | Current implementation contract |

MCP tools: 26/26 implemented

### CLI Commands (specs/PROGRAM_SPEC.md — M2)

| Command | Status | Notes |
|---------|--------|-------|
| tool import \<file\> | [x] | Eagle .lbr only in current slice |
| tool query \<design\> --nets | [x] | |
| tool query \<design\> --components | [x] | |
| tool query \<design\> --summary | [x] | |
| tool erc \<design\> | [x] | .kicad_sch only |
| tool drc \<design\> | [x] | Runs DRC, reports JSON/text, exits nonzero on violations |
| tool pool search \<query\> | [x] | |
| tool check \<design\> | [x] | Unified check report |
| CLI exit codes (0/1/2) | [x] | Checking commands now return 1 on violations and 2 only on execution errors |

### Other M2 Requirements

| Criterion | Status | Notes |
|-----------|--------|-------|
| MCP registration in active MCP host config | [x] | Configured `datum-eda` MCP entry with `EDA_ENGINE_SOCKET=/tmp/datum-eda-engine.sock` (2026-03-25) |
| Test corpus: 10+ designs with ERC/DRC golden files | [x] | `m2_quality` reports 11 unique designs (erc=5, drc=6) |
| Quality-rate harness (ERC/DRC FP/FN + corpus gate) | [x] | `m2_quality` implemented with checked-in manifest; current gate state is `pass=true` |
| ERC on DOA2526 < 3 seconds | [x] | `m2_perf` median ERC = 2ms (3-iteration run, 2026-03-25) |
| DRC on DOA2526 < 5 seconds | [x] | `m2_perf` median DRC = 18ms baseline, 20ms compare run (2026-03-25) |
| MCP transport (daemon ↔ MCP server) | [x] | Unix socket JSON-RPC implemented in daemon + Python client; live transport smoke is tracked separately in unrestricted CI |

**M2 overall**: [x] Complete for the current implementation slice — ERC/DRC checks, MCP/CLI parity, quality/performance harnesses, and local MCP host registration are all in place.

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
| M3 sidecar round-trip fidelity hook behavioral | [x] | `crates/test-harness/src/bin/m3_sidecar_roundtrip_fidelity.rs` passes for current `set_design_rule`, `assign_part`, `set_package`, `set_package_with_part`, `replace_component`, `replace_components`, `apply_component_replacement_plan`, `apply_component_replacement_policy`, `apply_scoped_component_replacement_policy`, `apply_scoped_component_replacement_plan`, and `set_net_class` save→reimport→save artifact stability slice |
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

## PROGRAM_SPEC.md — R1 Research Gates

| Criterion | Status | Notes |
|-----------|--------|-------|
| R1-G0 Foundation Gate | [x] | Minimum history/context gate is now evidenced in `docs/R1_G0_FOUNDATION.md` and blocks downstream milestone completion claims unless maintained |
| R1-G0 tool lineage + format evolution map | [x] | Evidence: `docs/R1_G0_FOUNDATION.md` §1 covers KiCad/Eagle/Altium/PADS/OrCAD lineage, likely ingestion surfaces, and roadmap implications |
| R1-G0 migration pain taxonomy | [x] | Evidence: `docs/R1_G0_FOUNDATION.md` §2 classifies migration failure modes using the current KiCad/Eagle design/library corpus and current interop workflow evidence |
| R1-G0 fidelity boundary policy | [x] | Evidence: `docs/R1_G0_FOUNDATION.md` §3 defines exactness, approximation, preservation-as-metadata, and unsupported-loss boundaries for future interop claims |

**R1 overall**: [~] Minimal context gate is complete, but broader commercial-interop research exit criteria remain open (corpus, legal posture, prototypes, recommendation)

---

## PROGRAM_SPEC.md — M3 Exit Criteria

| Criterion | Status | Notes |
|-----------|--------|-------|
| MoveComponent | [x] | Detailed status and evidence anchors are maintained in `specs/progress/m3_details.md#movecomponent`. |
| RotateComponent | [x] | Detailed status and evidence anchors are maintained in `specs/progress/m3_details.md#rotatecomponent`. |
| DeleteComponent | [x] | Detailed status and evidence anchors are maintained in `specs/progress/m3_details.md#deletecomponent`. |
| SetValue | [x] | Detailed status and evidence anchors are maintained in `specs/progress/m3_details.md#setvalue`. |
| SetReference | [x] | Detailed status and evidence anchors are maintained in `specs/progress/m3_details.md#setreference`. |
| AssignPart | [x] | Detailed status and evidence anchors are maintained in `specs/progress/m3_details.md#assignpart`. |
| SetPackage | [x] | Detailed status and evidence anchors are maintained in `specs/progress/m3_details.md#setpackage`. |
| SetPackageWithPart | [x] | Detailed status and evidence anchors are maintained in `specs/progress/m3_details.md#setpackagewithpart`. |
| ReplaceComponents | [x] | Detailed status and evidence anchors are maintained in `specs/progress/m3_details.md#replacecomponents`. |
| ApplyComponentReplacementPlan | [x] | Detailed status and evidence anchors are maintained in `specs/progress/m3_details.md#applycomponentreplacementplan`. |
| ApplyComponentReplacementPolicy | [x] | Detailed status and evidence anchors are maintained in `specs/progress/m3_details.md#applycomponentreplacementpolicy`. |
| ApplyScopedComponentReplacementPolicy | [x] | Detailed status and evidence anchors are maintained in `specs/progress/m3_details.md#applyscopedcomponentreplacementpolicy`. |
| ApplyScopedComponentReplacementPlan | [x] | Detailed status and evidence anchors are maintained in `specs/progress/m3_details.md#applyscopedcomponentreplacementplan`. |
| ScopedReplacementPlanManifest | [x] | Detailed status and evidence anchors are maintained in `specs/progress/m3_details.md#scopedreplacementplanmanifest`. |
| Package-change introspection | [x] | Detailed status and evidence anchors are maintained in `specs/progress/m3_details.md#package-change-introspection`. |
| Part-change introspection | [x] | Detailed status and evidence anchors are maintained in `specs/progress/m3_details.md#part-change-introspection`. |
| Replacement-plan introspection | [x] | Detailed status and evidence anchors are maintained in `specs/progress/m3_details.md#replacement-plan-introspection`. |
| SetNetClass | [x] | Detailed status and evidence anchors are maintained in `specs/progress/m3_details.md#setnetclass`. |
| SetDesignRule | [x] | Detailed status and evidence anchors are maintained in `specs/progress/m3_details.md#setdesignrule`. |
| DeleteTrack | [x] | Detailed status and evidence anchors are maintained in `specs/progress/m3_details.md#deletetrack`. |
| DeleteVia | [x] | Detailed status and evidence anchors are maintained in `specs/progress/m3_details.md#deletevia`. |
| Undo/redo (100% undoable) | [x] | Detailed status and evidence anchors are maintained in `specs/progress/m3_details.md#undoredo-100-undoable`. |
| Operation determinism | [x] | Detailed status and evidence anchors are maintained in `specs/progress/m3_details.md#operation-determinism`. |
| KiCad write-back | [x] | Detailed status and evidence anchors are maintained in `specs/progress/m3_details.md#kicad-write-back`. |
| Round-trip fidelity | [x] | Detailed status and evidence anchors are maintained in `specs/progress/m3_details.md#round-trip-fidelity`. |
| MCP write tools | [x] | Detailed status and evidence anchors are maintained in `specs/progress/m3_details.md#mcp-write-tools`. |
| Derived data recomputation | [x] | Detailed status and evidence anchors are maintained in `specs/progress/m3_details.md#derived-data-recomputation`. |
| CLI modify command | [x] | Detailed status and evidence anchors are maintained in `specs/progress/m3_details.md#cli-modify-command`. |

**M3 overall**: [x] M3 is complete for the implemented imported-board write slice; criterion-level details and evidence anchors are consolidated in `specs/progress/m3_details.md`.

---

## PROGRAM_SPEC.md — M4 Exit Criteria

| Criterion | Status | Notes |
|-----------|--------|-------|
| Native format | [x] | Detailed status and evidence anchors are maintained in `specs/progress/m4_details.md#native-format`. |
| Schematic operations | [x] | Detailed status and evidence anchors are maintained in `specs/progress/m4_details.md#schematic-operations`. |
| Board operations | [x] | Detailed status and evidence anchors are maintained in `specs/progress/m4_details.md#board-operations`. |
| Schematic query parity | [x] | Detailed status and evidence anchors are maintained in `specs/progress/m4_details.md#schematic-query-parity`. |
| Forward annotation | [x] | Detailed status and evidence anchors are maintained in `specs/progress/m4_details.md#forward-annotation`. |
| Gerber export | [x] | Detailed status and evidence anchors are maintained in `specs/progress/m4_details.md#gerber-export`. |
| Drill export | [x] | Detailed status and evidence anchors are maintained in `specs/progress/m4_details.md#drill-export`. |
| BOM export | [x] | Detailed status and evidence anchors are maintained in `specs/progress/m4_details.md#bom-export`. |
| PnP export | [x] | Detailed status and evidence anchors are maintained in `specs/progress/m4_details.md#pnp-export`. |
| Gerber comparison | [x] | Detailed status and evidence anchors are maintained in `specs/progress/m4_details.md#gerber-comparison`. |

**M4 overall**: [x] Closed for scope; details remain in `specs/progress/m4_details.md`.

## M5 Opening Charter

Status: [~] In progress
- Authority for the proposed opening charter and entry criteria:
  `specs/progress/m5_opening.md`
- Recommended M5 focus: deterministic layout-kernel groundwork from persisted
  native board state, opening with one narrow routing/constraint slice rather
  than broad placement/routing ambition.
- M5 must not inherit M4 parity work by default; new slices require an
  explicit layout-kernel contract and acceptance criteria.
- Current contract (2026-03-28): `project query <dir> routing-substrate`
  extracts a deterministic routing-kernel substrate report from persisted
  native board state only, covering outline, stackup/layer set, keepouts,
  authored/persisted pads, tracks, vias, zones, nets, and net classes.
- Current route slice (2026-03-28): `project query <dir> route-preflight --net
  <uuid>` reports deterministic single-net preflight state from persisted
  native board state only, covering persisted board-pad anchors for the target
  net, candidate copper layers from persisted stackup plus currently available
  persisted routing facts, authored obstacle inventory, and explicit
  `preflight_ready` / `blocked_by_authored_obstacle` /
  `insufficient_authored_inputs` status.
- Current next slice (2026-03-28): `project query <dir> route-corridor --net
  <uuid>` reports deterministic single-net corridor geometry from persisted
  native board state only, reusing the same authored anchors and candidate
  copper layers as `route-preflight` while adding deterministic authored
  obstacle geometry plus available/blocked corridor spans without entering
  route search or path scoring.
- Current path slice (2026-03-28): `project query <dir> route-path-candidate
  --net <uuid> --from-anchor <pad_uuid> --to-anchor <pad_uuid>` reports a
  deterministic single-layer point-to-point path result for one authored
  anchor pair under the existing corridor/span model, returning exactly
  `deterministic_path_found` with one ordered polyline taken directly from the
  first unblocked matching corridor span in corridor report order, or
  `no_path_under_current_authored_constraints`.
- Current explanation slice (2026-03-28): `project query <dir>
  route-path-candidate-explain --net <uuid> --from-anchor <pad_uuid>
  --to-anchor <pad_uuid>` reports the current single-layer path-candidate
  result as a deterministic explanation surface, including selected span when
  found or explicit no-match/all-blocked cause buckets from existing
  corridor/path facts only.
- Current M5 checkpoint boundary (2026-03-28): the accepted deterministic
  read-only kernel chain is `routing-substrate` -> `route-preflight` ->
  `route-corridor` -> `route-path-candidate` ->
  `route-path-candidate-explain`. This state is intentionally checkpointed;
  the next contract is not opened implicitly from here.
- Next planning question after this checkpoint is explicit and separate:
  either stop M5 here temporarily or open one new tightly-scoped contract in a
  later planning step.
- Current reopened slice (2026-03-29): `project query <dir>
  route-path-candidate-via --net <uuid> --from-anchor <pad_uuid>
  --to-anchor <pad_uuid>` reports a deterministic point-to-point path
  candidate that reuses one already-authored persisted via only, chosen by an
  explicit ascending-UUID via selection rule with no invented transition
  permissions.
- Current follow-on explanation slice (2026-03-29): `project query <dir>
  route-path-candidate-via-explain --net <uuid> --from-anchor <pad_uuid>
  --to-anchor <pad_uuid>` reports the current selected via when found, or
  whether failure came from no matching authored via versus all matching vias
  blocked, using only existing via/path facts from the accepted via slice.
- Current reopened capability slice (2026-03-29): `project query <dir>
  route-path-candidate-two-via --net <uuid> --from-anchor <pad_uuid>
  --to-anchor <pad_uuid>` reports a deterministic point-to-point path
  candidate that reuses exactly two already-authored persisted vias only,
  chosen by an explicit ascending `(via_a_uuid, via_b_uuid)` rule when their
  layer sequence connects the requested anchor layers through one intermediate
  copper layer.
- Current follow-on explanation slice (2026-03-29): `project query <dir>
  route-path-candidate-two-via-explain --net <uuid> --from-anchor <pad_uuid>
  --to-anchor <pad_uuid>` reports the current selected via pair when found, or
  whether failure came from no matching authored via pair versus all matching
  via pairs blocked, using only existing two-via/path facts from the accepted
  two-via slice.
- Acceptance checks for the opening slice:
  - deterministic repeated output on unchanged persisted state
  - no pool re-resolution or live import-session dependence
  - no invented routing geometry or inferred constraints beyond persisted facts
  - engine + CLI test coverage for sorted/deterministic extraction

---

## ENGINE_SPEC.md — Core Types

### §1.1 Geometry Primitives

| Type | Status |
|------|--------|
| Point | [x] |
| Rect | [x] |
| Polygon | [x] |
| Arc | [x] |
| LayerId | [x] |

### §1.1a Shared Enums

| Type | Status | Notes |
|------|--------|-------|
| PinDirection (10 variants) | [x] | In pool/mod.rs as PinElectricalType on SymbolPin |
| Lifecycle | [x] | In pool/mod.rs |
| StackupLayerType | [x] | In board/mod.rs |
| PortDirection | [x] | In schematic/mod.rs |
| Primitive | [x] | In pool/mod.rs (Symbol graphics) |

### §1.2 Pool Types

| Type | Status |
|------|--------|
| Pin | [x] |
| Unit | [x] |
| Gate | [x] |
| Entity | [x] |
| Pad | [x] |
| Package | [x] |
| PadMapEntry | [x] |
| Part | [x] |

### §1.3 Board Types

| Type | Status |
|------|--------|
| Board | [x] |
| PlacedPackage | [x] |
| Track | [x] |
| Via | [x] |
| Zone | [x] |
| Net | [x] |
| NetClass | [x] |
| Stackup / StackupLayer | [x] |
| Keepout | [x] |
| Dimension | [x] |
| BoardText | [x] |

### §1.4 Schematic Types

| Type | Status |
|------|--------|
| Schematic | [x] |
| Sheet | [x] |
| PlacedSymbol | [x] |
| SheetDefinition | [x] |
| SheetInstance | [x] |
| SchematicWire | [x] |
| Junction | [x] |
| NetLabel / LabelKind | [x] |
| Bus | [x] |
| BusEntry | [x] |
| HierarchicalPort | [x] |
| NoConnectMarker | [x] |
| Variant | [x] |

### §1.5 Rule Types

| Type | Status |
|------|--------|
| Rule | [x] |
| RuleType (7 variants) | [x] |
| RuleParams (7 variants) | [x] |
| RuleScope (leaf nodes) | [x] |
| RuleScope (combinators) | [x] parse, [—] eval deferred to M6 |

### §2 Object Invariants

| Invariant | Status | Notes |
|-----------|--------|-------|
| All authored objects have non-nil UUID | [x] | Enforced by constructors |
| No duplicate UUIDs within type | [~] | Not explicitly validated |
| Non-dangling references | [~] | DanglingReference error exists; validation not run on import |
| Integer coordinates (no float) | [x] | i64 throughout |
| Connectivity recomputed from authored data | [x] | |

### §3 Operations

| Item | Status |
|------|--------|
| Operation trait | [ ] |
| OpDiff | [ ] |
| Transaction | [ ] |
| Undo/redo semantics | [ ] |
| Derived data invalidation | [ ] |

### §5 Engine API

| Method | Status | Notes |
|--------|--------|-------|
| new() | [x] | |
| has_open_project() | [x] | |
| import() | [x] | KiCad skeleton + Eagle .lbr |
| save() | [~] | Writes unmodified imported KiCad boards byte-identically, persists current `delete_track`/`delete_via`/`delete_component` removals, rewrites current `move_component`/`rotate_component` footprint placement and component `Value`/`Reference` properties, rewrites package-backed footprint bodies for current `set_package`, `set_package_with_part`, `replace_component`, `replace_components`, `apply_component_replacement_plan`, `apply_component_replacement_policy`, `apply_scoped_component_replacement_policy`, and `apply_scoped_component_replacement_plan` slices, and writes rule/part-assignment/package-assignment/net-class sidecars for current `set_design_rule`/`assign_part`/`set_package`/`set_package_with_part`/`replace_component`/`replace_components`/`apply_component_replacement_plan`/`apply_component_replacement_policy`/`apply_scoped_component_replacement_policy`/`apply_scoped_component_replacement_plan`/`set_net_class` slice |
| save_to_original() | [~] | Current M3 helper for imported-design write-back to original file |
| search_pool() | [x] | |
| import_eagle_library() | [x] | |
| get_board_summary() | [x] | |
| get_components() | [x] | |
| get_package_change_candidates() | [x] | Current engine API supports component-scoped package compatibility introspection |
| get_part_change_candidates() | [x] | Current engine API supports component-scoped part compatibility introspection |
| get_component_replacement_plan() | [x] | Current engine API supports unified component replacement planning introspection |
| get_scoped_component_replacement_plan() | [x] | Current engine API supports scoped policy-driven replacement preview introspection |
| edit_scoped_component_replacement_plan() | [x] | Current engine API supports scoped replacement preview post-processing via exclusions and explicit compatible target overrides |
| replace_components() | [x] | Current M3 board write slice supports batched explicit component replacement as one transaction / one undo step |
| apply_component_replacement_plan() | [x] | Current M3 board write slice resolves package/part selections from the unified replacement plan and applies them as one transaction |
| apply_component_replacement_policy() | [x] | Current M3 board write slice resolves deterministic best-candidate replacement policies from the unified replacement plan and applies them as one transaction |
| apply_scoped_component_replacement_policy() | [x] | Current M3 board write slice resolves deterministic best-candidate replacement policies over a scoped component filter and applies them as one transaction |
| apply_scoped_component_replacement_plan() | [x] | Current M3 board write slice validates and applies a previously previewed scoped replacement plan without re-resolving policy |
| get_net_info() | [x] | Returns all nets, not single-net |
| get_stackup() | [x] | |
| get_unrouted() | [x] | |
| get_schematic_summary() | [x] | |
| get_sheets() | [x] | |
| get_symbols() | [x] | |
| get_ports() | [x] | |
| get_labels() | [x] | |
| get_buses() | [x] | |
| get_noconnects() | [x] | |
| get_hierarchy() | [x] | |
| get_schematic_net_info() | [x] | Returns all nets, not single-net |
| get_connectivity_diagnostics() | [x] | |
| get_check_report() | [x] | |
| delete_component() | [x] | Current M3 board write slice |
| delete_track() | [x] | Current M3 board write slice |
| delete_via() | [x] | Current M3 board write slice |
| move_component() | [x] | Current M3 board write slice |
| rotate_component() | [x] | Current M3 board write slice |
| set_value() | [x] | Current M3 board write slice |
| set_reference() | [x] | Current M3 board write slice |
| assign_part() | [x] | Current M3 board write slice |
| set_package() | [x] | Current M3 board write slice |
| set_package_with_part() | [x] | Current M3 board write slice |
| set_net_class() | [x] | Current M3 board write slice |
| set_design_rule() | [x] | Current M3 board write slice |
| run_erc_prechecks() | [x] | |
| run_drc() | [x] | Engine API method implemented (connectivity + clearance checks currently) |
| execute() / execute_batch() | [ ] | |
| undo() / redo() | [~] | Current engine API supports transaction reversal for `delete_component`, `delete_track`, `delete_via`, `move_component`, `rotate_component`, `set_value`, `set_reference`, `assign_part`, `set_package`, `set_package_with_part`, `set_net_class`, and `set_design_rule` |

---

## CHECKING_ARCHITECTURE_SPEC.md

| Item | Status | Notes |
|------|--------|-------|
| CheckDomain enum (ERC, DRC) | [x] | Implemented as check report domain field |
| CheckSeverity (Error, Warning, Info) | [x] | ErcSeverity in erc/mod.rs |
| CheckSummary | [x] | In api/mod.rs |
| CheckWaiver (domain, target, rationale) | [x] | In schematic/mod.rs |
| WaiverTarget variants | [~] | Basic UUID matching; RuleObjects not fully exercised |
| Waiver matching in ERC | [x] | |
| Waiver matching in DRC | [ ] | DRC checks exist; waiver matching for DRC violations not implemented yet |
| Cross-domain checks excluded from M2 | [x] | Not implemented (correct) |

---

## ERC_SPEC.md

| Item | Status | Notes |
|------|--------|-------|
| PinElectricalType enum (10 variants) | [x] | In schematic/mod.rs SymbolPin |
| NetSemanticClass enum | [~] | Partial; power/signal inference exists |
| M2 rule: output_to_output_conflict | [x] | |
| M2 rule: undriven_input | [x] | |
| M2 rule: power_without_source | [x] | |
| M2 rule: noconnect_connected | [x] | |
| M2 rule: unconnected_required_pin | [x] | |
| M2 rule: passive_only_net | [x] | |
| M2 rule: hierarchical_connectivity_mismatch | [x] | Sheet-level hierarchical label/port mismatch check implemented |
| Compatibility matrix (pin-pair) | [~] | Driving analysis exists; full matrix not formalized |
| ErcReport struct | [x] | Via CheckReport |
| ErcViolation struct | [x] | ErcFinding |
| Waiver integration | [x] | |
| Configurable severity | [x] | ErcConfig with BTreeMap overrides |

---

## IMPORT_SPEC.md

| Item | Status | Notes |
|------|--------|-------|
| NAMESPACE_KICAD UUID | [x] | ir/ids.rs |
| NAMESPACE_EAGLE UUID | [x] | ir/ids.rs |
| import_uuid() function | [x] | |
| KiCad object path construction | [~] | Skeleton parser; not all object types mapped |
| Eagle object path construction | [x] | Full library mapping |
| .ids.json schema | [x] | import/ids_sidecar.rs |
| .ids.json precedence rules (3 cases) | [x] | restore_or_merge_mappings() |
| KiCad board feature matrix (Required items) | [~] | Skeleton only; geometry not fully parsed |
| KiCad schematic feature matrix (Required items) | [~] | Skeleton only |
| Eagle board feature matrix | [ ] | Design import not implemented |
| Eagle schematic feature matrix | [ ] | Design import not implemented |
| Eagle library feature matrix | [x] | Symbols, packages, devicesets |

---

## SCHEMATIC_CONNECTIVITY_SPEC.md

| Item | Status | Notes |
|------|--------|-------|
| Sheet-local wire connectivity | [x] | Union-find in connectivity/mod.rs |
| Junction semantics | [x] | |
| Local labels | [x] | |
| Global labels | [x] | |
| Hierarchical labels/ports | [~] | Basic support; full hierarchy links partial |
| Power symbols | [x] | |
| Bus/member expansion | [~] | Bus structures exist; expansion logic partial |
| Net naming and identity | [x] | |
| Connectivity diagnostics | [x] | |
| Deterministic graph output | [x] | Sorted by UUID |

---

## SCHEMATIC_EDITOR_SPEC.md — M4 Operations

Status: [x] Closed for scoped M4 slice
- Verified by code audit in `crates/cli/src/cli_args.rs`,
  `crates/cli/src/command_exec.rs`, and dedicated CLI test shards:
  `main_tests_project_symbol*.rs`, `main_tests_project_label.rs`,
  `main_tests_project_wire.rs`, `main_tests_project_junction.rs`,
  `main_tests_project_port.rs`, `main_tests_project_bus.rs`,
  and `main_tests_project_noconnect.rs`.
- Implemented schematic operation families include symbol
  place/move/rotate/mirror/delete, symbol field add/edit/delete,
  label place/rename/delete, wire draw/delete, junction place/delete,
  hierarchical port place/edit/delete, bus create/edit/place-entry/delete-entry,
  and no-connect place/delete.
- Remaining editor expansion items such as power-symbol placement,
  sheet-instance lifecycle operations, and deterministic annotate flow
  completion are deferred beyond scoped M4 closure.

---

## NATIVE_FORMAT_SPEC.md

Status: [x] Closed for scoped M4 slice
- Native project scaffold, deterministic file layout, and first native
  read/query/check surfaces are implemented in the current M4 slice.
- Remaining native-format contract areas (full schema coverage, migration
  completeness, and richer manufacturing/output semantics remain open beyond
  scoped M4 closure.

---

## Infrastructure

| Item | Status | Notes |
|------|--------|-------|
| Rust workspace (4 crates) | [x] | engine, cli, engine-daemon, test-harness |
| Engine compiles without GUI deps | [x] | |
| Test harness (golden file utilities) | [x] | test-harness crate |
| Test corpus (real designs) | [ ] | tests/corpus/ empty |
| Daemon JSON-RPC dispatch | [x] | 53 methods in `dispatch.rs`, with coverage in daemon tests |
| Daemon socket transport | [x] | `main()` parses `--socket` and serves Unix socket; live smoke is environment-gated because sandboxed local runs deny socket IPC |
| MCP Python server (tool host) | [x] | Tool definitions + stdio dispatch |
| MCP→daemon transport | [x] | `EngineDaemonClient.call()` uses Unix socket JSON-RPC; behavioral parity remains covered separately from live socket smoke |
| Git repository initialized | [x] | `main` branch with GitHub remote configured |
| CI pipeline | [x] | `.github/workflows/alignment.yml` runs alignment and file-size budget checks |
