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

Out of scope for this slice: hierarchy/sheet-structure authoring
(create-sheet / sheet-instance / sheet-pin), variant overlay editing, and
library object authoring (see `008` contract). These are documented as
gaps, not tools.

### Current-engine grounding (verified)

Native schematic authoring today does **not** flow through the engine
Operation/commit/journal. It is a private JSON write path, confirmed in:

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

All load via `load_native_project`, mutate a `serde_json::Value`
(`write_symbol_into_sheet` etc. in
`crates/cli/src/command_project_schematic_helpers.rs`), and persist with
`write_canonical_json`. There is NO `OperationResult`, NO
`TransactionRecord`, NO undo/redo, NO `model_revision`. By contrast the
imported-board ops in `crates/engine/src/api/write_ops/basic_mutations.rs`
push a `TransactionRecord` onto the undo stack
(`crates/engine/src/api/save_kicad/transaction_state.rs`) and return an
`OperationResult{OperationDiff}` — but they are board/KiCad-only and still
lack an fsync/journal `commit()`.

Confirmed IR facts the matrix relies on:

- `LabelKind` (`Local|Global|Hierarchical|Power`) is already a field on
  `NetLabel` (`crates/engine/src/schematic/mod.rs:145`). Label kind is a
  parameter, not a reason for separate tools.
- `BusEntry` carries `bus: Uuid` + optional `wire: Uuid` + position
  (`schematic/mod.rs:168`). It is bus-bound, not a free point marker.
- `Schematic.sheet_instances: HashMap<Uuid, SheetInstance>` exists in the
  IR (`schematic/mod.rs:13`) but has NO authoring op — only the
  `ProjectNew` scaffold.
- ERC runs over an in-memory `Schematic`
  (`crates/engine/src/erc/mod.rs:50`,
  `run_prechecks_with_config_and_waivers`): 7 rules + a
  hierarchical-connectivity-mismatch check
  (`erc/mod.rs:291`), with severity overrides and waivers; NOT
  revision-keyed.
- There is NO `ProjectResolver`, NO single resolved `DesignModel`, and NO
  `ComponentInstance` in the schematic IR. `PlacedSymbol` carries a
  reference string + optional part/entity/gate UUIDs.
- MCP catalog (`mcp-server/tools_catalog_data.py`) exposes only read-only
  schematic tools (`get_schematic_summary`/`net_info`/`symbols`/... and
  the unified check report). There are NO schematic write tools.

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
  schematic operation path, but ComponentInstance mint semantics remain
  follow-on.
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
- **(8) Not yet supported:** Automatic ComponentInstance minting/binding at
  place time remains unresolved.

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
- **(8) Not yet supported:** Three redundant CLI verbs today instead of
  one parametric op; no undo; no operation emission.

### 3. delete-object (any schematic object, by id list)

- **(1) Manual UI:** Select any schematic object, press Delete; removes it
  as a local undoable edit.
- **(2) Operation:** `DeleteObjects{ids:[ObjectId]}` (one op,
  heterogeneous id list) → `OperationBatch`. Batch/cross-domain deletes
  (e.g. a symbol with a bound board `ComponentInstance`) escalate to a
  Proposal. Today ~10 separate `delete_native_project_*` JSON writers,
  no undo.
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
- **(8) Not yet supported:** Ten type-specific delete verbs today instead
  of one id-list op; no undo; no proposal escalation for board-bound
  symbols.

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
- **(8) Not yet supported:** No operation/undo. A KiCad-style surface
  would split this into 3–4 label tools; Datum keeps one parametric op.

### 6. place-marker (kind: junction | no-connect)

- **(1) Manual UI:** `Place > Junction` at a wire crossing to force a
  connection; `Place > No-Connect` on an intentionally open pin.
- **(2) Operation:** `PlaceMarker{sheet_id, position, kind:
  Junction|NoConnect}` → `OperationBatch` (local edits). Junction is also
  auto-emitted inside `DrawWire` batches. Today
  `place_native_project_junction` and `place_native_project_noconnect`
  are two near-identical single-point JSON writers; both are
  zero-dimension point annotations differing only by kind.
- **(3) CLI:** `datum-eda project place-marker --root <dir> --sheet <uuid>
  --x <nm> --y <nm> --kind <junction|noconnect>` (collapses
  place-junction / place-noconnect).
- **(4) MCP/AI:** `place_marker` (not present). One tool, kind enum;
  returns `OperationResult`.
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
- **(8) Not yet supported:** No operation/undo; junction not auto-inferred
  during wire draw; two redundant point-placement verbs today instead of
  one parametric op.

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
- **(8) Not yet supported:** No operation/undo. No
  sheet-symbol-with-pins authoring op (see the create-sheet gap).
  edit-port collapses into this create-or-edit op.

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
- **(8) Not yet supported:** The future unified bus create/edit operation
  that folds geometry, members, and entries into one richer authoring command
  is not yet exposed.

### 9. set-symbol-field (reference/value/lib_id/part/entity/unit/gate/...)

- **(1) Manual UI:** Double-click a symbol or use the
  Inspector / Symbol Fields Table to edit reference, value, footprint,
  custom fields; assign part/library.
- **(2) Operation:** `SetSymbolField{symbol_id, field, value, clear?}` as
  ONE parametric op. The field enum covers
  `reference|value|lib_id|part|entity|unit|gate|display_mode|hidden_power|
  custom:<name>`. Today ~16 distinct set_/clear_ JSON writers +
  add/edit/delete field.
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
- **(8) Not yet supported:** Sixteen redundant set/clear verbs collapse to
  one op + field enum (+`clear` flag); no undo; part assignment does not
  yet mint/bind a `ComponentInstance`.

### 10. run-erc (read-only) + author-waiver

- **(1) Manual UI:** `Tools > Electrical Rules Check`; review violations;
  waive an accepted finding.
- **(2) Operation:** run-erc emits NO operation (read-only derived check)
  producing a revision-keyed `CheckRun`/`CheckFinding` set keyed to
  `model_revision`. Authoring a waiver DOES emit `AuthorWaiver{rule,
  target, justification}` → a proposal-gated transaction (a waiver is a
  durable authored record). Today
  `run_prechecks_with_config_and_waivers` runs over an in-memory
  `Schematic`, not revision-keyed; there is no waiver-authoring op.
- **(3) CLI:** `datum-eda project erc --root <dir>` (and `datum-eda project
  check` for the unified report) — IMPLEMENTED
  (`query_native_project_erc`). Waiver: `datum-eda project erc-waive --root
  <dir> --rule <code> --target <id> --reason <text>` (NOT YET).
- **(4) MCP/AI:** the read-only ERC report is PRESENT via the shared
  check surface (`get_check_report` in `tools_catalog_data.py`).
  `author_waiver` is not present (proposal-gated).
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
- **(8) Not yet supported:** Findings not yet revision/hash-keyed to
  `model_revision`; waiver authoring not yet exposed as a proposal-gated
  operation/CLI/MCP tool (owner to confirm whether it lands here or in the
  `009` rules/checks contract).

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

The load-bearing, non-negotiable work is NOT new tools but routing all ten
through the missing engine `commit()`/journal/`ProjectResolver` and
minting/binding `ComponentInstance` at place-symbol and at part
assignment. Today every native schematic write is a private JSON path with
no undo, directly violating the readiness audit. ERC stays read-only (no
operation) and is implemented; only proposal-gated waiver authoring is new
and may defer to the `009` contract.

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
- **create-sheet / sheet-instance / sheet-pin authoring (this slice)** —
  NOT redundant but OUT OF SCOPE: entirely unimplemented (only the
  `ProjectNew` scaffold + read queries; `sheet_instances` exists in the IR
  but has no authoring op), structural and cross-sheet (net-rewiring →
  proposal class), and an open owner question. Demoted from the tool set
  to a documented gap; including it would be a maximal-catalog defect for
  a slice that cannot prove it. See **Not-Yet-Supported**.
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
  `commit()`; this contract ships no per-domain undo.

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
future create-sheet proof, ASK the owner to provide a real one rather than
fabricating sheet JSON (per the no-synthetic-kicad / ask-for-fixtures
memory).

**End-to-end gate for this contract:** a place-symbol → set-symbol-field
(part bind) → draw-wire → place-label → run-erc sequence on `datum-test`
that, for each step, produces exactly one journaled `TransactionRecord`
with provenance, a `model_revision` that moves and reverts under undo, and
a `ComponentInstance` minted at place-symbol and bound at part assignment.

## Not-Yet-Supported

- **commit()/journal/ProjectResolver substrate.** All ten ops currently
  run as private JSON writes (`load_native_project` →
  `write_canonical_json`) with no `OperationResult`, no
  `TransactionRecord`, no undo, no `model_revision`. This substrate is the
  hard prerequisite for the entire contract and is a foundational slice in
  its own right (see the shared-surface substrate prerequisite).
- **ComponentInstance.** Not present in the schematic IR; `PlacedSymbol`
  carries a reference string + optional part/entity/gate UUIDs. The
  electrical-to-physical join cannot be formed until it is minted at
  place-symbol and bound at part assignment.
- **Hierarchy authoring.** create-sheet / sheet-instance / sheet-pin have
  no authoring op (only `ProjectNew` scaffold + read queries;
  `sheet_instances` is in the IR). Deferred to a later slice; net-rewiring
  sheet changes are proposal-first.
- **Revision-keyed ERC + waiver authoring.** ERC findings are not yet
  `model_revision`/fingerprint-keyed (current addressing is
  `(domain,index)`), and the proposal-gated `author-waiver` op /
  `erc-waive` CLI / `author_waiver` MCP tool are unimplemented. run-erc
  itself is implemented and read-only.
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
4. **Hierarchy deferral.** Confirm create-sheet / sheet-instance /
   sheet-pin authoring is deferred to a later slice (only `ProjectNew`
   scaffold + read queries exist today).
5. **Waiver home.** Should the proposal-gated `author-waiver` op land in
   THIS schematic contract, or move to the `009` rules/checks contract?
   (run-erc itself is already implemented and read-only.)
6. **MCP write timing.** First schematic MCP writes now exist only for
   journaled connectivity primitives (`draw_wire`, `delete_wire`,
   `place_junction`, `delete_junction`, `place_noconnect`,
   `delete_noconnect`) plus labels (`place_label`, `rename_label`,
   `delete_label`), ports (`place_port`, `edit_port`, `delete_port`), and
   buses (`create_bus`, `edit_bus_members`, `place_bus_entry`,
   `delete_bus_entry`), and symbol lifecycle edits (`place_symbol`,
   `move_symbol`, `rotate_symbol`, `mirror_symbol`, `delete_symbol`,
   `set_symbol_reference`, `set_symbol_value`, symbol metadata, pin override,
   and symbol field aliases). Remaining schematic write tools, especially
   unified heterogeneous `delete_objects`, should land only behind the same
   journaled substrate.
