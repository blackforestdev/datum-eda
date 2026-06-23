# Datum EDA GUI — Information Architecture (Area Spec)

Status: draft GUI area specification, 2026-06-22, benchmarked to
commercial EDA. Controlling for its domain. Conforms to
`specs/GUI_SPEC.md` (the master) and does not relax its bar, thesis,
architecture constraints, or buildability standard.

Area: information-architecture — the dockable/floating panel system, the
context-sensitive Properties/Inspector panel, the domain panels (design
tree, Components, Nets, Rules, Output/OutJob, Violations/DRC,
Navigator), and saved per-task WorkbenchProfiles. This spec defines the
panel TAXONOMY, the DOCKING MODEL, and WHICH PANEL OWNS WHAT.

Driven by:
- `specs/GUI_SPEC.md` (master: bar/thesis/constraints/buildability)
- `docs/decisions/PRODUCT_MECHANICS_000B_VIEW_COMPOSITION_AND_LIVE_PRODUCTION.md`
  (`ViewTab`/`Pane`/`LayoutGraph`/`WorkbenchProfile`/`PinnedContext`)
- `docs/decisions/PRODUCT_MECHANICS_007_PROJECT_WORKSPACE_MODEL.md`
  (`Project`/`Workspace`/`WorkspaceState`/`SessionState`/`NavigationStack`)
- `docs/decisions/PRODUCT_MECHANICS_002_MANUAL_EDITOR_BASELINE.md`
  (single `commit()`, `QG-DIRECT-EDITING-FEEL`)
- `docs/decisions/PRODUCT_MECHANICS_013_GUI_SUPERVISION_AND_PARITY.md`
  (supervision-reflection first)
- `docs/contracts/PCB_LAYOUT_TOOL_CONTRACT.md`,
  `docs/contracts/RULES_CHECKS_TOOL_CONTRACT.md`,
  `docs/contracts/SCHEMATIC_AUTHORING_TOOL_CONTRACT.md`,
  `docs/contracts/MANUFACTURING_OUTPUT_TOOL_CONTRACT.md`,
  `docs/contracts/AI_CLI_MCP_TOOL_SURFACE.md` (the data each panel surfaces;
  the seven shared operations — referenced, never restated)
- Existing shell substrate: `crates/gui-render` `ShellLayout`
  (`left_sidebar`/`right_sidebar`/`bottom_strip`), `HitTarget`,
  `crates/gui-protocol` `WorkspaceUiState`/`DockTab`/`WorkspaceFilterState`,
  the §2.5 identity triple on every renderable
  (`object_id`/`object_kind`/`source_object_uuid`)

---

## 1. Purpose

A commercial EDA tool is, structurally, a docking host wrapped around one
authority model. The canvas is one surface; the value of the application
is in the PANELS around it and how the user composes them per task. This
spec defines that frame for Datum:

1. The panel taxonomy — every panel kind, what authority it reflects or
   edits, and its single owner.
2. The docking model — how panels are placed, docked, floated, tabbed,
   pinned, and persisted, as a typed `LayoutGraph` that is consumer state
   and never design authority (007).
3. The Inspector contract — the one context-sensitive Properties panel
   that edits whatever is selected, matching the Altium Properties panel,
   and the validate-before-commit field path that makes every edit a typed
   `Operation` through the single `commit()` (002).

This area is the SHELL the other five area specs hang their surfaces on.
Where a domain owns the CONTENT of a panel (e.g. the Rules content is
owned by `GUI_LIVE_FEEDBACK_AND_RULES.md`/the rules domain), this spec owns the
panel's PLACEMENT, DOCKING, OWNERSHIP, and PERSISTENCE. The split is
stated per panel in §3.

---

## 2. The Bar (this area)

Benchmark, per the master, against the best commercial tools. For
information architecture the named references are:

- Altium Designer — the gold standard for this area. The single
  selection-driven **Properties panel** ("one panel edits whatever is
  selected"), the **PCB/SCH/Components/Nets/Navigator/Messages panels**,
  the **Panels button** docking menu, dockable/floating/auto-hide
  ("popout") panel states, and **saved Workspace/View configurations**.
  Datum's Inspector and docking model are measured against this directly.
- Cadence Allegro / OrCAD — the **Constraint Manager** as a separate
  spreadsheet-class domain window, the **Find/Filter** object panel, and
  **Visibility (Color/Visibility)** control. Datum's Rules panel and
  layer/filter visibility are measured against this.
- Siemens Xpedition — **Display Control** (layer/object visibility as a
  first-class docked panel) and dockable batch/interactive **DRC results**
  navigation. Datum's Layers panel and Violations panel are measured
  against this.
- Zuken CR-8000 — concurrent multi-board **navigator/design-tree** over a
  multi-board project. Datum's Design Tree is measured against this for
  multi-board/multi-sheet projects (000E).

KiCad is a FLOOR: KiCad's fixed left/right panes and modal DRC dialog are
the minimum we exceed. We do not target them. Every panel in §3 cites the
commercial pattern it matches or beats.

The "good enough?" test for this area: a professional opening Datum should
be able to compose a layout/review/manufacturing workspace, dock the
panels they need, select an object and edit it in one Inspector, and save
that arrangement as a profile — without it feeling worse than the tool
they pay $10k/seat for.

---

## 3. Panel Taxonomy

Every panel has: a `PanelKind`, an AUTHORITY CLASS (what it reflects/edits),
an OWNER (the single spec/domain that defines its content), a default
DOCK REGION, and a SUPERVISION-FIRST posture (read-only reflection ships
before any edit affordance, per 013). This area spec owns the row's
placement/docking/persistence columns; the OWNER column owns the content.

Authority classes:
- `Reflect` — read-only view of committed `DesignModel`/derived state. No
  commit path. (Decision 013 supervision surface.)
- `Edit` — has fields that produce typed `Operation`s through `commit()`.
- `Compose` — manipulates consumer state only (`LayoutGraph`, selection,
  visibility, camera); never journaled, never advances `model_revision`
  (007).
- `Navigate` — changes selection / `NavigationStack` / active tab; consumer
  state.

### 3.1 The Panel Catalogue

| PanelKind | Authority | Owner (content) | Default region | First posture (013) | Commercial benchmark |
|---|---|---|---|---|---|
| `Inspector` (Properties) | Edit (+Reflect) | this spec §4 | right, top | Reflect-only | Altium Properties panel |
| `DesignTree` (Project/Navigator structure) | Navigate + Reflect | this spec §3.3 | left, top | Reflect | Altium PCB/Projects + CR-8000 multi-board nav |
| `Components` | Navigate + Reflect | PCB/SCH contracts | left, tab w/ tree | Reflect | Altium Components/PCB list |
| `Nets` | Navigate + Reflect | PCB layout contract | left, tab | Reflect | Altium Nets / Allegro net list |
| `Layers` (Display Control) | Compose | this spec §3.4 | right, bottom | Active (compose is not authority) | Xpedition Display Control / Altium View Config |
| `Rules` (Constraint surface) | Edit (+Reflect) | rules domain / `GUI_LIVE_FEEDBACK_AND_RULES.md` | floating / right tab | Reflect | Allegro Constraint Manager |
| `Violations` (Checks/DRC/ERC findings) | Navigate + Reflect | rules/checks contract | bottom dock tab | Reflect | Altium Messages / Xpedition DRC results |
| `Output` (OutJob/Production) | Reflect (+Navigate) | manufacturing contract / 000B | bottom dock tab (`Outputs`) | Reflect | Altium OutputJob / Output panel |
| `Terminal` | Compose (PTY) + indirect-Edit | embedded-terminal (005) | bottom dock tab | Active (exists) | (Datum wedge; no commercial equivalent) |
| `Assistant` | Navigate + Propose | assistant surface (006) | bottom dock tab | Active (exists) | (Datum wedge) |
| `ArtifactPreview` | Reflect | 000B live-CAM | tab / PiP | Reflect | Altium CAMtastic / Gerber preview |
| `NavigatorOverview` (whole-board minimap) | Navigate | this spec §3.5 | floating overlay | Active | Altium Navigator overview |

`Terminal`, `Assistant`, `Outputs` already exist as `DockTab` variants in
`crates/gui-protocol` `WorkspaceUiState`; this spec extends the dock tab
set, it does not replace it.

### 3.2 The OWNERSHIP rule (which panel owns what)

The single hardest IA failure in EDA tools is two panels claiming to own
the same edit. Datum forbids it by construction:

1. **Exactly one Edit panel per object class is canonical.** The
   `Inspector` is the canonical editor for per-object properties of ANY
   selected object. The `Rules` panel is the canonical editor for
   constraint/net-class rows. No other panel exposes an edit field for the
   same property. A domain panel (Components/Nets) may show a property
   read-only and offer "edit in Inspector" (selects the object + focuses
   Inspector) — it never duplicates the editable field.
2. **Reflect panels never write.** Authority class `Reflect`/`Navigate`/
   `Compose` panels have no `commit()` path. A Navigate action changes
   selection or the `NavigationStack`; it does not mutate the model.
3. **Compose panels touch only the `LayoutGraph`/visibility/camera.** The
   `Layers` panel and the docking host mutate consumer state only;
   `WorkspaceState` save/restore does not advance `model_revision` (007).
4. **Every edit, from any panel, is the same typed `Operation` through the
   same `commit()`.** A field changed in the Inspector and the identical
   change made via CLI/MCP produce the same `OperationBatch` and the same
   journal record (002, AI_CLI_MCP_TOOL_SURFACE). The panel is a producer
   of typed ops, never a private write path. This is the deterministic
   wedge applied to IA.

### 3.3 DesignTree (project / navigator structure)

Authority: `Navigate` + `Reflect`. Owner of content: this spec (it is the
structural index of the resolved `DesignModel`, not a domain). Reflects
the `ProjectManifest`-resolved structure (007 §ProjectManifest): project →
boards/sheets (000E) → variant → manufacturing plans/output jobs →
artifact registry. Multi-board/multi-sheet projects show the full tree
(CR-8000 multi-board nav is the bar; KiCad's single-board tree is the
floor). Every node carries the identity triple; clicking a node is a
`Navigate` action that sets selection and pushes a `NavigationStack` entry.
Recovery/diagnostic mode (007: split/incoherent project) renders here as a
distinct badged subtree, never as accepted truth (`QG-RESOLVER-RECOVERY`).

### 3.4 Layers (Display Control)

Authority: `Compose`. Backed by the existing `WorkspaceFilterState`
(`show_authored`/`show_proposed`/`show_unrouted`/`dim_unrelated`/
`layer_visibility`) in `crates/gui-protocol`. This spec promotes that
state from ad-hoc shell toggles to a first-class docked panel matching
Xpedition Display Control / Altium View Configuration. It changes
visibility/dimming only; it never edits geometry, never commits, and never
advances `model_revision`. The honesty rule (000B `QG-ZONEFILL-HONESTY`)
governs how zone-fill state is shown but the layer panel only toggles
visibility, not fill state.

### 3.5 NavigatorOverview

Authority: `Navigate`. A whole-board minimap (Altium Navigator overview)
that frames the canvas camera. Pure consumer state: clicking re-frames the
camera; it never mutates the model. Deferred behind core panels per master
Non-Goals but specified here for completeness of the catalogue.

---

## 4. The Inspector (context-sensitive Properties panel)

The Inspector is the marquee table-stakes surface of this area and the
direct Altium-Properties-panel benchmark: **one panel that edits whatever
is selected.** It is the canonical per-object editor (§3.2 rule 1).

### 4.1 Context rule (content per selection class)

The Inspector content is a pure function of the current selection class.
Selection is consumer state (002/000B) supplied by the canvas/tree/list
panels; the Inspector never owns selection, it reacts to it.

**Op vocabulary rule (no invented ops).** Every field in the table below
maps to a typed `Operation` that EXISTS in
`docs/contracts/PCB_LAYOUT_TOOL_CONTRACT.md` /
`SCHEMATIC_AUTHORING_TOOL_CONTRACT.md` or to the generic field-setter
`SetObjectField` (the contract's per-field setter keyed by the §2.5
`object_id` + a typed field path). The Inspector NEVER invents an op name;
geometry edits (vertex drag, boundary edit, free move) are produced by the
canvas tool grammar (`GUI_INTERACTION_GRAMMAR.md`), not by the Inspector —
the Inspector emits property-set ops, the canvas emits move/geometry ops.
Where a specific op exists it is used; otherwise the field routes through
`SetObjectField`. This is the contract-parity wedge: the same op the CLI/MCP
emits for that field (002, AI_CLI_MCP_TOOL_SURFACE).

| Selection class | Inspector content (fields) | Typed op per field (contract-verified) |
|---|---|---|
| none | empty / "no selection" placeholder | — |
| board `Pad` | layer (ro for SMD), size, shape, pad-type (ro), net | net → `SetPadNet`; size/shape → `SetObjectField`; position not editable here (canvas move op) |
| `Track` segment | layer, width, net, endpoints (ro in Inspector) | width/layer → `SetObjectField`; net is derived (ro); endpoint move is a canvas op |
| `Via` | drill, pad size, layer span, net | drill/pad/span → `SetObjectField`; net derived (ro); position is a canvas op |
| `Zone` | net, layer, fill-state badge (000B honesty), clearance | net/layer/clearance → `SetObjectField`; fill is `SetZoneFill`/`DeleteZoneFill` (not a field edit); boundary edits are canvas tool ops |
| `ComponentInstance` / footprint | refdes, value, footprint id (ro), rotation, side, locked, variant fit | refdes → `SetReference`; value → `SetValue`; rotation → `RotateComponent`; side → `SetComponentSide`; locked → `SetComponentLocked`; position via canvas `MoveComponent` |
| board `Text` | string, layer, height, family, render intent, mirror, keep-upright, bold, h/v align, rotation, line-spacing | the realized `EditSelectedBoardText*` / `CycleSelectedBoardText*` / `ToggleSelectedBoardText*` ops in `crates/gui-render` `HitTarget` (the FIRST shipped Inspector slice) |
| schematic symbol | refdes, value, properties, position | refdes → `SetReference`; value → `SetValue`; arbitrary property → `SetSymbolField`; position via canvas op |
| `Net` / net-class | name (ro), net-class, class rule summary | net-class → `SetNetClass`; class RULE rows defer to the `Rules` panel (`GUI_LIVE_FEEDBACK_AND_RULES.md`), never edited inline here (§3.2 rule 1) |
| multi-select (homogeneous) | shared editable fields; mixed values shown as `<varies>` | one `OperationBatch` of the per-field op over all selected (Altium multi-edit parity); above the §4.3 threshold routes to a `Proposal` |
| multi-select (heterogeneous) | common fields only (e.g. layer) or "N objects" summary | `OperationBatch` over the common field; never invents a cross-type op |

The existing board-text Inspector affordances in `crates/gui-render`
`HitTarget` (`ToggleSelectedBoardText*`, `CycleSelectedBoardText*`,
`EditSelectedBoardText*`) are the FIRST realized Inspector slice and the
template every other selection class follows. Any field marked `(ro)` is
read-only in the Inspector and has NO edit affordance even in the
interactive phase — it is changed (if at all) by a canvas tool op or a
derived recomputation, preserving §3.2 rule 1 (one editor per property).

### 4.2 Validate-before-commit field path (`QG-DIRECT-EDITING-FEEL`)

Every editable Inspector field follows one path, stated as a state machine
in §6.2. The non-negotiable behavior (002):
- a field edit is validated BEFORE it commits; an invalid value is
  rejected with the error reported AT the edited field (not a modal), and
  no `Operation` is produced;
- a valid edit produces exactly one typed `Operation` (or an
  `OperationBatch` for multi-select) through the single `commit()`;
- the commit is journaled with durable undo (002); undo across
  close/reopen works because undo is a journal cursor, not an in-memory
  stack (master 3.2 #3);
- supervision-first (013): the Inspector ships read-only first — it
  reflects committed field values for the selected object with NO edit
  affordance — and the edit affordances arm in the named interactive
  phase, per-selection-class. "NO edit affordance" is a STRUCTURAL
  guarantee, not a disabled-control one: in the supervision build every
  `InspectorField.editable == false` AND the scene contract emits the field
  as a `ReadOnlyField` primitive with no focus/edit hit-target, so the
  `gui_ia_supervision_shell` golden + `it_ia_supervision_readonly` itest can
  assert the rendered scene contains zero editable Inspector hit-targets
  (not merely greyed ones). This is the auditability posture: a reviewer can
  trust the supervision build cannot mutate by construction, not by policy.

### 4.3 High-risk fields route to PROPOSALS, not direct commit

Per master Open Question 5 and 002 OQ6, certain Inspector edits must be
PROPOSALS (a `Proposal` that later calls the same `commit()`), not direct
commits. The MECHANISM is fixed (proposal → review surface in
`GUI_AI_SURFACES.md` → same `commit()`); the `commit_kind` is computed by a
pure classifier so the routing is deterministic and testable, not a UX
afterthought:

```
fn commit_kind(field: &InspectorField, sel: &Selection) -> CommitKind {
    // Default-buildable policy; the EXACT membership is OQ §10.4.
    if field.is_destructive()                 { Proposal } // delete/clear net
    || field.touches_imported_geometry()      // repair of imported objects (IMP track)
    || field.is_standards_deviation()         // overrides a STANDARDS_COMPLIANCE rule
    || field.is_relationship_state_change()   // net/class reassignment that re-derives connectivity
    || sel.count() > BATCH_PROPOSAL_THRESHOLD // default 25 objects (OQ §10.4)
        { CommitKind::Proposal } else { CommitKind::DirectOp }
}
```

A `Proposal`-classified field renders a "propose" affordance instead of an
inline commit; SM-1's `Validating → Idle (enqueue Proposal)` transition
fires (never `Committing`), so a high-risk Inspector edit CANNOT silently
direct-commit. The default `BATCH_PROPOSAL_THRESHOLD = 25` and the exact
predicate membership are the open owner question (§10.4); the classifier
shape, the SM-1 routing, and the "no high-risk field direct-commits" gate
(`it_ia_inspector_propose_routing`) are fixed by this spec.

---

## 5. Docking Model

### 5.1 The model: typed `LayoutGraph`, consumer state only

The docking model is the `LayoutGraph` from 000B/007: a UI graph of panes,
splits, tiles, floating windows, overlays, PiP, and pinned sidecars, with
tab references by id. It is **consumer state**: it lives in
`WorkspaceState`, has its own `workspace_revision`, and its save/restore
NEVER advances `model_revision` and NEVER emits an `Operation` (007). This
is the hard line that keeps the docking host out of design authority.

The existing shell (`crates/gui-render` `ShellLayout`:
`left_sidebar`/`right_sidebar`/`bottom_strip`/`viewport`) is the FIXED-REGION
floor. This spec specifies the path from that floor to a real docking
graph: the fixed regions become the DEFAULT placement of panes in a
`LayoutGraph`, and docking/floating/tabbing become typed mutations of that
graph. The fixed-region shell remains the fallback when no `WorkspaceState`
is persisted.

### 5.2 Panel placement states

Matching Altium's panel states (docked / floating / popout-auto-hide /
tabbed) and exceeding the KiCad fixed-pane floor:

- `Docked(region)` — region ∈ {LeftEdge, RightEdge, BottomEdge, TopEdge,
  Center}. Multiple panels in one region stack as a `TabGroup`.
- `Floating(window)` — a free window (single-monitor first; multi-monitor
  is a master Non-Goal for the first cut).
- `AutoHide(edge)` — collapsed to an edge strip, expands on hover
  (Altium "popout"). Deferred behind docked/floating.
- `PinnedContext(selection)` — a panel/tab pinned to the active selection
  (000B `PinnedContext`): e.g. schematic PiP pinned while routing. The
  pin link is `LayoutGraph` edge `pin-to-selection` (007).

### 5.3 Persistence boundary (the three revisions)

The IA must make the three revision streams visibly distinct (007 first
proof slice; master must show source vs workspace vs artifact):
- `model_revision` — advances only on a source `commit()`.
- `workspace_revision` — advances on docking/layout/visibility changes;
  never touches the model.
- artifact revision — per `Artifact` snapshot (000B).

A status surface (project header / DesignTree root) shows all three plus
stale-projection state. `SessionState` (active tool, hover, drag preview,
unsent terminal scrollback) is volatile and discarded on crash; its loss
must never lose committed design or persisted layout (007).

### 5.4 WorkbenchProfiles (saved per-task workspaces)

A `WorkbenchProfile` (000B/007) is a named, reusable `LayoutGraph` +
panel-visibility + tool defaults. Applying one rearranges views; it does
NOT mutate design/rules/artifacts/manufacturing (007). Initial profile set
(007 expected list):
- `supervision` (FIRST, per 013) — DesignTree + Inspector(read-only) +
  Violations + Output, all reflecting committed state; the instrument panel.
- `pcb-layout` — DesignTree/Components/Nets left, Inspector + Layers right,
  Violations bottom.
- `schematic-capture` — sheet tree left, Inspector right, ERC findings
  bottom.
- `manufacturing-review` — Output/OutJob + ArtifactPreview + Layers tiled,
  source PiP (000B UI consequences).
- `checks-and-rules` — Rules (Constraint surface) + Violations focused.
- `ai-debug` — Assistant + Terminal + proposal/diff tabs (master
  differentiator 1 lives here).
- `focused-single-view`, `multi-monitor-review` (007).

Profiles are persisted as `WorkspaceState` records (own `workspace_revision`),
keyed by owner scope (project-default / user-local / shared — 007 OQ2,
open). Old profiles degrade gracefully on schema change, never blocking
project open (007 OQ7).

---

## 6. Buildable Content

### 6.1 Scene-contract / workspace-state schema extensions

These extend `crates/gui-protocol` (the `WorkspaceUiState`/`DockTab`
family), in the determinism shape the master §6.1 requires: versioned,
byte-stable ordering, identity-stable on unchanged persisted state, no new
field introducing design authority. NOT engine `DesignModel` types — these
are CONSUMER-STATE types (007). Sketch (illustrative, the workflow lands
the exact Rust):

```
// Panel taxonomy
enum PanelKind {
    Inspector, DesignTree, Components, Nets, Layers, Rules,
    Violations, Output, Terminal, Assistant, ArtifactPreview,
    NavigatorOverview,
}
enum PanelAuthority { Reflect, Edit, Compose, Navigate }

// Docking graph (consumer state; lives in WorkspaceState, not DesignModel)
struct LayoutGraphV1 {
    schema_version: u32,                 // degrade-safe (007 OQ7)
    nodes: Vec<LayoutNode>,              // byte-stable sorted by node_id
    edges: Vec<LayoutEdge>,              // containment/focus/pin/projection-link
    focused_pane: Option<PaneId>,
}
enum LayoutNode {
    Pane { id: PaneId, tabs: Vec<ViewTabRef>, active_tab: Option<ViewTabId> },
    Split { id: PaneId, axis: Axis, ratio_ppm: u32, a: PaneId, b: PaneId },
    Tile  { id: PaneId, children: Vec<PaneId> },
    Floating { id: PaneId, pane: PaneId, frame_px: RectPx },
    Overlay { id: PaneId, pane: PaneId },
    Pip { id: PaneId, pane: PaneId, pinned_to: Option<ObjectId> },
    Sidecar { id: PaneId, pane: PaneId, edge: DockEdge },
}
struct ViewTabRef { id: ViewTabId, kind: PanelKind, title: String }
enum DockEdge { LeftEdge, RightEdge, BottomEdge, TopEdge }

// Persisted workspace (007) — NOT design authority
struct WorkspaceStateV1 {
    schema_version: u32,
    workspace_id: String,
    project_id: String,
    owner_scope: OwnerScope,             // ProjectDefault | UserLocal | Shared
    layout_graph: LayoutGraphV1,
    active_profile: Option<WorkbenchProfileId>,
    selected_object_ids: Vec<ObjectId>,  // by stable ID; degrade if missing
    navigation_stack: Vec<NavEntry>,     // stable-ID refs (007 NavigationStack)
    visibility: WorkspaceFilterState,    // reuses existing type
    last_known_model_revision: String,   // staleness display only, NOT authority
    workspace_revision: u64,             // advances on layout change only
}
struct WorkbenchProfileV1 {
    schema_version: u32,
    id: WorkbenchProfileId,
    name: String,
    layout_graph: LayoutGraphV1,
    panel_visibility: Vec<(PanelKind, bool)>,
    tool_defaults: Vec<(String, String)>,
}

// Inspector context model (consumer state; the Edit path is §6.2)
struct InspectorModelV1 {
    selection_class: SelectionClass,     // pure fn of current selection
    fields: Vec<InspectorField>,         // byte-stable ordered
}
struct InspectorField {
    key: String,
    label: String,
    value: InspectorValue,               // Concrete | Varies (multi-select)
    render: FieldRender,                  // ReadOnly | Editable — structural, §4.2
    op_target: OpTarget,                  // the contract op this field emits (§4.1)
    commit_kind: CommitKind,             // DirectOp | Proposal (§4.3 classifier)
}
enum FieldRender { ReadOnly, Editable }  // supervision build => all ReadOnly,
                                         //   no edit hit-target emitted (013)
enum OpTarget {                          // never an invented op (§4.1)
    None,                                // (ro) / derived field
    Named(&'static str),                 // e.g. "SetPadNet", "RotateComponent"
    ObjectField { field_path: String },  // generic SetObjectField fallback
}
enum CommitKind { DirectOp, Proposal }
```

The supervision build (013) constructs every `InspectorField` with
`render = ReadOnly`, and the scene contract emits NO edit hit-target for a
`ReadOnly` field — the structural guarantee §4.2 requires and
`it_ia_supervision_readonly` asserts. The interactive phase flips eligible
fields to `Editable` per the §4.1 table; `(ro)`/`None` fields stay
`ReadOnly` permanently. `op_target` makes the contract-parity gate
(PS-IA-2) checkable: the itest asserts the `Operation` the field emits is
byte-identical to the one the named CLI op emits.

Determinism: `WorkspaceStateV1` is keyed by `workspace_revision`, never by
`model_revision`; save/restore is a no-op on the journal. The
`last_known_model_revision` is diagnostic only (007).

### 6.2 Interaction state machines

#### SM-1: Inspector field edit (`QG-DIRECT-EDITING-FEEL`)

States: `Idle` → `Editing` → `Validating` → (`Committing` | `Rejected`).

| From | Event | To | Consumer state mutated | Crosses to journal? |
|---|---|---|---|---|
| `Idle` | selection changes | `Idle` | `InspectorModel` rebuilt from selection (pure fn) | no |
| `Idle` | field focus + edit (interactive phase only) | `Editing` | field buffer | no |
| `Editing` | keystroke | `Editing` | field buffer | no |
| `Editing` | cancel (Esc) | `Idle` | discard buffer | no — zero mutation, zero dirty shard |
| `Editing` | commit (Enter/blur) | `Validating` | — | no |
| `Validating` | invalid | `Rejected` | error attached at field; buffer kept | no — no `Operation` produced |
| `Rejected` | edit | `Editing` | field buffer | no |
| `Validating` | valid + `commit_kind=DirectOp` | `Committing` | — | YES — builds one `Operation`/`OperationBatch`, calls `commit()` |
| `Validating` | valid + `commit_kind=Proposal` | `Idle` | enqueues `Proposal` (review surface owns accept) | no — proposal, not direct commit |
| `Committing` | commit ok | `Idle` | `InspectorModel` rebuilt from new committed value | journaled `TransactionRecord`, durable undo |
| `Committing` | commit err | `Rejected` | error at field | no shard change |

Invariant: the only journal-crossing transition is `Validating → Committing`
on a valid `DirectOp`. Selection/hover/buffer never cross.

#### SM-2: Dock manipulation (`Compose`; never journaled)

States: `Resting` → (`Dragging` | `Resizing` | `Tabbing` | `Floating`).

| From | Event | To | Mutated | Journal? |
|---|---|---|---|---|
| `Resting` | drag panel header | `Dragging` | drag preview (`SessionState`) | no |
| `Dragging` | hover dock target | `Dragging` | drop-zone highlight | no |
| `Dragging` | drop on region | `Resting` | `LayoutGraph` node added/moved; `workspace_revision`++ | no — consumer state |
| `Dragging` | drop on float | `Floating`→`Resting` | `Floating` node added | no |
| `Dragging` | cancel (Esc) | `Resting` | discard preview | no |
| `Resting` | drag splitter | `Resizing` | `Split.ratio_ppm` live preview | no |
| `Resizing` | release | `Resting` | commit ratio; `workspace_revision`++ | no |
| `Resting` | drop tab into pane | `Tabbing`→`Resting` | move `ViewTabRef` between panes | no |

Invariant: no transition in SM-2 produces an `Operation` or advances
`model_revision`. Crash during any state discards only `SessionState`.

#### SM-3: Profile apply / save

States: `Active` → (`Applying` | `Saving`).

| From | Event | To | Mutated | Journal? |
|---|---|---|---|---|
| `Active` | apply profile P | `Applying`→`Active` | `LayoutGraph` replaced from P; tabs opened by reference | no — opening a tab is not replaying edits (007) |
| `Active` | save-as profile | `Saving`→`Active` | new `WorkbenchProfileV1` persisted; `workspace_revision`++ | no |
| `Active` | restore workspace (open) | `Active` | `WorkspaceState` resolved against CURRENT model; missing objects degrade | no — stale workspace marked, model not rolled back (007) |

#### SM-4: Cross-panel navigate (DesignTree/Components/Nets → selection)

States: `Idle` → `Navigating` → `Idle`.

| From | Event | To | Mutated | Journal? |
|---|---|---|---|---|
| `Idle` | click tree/list node | `Navigating` | selection set to node's `object_id`; `NavigationStack` push | no |
| `Navigating` | resolve | `Idle` | Inspector rebuilds (SM-1 `Idle`); canvas may re-frame (camera, consumer) | no |

### 6.3 Panel / component specs with context rules

Per master §6.3, each panel ships with: content-per-selection-class
(the context rule), validate-before-commit with the error at the edited
field, and the commit path each editable field takes.

- **Inspector** — context rule is the §4.1 table; commit path is SM-1;
  read-only supervision source is the selected object's committed fields at
  `model_revision`.
- **DesignTree** — context: reflects `ProjectManifest`-resolved structure;
  no edit path (Navigate only); supervision source: resolved `DesignModel`
  structure + recovery diagnostics.
- **Components/Nets** — context: list filtered by active board/sheet/variant;
  Navigate only; "edit" defers to Inspector (§3.2 rule 1); source:
  component/net projections of the model.
- **Layers** — context: layers present in the model + filter toggles;
  Compose only; source: `WorkspaceFilterState`.
- **Rules** — context: rule rows for the selected net-class/scope; Edit via
  rule ops; content OWNED by the rules domain / `GUI_LIVE_FEEDBACK_AND_RULES.md`,
  placement/docking owned here.
- **Violations** — context: findings for active board/run; Navigate
  (finding → source object + projection, 000B); source: `CheckRun`/
  `CheckFinding`; no edit (repair is proposal-first, 009).
- **Output** — context: output jobs/artifacts/manufacturing plans; Reflect
  +Navigate; source: `ProductionStatus` (already in `gui-protocol`).

### 6.4 Proof slices

Each names the fixture (default `datum-test`) and the gates it must pass.

- **PS-IA-1 (supervision shell, FIRST, 013).** Open `datum-test`. The
  `supervision` WorkbenchProfile renders DesignTree (full structure),
  Inspector in read-only mode reflecting committed fields for a selected
  pad/footprint/track, Violations reflecting the committed `CheckRun`, and
  Output reflecting `ProductionStatus` — ALL read-only, no edit affordance.
  Gates: renders real committed state at a known `model_revision`;
  `QG-RESOLVER-RECOVERY` (a deliberately split project opens this profile
  in badged recovery, not as truth); visual golden (PS-IA-1.golden).
- **PS-IA-2 (Inspector edit through commit, interactive phase).** Select a
  board-text object on `datum-test`; edit a field via SM-1; the change
  produces ONE typed `Operation`, journals a `TransactionRecord`, and is
  undoable across close/reopen (journal cursor). An invalid value is
  rejected at the field with zero `Operation` produced. Gates:
  `QG-DIRECT-EDITING-FEEL`, durable-undo, single-`commit()` parity with the
  equivalent CLI op; interaction test PS-IA-2.itest.
- **PS-IA-3 (docking + profile persistence, no authority).** Dock/float/
  tab panels (SM-2), save a WorkbenchProfile, restart, restore it (SM-3).
  Gate: `workspace_revision` advanced, `model_revision` UNCHANGED, journal
  empty (`PG-SHARD-DIFF-ISOLATION`-style assertion: zero source-shard byte
  change); session-state loss tolerance (drag preview discarded, layout +
  committed data recovered, 007). Interaction test PS-IA-3.itest.
- **PS-IA-4 (cross-panel navigate).** Click a net in the Nets panel (SM-4);
  selection + `NavigationStack` update; Inspector rebuilds to the net's
  fields; canvas re-frames. Gate: Navigate produces zero `Operation`;
  visual golden PS-IA-4.golden.
- **PS-IA-5 (high-risk field routes to proposal, not direct commit).** On
  `datum-test`, select an object and trigger a `commit_kind=Proposal` field
  (a destructive clear, or a homogeneous multi-select above
  `BATCH_PROPOSAL_THRESHOLD`). SM-1 takes the `Validating → Idle (enqueue
  Proposal)` transition. Gate: a `Proposal` is enqueued, ZERO direct
  `commit()` / zero `TransactionRecord` is produced by the Inspector, and
  the propose affordance (not an inline-commit control) is rendered. This is
  the IA-side guard for the AI-native ghost/diff wedge (master differentiator
  1). Interaction test `it_ia_inspector_propose_routing`; golden
  `gui_ia_inspector_propose`.

### 6.5 Visual-golden acceptance

Per master §6.4/§6.5, every surface ships a golden + interaction test in
the `gui-render` harness (`visual_runner.rs`/`visual_manifest.rs`/
`visual_diff.rs`; bless + exact/tolerance diff).

| Surface | Golden | Interaction test | Fixture / scene | Acceptance |
|---|---|---|---|---|
| Supervision shell | `gui_ia_supervision_shell` | `it_ia_supervision_readonly` | `datum-test`, supervision profile | exact diff; no edit affordance present |
| Inspector context (per class) | `gui_ia_inspector_<class>` (pad/track/via/zone/component/text/net) | `it_ia_inspector_edit_commit` | `datum-test` selection scenes | tolerance diff on panel; itest asserts one `Operation` + durable undo |
| Inspector reject | `gui_ia_inspector_reject` | `it_ia_inspector_invalid_field` | `datum-test` | error rendered at field; zero `Operation` |
| Inspector propose routing | `gui_ia_inspector_propose` | `it_ia_inspector_propose_routing` | `datum-test` destructive/batch-over-threshold scene | propose affordance shown (not inline commit); SM-1 enqueues a `Proposal`, zero direct `commit()` |
| Docking states | `gui_ia_dock_docked`, `gui_ia_dock_floating`, `gui_ia_dock_tabbed` | `it_ia_dock_no_journal` | synthetic layout scene | exact diff; itest asserts zero source-shard change |
| WorkbenchProfile restore | `gui_ia_profile_<name>` (supervision/pcb-layout/manufacturing-review) | `it_ia_profile_restore` | `datum-test` | exact diff after restart; `model_revision` unchanged |
| DesignTree | `gui_ia_design_tree`, `gui_ia_design_tree_recovery` | `it_ia_tree_navigate` | `datum-test` + split fixture | recovery subtree badged distinctly |
| Cross-panel navigate | `gui_ia_nav_select` | `it_ia_navigate_no_op` | `datum-test` | Navigate produces zero `Operation` |

A surface is not accepted on prose. A golden must render real committed
state; a layout-only golden (docking) must additionally assert no journal
crossing in its paired interaction test.

---

## 7. Match vs Exceed

MATCH (table stakes — reach the bar, do not innovate):
- The single context-sensitive Inspector ("one panel edits the selection")
  matches the Altium Properties panel behavior and multi-select `<varies>`.
- Dockable/floating/tabbed/pinned panels with saved profiles match Altium
  Workspaces and Xpedition Display Control.
- A separate Constraint surface (Rules panel) matches Allegro Constraint
  Manager; a findings navigator (Violations) matches Altium Messages.
- Multi-board/multi-sheet DesignTree matches CR-8000 navigation.

EXCEED (the architecture wedge — most design attention):
- **Deterministic IA.** Every Inspector/Rules edit is a typed `Operation`
  through ONE `commit()` (SM-1), identical to the CLI/MCP path, so every
  panel action is replayable, scriptable, and undoable across reopen via
  journal cursors. Commercial Inspectors mutate an in-memory document and
  rely on a save+undo-stack; Datum's panels are deterministic instruments.
- **Authority firewall by construction.** The docking graph CANNOT corrupt
  design state: `LayoutGraph`/`WorkspaceState` is consumer state with its
  own `workspace_revision`, and SM-2/SM-3 provably never cross the journal
  (PS-IA-3). Closing/moving/floating a tab can never lose or re-author
  design data (007) — a guarantee commercial document-window models do not
  make structurally.
- **Supervision-first IA.** The first shipped profile is read-only
  supervision (013): the IA exists to let a human AUDIT committed engine
  state before any edit affordance arms. No commercial tool ships a
  read-only-first instrument posture as a deliberate phase.
- **Proposal-aware Inspector.** High-risk fields route to PROPOSALS that
  flow back through the same `commit()` (§4.3) — the IA hook for the
  AI-native ghost/diff wedge (master differentiator 1) lives in the
  Inspector's `commit_kind`.

---

## 8. Non-Goals (this area)

Inherits the master Non-Goals. Additionally, this area does NOT require:
- multi-monitor floating windows in the first cut (single-monitor docking +
  floating first; 007/012, master Non-Goal);
- auto-hide ("popout") panels before docked/floating are credible;
- the `NavigatorOverview` minimap before the core catalogue panels;
- a finalized on-disk `WorkspaceState`/`WorkbenchProfile` serialization
  format (007 Non-Goal: no final workspace serialization format);
- treating tabs as files or as a data authority (007/000B);
- journaling workspace/layout changes for audit (007 OQ6 is open; default
  is NOT journaled — layout is consumer state);
- duplicating any editable property across two panels (§3.2 rule 1
  forbids it);
- a Rules-panel content spec — placement/docking owned here, content owned
  by the rules domain / `GUI_LIVE_FEEDBACK_AND_RULES.md`.

---

## 9. Dependencies / Interfaces with Other Areas

The six GUI area specs live in `specs/gui/`. This spec is the SHELL
(area: information-architecture). Names below are the actual files, not
placeholders. Each interface states the ownership split and the conflict
resolution rule so two specs can never both claim an authority surface.

- `GUI_SUPERVISION_REFLECTION.md` (GUI area 1, the FIRST deliverable per
  013) — owns the Reflect CONTENT (what committed/derived state each
  read-only panel surfaces and how staleness is shown). This spec owns the
  supervision PROFILE's panel placement/docking (§5.4) and that PS-IA-1 is
  the IA instantiation of that area's instrument. **Conflict rule:**
  supervision spec on Reflect content, this spec on placement/persistence.
- `GUI_LIVE_FEEDBACK_AND_RULES.md` (area 3.b) — owns the Rules/Constraint
  editor CONTENT (rule rows, scope resolution, validation rules) and the
  Violations finding CONTENT and online-DRC feedback. This spec owns where
  the `Rules` and `Violations` panels DOCK, how they PERSIST, and the
  ownership firewall (§3.2). The two must agree on the §4.1 Net/net-class
  rows and the §3.2-rule-1 firewall. **Conflict rule:** live-feedback spec
  on rule/finding content + validation, this spec on placement + the
  one-editor-per-class invariant.
- `GUI_INTERACTION_GRAMMAR.md` (area 2) — owns selection/hover/drag/tool
  modes as consumer state (002/000B), and the Inspector field-edit GESTURE
  grammar. This spec consumes selection to drive the Inspector context rule
  (§4.1) and Navigate (SM-4). **Conflict rule:** grammar spec on the
  gesture/selection model, this spec on what the resulting selection renders
  in the Inspector and how SM-1 reaches `commit()`.
- `GUI_CANVAS_AND_RENDERING.md` (area 3.a) — owns the canvas surface, the
  identity-triple-bearing renderables, camera, and the Output/ArtifactPreview
  live-CAM render. This spec owns those panels' PLACEMENT and the
  `manufacturing-review` profile; the canvas spec owns what they DRAW.
  **Conflict rule:** canvas spec on render content + camera, this spec on
  panel placement + profile composition.
- `GUI_AI_SURFACES.md` (area 4, the wedge) — owns the ghost/diff proposal
  review surface and the Assistant panel CONTENT. This spec owns the
  Inspector `commit_kind=Proposal` panel-side HOOK (§4.3): a high-risk
  Inspector field enqueues a `Proposal` that the AI-surfaces review surface
  renders and accepts back through the same `commit()`. **Conflict rule:**
  AI-surfaces spec on proposal review/accept UX, this spec on the Inspector
  field's propose affordance and `commit_kind` routing.

Cross-probe (N-way cross-select across schematic/PCB/3D/BOM/CAM/findings)
has its own dedicated coordinating spec `GUI_CROSS_PROBE.md` (the marquee
differentiator 2), which owns the cross-probe MODEL and consumes the
selection identity (`GUI_INTERACTION_GRAMMAR.md`) and emphasis rendering
(`GUI_CANVAS_AND_RENDERING.md`). THIS spec supplies the `PinnedContext`/PiP
placement (§5.2), the multi-pane/floating `LayoutGraph`, and the
`NavigationStack` IA substrate that cross-probe's jump/push verbs compose in
(`GUI_CROSS_PROBE.md` §9.4/§10). **Conflict rule:** cross-probe spec owns
the resolution model + emphasis-broadcast coordination, this spec owns the
pane/PiP/floating layout a jump or push lands in. Live-production (OutJob)
behaviors are owned within `GUI_CANVAS_AND_RENDERING.md` /
`GUI_LIVE_FEEDBACK_AND_RULES.md`; this spec supplies their panel placement.

---

## 10. Open Owner Questions

1. Which docking states are mandatory FIRST (007 OQ1 / 000B OQ2): docked +
   tabbed only, or docked + floating + pinned-sidecar in the first cut?
2. `WorkspaceState` default owner scope (007 OQ2): project-local shared,
   user-local sidecar, or both with explicit scope?
3. Are any workspace/layout changes journaled for audit (007 OQ6), or is
   layout strictly non-journaled consumer state (this spec's default)?
4. (§10.4) The §4.3 `commit_kind` classifier shape and the SM-1 propose
   routing are FIXED; OPEN is the exact predicate membership (which of:
   relationship-state changes, destructive deletes, imported-geometry
   repair, standards deviations) and the value of `BATCH_PROPOSAL_THRESHOLD`
   (default 25). Confirm/adjust the four predicates and the threshold (master
   OQ5 / 002 OQ6).
5. Minimum WorkbenchProfile set for first release (007 OQ4): is `supervision`
   the only mandatory profile, or are `pcb-layout` + `manufacturing-review`
   also required day one?
6. Multi-select Inspector scope: homogeneous-only first, or heterogeneous
   common-field editing in the first interactive cut?
7. Project dashboard/status indicators required before serious manual
   workflows (007 OQ5): which of model/workspace/artifact revision + check
   status + active variant/output-job must be on-screen always?
8. Schema-degrade policy for old `WorkspaceState`/profile records (007 OQ7):
   silently drop unknown nodes, or open in a layout-recovery mode mirroring
   `QG-RESOLVER-RECOVERY`?
9. Does the Layers/Display-Control panel belong left or right by default,
   and should layer visibility be part of a WorkbenchProfile or always
   user-local?
