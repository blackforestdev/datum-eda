# Engine Specification

## 1. Core Types

All coordinates are `i64` nanometers. All angles are `i32` tenths of degree.
All identifiers are `Uuid`. See docs/CANONICAL_IR.md for rationale.

### 1.1 Geometry Primitives

```rust
pub struct Point { pub x: i64, pub y: i64 }
pub struct Rect { pub min: Point, pub max: Point }

pub struct Polygon {
    pub vertices: Vec<Point>,
    pub closed: bool,          // last→first implicit edge
}

pub struct Arc {
    pub center: Point,
    pub radius: i64,
    pub start_angle: i32,      // tenths of degree
    pub end_angle: i32,
}

pub type LayerId = i32;        // well-known: 1=Top, N=Bottom, 2..N-1=Inner
```

### 1.1a Shared Enums and Reference Types

```rust
pub enum PinDirection {
    Input,
    Output,
    Bidirectional,
    Passive,
    PowerIn,
    PowerOut,
    OpenCollector,
    OpenEmitter,
    TriState,
    NoConnect,
}

pub struct AlternateName {
    pub name: String,
    pub kind: String,
}

pub enum Lifecycle {
    Active,
    Nrnd,
    Eol,
    Obsolete,
    Unknown,
}

pub enum StackupLayerType {
    Copper,
    Dielectric,
    SolderMask,
    Silkscreen,
    Paste,
    Mechanical,
}

pub enum PortDirection {
    Input,
    Output,
    Bidirectional,
    Passive,
}

pub struct ModelRef {
    pub path: String,
    pub transform: Option<serde_json::Value>,  // exact 3D transform deferred
}

pub enum Primitive {
    Line { from: Point, to: Point, width: i64 },
    Rect { min: Point, max: Point, width: i64 },
    Circle { center: Point, radius: i64, width: i64 },
    Polygon { polygon: Polygon, width: i64 },
    Arc { arc: Arc, width: i64 },
    Text { text: String, position: Point, rotation: i32 },
}
```

### 1.2 Pool Types

```rust
pub struct Pin {
    pub uuid: Uuid,
    pub name: String,
    pub direction: PinDirection,  // Input, Output, Bidirectional, Passive, Power, OpenCollector, NC
    pub swap_group: u32,          // 0 = not swappable
    pub alternates: Vec<AlternateName>,
}

pub struct Unit {
    pub uuid: Uuid,
    pub name: String,
    pub manufacturer: String,
    pub pins: HashMap<Uuid, Pin>,
    pub tags: HashSet<String>,
}

pub struct Gate {
    pub uuid: Uuid,
    pub name: String,
    pub unit: Uuid,       // → Unit
    pub symbol: Uuid,     // → Symbol
}

pub struct Entity {
    pub uuid: Uuid,
    pub name: String,
    pub prefix: String,   // R, C, U, etc.
    pub manufacturer: String,
    pub gates: HashMap<Uuid, Gate>,
    pub tags: HashSet<String>,
}

pub struct Pad {
    pub uuid: Uuid,
    pub name: String,
    pub position: Point,
    pub padstack: Uuid,   // → Padstack
    pub layer: LayerId,
}

pub struct Package {
    pub uuid: Uuid,
    pub name: String,
    pub pads: HashMap<Uuid, Pad>,
    pub courtyard: Polygon,
    pub silkscreen: Vec<Primitive>,
    pub models_3d: Vec<ModelRef>,
    pub tags: HashSet<String>,
}

pub struct PadMapEntry {
    pub gate: Uuid,       // → Gate
    pub pin: Uuid,        // → Pin
}

pub struct Part {
    pub uuid: Uuid,
    pub entity: Uuid,     // → Entity
    pub package: Uuid,    // → Package
    pub pad_map: HashMap<Uuid, PadMapEntry>,  // Pad UUID → (Gate, Pin)
    pub mpn: String,
    pub manufacturer: String,
    pub value: String,
    pub description: String,
    pub datasheet: String,
    pub parametric: HashMap<String, String>,
    pub orderable_mpns: Vec<String>,
    pub tags: HashSet<String>,
    pub lifecycle: Lifecycle,  // Active, NRND, EOL, Obsolete, Unknown
    pub base: Option<Uuid>,   // → Part (inheritance)
}
```

### 1.3 Board Types

```rust
pub struct Board {
    pub uuid: Uuid,
    pub name: String,
    pub stackup: Stackup,
    pub outline: Polygon,
    pub packages: HashMap<Uuid, PlacedPackage>,
    pub tracks: HashMap<Uuid, Track>,
    pub vias: HashMap<Uuid, Via>,
    pub zones: HashMap<Uuid, Zone>,
    pub nets: HashMap<Uuid, Net>,
    pub net_classes: HashMap<Uuid, NetClass>,
    pub rules: RuleSet,
    pub keepouts: Vec<Keepout>,
    pub dimensions: Vec<Dimension>,
    pub texts: Vec<BoardText>,
}

pub type RuleSet = Vec<Rule>;

pub struct PlacedPackage {
    pub uuid: Uuid,
    pub part: Uuid,           // → Part
    pub reference: String,    // R1, U1, etc.
    pub value: String,
    pub position: Point,
    pub rotation: i32,        // tenths of degree
    pub layer: LayerId,       // top or bottom
    pub locked: bool,
}

pub struct Track {
    pub uuid: Uuid,
    pub net: Uuid,
    pub from: Point,
    pub to: Point,
    pub width: i64,
    pub layer: LayerId,
}

pub struct Via {
    pub uuid: Uuid,
    pub net: Uuid,
    pub position: Point,
    pub drill: i64,
    pub diameter: i64,
    pub from_layer: LayerId,
    pub to_layer: LayerId,
}

pub struct Zone {
    pub uuid: Uuid,
    pub net: Uuid,
    pub polygon: Polygon,     // authored boundary
    pub layer: LayerId,
    pub priority: u32,
    pub thermal_relief: bool,
    pub thermal_gap: i64,
    pub thermal_spoke_width: i64,
    // fill geometry is DERIVED, not stored here
}

pub struct Net {
    pub uuid: Uuid,
    pub name: String,
    pub class: Uuid,          // → NetClass
}

pub struct NetClass {
    pub uuid: Uuid,
    pub name: String,
    pub clearance: i64,
    pub track_width: i64,
    pub via_drill: i64,
    pub via_diameter: i64,
    pub diffpair_width: i64,
    pub diffpair_gap: i64,
}

pub struct Stackup {
    pub layers: Vec<StackupLayer>,
}

pub struct StackupLayer {
    pub id: LayerId,
    pub name: String,
    pub layer_type: StackupLayerType,  // Copper, Dielectric
    pub thickness_nm: i64,
    // M8+: dk, df, copper_weight, roughness
}

pub struct Keepout {
    pub uuid: Uuid,
    pub polygon: Polygon,
    pub layers: Vec<LayerId>,
    pub kind: String,          // copper, via, component, mixed
}

pub struct Dimension {
    pub uuid: Uuid,
    pub from: Point,
    pub to: Point,
    pub text: Option<String>,
}

pub struct BoardText {
    pub uuid: Uuid,
    pub text: String,
    pub position: Point,
    pub rotation: i32,
    pub layer: LayerId,
}
```

### 1.4 Schematic Types

```rust
pub struct Schematic {
    pub uuid: Uuid,
    pub sheets: HashMap<Uuid, Sheet>,
    pub sheet_definitions: HashMap<Uuid, SheetDefinition>,
    pub sheet_instances: HashMap<Uuid, SheetInstance>,
    pub variants: HashMap<Uuid, Variant>,
    pub waivers: Vec<CheckWaiver>,
}

pub struct Sheet {
    pub uuid: Uuid,
    pub name: String,
    pub frame: Option<SheetFrame>,
    pub symbols: HashMap<Uuid, PlacedSymbol>,
    pub wires: HashMap<Uuid, SchematicWire>,
    pub junctions: HashMap<Uuid, Junction>,
    pub labels: HashMap<Uuid, NetLabel>,
    pub buses: HashMap<Uuid, Bus>,
    pub bus_entries: HashMap<Uuid, BusEntry>,
    pub ports: HashMap<Uuid, HierarchicalPort>,
    pub noconnects: HashMap<Uuid, NoConnectMarker>,
    pub texts: HashMap<Uuid, SchematicText>,
    pub drawings: HashMap<Uuid, SchematicPrimitive>,
}

pub struct SheetFrame {
    pub uuid: Uuid,
    pub title: String,
    pub revision: Option<String>,
    pub company: Option<String>,
    pub page_number: Option<String>,
}

pub struct SheetInstance {
    pub uuid: Uuid,
    pub definition: Uuid,   // → SheetDefinition
    pub parent_sheet: Option<Uuid>,
    pub position: Point,
    pub name: String,
}

pub struct SheetDefinition {
    pub uuid: Uuid,
    pub root_sheet: Uuid,   // → Sheet
    pub name: String,
}

pub struct PlacedSymbol {
    pub uuid: Uuid,
    pub part: Option<Uuid>,       // → Part, if assigned
    pub entity: Option<Uuid>,     // → Entity, if imported without full Part
    pub gate: Option<Uuid>,       // → Gate for multi-gate entities
    pub reference: String,
    pub value: String,
    pub fields: Vec<SymbolField>,
    pub position: Point,
    pub rotation: i32,
    pub mirrored: bool,
    pub unit_selection: Option<String>,
    pub display_mode: SymbolDisplayMode,
    pub pin_overrides: Vec<PinDisplayOverride>,
    pub hidden_power_behavior: HiddenPowerBehavior,
}

pub struct SymbolField {
    pub uuid: Uuid,
    pub key: String,
    pub value: String,
    pub position: Option<Point>,
    pub visible: bool,
}

pub enum SymbolDisplayMode {
    LibraryDefault,
    ShowHiddenPins,
    HideOptionalPins,
}

pub struct PinDisplayOverride {
    pub pin: Uuid,              // → Pin
    pub visible: bool,
    pub position: Option<Point>,
}

pub enum HiddenPowerBehavior {
    SourceDefinedImplicit,
    ExplicitPowerObject,
    PreservedAsImportedMetadata,
}

pub struct SchematicWire {
    pub uuid: Uuid,
    pub from: Point,
    pub to: Point,
}

pub struct Junction {
    pub uuid: Uuid,
    pub position: Point,
}

pub enum LabelKind {
    Local,
    Global,
    Hierarchical,
    Power,
}

pub struct NetLabel {
    pub uuid: Uuid,
    pub kind: LabelKind,
    pub name: String,
    pub position: Point,
}

pub struct Bus {
    pub uuid: Uuid,
    pub name: String,
    pub members: Vec<String>,
}

pub struct BusEntry {
    pub uuid: Uuid,
    pub bus: Uuid,              // → Bus
    pub wire: Option<Uuid>,     // → SchematicWire
    pub position: Point,
}

pub struct HierarchicalPort {
    pub uuid: Uuid,
    pub name: String,
    pub direction: PortDirection,
    pub position: Point,
}

pub struct NoConnectMarker {
    pub uuid: Uuid,
    pub symbol: Uuid,           // → PlacedSymbol
    pub pin: Uuid,              // → Pin
    pub position: Point,
}

pub struct SchematicText {
    pub uuid: Uuid,
    pub text: String,
    pub position: Point,
    pub rotation: i32,
}

pub enum SchematicPrimitive {
    Line { uuid: Uuid, from: Point, to: Point },
    Rect { uuid: Uuid, min: Point, max: Point },
    Circle { uuid: Uuid, center: Point, radius: i64 },
    Arc { uuid: Uuid, arc: Arc },
}

pub struct Variant {
    pub uuid: Uuid,
    pub name: String,
    pub fitted_components: HashMap<Uuid, bool>,  // logical component UUID → fitted state
}

pub struct BoardNetInfo {
    pub uuid: Uuid,
    pub name: String,
    pub class: String,
    pub pins: Vec<NetPinRef>,
    pub tracks: usize,
    pub vias: usize,
    pub routed_length_nm: i64,
    pub routed_pct: f32,
}

pub struct SchematicNetInfo {
    pub uuid: Uuid,
    pub name: String,
    pub class: Option<String>,
    pub pins: Vec<NetPinRef>,
    pub labels: usize,
    pub ports: usize,
    pub sheets: Vec<String>,
    pub semantic_class: Option<String>,
}

pub struct NetPinRef {
    pub component: String,
    pub pin: String,
}

pub struct SheetSummary {
    pub uuid: Uuid,
    pub name: String,
    pub symbols: usize,
    pub ports: usize,
    pub labels: usize,
    pub buses: usize,
}

pub struct SymbolInfo {
    pub uuid: Uuid,
    pub sheet: Uuid,
    pub reference: String,
    pub value: String,
    pub position: Point,
    pub rotation: i32,
    pub mirrored: bool,
    pub part_uuid: Option<Uuid>,
    pub entity_uuid: Option<Uuid>,
    pub gate_uuid: Option<Uuid>,
}

pub struct PortInfo {
    pub uuid: Uuid,
    pub sheet: Uuid,
    pub name: String,
    pub direction: PortDirection,
    pub position: Point,
}

pub struct LabelInfo {
    pub uuid: Uuid,
    pub sheet: Uuid,
    pub kind: LabelKind,
    pub name: String,
    pub position: Point,
}

pub struct BusInfo {
    pub uuid: Uuid,
    pub sheet: Uuid,
    pub name: String,
    pub members: Vec<String>,
}

pub struct BusEntryInfo {
    pub uuid: Uuid,
    pub sheet: Uuid,
    pub bus: Uuid,
    pub wire: Option<Uuid>,
    pub position: Point,
}

pub struct NoConnectInfo {
    pub uuid: Uuid,
    pub sheet: Uuid,
    pub symbol: Uuid,
    pub pin: Uuid,
    pub position: Point,
}

pub struct SymbolFieldInfo {
    pub uuid: Uuid,
    pub symbol: Uuid,
    pub key: String,
    pub value: String,
    pub visible: bool,
    pub position: Option<Point>,
}

pub struct HierarchyInfo {
    pub instances: Vec<SheetInstanceInfo>,
    pub links: Vec<HierarchicalLinkInfo>,
}

pub struct SheetInstanceInfo {
    pub uuid: Uuid,
    pub definition: Uuid,
    pub parent_sheet: Option<Uuid>,
    pub position: Point,
    pub name: String,
}

pub struct HierarchicalLinkInfo {
    pub parent_sheet: Uuid,
    pub child_sheet: Uuid,
    pub parent_port: Uuid,
    pub child_port: Uuid,
    pub net: Uuid,
}

pub struct ConnectivityDiagnosticInfo {
    pub kind: String,
    pub severity: String,
    pub message: String,
    pub objects: Vec<Uuid>,
}
```

### 1.5 Rule Types

```rust
pub struct Rule {
    pub uuid: Uuid,
    pub name: String,
    pub scope: RuleScope,      // expression tree (see CANONICAL_IR.md §6)
    pub priority: u32,         // higher wins
    pub enabled: bool,
    pub rule_type: RuleType,
    pub parameters: RuleParams,
}

pub enum RuleType {
    ClearanceCopper,
    TrackWidth,
    ViaHole,
    ViaAnnularRing,
    HoleSize,
    SilkClearance,
    Connectivity,
    // M5+: Impedance, LengthMatch, DiffpairGap, DiffpairSkew
}

pub enum RuleParams {
    Clearance { min: i64 },
    TrackWidth { min: i64, preferred: i64, max: i64 },
    ViaHole { min: i64, max: i64 },
    ViaAnnularRing { min: i64 },
    HoleSize { min: i64, max: i64 },
    SilkClearance { min: i64 },
    Connectivity {},  // hard pass/fail, no parameters
}
```

### 1.6 RuleScope Support Matrix

| Node | M2 | M6+ | Notes |
|------|-----|------|-------|
| `All` | eval | eval | Matches everything |
| `Net(uuid)` | eval | eval | Matches specific net |
| `NetClass(uuid)` | eval | eval | Matches all nets in class |
| `Layer(id)` | eval | eval | Matches objects on layer |
| `And(a, b)` | parse, error | eval | "rule uses And, not supported until M6" |
| `Or(a, b)` | parse, error | eval | |
| `Not(a)` | parse, error | eval | |
| `InComponent(uuid)` | parse, error | eval | |
| `HasPackage(glob)` | parse, error | eval | |
| `NetNameRegex(re)` | parse, error | eval | |
| `IsDiffpair` | parse, error | eval | |
| `IsVia` | parse, error | eval | |
| `IsPad` | parse, error | eval | |
| `IsSMD` | parse, error | eval | |
| `InArea(uuid)` | parse, error | eval | |

"parse, error" means: the expression deserializes correctly and is
preserved in the data model, but the M2 evaluator returns
`Err(UnsupportedScope { node: "And", available_from: "M6" })`.

---

## 2. Object Invariants

These must hold at all times in a valid design state.

### Identity
- Every authored object has a non-nil UUID.
- No two authored objects of the same type share a UUID.
- References (Uuid fields pointing to other objects) are non-dangling.
  A dangling reference is a hard validation error.

### Geometry
- All coordinates are nanometers (i64). No floating point in authored geometry.
- Board outline is a closed polygon with counter-clockwise winding.
- Track endpoints lie within the board outline (or are explicitly flagged).
- Via position lies within the board outline.

### Connectivity
- Every Track.net and Via.net references a valid Net.
- Every PlacedPackage.part references a valid Part in the pool.
- The connectivity graph is recomputed from authored data. It is never
  manually edited.
- Schematic connectivity is recomputed from authored schematic data. It is
  never manually edited or imported as canonical truth.

### Schematic Semantics
- Every placed symbol pin attachment resolves either to exactly one net or to
  an explicit unconnected/no-connect state.
- Hierarchical labels and ports must resolve deterministically.
- Buses are containers for scalar nets, not electrical nets themselves.
- No-connect markers target an exact symbol-pin pair.
- Multi-gate identity and field edits must survive annotation and ECO.
- Hidden power behavior must remain explicit in authored data or preserved as
  imported metadata until promoted.
- Sheet instances are authoritative hierarchy objects; hierarchy resolution
  never depends on page order alone.
- Variants are project-level authored data over logical components.
- Variant selection is schematic-authoritative; board fitted state is derived
  from component identity rather than separately authored per-package state.

### Rules
- Every Rule.scope is a valid RuleScope expression.
- Rules with higher priority override rules with lower priority.
- At most one rule of each RuleType matches any given object or object pair.

### Checking
- ERC findings are derived from schematic connectivity and pin semantics.
- DRC findings are derived from board geometry, board connectivity, and rules.
- Waivers are authored data and must target a specific checking domain.

---

## 3. Operations

### Operation Trait
See docs/CANONICAL_IR.md §4 for the canonical definition.

```rust
pub trait Operation: Send + Sync {
    fn validate(&self, design: &Design) -> Result<(), OpError>;
    fn execute(&self, design: &mut Design) -> Result<OpDiff, OpError>;
    fn inverse(&self, diff: &OpDiff) -> Option<Box<dyn Operation>>;
    fn describe(&self) -> String;
    fn serialize(&self) -> serde_json::Value;
}
```

### OpDiff
```rust
pub struct OpDiff {
    pub created: Vec<(ObjectType, Uuid)>,
    pub modified: Vec<(ObjectType, Uuid, serde_json::Value)>,  // previous state
    pub deleted: Vec<(ObjectType, Uuid, serde_json::Value)>,   // deleted object
}
```

### Transaction
```rust
pub struct Transaction {
    pub id: Uuid,
    pub operations: Vec<(Box<dyn Operation>, OpDiff)>,
    pub description: String,
}
```

### Undo semantics
- Undo reverts the most recent transaction (all operations in reverse order).
- Redo replays the most recently undone transaction.
- A new operation after undo clears the redo stack.
- Undo/redo operates on transactions, not individual operations.

### Derived data invalidation
After a transaction commits:
1. Identify which nets are affected (from OpDiff created/modified/deleted).
2. Recompute connectivity for affected nets only.
3. Recompute schematic connectivity for affected sheets/nets only.
4. Recompute airwires for nets with board connectivity changes.
5. Re-run ERC for schematic-domain changes affecting electrical intent.
6. Re-run DRC for board-domain changes affecting physical checks.
7. Invalidate zone fills intersecting modified geometry (lazy recompute).

---

## 4. Serialization Contract

### JSON format
- Maps: keys sorted alphabetically.
- Integers: no leading zeros, no trailing `.0`.
- UUIDs: lowercase hyphenated form (`xxxxxxxx-xxxx-xxxx-xxxx-xxxxxxxxxxxx`).
- Coordinates: integer nanometers, never floating point.
- Arrays: ordered as semantically meaningful (vertices in polygon order,
  layers in stackup order) or by UUID (for unordered collections).
- Schema version: `"schema_version": 1` at document root.
- Encoding: UTF-8, no BOM.

### Determinism guarantee
Same authored data → byte-identical JSON output on every run, every platform.
This is a hard requirement verified by golden tests.

---

## 5. Engine API

### API Contract Mode

This section tracks two layers:
- **Current implementation contract (authoritative for code today)**:
  methods implemented in `crates/engine/src/api/mod.rs`.
- **Target M2+ contract**: fuller method surface required by later
  milestone exit gates.

Unless explicitly marked as `Target M2+`, method signatures in this section
refer to the current implementation contract.

### 5.1 Current Implemented Engine API (2026-03-25)

```rust
pub struct Engine {
    pool: Pool,
    design: Option<Design>,
    undo_stack: Vec<Transaction>,
    redo_stack: Vec<Transaction>,
}

impl Engine {
    // Lifecycle
    pub fn new() -> Result<Self>;
    pub fn has_open_project(&self) -> bool;
    pub fn import(&mut self, path: &Path) -> Result<ImportReport>;
    pub fn close_project(&mut self);
    pub fn save(&self, path: &Path) -> Result<()>;

    // Pool
    pub fn search_pool(&self, query: &str) -> Result<Vec<PartSummary>>;
    pub fn get_part(&self, uuid: &Uuid) -> Result<PartDetail>;
    pub fn get_package(&self, uuid: &Uuid) -> Result<PackageDetail>;
    pub fn get_package_change_candidates(&self, component_uuid: &Uuid)
        -> Result<PackageChangeCompatibilityReport>;
    pub fn get_part_change_candidates(&self, component_uuid: &Uuid)
        -> Result<PartChangeCompatibilityReport>;
    pub fn get_component_replacement_plan(&self, component_uuid: &Uuid)
        -> Result<ComponentReplacementPlan>;
    pub fn import_eagle_library(&mut self, path: &Path) -> Result<ImportReport>;

    // Queries (read-only)
    pub fn get_board_summary(&self) -> Result<BoardSummary>;
    pub fn get_components(&self) -> Result<Vec<ComponentInfo>>;
    pub fn get_net_info(&self) -> Result<Vec<BoardNetInfo>>;
    pub fn get_stackup(&self) -> Result<StackupInfo>;
    pub fn get_unrouted(&self) -> Result<Vec<Airwire>>;
    pub fn get_schematic_summary(&self) -> Result<SchematicSummary>;
    pub fn get_sheets(&self) -> Result<Vec<SheetSummary>>;
    pub fn get_symbols(&self, sheet: Option<&Uuid>) -> Result<Vec<SymbolInfo>>;
    pub fn get_symbol_fields(&self, symbol_uuid: &Uuid) -> Result<Vec<SymbolFieldInfo>>;
    pub fn get_ports(&self, sheet: Option<&Uuid>) -> Result<Vec<PortInfo>>;
    pub fn get_labels(&self, sheet: Option<&Uuid>) -> Result<Vec<LabelInfo>>;
    pub fn get_buses(&self, sheet: Option<&Uuid>) -> Result<Vec<BusInfo>>;
    pub fn get_bus_entries(&self, sheet: Option<&Uuid>) -> Result<Vec<BusEntryInfo>>;
    pub fn get_noconnects(&self, sheet: Option<&Uuid>) -> Result<Vec<NoConnectInfo>>;
    pub fn get_hierarchy(&self) -> Result<HierarchyInfo>;
    pub fn get_schematic_net_info(&self) -> Result<Vec<SchematicNetInfo>>;
    pub fn get_netlist(&self) -> Result<Vec<NetlistNet>>;
    pub fn get_connectivity_diagnostics(&self) -> Result<Vec<ConnectivityDiagnosticInfo>>;
    pub fn get_design_rules(&self) -> Result<Vec<Rule>>;
    pub fn get_check_report(&self) -> Result<CheckReport>;

    // Writes (current M3 slice)
    pub fn replace_components(&mut self, inputs: Vec<ReplaceComponentInput>)
        -> Result<OperationResult>;
    pub fn apply_component_replacement_plan(
        &mut self,
        inputs: Vec<PlannedComponentReplacementInput>,
    ) -> Result<OperationResult>;
    pub fn apply_component_replacement_policy(
        &mut self,
        inputs: Vec<PolicyDrivenComponentReplacementInput>,
    ) -> Result<OperationResult>;
    pub fn apply_scoped_component_replacement_policy(
        &mut self,
        input: ScopedComponentReplacementPolicyInput,
    ) -> Result<OperationResult>;

    // Checking
    pub fn run_erc_prechecks(&self) -> Result<Vec<ErcFinding>>;
    pub fn run_erc_prechecks_with_config(&self, config: &ErcConfig) -> Result<Vec<ErcFinding>>;
    pub fn run_erc_prechecks_with_config_and_waivers(
        &self,
        config: &ErcConfig,
        waivers: &[CheckWaiver],
    ) -> Result<Vec<ErcFinding>>;
    pub fn run_drc(&self, rule_types: &[RuleType]) -> Result<DrcReport>;
    pub fn explain_violation(
        &self,
        domain: ViolationDomain,
        index: usize,
    ) -> Result<ViolationExplanation>;

    // Undo/redo capability flags (write ops deferred)
    pub fn can_undo(&self) -> bool;
    pub fn can_redo(&self) -> bool;
}
```

Notes:
- Current `get_net_info` and `get_schematic_net_info` return full inventories
  for the open design, not single-net selectors.
- Current `get_symbol_fields` is UUID-targeted.
- Current `get_bus_entries` exposes the all-sheets form used by daemon/MCP.
- Current `get_netlist` returns deterministic board/schematic inventories
  rather than a richer graph object.
- Current checking surface exposes ERC precheck findings and unified
  `CheckReport`; DRC currently covers connectivity, clearance, track width,
  via-hole, via-annular, and silk-clearance checks.
- Current close-project lifecycle control and violation explanation are part of
  the implemented API.

### 5.2 Target M2+ Engine API

```rust
pub struct Engine {
    pool: Pool,
    design: Option<Design>,
    undo_stack: Vec<Transaction>,
    redo_stack: Vec<Transaction>,
}

impl Engine {
    // Lifecycle
    pub fn new(pool_path: &Path) -> Result<Self>;
    pub fn import(&mut self, path: &Path) -> Result<ImportReport>;
    pub fn save(&self, path: &Path) -> Result<()>;

    // Pool
    pub fn search_pool(&self, query: &str) -> Vec<PartSummary>;
    pub fn get_part(&self, uuid: &Uuid) -> Result<&Part>;
    pub fn get_package(&self, uuid: &Uuid) -> Result<&Package>;
    pub fn import_eagle_library(&mut self, path: &Path) -> Result<ImportReport>;

    // Queries (read-only)
    pub fn get_netlist(&self) -> Result<Netlist>;
    pub fn get_board_summary(&self) -> Result<BoardSummary>;
    pub fn get_schematic_summary(&self) -> Result<SchematicSummary>;
    pub fn get_components(&self) -> Result<Vec<ComponentInfo>>;
    pub fn get_sheets(&self) -> Result<Vec<SheetSummary>>;
    pub fn get_symbols(&self, sheet: Option<&Uuid>) -> Result<Vec<SymbolInfo>>;
    pub fn get_ports(&self, sheet: Option<&Uuid>) -> Result<Vec<PortInfo>>;
    pub fn get_labels(&self, sheet: Option<&Uuid>) -> Result<Vec<LabelInfo>>;
    pub fn get_buses(&self, sheet: Option<&Uuid>) -> Result<Vec<BusInfo>>;
    pub fn get_bus_entries(&self, sheet: Option<&Uuid>) -> Result<Vec<BusEntryInfo>>;
    pub fn get_noconnects(&self, sheet: Option<&Uuid>) -> Result<Vec<NoConnectInfo>>;
    pub fn get_symbol_fields(&self, symbol: &Uuid) -> Result<Vec<SymbolFieldInfo>>;
    pub fn get_hierarchy(&self) -> Result<HierarchyInfo>;
    pub fn get_schematic_net_info(&self, net: &Uuid) -> Result<SchematicNetInfo>;
    pub fn get_connectivity_diagnostics(&self) -> Result<Vec<ConnectivityDiagnosticInfo>>;
    pub fn get_net_info(&self, net: &Uuid) -> Result<BoardNetInfo>;
    pub fn get_unrouted(&self) -> Result<Vec<Airwire>>;
    pub fn get_design_rules(&self) -> Result<Vec<RuleInfo>>;

    // Checking
    pub fn run_erc(&self) -> Result<ErcReport>;
    pub fn run_drc(&self, rule_types: &[RuleType]) -> Result<DrcReport>;

    // Operations (write)
    pub fn execute(&mut self, op: impl Operation) -> Result<OpDiff>;
    pub fn execute_batch(&mut self, ops: Vec<Box<dyn Operation>>) -> Result<Vec<OpDiff>>;
    pub fn undo(&mut self) -> Result<OpDiff>;
    pub fn redo(&mut self) -> Result<OpDiff>;
    pub fn can_undo(&self) -> bool;
    pub fn can_redo(&self) -> bool;
}
```

Target parity note: later milestones still expand the write surface, native
save/export, and richer normalized query contracts. The current implemented
API reflects the active `M2` slice; see `specs/PROGRAM_SPEC.md` and
`specs/MCP_API_SPEC.md`.
