# PCB Layout Tool Contract

Status: draft implementation-spec 2026-06-19; derived from ratified
PRODUCT_MECHANICS 000-012

## Driven by

- `docs/decisions/PRODUCT_MECHANICS_000..012` (ratified product mechanics)
- `docs/audits/scope-integration/DATUM_PRODUCT_MECHANICS_IMPLEMENTATION_READINESS_AUDIT.md`
  (ratified vocabulary, invariants, acceptance checklist)
- `docs/contracts/AI_CLI_MCP_TOOL_SURFACE.md` (the seven shared
  operations: session/context, query, check, proposal, commit, artifact,
  journal — referenced, never restated here)

## Purpose & Scope

This contract specifies the concrete, minimal PCB-layout authoring tool
set for Datum EDA across the three isomorphic surfaces (manual UI / CLI /
MCP-AI). It covers physical board authoring: component placement and
transform, track and via authoring, zone boundaries and the separate
explicit fill, keepouts, board outline, layer stackup, design rules and
net-class assignment, object inspection/edit, and DRC.

It is an implementation-spec, not product philosophy. Every tool is
mapped to the eight questions: manual UI action, the typed Operation it
emits, CLI command, MCP/AI tool, AI query/context needed, validating
checks, proof slice, and what is explicitly not-supported-yet.

Out of scope here (owned elsewhere): schematic authoring, library/pool
authoring, manufacturing artifact generation, rule-engine internals, and
the automated routing kernel (`crates/engine/src/board/route_surface.rs`
plus 60+ `route_path_candidate_*.rs`). The route-proposal kernel is
referenced as an AI context source for track authoring, not duplicated.

The single load-bearing requirement of this domain is NOT a new tool. It
is collapsing the two divergent write paths into one `commit()`/journal
primitive. Imported-board mutations already journal correctly; native
board authoring bypasses `commit()` across 112 call sites in 8 CLI files.
Retiring those private writers so each CLI verb emits a typed Operation
is the foundational PCB-domain slice — the audit forbids private writers
(audit lines 60-61, 129-132, 344-346).

## Reference-Tool Survey

Surveyed for what is load-bearing vs ceremony in the physical-layout
domain. The lean rationale follows each.

- **Altium Designer (primary reference).** Place Component / Pad / Track
  / Via / Polygon Pour / Region / Keepout / Dimension; Interactive
  Routing (PNS) with width and via taken from rules; Diff-Pair Routing;
  Move / Rotate (Space) / Flip to back (L); the Align toolbar (left /
  right / top / bottom / h-space / v-space); Repour Selected/All
  (Shift+B); Board Shape redefine; Layer Stack Manager; PCB Rules &
  Constraints Editor; the PCB Inspector for multi-object shared-property
  edit including the Locked toggle; selection filters; Design > Run DRC
  online and batch.
  - Lean read: the Align toolbar is one logical operation with a mode,
    not eight tools. Repour is an explicit derived-state refresh, never
    authored copper. Locked is an Inspector checkbox, never a verb.
    Selection filters are consumer state, never a mutation. These map
    directly onto Datum's batch-op, ZoneFill-honesty, property-edit, and
    selection-is-not-an-operation invariants.

- **KiCad 7/8.** Place Footprint; Route Tracks (X) / Diff-Pair (6); Add
  Via (V) mid-route; Draw Filled Zone (Ctrl+Shift+Z) authoring a boundary
  + Fill All Zones (B) as a DISTINCT explicit step (unfilled zones warn);
  Add Keepout / Rule Area as a separate polygon kind; Board Setup
  (stackup + design rules + net classes as one config surface);
  align/distribute submenu; F to flip to back (mirrors pads); Properties
  panel inspector (lock is a checkbox); Run DRC.
  - Lean read: KiCad is the cleanest precedent for ZoneFill honesty —
    boundary authoring and fill are separate user actions, and an
    unfilled zone visibly warns. Datum adopts this split exactly:
    `author_zone` writes boundary only; `fill_zones` regenerates derived
    copper. Keepout is a distinct polygon kind with no net and no fill.

- **OrCAD / Allegro PCB Editor.** place / move / spin / mirror; Add
  Connect interactive plus slide; Add Via; Shape Add with Global Dynamic
  fill (dynamic vs static shapes); Constraint Manager; online and batch
  DRC.
  - Lean read: Allegro's dynamic-vs-static shape distinction is the
    direct industry analog of derived ZoneFill vs authored boundary. It
    confirms that fill regeneration is a derived recompute, not a source
    edit.

- **Horizon EDA.** place package; interactive route tool with via; draw
  polygon plus an explicit update-all-planes fill step; board setup
  rules; net-class editor; every mutation is a document action on one
  undo stack.
  - Lean read: Horizon is the closest open analog to Datum's
    operation/commit/journal model — one undo stack, every edit a typed
    document action, fill an explicit separate step. It validates that a
    single journaled commit path with one global undo cursor is the
    correct shape, not a per-domain undo.

Net survey conclusion: all four reference tools converge on (1) one
align/distribute operation parameterized by mode, (2) explicit
fill-as-separate-step with honest unfilled state, (3) lock as a property
not a verb, (4) selection as non-journaled consumer state, and (5) one
undo stack. Datum's lean set follows this convergence and omits the
catalog bloat (per-direction align tools, per-scope repour tools,
per-field property tools) that the reference UIs spell as buttons but
implement as single parameterized operations.

## Current-Engine Grounding

Two divergent write paths exist; collapsing them is the load-bearing
work.

**(A) Imported-board mutations correctly journal** through Engine +
`TransactionRecord` + undo/redo:
- `crates/engine/src/api/write_ops/basic_mutations.rs` — `delete_track`,
  `delete_via`, `delete_component`, `move_component`, `rotate_component`,
  `set_value`, `set_reference`.
- `crates/engine/src/api/write_ops/assign_package_rule.rs:194/272` —
  `set_net_class`, `set_design_rule`, `set_package`.
- `crates/engine/src/api/write_ops/component_replacements.rs`.
- Undo/redo: `crates/engine/src/api/write_ops/undo_redo/{undo.rs,redo.rs}`.
- Daemon bridges these (verified) in
  `crates/engine-daemon/src/dispatch.rs`: `delete_track`, `delete_via`,
  `delete_component`, `move_component`, `rotate_component`, `set_value`,
  `set_reference`, `set_package`, `set_package_with_part`,
  `set_net_class`, `set_design_rule`, `run_drc`, plus `undo`/`redo`
  (dispatch.rs:272/276).
- MCP catalog: `mcp-server/tools_catalog_data.py`.

**(B) Native board authoring BYPASSES `commit()`/journal** across 112
call sites in 8 files (grep-verified), all via
`load_native_project(root)` → mutate JSON → `write_canonical_json`:
- `command_project_board_layout.rs` (board text, keepout, outline,
  stackup; 9 write sites)
- `command_project_board_routing_net.rs` (`draw_track:91`,
  `place_zone:124`, `place_via:159`; 10 write sites)
- `command_project_board_pad.rs` (5)
- `command_project_board_component_layer.rs` (flip/side via raw field
  write; 2)
- `command_project_board_component_reference.rs` (2)
- `command_project_board_component_value.rs` (2)
- `command_project_board_component_mutations.rs` (locked; 8)
- `command_project_board_netclass_dimension.rs` (dimensions; 6)

This is the private-writer migration defect the readiness audit forbids.

**Board model:** `crates/engine/src/board/board_types.rs` —
`PlacedPackage.layer` (side) at `:17`, `PlacedPackage.locked` at `:18`;
`Track` (`:22`), `Via` (`:32`), `Zone` (`:43`), `Keepout` (`:74`),
`Dimension` (`:82`), `BoardText` (`:92`) types.

**DRC:** `crates/engine/src/drc/mod.rs`; daemon `run_drc` (dispatch.rs:455)
runs `Connectivity, ClearanceCopper, TrackWidth, ViaHole, ViaAnnularRing,
SilkClearance, ProcessAperture`. The rule set is hard-coded in dispatch,
not a CheckProfile parameter (a shared-surface gap).

**Routing kernel:** `crates/engine/src/board/route_surface.rs` plus 60+
`route_path_candidate_*.rs`; proposal export/apply/review in the MCP
catalog. Used here as AI context only.

**NO Operation enum exists** (per-method `TransactionRecord` pattern).
GREP-CONFIRMED ABSENT from the engine/commit layer:
- ZoneFill / `zone_fill` / `fill_zone` — zero hits. No filler, no
  `Filled`/`Unfilled`/`Stale` derived state, no Stale invalidation.
- `flip` / `SetComponentSide` as a journaled op — only the native field
  write in `component_layer.rs`; no pad-geometry mirroring.
- `set_locked` op — only a native write.
- align / distribute — none.
- board-outline / stackup / text / dimension through `commit()` — none.
- CLI undo/redo exposure — engine `undo()`/`redo()` exist and are
  daemon-bridged, but are unexposed in `crates/cli/src/main_project.rs`
  (grep-confirmed empty).

## Tool Inventory

Thirteen surviving tools. Each is the irreducible PCB authoring verb. The
shared operations (session, query, check, proposal, commit, artifact,
journal) are referenced by name from `AI_CLI_MCP_TOOL_SURFACE.md` and not
restated; this inventory adds only the PCB-domain typed Operation variants
and the domain `aiQueryContext`.

All authoring tools here are **direct-commit class** (local, visible,
undoable, no hidden cross-domain/destructive/batch implication) unless a
field says otherwise. Every "Operation it emits" reduces to a typed
`OperationBatch` through the single `commit()` (DatumCommitTool). Every
read named below folds under DatumQueryTool (`datum-eda query pcb.*`); every
`run_drc` folds under DatumCheckTool (`datum-eda check run --domain drc`).

---

### 1. place_component

- **Manual UI action:** pick a footprint from the pool/library palette,
  click the board to drop at cursor, type ref/value inline. Local visible
  undoable → direct commit.
- **Operation emitted:** `PlaceComponent{ component_instance: Uuid,
  package_uuid, position, rotation, side }` → one `commit()` →
  `TransactionRecord`. The join is the `ComponentInstance` Uuid, never the
  reference designator.
- **CLI command:** `datum-eda project board place-component --root <p>
  --component-instance <uuid> --package <uuid> --x <nm> --y <nm>
  [--rotation <deg>] [--side top|bottom]`
- **MCP/AI tool:** `place_component` (daemon-dispatched, same Operation).
- **AI query/context needed:** `model_revision`; available
  `package_uuid`s (DatumQueryTool `library.query` / existing
  `search_pool`/`get_package`); free board area; `ComponentInstance`
  whose derived relationship status is `PendingImplementation` and needs
  placement.
- **Validating checks:** `run_drc` courtyard/clearance after commit;
  resolver relationship status flips `PendingImplementation` →
  `Implemented`.
- **Proof slice:** datum-test — place one of the 43 footprints at a known
  coord; assert a `TransactionRecord` in the journal, a `model_revision`
  bump, undo restores prior state, and board summary shows the new
  `PlacedPackage`.
- **Explicitly not-supported-yet:** pool-to-board materialization through
  `commit()` only partial (`main_tests_project_board_component_pool_
  materialization.rs` exists); the CLI place verb native-writes today and
  must be retired into the typed op.

---

### 2. move_rotate_flip_component

- **Manual UI action:** drag = move; Space/handle = rotate; L/F = flip to
  the opposite side (mirrors pads). Local visible undoable → direct
  commit. Multi-select = one `OperationBatch`.
- **Operation emitted:** `MoveComponent` / `RotateComponent` /
  `SetComponentSide`. Move and rotate already journal
  (`basic_mutations.rs`); `SetComponentSide` is NEW and must mirror pad
  geometry. Multi-select batches as one `OperationBatch`.
- **CLI command:** `datum-eda project board move-component --root <p> --uuid
  <u> --x --y` ; `rotate-component --uuid --rotation` ; `flip-component
  --uuid --side bottom`
- **MCP/AI tool:** `move_component`, `rotate_component` (exist in
  dispatch.rs); `flip_component` (gap).
- **AI query/context needed:** current `PlacedPackage` position /
  rotation / side, locked flag, `model_revision`; overlap/collision
  context.
- **Validating checks:** `run_drc` courtyard/clearance; the
  placement-locked guard rejects move/flip on a locked package.
- **Proof slice:** datum-test — move then undo == identity; rotate
  90/270/0 round-trips pad geometry; flip moves the package to bottom and
  mirrors pad coords, undo restores byte-for-byte.
- **Explicitly not-supported-yet:** flip has no journaled op
  (`command_project_board_component_layer.rs` sets `.layer` via native
  write only); pad-geometry mirroring on flip is unimplemented.

---

### 3. align_distribute

- **Manual UI action:** multi-select components, Align toolbar: left /
  right / top / bottom / hcenter / vcenter / distribute-h / distribute-v.
  Visible undoable bulk geometry → ONE `OperationBatch`, direct commit,
  one undo entry.
- **Operation emitted:** an `OperationBatch` of `MoveComponent` ops
  computed by the engine from the `{mode}` parameter → single `commit()`.
  NOT proposal-first (local, visible, reversible bulk geometry per the
  direct-commit class in `AI_CLI_MCP_TOOL_SURFACE.md`).
- **CLI command:** `datum-eda project board align --root <p> --uuids
  <u1,u2,...> --mode left|right|top|bottom|hcenter|vcenter|distribute-h|
  distribute-v`
- **MCP/AI tool:** `align_components` (gap) — a first-class AI batch tool.
- **AI query/context needed:** selected component positions/bboxes,
  `model_revision`; the agent supplies the set and the mode (the
  AI-leverageable batch input).
- **Validating checks:** `run_drc` after the batch; each resulting move
  respects the locked guard (locked members are skipped and reported in
  the batch result).
- **Proof slice:** datum-test — align 3 footprints left → all share
  min-x; ONE undo reverts all three; a locked member is skipped with a
  finding.
- **Explicitly not-supported-yet:** no engine align/distribute op
  (grep-confirmed). Must ship as ONE mode-parameterized batch, never 8
  tools.

---

### 4. draw_track / delete_track

- **Manual UI action:** interactive route — click a pad, click waypoints,
  double-click to commit the chain; width/layer come from the active net
  class. Local visible undoable → direct commit per completed route as an
  `OperationBatch` of segments (+ vias).
- **Operation emitted:** `AddTrack{ net_id, layer, points[], width }`
  (+ `AddVia` on a layer change) as an `OperationBatch` → `commit()`.
  `DeleteTrack` already journals (`basic_mutations.rs`). `NetId` resolves
  via resolver connectivity, never by string.
- **CLI command:** `datum-eda project board draw-track --root <p> --net
  <netid> --layer <l> --width <nm> --points x1,y1:x2,y2:...` ;
  `delete-track --uuid <u>`
- **MCP/AI tool:** `delete_track` (exists); add/draw-track op (gap —
  native write today). AI typically uses the route-proposal kernel rather
  than freehand authoring.
- **AI query/context needed:** unrouted ratsnest (existing
  `get_unrouted` under DatumQueryTool `pcb.query`), pad anchor coords,
  active net-class width/clearance, layer stack, obstacles.
- **Validating checks:** `run_drc` `ClearanceCopper` + `TrackWidth`; net
  continuity vs ratsnest.
- **Proof slice:** datum-test — draw one track between two pads →
  `get_unrouted` drops by one; clearance DRC clean; `delete_track` + undo
  round-trips; the journal records the batch.
- **Explicitly not-supported-yet:** `add_track` has no `commit()` op
  (`routing_net.rs:91` native write). Push-and-shove and
  interactive-width-from-rules are out of scope (the kernel covers
  automated routing). Arcs are modeled as a per-segment attribute of
  `points[]`, not a separate primitive.

---

### 5. place_via / delete_via

- **Manual UI action:** V mid-route or standalone; padstack/drill from
  rules. Local undoable → direct commit (usually inside a track
  `OperationBatch`).
- **Operation emitted:** `AddVia{ net_id, position, from_layer, to_layer,
  diameter, drill }` → `commit()`. `DeleteVia` already journals
  (`basic_mutations.rs:36`).
- **CLI command:** `datum-eda project board place-via --root <p> --net
  <netid> --x --y --diameter <nm> --drill <nm> [--from-layer --to-layer]`
  ; `delete-via --uuid`
- **MCP/AI tool:** `delete_via` (exists); `place_via` op (gap — native
  write today).
- **AI query/context needed:** layer-transition need, via-rule limits
  (annular/drill), keepout/clearance context, `model_revision`.
- **Validating checks:** `run_drc` `ViaAnnularRing`, `ViaHole`,
  `ClearanceCopper`.
- **Proof slice:** datum-test — place a via on a net, annular/hole DRC
  pass; delete + undo round-trips; via count in board summary updates.
- **Explicitly not-supported-yet:** `add_via` has no `commit()` op
  (`routing_net.rs:159` native write). Blind/buried classification beyond
  the from/to layer pair is not modeled.

---

### 6. author_zone / delete_zone

- **Manual UI action:** draw a polygon pour boundary on a layer, set net
  + priority + clearance. Authors ONLY the boundary, NOT copper. Local
  undoable → direct commit.
- **Operation emitted:** `AddZone{ net_id, layer, polygon, priority,
  clearance }` → `commit()`. `Zone.polygon` is the authored boundary;
  `ZoneFill` is derived and is NOT created here.
- **CLI command:** `datum-eda project board place-zone --root <p> --net
  <netid> --layer <l> --priority <n> --polygon x1,y1:...` ; `delete-zone
  --uuid`
- **MCP/AI tool:** `author_zone` / `delete_zone` op (gap — native write
  today).
- **AI query/context needed:** net to pour, layer, board outline,
  keepouts, priority vs other zones, `model_revision`.
- **Validating checks:** boundary closed / simple / on-board; DRC and
  export act on FILLED copper only, never the boundary (ZoneFill
  honesty).
- **Proof slice:** datum-test — author a GND zone boundary → board shows
  a `Zone` with `ZoneFill{Unfilled}`; export emits NO copper plus a hard
  finding.
- **Explicitly not-supported-yet:** `commit()` op missing
  (`routing_net.rs:124` native write). `ZoneFill` derived state does not
  exist in the engine (grep-confirmed: no fill geometry, no
  `Filled`/`Unfilled`/`Stale`).

---

### 7. fill_zones

- **Manual UI action:** explicit Fill All Zones / Repour (KiCad B, Altium
  Shift+B). Recomputes derived copper from boundaries. NOT an authored
  mutation — it refreshes revision-keyed derived state.
- **Operation emitted:** NONE on source. Triggers `ZoneFill`
  regeneration keyed by `model_revision` → `ZoneFill{ Filled, islands,
  provenance }`. Derived, never journaled as authored copper. Scope
  (all / zone / net) is a filter PARAMETER, not separate tools.
- **CLI command:** `datum-eda project board fill-zones --root <p> [--zone
  <uuid>] [--net <netid>]`
- **MCP/AI tool:** `fill_zones` (gap) — a first-class AI derived-state
  tool; AI checks `ZoneFill` state before relying on copper.
- **AI query/context needed:** `model_revision` (to detect `Stale`), zone
  boundaries, obstacles, clearances.
- **Validating checks:** post-fill DRC on the resulting copper;
  island/thermal reporting; `Stale` flag if `model_revision` advanced
  since the last fill.
- **Proof slice:** datum-test — author a zone (`Unfilled`) → `fill-zones`
  → `ZoneFill{Filled}` with islands; move a component → `ZoneFill` marked
  `Stale` (not silently wrong); export uses only `Filled` copper.
- **Explicitly not-supported-yet:** the entire `ZoneFill` subsystem is
  unimplemented (no filler, no derived-state model, no `Stale`
  invalidation). This is the largest single gap in the zones domain.

---

### 8. author_keepout

- **Manual UI action:** draw a keepout/rule-area polygon, choose layers +
  kind (route / via / copper / component). Local undoable → direct
  commit. Edit/delete by uuid.
- **Operation emitted:** `AddKeepout{ polygon, layers[], kind }` →
  `commit()`. The `Keepout` type exists (`board_types.rs:74`). Kept
  distinct from `author_zone`: a keepout has no net and no fill state, so
  merging would corrupt the ZoneFill honesty boundary that `fill_zones`
  depends on.
- **CLI command:** `datum-eda project board place-keepout --root <p> --kind
  route|via|copper --layers <l1,l2> --polygon x1,y1:...` ; `edit-keepout`
  ; `delete-keepout`
- **MCP/AI tool:** `author_keepout` / `edit` / `delete` (gap — native
  write today).
- **AI query/context needed:** region to protect, affected layers, kind
  semantics, `model_revision`.
- **Validating checks:** DRC honors the keepout as an obstacle for
  matching-kind tracks/vias/copper; the route kernel treats it as a
  blockage (`route_segment_blockage.rs`).
- **Proof slice:** datum-test — place a route-keepout, draw a track
  through it → DRC/route finding; undo removes the keepout.
- **Explicitly not-supported-yet:** `commit()` op missing
  (`layout.rs:429` native write). Keepout enforcement in interactive DRC
  vs only the route kernel needs unification.

---

### 9. set_board_outline

- **Manual UI action:** define/redefine the board edge polygon
  (Edge.Cuts). Visible undoable → direct commit; large redefinitions may
  warrant confirmation.
- **Operation emitted:** `SetBoardOutline{ polygon }` → `commit()`.
  Replaces the authored outline; the previous outline is captured for
  undo.
- **CLI command:** `datum-eda project board set-outline --root <p> --polygon
  x1,y1:x2,y2:...`
- **MCP/AI tool:** `set_board_outline` (gap — native write today).
- **AI query/context needed:** current outline, component extents that
  must fit, mounting/keepout context, `model_revision`.
- **Validating checks:** outline closed / simple; components/zones
  outside the outline → findings; board-edge clearance DRC.
- **Proof slice:** datum-test — read the current outline, set a larger
  rectangle, undo restores the original polygon byte-for-byte; an
  off-board component is flagged.
- **Explicitly not-supported-yet:** `commit()` op missing (`layout.rs`
  native write). Cutouts / multiple-hole outline representation is
  minimal.

---

### 10. edit_stackup

- **Manual UI action:** Layer Stack Manager — layer count,
  copper/dielectric thickness, material, layer names. Visible config edit
  → direct commit; impedance targets deferred.
- **Operation emitted:** `SetStackup{ layers[] }` → `commit()`. The
  `StackupLayer` type exists
  (`command_exec_board_stackup.rs` / `stackup.rs`).
- **CLI command:** `datum-eda project board set-stackup --root <p> --layers
  <json|spec>` (plus an `add-default-top-stackup` convenience).
- **MCP/AI tool:** `edit_stackup` (gap — native write today).
- **AI query/context needed:** current stackup, layer count needed for
  routing, `model_revision`; impedance is a deferred stub
  (`ImpedanceSpec`).
- **Validating checks:** track/via/zone layer references resolve to
  existing stackup layers; via from/to layers valid.
- **Proof slice:** datum-test — set a 4-layer stackup, confirm via
  from/to-layer ops can target inner layers; undo restores 2-layer;
  orphan-layer references flagged.
- **Explicitly not-supported-yet:** `commit()` op missing (`layout.rs`
  native write). The controlled-impedance solver is deferred (Standards
  Audit stub only).

---

### 11. set_design_rules / set_net_class

- **Manual UI action:** Board Setup rules editor — clearance, track
  width, via sizes, annular; assign nets to net classes. Config edit →
  direct commit; broad-scope changes may preview.
- **Operation emitted:** `SetDesignRule` / `SetNetClass` → `commit()`.
  Both already journal (`assign_package_rule.rs:194/272`).
- **CLI command:** `datum-eda project board set-design-rule ...` ;
  `set-net-class --net <uuid> --class <name> --clearance --track-width`
  (exists).
- **MCP/AI tool:** `set_net_class` (exists, dispatch.rs:243);
  `set_design_rule` (engine op + daemon dispatch exist at
  dispatch.rs:259; verify MCP catalog exposure).
- **AI query/context needed:** current rules/net classes, the `RuleType`
  taxonomy (`Connectivity`, `ClearanceCopper`, `TrackWidth`, `ViaHole`,
  `ViaAnnularRing`, `SilkClearance`, `ProcessAperture`), nets needing
  constraints.
- **Validating checks:** rule changes re-trigger `run_drc`; net-class
  membership consistent with connectivity.
- **Proof slice:** datum-test — set a tighter clearance → `run_drc`
  surfaces a new `ClearanceCopper` violation (clean before); undo
  restores the rule and clears findings.
- **Explicitly not-supported-yet:** diff-pair / length-match / impedance
  rule classes not modeled. Altium-style rule-query scoping absent; scope
  is per-net / per-class only.

---

### 12. inspect_edit

- **Manual UI action:** select object(s) (filter by type), open the
  Inspector, edit shared properties (width, net, layer, side, value,
  reference, LOCKED) across the selection. Each commit goes through the
  typed op for that property.
- **Operation emitted:** Selection is NOT an Operation (consumer state,
  per the spine — never journaled; exposed read-only via
  DatumQueryTool `selection.describe`). Edits emit the relevant typed op:
  `SetValue` / `SetReference` (exist), `SetPadNet`, `SetComponentLocked`,
  track-width/layer edit. Multi-object edit = one `OperationBatch` (one
  undo entry). `SetComponentLocked` is FOLDED HERE as a boolean property,
  not its own tool.
- **CLI command:** `datum-eda project board inspect --root <p> --uuid <u>`
  (read); property edits use the specific verb: `set-value`,
  `set-reference`, `set-pad-net`, `set-locked`, `edit-pad`, `edit-track`.
- **MCP/AI tool:** `get_board_summary` / `get_components` (read, exist,
  fold under DatumQueryTool); `set_value` / `set_reference` (write,
  exist); `set_locked` + multi-object batch (gap).
- **AI query/context needed:** object properties at `model_revision`;
  which fields are authored vs derived (refdes display vs
  `ComponentInstance` identity).
- **Validating checks:** type-appropriate DRC after each property commit;
  reference uniqueness; net exists; the locked flag guards subsequent
  move/flip/route.
- **Proof slice:** datum-test — read footprint props, `set_value` +
  `set_reference`; toggle locked then attempt move → rejected; batch set
  net on two pads → ONE undo entry; a refdes change leaves the
  `ComponentInstance` Uuid unchanged.
- **Explicitly not-supported-yet:** no unified Inspector/selection model
  in the engine; per-property native-CLI writes (`pad.rs`,
  `component_mutations.rs` locked, `component_value/reference.rs`) bypass
  `commit()`. Multi-object batch edit not wired as a single
  `OperationBatch`. Board-text/dimension graphic-primitive authoring
  (`place_native_project_board_text` in `layout.rs:68`, dimensions in
  `netclass_dimension.rs`) also native-write today and should fold into a
  future place-graphic op rather than spawn dedicated tools.

---

### 13. run_drc

- **Manual UI action:** Design > Run DRC (batch) + online DRC while
  editing. A read-only check pass → `CheckRun` / `CheckFinding`, never an
  authored mutation.
- **Operation emitted:** NONE (read/derived). Produces a revision-keyed
  `CheckRun{ model_revision, findings[] }`. This folds under DatumCheckTool
  (`datum-eda check run --domain drc`); `run_drc` survives only as a thin
  compatibility alias. Repairs are proposal-first via DatumProposalTool,
  never silent edits.
- **CLI command:** `datum-eda project board drc --root <p> [--rules
  clearance_copper,track_width,...]` (alias of `datum-eda check run --domain
  drc`).
- **MCP/AI tool:** `run_drc` / `drc` (exist, dispatch.rs:455).
- **AI query/context needed:** active `RuleType`s, `model_revision` (to
  key findings), object IDs per finding so AI targets repairs as
  proposals.
- **Validating checks:** self-validating; the 0% FP/FN gate against the
  corpus; findings carry affected object IDs + rule basis.
- **Proof slice:** datum-test — `run_drc` on the imported board, finding
  count/identity stable across runs at the same `model_revision`;
  introduce a clearance violation via move, re-run, exactly one new
  finding keyed to the moved object.
- **Explicitly not-supported-yet:** zone-copper DRC blocked on missing
  `ZoneFill` (cannot check pour copper honestly). Proposal-based
  auto-repair not wired. The DRC rule set is hard-coded in dispatch.rs:455
  instead of a `CheckProfile` parameter (a shared-surface gap).

## Minimal-Set Recommendation

Thirteen surviving tools, down from a 17-across-13 framing, by folding
`set_component_locked` into `inspect_edit` as a typed boolean property.
The set maps 1:1 onto the irreducible PCB authoring verbs:

1. `place_component`
2. `move_rotate_flip_component`
3. `align_distribute` (one mode-parameterized batch)
4. `draw_track` / `delete_track`
5. `place_via` / `delete_via`
6. `author_zone` / `delete_zone` (boundary only)
7. `fill_zones` (separate explicit derived refresh)
8. `author_keepout`
9. `set_board_outline`
10. `edit_stackup`
11. `set_design_rules` / `set_net_class`
12. `inspect_edit` (property edits incl. locked; selection read-only)
13. `run_drc` (alias of DatumCheckTool drc domain)

The single highest-leverage requirement is NOT a new tool: it is
collapsing the two write paths into ONE `commit()`/journal primitive.
Imported-board ops journal correctly, but 112 native-write call sites
across 8 CLI files write board JSON directly via `load_native_project` +
`write_canonical_json` — the private-writer migration defect the audit
forbids (audit lines 60-61, 129-132, 344-346). Retiring those writers so
each CLI verb emits a typed Operation is the load-bearing work; the tool
names mostly already exist as CLI verbs.

AI leverage concentrates in three batch/derived tools that deserve
first-class MCP exposure: `align_distribute` (one batch op, not eight),
`fill_zones` (derived, revision-keyed, scope as a filter param), and
`run_drc` (findings drive proposals). The rest are mechanical
UI-gesture-to-op mappings.

`author_zone` and `author_keepout` are deliberately kept separate (not
merged) because a keepout has no net and no fill state, so merging would
corrupt the ZoneFill honesty boundary that `fill_zones` depends on.

## Omitted / Redundant Tools

A redundant tool is a defect. The following were considered and cut, each
justified against real-world practice.

- **`set_component_locked` (standalone tool)** — FOLDED INTO
  `inspect_edit`. Locked is a boolean property of `PlacedPackage`
  (`board_types.rs:18`), identical in kind to value/reference. Altium and
  KiCad both set lock via the inspector/properties checkbox, never a
  dedicated verb. The typed op `SetComponentLocked` still exists as the
  mechanism; it just is not its own tool. A tool per editable boolean is
  catalog bloat.

- **`align-left` / `align-right` / `align-top` / `align-bottom` /
  `distribute-h` / `distribute-v` (6 separate tools)** — one
  `align_distribute(mode=...)` op emitting one `OperationBatch`. Altium
  ships toolbar buttons but they are one operation with a mode parameter;
  six tools would be six ways to spell one batch op.

- **Separate `add-track-segment` / `add-track-arc` / `add-track-chain`**
  — one `draw_track` op taking a `points[]` polyline with an optional
  per-segment arc flag. KiCad/Altium model arcs as a track attribute, not
  a distinct primitive.

- **`repour-selected` / `repour-all` / `repour-net` (3 tools)** — one
  `fill_zones` with optional `--zone`/`--net` filter. Fill is
  derived-state regeneration keyed by `model_revision`; scope is a
  parameter, not a distinct tool.

- **`set-track-width` / `set-track-net` / `set-track-layer` /
  `set-via-size` as standalone top-level tools** — property edits
  surfaced through `inspect_edit`, reducing to typed property ops
  (`edit-track`, `edit-pad`, `set-pad-net`). The Inspector is the
  surface, the typed op is the mechanism; one tool per editable field is
  bloat.

- **standalone `select` / `deselect` / `selection-filter`** — per the
  spine, selection is consumer-side interactive state, never an Operation
  and never journaled. Exposed only via DatumQueryTool
  `selection.describe`, never as a mutation tool.

- **dedicated PCB-domain undo / redo tools** — undo/redo are global
  journal-cursor operations (engine `undo()`/`redo()` already exist,
  unexposed in CLI). They belong to DatumJournalTool, surfaced once
  project-wide, not per domain.

- **`route-differential-pair` interactive authoring tool** — for the
  first slice, diff-pair is a net-class attribute plus the existing
  route-proposal kernel, not a distinct hand-route tool. Flagged
  not-yet-supported rather than given a premature dedicated tool.

- **`place_board_text` / `place_dimension` as dedicated authoring
  tools** — both native-write today (`layout.rs:68` board text;
  `netclass_dimension.rs` dimensions) but are low-leverage
  silkscreen/documentation graphic primitives. Rather than expand the
  catalog with two tools, they should fold into a single future
  `place-graphic-primitive` op once the `commit()` migration lands (noted
  under `inspect_edit`'s not-yet-supported). Adding them now is premature
  catalog growth against the lean mandate.

## Shared Surface

This contract does not restate the seven shared operations. It references
them from `docs/contracts/AI_CLI_MCP_TOOL_SURFACE.md` by name:

- **DatumToolSession + DatumContextEnvelope** — every PCB tool invocation
  opens a project-scoped capability envelope carrying `session_id`,
  `actor_type`, `project_id`/`project_root`, `model_revision` read at
  creation, capabilities, and provenance seed. No PCB tool mints its own
  session. A tool refreshes context before propose/apply if its envelope
  `model_revision` is stale.
- **DatumQueryTool** — all PCB reads (`get_board_summary`,
  `get_components`, `get_unrouted`, `get_net_info`, `get_design_rules`,
  `selection.describe`) fold under `datum-eda query pcb.*` as a family
  parameter, not new tools. Cross-domain joins key on `ComponentInstance`,
  never refdes.
- **DatumCheckTool** — `run_drc` folds under `datum-eda check run --domain
  drc`; the DRC rule set becomes a `CheckProfile` parameter.
  `CheckFinding`s are revision/fingerprint-keyed, with explanation and
  `suggested_next_action` as fields.
- **DatumProposalTool** — high-risk/cross-domain/batch/destructive/
  AI-originated PCB mutations (board-bound deletes, bulk operations,
  AI-originated authoring) flow through one proposal lifecycle. Repair
  generators reuse the typed ops above, they are not new mutation
  primitives.
- **DatumCommitTool** — THE only mutation gateway. Every typed
  `OperationBatch` above flows through the single `commit()`. Any
  per-domain board save/write is a FORBIDDEN private path (the 112
  native-write sites are the migration defect to retire). Object removal
  reduces to the shared `DeleteObjects{ids:[ObjectId]}` op (object type
  recoverable from the id); the `delete_track`/`delete_via`/`delete_zone`/
  `delete-keepout` verbs paired with their authoring tools above are the
  existing engine-named ops and the UI gesture spellings of that one
  shared delete — not a PCB-private delete contract. Board-bound / bulk
  deletes escalate to a Proposal.
- **DatumArtifactTool** — manufacturing projection/export is owned there
  (Gerber/drill/BOM/PnP), never by a PCB authoring tool. ZoneFill honesty
  is enforced at the projection boundary: only `ZoneFill{Filled}`
  contributes copper.
- **DatumJournalTool** — global `datum-eda journal list|show|undo|redo`. PCB
  ships no per-domain undo/redo; removal is undo of the originating
  transaction.

## Proof Slice & Fixture

**Fixture:** the datum-test KiCad project at
`/home/bfadmin/Documents/kicad_projects/Datum-eda/datum-test/datum-test.kicad_pcb`
(43 footprints, largely unrouted — ideal for place/move/flip/route/via/
zone authoring and unrouted-count regression). This is the canonical M7
regression fixture per user memory.

- Use it **imported** (Import Map `import_key` identity) for
  move/rotate/flip/delete/route/DRC proof slices, and as the seed board
  for authoring tracks/vias/zones/keepouts.
- For greenfield authoring ops needing a native (non-imported) project,
  **request a small native board project from the owner** rather than
  fabricating one, per the no-synthetic-fixture rule.

**First proof slice (load-bearing):** retire one native-write CLI verb
(recommend `draw-track`, `routing_net.rs:91`) into a typed `AddTrack`
Operation through `commit()`, and prove: a `TransactionRecord` appears in
the journal, `model_revision` bumps, `datum-eda journal undo` restores byte
state, and the imported and native write paths now share one commit
gateway. This single retirement is the template for the remaining 111
sites.

## Not-Yet-Supported

- The `ZoneFill` subsystem is entirely unimplemented — no filler, no
  `Filled`/`Unfilled`/`Stale` derived state, no `Stale` invalidation.
  Largest single gap. `author_zone` ships boundary-only with a hard
  no-copper finding; `fill_zones` is a distinct later slice.
- Component flip / change-side has no journaled `SetComponentSide` op and
  no pad-geometry mirroring; only a native `.layer` field write exists.
- Engine `align`/`distribute` op does not exist.
- `add_track`, `add_via`, `author_zone`, `author_keepout`,
  `set_board_outline`, `edit_stackup`, board-text, and dimensions all
  lack `commit()` ops (native write today).
- Diff-pair and length-matched routing are deferred (net-class attribute
  + kernel), not given dedicated authoring tools.
- Altium-style rule-query scoping is absent; scope is per-net/per-class
  only.
- CLI undo/redo is unexposed (`main_project.rs`); engine `undo()`/`redo()`
  exist and are daemon-bridged.
- Push-and-shove and interactive-width-from-rules are out of scope (the
  route-proposal kernel covers automated routing).
- Blind/buried via classification beyond from/to layer pair, board
  cutouts/multi-hole outlines, and the controlled-impedance solver are
  deferred (Standards Audit stub only).
- Board-text and dimension authoring are deferred into a future
  `place-graphic-primitive` op.

## Open Owner Questions

1. Native board authoring (track/via/zone/keepout/outline/stackup/pad/
   component-layer/value/reference/locked/text/dimension) writes board
   JSON directly across 112 call sites in 8 CLI files, bypassing
   `commit()`/journal. Confirm retiring these private writers into typed
   Engine operations is the FIRST PCB-domain implementation slice,
   blocking everything downstream.
2. `ZoneFill` derived state does not exist in the engine at all
   (grep-confirmed). Ship `author_zone` boundary-only first (with a hard
   no-copper finding) and `fill_zones` as a distinct later slice, or one
   deliverable?
3. Component flip/change-side: today only
   `command_project_board_component_layer.rs` native-writes the `.layer`
   field; there is no journaled `SetComponentSide` op and no pad-geometry
   mirroring. Confirm flip is in the first manual-authoring slice (given
   datum-test components may need bottom-side placement), and confirm pad
   mirroring is in scope.
4. Confirm `align_distribute` is ONE mode-parameterized batch op
   (recommended) rather than discrete align tools, and confirm
   direct-commit (visible/undoable) over proposal-first for bulk geometry
   moves.
5. Confirm `set_component_locked` should be a property edit under
   `inspect_edit` (recommended) rather than a standalone tool.
6. Should interactive freehand track drawing be in the first slice at
   all, or should hand-routing defer to the route-proposal kernel while
   only `delete_track` + apply-proposal are exposed early?
7. Confirm diff-pair and length-matched routing are deferred (net-class
   attribute + kernel) and not given dedicated authoring tools in the
   first matrix.
8. Confirm CLI undo/redo should be a global project-level command (under
   DatumJournalTool), not per-PCB-domain.
9. Board-text and dimension authoring exist as native-write CLI verbs but
   are omitted from the surviving matrix as low-leverage graphic
   primitives slated to fold into a future place-graphic op. Confirm this
   deferral, or are silkscreen text/dimensions required in the first
   authoring slice?
10. Altium-style rule-query scoping vs current per-net/per-class only — is
    scoped-rule expressiveness in scope for the first rules slice or
    deferred?
