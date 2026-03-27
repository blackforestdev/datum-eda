# Native File Format — Design Rationale

> **Status**: Non-normative design rationale.
> The controlling native format specification is `specs/NATIVE_FORMAT_SPEC.md`.
> The controlling serialization contract is `docs/CANONICAL_IR.md` §5.
> This document provides rationale, tradeoffs, and explanatory examples.
> If any contract here conflicts with `specs/NATIVE_FORMAT_SPEC.md`, the spec
> file wins.

## Purpose
Proposes the on-disk representation for projects created natively by this
tool. The native format is introduced in M4. Before M4, designs exist only
as imported formats (KiCad/Eagle files) with the canonical IR held in memory.

Current live slice:
- `eda project new <dir> [--name <project-name>]` creates the initial native
  scaffold for `project.json`, `schematic/schematic.json`, `board/board.json`,
  and `rules/rules.json`.
- `eda project inspect <dir>` loads that scaffold, validates the resolved file
  layout, and reports current schema/UUID/path/count summary without opening a
  mutable editing session.
- `eda project query <dir> summary` and
  `eda project query <dir> design-rules` provide the first native read surface
  directly from the on-disk scaffold.
- `eda project place-symbol <dir> --sheet <uuid> --reference <text> --value <text>
  [--lib-id <text>] --x-nm <i64> --y-nm <i64> [--rotation-deg <i32>] [--mirrored]`,
  `eda project move-symbol <dir> --symbol <uuid> --x-nm <i64> --y-nm <i64>`, and
  `eda project rotate-symbol <dir> --symbol <uuid> --rotation-deg <i32>`,
  `eda project mirror-symbol <dir> --symbol <uuid>`, and
  `eda project delete-symbol <dir> --symbol <uuid>`, `eda project set-symbol-reference <dir> --symbol <uuid> --reference <text>`, and
  `eda project set-symbol-value <dir> --symbol <uuid> --value <text>`,
  `eda project set-symbol-unit <dir> --symbol <uuid> --unit <text>`,
  `eda project clear-symbol-unit <dir> --symbol <uuid>`,
  `eda project set-symbol-gate <dir> --symbol <uuid> --gate <uuid>`, and
  `eda project clear-symbol-gate <dir> --symbol <uuid>`,
  `eda project set-symbol-display-mode <dir> --symbol <uuid> --mode <...>`,
  `eda project set-symbol-hidden-power-behavior <dir> --symbol <uuid> --behavior <...>`,
  `eda project set-pin-override <dir> --symbol <uuid> --pin <uuid> --visible <true|false>
  [--x-nm <i64> --y-nm <i64>]`, and
  `eda project clear-pin-override <dir> --symbol <uuid> --pin <uuid>` add the first native
  authored schematic symbol placement/transform/delete/semantic-selection path, and
  `eda project add-symbol-field <dir> --symbol <uuid> --key <text> --value <text>
  [--hidden] [--x-nm <i64> --y-nm <i64>]`,
  `eda project edit-symbol-field <dir> --field <uuid> ...`, and
  `eda project delete-symbol-field <dir> --field <uuid>` extend that path to
  native symbol field authoring, while `eda project query <dir> symbols`,
  `eda project query <dir> symbol-fields --symbol <uuid>`, and
  `eda project query <dir> symbol-semantics --symbol <uuid>`, and
  `eda project query <dir> symbol-pins --symbol <uuid>` read back the
  persisted symbol slice, including any stored per-pin override state.
- `eda project place-text <dir> --sheet <uuid> --text <text> --x-nm <i64>
  --y-nm <i64> [--rotation-deg <i32>]`, `eda project edit-text <dir> --text <uuid> ...`,
  and `eda project delete-text <dir> --text <uuid>` add the first native
  non-electrical schematic text object family, while
  `eda project query <dir> texts` reads back the persisted text slice.
- `eda project place-drawing-line <dir> --sheet <uuid> --from-x-nm <i64>
  --from-y-nm <i64> --to-x-nm <i64> --to-y-nm <i64>`,
  `eda project place-drawing-rect <dir> --sheet <uuid> ...`,
  `eda project place-drawing-circle <dir> --sheet <uuid> ...`,
  `eda project place-drawing-arc <dir> --sheet <uuid> ...`,
  `eda project edit-drawing-line <dir> --drawing <uuid> ...`,
  `eda project edit-drawing-rect <dir> --drawing <uuid> ...`,
  `eda project edit-drawing-circle <dir> --drawing <uuid> ...`,
  `eda project edit-drawing-arc <dir> --drawing <uuid> ...`, and
  `eda project delete-drawing <dir> --drawing <uuid>` add the first native
  schematic drawing primitive family, while `eda project query <dir> drawings`
  reads back the persisted drawing slice with kind-tagged objects.
- `eda project place-label <dir> --sheet <uuid> --name <text> --x-nm <i64>
  --y-nm <i64>` is the first native authored schematic mutation and writes
  directly to a referenced sheet file.
- `eda project rename-label <dir> --label <uuid> --name <text>` and
  `eda project delete-label <dir> --label <uuid>` complete the first native
  schematic object family on the same deterministic sheet-file mutation path.
- `eda project draw-wire <dir> --sheet <uuid> --from-x-nm <i64> --from-y-nm <i64>
  --to-x-nm <i64> --to-y-nm <i64>` and `eda project delete-wire <dir> --wire <uuid>`
  add the first connectivity-bearing native schematic geometry mutation path.
- `eda project place-junction <dir> --sheet <uuid> --x-nm <i64> --y-nm <i64>`
  and `eda project delete-junction <dir> --junction <uuid>` extend that path
  to authored net-topology join points.
- `eda project place-port <dir> --sheet <uuid> --name <text> --direction <...>
  --x-nm <i64> --y-nm <i64>`, `eda project edit-port <dir> --port <uuid> ...`,
  and `eda project delete-port <dir> --port <uuid>` add the first cross-sheet
  connectivity-interface object family on the same deterministic mutation path.
- `eda project create-bus <dir> --sheet <uuid> --name <text> --member <text>...`,
  `eda project edit-bus-members <dir> --bus <uuid> --member <text>...`,
  `eda project place-bus-entry <dir> --sheet <uuid> --bus <uuid> [--wire <uuid>]`,
  and `eda project delete-bus-entry <dir> --bus-entry <uuid>` extend that path
  to structured multi-net connectivity objects.
- `eda project place-noconnect <dir> --sheet <uuid> --symbol <uuid> --pin <uuid>
  --x-nm <i64> --y-nm <i64>` and `eda project delete-noconnect <dir> --noconnect <uuid>`
  add the no-connect marker object family on the same deterministic sheet-file path.

## Design Principle
The native format is a direct serialization of the canonical IR. No
translation layer, no lossy conversion. What is in memory is what is on
disk, serialized as deterministic JSON.

---

## 1. Project Structure

A native project is a directory containing:

```
myproject/
├── project.json              # Project manifest
├── schematic/
│   ├── schematic.json        # Schematic metadata + variants + waivers
│   ├── sheets/
│   │   ├── <uuid>.json       # One file per sheet
│   │   └── ...
│   └── definitions/
│       ├── <uuid>.json       # One file per SheetDefinition
│       └── ...
├── board/
│   └── board.json            # Complete board state
├── pool/                     # Project-local pool (optional)
│   ├── units/
│   ├── entities/
│   ├── symbols/
│   ├── packages/
│   ├── padstacks/
│   ├── parts/
│   └── pool.sqlite           # Pool index (rebuildable)
├── rules/
│   └── rules.json            # Design rules (shared by ERC + DRC)
├── settings/
│   ├── checking.json         # ERC/DRC severity overrides, waiver config
│   └── output.json           # Export presets (M4+: Gerber, BOM, etc.)
└── .ids/                     # Import sidecar data (if project was imported)
    └── <original_filename>.ids.json
```

### Why multiple files instead of one

- **Diffability**: `git diff` shows which sheets changed, not a monolithic blob
- **Merge-friendliness**: two people editing different sheets don't conflict
- **Incremental save**: only write files for modified objects
- **AI-friendliness**: read one sheet without loading the entire project
- **Performance**: large boards serialize/deserialize faster in isolation

---

## 2. File Schemas

### 2.1 project.json

```json
{
  "schema_version": 1,
  "uuid": "project-uuid",
  "name": "My Project",
  "created": "2026-03-24T12:00:00Z",
  "modified": "2026-03-24T14:30:00Z",
  "pools": [
    { "path": "pool", "priority": 1 },
    { "path": "/shared/team-pool", "priority": 2 }
  ],
  "schematic": "schematic/schematic.json",
  "board": "board/board.json",
  "rules": "rules/rules.json",
  "settings": {
    "checking": "settings/checking.json",
    "output": "settings/output.json"
  }
}
```

### 2.2 schematic.json

```json
{
  "schema_version": 1,
  "uuid": "schematic-uuid",
  "sheets": {
    "sheet-uuid-1": "sheets/sheet-uuid-1.json",
    "sheet-uuid-2": "sheets/sheet-uuid-2.json"
  },
  "definitions": {
    "def-uuid-1": "definitions/def-uuid-1.json"
  },
  "instances": [
    {
      "uuid": "instance-uuid",
      "definition": "def-uuid-1",
      "parent_sheet": null,
      "position": { "x": 0, "y": 0 },
      "name": "Main Sheet"
    }
  ],
  "variants": {
    "variant-uuid": {
      "name": "Standard",
      "fitted_components": {}
    }
  },
  "waivers": []
}
```

### 2.3 Sheet file (sheets/<uuid>.json)

```json
{
  "schema_version": 1,
  "uuid": "sheet-uuid",
  "name": "Power Supply",
  "frame": null,
  "symbols": {
    "sym-uuid": {
      "uuid": "sym-uuid",
      "part": "part-uuid",
      "entity": "entity-uuid",
      "gate": "gate-uuid",
      "reference": "U1",
      "value": "LM3671",
      "fields": [],
      "position": { "x": 25400000, "y": 19050000 },
      "rotation": 0,
      "mirrored": false,
      "unit_selection": null,
      "display_mode": "LibraryDefault",
      "pin_overrides": [],
      "hidden_power_behavior": "SourceDefinedImplicit"
    }
  },
  "wires": {},
  "junctions": {},
  "labels": {},
  "buses": {},
  "bus_entries": {},
  "ports": {},
  "noconnects": {},
  "texts": {},
  "drawings": {}
}
```

### 2.4 board.json

The board is a single file because board objects are heavily
cross-referenced (tracks reference nets, nets reference components,
zones reference nets and interact geometrically). Splitting the board
across files would create complex cross-file references.

```json
{
  "schema_version": 1,
  "uuid": "board-uuid",
  "name": "My Board",
  "stackup": {
    "layers": [
      { "id": 1, "name": "Top", "type": "Copper", "thickness_nm": 35000 },
      { "id": 2, "name": "Dielectric", "type": "Dielectric", "thickness_nm": 1600000 },
      { "id": 3, "name": "Bottom", "type": "Copper", "thickness_nm": 35000 }
    ]
  },
  "outline": {
    "vertices": [
      { "x": 0, "y": 0 },
      { "x": 53800000, "y": 0 },
      { "x": 53800000, "y": 37500000 },
      { "x": 0, "y": 37500000 }
    ],
    "closed": true
  },
  "packages": {},
  "tracks": {},
  "vias": {},
  "zones": {},
  "nets": {},
  "net_classes": {},
  "keepouts": [],
  "dimensions": [],
  "texts": []
}
```

All coordinates are `i64` nanometers. All angles are `i32` tenths of degree.
All UUIDs are lowercase hyphenated strings.

### 2.5 rules.json

```json
{
  "schema_version": 1,
  "rules": [
    {
      "uuid": "rule-uuid",
      "name": "Default Clearance",
      "scope": "All",
      "priority": 1,
      "enabled": true,
      "rule_type": "ClearanceCopper",
      "parameters": { "Clearance": { "min": 100000 } }
    }
  ]
}
```

---

## 3. Serialization Contract

All native format files follow the deterministic serialization contract
from CANONICAL_IR.md §5:

- Map keys sorted alphabetically
- UUIDs lowercase hyphenated
- Coordinates as integer nanometers (never floating point)
- Arrays ordered semantically (vertices in polygon order, layers in
  stackup order) or by UUID (for unordered collections)
- Schema version at document root
- UTF-8 encoding, no BOM
- No timestamp or random data in serialized design objects

**Guarantee**: Same authored data → byte-identical file content on every
save, every platform, every run. Verified by golden tests.

---

## 4. Schema Versioning

Every file includes `"schema_version": N`. Version rules:

- The engine can **read** any version ≥ 1
- The engine always **writes** the current version
- Migration logic is explicit per-version (not implicit)
- Migration is forward-only: version 1 → 2, never 2 → 1
- If a file has an unknown version (higher than the engine knows),
  the engine returns an error with a clear message

### Migration example
```
Version 1: { "stackup": { "layers": [...] } }
Version 2: { "stackup": { "layers": [...], "material_db": "iec-61249" } }

Migration 1→2: add "material_db": null to stackup
```

Migrations are collected in a module, one function per version transition.
They are tested against saved golden files of each version.

---

## 5. Imported Project Persistence

Before M4 (native format), imported projects persist through:

1. **Source files**: original .kicad_pcb / .kicad_sch / .brd / .sch
2. **Sidecar .ids.json**: UUID mapping for stable identity across sessions
3. **In-memory canonical IR**: loaded from source files on each open

After M4, imported projects can optionally be converted to native format:
- `tool convert <kicad_project> --to native`
- This creates a native project directory from the imported data
- The original source files are NOT modified or deleted
- Conversion is one-way (native → KiCad export is a separate operation)

---

## 6. File Locking and Concurrency

The engine assumes single-writer access to a project directory. It does
not implement file locking or concurrent-write detection in v1.

For multi-user scenarios:
- Use git branching (each user works on a branch)
- Merge at the git level (per-file structure makes this practical)
- Conflict resolution: git merge conflicts on JSON files are
  human-resolvable (one object per file, sorted keys)

Future: file-level locking or operational transform for real-time
collaboration (M8+ if ever).

---

## Milestone Position
- M0-M3: No native format. Designs exist as imported source files +
  sidecar .ids.json + in-memory canonical IR
- M4: Native format introduced. Schema version 1.
- M4: Convert-from-import tool
- M4+: Schema migrations as format evolves
