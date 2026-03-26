# Program Specification

## Ownership and Precedence

This document is the master entry point for the formal specification set.
It defines:
- product scope
- milestone sequencing
- milestone exit criteria
- non-goals
- subordinate spec ownership

The subordinate formal specifications own their respective contracts:
- `specs/ENGINE_SPEC.md`: core types, invariants, operations, serialization,
  and engine API
- `specs/IMPORT_SPEC.md`: source-format mapping, import identity, `.ids.json`,
  and fidelity/lossiness rules
- `specs/SCHEMATIC_CONNECTIVITY_SPEC.md`: schematic net resolution, hierarchy,
  buses, diagnostics, and deterministic connectivity recomputation
- `specs/ERC_SPEC.md`: electrical semantics, ERC rule set, and ERC result model
- `specs/CHECKING_ARCHITECTURE_SPEC.md`: shared checking domains, waiver model,
  and shared reporting rules
- `specs/SCHEMATIC_EDITOR_SPEC.md`: authored schematic model, editing semantics,
  and schematic operation coverage
- `specs/MCP_API_SPEC.md`: transport/session model, method catalog, payload
  schemas, and error contracts
- `specs/NATIVE_FORMAT_SPEC.md`: native project persistence, schema layout,
  versioning, and migration rules

Precedence rules:
- If a contract appears in both `docs/` and `specs/`, `specs/` wins.
- `docs/` exists to capture rationale, tradeoffs, and design context.
- Current explicit exception: `docs/CANONICAL_IR.md` remains canonical until it
  is promoted or subsumed by formal spec files.

## Scope Integrity Terms

The following terms are normative for scope framing across this repository:

- **Product identity**: Datum EDA as an AI-native EDA platform.
  This is cross-milestone and does not shrink to current feature coverage.
- **Implementation slice**: the currently shipped capability subset at a given
  milestone boundary (for example `M2` import/query/check, current `M3` write
  subset).
- **Execution strategy**: sequencing choices used to deliver the implementation
  slice with low risk (for example KiCad-first interoperability).
- **Non-goals**: explicit exclusions for a milestone, used to control delivery
  scope. Non-goals for one milestone are not permanent product exclusions.

Interpretation rule:
- Do not infer product identity from implementation-slice limits.
- Do not infer permanent product limits from milestone non-goals.

## v1 Definition

v1 = PLAN.md milestone M2.
v1 is the first public release.

**Product identity (all milestones)**: Datum EDA is an AI-native EDA platform
with a deterministic engine core and machine-native control surfaces.

**v1 implementation slice**: the best AI/CLI design analysis and automation
environment for Linux PCB projects.

**v1 does**: import KiCad/Eagle designs, query all design data, run ERC/DRC,
and expose that surface via MCP and CLI.

**v1 does not**: create designs, edit designs, route, export manufacturing
files, or provide a GUI.

Note: KiCad/Eagle-first support in v1 is an execution strategy for rapid,
verifiable delivery. It is not the long-term product boundary.

---

## Milestone Exit Criteria

### M0: Canonical IR + Foundation

**Exit gate**: `cargo test` passes, all items below verified.

| Criterion | Threshold |
|-----------|-----------|
| Pool types implemented | Unit, Pin, Entity, Gate, Package, Pad, Padstack, Part, Symbol |
| Pool serialization | Round-trip: struct → JSON → struct == original, 100% of types |
| Pool SQLite index | Create, insert, search by keyword, search by parametric value |
| Deterministic serialization | Byte-identical output on 3 consecutive runs |
| Eagle .lbr import | 20 libraries parse without error |
| Eagle canonicalization | Import `.lbr` → canonical pool JSON → deserialize == original canonical pool, 20 libraries |
| Eagle deterministic re-import | Same `.lbr` imported twice → identical canonical pool objects and UUIDs |
| RuleScope IR | Expression tree enum compiles, serializes, leaf evaluator works |
| UUID v5 import identity | Same .lbr imported twice → identical UUIDs |
| .ids.json sidecar | Written on import, restored on re-import |

**Non-goals for M0**: board data model, schematic data model, ERC/DRC, CLI,
MCP, Eagle `.lbr` write-back.

### M1: Design Ingestion + Query Engine

**Exit gate**: DOA2526 imports and queries correctly. Golden tests pass.

**Current implementation status**:
- KiCad-first import is active and ahead of the original `M1` floor:
  `.kicad_pro`, `.kicad_sch`, and `.kicad_pcb` import slices exist.
- Read-only board and schematic query surfaces already exist in the engine,
  CLI, daemon, and MCP stub for summaries, net info, components, labels,
  symbols, ports, buses, hierarchy, no-connects, diagnostics, and unified
  check reporting.
- Board-side `M1` is no longer just object counting:
  imported footprint pads exist in canonical board state, board net pins are
  derived from those pads, `get_unrouted` computes importer-backed airwires,
  and board diagnostics already distinguish empty-copper, via-only, and
  partially-routed nets.
- Early schematic connectivity and ERC prechecks already exist, but `M1`
  still closes on deterministic ingestion/query correctness, not on the
  partial checking surface.

| Criterion | Threshold |
|-----------|-----------|
| KiCad .kicad_pcb import | DOA2526 + 4 additional designs, 0 import errors |
| KiCad .kicad_sch import | DOA2526 + 2 additional schematics, 0 import errors |
| Eagle .brd/.sch import | 3 designs, 0 import errors |
| Schematic connectivity | Net resolution correct on imported schematics with hierarchy, labels, and power symbols |
| Board connectivity | Net-to-pin resolution correct on all imported boards |
| Airwire computation | Matches KiCad's unrouted count on DOA2526 |
| Board diagnostics | `net_without_copper`, `via_only_net`, and `partially_routed_net` classified correctly on DOA2526 and companion fixtures |
| Query API | get_netlist, get_components, get_nets, get_net_info, get_schematic_net_info, get_board_summary, get_schematic_summary, get_sheets, get_symbols, get_ports, get_labels, get_buses, get_bus_entries, get_noconnects, get_hierarchy, get_connectivity_diagnostics, get_unrouted — all return correct data on DOA2526 |
| Golden tests | 8+ designs with checked-in golden files |
| Deterministic import | Same file → byte-identical canonical JSON on 3 runs |
| Import fidelity (KiCad) | ≥ 90% features passing on fidelity matrix |
| Import fidelity (Eagle) | ≥ 85% features passing on fidelity matrix |

**Non-goals for M1**: full ERC corpus signoff, full DRC engine, write
operations, export, native format, GUI.

### M2: ERC + DRC + Reporting + MCP/CLI (v1)

**Exit gate**: An MCP-compatible AI client can open DOA2526 via MCP, query it,
run ERC/DRC.
`tool erc design.kicad_sch` and `tool drc board.kicad_pcb` work in CI.

**Current implementation status**:
- The transport and consumer surfaces are already in place ahead of full `M2`:
  CLI read/check commands exist, the engine daemon exposes JSON-RPC methods,
  and the Python MCP stdio host already proxies the current read/check surface.
- Unified `CheckReport`, raw connectivity diagnostics, and raw ERC findings
  already exist.
- The currently implemented MCP/daemon method subset is:
  `open_project`, `close_project`, `search_pool`, `get_part`, `get_package`,
  `get_package_change_candidates`, `get_part_change_candidates`,
  `get_component_replacement_plan`, `get_scoped_component_replacement_plan`,
  `edit_scoped_component_replacement_plan`,
  `replace_components`,
  `apply_component_replacement_plan`,
  `apply_component_replacement_policy`,
  `apply_scoped_component_replacement_policy`,
  `apply_scoped_component_replacement_plan`,
  `get_board_summary`, `get_components`, `get_netlist`,
  `get_schematic_summary`, `get_sheets`, `get_labels`, `get_symbols`,
  `get_symbol_fields`, `get_ports`, `get_buses`, `get_bus_entries`,
  `get_noconnects`, `get_hierarchy`, `get_net_info`,
  `get_schematic_net_info`, `get_unrouted`, `get_connectivity_diagnostics`,
  `get_design_rules`, `get_check_report`, `run_erc`, `run_drc`,
  `explain_violation`.
- The current `M2` implementation slice now covers the full M2 check/query
  tool catalog and quality/performance gates tracked in `specs/PROGRESS.md`.
- The controlling contract split for MCP wire schemas is in
  `specs/MCP_API_SPEC.md` (`Current implementation contract` vs `Target M2`).

| Criterion | Threshold |
|-----------|-----------|
| ERC rules implemented | output_to_output_conflict, undriven_input, power_without_source, noconnect_connected, unconnected_required_pin, passive_only_net, hierarchical_connectivity_mismatch |
| ERC correctness | 0% false negative rate on known ERC violations in test corpus |
| ERC correctness | ≤ 5% false positive rate on vetted clean schematics in test corpus |
| DRC rules implemented | clearance_copper, track_width, via_hole, via_annular, connectivity, unconnected_pins, silk_clearance |
| DRC correctness | ≤ 5% false positive rate vs. KiCad DRC on DOA2526 |
| DRC correctness | 0% false negative rate on known violations in test corpus |
| MCP tools | open_project, close_project, search_pool, get_part, get_package, get_package_change_candidates, get_part_change_candidates, get_component_replacement_plan, get_scoped_component_replacement_plan, edit_scoped_component_replacement_plan, replace_components, apply_component_replacement_plan, apply_component_replacement_policy, apply_scoped_component_replacement_policy, apply_scoped_component_replacement_plan, get_board_summary, get_schematic_summary, get_sheets, get_symbols, get_ports, get_labels, get_buses, get_bus_entries, get_noconnects, get_symbol_fields, get_hierarchy, get_netlist, get_components, get_net_info, get_schematic_net_info, get_connectivity_diagnostics, get_design_rules, get_unrouted, run_erc, run_drc, explain_violation |
| CLI commands | `tool import`, `tool query`, `tool erc`, `tool drc`, `tool pool search` |
| MCP registration | Works in host MCP client config; tools callable via a standards-compliant MCP client |
| Test corpus | 10+ designs with ERC/DRC golden files |
| CLI exit codes | 0 = pass, 1 = violations, 2 = error |
| Response time | DRC on DOA2526 < 5 seconds |
| Response time | ERC on DOA2526 schematic < 3 seconds |

**Non-goals for M2**: write operations, export, native format, GUI.

### R1: Commercial Interop Research Track

This is a post-`M2` research stream, not a delivery milestone and not a
support commitment. Its purpose is to prepare future migration paths from
commercial Windows-only EDA tools without distorting the `M0-M2` foundation.

| Criterion | Threshold |
|-----------|-----------|
| Corpus gathered | Representative Altium, PADS, and OrCAD/Allegro sample set assembled |
| Format analysis | Native, exported, and tool-assisted ingestion paths documented per target |
| Legal posture | Licensing/reverse-engineering posture documented for each proposed path |
| Migration rubric | Exact / approximated / preserved-as-metadata / unsupported categories defined |
| Library extraction prototypes | At least one commercial target evaluated at library/component level |
| Recommendation | First supported commercial target and ingestion strategy selected |

**Non-goals for R1**: shipping commercial import support, write-back, fidelity guarantees, CLI or MCP surface changes.

### M3: Write Operations on Imported Designs

**Exit gate**: AI agent can move components and save back to KiCad format.

Imported schematic editing is intentionally deferred. `M3` remains board-side
write-back on imported designs only. Schematic parity is achieved by making the
native schematic editor in `M4` equally specified and equally scriptable, not
by forcing fragile imported-schematic write-back into `M3`.

| Criterion | Threshold |
|-----------|-----------|
| Operations implemented | MoveComponent, RotateComponent, DeleteComponent, SetValue, SetReference, AssignPart, SetPackage, SetPackageWithPart, SetNetClass, SetDesignRule, DeleteTrack, DeleteVia |
| Undo/redo | 100% of operations undoable |
| Operation determinism | Same operation sequence → identical result |
| KiCad write-back | Import → modify → export → opens in KiCad without errors |
| Round-trip fidelity | Unmodified objects byte-identical after round-trip |
| MCP write tools | move_component, rotate_component, set_value, set_reference, assign_part, set_package, set_package_with_part, set_net_class, set_design_rule, delete_component, delete_track, delete_via, undo, redo, save |
| Derived data update | Connectivity, airwires, DRC recompute after each operation |
| CLI modify command | `tool modify <design> --move U1 50,30` works |

**Non-goals for M3**: imported schematic editing, native format, routing, export.

### M4: Native Project Creation + Export

**Exit gate**: Create a design from schematic to Gerber without a GUI.

| Criterion | Threshold |
|-----------|-----------|
| Native format | JSON, schema documented, versioned |
| Schematic operations | PlaceSymbol, MoveSymbol, RotateSymbol, MirrorSymbol, DeleteSymbol, DrawWire, DeleteWire, PlaceJunction, DeleteJunction, PlaceLabel, RenameLabel, DeleteLabel, PlacePowerSymbol, CreateBus, EditBusMembers, PlaceBusEntry, DeleteBusEntry, CreateSheetInstance, MoveSheetInstance, DeleteSheetInstance, PlaceHierarchicalPort, EditHierarchicalPort, DeleteHierarchicalPort, PlaceNoConnect, DeleteNoConnect, SetFieldValue, MoveField, SetFieldVisibility, Annotate, AssignPart, AssignGate |
| Board operations | PlaceComponent, AddTrack, AddVia, AddZone, PourCopper, AddKeepout |
| Schematic query parity | Labels, buses, bus entries, no-connects, hierarchy, field queries available through engine API, CLI, and MCP |
| Forward annotation | ECO with per-change accept/reject |
| Gerber export | RS-274X, validates in gerbv without warnings |
| Drill export | Excellon, validates in gerbv |
| BOM export | CSV and JSON |
| PnP export | CSV |
| Gerber comparison | Matches KiCad Gerber output on DOA2526 (layer alignment, aperture correctness) |

**Non-goals for M4**: placement solver, routing solver, GUI, 3D.

Variants are deferred beyond `M4` except for canonical data-model ownership.
The foundation freezes variants as project-level logical-component fit choices;
editing, UI, and export behavior for variants are specified in a later
milestone.

---

## User Stories

### M2 (v1) User Stories
- As an EE, I run `tool erc design.kicad_sch` in CI and the build fails on electrical mistakes.
- As an EE, I run `tool drc board.kicad_pcb` in CI and the build fails on clearance violations.
- As an EE, I ask my MCP-compatible AI client "what nets are in this design?" and get a structured answer via MCP.
- As an EE, I ask my MCP-compatible AI client "run ERC and tell me what is undriven" and get a human-readable explanation.
- As an EE, I ask my MCP-compatible AI client "run DRC and explain the worst violation" and get a human-readable explanation.
- As an EE, I search the pool for "100nF 0402" and get matching parts with MPNs.
- As an EE, I import an Eagle design and query its component list.

### M3 User Stories
- As an EE, I ask my MCP-compatible AI client "move U1 to (25, 15) and rotate 90 degrees" and it modifies my KiCad design.
- As an EE, I ask my MCP-compatible AI client "change all 10k resistors to 4.7k" and it does a batch modification.
- As an EE, I undo the last change via CLI.
- As an EE, I save modifications back to .kicad_pcb and open in KiCad.

### M4 User Stories
- As an EE, I create a new project from the command line.
- As an EE, I build a schematic by placing symbols and wiring via CLI/MCP.
- As an EE, I create multi-sheet hierarchy, ports, buses, and no-connect markers from CLI/MCP.
- As an EE, I edit symbol fields, assign gates on multi-gate devices, and annotate deterministically.
- As an EE, I inspect labels, buses, hierarchy links, symbol fields, and connectivity diagnostics through the same MCP surface I use for board data.
- As an EE, I forward-annotate to create a board, review the ECO, and accept changes.
- As an EE, I export Gerber files and submit to JLCPCB.
