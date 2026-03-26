# Canonical Intermediate Representation

## Purpose
This document defines the invariants, identity model, precision model,
and data boundaries for the engine's internal representation. The IR is
what makes the system coherent — it defines not just object shapes, but
what must be true about them at all times.

---

## 1. Identity

### UUIDs
All authored objects have a stable UUID. UUIDs are immutable for the
lifetime of an object. They survive import, export, serialization,
undo/redo, and cross-reference.

**Native objects** (created in this tool): UUID v4, randomly generated
at creation time.

**Imported objects** (from KiCad, Eagle, etc.): UUID v5 (deterministic,
namespace-based). The UUID is derived from:
- Namespace: import format identifier (e.g., `kicad`, `eagle`)
- Name: the stable object identifier from the source format
  - KiCad: uses the file's internal UUID (KiCad assigns UUIDs to objects)
  - Eagle: uses element name + type path (e.g., `element:R1`, `signal:VCC`)

This guarantees: **the same file imported twice produces identical UUIDs.**
Deterministic import identity is required for:
- Operation replay across sessions on imported projects
- MCP/CLI workflows that address objects by UUID
- Deterministic diffs between import runs

**Sidecar persistence**: Before native format exists (M0-M3), imported
designs persist their UUID mapping in a sidecar file (`<filename>.ids.json`)
alongside the source file. This file maps source object identifiers to
assigned UUIDs. If the sidecar exists on re-import, UUIDs are restored
from it rather than recomputed (handles cases where the source format
lacks stable identifiers).

Derived objects (airwires, DRC markers, connectivity edges) do NOT have
stable UUIDs. They are recomputed and may change identity between runs.

### References
Objects reference each other by UUID, never by index, name, or position.
A Part references an Entity by UUID. A PlacedComponent references a Part
by UUID. A Track references a Net by UUID.

Dangling references (UUID points to nothing) are a hard error in validation,
not a silent degradation.

### Naming
Human-readable names (net names, reference designators, part MPNs) are
attributes, not identities. Two objects can have the same name. The engine
never uses names for lookup — only UUIDs.

---

## 2. Units and Precision

### Internal unit: nanometers (i64)
All coordinates, dimensions, and distances are stored as 64-bit signed
integers in nanometers. This provides:
- Sub-micron precision (1 nm resolution)
- No floating-point accumulation errors
- Exact comparison (a == b, not |a-b| < epsilon)
- Range: ±9.2 × 10⁹ mm = ±9,200 km (more than sufficient)

### Why not floating point
Floating point causes:
- DRC false positives from rounding (two pads that should be 0.1mm apart
  compute as 0.09999999mm and fail a 0.1mm clearance check)
- Non-deterministic serialization (same design, different string repr)
- Order-dependent geometry (A+B+C ≠ C+B+A)

Every serious EDA engine uses integer coordinates internally (KiCad: nm,
Altium: 0.01 mil ≈ 254 nm, Eagle: 1/320000 inch ≈ 79 nm).

### Angles
Angles stored as tenths of a degree (i32). Range: 0..3599.
0.1° resolution matches industry practice.
Rotation: 0 = right, 900 = up, 1800 = left, 2700 = down.

### User-facing units
The engine accepts and produces coordinates in user-preferred units
(mm, mil, inch) at the API boundary. Conversion happens at the edge,
never internally.

---

## 3. Authored vs. Derived Data

### Authored data (persisted, user-created)
- Pool items (units, entities, parts, packages, symbols, padstacks)
- Board: placed components, tracks, vias, zones (polygons), keepouts,
  dimensions, text, net class assignments, design rules, stackup
- Schematic: placed symbols, wires, junctions, labels, power symbols,
  buses, hierarchical block instances
- Check waivers/suppressions (ERC and DRC, domain-scoped)
- Project settings, output job configurations

### Derived data (computed, not persisted in design files)
- Connectivity graph (which pins connect to which nets via which paths)
- Schematic connectivity graph (resolved nets, pin attachments, hierarchical links)
- Airwires (unrouted connections — shortest-distance pairs)
- Copper connectivity islands (contiguous copper regions per net per layer)
- Zone fill geometry (pour result, not the zone polygon itself)
- DRC markers (violation locations and descriptions)
- ERC markers (violation locations and descriptions)
- Rule match cache (which rule applies to which object pair)
- BOM aggregation, net statistics, board summary

### Invariant
Derived data can always be recomputed from authored data alone.
If the engine loads a design file and recomputes all derived data,
the result must be identical to the previous computation (determinism).

Derived data may be cached to disk for performance (zone fills are
expensive) but the cache is always invalidated when authored data changes.

---

## 4. Transaction Model

### Operations
Every modification to authored data is an Operation. Operations are:

```rust
pub trait Operation: Send + Sync {
    /// Check preconditions without modifying state
    fn validate(&self, design: &Design) -> Result<(), OpError>;

    /// Execute and return what changed
    fn execute(&self, design: &mut Design) -> Result<OpDiff, OpError>;

    /// Inverse operation for undo (if possible)
    fn inverse(&self, diff: &OpDiff) -> Option<Box<dyn Operation>>;

    /// Human-readable description
    fn describe(&self) -> String;

    /// Serializable representation (for operation log)
    fn serialize(&self) -> serde_json::Value;
}
```

### OpDiff
Every operation returns a diff describing what changed:
```rust
pub struct OpDiff {
    pub created: Vec<(ObjectType, Uuid)>,
    pub modified: Vec<(ObjectType, Uuid, serde_json::Value)>,  // old value
    pub deleted: Vec<(ObjectType, Uuid, serde_json::Value)>,   // deleted object
}
```

The diff contains enough information to:
- Undo the operation (restore old values, delete created, recreate deleted)
- Notify derived-data engines which objects changed
- Send to GUI for incremental canvas update
- Send to MCP/CLI as structured change report

### Transaction boundaries
A single user action may produce multiple operations (e.g., "delete
component" also deletes its tracks and vias). These are grouped into
a transaction:

```rust
pub struct Transaction {
    pub operations: Vec<(Box<dyn Operation>, OpDiff)>,
    pub description: String,
    pub timestamp: Instant,
}
```

Undo/redo operates on transactions, not individual operations.

### Derived data invalidation
After a transaction commits, the engine recomputes affected derived data:
1. Connectivity: only nets touching modified objects
2. Airwires: only for nets with connectivity changes
3. Schematic connectivity: only sheets/nets touching modified schematic objects
4. ERC: only rules/findings whose scope includes modified schematic objects
5. DRC: only rules whose scope includes modified board objects
6. Zone fills: only zones intersecting modified geometry

This incremental approach is critical for interactive performance.

---

## 5. Serialization

### Determinism
Given the same authored data, serialization must produce byte-identical
output on every run, on every platform. This requires:
- Sorted map keys (alphabetical)
- Stable float representation (if any floats exist — prefer integers)
- No platform-dependent output (endianness, locale)
- No timestamp or random data in output

### Format: JSON
Design files are JSON. Chosen for:
- Human-readable, diffable, git-friendly
- AI-parseable without custom tooling
- Universal parser availability
- Adequate performance for PCB-scale data (not IC-scale)

### Schema versioning
Every JSON file includes a schema version number. The engine can read
any version >= 1 and always writes the current version. Migration logic
is explicit per-version, not implicit.

---

## 6. Rule Representation

The canonical rule model is an expression tree from M0. The data model
is always expression-based. Milestones expand the evaluator, not the
representation.

```rust
/// Rule scope — expression tree
pub enum RuleScope {
    // Leaf nodes (M2 evaluator)
    All,
    Net(Uuid),
    NetClass(Uuid),
    Layer(LayerId),

    // Combinators (M6+ evaluator)
    And(Box<RuleScope>, Box<RuleScope>),
    Or(Box<RuleScope>, Box<RuleScope>),
    Not(Box<RuleScope>),

    // Extended functions (M6+ evaluator)
    InComponent(Uuid),
    HasPackage(String),       // glob pattern
    NetNameRegex(String),
    IsDiffpair,
    IsVia,
    IsPad,
    IsSMD,
    InArea(Uuid),             // keepout or region UUID
}

/// A design rule
pub struct Rule {
    pub id: Uuid,
    pub name: String,
    pub scope: RuleScope,       // always an expression
    pub priority: u32,          // higher wins
    pub enabled: bool,
    pub rule_type: RuleType,    // clearance, width, via, etc.
    pub parameters: RuleParams, // type-specific values
}
```

The expression tree serializes to JSON. A rule scoped to
`InNetClass("HighSpeed") And OnLayer("Top")` is stored as:

```json
{
  "scope": {
    "And": [
      {"NetClass": "uuid-of-highspeed-class"},
      {"Layer": 1}
    ]
  }
}
```

**M2 evaluator** handles: All, Net, NetClass, Layer. If a rule uses
unsupported combinators, the evaluator returns a clear error ("rule
uses And combinator, not supported until M6") rather than silently
ignoring the scope.

**M6+ evaluator** handles the full expression tree.

This design means:
- Rules imported from complex designs are preserved, even if the
  current evaluator can't fully evaluate them.
- No data migration is ever needed — the IR is stable.
- The evaluator is the only thing that grows.

---

## 7. Coordinate System

- Origin: bottom-left of the board (matches Gerber convention)
- X: positive right
- Y: positive up
- Layers: numbered, with well-known names (Top=1, Bottom=N, Inner1..N-2)
- Board outline: closed polygon, counter-clockwise winding

---

## 8. Pool Index

The pool index is a SQLite database that caches metadata from pool JSON
files for fast querying. It is always rebuildable from the JSON files.

Key tables:
- units (uuid, name, manufacturer, filename, tags)
- entities (uuid, name, prefix, n_gates, filename, tags)
- symbols (uuid, unit_uuid, name, filename)
- packages (uuid, name, n_pads, filename, tags)
- parts (uuid, mpn, manufacturer, entity_uuid, package_uuid, description,
         parametric_table, tags, lifecycle)
- parametric_* (dynamic tables per parametric category)
- tags (uuid, type, tag) with FTS index

The pool index is derived data — it can be rebuilt from pool files.
