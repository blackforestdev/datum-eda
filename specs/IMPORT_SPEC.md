# Import Specification

## 1. Deterministic UUID v5 Algorithm

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
Same file, same import code → identical UUIDs. This is tested by
importing the same file twice and asserting byte-identical output.

---

## 2. Sidecar .ids.json

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
| Spice models | Deferred | |

---

## 5. Round-Trip Guarantees

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
