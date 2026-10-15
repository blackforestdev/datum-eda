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

**Imported identity authority.** [PM000 Object Identity](decisions/PRODUCT_MECHANICS_000_UNIFIED_DESIGN_MODEL.md#object-identity)
owns the persisted-surrogate rule. Existing imported v5 values are retained as
initial persisted import seeds; this does not require a new or different UUID.
[ENGINE_SPEC §0.7](../specs/ENGINE_SPEC.md#07-import-map-and-provenance) owns
resolution through the persisted Import Map keyed by `import_key`, with
`source_hash` as evidence rather than identity. The stored mapping governs
subsequent native identity; source names, paths and hashes do not replace it.
[NATIVE_FORMAT_SPEC §6.9/§9](../specs/NATIVE_FORMAT_SPEC.md#9-imported-project-persistence)
owns source-origin metadata and native persistence. This is an authority/scope
clarification, not a new allocation, remapping or runtime qualification rule.

**Deterministic import seeds (compatibility algorithm)** from KiCad, Eagle,
etc. use UUID v5 (namespace-based). The seed value is derived from:
- Namespace: import format identifier (e.g., `kicad`, `eagle`)
- Name: the stable object identifier from the source format
  - KiCad: uses the file's internal UUID (KiCad assigns UUIDs to objects)
  - Eagle: uses element name + type path (e.g., `element:R1`, `signal:VCC`)

For this seed algorithm, **the same file and import code produce identical
UUID seed values**. The recorded compatibility rationale for deterministic
import identity was:
- Operation replay across sessions on imported projects
- MCP/CLI workflows that address objects by UUID
- Deterministic diffs between import runs

**Sidecar persistence (legacy compatibility description).** The following
M0–M3 description records the earlier sidecar contract; it does not establish
that this legacy runtime path remains active or govern native Import Map identity.
Before native format exists (M0-M3), imported
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

The exact operation, batch, proposal, transaction and commit contract is owned by
[ENGINE_SPEC §0.8](../specs/ENGINE_SPEC.md#08-operations-proposals-transactions-and-commit).
Every authored mutation uses that shared typed-operation path, including revision
preflight and the policy distinction between direct commits and proposals. This
companion does not maintain a second variant set or `OperationBatch` schema.

### CommitDiff

Change reporting and affected identities follow the engine-owned transaction
contract above. The former local `CommitDiff` sketch and `batch_id` example are
retired; neither defines a competing return type or persistence schema. Diffs
support invalidation and consumer reporting; undo uses compensating batches.

### Commit, journal, and revisions

The engine contract owns validation, journal commit point, recovery and
compensating undo/redo. Transactions alone produce object/model revisions;
no wall-clock or `Instant` field is stored in source state because it would
violate serialization determinism. [NATIVE_FORMAT_SPEC](../specs/NATIVE_FORMAT_SPEC.md#5-file-ownership)
owns persisted authored/derived boundaries, with explicit ZoneFill generated
evidence and revision/source qualification. Generated evidence stored on disk
does not become authored geometry. This document retains determinism and
incremental-recomputation rationale; PM053/054 reconciliations below remain.

Selection is a consumer subject, governed by
[PM026](decisions/PRODUCT_MECHANICS_026_SELECTION_IDENTITY.md#decision): an
Object/Compound resolves authored identity; Run/Global Net/Bus derive membership
under their revision law. Those subjects do not introduce authored source shards.
Their S5A read-only boundary and later atomic mutation guard remain distinct
from source persistence and engine transactions.

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
Native schema compatibility and migration are owned by
[NATIVE_FORMAT_SPEC §8](../specs/NATIVE_FORMAT_SPEC.md#8-versioning-and-migration).
Use its supported-version rules, unknown-future-version refusal and explicit
current-versus-target migration status; this IR rationale does not define a
second version-support policy.

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

## PM053 C01–C03 owning reconciliation — 2026-10-01

[PM053](/docs/decisions/PRODUCT_MECHANICS_053_ELECTRICAL_IDENTITY_AUTHORITY.md) ratifies stable NetId final-state atomic nomination/fallback,
explicit logical-to-board identity relationships (pending/mismatch are lawful
incomplete states), and semantic Bus declarations with occurrence-qualified
bindings. These clauses control conflicting earlier summaries in this document:
read/cache/reopen never allocate identity; display rename preserves identity,
while a label edit changing electrical groups follows recorded transitions.
Bus drawing split/member reorder does not split its declaration; semantic split/
merge explicitly records survivor and complete redistribution. A declaration
can resolve with zero drawings. Full interface equality differs from related
subset/remapping; names/member equality never establishes Bus identity and Bus
has no board projection. Occurrence identity is source plus stable instance path.
Malformed references refuse atomically; pending/mismatch must never be published
as complete cross-domain Net selection. Authored identity/binding source is
distinct from recomputable graph/projection geometry. Canonical writes and replay
retain all recorded identities. No C04–C07, M1 numerical policy, geometry/CAM
support, selection mutation or native acceptance is ratified by this amendment.


## PM054 C04–C06 owning reconciliation

The owner-ratified [PM054](decisions/PRODUCT_MECHANICS_054_NOMINAL_COPPER_DERIVED_AUTHORITY.md)
controls conflicting earlier contact/Run, Zone qualification and Track summaries
in this document. A Run is complete within its origin's resolved Net assignment
on actual conductive layers/current geometry; foreign-Net contacts remain separate
fault evidence. Unknown assignment or required geometry is typed unavailable.

One authored Zone identity has an engine-owned revision/basis-bound component
qualifier. Acquisition uses occupied copper and refuses ambiguous nonpoint origin.
Exact unchanged-region equivalence or validated generator/transaction lineage
can prove a unique successor; overlap/proximity/source survival alone cannot.
Split or verified deletion clears; current Unknown requests reacquisition; missing,
stale or unsupported basis suspends rather than proves deletion. Current solved
empty fill clears. Undo does not resurrect dissolved selection. Object projection
remains one authored Zone; Run projects only its acquired connected copper.

Generated ZoneFill version 2 permits successful Filled with zero islands, retaining
nonempty provenance and exact model/source-zone basis. Version 1 nonempty Filled
retains meaning; omitted version is legacy 1; old Unsupported is never empty.
New producers write version 2, and incompatible readers refuse explicitly.

Track may store an optional integer-nm on-arc midpoint; absence preserves straight
source. Distinct noncollinear start/mid/end define the directed locus, including
major sweeps, with existing UUID/Net/layer/width. Canonical writes, inverse, undo/
replay/reopen preserve these anchors. Nominal copper uses width/2 including half-
nm radii, certified pair predicates and checked arithmetic; rendering/chords/
rounded graphic angles cannot establish geometry or membership. Invalid source
refuses atomically; unsupported consumers disclose source-bound capability.

E2–E4 nominal execution and focused independent review are owner-authorized.
C07/M1 manufacturing policy and mandatory rational-center CAM/T04 remain deferred
and unpassed. This amendment supplies no approximation allowance, general CAM
claim, full foundation/S5A acceptance or selection-triggered design mutation.
