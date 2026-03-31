# Engine Architecture

> **Status**: Non-normative architecture rationale.
> The controlling API and type contracts are `specs/ENGINE_SPEC.md`,
> `specs/PROGRAM_SPEC.md`, and `specs/MCP_API_SPEC.md`.
> If this document conflicts with `specs/`, `specs/` win.

## Principle: Headless-First

The engine is a Rust library with no GUI dependencies. It compiles and runs
as a library, a CLI tool, an MCP server backend, and a Python module —
all without linking any windowing or rendering code.

The GUI is a separate crate that consumes the engine API, just like the
CLI and MCP server do. Every committed design change flows through the
operation model. Interactive behaviors (drag, hover, route-in-progress)
are consumer-specific — they produce operations, they are not operations.

For the opening `M7` slice, the GUI should consume versioned deterministic
review-scene and review-payload contracts rather than reading ad hoc engine
state directly. The frontend may own workspace, viewport, hover/focus,
selection, terminal-lane, and AI-lane presentation state, but it must not own
parallel design truth or unofficial write semantics.
The locked `M7` shell and lane decisions do not change that authority
boundary; they only constrain how the frontend presents deterministic review
state.

```
engine (lib crate)
├── No GUI dependencies
├── No rendering dependencies
├── No windowing dependencies
├── Compiles to: library, CLI binary, Python module (via PyO3)
│
├── Consumed by:
│   ├── gui (bin crate) — wgpu renderer + editor
│   ├── cli (bin crate) — batch operations
│   ├── mcp-server (Python) — AI agent interface
│   └── python module — scripting
```

## Data Model

See docs/CANONICAL_IR.md for identity strategy, precision model,
authored/derived boundary, transaction semantics, and serialization rules.

### Pool (Component Library)

Derived from Horizon's pool concept, redesigned for AI queryability.

```
Unit (electrical identity)
  ├── uuid: UUID
  ├── name: String
  ├── manufacturer: String
  ├── pins: Map<UUID, Pin>
  │   └── Pin { name, direction, swap_group, alternates[] }
  └── tags: Set<String>

Symbol (schematic representation of a Unit)
  ├── uuid: UUID
  ├── unit: UUID → Unit
  ├── name: String
  └── graphics: Vec<Primitive> (lines, arcs, circles, text, pins)

Entity (multi-gate logical component)
  ├── uuid: UUID
  ├── name: String
  ├── prefix: String (R, C, U, etc.)
  ├── manufacturer: String
  └── gates: Map<UUID, Gate>
      └── Gate { name, unit: UUID → Unit, symbol: UUID → Symbol }

Package (physical footprint)
  ├── uuid: UUID
  ├── name: String
  ├── pads: Map<UUID, Pad>
  │   └── Pad { name, position, padstack: UUID, layer }
  ├── courtyard: Polygon
  ├── silkscreen: Vec<Primitive>
  └── models: Vec<Model3D> (STEP file references)

Part (the purchasable thing — key abstraction)
  ├── uuid: UUID
  ├── entity: UUID → Entity
  ├── package: UUID → Package
  ├── pad_map: Map<PadUUID, (GateUUID, PinUUID)>
  ├── attributes: { MPN, value, manufacturer, datasheet, description }
  ├── parametric: Map<String, String> (resistance, capacitance, etc.)
  ├── orderable_mpns: Vec<String>
  ├── supply_chain: Vec<SupplierInfo>
  ├── lifecycle: Lifecycle (active, nrnd, eol, obsolete)
  ├── tags: Set<String>
  └── base: Option<UUID → Part> (inheritance)
```

Storage: Individual JSON files in a directory tree, indexed by SQLite.
Same structural principle as Horizon, but schema designed from scratch.

### Board

```
Board
  ├── uuid: UUID
  ├── name: String
  ├── stackup: Stackup
  │   └── layers: Vec<Layer>
  │       └── Layer { type (copper/dielectric), thickness, material, Dk, Df }
  ├── outline: Polygon
  ├── packages: Map<UUID, PlacedPackage>
  │   └── PlacedPackage { part: UUID, position, rotation, layer, locked }
  ├── tracks: Map<UUID, Track>
  │   └── Track { net: UUID, from, to, width, layer }
  ├── vias: Map<UUID, Via>
  │   └── Via { net: UUID, position, drill, diameter, layers }
  ├── zones: Map<UUID, Zone>
  │   └── Zone { net: UUID, polygon, layer, priority, thermal_relief }
  ├── nets: Map<UUID, Net>
  │   └── Net { name, class: UUID }
  ├── net_classes: Map<UUID, NetClass>
  │   └── NetClass { name, clearance, track_width, via_drill, via_dia, diffpair_width, diffpair_gap }
  ├── rules: RuleSet (with query language)
  ├── keepouts: Vec<Keepout>
  └── dimensions: Vec<Dimension>
```

Storage: Single JSON file per board.

### Schematic

```
Schematic
  ├── uuid: UUID
  ├── sheets: Map<UUID, Sheet>
  │   └── Sheet { name, instances[], net_segments[], buses[], labels[] }
  ├── blocks: Map<UUID, Block> (hierarchical)
  │   └── Block { components, nets, net_ties, power_symbols }
  └── variants: Map<UUID, Variant>
```

Storage: JSON files — one per sheet, one for block hierarchy.

## Operation System

Every design modification goes through the Operation API:

See docs/CANONICAL_IR.md §4 for the canonical Operation trait definition
(validate, execute, inverse, describe, serialize).

```rust
// Summary — see CANONICAL_IR.md for full definition
pub trait Operation: Send + Sync {
    fn validate(&self, design: &Design) -> Result<(), OpError>;
    fn execute(&self, design: &mut Design) -> Result<OpDiff, OpError>;
    fn inverse(&self, diff: &OpDiff) -> Option<Box<dyn Operation>>;
    fn describe(&self) -> String;
    fn serialize(&self) -> serde_json::Value;
}
```

Operations are:
- **Atomic**: Each operation is a single undoable action
- **Validated**: Pre-flight check before execution
- **Diffable**: Returns what changed (for undo, sync, and AI reasoning)
- **Serializable**: Can be sent over IPC, logged, replayed
- **Composable**: Compound operations are sequences of atomic operations

### Operation Categories

```
Pool Operations:
  CreatePart, UpdatePart, ImportLibrary

Board Operations:
  PlaceComponent, MoveComponent, RotateComponent, DeleteComponent
  AddTrack, DeleteTrack, SetTrackWidth
  AddVia, DeleteVia
  ProposeRoute (AI-generated route for a net, validated before commit)
  ProposeRouteDiffpair (AI-generated diff pair route)
  AddTrackManual (point-to-point with clearance-aware pathfinding)
  TuneLength
  AddZone, UpdateZone, PourCopper
  SetDesignRule, DeleteDesignRule

Schematic Operations:
  PlaceSymbol, Wire, PlaceLabel, PlacePowerSymbol
  Annotate, AssignPart

Cross-Domain:
  SyncSchematicToBoard (ECO — generates list of atomic operations)
  SyncBoardToSchematic (backannotation)

Export:
  ExportGerber, ExportDrill, ExportBOM, ExportSTEP, ExportPDF
```

## Rule Engine

Expression-based rule scoping from M0 (see docs/CANONICAL_IR.md §6 for
the canonical RuleScope enum). The expression tree is the data model from
day one. The evaluator starts with leaf nodes (All, Net, NetClass, Layer)
in M2 and expands to full combinators in M6+.

Examples (informed by Altium's query language):

```
clearance {
    scope: InNetClass("HighSpeed") And OnLayer("Top")
    value: 0.15mm
    priority: 10
}

track_width {
    scope: InNet("USB_D+") Or InNet("USB_D-")
    min: 0.1mm
    preferred: 0.127mm  // 90Ω differential
    max: 0.2mm
    priority: 20
}

clearance {
    scope: All
    value: 0.1mm
    priority: 1  // default, lowest priority
}
```

Query functions: InNet, InNetClass, OnLayer, InComponent, HasPackage,
IsVia, IsPad, IsSMD, InArea, IsDiffpair, NetName matches regex.

Priority: higher number wins. First matching rule at each priority level.

## API Surface

Current implementation note (2026-03-25):
- The live engine API is a strict subset defined in
  `specs/ENGINE_SPEC.md` §5.1 (`Current Implemented Engine API`).
- The block below is the architectural target-state API shape (`Target M2+`
  and beyond), not a claim that every method is currently implemented.

```rust
// The engine API — consumed by GUI, CLI, MCP, Python
pub struct Engine {
    pub fn new() -> Self;
    pub fn open_project(path: &Path) -> Result<Project>;
    pub fn create_project(path: &Path, config: ProjectConfig) -> Result<Project>;

    // Pool
    pub fn search_pool(query: &str, filters: PoolFilters) -> Vec<PartSummary>;
    pub fn get_part(uuid: &Uuid) -> Result<Part>;
    pub fn import_eagle_library(path: &Path) -> Result<ImportReport>;
    pub fn import_kicad_library(path: &Path) -> Result<ImportReport>;

    // Design queries (read-only, no operation needed)
    pub fn get_netlist() -> Netlist;
    pub fn get_board_summary() -> BoardSummary;
    pub fn get_net_info(net: &Uuid) -> NetInfo;
    pub fn get_unrouted() -> Vec<Airwire>;
    pub fn get_drc_results() -> Vec<DrcViolation>;

    // Operations (the write path — every modification goes through here)
    pub fn execute(&mut self, op: impl Operation) -> Result<OperationDiff>;
    pub fn undo(&mut self) -> Result<OperationDiff>;
    pub fn redo(&mut self) -> Result<OperationDiff>;

    // Batch
    pub fn execute_batch(&mut self, ops: Vec<Box<dyn Operation>>) -> Result<Vec<OperationDiff>>;

    // Export
    pub fn export_gerber(settings: GerberSettings) -> Result<()>;
    pub fn export_drill(settings: DrillSettings) -> Result<()>;
    pub fn export_bom(settings: BomSettings) -> Result<Vec<BomLine>>;
    pub fn export_step(settings: StepSettings) -> Result<()>;

    // Routing (AI-proposes, engine-validates)
    pub fn propose_route(&self, net: &Uuid, settings: RouteSettings) -> Result<RouteProposal>;
    pub fn propose_route_diffpair(&self, pair: &DiffpairId, settings: RouteSettings) -> Result<RouteProposal>;
    pub fn accept_route(&mut self, proposal: RouteProposal) -> Result<OperationDiff>;
    pub fn add_track_manual(&mut self, points: &[Point], net: &Uuid, width: i64, layer: LayerId) -> Result<OperationDiff>;
}
```

Target-state parity note:
- At milestone closure for each phase, implemented methods should have
  corresponding consumer surfaces as required by `specs/PROGRAM_SPEC.md`.
- Current implementation intentionally exposes only a subset while `M1/M2`
  foundation work continues.
