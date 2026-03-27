# Specification Progress Tracker

> **Purpose**: Maps every requirement from the controlling specs to its
> implementation status. Updated when code changes.
>
> Legend: `[x]` done, `[~]` partial, `[ ]` not started, `[—]` deferred/N/A

Last updated: 2026-03-26

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
| MoveComponent | [x] | Engine/daemon/MCP/CLI current slice implemented for board components by UUID with persisted KiCad footprint `(at ...)` rewrites |
| RotateComponent | [x] | Engine/daemon/MCP/CLI current slice implemented for board components by UUID with persisted KiCad footprint rotation rewrites |
| DeleteComponent | [x] | Engine/daemon/MCP/CLI current slice implemented for board components by UUID with persisted KiCad `footprint` removal |
| SetValue | [x] | Engine/daemon/MCP/CLI current slice implemented for board components by UUID with persisted KiCad `Value` property rewrites |
| SetReference | [x] | Engine/daemon/MCP/CLI current slice implemented for board components by UUID with persisted KiCad `Reference` property rewrites |
| AssignPart | [x] | Engine/daemon/MCP/CLI current slice implemented for board components by UUID with sidecar-backed part assignment persistence and logical pin-net preservation when remapping between known pool parts |
| SetPackage | [x] | Engine/daemon/MCP/CLI current slice implemented for board components by UUID with package-backed KiCad footprint rewrite, sidecar-backed package assignment persistence, and logical pin-net preservation when a unique compatible pool part exists for the target package |
| SetPackageWithPart | [x] | Engine/daemon/MCP/CLI current slice implemented for board components by UUID with explicit caller-selected compatible part+package mutation, package-backed KiCad footprint rewrite, and logical pin-net preservation |
| ReplaceComponents | [x] | Engine/daemon/MCP current slice implements batched explicit component replacement as one transaction / one undo step; CLI collapses repeated `--replace-component` flags into that same batch transaction |
| ApplyComponentReplacementPlan | [x] | Engine/daemon/MCP current slice implements plan-driven replacement selection from `get_component_replacement_plan`; CLI exposes the same path via `--apply-replacement-plan` |
| ApplyComponentReplacementPolicy | [x] | Engine/daemon/MCP current slice implements deterministic best-candidate replacement selection from the current replacement plan; CLI exposes the same path via `--apply-replacement-policy` |
| ApplyScopedComponentReplacementPolicy | [x] | Engine/daemon/MCP current slice implements deterministic best-candidate replacement selection over a scoped component filter; CLI exposes the same path via `--apply-scoped-replacement-policy` |
| ApplyScopedComponentReplacementPlan | [x] | Engine/daemon/MCP current slice applies a previously previewed scoped replacement plan without re-resolving policy; CLI exposes the same path via `--apply-scoped-replacement-plan-file` and requires matching pool libraries for target part/package resolution |
| ScopedReplacementPlanManifest | [x] | CLI current slice supports versioned scoped replacement manifest export, inspect, explicit validation/preflight (single or batch manifest paths), rewrite-to-current-schema (to a new path or in place), manifest-backed apply, and legacy unversioned manifest upgrade-on-load with recorded board/library provenance, drift checks, inspect-time migration metadata, apply-time migration reporting, and human-readable text summaries via `plan export-scoped-replacement-manifest`, `plan inspect-scoped-replacement-manifest`, `plan validate-scoped-replacement-manifest`, `plan upgrade-scoped-replacement-manifest`, and `--apply-scoped-replacement-manifest`; dedicated CLI shards cover normal, legacy, text-output, and validation flows |
| Package-change introspection | [x] | Engine/daemon/MCP/CLI query surface exposes component-scoped package compatibility candidates and resolution status for current board slice |
| Part-change introspection | [x] | Engine/daemon/MCP/CLI query surface exposes component-scoped compatible part candidates for current board slice |
| Replacement-plan introspection | [x] | Engine/daemon/MCP/CLI query surface exposes unified per-component and scoped replacement planning reports plus first-class scoped-plan exclusion/override editing for current board slice, with batch apply support via `replace_components` |
| SetNetClass | [x] | Engine/daemon/MCP/CLI current slice implemented for board nets by UUID with sidecar-backed net-class persistence |
| SetDesignRule | [x] | Engine/daemon/MCP current slice implemented; CLI exposes the default all-scope clearance rule path and follow-up `query design-rules` verification surface |
| DeleteTrack | [x] | Engine/daemon/MCP/CLI current slice implemented for board tracks by UUID |
| DeleteVia | [x] | Engine/daemon/MCP/CLI current slice implemented for board vias by UUID |
| Undo/redo (100% undoable) | [x] | Current slice supports undo/redo for `delete_track`, `delete_via`, `delete_component`, `set_design_rule`, `set_value`, `set_reference`, `assign_part`, `set_package`, `set_package_with_part`, `set_net_class`, `move_component`, `rotate_component`, `replace_component`, `replace_components`, `apply_component_replacement_plan`, `apply_component_replacement_policy`, `apply_scoped_component_replacement_policy`, and `apply_scoped_component_replacement_plan`; dedicated round-trip hooks now behaviorally prove the full current undoable M3 slice |
| Operation determinism | [x] | `m3_op_determinism` plus `m3_replacement_op_determinism` now pass for the full current save-backed M3 mutation slice: `move_component`, `delete_track`, `delete_via`, `delete_component`, `rotate_component`, `set_value`, `set_reference`, `set_design_rule`, `assign_part`, `set_package`, `set_package_with_part`, `replace_component`, `replace_components`, `apply_component_replacement_plan`, `apply_component_replacement_policy`, `apply_scoped_component_replacement_policy`, `apply_scoped_component_replacement_plan`, and `set_net_class` |
| KiCad write-back | [x] | `save()` writes unmodified imported KiCad boards byte-identically, persists current `delete_track`/`delete_via`/`delete_component` removals, rewrites current `move_component`/`rotate_component` footprint `(at ...)` state and component `Value`/`Reference` properties, rewrites package-backed footprint bodies for current `set_package`, `set_package_with_part`, `replace_component`, `replace_components`, `apply_component_replacement_plan`, `apply_component_replacement_policy`, and `apply_scoped_component_replacement_plan` slices, and writes authored-rule, part-assignment, package-assignment, and net-class sidecars for current `set_design_rule`/`assign_part`/`set_package`/`set_package_with_part`/`replace_component`/`replace_components`/`apply_component_replacement_plan`/`apply_component_replacement_policy`/`apply_scoped_component_replacement_policy`/`apply_scoped_component_replacement_plan`/`set_net_class` slice; dedicated board and sidecar fidelity hooks now behaviorally prove those current write-back paths |
| Round-trip fidelity | [x] | Dedicated `m3_board_roundtrip_fidelity` and `m3_sidecar_roundtrip_fidelity` hooks now behaviorally prove save→reimport→save artifact stability for the full current M3 save slice: unmodified boards, `delete_track`, `delete_via`, `delete_component`, `move_component`, `rotate_component`, `set_value`, `set_reference`, `set_design_rule`, `assign_part`, `set_package`, `set_package_with_part`, `replace_component`, `replace_components`, `apply_component_replacement_plan`, `apply_component_replacement_policy`, `apply_scoped_component_replacement_policy`, `apply_scoped_component_replacement_plan`, and `set_net_class` |
| MCP write tools | [x] | `save`, `move_component`, `rotate_component`, `set_value`, `set_reference`, `assign_part`, `set_package`, `set_package_with_part`, `set_net_class`, `set_design_rule`, `delete_component`, `delete_track`, `delete_via`, `undo`, and `redo` wired in current slice |
| Derived data recomputation | [x] | Engine tests prove immediate post-op updates for current `delete_track` net/diagnostic/DRC state, `move_component` airwire/DRC state, `delete_via` net-info state, `assign_part` package-regenerated net-info state, and `set_package`/`set_package_with_part` logical-remap net-info preservation when a compatible target part exists; write-surface parity now also proves follow-up query/check behavior across daemon/MCP/CLI for all current write ops: `delete_track`, `delete_via`, `delete_component`, `move_component`, `rotate_component`, `set_value`, `set_reference`, `assign_part`, `set_package`, `set_package_with_part`, `set_net_class`, and `set_design_rule` |
| CLI modify command | [x] | Current slice supports `modify <board> --move-component/--rotate-component/--set-value/--set-reference/--assign-part/--set-package/--set-package-with-part/--set-net-class/--delete-component/--delete-track/--delete-via/--set-clearance-min-nm/--undo/--redo/--save`, with follow-up `query components`/`query nets`/`query design-rules` verification for current `delete_component`/`rotate_component`/`set_value`/`set_reference`/`assign_part`/`set_package`/`set_package_with_part`/`set_net_class`/`set_design_rule` |

**M3 overall**: [x] Complete for the implemented imported-board write slice; closure evidence remains in place and the milestone/spec wording has been audited against `R1-G0` so the completion claim stays scoped to the actual KiCad imported-board slice and its explicit fidelity boundaries

---

## PROGRAM_SPEC.md — M4 Exit Criteria

| Criterion | Status | Notes |
|-----------|--------|-------|
| Native format | [~] | Current M4 CLI slice supports deterministic scaffold creation, scaffold inspection, and the first native read/query surface: `project new <dir> [--name]` writes versioned native `project.json`, `schematic/schematic.json`, `board/board.json`, and `rules/rules.json`; `project inspect <dir>` validates the resolved file layout; `project query <dir> summary`, `project query <dir> design-rules`, `project query <dir> symbols`, `project query <dir> symbol-fields --symbol <uuid>`, `project query <dir> symbol-semantics --symbol <uuid>`, `project query <dir> symbol-pins --symbol <uuid>`, `project query <dir> texts`, and `project query <dir> drawings` read the native scaffold directly and report aggregated native schematic/board/rule state; focused CLI tests prove idempotent rewrite, inspect-path correctness, referenced-sheet summary counts, native rules payload reporting, and native symbol/text/drawing persistence-query behavior |
| Schematic operations | [~] | Nine native authored schematic object families now have focused coverage: symbols (`project place-symbol <dir> --sheet <uuid> --reference <text> --value <text> [--lib-id <text>] --x-nm <i64> --y-nm <i64> [--rotation-deg <i32>] [--mirrored]`, `project move-symbol <dir> --symbol <uuid> --x-nm <i64> --y-nm <i64>`, `project rotate-symbol <dir> --symbol <uuid> --rotation-deg <i32>`, `project mirror-symbol <dir> --symbol <uuid>`, `project delete-symbol <dir> --symbol <uuid>`, `project set-symbol-reference <dir> --symbol <uuid> --reference <text>`, `project set-symbol-value <dir> --symbol <uuid> --value <text>`, `project set-symbol-unit <dir> --symbol <uuid> --unit <text>`, `project clear-symbol-unit <dir> --symbol <uuid>`, `project set-symbol-gate <dir> --symbol <uuid> --gate <uuid>`, `project clear-symbol-gate <dir> --symbol <uuid>`, `project set-symbol-display-mode <dir> --symbol <uuid> --mode <library-default|show-hidden-pins|hide-optional-pins>`, `project set-symbol-hidden-power-behavior <dir> --symbol <uuid> --behavior <source-defined-implicit|explicit-power-object|preserved-as-imported-metadata>`, `project set-pin-override <dir> --symbol <uuid> --pin <uuid> --visible <true|false> [--x-nm <i64> --y-nm <i64>]`, `project clear-pin-override <dir> --symbol <uuid> --pin <uuid>`, `project add-symbol-field <dir> --symbol <uuid> --key <text> --value <text> [--hidden] [--x-nm <i64> --y-nm <i64>]`, `project edit-symbol-field <dir> --field <uuid> [--key <text>] [--value <text>] [--visible <true|false>] [--x-nm <i64> --y-nm <i64>]`, `project delete-symbol-field <dir> --field <uuid>`), text objects (`project place-text <dir> --sheet <uuid> --text <text> --x-nm <i64> --y-nm <i64> [--rotation-deg <i32>]`, `project edit-text <dir> --text <uuid> [--value <text>] [--x-nm <i64> --y-nm <i64>] [--rotation-deg <i32>]`, `project delete-text <dir> --text <uuid>`), drawing primitives (`project place-drawing-line <dir> --sheet <uuid> --from-x-nm <i64> --from-y-nm <i64> --to-x-nm <i64> --to-y-nm <i64>`, `project place-drawing-rect <dir> --sheet <uuid> --min-x-nm <i64> --min-y-nm <i64> --max-x-nm <i64> --max-y-nm <i64>`, `project place-drawing-circle <dir> --sheet <uuid> --center-x-nm <i64> --center-y-nm <i64> --radius-nm <i64>`, `project place-drawing-arc <dir> --sheet <uuid> --center-x-nm <i64> --center-y-nm <i64> --radius-nm <i64> --start-angle-mdeg <i64> --end-angle-mdeg <i64>`, `project edit-drawing-line <dir> --drawing <uuid> [--from-x-nm <i64> --from-y-nm <i64>] [--to-x-nm <i64> --to-y-nm <i64>]`, `project edit-drawing-rect <dir> --drawing <uuid> ...`, `project edit-drawing-circle <dir> --drawing <uuid> ...`, `project edit-drawing-arc <dir> --drawing <uuid> ...`, `project delete-drawing <dir> --drawing <uuid>`), labels (`project place-label <dir> --sheet <uuid> --name <text> [--kind local|global|hierarchical|power] --x-nm <i64> --y-nm <i64>`, `project rename-label <dir> --label <uuid> --name <text>`, `project delete-label <dir> --label <uuid>`), wires (`project draw-wire <dir> --sheet <uuid> --from-x-nm <i64> --from-y-nm <i64> --to-x-nm <i64> --to-y-nm <i64>`, `project delete-wire <dir> --wire <uuid>`), junctions (`project place-junction <dir> --sheet <uuid> --x-nm <i64> --y-nm <i64>`, `project delete-junction <dir> --junction <uuid>`), hierarchical ports (`project place-port <dir> --sheet <uuid> --name <text> --direction <input|output|bidirectional|passive> --x-nm <i64> --y-nm <i64>`, `project edit-port <dir> --port <uuid> [--name <text>] [--direction ...] [--x-nm <i64>] [--y-nm <i64>]`, `project delete-port <dir> --port <uuid>`), buses (`project create-bus <dir> --sheet <uuid> --name <text> --member <text>...`, `project edit-bus-members <dir> --bus <uuid> --member <text>...`, `project place-bus-entry <dir> --sheet <uuid> --bus <uuid> [--wire <uuid>] --x-nm <i64> --y-nm <i64>`, `project delete-bus-entry <dir> --bus-entry <uuid>`), and no-connect markers (`project place-noconnect <dir> --sheet <uuid> --symbol <uuid> --pin <uuid> --x-nm <i64> --y-nm <i64>`, `project delete-noconnect <dir> --noconnect <uuid>`); `project query <dir> symbols`, `project query <dir> symbol-fields --symbol <uuid>`, `project query <dir> symbol-semantics --symbol <uuid>`, `project query <dir> symbol-pins --symbol <uuid>`, `project query <dir> texts`, `project query <dir> drawings`, `project query <dir> labels`, `project query <dir> wires`, `project query <dir> junctions`, `project query <dir> ports`, `project query <dir> buses`, `project query <dir> bus-entries`, `project query <dir> noconnects`, and `project query <dir> summary` verify the persisted results through the native read surface |
| Board operations | [ ] | |
| Schematic query parity | [~] | Native CLI read/query currently covers symbols, symbol fields, symbol semantic selection state, symbol pins with per-pin override state, texts, drawing primitives, labels, hierarchy ports, wires, junctions, buses, bus entries, no-connects, summary aggregation, and design-rules inspection; full engine/daemon/MCP/CLI parity for all required schematic topology queries remains open |
| Forward annotation | [ ] | |
| Gerber export | [ ] | `export/mod.rs` empty |
| Drill export | [ ] | |
| BOM export | [ ] | |
| PnP export | [ ] | |
| Gerber comparison | [ ] | |

**M4 overall**: [~] Native-project persistence now has deterministic scaffold creation, minimal native inspect/query surfaces, and focused authored coverage for symbols, text objects, drawing primitives, labels, wires, junctions, hierarchical ports, buses/bus entries, and no-connect markers; most schematic ops, all board authoring ops, ECO, and manufacturing exports remain open

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

All 30 operations: [ ] Not started (M4 scope)

---

## NATIVE_FORMAT_SPEC.md

All items: [ ] Not started (M4 scope)

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
