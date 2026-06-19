# Engine Specification

## 0. Product-Mechanics Integration And Migration Target

Status: **Target**, unless explicitly labeled **Current implementation** or
**Compatibility**.

This section is the front-loaded target substrate for implementing the new
Datum product-mechanics scope. It integrates the readiness conclusions from
`docs/audits/scope-integration/DATUM_SCOPE_INTEGRATION_READINESS_AUDIT.md` and the shared tool contract
from `docs/contracts/AI_CLI_MCP_TOOL_SURFACE.md`.

The existing `Board`, `Schematic`, KiCad import/write-back, and per-method
engine APIs later in this spec remain truthful descriptions of the current
implementation and compatibility surface. They are not the target authority
model. New implementation work must route toward the substrate below.

### 0.1 Status Labels

- **Target**: desired product-mechanics contract and implementation direction.
- **Current implementation**: what the code implements today and tests may
  depend on.
- **Compatibility**: legacy/import/export/API behavior retained while the
  target substrate lands.

### 0.2 Target Authority Model

Datum has one canonical resolved `DesignModel`. Schematic, PCB, library,
rules, checks, proposals, transactions, manufacturing, artifacts, and
provenance are projections or partitions of that model, not independent
authorities.

```rust
pub struct DesignModel {
    pub project_id: Uuid,
    pub model_revision: ModelRevision,
    pub objects: HashMap<ObjectId, DomainObject>,
    pub component_instances: HashMap<ObjectId, ComponentInstance>,
    pub net_anchors: HashMap<NetId, NetAnchor>,
    pub relationships: HashMap<ObjectId, Relationship>,
    pub variants: HashMap<ObjectId, VariantOverlay>,
    pub output_jobs: HashMap<ObjectId, OutputJob>,
}

pub struct ProjectResolver {
    pub project_root: PathBuf,
    pub manifest: ProjectManifest,
    pub shard_index: Vec<SourceShardRef>,
}
```

The `ProjectResolver` is engine-owned. It assembles one in-memory
`DesignModel` from deterministic source shards, validates references, computes
derived caches, and marks stale projections. Shards are persistence
partitions, never authority boundaries. A schematic sheet shard, board shard,
relationship shard, variant shard, manufacturing-plan shard, transaction
journal shard, artifact metadata shard, or import-map shard can own source
bytes, but none can make facts authoritative without resolution into the
`DesignModel`.

### 0.3 Source Shards As Persistence Partitions

Target shard classes:

- Project manifest: project identity, schema/storage versions, shard index,
  accepted transaction tip, active variants/output contexts.
- Electrical sheets: authored schematic presentation and electrical intent.
- Physical boards: authored board implementation, not full project truth.
- Library bindings: project-local part/package/symbol/footprint/pad-map
  bindings and pinned reusable library revisions.
- Relationships: authored cross-domain relationship records and accepted
  deviations, partitioned by stable owner identity.
- Rules/check basis: rules, constraints, standards/process basis, waivers, and
  deviation records.
- Variants: sparse authored overlays over stable object identity.
- Manufacturing plans and output jobs: authored production and artifact
  generation recipes.
- Import Map: imported-source provenance keyed by `import_key`.
- Transactions: append-only journal and durable undo/redo cursors.
- Artifact metadata: generated-output manifests keyed to revisions.
- Derived caches: resolved connectivity, relationship status, zone fills,
  check runs, and manufacturing projections. These are cache/report state, not
  source authority.

### 0.4 Identity And Revision Substrate

All authored source-domain objects have a persisted stable `ObjectId`.
For the first target slice, `ObjectId = Uuid` on the wire. Identity is never
derived from file path, reference designator, display name, net label, array
order, sheet id, board id, or canvas position.

```rust
pub type ObjectId = Uuid;
pub type NetId = Uuid;

pub struct ObjectRevision(pub u64);
pub struct ModelRevision(pub String); // deterministic sha256 over object revisions + accepted transaction tip

pub struct RevisionedRef {
    pub object_id: ObjectId,
    pub object_revision: ObjectRevision,
}
```

Transactions are the sole producer of `ObjectRevision` and `ModelRevision`.
An object revision is a monotonic per-object `u64` bumped by the transaction
that changes that object. The model revision identifies the resolved project
state after applying accepted transactions and resolving all source shards.
Derived caches, proposals, check runs, artifacts, and query envelopes must
record the `model_revision` and relevant input object revisions they used.

### 0.5 Component, Net, And Relationship Identity

`ComponentInstance` is the canonical electrical-to-physical join. It replaces
reference-designator or name matching for cross-domain identity.

```rust
pub struct ComponentInstance {
    pub id: ObjectId,
    pub part_ref: Option<ObjectId>,
    pub placed_symbol_refs: Vec<ObjectId>,
    pub placed_package_refs: Vec<ObjectId>,
    pub object_revision: ObjectRevision,
}

pub enum NetAnchorKind {
    Label(ObjectId),
    WireEndpoint { wire: ObjectId, endpoint_index: u8 },
    Pin(ObjectId),
    Explicit(ObjectId),
}

pub struct NetAnchor {
    pub net_id: NetId,
    pub anchor: NetAnchorKind,
    pub derived_from_model_revision: ModelRevision,
}
```

A `NetId` is stable across rename and deterministic across split/merge through
`NetAnchor`. The resolver binds each resolved connectivity group to a `NetId`
through surviving or re-derived anchors. Display net names are properties, not
identity.

Relationships are split into authored records and derived resolver status.

```rust
pub struct Relationship {
    pub id: ObjectId,
    pub kind: RelationshipKind,
    pub from: Vec<RevisionedRef>,
    pub to: Vec<RevisionedRef>,
    pub authored_intent: Vec<AuthoredIntentRecord>,
    pub object_revision: ObjectRevision,
}

pub enum RelationshipKind {
    ImplementedBy,
    BoardOnly,
    SchematicOnly,
    ReverseEngineered,
    Pending,
    Mismatch,
}

pub enum DerivedRelationshipStatus {
    Implemented,
    PendingImplementation,
    UnresolvedMismatch,
}

pub enum AuthoredIntentRecord {
    LayoutDeviation { rationale: String, accepted_by: String },
    AcceptedDeviation { rationale: String, accepted_by: String },
    Waiver { waiver_id: ObjectId },
}

pub enum DerivedVariantPopulation {
    Applicable,
    NotApplicableForVariant,
}
```

`RelationshipKind` is authored source state. `DerivedRelationshipStatus` and
`DerivedVariantPopulation` are resolver outputs; they may be cached or stamped
into reports/artifacts, but no operation may persist them as relationship
authority. `ReverseEngineered` is a `RelationshipKind`, not a second deviation
axis. Import confidence, source basis, and repair history live in Import Map
and provenance metadata.

### 0.6 Sparse Variant Overlays

Variants are sparse overlays keyed by stable object identity. Switching
variants composes the overlay over the base `DesignModel`; it must not write
base objects or persist derived applicability values.

```rust
pub struct VariantOverlay {
    pub id: ObjectId,
    pub name: String,
    pub base_model_revision: ModelRevision,
    pub variant_revision: ObjectRevision,
    pub fitted: HashMap<ObjectId, FittedState>,
    pub relationship_overrides: HashMap<ObjectId, RelationshipOverride>,
    pub property_overrides: HashMap<ObjectId, serde_json::Value>,
}

pub enum FittedState {
    Fitted,
    Unfitted,
}
```

Variant overlays are invalidated by `model_revision` and
`variant_revision`. `NotApplicableForVariant` is derived from overlay,
option-link, and scope resolution and must never be stored as source
relationship status.

### 0.7 Import Map And Provenance

Imported identity resolves through an Import Map shard keyed by `import_key`,
not by `source_hash`.

```rust
pub struct ImportMapEntry {
    pub import_key: String,
    pub object_id: ObjectId,
    pub source_tool: String,
    pub source_path: Option<PathBuf>,
    pub source_object_ref: Option<String>,
    pub source_hash: Option<String>, // evidence only, never identity
    pub provenance: ImportProvenance,
}
```

Deterministic source UUIDs from KiCad/Eagle/importers are seeds and evidence.
They are not Datum identity once imported. Re-import and repair workflows must
reuse Datum `ObjectId`s by `import_key` and record unknown-basis markers when
source evidence is incomplete.

### 0.8 Operations, Proposals, Transactions, And Commit

All mutation surfaces emit typed operations over `ObjectId`s and revisions.

```rust
pub enum Operation {
    CreateObject { object: DomainObject },
    UpdateObject { id: ObjectId, expected_revision: ObjectRevision, patch: serde_json::Value },
    DeleteObjects { ids: Vec<ObjectId> },
    BindComponentInstance { component_instance_id: ObjectId, refs: Vec<ObjectId> },
    UpdateRelationship { relationship_id: ObjectId, patch: serde_json::Value },
    UpdateVariantOverlay { variant_id: ObjectId, patch: serde_json::Value },
    UpdateOutputJob { output_job_id: ObjectId, patch: serde_json::Value },
}

pub struct OperationBatch {
    pub id: Uuid,
    pub operations: Vec<Operation>,
    pub prepared_against: ModelRevision,
    pub provenance: OperationProvenance,
}

pub struct Proposal {
    pub id: Uuid,
    pub batch: OperationBatch,
    pub rationale: String,
    pub affected_objects: Vec<ObjectId>,
    pub checks_run: Vec<Uuid>,
    pub status: ProposalStatus,
}

pub struct Transaction {
    pub id: Uuid,
    pub parent_transaction_ids: Vec<Uuid>,
    pub batch: OperationBatch,
    pub affected_objects: Vec<RevisionedRef>,
    pub resulting_model_revision: ModelRevision,
    pub inverse_batch: Option<OperationBatch>,
}
```

There is exactly one mutation gateway:

```rust
pub fn commit(batch: OperationBatch) -> Result<Transaction>;
```

`commit()` validates expected object revisions and prepared-against model
revision, applies the batch to the resolved model in memory, stages touched
source-shard bytes, fsyncs the staged bytes, appends one `TransactionRecord`
to the journal and fsyncs it as the commit point, atomically renames staged
shards into place, then fsyncs containing directories. On project open,
recovery replays or rolls back the journal tail so the resolver only opens a
state produced by a committed transaction; otherwise it enters recovery mode.

Manual local visible edits may direct-commit by policy. AI/assistant,
import-repair, checker, high-risk, destructive, batch, and cross-domain edits
must produce a `Proposal` first and apply only after acceptance. Undo and redo
are compensating `OperationBatch` transactions through the same `commit()`;
there are no per-domain private undo/redo paths.

### 0.9 Zone And ZoneFill Honesty

`Zone` is authored boundary state. `ZoneFill` is derived projection state.
Copper checks and manufacturing exports may treat zone copper as real only
when a current `ZoneFill{Filled}` exists for the relevant model/projection
revision.

```rust
pub enum ZoneFillState {
    Filled,
    Unfilled,
    Stale,
    Unsupported,
}

pub struct ZoneFill {
    pub zone_id: ObjectId,
    pub state: ZoneFillState,
    pub source_zone_revision: ObjectRevision,
    pub model_revision: ModelRevision,
    pub islands: Vec<Polygon>,
    pub provenance: Option<ImportProvenance>,
}
```

Native unfilled/stale/unsupported zones emit no copper plus a hard
`CheckFinding` for fabrication/manufacturing contexts. Imported filled zones
may preserve source-tool islands as `Filled` derived geometry with provenance;
they remain derived-with-provenance, not authored board truth. Exporting a
zone boundary polygon directly as copper is a **Current implementation defect**
and is not target behavior.

### 0.10 Artifacts, Output Jobs, Checks, And Findings

`OutputJob` is authored source state. Generated files are artifacts: derived
snapshots with metadata, never design authority.

```rust
pub struct OutputJob {
    pub id: ObjectId,
    pub name: String,
    pub include: Vec<ArtifactKind>,
    pub board_or_panel: ObjectId,
    pub variant: Option<ObjectId>,
    pub manufacturing_plan: Option<ObjectId>,
    pub object_revision: ObjectRevision,
}

pub struct ArtifactMetadata {
    pub artifact_id: Uuid,
    pub kind: ArtifactKind,
    pub project_id: Uuid,
    pub model_revision: ModelRevision,
    pub output_job: Option<ObjectId>,
    pub variant: Option<ObjectId>,
    pub generator_version: String,
    pub files: Vec<ArtifactFile>,
    pub validation_state: ArtifactValidationState,
}

pub struct ArtifactFile {
    pub path: PathBuf,
    pub sha256: String,
}
```

Artifact generation uses one projection/oracle per run: generate, manifest,
validate, compare, and inspect must answer from the same model revision,
output job, variant, and generator version. BOM/PnP rows are keyed by
`ComponentInstance`, never reference string.

Checks are read-only derived work over revisions. Persisted check results are
evidence, not source authority.

```rust
pub struct CheckRun {
    pub id: Uuid,
    pub model_revision: ModelRevision,
    pub projection_revision: Option<ModelRevision>,
    pub checker_version: String,
    pub profile: String,
    pub status: CheckRunStatus,
    pub findings: Vec<CheckFinding>,
}

pub struct CheckFinding {
    pub id: Uuid,
    pub fingerprint: String,
    pub affected_objects: Vec<RevisionedRef>,
    pub rule_basis: String,
    pub severity: String,
    pub explanation: String,
    pub suggested_next_action: Option<String>,
}
```

Finding fingerprints must be deterministic for identical `model_revision`,
variant, and rule/checker versions. Waivers, proposals, and repairs target
stable finding fingerprints and affected `ObjectId`s, not `(domain, index)`.

### 0.11 Implementation Slices

Foundational slices, in dependency order:

1. Add `ObjectId`, `ObjectRevision`, `ModelRevision`, `ComponentInstance`,
   `NetId`, `NetAnchor`, and Import Map `import_key`.
2. Add `ProjectResolver` over source shards and preserve old project readers as
   compatibility loaders feeding the resolver.
3. Introduce typed `Operation` / `OperationBatch`, `Proposal`, and the single
   fsync-journaled `commit()` path.
4. Migrate current imported-board transactions and native JSON writers behind
   `commit()`; forbid private source-shard writes.
5. Add relationship records, derived relationship status, sparse variants, and
   revision-keyed invalidation.
6. Add honest `ZoneFill` derived state before treating zone copper as
   fabrication-valid.
7. Add `OutputJob`, artifact metadata, `CheckRun`, and `CheckFinding`
   revision/fingerprint addressing.

## 1. Core Types

Status: **Current implementation / Compatibility**, except where a type is
referenced by Section 0 as target substrate. The type catalog below documents
the current IR shape used by engine code and import/export paths. It should be
migrated behind `DesignModel` / `ProjectResolver`, not treated as a competing
authority model.

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

Status: **Current implementation / Compatibility**. `Board` currently groups
physical implementation state for imports, queries, DRC, and export. Target
implementation keeps board data as physical source shards resolved into
`DesignModel`; cross-domain joins must use `ComponentInstance`, `NetId`, and
`Relationship`, not reference strings or board-local assumptions.

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

Status: **Current implementation / Compatibility**. `Schematic` currently
groups electrical authoring/query state. Target implementation keeps schematic
sheets as electrical source shards resolved into `DesignModel`; variant
handling migrates from dense fitted-component maps to sparse
`VariantOverlay`, and schematic/PCB joins migrate to `ComponentInstance` and
`Relationship`.

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
    ProcessAperture,
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
    ProcessAperture { min_mask_expansion: i64, min_paste_reduction: i64 },
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

Status: **Current implementation / Compatibility**. The trait and `Transaction`
shape below describe the existing operation model. Target implementation is the
Section 0 `Operation` / `OperationBatch` / `Proposal` / single `commit()` model
with durable fsync journal and revision production.

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

This section tracks contract layers:
- **Target product-mechanics substrate (authoritative for new work)**:
  Section 0 `DesignModel`, `ProjectResolver`, source shards, stable identity,
  `OperationBatch`, `Proposal`, single `commit()`, journal/recovery,
  `CheckRun`, `ArtifactMetadata`, and `OutputJob`.
- **Current implementation contract (authoritative for code today)**:
  methods implemented in `crates/engine/src/api/mod.rs`.
- **Compatibility contract**: old Board/Schematic/KiCad/import APIs retained
  while migrated behind the target substrate.
- **Legacy Target M2+ contract**: fuller pre-product-mechanics method surface
  required by earlier milestone exit gates. It is superseded for new design by
  Section 0 but remains useful for compatibility planning.

Unless explicitly marked as `Target M2+`, method signatures in this section
refer to the current implementation contract.

### 5.1 Current Implemented Engine API

The total `pub fn` surface across `crates/engine/src/api/` (excluding
tests) is **locked via** `specs/SPEC_PARITY.md` → `engine_api_pub_fns`.
At the time of last refresh the inventory contains 64 methods. Any
add/rename/remove must refresh that inventory in the same change
(`python3 scripts/check_spec_parity.py --update`).

Methods are grouped below by defining file. The Engine struct itself is
defined in `crates/engine/src/api/mod.rs`.

#### `api/mod.rs` — Engine lifecycle and undo/redo flags

| Method | Notes |
|--------|-------|
| `new` | Construct an `Engine` with an in-memory pool |
| `has_open_project` | True once a design has been imported or natively opened |
| `can_undo` | True if the undo stack is non-empty |
| `can_redo` | True if the redo stack is non-empty |

#### `api/project_surface.rs` — Project lifecycle, pool access, explanations

Status: **Current implementation / Compatibility**. `import` and library import
APIs remain supported inputs, but target imported identity is the Import Map
`import_key` flow in Section 0.7. `explain_violation` is compatibility; target
explanations live on `CheckFinding`.

| Method | Notes |
|--------|-------|
| `import` | Import a KiCad or Eagle design |
| `close_project` | Drop the open design and reset undo/redo stacks |
| `import_eagle_library` | Import an Eagle `.lbr` library into the pool |
| `import_kicad_footprint` | Import a KiCad footprint into the pool |
| `search_pool` | Search pool by keyword/criteria |
| `get_part` | Resolve part detail by UUID |
| `get_package` | Resolve package detail by UUID |
| `explain_violation` | Produce a `ViolationExplanation` for an ERC/DRC finding |

#### `api/project_surface/project_surface_replacements.rs` — Replacement planning surfaces

| Method | Notes |
|--------|-------|
| `get_package_change_candidates` | Component-scoped package compatibility report |
| `get_part_change_candidates` | Component-scoped part compatibility report |
| `get_component_replacement_plan` | Unified replacement planning report |
| `get_scoped_component_replacement_plan` | Scoped preview using policy + filter |
| `edit_scoped_component_replacement_plan` | Apply exclusions / overrides to a preview |

#### `api/query_surface.rs` — Read-only queries and checks

| Method | Notes |
|--------|-------|
| `board` | Borrow the open board |
| `get_board_summary` | Board dimensions and counts |
| `get_components` | All components in the open design |
| `get_net_info` | Full board-net inventory (not a single-net selector) |
| `get_stackup` | Stackup definition |
| `get_unrouted` | Airwires |
| `get_schematic_summary` | Schematic counts and structure |
| `get_sheets` | Schematic sheet list |
| `get_symbols` | Symbol inventory (all-sheets in current slice) |
| `get_symbol_fields` | Fields for one symbol UUID |
| `get_ports` | Hierarchical ports |
| `get_labels` | Net labels |
| `get_buses` | Bus declarations |
| `get_bus_entries` | Bus entry wires (all-sheets form used by daemon/MCP) |
| `get_noconnects` | No-connect markers |
| `get_hierarchy` | Sheet hierarchy |
| `get_schematic_net_info` | Full schematic-net inventory |
| `get_netlist` | Deterministic board/schematic netlist inventory |
| `get_connectivity_diagnostics` | Connectivity diagnostics |
| `get_design_rules` | Resolved rule set |
| `get_check_report` | Unified ERC + DRC report |
| `run_erc_prechecks` | ERC with default config |
| `run_erc_prechecks_with_config` | ERC with explicit `ErcConfig` |
| `run_erc_prechecks_with_config_and_waivers` | ERC with config + waivers |
| `run_drc` | DRC over the requested rule types (connectivity, clearance, track width, via-hole, via-annular, silk-clearance, process-aperture currently) |

#### `api/save_kicad.rs` (and `api/save_kicad/`) — KiCad write-back

Status: **Compatibility**. KiCad save/write-back is an interchange path, not
target authority. Target source writes go through `OperationBatch -> commit()`
and source shards; KiCad emitters become projections/exporters or compatibility
writers.

| Method | Notes |
|--------|-------|
| `save` | Save modifications to a new path |
| `save_to_original` | Save back to the original imported file path |

#### `api/write_ops/basic_mutations.rs` — Basic board mutations

Status: **Current implementation / Migration input**. These methods are useful
operation seeds, but target mutations are typed `Operation` variants committed
through the single journaled `commit()` path.

| Method | Notes |
|--------|-------|
| `delete_component` | Delete one component as one transaction |
| `delete_track` | Delete one track as one transaction |
| `delete_via` | Delete one via as one transaction |
| `move_component` | Move one component (optionally with rotation) |
| `rotate_component` | Rotate one component |
| `set_value` | Set one component value |
| `set_reference` | Set one component reference |

#### `api/write_ops/assign_package_rule.rs` — Assignment / rule mutations

| Method | Notes |
|--------|-------|
| `assign_part` | Assign one part to one component |
| `set_package` | Assign one package to one component |
| `set_net_class` | Set one net class |
| `set_design_rule` | Set one design rule |

#### `api/write_ops/component_replacements.rs` — Component replacement applies

| Method | Notes |
|--------|-------|
| `set_package_with_part` | Replace package and explicit compatible part |
| `replace_component` | Replace one component with explicit package + part |
| `replace_components` | Batched explicit replacements as one transaction |
| `apply_component_replacement_plan` | Apply a unified replacement plan |
| `apply_component_replacement_policy` | Apply a best-candidate policy |
| `apply_scoped_component_replacement_policy` | Apply a scoped policy across matching components |
| `apply_scoped_component_replacement_plan` | Apply a previewed scoped plan |

#### `api/write_ops/undo_redo/` — Transaction reversal

| Method | Notes |
|--------|-------|
| `undo` | Reverse the most recent transaction |
| `redo` | Re-apply the most recent undone transaction |

#### Cross-cutting notes

- `get_net_info` and `get_schematic_net_info` return full inventories
  for the open design, not single-net selectors.
- `get_symbol_fields` is UUID-targeted.
- `get_netlist` returns deterministic board/schematic inventories rather
  than a richer graph object.
- DRC currently covers connectivity, clearance, track width, via-hole,
  via-annular, silk-clearance, and process-aperture checks.
- Native-authoring surfaces (place/edit/delete schematic and board objects,
  forward-annotation lifecycle, routing kernel queries, gerber/drill/BOM/PnP
  exports, route-strategy reporting, GUI review) are exposed through the
  CLI/MCP layers and consume the engine through its `Project` types and
  on-disk native scaffold rather than as additional `Engine::*` methods.
  Surface counts for those layers are tracked separately via
  `cli_project_commands` (182) and `mcp_runtime_methods` (75) in
  `specs/SPEC_PARITY.md`.

### 5.2 Target M2+ Engine API

Status: **Legacy target / Compatibility planning**. This was the pre-product-
mechanics target method surface. New implementation must not add parallel
per-domain commit/save/query authorities here; it should implement Section 0
and expose compatible wrappers where needed.

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
