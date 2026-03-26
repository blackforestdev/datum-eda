# Native Format Specification

## 1. Purpose

Defines the on-disk representation for native projects introduced in `M4`.

This specification owns:
- native project directory structure
- file boundaries
- per-file schema contracts
- persistence of authored versus derived data
- schema versioning
- migration rules
- imported-project persistence rules after conversion to native format

This specification does not own:
- canonical in-memory types
- engine API semantics
- import fidelity rules
- MCP/CLI command surfaces

Those remain in:
- `specs/ENGINE_SPEC.md`
- `specs/IMPORT_SPEC.md`
- `specs/MCP_API_SPEC.md`
- `specs/PROGRAM_SPEC.md`

The deterministic serialization contract remains controlled by
`docs/CANONICAL_IR.md`.

---

## 2. Scope

This specification defines:
- `project.json`
- native schematic persistence
- native board persistence
- rules persistence
- checking settings persistence
- output/export settings persistence
- pool references and project-local pool behavior
- import-sidecar persistence rules for converted projects

Deferred until needed:
- collaboration/locking semantics beyond single-writer assumptions
- variant editing semantics beyond data-model persistence
- advanced migration tooling UX

---

## 3. Design Principles

- The native format is a direct serialization of the canonical IR.
- Authored data is persisted; derived data is recomputed unless explicitly
  marked as cache.
- File layout should favor diffability, mergeability, and partial loading.
- Relative paths are preferred inside the project directory.
- The same authored data must produce byte-identical file content on every run.

---

## 4. Project Layout

A native project is a directory containing:

```text
myproject/
├── project.json
├── schematic/
│   ├── schematic.json
│   ├── sheets/
│   │   ├── <uuid>.json
│   │   └── ...
│   └── definitions/
│       ├── <uuid>.json
│       └── ...
├── board/
│   └── board.json
├── pool/                     # optional project-local pool
│   ├── units/
│   ├── entities/
│   ├── symbols/
│   ├── packages/
│   ├── padstacks/
│   ├── parts/
│   └── pool.sqlite
├── rules/
│   └── rules.json
├── settings/
│   ├── checking.json
│   └── output.json
└── .ids/
    └── <original_filename>.ids.json
```

Required:
- `project.json`
- `schematic/schematic.json`
- `board/board.json`
- `rules/rules.json`

Optional:
- `pool/`
- `settings/checking.json`
- `settings/output.json`
- `.ids/`

Path rules:
- paths inside project metadata must be relative to the project root
- absolute paths are allowed only for external pool references in `project.json`
- UUID-keyed filenames use lowercase hyphenated UUID strings

---

## 5. File Ownership

### 5.1 Authored Data

Persisted authored data includes:
- schematic root metadata
- sheet definitions
- sheet instances
- sheets and their authored contents
- board authored objects
- rules
- waivers
- variant selections
- project metadata
- pool references

### 5.2 Derived Data

Derived data is not canonical and must not be treated as authored truth.

Not persisted as canonical authored state:
- board connectivity graph
- schematic connectivity graph
- airwires
- DRC findings
- ERC findings
- copper fill geometry
- routability caches

Optional future caches may be persisted, but they must be marked as derived and
safe to discard.

### 5.3 Cross-File References

- Cross-file references always use UUID identity.
- Human-readable names are attributes, not reference keys.
- A file may reference objects stored in other files only by UUID.
- A dangling cross-file reference is a validation error.

---

## 6. Schemas

Every native file must contain `"schema_version": <int>` at the document root.

### 6.1 `project.json`

Purpose:
- project manifest
- top-level metadata
- references to schematic, board, rules, and settings
- pool search path ordering

Schema:

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

Rules:
- `uuid` identifies the project, not the schematic or board
- `pools` are ordered by `priority`; lower numeric priority wins
- missing optional settings files are allowed if omitted from `settings`

### 6.2 `schematic/schematic.json`

Purpose:
- schematic root metadata
- file references to sheets and definitions
- sheet-instance list
- variant selections
- checking waivers

Schema:

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

Rules:
- `sheets` maps sheet UUID to relative file path
- `definitions` maps sheet-definition UUID to relative file path
- `instances` is authored hierarchy, not derived data
- `variants` persists only the data-model surface defined in `ENGINE_SPEC.md`
- `waivers` use the canonical waiver model from
  `specs/CHECKING_ARCHITECTURE_SPEC.md`

### 6.3 `schematic/sheets/<uuid>.json`

Purpose:
- one authored sheet and all authored objects directly placed on that sheet

Schema example:

```json
{
  "schema_version": 1,
  "uuid": "sheet-uuid",
  "name": "Power Supply",
  "frame": null,
  "symbols": {},
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

Rules:
- all sheet-local authored objects persist here
- object maps are keyed by object UUID
- no derived connectivity data is stored in the sheet file

### 6.4 `schematic/definitions/<uuid>.json`

Purpose:
- persisted `SheetDefinition` object

Minimum schema:

```json
{
  "schema_version": 1,
  "uuid": "definition-uuid",
  "root_sheet": "sheet-uuid",
  "name": "Power Subsystem"
}
```

### 6.5 `board/board.json`

Purpose:
- complete authored board state

The board persists as one file in schema version 1.

Schema example:

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

Rules:
- board authored state is monolithic in version 1
- board connectivity, fills, and DRC findings are not canonical board content
- a future split requires an explicit schema version bump

### 6.6 `rules/rules.json`

Purpose:
- persisted rule set for board and checking consumers

Schema:

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

Rules:
- rule encoding must align with `ENGINE_SPEC.md`
- unsupported-but-parseable scope nodes may be persisted if they are valid IR

### 6.7 `settings/checking.json`

Purpose:
- checking severity overrides
- future checking configuration

Minimum owned content:
- ERC severity overrides
- DRC severity overrides if needed later

The exact shape may expand, but it must not duplicate authored waivers.

### 6.8 `settings/output.json`

Purpose:
- output/export presets for manufacturing and reporting

This file is optional in version 1.

### 6.9 `.ids/<filename>.ids.json`

Purpose:
- persist imported-object identity bridges for converted/import-derived projects

Rules:
- format must align with `IMPORT_SPEC.md`
- this directory is optional for purely native projects
- if present after conversion, it preserves source-origin identity metadata

---

## 7. Serialization Contract

All native files follow the deterministic serialization contract from
`docs/CANONICAL_IR.md`.

Required:
- map keys sorted alphabetically
- UUIDs in lowercase hyphenated form
- coordinates in integer nanometers
- angles in integer tenths of degree
- arrays ordered semantically or by UUID where unordered
- UTF-8 encoding, no BOM
- no random or non-deterministic object content

Guarantee:
- same authored data produces byte-identical file content on every save,
  every platform, every run

---

## 8. Versioning and Migration

Every native file includes `"schema_version": N`.

Rules:
- the engine may read any supported version `>= 1`
- the engine always writes the current version
- migrations are explicit and version-to-version
- migration is forward-only
- an unknown future version is a load error with a clear message

Migration implementation requirements:
- one explicit migration function per version transition
- golden tests for each supported historical version

Example:

```text
Version 1: stackup.layers only
Version 2: stackup.layers + material_db

Migration 1→2: add material_db = null
```

---

## 9. Imported Project Persistence

### 9.1 Before `M4`

Imported projects persist through:
1. source files (`.kicad_pcb`, `.kicad_sch`, `.brd`, `.sch`)
2. `.ids.json` sidecar identity files
3. in-memory canonical IR loaded from source on open

### 9.2 After `M4`

Imported projects may optionally be converted to native format.

Conversion rules:
- conversion creates a native project directory
- original source files are not modified or deleted
- source-origin identity metadata is preserved via imported identity state
- native persistence after conversion becomes authoritative for subsequent
  native editing

Conversion itself is not the same as source-format write-back.

---

## 10. Concurrency Model

Version 1 assumes single-writer access to a project directory.

Version 1 does not require:
- file locking
- collaborative live editing
- concurrent-write detection beyond normal filesystem behavior

Recommended workflow for multiple users:
- use version control branching
- merge at the file level

Atomic save expectations:
- a save should not leave partially written JSON files on normal success
- crash safety may use temp-file plus rename semantics

---

## 11. Acceptance Criteria

This specification is satisfied for `M4` when:
- every required file in the native project tree has a defined schema
- ownership of authored versus derived data is explicit
- save → load → save is byte-stable for unchanged native projects
- schema versioning is implemented and tested
- imported-project conversion preserves identity metadata correctly
- `specs/PROGRAM_SPEC.md` `M4` exit criteria reference this spec as the native
  format contract
