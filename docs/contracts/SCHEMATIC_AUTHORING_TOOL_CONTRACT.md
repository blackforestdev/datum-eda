# Schematic Authoring Tool Contract

Status: draft implementation-spec 2026-06-19; derived from ratified
PRODUCT_MECHANICS 000-012

## Driven by

- `docs/audits/scope-integration/DATUM_PRODUCT_MECHANICS_IMPLEMENTATION_READINESS_AUDIT.md`
  (ratified vocabulary and cross-doc invariants)
- `docs/decisions/PRODUCT_MECHANICS_001_CANONICAL_EDIT_MODEL.md`
  (typed Operation/OperationBatch, single `commit()`, journal)
- `docs/decisions/PRODUCT_MECHANICS_003_SCHEMATIC_PCB_AUTHORITY.md`
  (`ComponentInstance` join, relationship split, proposal-first ECO)
- `docs/decisions/PRODUCT_MECHANICS_004_AI_TOOLING_CONTRACT.md`
  (shared CLI/MCP/query/check/proposal/commit contract classes)
- `docs/decisions/PRODUCT_MECHANICS_009_RULES_CONSTRAINTS_CHECKS.md`
  (`CheckRun`/`CheckFinding`, waivers, proposal-first repair)
- Shared surface: `docs/contracts/AI_CLI_MCP_TOOL_SURFACE.md` (the seven
  shared operations — referenced, never restated here)

## Purpose & Scope

This contract specifies the minimal load-bearing tool set for authoring a
Datum schematic: placing and editing symbols, drawing connectivity, naming
nets, and reading electrical checks. It defines, per tool, the eight
implementation questions (manual UI action, Operation emitted, CLI command,
MCP/AI tool, AI query/context, validating checks, proof slice, and what is
explicitly not yet supported).

Scope is the per-sheet authoring surface over the single resolved
`DesignModel` at a known `model_revision`. It is deliberately NOT a
catalog of every native schematic verb that exists today. The current
engine exposes roughly 52 native-schematic CLI functions across three
mutation files; this contract collapses them to **ten** typed operations
by making kind, target, and edit-vs-create into parameters of existing
operations rather than separate tools.

Broader hierarchy authoring (sheet instances, sheet pins and cross-sheet net
rewiring), variant overlays and library object authoring (see `008` contract)
remain outside this ten-tool slice. Native flat schematic document creation
already has a journaled engine/CLI operation; its ordinary GUI binding is a
separately bounded implementation increment.

### Current-engine grounding (verified)

Native schematic authoring has partially migrated onto the engine
OperationBatch/journal/ProjectResolver path. The old private JSON writer
inventory is retained below as historical scope evidence, but those families
are no longer allowed to be the active authority for migrated surfaces:

- `crates/cli/src/command_project_schematic_symbol_mutations.rs` (25 fns:
  place/move/rotate/mirror/delete symbol; set_/clear_ reference, value,
  lib_id, entity, part, unit, gate; set display_mode, hidden_power, pin
  override + clear; add/edit/delete field)
- `crates/cli/src/command_project_schematic_connectivity_mutations.rs`
  (16 fns: place/rename/delete label; draw/delete wire; place/delete
  junction; place/edit/delete port; create_bus/edit_bus_members;
  place/delete bus_entry; place/delete noconnect)
- `crates/cli/src/command_project_schematic_text_drawing_mutations.rs`
  (12 fns: text + drawing primitive families)

Current evidence: schematic symbol, connectivity, text/drawing, sheet
definition, sheet-instance, and hierarchy binding mutations route through
journaled native-project operations and resolver replay; canonical MCP
`datum.schematic.*` aliases bridge to the matching CLI commands. The
remaining product gap is not "no schematic substrate"; it is completion and
normalization of the contract shape: heterogeneous delete/set-field collapse,
binding evidence breadth, revision-keyed ERC/binding semantics, and
proposal boundaries for higher-level cross-domain edits. The first
contract-normalized schematic operation is now engine-owned:
`Operation::PlaceSchematicMarker { marker_kind:
Junction|NoConnect, ... }` writes the same sheet maps as the legacy
junction/no-connect operations, replays from the journal, and generates
undo inverses; the compatibility CLI verbs `place-junction` and
`place-noconnect` now author that normalized operation. Pool-backed
`place-symbol` now mints an authored symbol-first `ComponentInstance` when the
pool symbol resolves to a unique entity/gate/compatible part. It reports
explicit status for compatibility `lib_id`s, unresolved/ambiguous entity gates,
bound-without-part, ambiguous compatible parts, incompatible parts, and
bound-with-part; board-package binding remains partial.

The bounded standalone Text slice (`dat-schematic-text-krdm`) uses the existing
Place Text action, shared content/rotation form, selected Inspector edit and
shared Move/Delete paths. Notes have independent absolute anchors, preserve
literal single-line content and raw source extensions, and use canonical
journaled operations with cancellation, Undo/Redo and reopening. Ordinary note
text defaults to 1.27 mm, classified as a routine visual choice under PM059 in
that task; this does not change the Rendering Book's title/fabrication-note
ladder or add a parent tether. New rotations are cardinal; translation preserves
an unchanged imported angle. Multiline editing, typography controls, repeated
source edits and broader hierarchy authoring remain outside this slice.

Confirmed IR facts the matrix relies on:

- `LabelKind` (`Local|Global|Hierarchical|Power`) is already a field on
  `NetLabel` (`crates/engine/src/schematic/mod.rs:145`). Label kind is a
  parameter, not a reason for separate tools.
- `BusEntry` carries `bus: Uuid` + optional `wire: Uuid` + position
  (`schematic/mod.rs:168`). It is bus-bound, not a free point marker.
- `Schematic.sheet_instances: HashMap<Uuid, SheetInstance>` exists in the
  IR. Current native builders in `api/native_write/schematic_sheets.rs`
  author definitions, instances and bindings through canonical journaled
  operations. Complete ordinary GUI hierarchy authoring remains pending.
- ERC runs over an in-memory `Schematic`
  (`crates/engine/src/erc/mod.rs:50`,
  `run_prechecks_with_config_and_waivers`): 7 rules + a
  hierarchical-connectivity-mismatch check
  (`erc/mod.rs:291`), with severity overrides and waivers. Native check
  results have revision-qualified `CheckRun` records and stable finding
  fingerprints; the in-memory checker is not the persistence boundary.
- `ProjectResolver` and resolver-materialized schematic read/write paths exist
  for the current native-project slices; `PlacedSymbol` still carries a
  reference string + optional part/entity/gate UUIDs rather than a guaranteed
  minted `ComponentInstance` join.
- MCP exposes canonical schematic write aliases for the current journaled
  slices. These are still mostly compatibility-shaped verbs rather than the
  normalized ten-operation contract below.

## Reference-Tool Survey

**Altium Designer (primary reference).** Symbols place/move/rotate from
the Libraries panel; `Place > Wire | Bus | Net Label | Port | Sheet
Symbol | Power Port`; junctions auto-created at T-intersections;
`Tools > Annotate`; ONE SCH List/Inspector property surface drives batch
field edits; `Tools > Electrical Rules Check` with an ERC config table;
ECO is the gate for any schematic→PCB change. **Load-bearing:** one
selection+property model, ECO as the cross-domain gate, a single ERC
config table. **Ceremony Datum omits:** separate place-wire vs place-line
modes, Smart-Paste variants, per-property menu items.

**KiCad 8 (eeschema).** `Place Symbol | Power | Wire | Bus | Label |
Global Label | Hierarchical Label | Sheet | No-connect | Junction`;
junctions mostly implicit; a Symbol Fields Table for bulk field edit;
hierarchical sheets with sheet pins. **Load-bearing:** implicit junction,
global/hierarchical label net semantics, sheet-pin ↔ hierarchical-label
matching. **Ceremony:** three separate label tools instead of one
label-kind parameter; separate Junction and No-Connect placement tools
that are both zero-dimension single-point markers.

**OrCAD/Cadence Capture.** `Place Part | Wire | Bus | Net Alias | Power |
Ground | Hierarchical Port | Off-Page Connector`; netlist handoff to PCB
Editor; DRC. **Load-bearing:** off-page connector as a cross-sheet net
join, occurrence-vs-instance hierarchy model. **Ceremony:** heavy modal
property editor, session-log checks.

**Horizon EDA.** Per-block schematic; net lines + net labels + bus
rippers + power symbols; implicit junctions; ERC inside the rules engine;
every edit is a tool producing a document delta. This is the lean model
closest to Datum's operation-first intent. **Load-bearing:**
block/instance hierarchy, net-tie, pool-backed symbols.

**Lean rationale.** Across all four tools the load-bearing primitives are:
one selection+property surface (not per-field menu items), kind as a
parameter on label/marker placement (not a tool per kind), and ECO as the
gate for changes that reach the board. Datum already has `LabelKind` in
the IR, so the per-kind tool fan-out is pure ceremony. The single
principle this contract applies consistently is **parametric kind +
optional-id edit-vs-create + heterogeneous id lists** — the same principle
the reference tools converge on through their Inspector/Properties and
implicit-junction behavior.

## Tool Inventory

Each tool answers the eight questions as labeled fields. Common reads,
session/context bootstrap, the proposal lifecycle, the single `commit()`,
and global undo/redo are defined ONCE in the shared surface (see
**Shared Surface** below) and are referenced by name here, never
restated.

### 1. place-symbol

- **(1) Manual UI:** Pick a symbol from the library panel, click to place
  on the active sheet, drag to set position, type reference/value.
- **(2) Operation:** `PlaceSymbol{sheet_id, library_ref, position,
  rotation, mirror, component_instance_id}` → single-Operation
  `OperationBatch` through `commit()`. Current CLI uses the journaled
  schematic operation path and, for pool-symbol UUID `lib_id`s that resolve to
  a unique entity/gate/part, batches `CreateSchematicSymbol` with a symbol-first
  `CreateComponentInstance`.
- **(3) CLI:** `datum-eda project place-symbol <dir> --sheet <uuid>
  --reference R1 --value 10k --lib-id <id> --x-nm <nm> --y-nm <nm>
  --rotation-deg <deg> [--mirrored]`
- **(4) MCP/AI:** `datum.schematic.place_symbol` exists for the current
  symbol lifecycle slice.
- **(5) AI query/context:** Resolved `DesignModel` at a known
  `model_revision`: pool library refs, existing `sheet_id`s, occupied
  grid positions, the current reference-designator set (collision
  avoidance), target net intent. A `ComponentInstance` id is minted and
  carried so `PlacedSymbol` ↔ `PlacedPackage` can join later. Reads fold
  under the shared `datum-eda query` namespace
  (`library.query`/`schematic.query`).
- **(6) Validating checks:** ERC `unconnected_component_pin` once pins
  exist; reference-designator uniqueness; `lib_id` resolvability against
  the pool; post-commit `model_revision` recompute.
- **(7) Proof slice:** On `datum-test`, place one R symbol on the main
  sheet; assert `OperationResult.created` carries the symbol UUID + a
  freshly minted `ComponentInstance` id, the journal gained one
  `TransactionRecord`, undo removes it, `model_revision` changes then
  reverts.
- **(8) Not yet supported:** Compatibility/non-pool symbols do not mint
  ComponentInstances, ambiguous/unresolved library bindings remain explicit
  non-ComponentInstance placements, and board package refs attach only after
  the physical package exists.

### 2. transform-symbol (move / rotate / mirror unified)

- **(1) Manual UI:** Select symbol(s); drag to move, R to rotate, X/Y to
  mirror; release commits a local undoable edit.
- **(2) Operation:** `TransformSymbol{symbol_id, position?, rotation?,
  mirror?}` (one op, three optional params) → direct local `commit()`
  (local/visible/undoable, direct-commit class). Today three separate
  fns (move/rotate/mirror_native_project_symbol) now use the journaled
  schematic operation path.
- **(3) CLI:** Current CLI keeps separate `move-symbol`, `rotate-symbol`,
  and `mirror-symbol` commands.
- **(4) MCP/AI:** `datum.schematic.move_symbol`, `rotate_symbol`, and
  `mirror_symbol` exist for the current transform slice.
- **(5) AI query/context:** `symbol_id`, current position/rotation/mirror
  (`schematic.query` symbols), grid, sheet bounds, attached wire
  endpoints that must stay connected.
- **(6) Validating checks:** Geometry on grid; wires re-evaluated by
  derived connectivity (`crates/engine/src/connectivity/mod.rs`); no ERC
  severity change required to commit (local edit).
- **(7) Proof slice:** On `datum-test`, move a symbol +1mm, rotate 90,
  mirror; assert `OperationResult.modified = symbol uuid`, undo restores
  the exact prior `PlacedSymbol` bytes, derived `net_info` recomputes
  identical net membership.
- **(8) Delivery boundary:** A compatible Symbol selection uses the existing
  drag, M and selected-symbol menu Move paths, with R/X/Y and one atomic edit
  (`dat-schematic-group-transform-7a5p`). The pickup point specified by UVT
  section 2.2.12 is the displayed common group reference, translated with the
  destination. Attached wires are updated once against the complete final
  geometry; owned fields follow their Symbols. Cancel restores the source.
  Unsupported members, locks, repeated occurrences and unsupported fixed
  contacts refuse the complete group. The existing native verb retains its
  singleton absolute-pose form and adds a distinct group-delta form. Supported
  schematic-only, single-Symbol `ComponentInstance` bindings are now preserved
  by the singleton/group transform implementation in the same atomic pose,
  attached-wire and history transaction (`dat-schematic-bound-move-p8ks`).
  Physical-package, multi-Symbol and stale bindings refuse the complete edit;
  occurrence/contact limits are unchanged. Independent GUI verification covered
  singleton/group transforms, current bindings, attached wires, Undo/Redo and reopening.
  Public CLI normalization and broader occurrence/contact transforms remain pending.

### 3. delete-object (any schematic object, by id list)

- **(1) Manual UI:** Select any schematic object, press Delete; removes it
  as a local undoable edit.
- **(2) Operation:** `DeleteObjects{ids:[ObjectId]}` (one op,
  heterogeneous id list) → `OperationBatch`. Batch/cross-domain deletes
  (e.g. a symbol with a bound board `ComponentInstance`) escalate to a
  Proposal. Current type-specific native delete builders emit canonical
  journaled operations; the normalized heterogeneous surface remains a target.
- **(3) CLI:** `datum-eda project delete --root <dir> --id <uuid>
  [--id <uuid>...]` (collapses delete-symbol/-wire/-junction/-label/
  -port/-bus/-bus-entry/-noconnect/-text/-drawing).
- **(4) MCP/AI:** `delete_objects` (not present). Returns
  `OperationResult` diff(deleted); a Proposal when deletion crosses into
  placed-package territory.
- **(5) AI query/context:** Object ids and their types (recoverable from
  the id in the resolved model); whether any deleted symbol has a bound
  `ComponentInstance` with a `PlacedPackage` (forces a Proposal); dangling
  wire/junction cleanup implications.
- **(6) Validating checks:** Post-delete ERC for newly
  unconnected/dangling objects; `ComponentInstance` orphan check;
  `model_revision` recompute.
- **(7) Proof slice:** On `datum-test`, delete one symbol + its attached
  wire in one batch op; assert two deleted refs, undo restores both
  atomically; deleting a board-bound symbol instead returns a Proposal.
- **(8) Delivery boundary:** Supported multi-object schematic selections use
  an inline Proposal preview, explicit Tab acceptance or Escape dismissal,
  and one undoable design deletion (`dat-schematic-batch-delete-m8zf`). The
  complete selected set is validated; owned children are deduplicated and
  surviving external dependencies refuse the operation. Dismissal preserves
  design source; Proposal review metadata may be journaled. Supported single-Symbol,
  schematic-only component bindings now use the same review, removing the owned
  binding and no-connect markers atomically while preserving unrelated wires and
  library source (`dat-schematic-bound-delete-ahmx`). Undo/Redo and reopening
  restore exact identities and valid binding pins. Existing root-view limits,
  physical/multi-Symbol bindings, stale or repeated sources, external dependencies,
  hierarchy-bound Ports and shared Bus restrictions remain. The public normalized id-list surface, remaining
  object classes and board-bound escalation are still pending.

### 4. draw-wire

- **(1) Manual UI:** `Place > Wire`, click start, click vertices,
  double-click/Esc to finish; auto-junction where it crosses an existing
  net.
- **(2) Operation:** `DrawWire{sheet_id, points:[Point], net_hint?}` →
  `OperationBatch`; co-emits implicit `PlaceMarker{kind:Junction}` at
  T-intersections in the SAME batch (one undo entry). Current
  `draw_native_project_wire` uses the journaled schematic operation path, but
  does not infer junctions.
- **(3) CLI:** `datum-eda project draw-wire <dir> --sheet <uuid>
  --from-x-nm <nm> --from-y-nm <nm> --to-x-nm <nm> --to-y-nm <nm>`
- **(4) MCP/AI:** `datum.schematic.draw_wire` and `delete_wire` exist for
  the current straight-segment slice. Future richer wire authoring must return
  the batch including any auto-junctions.
- **(5) AI query/context:** Sheet pin coordinates and existing
  wire/junction geometry (`schematic.query`
  wires/junctions/symbol_pins) so endpoints snap to pins and
  T-intersections auto-junction; target net name if labeling intent is
  known.
- **(6) Validating checks:** Endpoint-on-pin/grid; derived connectivity
  recompute merges/splits nets; ERC `output_to_output_conflict` / driver
  rules over the changed net.
- **(7) Proof slice:** On `datum-test`, draw a wire joining two symbol
  pins; assert derived `net_info` shows both pins on one net, an
  auto-junction op was batched at any T, undo restores prior
  connectivity.
- **(8) Not yet supported:** No auto-junction inference or polyline/arc wire
  authoring in the canonical MCP slice yet.

### 5. place-label (kind: local | global | hierarchical | power)

- **(1) Manual UI:** `Place > Net Label` (or Global/Hierarchical Label,
  or Power Port); click on a wire/pin and type the net name.
- **(2) Operation:** `PlaceLabel{sheet_id, position, text, kind:
  Local|Global|Hierarchical|Power}` → `OperationBatch`. `LabelKind`
  already exists in the IR, so kind is a field, not separate tools.
  Rename = `SetObjectField` on label text; delete = `DeleteObjects`.
- **(3) CLI:** `datum-eda project place-label --root <dir> --sheet <uuid>
  --x <nm> --y <nm> --text <net> --kind <local|global|hier|power>`
- **(4) MCP/AI:** `datum.schematic.place_label`, `rename_label`, and
  `delete_label` exist for the current label slice. `place_label` uses one
  kind enum instead of separate local/global/hierarchical/power tools.
- **(5) AI query/context:** Existing net names (`schematic.query`
  nets/labels), wire endpoints to attach to, hierarchy port names a
  hierarchical label must match, power-net conventions (GND/+3V3).
- **(6) Validating checks:** Derived connectivity re-resolves net
  membership/merge (lowest `NetId` survives); ERC
  `power_in_without_source`, hierarchical-connectivity-mismatch;
  `unconnected_interface_port` if a hierarchical label has no matching
  port.
- **(7) Proof slice:** On `datum-test`, add a global label VCC on a pin;
  assert that pin's net merges into VCC in derived `net_info`,
  `power_in_without_source` clears if a source is present, undo reverts
  net membership.
- **(8) Delivery boundary:** For a uniquely qualified source occurrence, native
  GUI creation, Move/Delete and Inspector name/kind editing use canonical writes
  and journal history. Editing previews an immutable replacement at the existing
  position, with Apply/Cancel and exact undo/redo/reopen (delivery
  `dat-schematic-label-edit-yx35`). Context-menu entry remains separate work;
  contract-shaped result/normalization coverage remains pending. Label kind
  remains one parameterized capability. Existing Label/Power Move preserves raw
  label and attached-wire metadata, including nested point extensions, through
  cancellation and canonical history (`dat-schematic-label-move-payload-h1m9`).
  This does not claim the same preservation for every label/port field writer.

### 6. place-marker (kind: junction | no-connect)

- **(1) Manual UI:** `Place > Junction` at a wire crossing to force a
  connection; `Place > No-Connect` on an intentionally open pin.
- **(2) Operation:** `PlaceMarker{sheet_id, position, kind:
  Junction|NoConnect}` → `OperationBatch` (local edits). Junction is also
  auto-emitted at exact T contacts inside `DrawWire` batches. Implemented substrate slice:
  `Operation::PlaceSchematicMarker { sheet_id, marker_id, marker_kind,
  marker }` is the journal vocabulary for both point-marker placements.
  It persists into either the `junctions` or `noconnects` sheet map based on
  `marker_kind`, replays through `ProjectResolver`, and undoes by emitting the
  matching legacy delete inverse.
- **(3) CLI:** `datum-eda project place-marker --root <dir> --sheet <uuid>
  --x <nm> --y <nm> --kind <junction|noconnect>` is the normalized target.
  Current compatibility commands `place-junction` and `place-noconnect` remain
  public but now author `PlaceSchematicMarker` through the engine write
  facade.
- **(4) MCP/AI:** `place_marker` (not present). One tool, kind enum;
  returns `OperationResult`. Current MCP compatibility aliases still expose
  `datum.schematic.place_junction` and `datum.schematic.place_noconnect`.
- **(5) AI query/context:** For junction, wire-crossing coordinates
  (`schematic.query` wires/junctions). For no-connect, pins intended to
  be left open (`schematic.query` symbol_pins). Same point-placement
  query shape, parameterized by kind.
- **(6) Validating checks:** `noconnect_connected` ERC (NC on a connected
  pin = warning); `unconnected_component_pin` suppressed where NC is
  present; derived connectivity merge at a junction.
- **(7) Proof slice:** On `datum-test`, place-marker kind=noconnect on a
  deliberately open pin; assert `unconnected_component_pin` for that pin
  clears while `noconnect_connected` stays absent, undo restores the
  warning. Then place-marker kind=junction at a T; assert net merge.
- **(8) Delivery boundary:** Native GUI placement uses the normalized journal
  operation through `native.write`; preview/cancel, singleton deletion and
  history/reopen are also delivered (`dat-schematic-marker-tools-jwiv`).
  Preview/cancel writes nothing; deletion uses the canonical delete operations.
  Junction acquisition uses
  exact nearby wire contacts, including off-grid crossings; ambiguous or
  unrepresentable intersections are refused rather than rounded. No-Connect
  requires an exact unique pin. Normalized public CLI/MCP entry points and
  contract-shaped `OperationResult` remain target work; native-write registry
  availability does not establish those public entry points.

### 7. place-port (create-or-edit, hierarchical / interface port)

- **(1) Manual UI:** `Place > Port` or a sheet-pin on a hierarchical
  sheet symbol; set name + direction; double-click to edit.
- **(2) Operation:** `PlacePort{port_id?, sheet_id, position, name,
  direction}` → `OperationBatch`. With no `port_id` it creates; with a
  `port_id` it edits (collapses the separate edit verb, mirroring
  `TransformSymbol`'s optional-param edit shape). Current CLI retains
  separate journaled create/edit/delete commands.
- **(3) CLI:** `datum-eda project place-port <dir> --sheet <uuid>
  --x-nm <nm> --y-nm <nm> --name <id> --direction <input|output|...>`;
  `edit-port <dir> --port <uuid> [--name ...] [--direction ...]`; and
  `delete-port <dir> --port <uuid>`.
- **(4) MCP/AI:** `datum.schematic.place_port`, `edit_port`, and
  `delete_port` exist for the current port slice.
- **(5) AI query/context:** Parent-sheet hierarchical-label names this
  port must mirror, existing ports (`schematic.query` ports), direction
  semantics for ERC.
- **(6) Validating checks:** Hierarchical-connectivity-mismatch (parent
  label set vs child port set), `unconnected_interface_port`; derived
  connectivity refresh.
- **(7) Proof slice:** On `datum-test` (or a hierarchy fixture), add a
  port matching a parent hierarchical label; assert the
  hierarchical-connectivity-mismatch count drops, undo restores it; then
  re-issue with `--port` to rename and assert a single modified diff.
- **(8) Not yet supported:** Complete sheet-symbol/sheet-pin authoring,
  hierarchy rewiring, double-click port editing, and normalized public CLI/MCP
  entry points remain open. Native `Place > Port` now configures the shared
  name/direction form and immutable pin/grid preview. Inspector editing preserves
  the selected port's identity and position, with one Apply/Cancel action; native
  `place_port` accepts an optional existing port and qualified occurrence path.
  Singleton deletion, undo/redo and reopening use the canonical journal.
  Deleting a port still bound to a sheet instance is refused until its binding
  is explicitly removed. This bounded interface-port workflow does not establish
  complete hierarchy authoring or full schematic qualification.

### 8. create-bus (members + bus-entry as parameters)

- **(1) Manual UI:** `Place > Bus`, draw it; `Place > Bus Entry` to tap a
  member wire onto it; bus members named D[0..7].
- **(2) Operation:** `CreateBus{bus_id?, sheet_id, points, members:[net],
  entries:[{position, wire?}]}` → `OperationBatch`. Members and bus
  entries are parameters of the bus op (`BusEntry` carries `bus:Uuid` +
  optional `wire` — it is bus-bound, NOT a free point marker, so it stays
  in the bus tool, not place-marker). With a `bus_id`, edits
  members/geometry (collapses edit-bus-members). Current CLI retains
  separate journaled bus and bus-entry commands.
- **(3) CLI:** `datum-eda project create-bus <dir> --sheet <uuid> --name
  <bus> --member <net>...`; `edit-bus-members <dir> --bus <uuid>
  --member <net>...`; `place-bus-entry <dir> --sheet <uuid> --bus <uuid>
  --x-nm <nm> --y-nm <nm> [--wire <uuid>]`; and `delete-bus-entry <dir>
  --bus-entry <uuid>`.
- **(4) MCP/AI:** `datum.schematic.create_bus`, `edit_bus_members`,
  `place_bus_entry`, and `delete_bus_entry` exist for the current bus slice.
- **(5) AI query/context:** Existing net names that should become bus
  members (`schematic.query` nets/buses), bus geometry, entry tap points
  on member wires.
- **(6) Validating checks:** Derived connectivity expands the bus into
  member nets; ERC over each member net; bus-member name well-formedness.
- **(7) Proof slice:** On `datum-test` (or a bus fixture), create bus
  D[0..3] with one entry tapping a wire; assert derived `net_info` shows
  4 member nets and the tapped wire joins D0, undo reverts.
- **(8) Not yet supported:** New scalar member Nets from previously unused
  names, shared-declaration/hierarchy editing, and normalized public CLI/MCP
  entry points remain open. Native `Place > Bus` uses the shared polyline
  gesture and explicit names of existing, distinct canonical scalar Nets;
  missing or ambiguous members are refused. Native `create_bus` accepts an
  optional existing bus and member/geometry/entry parameters. Bus Entry binds
  an existing member wire endpoint to an exact bus contact without shorting
  scalar members. The Bus Inspector edits name/members and deletes qualified
  owned entries. Deleting the selected bus removes its spine and owned entries
  in one journal transaction, preserves scalar wires/Net identities, and retains
  the semantic Bus identity under the existing projection lifecycle. Cancel,
  undo/redo and reopening remain part of this bounded native workflow; this is
  not complete bus/hierarchy authoring or full qualification.

### 9. set-symbol-field (reference/value/lib_id/part/entity/unit/gate/...)

- **(1) Manual UI:** Double-click a symbol or use the
  Inspector / Symbol Fields Table to edit reference, value, footprint,
  custom fields; assign part/library.
- **(2) Operation:** `SetSymbolField{symbol_id, field, value, clear?}` as
  ONE parametric op. The field enum covers
  `reference|value|lib_id|part|entity|unit|gate|display_mode|hidden_power|
  custom:<name>`. Current compatibility set/clear and add/edit/delete field
  builders use canonical journaled symbol operations; the normalized field
  surface remains a target.
- **(3) CLI:** `datum-eda project set-symbol-field --root <dir> --symbol
  <uuid> --field <reference|value|lib-id|part|entity|unit|gate|
  display-mode|hidden-power|custom:NAME> --value <v> [--clear]`
- **(4) MCP/AI:** `set_symbol_field` (not present). One tool, field+value
  params; returns `OperationResult` diff(modified).
- **(5) AI query/context:** Current symbol fields/semantics
  (`schematic.query` symbol_fields/symbol_semantics/symbol_pins), pool
  part/entity ids for assignment, reference uniqueness, gate/unit
  enumeration for multi-unit parts.
- **(6) Validating checks:** Reference uniqueness; part/entity/gate
  resolvability against the pool; pin-semantics recompute (drives ERC
  driver rules); `model_revision` recompute. Binding a part establishes
  the `ComponentInstance` → library join.
- **(7) Proof slice:** On `datum-test`, set R1 value 10k→4k7 and assign a
  pool part; assert a single `OperationResult.modified`, undo restores
  both, `symbol_semantics` reflect the new pin types.
- **(8) Delivery boundary:** The selected-Symbol form supports reference/value
  and UUID-qualified custom metadata add/edit/remove through shared controls
  (`dat-schematic-custom-fields-3bec`). New metadata starts hidden and unpositioned.
  Show/Hide and explicit Place/Move text controls author visibility and position
  (`dat-schematic-field-presentation-jvu3`); Show without a position enters
  placement rather than inventing one. Text remains owned by its Symbol, with a
  selection/gesture tether. Supported parent transforms carry positioned fields,
  including hidden ones, without extending occurrence/contact admission.
  Custom-field, reference/value and parent-transform writes preserve unrelated
  raw symbol/pin/field payload. Cancel/no-op do not write; accepted changes use
  canonical history.

  Bounded Part assignment implementation (`dat-schematic-part-assignment-dres`)
  adds named compatible Part/Gate choices through the selected Symbol Inspector
  and existing `Assign Part…` context command. It retains the native pool Symbol's
  Unit and geometry. Explicit shared Proposal acceptance atomically creates an
  initially missing `ComponentInstance` or updates one existing single-Symbol,
  schematic-only instance and its current library binding. Current source and
  compatible pin mappings are revalidated; pin UUIDs/anchors, fields, raw
  extensions and unaffected connectivity are preserved, with one-step design
  Undo/Redo and reopening. Physical-package bindings, multi-Symbol components,
  repeated occurrences and incompatible mappings remain visibly unavailable or
  refused with reasons. Independent GUI verification covered named assignment and
  reassignment, dismissal, Undo/Redo and reopening. Supported bound-Symbol
  Move/Rotate/Mirror implementation is covered by §2. Existing Reference/Value
  and custom metadata/presentation controls also preserve supported single-Symbol,
  schematic-only bindings (`dat-schematic-bound-fields-0r9i`), including raw
  payload, library identity and connectivity through cancellation, Undo/Redo
  and reopening. Physical, multi-Symbol, stale and repeated-source restrictions
  remain. Supported bound-Symbol deletion is covered by §3. The existing Inspector
  distinguishes a valid schematic binding with no declared board counterpart from
  stale, missing or ambiguous bindings (`dat-schematic-binding-inspector-a7rm`);
  owning a component does not imply cross-domain selection or a complete board graph.

  Automatic position presets, a bulk fields table, cross-Unit replacement,
  broader part/entity/gate reassignment, complete public-surface normalization
  and guaranteed `ComponentInstance` joins for all Symbols remain pending.

### 10. run-erc (read-only) + author-waiver

- **(1) Manual UI:** `Checks > Run ERC`; select a current finding, then
  `Checks > Waive…` or `Accept Deviation…` with a native Schematic pane
  focused. Enter a rationale in the shared form, preview the Proposal, then
  explicitly accept with Tab or dismiss with Escape.
- **(2) Operation:** ERC evaluation is read-only; explicit Run ERC persists
  revision-qualified check evidence. Findings are addressed by their actual
  `CheckRun` and stable fingerprint, never a row index. Existing
  `CreateSchematicWaiver` / `CreateSchematicDeviation` operations record durable
  dispositions. The GUI configures `native.proposal` with `CreateDisposition`,
  reusing these builders and the shared acceptance lifecycle rather than a
  direct waiver commit. Acceptance applies the disposition and Applied metadata
  atomically; dismissal leaves design source unchanged, though Proposal metadata
  may be journaled.
- **(3) CLI:** `datum-eda project query <root> erc` is the read-only report.
  Existing `project waive-finding <root> --fingerprint <id> --rationale <text>`
  and `project accept-deviation <root> --fingerprint <id> --rationale <text>`
  use canonical operations. Their direct-write policy reconciliation remains
  separate (`dat-waiver-direct-policy-hdj1`); backend availability does not
  waive the Proposal-first requirement. The normalized `erc-waive` alias remains
  pending.
- **(4) MCP/AI:** Read-only reports and existing compatibility check verbs remain
  on the shared surface. The normalized proposal-gated `author_waiver` alias is
  pending; this GUI slice does not change global write policy.
- **(5) AI query/context:** Resolved `DesignModel` + `ErcConfig` (severity
  overrides) + existing waivers; the full connectivity graph so findings
  cite stable object ids.
- **(6) Validating checks:** Self-validating: 7 rules
  (`output_to_output_conflict`, `power_in_without_source`,
  `noconnect_connected`, `input_without_explicit_driver`,
  `undriven_input_pin`, `unconnected_component_pin`,
  `unconnected_interface_port`) + hierarchical-connectivity-mismatch;
  0% FP/FN gate per M0–M5.
- **(7) Proof slice:** On `datum-test`, run ERC, assert a deterministic
  finding set + codes; introduce an output-output short via draw-wire,
  re-run, assert `output_to_output_conflict` appears, undo the wire,
  assert it clears.
- **(8) Bounded native implementation:** Current persisted ERC evidence,
  project/revision guards, unique fingerprints and nonblank rationale qualify
  the shared review (`dat-erc-disposition-gui-rs56`). Stale or already disposed
  findings refuse. Disposition changes and reopening use the existing read-only
  ERC query for reporting when current persisted evidence is absent; unchanged
  source refreshes retain that report without recomputation. These reports keep
  `persisted=false` and cannot authorize another disposition: Run ERC first.
  No extra check-evidence transaction follows acceptance, preserving one-step
  design Undo/Redo and reopening. The existing Inspector presents qualified
  human target/message, producing revision/currentness, status and available
  exact disposition rationale (`dat-erc-readable-inspector-1wof`). Technical
  identities remain expandable; shared wrapping/scrolling reaches the full
  content. Current persisted, computed read-only, stale and unavailable reports
  remain distinct; current rationale is never substituted for historical evidence.
  Removal/replacement UI, other check domains,
  normalized aliases and full qualification remain pending.

## Minimal-Set Recommendation

**Ten tools, not the ~52 native-schematic CLI functions that exist
today.** The matrix collapsed move/rotate/mirror → `TransformSymbol`, ~10
deletes → `DeleteObjects`, ~16 field setters → `SetSymbolField`, and 3–4
label kinds → `PlaceLabel{kind}`. Three further merges apply the same
parametric-kind / optional-id principle consistently:

1. place-junction + place-no-connect → `PlaceMarker{kind}` (both
   zero-dimension single-point sheet annotations).
2. edit-port folded into place-port via an optional `port_id`
   (create-or-edit, mirroring `TransformSymbol`'s optional-param edit
   shape).
3. edit-bus-members + place-bus-entry folded into create-bus as
   `members[]`/`entries[]` params with an optional `bus_id` for edit
   (`BusEntry` is bus-bound, so it correctly stays inside the bus tool
   rather than over-merging into `PlaceMarker`).

The load-bearing, non-negotiable work is NOT new tools but finishing the
contract normalization on top of the journaled schematic substrate and
minting/binding `ComponentInstance` at place-symbol and at part assignment.
Today the current schematic write slices are journaled and MCP-visible, but
the public shape is still compatibility-heavy. ERC evaluation is read-only;
explicit runs persist evidence. Existing canonical disposition operations now
serve the bounded Proposal-gated GUI in §10; normalized public aliases and
global direct-write policy reconciliation remain separate work.

Direct-commit class (local, visible, undoable): place-symbol,
transform-symbol, draw-wire, place-label, place-marker, place-port,
create-bus, set-symbol-field. Proposal-first class:
delete-of-a-board-bound-symbol, bulk delete, bulk reference reannotate,
any sheet-structure change that rewires existing nets, author-waiver.

## Omitted / Redundant Tools

- **rotate-symbol, mirror-symbol** — pure parameters of
  `TransformSymbol`. Altium/KiCad expose rotate/mirror as modifiers during
  a move, not separate operations. Two CLI verbs and one MCP tool removed
  with zero capability loss.
- **delete-symbol, -wire, -junction, -label, -port, -bus, -bus-entry,
  -noconnect, -text, -drawing** — all reduce to
  `DeleteObjects{ids:[ObjectId]}`; object type is recoverable from the id
  in the resolved model. Ten verbs → one. No reference tool has ten delete
  commands.
- **set-reference, set-value, set-lib-id, clear-lib-id, set-part,
  clear-part, set-entity, clear-entity, set-unit, clear-unit, set-gate,
  clear-gate, set-display-mode, set-hidden-power-behavior, set-pin-override,
  clear-pin-override, add-field, edit-field, delete-field** — all are
  `SetSymbolField{field, value, clear}`. Altium does this through one
  Inspector/Properties surface; `custom:<name>` + `value`/`clear` covers
  add/edit/delete field.
- **place-global-label, place-hierarchical-label, place-power-port as
  separate tools** — `LabelKind` already exists in the IR. KiCad's three
  label tools are ceremony; Datum keeps one `PlaceLabel{kind}`.
- **place-no-connect separate from place-junction** — both are
  zero-dimension single-point sheet annotations differing only by kind.
  Merged into `PlaceMarker{kind}`. KiCad keeps them separate (ceremony);
  the label-by-kind principle demands this merge for consistency.
- **edit-port separate from place-port** — a port edit is the same op
  shape as creation. Folded into `PlacePort` with an optional `port_id`.
- **edit-bus-members and place-bus-entry separate from create-bus** —
  members and bus entries are parameters of the bus, not independent
  objects (`BusEntry` carries `bus:Uuid`). Folded into `CreateBus` with
  `members[]`/`entries[]` and an optional `bus_id` for edit.
- **rename-label op** — rename is `SetObjectField` on the label text (a
  display change per the `NetAnchor` rule); no separate operation.
- **Sheet-structure authoring** — native sheet creation, definitions,
  instances and bindings already have canonical journaled operations. These
  structural operations remain outside this ten-tool inventory. Exposing flat
  document creation through the ordinary GUI does not deliver broader hierarchy
  editing; sheet-pin authoring and cross-sheet net rewiring retain their existing
  boundaries. See **Not-Yet-Supported**.
- **any commit/save schematic tool distinct from the global `commit()`** —
  there is exactly one `commit()` primitive in the ratified spine; a
  per-domain save tool would be a forbidden private write path (precisely
  the defect today).

## Shared Surface

This contract adds ONLY the ten schematic-specific typed Operations above.
It does NOT redefine the seven shared operations; see
`docs/contracts/AI_CLI_MCP_TOOL_SURFACE.md` for:

- **Session/Context** (`DatumToolSession` + `DatumContextEnvelope`,
  `datum-eda context get|refresh`) — every CLI/MCP/AI invocation opens here;
  this contract mints no session of its own.
- **Query** (`DatumQueryTool`, `datum-eda query <family>`) — all schematic
  reads (`schematic.query`, `library.query`, `object.get`,
  `relationships.query`) enter as a `--family`/`--domain` parameter and
  are the `aiQueryContext` source for every tool above. Reuse the
  existing read tools (`get_schematic_summary`/`net_info`/`symbols`/...),
  do not reinvent them per tool.
- **Run-check** (`DatumCheckTool`, `datum-eda check run|show`) — ERC is the
  schematic domain of the one check surface; `run_erc` is a thin alias.
- **Propose** (`DatumProposalTool`) — the lifecycle for board-bound-symbol
  delete, bulk delete, bulk reannotate, net-rewiring sheet changes, and
  author-waiver.
- **Apply/Commit** (`DatumCommitTool`, the single `commit()`) — the sole
  mutation gateway every op above flows through; there is no per-domain
  schematic save.
- **Artifact** (`DatumArtifactTool`) — schematic netlist export is an
  artifact projection, not a schematic authoring op.
- **Journal + Undo/Redo** (`DatumJournalTool`) — global undo/redo of any
  schematic transaction is a compensating batch through the same
  `commit()`; this contract ships no per-domain undo. Shared logical history
  tracks repeated Undo/Redo across reopening and clears pending Redo on a new
  normal transaction (`dat-journal-multistep-undo-lr67`). The journal remains
  append-only; noninvertible barriers and Proposal metadata treatment remain
  unchanged. Supported component compensation validates immutable recorded
  semantics and current source before rebasing only permitted revision pins.

Selection is consumer-side state, never an Operation and never journaled;
it is read-only via `selection.describe`. There is no schematic
select/deselect/filter mutation tool.

## Proof Slice & Fixture

**Primary fixture:** the `datum-test` KiCad fixture at
`~/Documents/kicad_projects/Datum-eda/datum-test/datum-test.kicad_sch` (a
real schematic, per the `reference_datum_test_fixture` memory). Import it,
then exercise place-symbol / transform-symbol / draw-wire / place-label /
place-marker / set-symbol-field / run-erc round-trips with undo.

**Hierarchy/native proofs** that `datum-test` cannot cover (port ↔
hierarchical-label matching, native authoring): use the in-repo native
fixtures under
`crates/test-harness/testdata/quality/native_validation_cases_v1/` and the
hierarchy-mismatch-demo golden referenced in
`crates/cli/src/main_tests_query_goldens.rs`.

If a clean multi-sheet hierarchical native fixture is needed for any
broader hierarchy proof, ASK the owner to provide a real one rather than
fabricating sheet JSON (per the no-synthetic-kicad / ask-for-fixtures
memory).

**End-to-end gate for this contract:** a place-symbol → set-symbol-field
(part bind) → draw-wire → place-label → run-erc sequence on `datum-test`
that, for each step, produces exactly one journaled `TransactionRecord`
with provenance, a `model_revision` that moves and reverts under undo, and
a `ComponentInstance` minted at place-symbol and bound at part assignment.

## Not-Yet-Supported

- **Contract-normalized schematic operations.** Current schematic write slices
  use journaled operations and resolver replay, but they remain exposed mostly
  as compatibility-shaped per-kind/per-field commands. The ten-operation
  contract now has its first normalized journal operation:
  `PlaceSchematicMarker` for junction/no-connect placement. It still needs
  normalized `DeleteObjects`, `SetSymbolField`, richer `CreateBus`, public
  `place-marker`/`datum.schematic.place_marker` aliases, and consistent
  `OperationResult` / `model_revision` reporting.
- **ComponentInstance mint/bind from schematic placement.** Pool-backed
  `place-symbol` now stores part/entity/gate UUIDs and mints an authored
  symbol-first `ComponentInstance` with a `part_ref` when the resolved
  entity/gate has one compatible part. Compatibility `lib_id`s, unresolved or
  ambiguous entity-gate matches, ambiguous compatible parts, incompatible part
  mappings, and later board-package attachment remain explicit partial states.
- **Hierarchy authoring.** Sheet definition, sheet instance create/delete/move,
  and parent-port bind/unbind operations are implemented and journaled.
  Sheet-pin authoring and broader net-rewiring hierarchy edits remain
  proposal-first target work.
- **Remaining disposition surfaces and normalization.** Revision-qualified
  ERC records, fingerprint targeting, canonical waiver/deviation operations
  and the bounded Proposal-gated GUI exist (§10). Normalized `erc-waive` /
  `author_waiver` aliases, removal/replacement UI and global direct-write policy
  reconciliation (`dat-waiver-direct-policy-hdj1`) remain pending.
- **Auto-junction inference** during `draw-wire`.
- **Remaining MCP schematic write tools.** The first journaled MCP
  schematic write aliases now exist for connectivity, labels, ports, buses,
  schematic text/drawing, and symbol lifecycle edits. Remaining authoring
  tools must land only behind the same `OperationBatch`/journal substrate so
  AI authoring is never an unjournaled surface.

## Open Owner Questions

1. **Substrate scope.** Current schematic native mutations for
   connectivity, labels, ports, buses, schematic text/drawing, and symbol
   lifecycle edits route through engine `OperationBatch` + journal +
   `ProjectResolver` replay rather than private JSON writes. The remaining
   scope question is which higher-level heterogeneous/bulk schematic edits
   graduate next, and which must be proposal-first.
2. **ComponentInstance timing.** Is `ComponentInstance` minted at
   place-symbol, or only at part assignment? Does its absence block any
   schematic authoring before the join exists?
3. **Direct-commit vs proposal boundary.** Confirm direct =
   place/transform/label/marker/field-edit/port/bus; proposal-first =
   delete-of-a-board-bound-symbol, bulk delete, bulk reference reannotate,
   and any sheet-structure change that rewires existing nets.
4. **Hierarchy GUI breadth.** Current native definition/instance/binding
   operations exist. Disposition complete GUI hierarchy/sheet-pin authoring
   and proposal-first net-rewiring scope without treating backend availability
   as ordinary designer delivery or creating a new deferral automatically.
5. **Disposition normalization.** Reconcile the remaining public aliases and
   global direct-write policy with the `009` rules/checks contract. Existing
   canonical operations and the bounded GUI in §10 do not resolve those gaps.
6. **MCP write timing.** First schematic MCP writes now exist only for
   journaled connectivity primitives (`draw_wire`, `delete_wire`,
   compatibility `place_junction`, `delete_junction`, `place_noconnect`,
   `delete_noconnect`; junction/no-connect placement now journals as
   `PlaceSchematicMarker`) plus labels (`place_label`, `rename_label`,
   `delete_label`), ports (`place_port`, `edit_port`, `delete_port`), and
   buses (`create_bus`, `edit_bus_members`, `place_bus_entry`,
   `delete_bus_entry`), and symbol lifecycle edits (`place_symbol`,
   `move_symbol`, `rotate_symbol`, `mirror_symbol`, `delete_symbol`,
   `set_symbol_reference`, `set_symbol_value`, symbol metadata, pin override,
   and symbol field aliases). Remaining schematic write tools, especially
   unified heterogeneous `delete_objects`, should land only behind the same
   journaled substrate.

## PM053 C01–C03 owning reconciliation — 2026-10-01

[PM053](../decisions/PRODUCT_MECHANICS_053_ELECTRICAL_IDENTITY_AUTHORITY.md) ratifies stable NetId final-state atomic nomination/fallback,
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
