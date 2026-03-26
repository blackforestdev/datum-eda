# Development Plan

## Mission Layers

**Product wedge**: Best AI/CLI design analysis and automation environment
for Linux PCB projects.

**Core platform**: Canonical design IR plus operation engine with import,
query, ERC/DRC, transformation, and export.

**Full CAD ambition**: Native editing, interactive routing, GUI, advanced
constraints. This is the trajectory, not the launch.

---

## Milestones

### M0: Canonical IR + Foundation
**Goal**: The data model exists, serializes deterministically, survives
canonical round-trip, and has tests. Import harness scaffolded.

**Progress (2026-03-25)**:
- [x] Milestone complete (`M0` closed)
- [ ] Project name selected

- [ ] Choose project name
- [ ] Initialize Rust workspace (engine lib, cli bin, workspace Cargo.toml)
- [ ] Define canonical IR (see docs/CANONICAL_IR.md):
  - Stable identity strategy (UUID generation, reference semantics)
  - Unit/precision model (nanometers internally, user-facing unit conversion)
  - Authored vs. derived data boundary
  - Transaction/operation envelope
  - Deterministic serialization (sorted keys, stable float repr)
- [ ] Implement pool foundation:
  - Unit, Pin (with direction, swap group, alternates)
  - Entity, Gate
  - Package, Pad, Padstack
  - Part (entity + package + pad_map + attributes + parametric)
  - Symbol (graphics primitives)
- [ ] SQLite pool index (create, insert, query, search)
- [ ] JSON serialization with round-trip golden tests
- [ ] Import harness scaffold (trait-based, format-pluggable)
- [ ] Eagle .lbr XML parser (validate against 300+ shipped libraries)
- [ ] Test corpus: minimum 20 Eagle libraries import into the pool without
  error and canonicalize deterministically
- [ ] Deterministic Eagle re-import: same `.lbr` imported twice yields
  identical canonical pool objects and UUIDs
- [ ] Explicitly import-only Eagle library support in `M0` (no `.lbr`
  write-back)

**Deliverable**: `cargo test` passes. Pool can ingest Eagle libraries,
store them, query them, and serialize canonical pool data deterministically.

### M1: Design Ingestion + Query Engine
**Goal**: Import real KiCad and Eagle designs. Query everything about them.
Golden tests against known-good designs.

**Progress (2026-03-25)**:
- [x] KiCad `.kicad_pcb` and `.kicad_sch` import slices implemented
- [x] Board and schematic query/read surfaces implemented in engine
- [x] Board/schematic connectivity diagnostics and airwire reporting implemented
- [x] Daemon + MCP read/check transport slices wired for current surface
- [ ] Eagle `.brd` / `.sch` import implementation (still unimplemented)
- [ ] Corpus/fidelity exit gates fully proven at `M1` spec threshold

- [ ] Board data model:
  - Layers, stackup (copper + dielectric, no solver yet)
  - Placed components (part reference + position + rotation + layer)
  - Tracks, vias, zones/pours (authored geometry)
  - Nets, net classes
  - Keepouts, dimensions, text
- [ ] Schematic data model:
  - Sheets, placed symbols, wires, junctions, labels
  - Net segments, buses, power symbols
  - Hierarchical blocks (sub-sheets)
- [ ] Schematic connectivity engine:
  - Hierarchy-aware net resolution across sheets and ports
  - Global labels, local labels, and power symbol propagation
  - Bus/member expansion into scalar nets
  - Pin-to-net attachment graph for ERC
- [ ] Connectivity engine:
  - Net-to-pin resolution
  - Board connectivity (track/via/pour graph)
  - Airwire computation (unrouted connections)
  - Incremental recomputation on change
- [ ] Import: KiCad .kicad_pcb + .kicad_sch parser
  - Target: DOA2526 imports without errors
  - Golden test: re-export and diff against source
- [ ] Import: Eagle .brd + .sch XML parser (using DTD from research)
  - Scope: bounded migration support, secondary to KiCad
- [ ] Query API:
  - get_netlist, get_components, get_nets, get_net_info
  - get_board_summary (dimensions, layer count, component count)
  - get_unrouted (airwire list)
  - search_pool (parametric, keyword, tag)
- [ ] Golden test corpus: minimum 10 real designs (mix of KiCad + Eagle)
  that import, query, and serialize deterministically

**Deliverable**: The engine can open DOA2526 (KiCad) and a bounded supported
subset of Eagle designs, answer arbitrary queries about them, and serialize
the result identically on every run. KiCad correctness takes priority.

### M2: ERC + DRC + Reporting + MCP/CLI
**Goal**: Electrical and physical rule checking work. AI agents can query and
check designs. CLI is useful for CI/CD pipelines.

**Progress (2026-03-25)**:
- [x] ERC precheck engine running with config + waiver support
- [x] Unified `CheckReport` shape implemented across engine/daemon/MCP/CLI
- [x] CLI supports `erc` and `check` reporting for current implemented checks
- [x] Initial DRC slice implemented (connectivity + clearance) in engine/daemon/MCP/CLI
- [x] DOA2526 ERC/DRC performance harness + baseline added in `crates/test-harness` (`m2_perf`)
- [x] M2 quality harness added for corpus + FP/FN gate measurement (`m2_quality`)
- [x] M2 quality corpus gate closed at 10/10 unique golden-backed designs (`m2_quality --json` pass)
- [x] ERC `noconnect_connected` implemented + importer pin-binding for `no_connect` markers
- [x] ERC `hierarchical_connectivity_mismatch` implemented (sheet-level hierarchical label/port mismatch check + fixture coverage)
- [x] ERC/DRC golden corpus expanded for implemented check coverage (including clearance fixture)
- [x] Added DRC checks for `track_width`, `via_hole`, `via_annular`, and unconnected single-pin nets
- [x] Added DRC `silk_clearance` check with fixture-backed golden coverage
- [x] Added MCP/daemon/query parity for `close_project`, `search_pool`, and `get_design_rules` (with Python MCP tool registration + self-test coverage)
- [x] Added MCP/daemon/query parity for `get_netlist` with deterministic board/schematic payload shape and cross-layer tests
- [x] Added MCP/daemon/query parity for `get_symbol_fields` (UUID-targeted lookup with not-found behavior and MCP registration/tests)
- [x] Added MCP/daemon/query parity for `get_bus_entries` (all-sheets current form + registration/tests)
- [x] Added MCP/daemon/pool parity for `get_part` and `get_package` (UUID lookup + pool detail payloads + tests)
- [x] Added MCP/daemon/checking parity for `explain_violation` (ERC/DRC domain + index explanation payload)
- [x] Fixed CLI check exit semantics so violation thresholds return code `1` (not execution-error `2`)
- [x] Registered `datum-eda` MCP server in `~/.claude/settings.json` with explicit daemon socket env
- [x] Full ERC rule-set parity with all `M2` exit-gate checks for the current M2 implementation slice
- [x] Full DRC rule-set parity with all `M2` exit-gate checks
- [x] Full CLI/MCP `M2` catalog parity for the current M2 surface

- [ ] ERC foundation:
  - Pin electrical semantics (input, output, passive, power_in, power_out,
    bidirectional, tri_state, no_connect, etc.)
  - Net driving analysis on schematic connectivity graph
  - Waiver/suppression model as authored data
- [ ] ERC checks:
  - Output-to-output conflict
  - Undriven input
  - Power pin with no valid source
  - no_connect pin actually connected
  - Unconnected required pin
  - Passive-only net warning
  - Hierarchical port / sheet connectivity mismatch
- [ ] Rule engine:
  - Rule definition (clearance, width, via, hole size, connectivity)
  - Rule scoping: expression-based IR from day one (see docs/CANONICAL_IR.md)
    M2 evaluator supports leaf nodes: All, Net, NetClass, Layer
    Later milestones add combinators: And, Or, Not, InComponent, etc.
    The data model never migrates — only the evaluator expands.
  - Priority ordering with first-match-wins
- [ ] DRC checks:
  - Copper-to-copper clearance
  - Track width vs. rule
  - Via drill/annular ring vs. rule
  - Board connectivity (all nets routed?)
  - Unconnected pins
  - Silk-to-copper clearance
- [ ] DRC reporting:
  - Structured results (violation type, location, objects, rule, severity)
  - Human-readable summary
  - JSON output for programmatic consumption
- [ ] ERC reporting:
  - Structured results (violation type, schematic location, objects, severity)
  - Human-readable summary
  - JSON output for programmatic consumption
- [ ] MCP server:
  - Pool tools: search_parts, get_part, get_package
  - Query tools: get_netlist, get_components, get_nets, get_board_summary
  - Checking tools: run_erc, run_drc, get_violations, explain_violation
  - Register in ~/.claude/settings.json
- [ ] CLI:
  - `tool import <file>` — import KiCad/Eagle design
  - `tool query <design> --nets|--components|--summary`
  - `tool erc <design>` — run ERC, exit code reflects pass/fail
  - `tool drc <design>` — run DRC, exit code reflects pass/fail
  - `tool pool search <query>` — search pool
- [ ] Test: Claude can open DOA2526 via MCP, query it, run ERC/DRC, explain results

**Deliverable**: Useful. A CI pipeline can run `tool erc design.kicad_sch`
or `tool drc board.kicad_pcb` and fail the build on violations. Claude can
inspect and check any design via MCP.

### R1: Commercial Interop Research Track
**Goal**: Prepare a credible migration path for users coming from Altium,
PADS, and OrCAD/Allegro without destabilizing the `M0-M2` delivery path.

This is a research and architecture track, not a supported user-facing
feature milestone. It starts after `M2` and runs in parallel with later
implementation milestones as capacity allows.

**Progress (2026-03-25)**:
- [ ] Not started (intentionally gated after `M2`)

- [ ] Gather representative commercial-tool corpus:
  - Altium libraries and projects
  - PADS libraries and projects
  - OrCAD/Allegro libraries and projects
- [ ] Document real-world format access patterns:
  - native files
  - exportable interchange artifacts
  - database-backed or tool-assisted extraction paths
- [ ] Establish legal/licensing posture for each ingestion path
- [ ] Define fidelity rubric for commercial migration:
  - imported exactly
  - approximated
  - preserved as source metadata
  - unsupported
- [ ] Prototype library extraction before full design import:
  - symbols
  - packages
  - component parameters
  - part/package binding recovery
- [ ] Define migration-report requirements for partial-fidelity imports
- [ ] Identify first supported commercial target
  - recommendation: Altium first, PADS second, OrCAD/Allegro third

**Deliverable**: A commercial interop implementation brief with corpus,
target format/version selection, recommended ingestion path, fidelity rubric,
and explicit non-goals. No support claim yet.

### M3: Write Operations on Imported Designs
**Goal**: Limited but safe modifications to imported designs.

- [ ] Operation model:
  - Operation trait (validate, execute, describe)
  - Operation diff (what changed, for undo and sync)
  - Undo/redo stack (operation replay)
  - Batch execution
- [ ] Operations (authored-data modifications only):
  - MoveComponent, RotateComponent, DeleteComponent
  - SetValue, SetReference, AssignPart
  - SetNetClass, SetDesignRule
  - DeleteTrack, DeleteVia (remove routing)
- [ ] Derived data recomputation after operations:
  - Connectivity update
  - Airwire update
  - DRC incremental re-check
- [ ] MCP write tools: move_component, set_value, set_rule, delete_track
- [ ] CLI: `tool modify <design> --move U1 50,30 --rotate U1 90`
- [ ] Export: KiCad .kicad_pcb write-back (modified design saves in original format)
- [ ] Test: import design → modify → export → re-import → verify changes

**Deliverable**: AI agent can reorganize component placement on an imported
design, update values, adjust rules, and save back to KiCad format.

### M4: Native Project Creation + Editing
**Goal**: Create new designs from scratch, not just modify imports.

- [ ] Native file format (JSON, schema documented)
- [ ] Project structure (pool reference, board, schematic, rules, settings)
- [ ] Schematic editing operations:
  - PlaceSymbol, DrawWire, PlaceLabel, PlacePowerSymbol
  - Annotate (auto-reference-designator)
  - AssignPart (map schematic symbol to pool part)
- [ ] Board editing operations:
  - PlaceComponent (from schematic netlist)
  - AddTrack, AddVia (manual trace placement, not interactive routing)
  - AddZone, PourCopper
  - AddKeepout, AddDimension
- [ ] Forward annotation (schematic → board ECO with per-change granularity)
- [ ] Backward annotation (board → schematic)
- [ ] Export: Gerber RS-274X / X2
- [ ] Export: Excellon drill
- [ ] Export: BOM (CSV, JSON)
- [ ] Export: Pick and Place

**Deliverable**: Create a design from schematic to manufacturing output
without any GUI. AI agent or CLI drives the entire flow.

### M5: Deterministic Layout Kernel
**Goal**: Unified placement + routing with full constraint awareness.
Placement and routing are one coupled solver, not separate tools.
See docs/LAYOUT_ENGINE.md for full specification.

Placement:
- [ ] Component grouping (explicit, schematic hierarchy, netlist clustering)
- [ ] Force-directed placement within groups
- [ ] Overlap resolution and legalization
- [ ] Routability estimation (HPWL, coarse congestion)

Routing:
- [ ] Routing discretization: adaptive graph (not nm grid search)
- [ ] Single-net routing: A* on obstacle-aware graph, Steiner decomposition
- [ ] Obstacle inflation (Minkowski sum for clearance corridors)
- [ ] Via rules: drill/diameter/layer-span enforcement during search
- [ ] Manual trace placement (point-to-point, clearance-aware)
- [ ] Multi-net: negotiated rip-up and reroute with priority ordering
- [ ] Diff pair prototype: coupled centerline routing with gap enforcement
- [ ] Copper pour engine (polygon fill with thermal relief)

Co-optimization:
- [ ] Routing→placement feedback (congestion triggers adjustment)
- [ ] Placement→routing feedback (move triggers reroute)
- [ ] Bounded iteration (max 3 cross-phase cycles)
- [ ] Unified LayoutProposal output (placement + routing + constraints)

Benchmarks:
- [ ] Placement: HPWL, congestion, constraint satisfaction
- [ ] Routing: completion rate, via count, length, clearance violations
- [ ] Combined: runtime, stability under repeated runs

**Deliverable**: Import a design, propose improved placement and routing.
Proposals reviewed via CLI/MCP. Benchmarked against known-good layouts.

### M6: Layout Strategy + AI Layer
**Goal**: Intelligent layout strategy and constraint-from-intent.

Placement intelligence:
- [ ] Circuit recognition (netlist topology + values → functional groups)
- [ ] Design pattern library (LDO, buck, USB, crystal, opamp patterns)
- [ ] Congestion-aware floorplanning
- [ ] AI placement intent: "isolate analog from switching regulator"

Routing intelligence:
- [ ] Net ordering optimization (constraint criticality, spatial analysis)
- [ ] Layer assignment per net (impedance-aware, from stackup)
- [ ] Impedance-to-width resolution (closed-form, IPC-2141/Wadell)
- [ ] Length matching (automated serpentine/accordion insertion)
- [ ] Diff pair: skew matching, via stagger patterns

Unified:
- [ ] AI intent translation: "DDR4 interface" → placement groups +
      routing constraints (one command, both domains)
- [ ] AI strategy suggestion: floorplan, routing order, layer budget
- [ ] AI post-layout analysis: placement + routing quality, weak points
- [ ] Proposal ranking: score candidates, present best with rationale

**Deliverable**: AI agent can say "layout the SPI bus at 50Ω with
controller and peripherals grouped" and get a placement + routing
proposal with explanation.

### M7: GUI + Review Interface
**Goal**: Visual editing, route review, command line.

- [ ] wgpu 2D canvas (layers, selection, zoom/pan)
- [ ] Properties panel (Altium-style: context-sensitive, multi-select)
- [ ] Route review UI (accept/reject/adjust proposals per net)
- [ ] Net length gauge, route state visualization, clearance display
- [ ] Command line input (Eagle-style + natural language pass-through)
- [ ] Keyboard-centric routing shortcuts (width/via/mode cycling)
- [ ] Schematic editor (place, wire, annotate)

### M8: Professional Features
- [ ] Full rule query evaluator (And, Or, Not, InComponent, HasPackage, etc.
      — expression IR exists from M0, evaluator expanded here)
- [ ] Impedance-aware layer stack manager with field solver
- [ ] Supply chain integration
- [ ] Design reuse blocks
- [ ] Variant management
- [ ] Output job system
- [ ] STEP export (3D)

---

## Critical Path

```
M0 (IR + pool) ──→ M1 (import + query) ──→ M2 (DRC + MCP/CLI)
                                                    │
                                           M3 (write ops) ──→ M4 (native authoring)
                                                                       │
                                              M5 (layout kernel) ──→ M6 (strategy + AI)
                                                                       │
                                                              M7 (GUI) ──→ M8 (pro)
```

M2 is the first "useful" milestone. Everything before it is foundation.
M3 is the first "AI can modify designs" milestone.
M4 is the first "don't need another tool" milestone.
M5 is the first "layout" milestone — placement + routing kernel, no AI yet.
M6 adds the intelligence layer on a proven routing foundation.
M7 is the first "visual tool" milestone.

---

## Decision Log

| Date | Decision | Rationale |
|------|----------|-----------|
| 2026-03-24 | Study Eagle/Horizon/Altium | Know the domain before building |
| 2026-03-24 | Build from scratch, not fork | Fork inherits unwanted architecture, GPL, GUI-first design |
| 2026-03-24 | Rust engine | Memory safety, modern toolchain, cargo ecosystem |
| 2026-03-24 | Engine-first, GUI-last | Headless is the differentiator; GUI is a consumer |
| 2026-03-24 | Import-first, author-later | Useful before complete; real designs validate the engine |
| 2026-03-24 | MCP as AI interface | Standard protocol, Claude Code integration |
| 2026-03-24 | Altium UX as benchmark | Professional features = domain vocabulary for AI |
| 2026-03-24 | One architect | Community-driven EDA has failed; opinionated direction required |
| 2026-03-24 | No KiCad PNS dependency | AI-proposes/human-reviews routing paradigm eliminates need for push-and-shove dependency. Traditional autorouter UX is not the right interface; constraint-formalized routing plus review is the path. GPL question dissolves. |
| 2026-03-24 | v1 scope: analysis + automation | Full CAD is the trajectory, not the launch |
