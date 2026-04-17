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

pub struct Point3D {
    pub x: i64,
    pub y: i64,
    pub z: i64,
}

pub struct Euler3D {
    pub roll_tenths_deg: i32,
    pub pitch_tenths_deg: i32,
    pub yaw_tenths_deg: i32,
}

pub struct Transform3D {
    pub translation_nm: Point3D,
    pub rotation_tenths_deg: Euler3D,
    pub scale: f32,                       // 1.0 default
}

pub enum ModelFormat {
    Step,                                 // .step / .stp
    Wrl,                                  // VRML — KiCad legacy
    Iges,                                 // .igs / .iges — legacy
    Obj,                                  // Wavefront — hobbyist
    Gltf,                                 // glTF 2.0 — web / M7 GUI
}

/// Provenance of any attached external file (3D model OR behavioural model).
/// Single canonical shape so 3D-geometry and behavioural-model attachments
/// share one provenance contract.
pub struct ModelProvenance {
    pub source: String,                   // URL or local path of origin
    pub vendor: Option<String>,           // canonical vendor (JEP106-normalised)
    pub fetched_at: Option<DateTime<Utc>>,
    pub sha256: String,                   // identity-stable hash
}

pub struct ModelRef {
    pub path: String,
    pub format: ModelFormat,
    pub transform: Transform3D,
    pub provenance: Option<ModelProvenance>,
}

/// Behavioural-model attachment: IBIS / SPICE / Touchstone / IBIS-AMI /
/// Verilog-A / VHDL-AMS / Compact Thermal. Distinct from `ModelRef`,
/// which is for 3D geometry only.
pub enum ModelRole {
    Spice,                                // SPICE netlist (.cir / .lib / .sub / .inc)
    Ibis,                                 // IBIS .ibs file
    IbisIss,                              // IBIS-ISS subcircuit
    IbisAmi,                              // IBIS-AMI bundle (.ami + binaries)
    Touchstone,                           // S-parameter file (.s1p .. .sNp)
    VerilogA,                             // Verilog-A source
    VerilogAms,                           // Verilog-AMS source
    VhdlAms,                              // VHDL-AMS source
    CompactThermal,                       // CTM / ECXML / JESD15-4
}

pub enum SpiceDialect {
    Berkeley3,
    Ngspice,
    LTspice,
    PSpice,
    HSpice,
    Xyce,
    Spectre,
    Unknown,
}

pub enum EncryptionScheme {
    IbisBird176,                          // IBIS BIRD-176 (AES-128, vendor key)
    PSpiceEncryptIt,
    HSpiceAvantHash,
    LTspiceObfuscation,
    SpectreEncrypt,
    Other(String),
}

pub enum ModelFormatMetadata {
    Spice { ngspice_validates: Option<bool> },
    Ibis { ibis_version: String, has_ami: bool },
    IbisAmi { ami_version: String, platform_binaries: HashMap<String, String> },
    Touchstone { ports: u32, frequency_range_hz: (f64, f64) },
    None,
}

pub struct ModelAttachment {
    pub uuid: Uuid,                       // pool-resolved UUID
    pub model_uuid: Uuid,                 // → pool model entity (pool/models/<role>/<sha256>)
    pub role: ModelRole,
    pub dialect: Option<SpiceDialect>,    // for SPICE only
    pub model_names: Vec<String>,         // [Model] names (IBIS), .MODEL/.SUBCKT names (SPICE)
    pub encrypted: bool,
    pub encryption_scheme: Option<EncryptionScheme>,
    pub provenance: Option<ModelProvenance>,
    pub format_metadata: ModelFormatMetadata,
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
    pub body_height_nm: Option<i64>,           // tallest authored body height — required for IDF 3.0 export
    pub body_height_mounted_nm: Option<i64>,   // tall-component / standoff-aware mounted height
    pub tags: HashSet<String>,
}

pub struct PadMapEntry {
    pub gate: Uuid,       // → Gate
    pub pin: Uuid,        // → Pin
}

pub struct Part {
    pub uuid: Uuid,
    pub entity: Uuid,                                // → Entity
    pub package: Uuid,                               // → Package
    pub pad_map: HashMap<Uuid, PadMapEntry>,         // Pad UUID → (Gate, Pin)
    pub mpn: String,
    pub manufacturer: String,
    pub manufacturer_jep106: Option<u16>,            // JEDEC JEP106 manufacturer ID (canonicalised vendor identity)
    pub value: String,
    pub description: String,
    pub datasheet: String,
    pub parametric: HashMap<String, String>,
    pub orderable_mpns: Vec<String>,
    pub packaging_options: Vec<PackagingOption>,     // EIA-481 reel / tape / tray / tube / cut variants
    pub tags: HashSet<String>,
    pub lifecycle: Lifecycle,                        // Active, NRND, EOL, Obsolete, Unknown
    pub base: Option<Uuid>,                          // → Part (inheritance)
    pub behavioural_models: Vec<ModelAttachment>,    // IBIS / SPICE / Touchstone / IBIS-AMI / Compact-Thermal
    pub thermal: Option<ThermalSpec>,                // JESD15-3 two-resistor + max junction
    pub supply_chain_offers: Option<Vec<SupplyOffer>>,    // Octopart / distributor cache (derived)
    pub last_supply_chain_check: Option<DateTime<Utc>>,   // when the cache was last refreshed
}

/// Two-resistor thermal model per JEDEC JESD15-3 plus DELPHI/JESD15-4
/// boundary parameters. Compact-thermal multi-node models attach via
/// `Part.behavioural_models` with `ModelRole::CompactThermal`.
pub struct ThermalSpec {
    pub theta_ja_c_per_w: Option<f32>,
    pub theta_jc_top_c_per_w: Option<f32>,
    pub theta_jc_bot_c_per_w: Option<f32>,
    pub theta_jb_c_per_w: Option<f32>,
    pub max_junction_c: Option<f32>,
    pub thermal_reference: Option<String>,           // "JESD51-2 still-air, 1S board"
}

/// EIA-481 packaging options keyed off the canonical part MPN.
pub enum PackagingKind {
    Reel { tape_width_mm: u16, reel_diameter_inch: u8, qty_per_reel: u32 },
    Tray { qty_per_tray: u32 },
    Tube { qty_per_tube: u32 },
    Bag { qty_per_bag: u32 },
    Cut { qty: u32 },                                // cut tape strip
}

pub struct PackagingOption {
    pub kind: PackagingKind,
    pub mpn_suffix: Option<String>,                  // e.g. TI's 'R' or 'T' suffix
}

/// Distributor offer cache. Derived data — refreshed on demand via the
/// `refresh_supply_chain` MCP tool. Not authored.
pub struct SupplyOffer {
    pub distributor: String,
    pub price_breaks: Vec<(u32, f64, String)>,       // (qty, price, currency)
    pub stock: Option<u32>,
    pub lead_time_weeks: Option<u32>,
    pub link: String,
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
    pub class: Uuid,                          // → NetClass
    pub controlled_impedance: Option<ImpedanceSpec>,  // per-net impedance target — IPC-2581 Rev C / ODB++ aware
}

/// Per-net controlled-impedance specification. Consumed by the deferred
/// M5+ `Impedance` rule type and by IPC-2581 Rev C `<ImpedancesProperties>`
/// / ODB++ impedance-control attribute export.
pub struct ImpedanceSpec {
    pub target_ohms: f32,                     // 50 (single-ended), 90 (USB), 100 (LVDS), etc.
    pub tolerance_pct: f32,                   // ±10 typical
    pub controlled_dielectric: Option<Uuid>,  // → StackupLayer the impedance is controlled against
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
    pub layer_type: StackupLayerType,         // Copper, Dielectric, SolderMask, Silkscreen, Paste, Mechanical
    pub thickness_nm: i64,
    // Material properties: required by ODB++, IPC-2581 Rev C, and any
    // controlled-impedance work. All `Option<>`-wrapped so existing
    // imports (KiCad ≤7) deserialize without populating them; KiCad 8+
    // populates from the `stackup-material` block in `.kicad_pro`.
    pub dielectric_constant: Option<f32>,     // Dk, dimensionless
    pub loss_tangent: Option<f32>,            // Df, dimensionless
    pub copper_weight_oz: Option<f32>,        // 0.5, 1.0, 2.0 typical
    pub roughness_um: Option<f32>,            // Rrms surface roughness (microns)
    pub material_name: Option<String>,        // FR-4, Rogers 4350B, etc.
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

### Library Model Attachment Operations

Attaching or detaching a behavioural model on a `Part` is an authored
operation: it must flow through the transaction model so the audit
trail and undo/redo stack remain correct.

```rust
/// Attach a behavioural-model file (IBIS / SPICE / Touchstone /
/// IBIS-AMI / Verilog-A / VHDL-AMS / Compact Thermal) to a Part.
/// Reads the file at attach time to populate `ModelAttachment`
/// metadata (model names, encryption status, format metadata).
pub struct AttachModel {
    pub part_uuid: Uuid,
    pub model_path: PathBuf,                  // file to attach
    pub role: ModelRole,
}

/// Detach a previously-attached model from a Part. Reversible —
/// the OpDiff carries the full `ModelAttachment` so `inverse()`
/// can recreate it.
pub struct DetachModel {
    pub part_uuid: Uuid,
    pub model_attachment_uuid: Uuid,
}
```

Both operations are **fully reversible**:

- `AttachModel.inverse()` returns a `DetachModel` for the same
  attachment UUID.
- `DetachModel.inverse()` returns an `AttachModel` re-creating the
  original attachment from the OpDiff payload (including the source
  path snapshot).

The pool-side model file lives in
`pool/models/<role>/<sha256>.<ext>`; attach/detach manipulates the
`Part.behavioural_models` reference list, never the pool file itself.

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
    pub fn get_scoped_component_replacement_plan(
        &self,
        input: ScopedComponentReplacementPolicyInput,
    ) -> Result<ScopedComponentReplacementPlan>;
    pub fn edit_scoped_component_replacement_plan(
        &self,
        plan: ScopedComponentReplacementPlan,
        edit: ScopedComponentReplacementPlanEdit,
    ) -> Result<ScopedComponentReplacementPlan>;
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
    pub fn apply_scoped_component_replacement_plan(
        &mut self,
        plan: ScopedComponentReplacementPlan,
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
