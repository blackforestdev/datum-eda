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
│   ├── models/               # behavioural-model files (IBIS / SPICE / Touchstone / IBIS-AMI / thermal)
│   │   ├── ibis/
│   │   ├── spice/
│   │   ├── touchstone/
│   │   ├── ami/
│   │   └── thermal/
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

### 5.2.1 ZoneFill Generated Evidence

Native board copper-pour boundaries are authored as `board.zones`; generated
filled copper is not authored board truth. The current native generated-evidence
path for zone fills is:

```text
.datum/
└── zone_fills/
    └── <zone_uuid>.json
```

Each file is a `ZoneFill` record owned by the engine substrate with
`schema_version: 1`, `zone_id`, `state`, `source_zone_revision`,
`model_revision`, `islands`, and optional `provenance`. The filename UUID must
match `zone_id`; journaled `SetZoneFill` validates the same id before staging.
Resolver materialization classifies these shards as
`SourceShardKind::ZoneFill`, `SourceShardTaxon::ZoneFill`, and
`SourceShardAuthority::GeneratedEvidence`.

`state = "filled"` may carry closed, non-degenerate, non-self-intersecting
polygon islands. `state = "unfilled"` and `state = "unsupported"` carry no
islands. `state = "stale"` marks persisted evidence whose recorded
`model_revision` or `source_zone_revision` no longer matches the resolved
project. Missing persisted evidence for an authored zone resolves as an
`Unfilled` in-memory record; it is not promoted to disk until an explicit
producer writes generated evidence.

The only public producer in the current product surface is
`datum-eda check fill-zones` / MCP `datum.check.fill_zones`. It computes the
bounded native solver slice, commits `SetZoneFill` generated-evidence
operations with undo/redo inverses (`DeleteZoneFill` or restoration of the
previous payload), and never writes `.datum/zone_fills` directly from CLI code.
`datum.query.zone_fills` exposes resolver-derived state. Copper projection,
DRC, connectivity, and manufacturing export may consume only current
`Filled` islands; authored zone boundaries with missing, stale, unsupported, or
unfilled evidence emit no copper.

### 5.3 Cross-File References

- Cross-file references always use UUID identity.
- Human-readable names are attributes, not reference keys.
- A file may reference objects stored in other files only by UUID.
- A dangling cross-file reference is a validation error.
- The product-facing native-project validation surface for these errors is
  `project validate <dir>`.

---

## 6. Schemas

Every native file must contain `"schema_version": <int>` at the document root.
Schema-version compatibility is reported by `project validate <dir>`.

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
- the manifest carries no wall-clock `created`/`modified` timestamps:
  per the § 7 determinism invariant, identical authored data must
  serialize byte-identically on every save/platform/run, which any
  wall-clock field would violate
- `pools` are ordered by `priority`; lower numeric priority wins
- missing optional settings files are allowed if omitted from `settings`
- `project validate` checks declared pool directories when present. For
  `units`, `symbols`, `entities`, `parts`, `packages`, `padstacks`, and
  `pin_pad_maps` JSON shards it requires document-root `schema_version: 1`,
  filename/payload UUID parity, and deterministic dangling-reference findings
  for the first logical library graph: `Symbol.unit`, entity gate
  `unit`/`symbol`, package pad `padstack`, `Part.entity`, `Part.package`,
  legacy `Part.pad_map` pad/gate/pin links, `pin_pad_maps[*].part`, and
  `pin_pad_maps.mappings` pad keys plus `{gate, pin}` values against the
  referenced part. Runtime part-compatibility signatures
  and component-pad net remapping prefer `Part.default_pin_pad_map` resolved
  through `pool/pin_pad_maps` when valid, then fall back to legacy
  `Part.pad_map` for migration/import compatibility. This is current
  implementation evidence, not the full target authority model: `Footprint`
  geometry and first-class `PinPadMap` validation must move into the engine
  library graph.
- Journaled `CreatePoolLibraryObject` and `SetPoolLibraryObject` enforce the
  first write-time schema gate before staging authored pool shards:
  document-root `schema_version: 1`, UUID/path/kind parity, canonical pool
  struct deserialization for `units`, `symbols`, `entities`, `parts`,
  `packages`, `footprints`, `padstacks`, and the first `pin_pad_maps` envelope
  shape. `Footprint` is the canonical land-pattern payload; package land-pattern
  fields remain readable only for import and compatibility fallback.

### Library foundation target schema

The native library target is governed by
`docs/decisions/PRODUCT_MECHANICS_008_LIBRARY_COMPONENT_SYSTEM.md` and
`docs/contracts/LIBRARY_AUTHORING_TOOL_CONTRACT.md`. Current package-compatible
footprint handling is a migration bridge only. The canonical pool graph must
serialize these object boundaries:

- `Package`: component body/package-family data. Required fields:
  `schema_version`, `uuid`, `object_revision`, `display_name`, package family,
  mounting style, body dimensions/tolerances, body height, terminal definitions,
  provenance/review/lifecycle state, and optional package-level model refs. It
  must not be the authoritative owner of PCB land-pattern pads, courtyard,
  silkscreen, paste, or solder-mask openings.
  Current compatibility note: the engine can now deserialize body-only packages
  and explicit package terminals, while legacy `pads`/`courtyard`/`silkscreen`
  package fields remain accepted for import/materialization compatibility until
  those paths are migrated to `Footprint`.
- `Footprint`: PCB land pattern. Required fields: `schema_version`, `uuid`,
  `object_revision`, `package_ref`, `display_name`, origin, pad records
  referencing `padstack_ref`s and package terminals, courtyard/fab/silkscreen/
  assembly/mechanical primitives, process-aperture policy, standards basis,
  deviations, provenance/review/lifecycle state, and optional footprint-level
  model refs.
- `Padstack`: reusable pad/process primitive. Required fields:
  `schema_version`, `uuid`, `object_revision`, layer span, copper apertures,
  drill definition, plated/non-plated state, annular policy, thermal/anti-pad
  policy, mask policy, paste policy, unknown/import-preserved process states,
  provenance/review/lifecycle state, and standards/deviation metadata when
  derived from a known basis.
- `PinPadMap`: first-class logical-to-physical binding. It is gate-aware:
  canonical mapping rows are `pad -> {gate, pin}` rather than `pin -> pad`.
  Required fields:
  `schema_version`, `uuid`, `object_revision`, `entity_ref`, `symbol_ref` when
  symbol-specific mapping matters, `package_ref`, `footprint_ref`, mapping rows
  from physical pads to `{gate, pin}` logical bindings, optional electrical
  role and variant condition, provenance/review/lifecycle state, and conflict
  diagnostics for duplicate, open, shorted, stale, or missing mappings.
- `Part`: purchasable/selectable component. Required fields:
  `schema_version`, `uuid`, `object_revision`, `entity_ref`, default
  `package_ref`, default `footprint_ref`, default `pin_pad_map_ref`, MPN/value/
  manufacturer/lifecycle/parametric/orderable/supply-chain fields, optional
  model attachment refs, provenance/review state, and compatibility import data.
  Legacy `Part.pad_map` may seed a `PinPadMap` during migration but is not the
  target authority for schematic-to-PCB binding.
- `ModelAttachment`: governed attachment record or governed content object with
  `schema_version`, `uuid`, `object_revision`, target object ref, role, format,
  content hash, source/provenance, transform or simulation binding metadata,
  encrypted/opaque-content policy, review state, and lifecycle.

Validation tiers:
- Commit-time validation rejects malformed object identity, unsupported schema
  versions, stale object revisions, bad UUID/path/kind parity, and references
  that cannot be resolved in the current library graph for strict fields.
- Resolver diagnostics report shadowed objects, duplicate UUIDs across pool
  layers, missing optional references, imported unknown-basis process data,
  stale model blobs, and non-fatal compatibility migration issues.
- `project validate` runs the full dependency graph and standards/process checks
  without mutating the project.
- Typed unit-pin authoring mutates the `Unit.pins` map through the same
  `SetPoolLibraryObject` gate, rejecting duplicate pin IDs, blank names, and
  unsupported pin-direction enum values before the staged shard is promoted.
- Typed package authoring can now create body-only `Package` records without
  package land-pattern pads. Legacy `--pad` / `--padstack` compatibility input
  remains accepted for old scripts and mutates the legacy `Package.pads` map
  only when supplied, rejecting duplicate pad IDs, blank names, missing
  padstacks, and nonpositive layer ids before the staged shard is promoted.
  Runtime board pad regeneration prefers first-class `Footprint` pads and uses
  `Package.pads` only as fallback when no usable footprint is resolved.
- Legacy typed package-pad mutation is now a compatibility entry point into
  Footprint authority: `set-pool-package-pad` requires exactly one
  package-linked `Footprint` in the requested pool and writes
  `Footprint.pads`, leaving legacy `Package.pads` unchanged. New land-pattern
  pads should be authored directly with `set-pool-footprint-pad`.
- Typed first-class footprint authoring creates `Footprint` pool objects tied
  to existing `Package` records and mutates `Footprint.pads` through the same
  gate, rejecting missing packages, missing padstacks, blank pad names, and
  nonpositive layer ids before the staged shard is promoted.
- Typed first-class footprint-courtyard authoring mutates
  `Footprint.courtyard` through the same gate, accepting rectangular min/max
  nanometer bounds when they form nonzero width and height, or a closed polygon
  from `--vertices "x,y;x,y;..."` when at least three vertices parse cleanly,
  before the staged shard is promoted.
- Typed first-class footprint-silkscreen authoring mutates
  `Footprint.silkscreen` through the same gate, appending `Primitive::Line`,
  `Primitive::Rect`, `Primitive::Circle`, or `Primitive::Polygon` entries and
  rejecting zero-length lines, zero-area rectangles, non-positive circle radii,
  malformed or underspecified polygons/polylines, and non-positive stroke
  widths before the staged shard is promoted.
- Compatibility typed package-courtyard authoring is now a package-named
  Footprint authority shim: it requires exactly one package-linked `Footprint`
  in the requested pool and mutates `Footprint.courtyard`, leaving legacy
  `Package.courtyard` unchanged. Rectangular min/max nanometer bounds must form
  nonzero width and height; polygon vertices must parse cleanly and contain at
  least three points before the staged shard is promoted.
- Compatibility typed package-silkscreen authoring is now a package-named
  Footprint authority shim: it requires exactly one package-linked `Footprint`
  in the requested pool and mutates `Footprint.silkscreen`, leaving legacy
  `Package.silkscreen` unchanged.
- Compatibility typed package-silkscreen line authoring appends one
  `Primitive::Line` to `Footprint.silkscreen` and rejects zero-length endpoints
  or non-positive stroke widths before the staged shard is promoted.
- Compatibility typed package-silkscreen rectangle authoring appends one
  `Primitive::Rect` to `Footprint.silkscreen` and rejects zero-area bounds or
  non-positive stroke widths before the staged shard is promoted.
- Compatibility typed package-silkscreen circle authoring appends one
  `Primitive::Circle` to `Footprint.silkscreen` and rejects non-positive
  radius or stroke width before the staged shard is promoted.
- Compatibility typed package-silkscreen polygon/polyline authoring appends one
  `Primitive::Polygon` to `Footprint.silkscreen` from
  `--vertices "x,y;x,y;..."` plus `--closed true|false`, persisting vertices,
  closed state, and `width_nm`, and rejecting malformed vertices, nonpositive
  widths, too few vertices, or closed polygons with fewer than three vertices
  before the staged shard is promoted.
- Compatibility typed package-silkscreen arc authoring appends one
  `Primitive::Arc` to `Footprint.silkscreen`, persisting center, radius, start
  angle, end angle, and stroke width, and rejecting non-positive radius or
  stroke width before the staged shard is promoted.
- Compatibility typed package-silkscreen text authoring appends one
  `Primitive::Text` to `Footprint.silkscreen`, rejecting blank text, and
  persisting authored position and rotation before the staged shard is
  promoted.
- Typed package 3D model authoring mutates `Package.models_3d` through the
  same gate, appending one `ModelRef` from `--model-path` plus optional
  `--transform-json`, persisting both the relative path and parsed transform,
  and rejecting blank, absolute, or traversal model paths and malformed
  transform JSON before the staged shard is promoted.
- Typed symbol line authoring mutates `Symbol.drawings` through the same gate,
  appending one `Primitive::Line` and rejecting zero-length endpoints or
  non-positive stroke widths before the staged shard is promoted.
- Typed symbol rectangle authoring mutates `Symbol.drawings` through the same
  gate, appending one `Primitive::Rect` and rejecting zero-area bounds or
  non-positive stroke widths before the staged shard is promoted.
- Typed symbol circle authoring mutates `Symbol.drawings` through the same
  gate, appending one `Primitive::Circle` and rejecting non-positive radius or
  stroke width before the staged shard is promoted.
- Typed symbol polygon/polyline authoring mutates `Symbol.drawings` through the
  same gate, appending one `Primitive::Polygon` from
  `--vertices "x,y;x,y;..."` plus `--closed true|false`, persisting vertices,
  closed state, and `width_nm`, and rejecting malformed vertices, nonpositive
  widths, too few vertices, or closed polygons with fewer than three vertices
  before the staged shard is promoted.
- Typed symbol arc authoring mutates `Symbol.drawings` through the same gate,
  appending one `Primitive::Arc`, persisting center, radius, start angle, end
  angle, and stroke width, and rejecting non-positive radius or stroke width
  before the staged shard is promoted.
- Typed symbol text authoring mutates `Symbol.drawings` through the same gate,
  appending one `Primitive::Text`, rejecting blank text, and persisting authored
  position and rotation before the staged shard is promoted. This makes
  `Symbol.drawings` authoring explicitly support `Line`, `Rect`, `Circle`,
  `Text`, `Arc`, and `Polygon` primitives in the native library format.
- Typed symbol pin-anchor authoring mutates `Symbol.pin_anchors` through the
  same gate, setting one unit-pin UUID to a symbol-local nanometer position and
  rejecting missing units or pins before the staged shard is promoted. Placing a
  schematic symbol with `--lib-id` set to a pool symbol UUID materializes
  placed `SymbolPin` entries from those anchors while preserving arbitrary
  non-UUID `lib_id` strings as unresolved compatibility identifiers.
- Typed package 3D-model authoring appends typed `ModelRef` objects to
  `Package.models_3d` through the same gate, deriving or accepting
  `ModelFormat`, constructing typed `Transform3D` translation/rotation/scale
  values, rejecting blank/absolute/traversal paths, unsupported formats,
  malformed transform JSON, and nonpositive scale values before the staged
  shard is promoted.
- Typed package body-height authoring mutates or clears
  `Package.body_height_nm` and `Package.body_height_mounted_nm` through the
  same gate, rejecting empty no-op requests and nonpositive heights before the
  staged shard is promoted.
- Typed part metadata authoring mutates the existing `Part` shard through the
  same gate, preserving omitted fields while updating supplied `mpn`,
  `manufacturer`, `manufacturer_jep106`, `value`, `description`, `datasheet`,
  and `lifecycle` values, and rejecting empty no-op requests, unsupported
  lifecycle enum values, or out-of-range JEP106 manufacturer codes before the
  staged shard is promoted.
- Typed part parametric authoring mutates the `Part.parametric` map through the
  same gate in explicit `merge` or `replace` mode from repeatable `key=value`
  entries, rejecting malformed entries, blank keys, duplicate request keys, and
  unsupported modes before the staged shard is promoted.
- Typed part orderable-MPN authoring mutates the `Part.orderable_mpns` list
  through the same gate in explicit `merge` or `replace` mode from repeatable
  MPN values, rejecting blank values, duplicate request MPNs, and unsupported
  modes before the staged shard is promoted.
- Typed part packaging-option authoring mutates the `Part.packaging_options`
  list through the same gate in explicit `merge` or `replace` mode from
  repeatable JSON objects matching `PackagingKind` / `PackagingOption`,
  rejecting malformed JSON, schema-invalid options, duplicate request options,
  and unsupported modes before the staged shard is promoted.
- Typed part behavioural-model authoring mutates the
  `Part.behavioural_models` list through the same gate in explicit `merge` or
  `replace` mode from repeatable JSON objects matching `ModelAttachment`,
  rejecting malformed JSON, schema-invalid attachments, duplicate request
  attachments, invalid model-name/provenance strings, inconsistent encryption
  metadata, and unsupported modes before the staged shard is promoted.
- Typed part model attachment authoring copies a source behavioural-model file
  into `pool/models/<role>/<sha256>.<ext>`, derives deterministic model and
  attachment UUIDs from the hash/target part, and appends the generated
  `ModelAttachment` through typed `AttachPoolPartModel` substrate operations.
  Typed part model detach removes matching attachment edges by attachment UUID
  or model UUID through typed `DetachPoolPartModel` substrate operations. Undo
  restores the part attachment list exactly and intentionally retains the
  content-addressed blob; broader blob garbage-collection policy remains
  additive target work.
- Typed part thermal authoring mutates or clears the optional `Part.thermal`
  object through the same gate from supplied `ThermalSpec` fields, rejecting
  empty no-op requests, malformed/non-negative numeric values, blank
  references, and schema-invalid thermal objects before the staged shard is
  promoted.
- Typed part supply-chain authoring mutates or clears the derived
  `Part.supply_chain_offers` cache and `Part.last_supply_chain_check`
  timestamp through the same gate from repeatable `SupplyOffer` JSON objects
  and a checked-at timestamp string, rejecting empty no-op requests, malformed
  JSON, schema-invalid offers, blank distributor/link/currency values, zero
  quantity breaks, negative prices, and duplicate request offers before the
  staged shard is promoted.
- Typed part tag authoring mutates the `Part.tags` list through the same gate
  in explicit `merge` or `replace` mode from repeatable tag values, rejecting
  blank values, duplicate request tags, and unsupported modes before the staged
  shard is promoted.
- Typed first-class PinPadMap authoring creates and updates `pool/pin_pad_maps`
  records through the same gate using canonical `pad_uuid:gate_uuid:pin_uuid`
  mappings. The compatibility shorthand `pin_uuid:pad_uuid` is accepted only
  when the pin resolves to exactly one entity gate. Authoring rejects duplicate
  request pads, duplicate gate/pin pairs, missing entity gates or pins, missing
  first-class footprint pads when `footprint` is present, missing legacy package
  pads when no footprint is present, and footprint/package mismatches before the
  staged shard is promoted.
  Read-side validation also recognizes historical persisted pin-keyed JSON rows
  (`pin_uuid: pad_uuid` or `pin_uuid: { "pad": pad_uuid }`) under the same
  exact-one-gate constraint; pins shared by multiple entity gates are reported
  as ambiguous and are never silently resolved.
  Runtime part compatibility and board pad-net remapping consume these records
  first through `Part.default_pin_pad_map` when present and valid. Legacy-named
  part pad-map commands accept `pad:gate:pin` compatibility input, require a
  `Part.default_pin_pad_map`, and update the referenced first-class
  `pool/pin_pad_maps` record; they do not write `Part.pad_map`. Runtime uses
  `Part.pad_map` only as fallback when the first-class map is absent or
  unusable.

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
      { "id": 1, "name": "Top", "layer_type": "Copper", "thickness_nm": 35000, "copper_weight_oz": 1.0, "roughness_um": 0.4, "material_name": "RA Copper" },
      { "id": 2, "name": "Dielectric", "layer_type": "Dielectric", "thickness_nm": 1600000, "dielectric_constant": 4.2, "loss_tangent": 0.018, "material_name": "FR-4" },
      { "id": 3, "name": "Bottom", "layer_type": "Copper", "thickness_nm": 35000, "copper_weight_oz": 1.0, "roughness_um": 0.4, "material_name": "RA Copper" }
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
- board net entries may include optional `controlled_impedance` metadata with
  `target_ohms`, `tolerance_pct`, and `controlled_dielectric`; the dielectric
  reference is the current `StackupLayer.id`, and historical net entries
  without this object deserialize with no impedance target
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

### 6.10 `pool/models/` (Pool Model Files)

Purpose:
- persist behavioural-model files referenced from
  `Part.behavioural_models` (per `ENGINE_SPEC.md` § 1.1a `ModelAttachment`
  and § 1.2 `Part`)

Layout:
- one file per model, content-addressed by SHA-256 of the file bytes
- subdirectories partition by `ModelRole` (`ibis/`, `spice/`,
  `touchstone/`, `ami/`, `thermal/`, `verilog_a/`, `vhdl_ams/`)
- IBIS-AMI model bundles are stored as a directory
  (`ami/<sha256>/`) containing the `.ami` file plus per-platform
  binary cosimulation libraries; the directory's content sha256 is
  the bundle's identifier

File contents:
- raw vendor file bytes; Datum never rewrites or normalises model
  payloads
- encrypted vendor blocks pass through verbatim — Datum does not
  decrypt and does not re-encrypt

Current validation/query surface:
- `datum-eda project query <root> pool-models` enumerates regular
  `pool/models/<role>/<sha256>.<ext>` blobs, recomputes file SHA-256,
  reports whether the filename hash matches file bytes, derives the
  deterministic model UUID, and reports part attachment references discovered
  from `Part.behavioural_models`, including explicit `referenced` / `orphaned`
  lifecycle flags for retained blobs
- `datum-eda project validate <root>` reports missing provenance-backed model
  blobs, filename/hash mismatches, and `Part.behavioural_models` model UUIDs
  that no longer match the deterministic UUID derived from the referenced
  content hash
- `datum-eda project gc-pool-models <root>` is dry-run by default and removes
  orphaned regular model blob files only when `--apply` is supplied; hash
  mismatches, non-regular files, referenced blobs, and AMI bundle roles are
  preserved
- AMI bundle hashing, richer garbage-collection policy, and migration behavior
  remain additive target work (spec-ahead-of-code as of 2026-06-22: the
  `pool-models` / `validate` / `gc-pool-models` surfaces above ship, but the
  AMI-bundle and migration portions of this section are not implemented)

Rules:
- the model UUID is the deterministic UUID v5 over the SHA-256 hex
  string under a fixed namespace; same file at any pool path resolves
  to the same UUID
- the `models` and `part_model_attachments` index tables (per
  `docs/POOL_ARCHITECTURE.md` § 2) are derived from the file tree and
  are always rebuildable
- this directory is optional for projects that do not use behavioural
  attachments
- the schema is purely additive to the existing native format;
  existing projects without `pool/models/` continue to deserialize

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

> **Code status (2026-06-22):** the unknown-future-version load gate is
> shipped at the shard layer — shard/pool readers reject any
> `schema_version` other than the supported `1` with a clear
> "unsupported … schema_version …" error (`substrate/source_shard.rs`,
> `substrate/pool_journal_ops.rs`). `ProjectResolver` already detects
> these via `is_unsupported_schema_version_error` and propagates them as a
> load failure rather than swallowing them as a missing-shard diagnostic
> (`substrate/project_resolver.rs`), and `datum-eda project validate <root>`
> exercises this path. The remaining items below are **spec-ahead-of-code**:
> only schema version `1` exists, so no migration functions, no
> version-to-version transitions, and no `material_db` (the §8 example
> below) are implemented yet.

Migration implementation requirements (target — no migration machinery
ships yet; only schema version `1` exists):
- one explicit migration function per version transition
- golden tests for each supported historical version

Stackup material fields are optional per layer in schema version 1. Historical
board files that only contain `id`, `name`, `layer_type`, and `thickness_nm`
deserialize with unset material metadata; no migration is required for this
additive shape.

Example (illustrative target shape; `material_db` and the 1→2 transition
are not implemented — only schema version `1` exists in code today):

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
