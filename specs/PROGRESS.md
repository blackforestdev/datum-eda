# Specification Progress Tracker

> **Purpose**: Maps every requirement from the controlling specs to its
> implementation status. Updated when code changes.
>
> Legend: `[x]` done, `[~]` partial, `[ ]` not started, `[—]` deferred/N/A

Last updated: 2026-06-19

**Current-vs-target framing**:
- **Current implementation evidence**: the historical milestone tables below
  remain truthful records of the implementation slices that have landed.
- **Active target**: product-mechanics substrate readiness before further
  imported-board fidelity expansion.
- **Not the North Star**: legacy M0-M7 milestone completion rows are retained as
  evidence, but they no longer define the next implementation priority.

**Active milestone**: substrate readiness for product mechanics. Imported-board
fidelity work is paused unless it directly proves or unblocks the substrate
contracts below.
**Frozen**: M6 (strategy reporting layer landed; pending repeated evidence
runs from the checked-in baseline gate).
**Closed for scope**: M0–M5.
**Spec stubs awaiting implementation**: Standards Audit Batch 1 — see
section "Standards Audit Batch 1 — Spec Stubs Awaiting Implementation"
below.

Machine-checked inventory shapes live in `specs/SPEC_PARITY.md` (gated by
`scripts/check_spec_parity.py`, wired into `scripts/run_drift_gates.sh`).
Surfaces currently locked: `mcp_runtime_methods`, `cli_project_commands`,
`engine_text_modules`, `m7_text_visual_fixtures`, `workspace_crates`.

---

## Scope Integration / Substrate Readiness

This is the active tracking surface for the new scope. Items here should move
from target to current evidence only when there is an implementation anchor,
test/gate, and CLI/MCP/API surface where applicable. Imported-board fidelity can
continue only after the relevant substrate contracts are implemented or after an
explicit governance decision records why a fidelity slice is exempt.

| Substrate Area | Status | Current Evidence | Target / Readiness Definition |
|----------------|--------|------------------|-------------------------------|
| ProjectResolver | [ ] | Existing import/query paths resolve files enough for current fixtures, but no product-level resolver contract is tracked here yet. | One resolver owns project roots, source discovery, dependency resolution, identity lookup, and deterministic diagnostics across native, imported, CLI, MCP, and GUI paths. |
| Source shards | [ ] | Native fixture validation exists for current project files; imported KiCad/Eagle paths remain format-centric. | Project state is decomposed into source shards with explicit ownership, load order, dirty-state tracking, and recovery semantics. |
| ObjectId / ObjectRevision / ModelRevision | [ ] | Current code uses stable IDs in several implementation slices, with import identity evidence in older milestones. | Every mutable product object has stable identity, per-object revision, and model revision semantics suitable for diff, undo/redo, proposal/apply, collaboration, and artifact provenance. |
| ComponentInstance | [ ] | Component mutation and replacement evidence exists for the imported-board write slice. | Component instances are first-class product objects spanning schematic, board, parts, packages, fields, placement, connectivity attachments, and import provenance. |
| Import Map `import_key` | [ ] | UUID v5 import identity and sidecar evidence exists for earlier import slices. | Imported objects carry durable `import_key` mappings that preserve source fidelity without making source-format identity the internal product identity. |
| OperationBatch | [ ] | Historical M3 mutation operations have determinism and undo/redo evidence. | Product edits are grouped as atomic, typed batches with validation, deterministic ordering, journal entries, revision updates, and surfaced results. |
| `commit()` / journal / recovery | [ ] | Save-backed mutation slices and round-trip evidence exist, but no active recovery contract is tracked here yet. | Commits persist operation batches through a journal with crash recovery, idempotency, replay, and clear failure boundaries. |
| proposal / apply | [ ] | Replacement plans and policies exist for current package/component operations. | All AI/tool-suggested changes can be proposed, inspected, checked, and applied through one substrate path with stable IDs and revision guards. |
| CheckRun / CheckFinding | [ ] | ERC/DRC/check reports exist across older engine, CLI, and MCP slices. | Checks produce first-class runs and findings with provenance, affected objects, waivers, revisions, severity policy, and artifact linkage. |
| ZoneFill | [ ] | Board import and DRC evidence exists for current board geometry subsets. | Zone fill is represented as a product operation/result with deterministic generated geometry, invalidation, check integration, and artifact provenance. |
| OutputJob / artifacts | [ ] | Some export/golden/test artifacts exist as evidence for historical gates. | Outputs are modeled as jobs with inputs, revisions, produced artifacts, logs, status, reproducibility metadata, and CLI/MCP/GUI visibility. |
| PTY terminal | [ ] | No active product terminal substrate is tracked in this file. | A PTY-backed terminal can run project-scoped jobs/commands with streamed output, cancellation, exit status, and artifact/job linkage. |
| CLI/MCP taxonomy `datum-eda` | [ ] | Current CLI/MCP surfaces are inventoried in `specs/SPEC_PARITY.md`; runtime MCP catalog and CLI command counts are locked by parity gates. | CLI and MCP expose one coherent `datum-eda` product taxonomy aligned to substrate nouns, not milestone-era command accumulation. |
| Spec parity / governance | [~] | `specs/SPEC_PARITY.md` and drift gates lock selected current inventories. | Governance defines which specs are source of truth, how target/current claims are separated, and which gates prevent stale milestone evidence from driving active scope. |

Open tracking rule:
- Add implementation evidence here before marking a substrate row complete.
- If a row is intentionally deferred, record the governance reason and the
  downstream fidelity work allowed despite the deferral.
- Do not promote legacy milestone completion to substrate readiness without a
  direct product-mechanics contract.

---

## Current Repo Health

Current repo health status (2026-03-25 audit):
- `cargo test -q` currently passes.
- The current `M2` implementation slice is backed by `m2_quality`,
  `m2_perf`, CLI tests, daemon tests, and MCP self-test.
- Milestone completion status and workspace health are currently reconciled.
- API/daemon/MCP test-support monolith risk has been reduced via module splits,
  and file-size budgets are now enforced in CI.

Current M7 delivery rule (2026-04-16):
- opening `M7` work may not advance on a "low resolution but technically
  implemented" basis
- user-facing slices must be intentionally triggerable, externally observable,
  and supported by the minimum interaction/render substrate they depend on
- missing prerequisite work in selection, hit-testing, focus/relatedness,
  visibility, or render-state consistency is not "scope creep"; it is
  prerequisite completion for the slice already being claimed
- working note:
  `docs/gui/M7_DELIVERY_GATES.md`

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

## Legacy Implementation Evidence

The following milestone tables preserve historical/current implementation
evidence. They are not the active North Star for the new scope. Use them to
ground factual current-state claims, then promote only substrate-relevant facts
into "Scope Integration / Substrate Readiness" above.

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
| Schematic connectivity: hierarchy, labels, power symbols | [~] | Union-find solver in `connectivity/mod.rs`; imported KiCad child-sheet hierarchical linking and basic bus/member expansion now feed existing net/query/check surfaces for the current supported subset, while wider import fidelity and advanced bus syntax remain open |
| Board connectivity: net-to-pin resolution | [x] | `board/mod.rs` net_pins(), net_pad_points() |
| Airwire computation: matches KiCad on DOA2526 | [~] | Algorithm implemented; DOA2526 validation not confirmed |
| Board diagnostics: net_without_copper, via_only, partially_routed | [x] | `board/mod.rs` diagnostics() |
| Golden tests: 8+ designs with checked-in golden files | [ ] | `tests/corpus/` is empty |
| Deterministic import: byte-identical on 3 runs | [~] | Current KiCad import/query corpus is now gated by `scripts/check_import_query_determinism.py` over checked-in query/check fixtures and passes 3-run stability for the current supported subset; broader import fidelity and non-query/save-backed import determinism remain open |
| Import fidelity KiCad ≥ 90% | [ ] | No fidelity matrix exists |
| Import fidelity Eagle ≥ 85% | [ ] | Eagle design import not implemented |

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

MCP tools: M2 slice 26/26 implemented. Current MCP runtime catalog: 75
methods (daemon-dispatched + CLI-bridged via `mcp-server/server_runtime.py`),
locked via `specs/SPEC_PARITY.md` → `mcp_runtime_methods`.

### CLI Commands (specs/PROGRAM_SPEC.md — M2)

> **Scope note.** The table below is the *M2 historical slice* — the eight
> commands M2 froze. It is **not** the current CLI surface. The present
> `tool project` surface is **182 commands**, locked via
> `specs/SPEC_PARITY.md` → `cli_project_commands`. For the authoritative,
> code-derived enumeration run
> `python3 scripts/check_spec_parity.py --print` and read the
> `[cli_project_commands]` block. Do not read this M2 table as today's reach.

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

Status: [~] Routing-kernel scope complete; closure review pending
- Authority for the proposed opening charter and entry criteria:
  `specs/progress/m5_opening.md`
- Recommended M5 focus: deterministic layout-kernel groundwork from persisted
  native board state, opening with one narrow routing/constraint slice rather
  than broad placement/routing ambition.
- Governance narrowing (2026-03-30):
  - for milestone closure, `M5` is now interpreted as the deterministic
    persisted-state routing-kernel milestone recorded in the opening charter
    and current frontier below
  - placement-kernel and placement/routing co-optimization work are deferred
    to a later reopened milestone/slice and are not required for `M5` closure
  - `M6` may open from the completed routing-kernel substrate once closure
    review accepts this narrowed milestone scope
- M5 must not inherit M4 parity work by default; new slices require an
  explicit layout-kernel contract and acceptance criteria.
- MCP implementation parity remains deferred unless explicitly reopened, but
  deferred parity tracking for newer native M4/M5 slices must stay current in
  `specs/MCP_API_SPEC.md`.
- A narrow MCP parity exception is now live for the policy-selected
  authored-copper-graph proposal export surface; broader M5 MCP query/apply
  parity remains deferred.
- That narrow exception now covers the full route-proposal artifact lifecycle
  for the policy-selected authored-copper-graph family: export, inspect, and
  apply.
- Current M5 checkpoint chain (routing-kernel read/query lane):
  - `project query <dir> routing-substrate`
  - `project query <dir> route-preflight --net <uuid>`
  - `project query <dir> route-corridor --net <uuid>`
  - canonical route query surface: `project query <dir> route-path-candidate --net <uuid> --from-anchor <pad_uuid> --to-anchor <pad_uuid> --candidate <accepted_candidate> [--policy <policy>]`
  - canonical route explanation surface: `project query <dir> route-path-candidate-explain --net <uuid> --from-anchor <pad_uuid> --to-anchor <pad_uuid> --candidate <accepted_candidate> [--policy <policy>]`
  - contract-specific `route-path-candidate-*` and
    `route-path-candidate-*-explain` commands are now compatibility wrappers
    around those bounded generic surfaces
  - the remaining legacy wrapper implementations now also dispatch through the
    same shared generic candidate/policy executor path as the canonical
    surfaces, reducing wrapper-only branching ahead of removal
- Current M5 routing expansion (persisted-via reuse lane):
  - fixed-arity via reuse was proven through bounded ordinal slices and is no
    longer the preferred growth model
  - current generalized contract is `project query <dir>
    route-path-candidate --net <uuid> --from-anchor <pad_uuid> --to-anchor
    <pad_uuid> --candidate route-path-candidate-authored-via-chain`
  - current adjacent observability surface is `project query <dir>
    route-path-candidate-explain --net <uuid> --from-anchor <pad_uuid>
    --to-anchor <pad_uuid> --candidate route-path-candidate-authored-via-chain`
  - a same-layer synthesis slice now also exists via `project query <dir>
    route-path-candidate --net <uuid> --from-anchor <pad_uuid> --to-anchor
    <pad_uuid> --candidate route-path-candidate-orthogonal-dogleg`, with the
    paired explanation surface under `route-path-candidate-explain`
  - that same same-layer synthesis lane now also covers `project query <dir>
    route-path-candidate --net <uuid> --from-anchor <pad_uuid> --to-anchor
    <pad_uuid> --candidate route-path-candidate-orthogonal-two-bend`, with the
    paired explanation surface under `route-path-candidate-explain`
  - that same same-layer synthesis lane now also covers `project query <dir>
    route-path-candidate --net <uuid> --from-anchor <pad_uuid> --to-anchor
    <pad_uuid> --candidate route-path-candidate-orthogonal-graph`, with the
    paired explanation surface under `route-path-candidate-explain`
  - a first bounded cross-layer graph-search lane now also exists via
    `project query <dir> route-path-candidate --net <uuid> --from-anchor
    <pad_uuid> --to-anchor <pad_uuid> --candidate
    route-path-candidate-orthogonal-graph-via`, with the paired explanation
    surface under `route-path-candidate-explain`
  - that same bounded cross-layer graph-search lane now also covers `project
    query <dir> route-path-candidate --net <uuid> --from-anchor <pad_uuid>
    --to-anchor <pad_uuid> --candidate
    route-path-candidate-orthogonal-graph-two-via`, with the paired
    explanation surface under `route-path-candidate-explain`
  - that same bounded cross-layer graph-search lane now also covers the
    remaining authored-via graph sequence via `project query <dir>
    route-path-candidate --net <uuid> --from-anchor <pad_uuid> --to-anchor
    <pad_uuid> --candidate
    route-path-candidate-orthogonal-graph-three-via|route-path-candidate-orthogonal-graph-four-via|route-path-candidate-orthogonal-graph-five-via|route-path-candidate-orthogonal-graph-six-via`,
    with the paired explanation surface under `route-path-candidate-explain`
  - the shared orthogonal graph selector now uses an explicit deterministic
    cost rule across that whole family: bend count ascending, then segment
    count ascending, then point-sequence coordinate ascending
  - the same orthogonal graph query and explanation reports now expose that
    selected path cost directly as bend/segment/point counts on the returned
    path or per-segment path data
  - the same orthogonal graph route proposal artifact lane now preserves the
    selected bend count in exported actions and exposes it again through
    artifact inspection/apply reporting
  - `export-route-path-proposal` now also returns the recorded
    orthogonal-graph layer-segment bend/point/track-action breakdown
  - the same-layer orthogonal-graph `route-path-candidate` and
    `route-path-candidate-explain` reports now also return `segment_evidence`
    so direct query output matches the artifact lane vocabulary
  - the same-layer orthogonal-graph candidate and explain surfaces now also
    share one internal graph-search spine, keeping the public reports
    unchanged while consolidating the preflight/search/segment-evidence path
  - the one-authored-via orthogonal-graph candidate and explain surfaces now
    also share one internal via-selection spine that reuses the same
    orthogonal-graph layer-search structure without changing the public
    reports
  - the two-authored-via orthogonal-graph candidate and explain surfaces now
    also share one internal pair-selection spine that reuses the same
    orthogonal-graph layer-search structure without changing the public
    reports
  - orthogonal-graph artifact apply now reports whether stale proposals
    drifted because candidate availability changed, the deterministic ranked
    winner changed, or same-rank geometry changed
  - `inspect-route-proposal-artifact` now also returns the recorded
    orthogonal-graph layer-segment bend/point/track-action breakdown
  - the same lane now also has `project revalidate-route-proposal-artifact
    <dir> --artifact <path>` so callers can read that drift classification
    and live/recorded path summaries without applying
  - that revalidation report now also carries segment-level orthogonal-graph
    evidence so stale proposals can show which layer-side segment changed and
    how its bend/point/track-action facts differ live
- Current M5 existing-copper readback lane:
  - deterministic authored-copper graph path queries now exist in
    increasingly filtered/readback-focused forms recorded in
    `specs/progress/m5_opening.md`
  - that same generalized query surface also covers the accepted
    authored-copper-graph family when `--candidate authored-copper-graph
    --policy <policy>` is selected
  - that same generalized explanation surface also covers the accepted
    authored-copper-graph family when `--candidate authored-copper-graph
    --policy <policy>` is selected
  - accepted bounded policies are `plain`, `zone_aware`, `obstacle_aware`,
    `zone_obstacle_aware`, `zone_obstacle_topology_aware`, and
    `zone_obstacle_topology_layer_balance_aware`
  - the plus-one-gap bridge is now reached canonically through `project query
    <dir> route-path-candidate --net <uuid> --from-anchor <pad_uuid>
    --to-anchor <pad_uuid> --candidate authored-copper-plus-one-gap`
  - a first write-capable bridge now exists for that accepted query contract:
    `project export-route-path-proposal <dir> --net <uuid> --from-anchor
    <pad_uuid> --to-anchor <pad_uuid> --candidate
    authored-copper-plus-one-gap --out <path>`,
    `project inspect-route-proposal-artifact <path>`, and
    `project apply-route-proposal-artifact <dir> --artifact <path>`
  - the same route-proposal artifact lane now also covers the accepted
    completed write-capable family, now canonically via `project
    export-route-path-proposal <dir> --net <uuid> --from-anchor <pad_uuid>
    --to-anchor <pad_uuid> --candidate <accepted_candidate> [--policy
    <policy>] --out <path>`, spanning the accepted single-layer, via-path, and
    authored-copper-graph policy-selected contracts
  - a native direct convenience apply surface now also exists for that same
    accepted single-layer path contract via `project route-apply <dir> --net
    <uuid> --from-anchor <pad_uuid> --to-anchor <pad_uuid> --candidate
    route-path-candidate`
  - a native direct convenience apply surface now also exists for that same
    accepted bounded single-via contract via `project route-apply <dir> --net
    <uuid> --from-anchor <pad_uuid> --to-anchor <pad_uuid> --candidate
    route-path-candidate-via`
  - a native direct convenience apply surface now also exists for that same
    accepted bounded two-via contract via `project route-apply <dir> --net
    <uuid> --from-anchor <pad_uuid> --to-anchor <pad_uuid> --candidate
    route-path-candidate-two-via`
  - a native direct convenience apply surface now also exists for that same
    accepted bounded three-via contract via `project route-apply <dir> --net
    <uuid> --from-anchor <pad_uuid> --to-anchor <pad_uuid> --candidate
    route-path-candidate-three-via`
  - the same direct convenience apply lane now also covers the remaining
    bounded ordinal via contracts via `project route-apply <dir> --net <uuid>
    --from-anchor <pad_uuid> --to-anchor <pad_uuid> --candidate
    route-path-candidate-four-via|route-path-candidate-five-via|route-path-candidate-six-via`
  - that same direct convenience apply lane now also covers the accepted
    deterministic authored-via-chain contract via `project route-apply <dir>
    --net <uuid> --from-anchor <pad_uuid> --to-anchor <pad_uuid> --candidate
    route-path-candidate-authored-via-chain`
  - that same direct convenience apply lane now also covers the accepted
    same-layer orthogonal dogleg contract via `project route-apply <dir> --net
    <uuid> --from-anchor <pad_uuid> --to-anchor <pad_uuid> --candidate
    route-path-candidate-orthogonal-dogleg`
  - that same direct convenience apply lane now also covers the accepted
    same-layer orthogonal two-bend contract via `project route-apply <dir> --net
    <uuid> --from-anchor <pad_uuid> --to-anchor <pad_uuid> --candidate
    route-path-candidate-orthogonal-two-bend`
  - that same direct convenience apply lane now also covers the accepted
    same-layer orthogonal graph contract via `project route-apply <dir> --net
    <uuid> --from-anchor <pad_uuid> --to-anchor <pad_uuid> --candidate
    route-path-candidate-orthogonal-graph`
  - that same direct convenience apply lane now also covers the accepted
    one-authored-via orthogonal graph contract via `project route-apply <dir>
    --net <uuid> --from-anchor <pad_uuid> --to-anchor <pad_uuid> --candidate
    route-path-candidate-orthogonal-graph-via`
  - that same direct convenience apply lane now also covers the accepted
    two-authored-via orthogonal graph contract via `project route-apply <dir>
    --net <uuid> --from-anchor <pad_uuid> --to-anchor <pad_uuid> --candidate
    route-path-candidate-orthogonal-graph-two-via`
  - that same direct convenience apply lane now also covers the remaining
    authored-via orthogonal graph sequence via `project route-apply <dir>
    --net <uuid> --from-anchor <pad_uuid> --to-anchor <pad_uuid> --candidate
    route-path-candidate-orthogonal-graph-three-via|route-path-candidate-orthogonal-graph-four-via|route-path-candidate-orthogonal-graph-five-via|route-path-candidate-orthogonal-graph-six-via`
  - a bounded convenience export surface now also exists for the completed
    write-capable route family via `project export-route-path-proposal <dir>
    --net <uuid> --from-anchor <pad_uuid> --to-anchor <pad_uuid> --candidate
    <accepted_candidate> [--policy <policy>] --out <path>`
  - `project route-apply` now parses `--candidate` from a bounded accepted
    value set instead of a free-form string, and enforces `--policy` only for
    `--candidate authored-copper-graph`
  - a native direct convenience apply surface now also exists for that same
    policy-selected family via `project route-apply <dir> --net <uuid>
    --from-anchor <pad_uuid> --to-anchor <pad_uuid> --candidate
    authored-copper-graph --policy <policy>`
  - the bounded convenience export surface is now exposed through MCP as
    `export_route_path_proposal`
  - the bounded direct route-apply surface is now exposed through MCP as
    `route_apply`
  - the matching generic artifact follow-up surfaces are now exposed through
    MCP as `inspect_route_proposal_artifact`,
    `revalidate_route_proposal_artifact`, and
    `apply_route_proposal_artifact`
  - `specs/PROGRESS.md` tracks only the checkpoint/frontier; detailed per-slice
    history stays in `specs/progress/m5_opening.md`
- Current M5 frontier:
  - deterministic persisted-state layout-kernel/routing queries continue under
    explicit contract selection from `specs/progress/m5_opening.md`
  - a bounded native route selector now exists via `project route-proposal
    <dir> --net <uuid> --from-anchor <pad_uuid> --to-anchor <pad_uuid>`,
    selecting the first successful family from the explicit accepted candidate
    order recorded in `specs/progress/m5_opening.md`
  - that same selector now also feeds a selected-proposal write lane via
    `project export-route-proposal <dir> --net <uuid> --from-anchor
    <pad_uuid> --to-anchor <pad_uuid> --out <path>` and
    `project route-apply-selected <dir> --net <uuid> --from-anchor
    <pad_uuid> --to-anchor <pad_uuid>`
  - that same selector now also has a bounded explanation surface via
    `project route-proposal-explain <dir> --net <uuid> --from-anchor
    <pad_uuid> --to-anchor <pad_uuid>`
  - that same selector now also accepts a bounded deterministic profile set
    via `--profile default|authored-copper-priority`; the default order is
    unchanged, while `authored-copper-priority` prepends the accepted
    authored-copper-graph policy family ahead of the existing default family
    order without introducing scoring
  - a narrow MCP parity reopening now covers the selector lane via
    `route_proposal`, `route_proposal_explain`, `export_route_proposal`, and
    `route_apply_selected`
  - the first artifact/export/apply write lane now covers:
    the accepted plus-one-gap bridge, the accepted single-layer
    `route-path-candidate` contract, and the accepted bounded single-via
    `route-path-candidate-via` contract, and the accepted bounded two-via
    `route-path-candidate-two-via` contract, and the remaining bounded
    `three`/`four`/`five`/`six`-via plus `authored-via-chain` contracts, and
    the accepted same-layer orthogonal dogleg contract, and the accepted
    same-layer orthogonal two-bend contract, and
    the accepted zone-aware and zone-obstacle-aware existing-copper graph
    reuse contracts, and the accepted topology-aware plus layer-balance-aware
    topology-aware zone-obstacle-aware existing-copper graph reuse contracts,
    and the accepted obstacle-aware existing-copper graph reuse contract, and
    the policy-selected authored-copper graph family over the accepted bounded
    policy set
  - new slices must still avoid broad autorouting semantics, invented
    constraints, and untracked MCP drift
  - closure interpretation:
    - this frontier is the intended `M5` closure target under the narrowed
      routing-kernel milestone definition above
    - remaining placement-kernel work is explicitly out of scope for `M5`
      closure

## M6 Opening Charter

Status: [~] Frozen pending evidence
- Authority for the proposed opening charter and entry criteria:
  `specs/progress/m6_opening.md`
- Recommended M6 focus: read-only deterministic strategy reporting layered on
  top of the completed M5 routing-kernel substrate.
- Current M6 checkpoint chain:
  - `project route-strategy-report <dir> --net <uuid> --from-anchor <pad_uuid> --to-anchor <pad_uuid> [--objective <objective>]`
  - `project route-strategy-compare <dir> --net <uuid> --from-anchor <pad_uuid> --to-anchor <pad_uuid>`
  - `project route-strategy-delta <dir> --net <uuid> --from-anchor <pad_uuid> --to-anchor <pad_uuid>`
  - `project write-route-strategy-curated-fixture-suite --out-dir <path> [--manifest <path>]`
  - `project capture-route-strategy-curated-baseline --out-dir <path> [--manifest <path>] [--result <path>]`
  - `project route-strategy-batch-evaluate --requests <path>`
  - `project inspect-route-strategy-batch-result <path>`
  - `project validate-route-strategy-batch-result <path>`
  - `project compare-route-strategy-batch-result <before> <after>`
  - `project gate-route-strategy-batch-result <before> <after> [--policy <policy>]`
  - `project summarize-route-strategy-batch-results [--dir <path> | --artifact <path> ...] [--baseline <path> --policy <policy>]`
  - accepted objective set currently reuses the selector profile vocabulary:
    - `default`
    - `authored-copper-priority`
  - the report maps the requested objective to one existing selector profile,
    explains that mapping, and includes the current live selector outcome under
    that profile without reopening M5 routing semantics
  - the comparison report evaluates that same accepted objective/profile set,
    reports the current live selector outcome for each entry, and recommends
    one profile under a deterministic baseline-preserving comparison rule
  - the decision-delta report reduces that same accepted objective/profile set
    to one bounded explicit delta classification plus one short material
    difference summary for the user
  - the batch evaluation surface runs the existing report/compare/delta
    surfaces over a versioned explicit request manifest and returns both
    per-request evidence and aggregate summary counts
  - one curated fixture-suite writer now materializes a deterministic native
    fixture set plus a compatible batch-request manifest for repeated evidence
    runs over real persisted projects
  - one curated baseline-capture surface now materializes that fixture suite,
    runs the existing batch evaluator, and saves one reusable versioned
    batch-result artifact with deterministic default paths
  - one checked-in repo baseline asset set now lives under
    `crates/test-harness/testdata/quality/route_strategy_curated_baseline_v1`
    and the alignment CI lane regenerates and strict-gates a fresh run against
    that baseline through `scripts/check_route_strategy_evidence.py`
  - the current curated suite covers:
    - same-outcome baseline route selection
    - profile divergence between `default` and
      `authored-copper-priority`
    - no-proposal-under-any-profile
    - one cross-layer routable same-outcome case
  - the batch-evaluate JSON output is now also the explicit saved result
    artifact format with `kind = native_route_strategy_batch_result_artifact`
    and `version = 1`
  - saved batch result artifacts can now be inspected and structurally
    validated through read-only surfaces that report summary/distribution
    details, per-request outcomes, malformed entries, version compatibility,
    required-field coverage, and deterministic summary/result integrity checks
  - saved batch result artifacts can now also be compared without live
    re-evaluation by `request_id`, reporting aggregate count deltas,
    added/removed/common request ids, common-request recommendation/delta/live
    outcome changes, and one bounded summary classification
  - saved batch result artifact comparisons can now also be evaluated as a
    read-only CI/review gate under the explicit accepted policy set
    `strict_identical|allow_aggregate_only|fail_on_recommendation_change`,
    reporting pass/fail reasons and count facts while returning CLI exit code
  - the current M6 implementation frontier is intentionally frozen pending
    repeated evidence from the checked-in baseline gate and curated fixture
    suite; new semantics are not the default next step

## M7 Opening Charter

Status: [~] Opened narrowly as a read-only route-proposal review layer
- Authority for the opening charter and entry criteria:
  `specs/progress/m7_opening.md`
- Concrete workspace/contract/spike definition:
  `specs/M7_FRONTEND_SPEC.md`
- Active imported-board fidelity execution plan:
  `docs/gui/M7_IMPORTED_BOARD_FIDELITY_PLAN.md`
- Active board-review fidelity diagnosis:
  `docs/gui/M7_BOARD_REVIEW_FIDELITY_GAP.md`
- Active M7 focus: one narrow visual review layer on top of the closed M5
  routing-kernel substrate and the frozen M6 strategy-reporting/evidence
  stack.
- Entry conditions satisfied before opening:
  - daemon result serialization no longer unwrap-panic on response encoding in
    `crates/engine-daemon/src/dispatch.rs`; serialization failures now return
    structured internal JSON-RPC errors instead of crashing the daemon
  - one remaining imported-connectivity runtime unwrap in
    `crates/engine/src/connectivity/mod.rs` was removed from the hierarchical
    link resolver path
  - `mcp-server/server_runtime.py` now generates the plain daemon-backed MCP
    request/call wrappers from one shared `DAEMON_CLIENT_METHOD_SPECS` table
    instead of duplicating those wrappers by hand across request builders and
    call helpers
  - `mcp-server/daemon_client_request_tests.py` now sanity-checks that the
    generated daemon client wrappers are installed and preserve required fixed
    parameter shapes such as `rotate_component`
  - governance/docs no longer treat `M5` as the active execution window; M5 is
    closed and M6 is frozen pending evidence
- Implemented opening slice:
  - new native CLI review surface:
    `project review-route-proposal <dir> --net <uuid> --from-anchor <pad_uuid> --to-anchor <pad_uuid> [--profile <profile>]`
  - the same command also supports saved-artifact review with:
    `project review-route-proposal --artifact <path>`
  - the review payload reuses the existing selected route-proposal and route
    proposal artifact data structures, exposing deterministic proposal actions,
    segment evidence, selected contract metadata, and source identity for one
    frontend-consumable review object
  - the same review surface is exposed through MCP as
    `review_route_proposal`
  - apply/export remain in the existing machine-native CLI/MCP surfaces; M7 is
    review-only in this opening slice

- Next accepted frontend slice for `M7`:
  - one fixed route-proposal review workspace defined in
    `specs/M7_FRONTEND_SPEC.md`
  - one deterministic `board_review_scene_v1` contract for the opening board
    review context
  - one locked viewport-centered three-column shell with bottom dock strip
  - one fixed panel taxonomy: `Project`, `Filters`, `Inspector`, `Review`
  - one single-selection read-only interaction model with a separate active
    review target
  - one fixed panel set plus integrated terminal and AI supporting lanes, all
    remaining subordinate to engine authority
  - one explicit authored/proposed/diagnostic visual-state model

- Accepted post-spike correction track inside opening `M7`:
  - one bounded imported-board fidelity program defined in
    `docs/gui/M7_IMPORTED_BOARD_FIDELITY_PLAN.md`
  - scope split:
    - import fidelity for KiCad PCB truth preservation
    - scene-contract fidelity for explicit authored / unrouted / proposed /
      diagnostic lanes
    - renderer fidelity for PCB-native semantic readability
  - sequencing rule:
    - execute after the architecture spike proves the
      `gui-app` / `gui-protocol` / `gui-render` boundary
    - execute before broadening imported-board review claims or generalizing
      the opening `M7` workflow beyond the current bounded review surface
  - current correction-track read:
    - Stage 3 unrouted-lane work is functionally landed on the canonical
      half-routed fixture
    - previously recorded Stage 1 / Stage 2 blockers around outline ownership,
      pad rotation, roundrect semantics, outline layer-id carriage, and outline
      visibility gating have materially landed in code; the fidelity docs are
      now being reconciled to that newer repo state
    - `M7-INT-001` closed its first slice (2026-06-09): authored-object
      selection ownership, switch-clears-prior, and hover-preview-only
      behavior are regression-locked in
      `crates/gui-render/tests/selection_ownership.rs`
    - `M7-REN-006` closed (2026-06-09): the render-stack policy now has a
      single code encoding (`RenderStage` declaration order, shared
      `POST_COPPER_STAGES` walk), copper-layer appearance is constructed
      material-first, the bounded exception set is documented at the
      retained-geometry pass header, and contract regression tests lock the
      declared stage ladder
    - `M7-REN-003` closed (2026-06-12): proposed overlays verified
      copper-like at world-true width in selected and non-selected states;
      the diagnostic per-vertex marker violation ("generic path nodes" over
      proposed copper) was removed and regression-locked
    - `M7-REN-004` closed (2026-06-12): filled-zone copper is a declared
      derived shade of the layer material so pad/teardrop/pour boundaries
      read correctly (teardrop tangency criterion verified on DOA2526), and
      dim-unrelated readability is verified on the canonical fixture with
      regression locks in `render_contract_tests.rs`
    - active next implementation frontier is `M7-REG-001..003` fixture-backed
      import/scene/visual regression coverage (owner-ordered 2026-06-11)
    - the renderer semantic contract note now lives in
      `docs/gui/M7_RENDER_SEMANTIC_CONTRACT.md`
  - standards amendment for the opening slice:
    - opening `M7` remains review-focused and does not expand into a full IPC
      authoring/validation milestone
    - standards-relevant imported observables already exposed in the review
      surface must preserve source truth
    - bounded import-audit diagnostics are in-scope where they report delta
      without mutating imported geometry

- Acceptance checks met for the opening slice:
  - deterministic repeated output on unchanged persisted state
  - no pool re-resolution or live import-session dependence
  - no invented routing geometry or inferred constraints beyond persisted facts
  - engine + CLI test coverage for sorted/deterministic extraction

- Acceptance checks required for the next frontend slice:
  - deterministic repeated `board_review_scene_v1` output on unchanged
    persisted state
  - stable authored and review companion identities on unchanged state
  - no frontend-owned semantic geometry or review inference beyond explicit
    machine contracts
  - explicit authored vs proposed vs diagnostic visual separation in the
    opening workspace
  - terminal and AI supporting lanes consume explicit selection/review context
    without becoming parallel design truth
  - the opening route review starts from the first proposal action in
    deterministic review order

- Acceptance checks required for the imported-board fidelity track:
  - supported KiCad PCB layer identities do not silently collapse to fallback
    copper layers
  - supported imported pads preserve the physical dimensions and shape
    semantics required for board review
  - unsupported imported-board cases fail explicitly or remain clearly bounded
    instead of silently producing materially wrong board meaning
  - the accepted fixture set can visually distinguish authored copper,
    unrouted connectivity, proposed overlay geometry, and board-context
    primitives reliably enough for a PCB user to trust the review
  - fixture-backed tests plus screenshot or image-based review cover the
    canonical half-routed board and the supporting imported-board edge cases

---

## Standards Audit Batch 1 — Spec Stubs Awaiting Implementation

Spec edits landed in commit `db98eff` (2026-04-17) per the apply order in
`docs/STANDARDS_AUDIT_BATCH_1_GUIDANCE.md`. This section tracks each stub
against its implementation status. Status semantics:
- `[x]` spec/doc text landed in the named anchor
- `[ ]` implementation work not started (no Rust type, no engine op, no
  pool storage, no MCP runtime entry, no importer/exporter, etc.)
- A row may carry both: `[x]` spec landed + `[ ]` implementation pending —
  shown as two columns

### Pass 0 — Standards Compliance Disposition Refresh

| Stub | Spec anchor | Spec | Impl |
|------|-------------|:----:|:----:|
| Domain 1 disposition refresh (STEP/IDF/IDX/EDMD/DXF prerequisites; Gerber X3 / IPC-2581C / IPC-D-356 / ODB++ contracts) | `specs/STANDARDS_COMPLIANCE_SPEC.md` §4.1 | [x] | [—] N/A (disposition text only) |
| Domain 2 disposition refresh (IBIS/Touchstone/SPICE attachment; encrypted-content policy) | `specs/STANDARDS_COMPLIANCE_SPEC.md` §4.2 | [x] | [—] N/A (disposition text only) |

### Pass 1 — Engine Schema Bedrock

| Stub | Spec anchor | Spec | Impl |
|------|-------------|:----:|:----:|
| `ModelRole`, `SpiceDialect`, `EncryptionScheme`, `ModelAttachment`, `ModelProvenance`, `ModelFormatMetadata` (D2-1) | `specs/ENGINE_SPEC.md` §1.1a | [x] | [ ] |
| `ModelFormat` enum, typed `Transform3D`, expanded `ModelRef`, `Package.body_height_nm` / `body_height_mounted_nm` (D1-2) | `specs/ENGINE_SPEC.md` §1.1a + §1.2 | [x] | [ ] |
| `StackupLayer` material fields (`dielectric_constant`, `loss_tangent`, `copper_weight_oz`, `roughness_um`, `material_name`) (D1-3) | `specs/ENGINE_SPEC.md` §1.3 | [x] | [ ] |
| `Net.controlled_impedance: Option<ImpedanceSpec>` and `ImpedanceSpec` (D1-4) | `specs/ENGINE_SPEC.md` §1.3 | [x] | [ ] (solver deferred) |
| `Part` extensions (`manufacturer_jep106`, `packaging_options`, `behavioural_models`, `thermal`, `supply_chain_offers`, `last_supply_chain_check`) plus `ThermalSpec`, `PackagingKind`, `PackagingOption`, `SupplyOffer` (D2-2) | `specs/ENGINE_SPEC.md` §1.2 | [x] | [ ] |
| `AttachModel` / `DetachModel` operations with `inverse()` reversibility (D2-4) | `specs/ENGINE_SPEC.md` §3 | [x] | [ ] (no op impl, no undo/redo wiring) |

### Pass 2 — Pool and Native Persistence

| Stub | Spec anchor | Spec | Impl |
|------|-------------|:----:|:----:|
| `pool/models/{ibis,spice,touchstone,ami,thermal}/` directory; `models` and `part_model_attachments` SQL index tables (D2-3) | `docs/POOL_ARCHITECTURE.md` §2 | [x] | [ ] (no pool storage, no SQL tables) |
| `pool/models/` in native project layout; new "Pool Model Files" schema (D2-9) | `specs/NATIVE_FORMAT_SPEC.md` §4 + §6.x | [x] | [ ] |

### Pass 3 — MCP API Stubs

| Stub | Spec anchor | Spec | Impl |
|------|-------------|:----:|:----:|
| M7+ Export Tools: `export_step`, `export_idf`, `export_odbpp`, `export_ipc2581`, `import_dxf_outline` (D1-5) | `specs/MCP_API_SPEC.md` "M7+ Export Tools" | [x] | [ ] (not in `tools_catalog_data.py`; would need exporter implementations) |
| Component Modelling Tools: `attach_ibis`, `attach_touchstone`, `attach_spice`, `validate_*`, `extract_*`, `export_spice_netlist`, `lookup_part_*`, `refresh_supply_chain`, `find_alternate_parts`, `query_packaging_options`, `normalize_manufacturer`, `infer_diffpair_from_pinnames` (D2-5) | `specs/MCP_API_SPEC.md` "Component Modelling Tools (M7+)" | [x] | [ ] (not in `tools_catalog_data.py`) |
| Encrypted Content Handling Policy (D2-6) | `specs/MCP_API_SPEC.md` top-level | [x] | [—] N/A (policy framing) |

### Pass 4 — Import Spec

| Stub | Spec anchor | Spec | Impl |
|------|-------------|:----:|:----:|
| IPC-2581 Import (Future — Post-M7) rationale + feature matrix (D1-7) | `specs/IMPORT_SPEC.md` §5 | [x] | [ ] (importer deferred) |
| KiCad and Eagle import matrices: SPICE/IBIS/Touchstone rows promoted Deferred → Best-effort (M7+) with `Part.behavioural_models` mapping note (D2-8) | `specs/IMPORT_SPEC.md` §3 + §4 | [x] | [ ] (importer deferred) |

### Pass 5 — Architecture and Scope Docs

| Stub | Spec anchor | Spec | Impl |
|------|-------------|:----:|:----:|
| Behavioural model attachment subsection (D2-7) | `docs/LIBRARY_ARCHITECTURE.md` | [x] | [—] N/A (architecture text) |
| Interop scope re-organisation — Hard/Should/On-demand/Out-of-scope per Domain 1 (D1-1) | `docs/INTEROP_SCOPE.md` §Future (M5+) | [x] | [—] N/A (scope text) |
| Behavioural model attachment & export scope buckets (D2-10) | `docs/INTEROP_SCOPE.md` new section | [x] | [—] N/A (scope text) |

### Deferred — Held For A Later Batch

| # | Spec target | Why deferred |
|---|-------------|--------------|
| D1-6 | `specs/NATIVE_FORMAT_SPEC.md` §12 or `docs/POOL_ARCHITECTURE.md` | `.gitignore` / `.gitattributes` conventions — descriptive, no contract impact |
| D1-8 | `docs/COMMERCIAL_INTEROP_STRATEGY.md` §10 | "Datum's Open-Stack Position" appendix — marketing position |
| D2-11 | `docs/COMMERCIAL_INTEROP_STRATEGY.md` | "Behavioural Model Stack — Open-Stack Position" appendix — marketing position |

**Standards Audit Batch 1 overall**: [x] spec/doc/policy stubs landed; [ ]
implementation work not started for any of the bedrock types, operations,
pool persistence, native-format support, MCP runtime entries, or importer/
exporter handlers.

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
| No duplicate UUIDs within type | [x] | `project validate` checks persisted UUID-key consistency and duplicate authored UUIDs across native project object types |
| Non-dangling references | [x] | `project validate` checks required native files plus persisted schematic/board cross-file references and board-side object references |
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
| WaiverTarget variants | [x] | Object, RuleObject, and RuleObjects matching exercised across ERC/DRC tests |
| Waiver matching in ERC | [x] | Includes authored native-project waivers in existing project ERC/check flows |
| Waiver matching in DRC | [x] | `run_drc` now applies authored DRC waivers and keeps waived findings visible |
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
| Bus/member expansion | [x] | Basic imported KiCad subset now supports deterministic `NAME[n]` scalar member normalization, `NAME[a..b]` bus-range expansion, and geometric bus-entry association; advanced syntax edge cases remain deferred |
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
| Rust workspace (7 crates) | [x] | engine, cli, engine-daemon, test-harness, gui-protocol, gui-render, gui-app (locked via `specs/SPEC_PARITY.md` → `workspace_crates`) |
| Engine compiles without GUI deps | [x] | |
| Test harness (golden file utilities) | [x] | test-harness crate |
| Test corpus (real designs) | [ ] | tests/corpus/ empty |
| Daemon JSON-RPC dispatch | [x] | 53 methods in `dispatch.rs`, with coverage in daemon tests |
| Daemon socket transport | [x] | `main()` parses `--socket` and serves Unix socket; live smoke is environment-gated because sandboxed local runs deny socket IPC |
| MCP Python server (tool host) | [x] | Tool definitions + stdio dispatch |
| MCP→daemon transport | [x] | `EngineDaemonClient.call()` uses Unix socket JSON-RPC; behavioral parity remains covered separately from live socket smoke |
| Git repository initialized | [x] | `main` branch with GitHub remote configured |
| CI pipeline | [x] | `.github/workflows/alignment.yml` runs alignment and file-size budget checks |
