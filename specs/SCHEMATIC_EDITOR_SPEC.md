# Schematic Editor Specification

## 1. Purpose

Defines the authored schematic object model and editing operation surface.
This specification is the schematic-editor peer to the board editing model.

The schematic editor must support:
- professional multi-sheet design entry
- deterministic connectivity generation
- ERC as a first-class checking flow
- forward/back annotation through stable object identity

---

## 2. Authored Object Model

### 2.1 Schematic Root

```rust
pub struct Schematic {
    pub uuid: Uuid,
    pub sheets: HashMap<Uuid, Sheet>,
    pub sheet_definitions: HashMap<Uuid, SheetDefinition>,
    pub sheet_instances: HashMap<Uuid, SheetInstance>,
    pub variants: HashMap<Uuid, Variant>,
    pub waivers: Vec<CheckWaiver>,
}
```

### 2.2 Sheet and Placement

```rust
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

pub struct SheetDefinition {
    pub uuid: Uuid,
    pub root_sheet: Uuid,      // -> Sheet
    pub name: String,
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
    pub definition: Uuid,   // -> SheetDefinition
    pub parent_sheet: Option<Uuid>,
    pub position: Point,
    pub name: String,
}
```

### 2.3 Symbol Placement

```rust
pub struct PlacedSymbol {
    pub uuid: Uuid,
    pub part: Option<Uuid>,         // -> Part
    pub entity: Option<Uuid>,       // -> Entity if part unresolved
    pub gate: Option<Uuid>,         // -> Gate for multi-gate entities
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
    pub pin: Uuid,              // -> Pin
    pub visible: bool,
    pub position: Option<Point>,
}

pub enum HiddenPowerBehavior {
    SourceDefinedImplicit,
    ExplicitPowerObject,
    PreservedAsImportedMetadata,
}
```

### 2.4 Connectivity-Bearing Objects

```rust
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
    pub bus: Uuid,              // -> Bus
    pub wire: Option<Uuid>,     // -> SchematicWire
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
    pub symbol: Uuid,           // -> PlacedSymbol
    pub pin: Uuid,              // -> Pin
    pub position: Point,
}
```

### 2.5 Non-Electrical Objects

```rust
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
```

---

## 3. Foundational Invariants

- Every placed symbol resolves to exactly one symbol definition and at most one
  part assignment at a time.
- Multi-gate devices must preserve stable gate identity across annotation and
  schematic↔board sync.
- No-connect markers target an exact symbol-pin pair.
- Buses are containers for scalar nets, not nets themselves.
- Sheet instances and hierarchical ports must resolve deterministically.
- Hidden power behavior imported from source formats must remain representable
  in authored data or preserved as source metadata until promoted.
- Reusable hierarchy is authored through stable sheet definitions and
  sheet instances, not by copying sheet graphics ad hoc.

---

## 4. Editing Operations

### 4.1 M4 Required Operations

```rust
PlaceSymbol
MoveSymbol
RotateSymbol
MirrorSymbol
DeleteSymbol

DrawWire
DeleteWire
PlaceJunction
DeleteJunction

PlaceLabel
RenameLabel
DeleteLabel
PlacePowerSymbol

CreateBus
EditBusMembers
PlaceBusEntry
DeleteBusEntry

CreateSheetInstance
MoveSheetInstance
DeleteSheetInstance

PlaceHierarchicalPort
EditHierarchicalPort
DeleteHierarchicalPort

PlaceNoConnect
DeleteNoConnect

SetFieldValue
MoveField
SetFieldVisibility

Annotate
AssignPart
AssignGate
```

These are the minimum operations for a first-class schematic editor. They are
the schematic-side peers to board placement/routing-independent editing.

### 4.2 Compound Operations

- `Annotate` may rename many references in one transaction
- `AssignPart` may update fields and gate/unit selection in one transaction
- `CreateSheetInstance` may also create initial port bindings

---

## 5. Annotation and Identity

- Reference annotation is authored data
- Annotation must be deterministic given the same sheet ordering and symbol set
- Gate assignment for multi-gate entities must survive undo/redo and ECO
- Renaming a symbol reference is not the same as re-creating the symbol
- Sheet-instance identity must survive hierarchy renames and instance moves

---

## 6. Editing Semantics

### 6.1 Snap and Attachment Rules

- Wire endpoints snap to symbol pins, junctions, wire endpoints, port anchors,
  and bus-entry anchors within the editor snap tolerance.
- Symbol movement preserves wire attachment only through explicit reconnect
  operations; moving a symbol does not silently drag unrelated geometry.
- Label, power-symbol, and port attachment semantics are connectivity-bearing,
  not cosmetic placement hints.

### 6.2 Wire Split and Merge

- Drawing a wire that terminates on an existing wire creates a junction only
  when the operation explicitly places one or the source format semantics
  require it.
- Deleting a junction may split one electrical segment into multiple segments.
- Collinear wire segments may be merged as a canonicalization step only if the
  resulting authored connectivity is unchanged.

### 6.3 Bus and Bus Entry Behavior

- A bus entry must reference exactly one bus and may reference at most one
  scalar wire.
- Editing bus members may invalidate existing bus entries; such invalidation is
  a validation error until resolved.
- Bus naming expansion must preserve source-format semantics where imported.

### 6.4 Field and Reference Semantics

- Symbol fields are authored objects with stable UUIDs and survive annotation.
- Moving a field changes only field presentation, not symbol identity.
- Renaming a reference updates authored reference data only; it must not create
  a new symbol or gate assignment.

### 6.5 Hidden Power and Imported Semantics

- Hidden power pins imported from source formats must remain recoverable as
  authored behavior until the designer promotes them to explicit objects.
- Promotion from implicit to explicit power connectivity is an authored
  operation and must be undoable.

### 6.6 Hierarchy Rename and Propagation

- Renaming a sheet definition updates all instances that reference it without
  changing instance UUIDs.
- Renaming a hierarchical port or label may require connectivity re-resolution
  across the hierarchy and must be validated before commit.

---

## 7. ERC-Relevant Editor Behavior

- A no-connect marker suppresses only the targeted unconnected-pin condition
- Power symbols are authored connectivity objects, not pure graphics
- Hierarchical ports are connectivity objects, not pure labels
- Bus entries are connectivity-shaping objects and must be preserved explicitly

---

## 8. M4 Exit Surface

Schematic authoring is baseline-complete for `M4` when:
- multi-sheet schematics can be created and edited
- connectivity resolves deterministically after edits
- ERC can run on authored native schematics
- forward annotation can produce a reviewable ECO for board creation
- all M4 required operations are exposed through engine API, CLI, and MCP
