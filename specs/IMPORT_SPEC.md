# Import Specification

## 1. Deterministic UUID v5 Algorithm

This section records deterministic import-seed derivation and compatibility
examples, not the native identity authority or current runtime coverage.
[PM000 Object Identity](../docs/decisions/PRODUCT_MECHANICS_000_UNIFIED_DESIGN_MODEL.md#object-identity)
retains existing imported v5 values as initial persisted seeds; no new or
different UUID is required by this distinction.
[ENGINE_SPEC §0.7](ENGINE_SPEC.md#07-import-map-and-provenance) owns subsequent
identity resolution through the persisted Import Map keyed by `import_key`.
`source_hash` remains evidence, not its identity key. Preserve that mapping;
this documentation clarification introduces no allocation or remapping policy.

### Namespace UUIDs (fixed, hardcoded)
```
NAMESPACE_KICAD = UUID v5(DNS, "import.kicad.eda-tool")
NAMESPACE_EAGLE = UUID v5(DNS, "import.eagle.eda-tool")
```

### Object path construction

#### KiCad
KiCad files already contain UUIDs on most objects. Use them directly:

```
KiCad object has UUID → UUID v5(NAMESPACE_KICAD, kicad_uuid_string)
KiCad object has no UUID → UUID v5(NAMESPACE_KICAD, synthetic_path)

Synthetic paths:
  Net by name:       "net:{net_name}"
  Net class by name: "netclass:{class_name}"
  Layer by number:   "layer:{layer_number}"
  Design rule:       "rule:{rule_name}:{index}"
```

#### Eagle
Eagle XML uses element names, not UUIDs. Object paths are constructed
from the XML hierarchy:

```
Library part:       "lbr:{library_name}:{deviceset_name}:{device_name}"
Board element:      "element:{element_name}"
Board signal:       "signal:{signal_name}"
Signal wire:        "signal:{signal_name}:wire:{index}"
Signal via:         "signal:{signal_name}:via:{index}"
Signal contactref:  "signal:{signal_name}:contactref:{element}:{pad}"
Net class:          "class:{class_number}:{class_name}"
Design rule param:  "designrule:{param_name}"
```

The `index` for wires/vias is the positional index within the XML
`<signal>` element, which is stable for unmodified files.

### UUID computation
```rust
fn import_uuid(namespace: Uuid, object_path: &str) -> Uuid {
    Uuid::new_v5(&namespace, object_path.as_bytes())
}
```

### Guarantee
The recorded compatibility guarantee is: same file, same import code → identical
UUID seed values. Its original test description imports the same file twice and
asserts byte-identical output; no current execution or qualification result is
asserted here. Native identity continuity follows the persisted mapping above.

---

## 2. Sidecar .ids.json

The schema, precedence, source-path rename rationale and file location below are
retained legacy compatibility descriptions. They do not establish that this
sidecar runtime remains active, nor define native identity resolution.
[NATIVE_FORMAT_SPEC §6.9/§9](NATIVE_FORMAT_SPEC.md#9-imported-project-persistence)
owns preservation of source-origin identity metadata and native persistence.
The target Import Map follows PM000/ENGINE §0.7 above: `import_key` resolves the
stored mapping; a source-hash refresh does not become the native identity law.
[PM011](../docs/decisions/PRODUCT_MECHANICS_011_IMPORT_INTEROP_ROLE.md#decision)
retains native authority after conversion and the import freeze. Nothing here
changes a schema, implements conversion or authorizes source-format write-back.
These compatibility labels apply only to the seed/sidecar material in §§1–2;
other import contracts retain their existing scope and authority.

### Purpose
Persist UUID assignments alongside source files for cross-session
stability, especially for objects whose source-format identifiers
are positional (Eagle wire indices) and may shift on re-save.

### Schema
```json
{
  "schema_version": 1,
  "format": "kicad",
  "source_file": "board.kicad_pcb",
  "source_hash": "sha256:abc123...",
  "generated_at": "2026-03-24T12:00:00Z",
  "mappings": {
    "object_path_1": "uuid-1",
    "object_path_2": "uuid-2"
  }
}
```

### Precedence rules
1. If .ids.json exists AND source_hash matches current file:
   → use sidecar UUIDs (exact restoration)
2. If .ids.json exists AND source_hash does NOT match:
   → recompute UUIDs via UUID v5 algorithm
   → merge: objects present in both get sidecar UUID,
     new objects get computed UUID, deleted objects are dropped
   → write updated .ids.json
3. If .ids.json does not exist:
   → compute all UUIDs via UUID v5 algorithm
   → write .ids.json

### Rename behavior
The original compatibility rationale below describes source-path seed changes.
It is not a native identity rule or an instruction to delete/recreate a native
object on display rename; the persisted Import Map remains authoritative.

If an Eagle element is renamed (e.g., R1 → R101), its object path
changes, producing a new UUID. The old UUID is lost. This is correct
behavior — the import treats it as a delete + create, which matches
the engineering reality (renaming a component IS a design change).

For KiCad, renames don't affect UUIDs because KiCad uses internal
UUIDs, not names, as the identity source.

### File location
`.ids.json` is written in the same directory as the source file:
```
project/
  board.kicad_pcb
  board.kicad_pcb.ids.json
```

---

## 3. KiCad Import Feature Matrix

Target: KiCad 7, 8, 9 (.kicad_pcb v20221018+, .kicad_sch v20230121+)

### Board (.kicad_pcb)

| Feature | M1 | Notes |
|---------|-----|-------|
| Board outline | Required | |
| Components (footprints) | Required | position, rotation, layer, reference, value |
| Tracks | Required | width, layer, net |
| Vias | Required | drill, diameter, net, layer span |
| Zones | Required | polygon, net, priority, thermal settings |
| Zone fills | Ignored | Derived data — recomputed by engine |
| Net classes | Required | clearance, width, via rules |
| Design rules | Best-effort | KiCad custom constraints → approximate mapping |
| Stackup | Required | Layer count, thickness, names |
| Stackup materials | Deferred | Dk/Df not in KiCad format before v8 |
| Keepouts | Required | |
| Dimensions | Required | |
| Text | Required | |
| Drawings (lines, arcs) | Required | |
| Groups | Best-effort | |
| Teardrops | Deferred | Import as track geometry, lose tuning metadata |
| 3D model references | Deferred | No 3D in v1 |
| Custom DRC rules | Deferred | KiCad's constraint language is complex |

### Schematic (.kicad_sch)

| Feature | M1 | Notes |
|---------|-----|-------|
| Symbols | Required | Position, rotation, reference, value |
| Pin electrical types | Required | Required for ERC |
| Wires | Required | |
| Junctions | Required | |
| Labels (net, global, hierarchical) | Required | |
| Power symbols | Required | |
| Buses | Required | Scalar member expansion required for ERC |
| Hierarchical sheets | Required | Sub-sheet references resolve |
| Hierarchical ports | Required | Parent/child port mapping required for ERC |
| No-connect flags | Required | |
| Hidden power pins | Best-effort | Preserve when source exposes them |
| Text, drawings | Required | |

### Library (.kicad_sym, .kicad_mod)

| Feature | M0 | Notes |
|---------|-----|-------|
| Symbols (pins, graphics) | Required | |
| Footprints (pads, silk, courtyard) | Required | |
| 3D model references | Deferred | |
| Symbol alternates | Best-effort | |
| SPICE models | Best-effort (M7+) | Imported from `.kicad_sym` simulation-model URI; attached as `Part.behavioural_models` per `ENGINE_SPEC.md` § 1.2 |
| IBIS models | Best-effort (M7+) | Imported from `.kicad_sym` IBIS-model URI when present; attached as `Part.behavioural_models` |
| Touchstone files | Best-effort (M7+) | Imported from `.kicad_sym` Touchstone-model URI when present; attached as `Part.behavioural_models` |

---

## 4. Eagle Import Feature Matrix

Target: Eagle 6.x through 9.6.2 (XML, DTD-defined)

### Board (.brd)

| Feature | M1 | Notes |
|---------|-----|-------|
| Board outline (layer 20) | Required | |
| Elements (components) | Required | position, rotation, mirror, value |
| Signals (nets) | Required | wires, vias, contactrefs |
| Signal wires (tracks) | Required | width, layer |
| Signal vias | Required | drill, diameter, layer extent |
| Signal polygons (zones) | Required | pour settings |
| Net classes | Required | |
| Design rules | Best-effort | Param-based → approximate mapping |
| Layers | Required | Standard Eagle layer mapping |
| Keepouts (restrict layers) | Required | |
| Text, dimensions | Required | |
| Holes | Required | |

### Schematic (.sch)

| Feature | M1 | Notes |
|---------|-----|-------|
| Instances (placed symbols) | Required | |
| Pin electrical types | Required | Required for ERC |
| Nets (segments, wires, junctions) | Required | |
| Net labels | Required | |
| Buses | Best-effort | |
| Power symbols | Required | |
| Sheets (multi-sheet) | Required | |
| Modules (hierarchical) | Best-effort | Late addition to Eagle |
| No-connect semantics | Required | Required for ERC |

### Library (.lbr)

| Feature | M0 | Notes |
|---------|-----|-------|
| Symbols | Required | |
| Packages (footprints) | Required | Pads, SMDs, silkscreen |
| Devicesets → Entity mapping | Required | gates, connect pin→pad |
| Technology variants | Best-effort | |
| 3D packages | Deferred | |
| Spice models | Best-effort (M7+) | Eagle stores SPICE in `<spice><pinmapping/><model/></spice>` blocks; imported as `Part.behavioural_models` attachments per `ENGINE_SPEC.md` § 1.2 |
| Ibis models | Best-effort (M7+) | Eagle deviceset attachment fields, when present; imported as `Part.behavioural_models` attachments |

---

## 5. IPC-2581 Import (Future — Post-M7)

Target: IPC-2581 Rev B and Rev C archives (single-XML DPMX format).

### Why imported as a first-class format

IPC-2581 import is the **practical Altium / OrCAD / PADS migration
path** that does not require reverse-engineering commercial native
binary formats. Every Tier-1 commercial EDA tool can export
IPC-2581 Rev C. Datum's IPC-2581 importer is the receiving end of
that migration without taking on binary-format reverse-engineering
risk.

This is also the operational reason the commercial native binary
formats (Altium `.PcbDoc`, OrCAD/Allegro binaries, PADS binaries)
remain `Out of scope` per `STANDARDS_COMPLIANCE_SPEC.md` § 4.1: the
intermediate-format path covers most user-migration scenarios at a
fraction of the engineering and licensing risk.

### Feature Matrix (target)

| Feature | Required | Notes |
|---|---|---|
| Board outline | Required | |
| Stackup (with materials) | Required | Maps to extended `StackupLayer` material fields per `ENGINE_SPEC.md` § 1.3 |
| Components | Required | Placement, rotation, layer |
| Tracks (per-layer) | Required | |
| Vias | Required | Including blind/buried |
| Zones (filled polygons) | Required | |
| Net list | Required | With component-pin connectivity |
| Net classes | Required | |
| Controlled impedance | Required | Maps to `Net.controlled_impedance` per `ENGINE_SPEC.md` § 1.3 |
| Diff pairs | Required | Maps to NetClass `diffpair_width` / `diffpair_gap` |
| BOM | Best-effort | Imported as Part metadata enrichment |
| Pad-stack data | Required | Maps to Datum's Padstack pool entries |
| Drill data | Required | |
| Fabrication / assembly notes | Best-effort | Stored as project-level metadata |
| Embedded components | Deferred | M8+ |
| Cavities | Deferred | M8+ |
| DFM annotations | Deferred | One-way fab → designer |

### Round-Trip Note

IPC-2581 is intended for design exchange, not for full design-tool
round-trip. Imported IPC-2581 projects become native Datum projects
on import; the path back to the originating commercial tool is
through the originating tool's own IPC-2581 import (when supported)
rather than through a Datum-side write-back guarantee.

---

## 6. Round-Trip Guarantees

### KiCad write-back (M3)

| Guarantee | Level |
|-----------|-------|
| Unmodified objects | Byte-identical in re-exported .kicad_pcb |
| Modified objects | Correct values, formatting may differ |
| Added objects | Valid KiCad syntax, opens without errors |
| Deleted objects | Absent from output |
| Object ordering | May differ from original (sorted by UUID) |
| Comments/whitespace | Not preserved |
| Opens in KiCad | Required — 0 errors, 0 warnings |

### Accepted lossiness

These are known losses that are documented, not bugs:

| Lost on import | Reason |
|---------------|--------|
| Zone fill geometry | Derived data — recomputed |
| Teardrop metadata | Not yet supported |
| 3D model assignments | Not in v1 scope |
| Custom DRC constraint expressions | Complex mapping deferred |
| Some source-specific ERC cosmetic markers | Preserved only if electrically meaningful |
| KiCad-specific rendering hints | Not applicable |
| Eagle ULP-generated content | No ULP runtime |
| Eagle Fusion 360 cloud references | Inaccessible URN assets |

## Saved Zone copper in native adoption (PM056)

[PM056](../docs/decisions/PRODUCT_MECHANICS_056_IMPORTED_ZONE_FILL_ADOPTION.md)
controls the bounded S5A-C03 native-fill repair. Preserve authored Zone boundary
and identity separately from complete saved filled islands. Validated saved
copper enters the engine's generated-evidence owner through the same canonical
import batch, bound to source hash and final native revisions. Native edits make
that evidence stale; opening/selecting does not refill. Exact bridged-hole
conversion must preserve area and oriented boundary without new coordinates.
Legacy island-as-source migration requires unchanged source/provenance;
malformed or ambiguous data refuses without partial adoption. No broader import
feature expansion, native solver change or manufacturing qualification follows.

## Independent board graphics in native adoption (PM057)

The native KiCad board import explicitly captures supported non-copper Line,
three-point Arc, Rectangle, integer-radius Circle and linear Polygon records.
Stable source UUIDs qualify existing engine ImportMap allocations; missing or
ambiguous identities, unsupported styles/fields and inexact geometry refuse
before adoption. Structured dispositions distinguish supported graphic sources,
existing outline contributions, pending outline obligations and refusals.
Edge.Cuts line/arc contributions remain solely with the existing BoardOutline
owner, including historical identifier-less input; no second graphic is minted.

Required profile-4 adoption is a separate guarded forward-only transaction,
reported by transaction ID. The subsequent complete import records graphic/map
preimages and binds saved copper to the final combined source basis. Reimport
reuses identities; unchanged input is a no-op; source refresh cannot overwrite
native graphic edits. Missing external sources retain native graphics with
MissingInSource provenance. Actual reopened undo/redo must restore both board
and sidecar bytes. Source support is not exact manufacturing support: matching
CAM, rational-centre T04 and C04–C06 remain required without a tolerance waiver.
