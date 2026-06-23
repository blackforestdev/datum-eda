# Datum EDA GUI Area Specification: Interaction Grammar

Status: draft GUI area specification, 2026-06-22, benchmarked to
commercial EDA. Controlling for the interaction-grammar domain. Conforms
to and inherits from `specs/GUI_SPEC.md` (the master): its bar, thesis,
architecture constraints, and five-part buildability standard apply here
unchanged and are not restated except where this area adds detail.

Driven by:
- `specs/GUI_SPEC.md` (master)
- `docs/decisions/PRODUCT_MECHANICS_002_MANUAL_EDITOR_BASELINE.md`
- `docs/decisions/PRODUCT_MECHANICS_000B_VIEW_COMPOSITION_AND_LIVE_PRODUCTION.md`
- `docs/decisions/PRODUCT_MECHANICS_005_EMBEDDED_TERMINAL.md`
- `docs/decisions/PRODUCT_MECHANICS_013_GUI_SUPERVISION_AND_PARITY.md`
- `specs/SCHEMATIC_EDITOR_SPEC.md`
- `crates/gui-app/src/terminal_input.rs` (existing key-action model)
- `crates/gui-protocol/src/lib.rs` (`BoardReviewSceneV1`, identity triple)
- `crates/gui-protocol/src/terminal_command_catalog.rs` (handoff conduit)

This area is master area #2's grammar half: where
`GUI_CANVAS_AND_RENDERING.md` owns canvas quality and the per-domain
tool state machines' geometric detail, THIS spec owns the **interaction
grammar** that binds all tools: the command-first modeless model, full
keyboard reachability, the selection model and filter, snapping, the
ghost-preview contract, undo/redo granularity against the journal, and
the command palette. It defines the SHARED interaction state machine
every tool plugs into and the per-tool state machines for the grammar's
own tools (select, command-palette, pan/zoom).

---

## 1. Purpose

State the controlling interaction grammar for the Datum GUI: the rules
that make every surface feel like one coherent professional instrument
rather than a collection of dialogs. The grammar is the part of the GUI
a designer touches on every single action — selection, keyboard, snap,
preview, undo — so it is held to the highest fluency bar in the product.

The grammar is **engine-neutral consumer state** (master §4.2): nothing
in this spec is journaled except the single `commit()` transition each
tool's state machine reaches. Selection, hover, the active tool, the
snap state, the in-progress ghost, the palette query buffer, and the
camera are all consumer state, never `Operation`s, never written to a
shard.

---

## 2. Commercial Benchmark + Match-vs-Exceed

### 2.1 The named patterns we benchmark

| Datum grammar element | Commercial pattern matched | Tool |
|---|---|---|
| Heads-up display (HUD) cursor readout (fields fixed by `HudReadout`, §4.2): absolute X/Y, relative dX/dY/dist/angle from gesture anchor, active segment length + width, active layer, resolved snap source, net name + projected DRC during routing | Altium **heads-up display** (Shift+H toggle) — absolute/relative coords, length, angle, net, layer during placement/routing | Altium Designer |
| Modeless command-first interaction; a shortcut starts a tool, Esc/right-click cancels, no trapping modal | Altium command system + **interactive routing modelessness**; Allegro right-mouse-button (RMB) verb menu | Altium / Allegro |
| Object-aware right-click verb menu (context built from selection class) | Allegro **RMB context menu**; Altium right-click | Allegro / Altium |
| Selection filter (toggle which object classes are selectable) | Altium **Selection Filter** toolbar / `Shift+C` style filters | Altium Designer |
| Select-same-net / select-connected-copper | Altium **Select > Connected Copper / Net**; Allegro highlight-net | Altium / Allegro |
| Live ghost preview during placement and routing (rubber-band track, footprint outline on cursor) | Altium **interactive routing** rubber-band; Xpedition **sketch routing** preview | Altium / Xpedition |
| Snap to grid / pad / track-end / guides with snap-source feedback | Altium snap + **guides**; Allegro snap grid | Altium / Allegro |
| Command palette (fuzzy command finder) | Altium **command access** (`Ctrl+'`-style runner); modern pro-app command palettes (VS Code-class) | Altium / modern pro-app |
| Undo/redo with meaningful granularity (one user gesture = one undo step) | Every commercial tool's transaction undo | All |
| Full keyboard reachability (no action mouse-only) | Allegro/Altium scriptable command lines; accessibility floor | All |

KiCad is the floor: KiCad has modeless tools, a selection filter, and
highlight-net, so Datum's grammar must at minimum equal those before any
grammar surface is claimed shipped. We do not benchmark against it; it
is the line below which we cannot fall.

### 2.2 Where we MATCH

- HUD readout, modeless tools, object-aware right-click, selection
  filter, select-same-net/connected-copper, ghost preview, snap with
  feedback, and a command palette are **table stakes**. We match the
  commercial bar on fluency and correctness; we do not innovate on them.
  A grammar element is not "done" because it functions; it is done when
  it would not embarrass us next to Altium on the identical gesture.

### 2.3 Where we EXCEED (the architecture wedge)

1. **Undo is journal-durable, not an in-memory stack.** Altium/Allegro
   undo dies on close. Datum undo/redo maps to journal cursors (master
   §4.2; Decision 002 recovery proof): every user gesture is one typed
   `OperationBatch` on `commit()`, so undo survives close/reopen and is
   identical across CLI/MCP/GUI. We exceed by construction.
2. **The command palette is the same op vocabulary as CLI/MCP.** A
   palette entry that mutates does not call a private GUI handler; it
   constructs the identical `OperationBatch` the CLI sub-command and the
   MCP tool construct, or it prefills the sanctioned terminal-handoff
   conduit (`terminal_command_catalog.rs`,
   `TerminalCommandHandoff`). The palette is a typed-op launcher, not a
   menu reskin — so every palette action is replayable and scriptable.
3. **Selection is one identity, not a name join.** Select-same-net and
   cross-probe resolve over the single `DesignModel`'s identity triple
   (`object_id` / `object_kind` / `source_object_uuid`), not a refdes or
   net-name match across two file authorities. Same wedge as master
   differentiator 2; the grammar's selection model is built on it.
4. **Ghosts are a first-class scene-contract primitive, shared with the
   AI canvas.** The same ghost/preview primitive that renders a tool's
   in-progress geometry renders an agent's proposed edit (master
   differentiator 1). Manual preview and AI proposal are one rendering
   path, so a designer reads "uncommitted geometry" identically whether
   it came from their cursor or an agent.

---

## 3. The Grammar Doctrine (normative rules)

These hold across every tool and surface. A tool spec may not relax them.

- **G1 — Command-first, modeless.** Every tool is entered by an
  intentional command (keybinding or palette), runs until explicitly
  ended, and never traps the user in a modal that blocks pan/zoom,
  selection inspection, or cancellation. There is no "OK/Cancel dialog"
  wall between intent and canvas for routine edits.
- **G2 — Esc is universal cancel; it never commits.** Esc steps back one
  level (drop the in-progress vertex, then drop the tool). Cancel leaves
  **zero committed mutation and zero dirty shard** (master §6.2).
- **G3 — Every action is keyboard-reachable.** Each command has a stable
  `command_id` and at least one default binding; the palette reaches the
  long tail. No action is mouse-only. (Allegro/Altium scriptable-command
  floor; accessibility requirement.)
- **G4 — Selection/hover/active-tool/snap/ghost/camera are consumer
  state.** Never journaled, never an `Operation` (master §4.2; 000B).
- **G5 — One gesture = one undo step.** The granularity rule (§8): a
  single user-perceived action produces exactly one `OperationBatch` on
  `commit()`, hence exactly one undo step. Multi-segment routes and
  multi-object moves are explicitly defined as one batch.
- **G6 — Preview is honest.** A ghost shows what WILL be committed if the
  commit transition fires now, drawn distinctly from committed geometry,
  including projected DRC state where the tool computes it (matches
  Altium online-DRC-while-routing). It never shows geometry that the
  commit would not produce.
- **G7 — The right-click menu is built from the live selection class.**
  Verb availability is a pure function of the current selection set and
  active tool; it lists only verbs that would commit a valid op
  (Allegro RMB pattern).

---

## 4. Command Model (scene-contract extension)

The grammar needs a typed, versioned command catalog the GUI, palette,
goldens, and keymap all read from. It extends the existing
`terminal_command_catalog` shape (`TerminalCommandCatalogEntry`,
`TerminalCommandHandoff`) into a general GUI command catalog.

```rust
/// Versioned command catalog the palette, keymap, right-click menu, and
/// goldens all resolve against. Pure metadata: holds NO design authority.
pub const GUI_COMMAND_CATALOG_VERSION: &str = "datum.gui_command_catalog.v1";

pub struct GuiCommandEntry {
    /// Stable identifier, e.g. "edit.route.start", "select.same_net".
    pub command_id: String,
    /// Human label shown in palette + right-click menu.
    pub label: String,
    /// Searchable synonyms for palette fuzzy match (e.g. "track","trace").
    pub keywords: Vec<String>,
    /// Grouping for palette/menu sectioning ("Select","Edit","View"...).
    pub category: String,
    /// Default key chord(s); see KeyChord. May be empty (palette-only).
    pub default_bindings: Vec<KeyChord>,
    /// Selection classes for which this command is ENABLED. Empty = always.
    /// Drives both right-click verb availability (G7) and palette greying.
    pub enabled_for_selection: Vec<SelectionClass>,
    /// How the command reaches the journal when it mutates.
    pub commit_kind: CommandCommitKind,
}

pub enum CommandCommitKind {
    /// Pure consumer-state action: never touches commit() (e.g. pan,
    /// toggle filter, open palette, start a tool).
    ConsumerState,
    /// Constructs an OperationBatch and calls commit() directly.
    DirectOp { op_family: String },
    /// Creates a Proposal that later calls the same commit() on accept
    /// (high-risk edits per Decision 002 OQ6 / master OQ5).
    Proposal { proposal_kind: String },
    /// Prefills the sanctioned terminal handoff conduit; reduces to a
    /// typed op on commit() in the launched session (Decision 005;
    /// terminal_command_catalog.rs). Never a private file write.
    TerminalHandoff { handoff_id: String },
}

pub struct KeyChord {
    /// Ordered key tokens, e.g. ["Ctrl","Shift","KeyR"] or ["KeyR"].
    /// First-press tools follow Altium single-key tool entry; chords
    /// follow the Ctrl/Shift/Alt convention already in terminal_input.rs.
    pub tokens: Vec<String>,
    /// True for multi-stroke sequences (e.g. "G" then "G"); modern
    /// pro-app chord pattern. Default false.
    pub is_sequence: bool,
}

pub enum SelectionClass {
    Empty,
    Pad, Track, Via, Zone, Component, BoardText, BoardGraphic, Net,
    Symbol, Wire, SchLabel, SheetPin, Junction,
    Mixed,
}
```

Determinism rules (master §6.1): the catalog serializes in byte-stable
`command_id` order; `SelectionClass` is the coarse `object_kind`
vocabulary already on every renderable primitive, so selection-class
gating reuses the existing identity triple rather than inventing a parallel
taxonomy. No field introduces design authority.

**Single op-builder rule (the wedge as a contract, not a claim).** A
`DirectOp { op_family }` MUST resolve to the SAME op-construction surface
the CLI sub-command and MCP tool of that family already call — the GUI
constructs no private `OperationBatch` builder. `op_family` is therefore a
key into the existing shared op vocabulary, not a GUI-local string. This
is the falsifiable form of §2.3.2: `it_palette_op_equiv_cli` (§12) asserts
the GUI-built batch is byte-identical to the CLI-built batch for the same
`op_family` + arguments. If they can diverge, the wedge is not real, so the
test is a gate, not a nicety.

### 4.2 HUD readout (scene-contract extension)

The heads-up display is the single most-glanced grammar element (every
gesture shows it), so it is a typed, versioned, golden-assertable struct,
not free-form text. It is consumer state, recomputed per cursor frame from
the active tool + `SnapState` + the resolved `DesignModel`; it is never
journaled. The renderer draws it as an in-canvas overlay near the cursor
(Altium HUD placement), not a status bar, so it is in the eye-line during
routing.

```rust
pub const HUD_READOUT_VERSION: &str = "datum.hud_readout.v1";

/// Cursor heads-up readout. Every field is optional so a golden can assert
/// the EXACT set shown for a given tool/state (e.g. routing shows net +
/// projected_drc; idle hover does not). All lengths in nm; angle in mdeg.
pub struct HudReadout {
    /// Absolute snapped cursor position (post-snap, not raw mouse).
    pub abs_x_nm: i64,
    pub abs_y_nm: i64,
    /// Relative to the active gesture anchor (route start / move pickup),
    /// None when no gesture is in progress. Altium relative-coord mode.
    pub rel_dx_nm: Option<i64>,
    pub rel_dy_nm: Option<i64>,
    pub rel_dist_nm: Option<i64>,
    pub rel_angle_mdeg: Option<i32>,
    /// Active in-progress segment length + width (routing/draw).
    pub segment_len_nm: Option<i64>,
    pub segment_width_nm: Option<i64>,
    /// Active layer the tool will commit to.
    pub active_layer_id: String,
    /// What the cursor snapped to this frame (mirrors SnapResolution.source).
    pub snap_source: Option<String>,
    /// Net under the cursor / being routed, resolved over the one
    /// DesignModel (wedge §2.3.3) — net NAME for display, identity for logic.
    pub net_name: Option<String>,
    /// Projected DRC of the in-progress geometry (routing) — mirrors the
    /// ghost's GhostDrcState so HUD and ghost can never disagree (G6).
    pub projected_drc: Option<GhostDrcState>,
}
```

Contract: when a tool computes `projected_drc`, the HUD field and the
`GhostPrimitive.projected_drc` (§7) MUST be the same value on the same
frame — the HUD is a textual view of the ghost, never a second source of
truth. `grammar_hud_hover.png` (exact golden) pins the idle-hover field
set; `grammar_route_ghost_violation.png` pins the routing field set
including the red `Violation` net + DRC readout.

### 4.3 Default keymap (initial, owner-overridable)

Single-key tool entry (Altium-style), active when the canvas has focus
and no text field is capturing:

| `command_id` | Default chord | Commit kind |
|---|---|---|
| `tool.select` | `Esc` (also returns to select) | ConsumerState |
| `tool.route.start` | `KeyR` | ConsumerState (route in-progress) |
| `tool.place.component` | `KeyP` | ConsumerState (place in-progress) |
| `tool.draw.line` | `KeyL` | ConsumerState |
| `tool.zone.draw` | `KeyZ` | ConsumerState |
| `edit.move` | `KeyM` | DirectOp `PlacementOps` |
| `edit.rotate` | `Space` (while moving), `KeyR` (when not routing) | DirectOp |
| `edit.delete` | `Delete` / `Backspace` | DirectOp / Proposal (destructive: OQ) |
| `select.cycle_under_cursor` | `Tab` (no move) | ConsumerState |
| `select.same_net` | `KeyN` then click, or RMB verb | ConsumerState |
| `select.connected_copper` | `Ctrl+KeyH` | ConsumerState |
| `select.all` | `Ctrl+KeyA` | ConsumerState |
| `select.filter.toggle` | `Shift+KeyC` | ConsumerState |
| `view.palette.open` | `Ctrl+KeyP` (also `F1`-class) | ConsumerState |
| `view.zoom.fit` | `KeyF` | ConsumerState |
| `edit.undo` | `Ctrl+KeyZ` | ConsumerState (journal cursor move) |
| `edit.redo` | `Ctrl+Shift+KeyZ` / `Ctrl+KeyY` | ConsumerState |

Reuse note: chord parsing reuses the `ModifiersState` + `PhysicalKey`
model already present in `crates/gui-app/src/terminal_input.rs`. Canvas
keybindings and terminal-lane keybindings are dispatched by which lane
holds focus; the existing `terminal_key_action` governs the terminal
lane unchanged, and the canvas grammar governs the canvas lane.

---

## 5. Selection Model (scene-contract + consumer-state)

Selection is consumer state (G4). It is a set of identity triples plus a
derived render mode. It is never serialized into a shard.

```rust
/// Consumer-state selection set. Lives in the GUI, never journaled.
pub struct SelectionState {
    /// Selected objects by identity triple (object_id is the key;
    /// object_kind + source_object_uuid carried for cross-probe).
    pub selected: Vec<SelectedRef>,
    /// Hover target (at most one); distinct render treatment from select.
    pub hovered: Option<SelectedRef>,
    /// Active selection filter: which classes are pickable.
    pub filter: SelectionFilter,
    /// Emphasis mode applied to the scene as a result of selection.
    pub emphasis: EmphasisMode,
}

pub struct SelectedRef {
    pub object_id: String,
    pub object_kind: String,      // SelectionClass vocabulary
    pub source_object_uuid: String,
}

pub struct SelectionFilter {
    /// Class -> pickable. A class set false cannot be hit-tested/box-hit.
    /// Mirrors Altium Selection Filter. Default: all true.
    pub pickable: BTreeMap<String /*object_kind*/, bool>,
}

pub enum EmphasisMode {
    /// No emphasis; full-color scene.
    None,
    /// Selected/related rendered normal, everything else DIMMED.
    /// Matches Altium net-highlight dim ("mask") behavior.
    DimOthers,
    /// Selected rendered HIGHLIGHTED over a normal scene (no dim).
    HighlightOnly,
}
```

The renderer consumes `SelectionState` as an overlay channel on the
existing scene: it does not require regenerating `BoardReviewSceneV1`,
because selection is consumer state. Highlight and dim are render
treatments keyed by `object_id` membership, not new primitives.

### 5.1 Picking verbs

- **Click** — pick the front-most pickable object under the cursor
  (respecting the filter). "Front-most" is deterministic, not z-buffer
  ambiguous: resolve by (1) smaller hit-test area wins (a via over the
  zone it sits in), then (2) active layer over inactive, then (3) stable
  `object_id` order as the final tiebreaker — so the same cursor pixel
  always picks the same object across runs and is golden-assertable.
  Click empty space clears selection.
- **Pick cycling** — when multiple pickable objects overlap the cursor,
  repeated clicks without moving (or `Tab`) cycle through the overlap
  stack in the deterministic order above (Altium/Allegro overlap cycling).
  The HUD shows "n of m" so the designer knows the stack depth. Required
  on dense multi-layer boards; a single "topmost" rule is a KiCad-floor
  behavior we exceed.
- **Shift+Click** — add/remove from set (toggle).
- **Box select** — drag a rectangle; left-to-right = enclosed-only,
  right-to-left = crossing (Altium/Allegro convention). Honors filter.
- **Lasso** — modifier-drag a freeform polygon (`Alt`+drag); enclosed-only.
- **Select same net** (`select.same_net`) — replace/extend selection with
  every object sharing the picked object's net identity, resolved over the
  one `DesignModel` (wedge §2.3.3), not a net-name string match.
- **Select connected copper** (`select.connected_copper`) — flood the
  galvanically connected copper graph (tracks/vias/pads/zones) from the
  picked object; matches Altium Connected Copper. Distinct from same-net:
  same-net includes unrouted/other-island members, connected-copper is
  the physically-joined island only.
- **Select all** (`select.all`) — all pickable objects under the filter.

### 5.2 Selection filter

A toggle bar plus `select.filter.toggle` cycles a saved filter. A class
set non-pickable is excluded from click, box, lasso, and select-all. This
matches the Altium Selection Filter exactly and is required before box
select on a dense board is usable. The filter is consumer state; it
persists in the workbench session, never in the design.

### 5.3 Highlight vs dim

Two emphasis modes (above). `DimOthers` is the default for
select-same-net and cross-probe (it reads like Altium's net mask);
`HighlightOnly` is the default for a plain click. The owner question on
default emphasis per gesture is OQ4.

---

## 6. Snapping, Grid, Guides

Snapping is consumer state computed each cursor move; it feeds the HUD
and the ghost.

```rust
pub struct SnapState {
    /// Active snap grid in nm (0 = off). Geometry units are nm (master §6.1).
    pub grid_nm: i64,
    /// Which snap targets are enabled.
    pub targets: SnapTargets,
    /// The resolved snap result for the current cursor frame (consumer
    /// state, recomputed per move; never journaled).
    pub resolved: Option<SnapResolution>,
}

pub struct SnapTargets {
    pub grid: bool,
    pub pad: bool,
    pub track_end: bool,
    pub track_midpoint: bool,
    pub via: bool,
    pub pin: bool,          // schematic pin endpoints (SCH_EDITOR §6.1)
    pub guide: bool,        // user/auto alignment guides
    pub object_origin: bool,
}

pub struct SnapResolution {
    pub world_x_nm: i64,
    pub world_y_nm: i64,
    /// What the cursor snapped to, for HUD readout + golden assertion.
    pub source: SnapSource,        // Grid | Pad | TrackEnd | Pin | Guide | ...
    /// The identity of the snapped object if any (for select/route intent).
    pub target_ref: Option<SelectedRef>,
}
```

Rules:
- **Deterministic snap precedence (pinned default).** When more than one
  enabled target is within snap radius of the cursor, resolution is by a
  fixed precedence so a route golden is reproducible:
  `Pad/Pin (1) > TrackEnd (2) > Via (3) > TrackMidpoint (4) > Guide (5) >
  ObjectOrigin (6) > Grid (7)`. Ties within a class break by nearest, then
  stable `object_id`. The precedence is owner-tunable (OQ7) but a default
  is pinned here because goldens cannot be deterministic against an
  unspecified order. Snap radius is a consumer-state pixel threshold scaled
  by zoom (Altium snap-distance), independent of `grid_nm`.
- Snap targets follow the schematic editor's attachment rules
  (`SCHEMATIC_EDITOR_SPEC.md §6.1`): wire endpoints snap to pins,
  junctions, wire endpoints, port anchors, bus-entry anchors. The board
  side adds pad/track-end/via/guide.
- The resolved snap source is surfaced in the HUD and is a golden
  assertion target (a route golden asserts the cursor snapped to the pad
  center, not "near").
- **Guides**: alignment guides appear when a dragged/placed object's
  edge or center aligns with another object (Altium guides). Guides are
  consumer state, render as thin overlay lines, and provide a snap
  target when `targets.guide` is on.

---

## 7. Ghost Preview Contract (scene-contract extension)

The single most fluency-critical element and the bridge to the AI canvas
(§2.3.4). The ghost is the in-progress, uncommitted geometry a tool draws
on the cursor frame. It is consumer state and a versioned scene-contract
companion primitive.

```rust
pub const GHOST_PREVIEW_VERSION: &str = "datum.ghost_preview.v1";

/// Uncommitted preview overlay. Rendered above committed geometry with a
/// distinct ghost treatment. Carries the identity triple so an accepted
/// ghost commits to the same object identity (and so the AI canvas can
/// reuse this primitive for proposal diffs — master differentiator 1).
pub struct GhostPrimitive {
    pub object_id: String,        // synthetic until commit, then stable
    pub object_kind: String,      // SelectionClass vocabulary
    pub source_object_uuid: String, // empty for net-new until commit
    pub layer_id: String,
    /// The would-be geometry, in nm, in the same shapes the committed
    /// primitives use (track polyline, pad rect, footprint outline...).
    pub geometry: GhostGeometry,
    /// Origin of the ghost, so the renderer + golden can distinguish a
    /// manual tool preview from an agent proposal using ONE primitive.
    pub origin: GhostOrigin,
    /// Projected check state for this preview, if the tool computes it
    /// (matches Altium online-DRC-while-routing). Renderer colors the
    /// ghost by severity.
    pub projected_drc: Option<GhostDrcState>,
}

pub enum GhostOrigin {
    /// Drawn by the user's active tool (route/place/draw/move).
    Tool,
    /// Drawn from an agent Proposal (the AI-native canvas reuses this).
    Proposal { proposal_id: String },
}

pub enum GhostGeometry {
    Track { path: Vec<PointNm>, width_nm: i64 },
    Via { center: PointNm, drill_nm: i64, diameter_nm: i64 },
    FootprintOutline { origin: PointNm, rotation_mdeg: i32, outline: Vec<PointNm> },
    Zone { boundary: Vec<PointNm> },
    Wire { from: PointNm, to: PointNm },     // schematic
    Move { delta_x_nm: i64, delta_y_nm: i64, refs: Vec<SelectedRef> },
}

pub enum GhostDrcState { Clean, Warning, Violation }
```

Contract (G6): the ghost geometry MUST be exactly what the tool's commit
transition would produce. If the tool cannot legally commit at the current
frame, the ghost shows `projected_drc: Violation` (red), not a hidden
"snap will fix it" — the preview is honest. A golden can therefore assert
that a violating route renders red and committing it produces the same
finding the ghost predicted.

This is shared with `GUI_AI_SURFACES.md`: the AI canvas's
proposal diff is `GhostPrimitive` with `origin: Proposal`. The renderer
has one ghost path; the difference is treatment (tool ghost vs proposal
diff) and the accept/reject affordance, not a second primitive type.

---

## 8. Undo/Redo Granularity (mapped to the journal)

Undo/redo is the grammar's clearest wedge (§2.3.1). Rules:

- **U1 — One gesture, one batch, one undo step (G5).** A user-perceived
  action = one `OperationBatch` on one `commit()` = one journal entry =
  one undo step. The state machines below define exactly which transition
  fires `commit()`, and there is exactly one such transition per gesture.
- **U2 — Multi-part gestures are still one batch.** A route from pad A to
  pad B placing 3 segments + 1 via is ONE batch (`RoutingOps`), one undo
  step. A box-move of 12 footprints is ONE batch (`PlacementOps`). This
  matches the commercial expectation (Altium routes undo as one stroke).
- **U3 — Undo/redo are consumer-state cursor moves, not ops.** `edit.undo`
  moves the journal cursor back; it appends nothing. Therefore undo/redo
  themselves are never journaled and never appear as design history noise.
- **U4 — Durable across reopen.** Because the cursor is over the journal,
  not an in-memory stack, close/reopen restores the undo cursor and the
  full redo tail (Decision 002 recovery proof). This is the explicit
  beat-Altium claim and is golden-tested by a close/reopen interaction
  test.
- **U5 — Cancel is not undo.** An Esc-cancelled in-progress gesture
  produced no batch, so there is nothing to undo and the redo tail is
  untouched (G2).

---

## 9. Command Palette (component spec)

Matches the Altium command runner / modern pro-app command palette.

- Opened by `view.palette.open` (`Ctrl+KeyP`); a single text input with a
  ranked result list under it. Fully keyboard driven (G3): arrows + Enter,
  Esc closes (consumer state, no commit).
- Results are `GuiCommandEntry` rows ranked by fuzzy match over `label` +
  `keywords`, sectioned by `category`. Disabled entries (selection class
  not in `enabled_for_selection`, §4) render greyed with the reason.
- **Deterministic ranking (required for the exact golden).** The rank key
  is a total order so `grammar_palette_ranked.png` is reproducible:
  (1) prefix-on-`label` matches before substring matches before
  subsequence matches; (2) shorter match span wins; (3) enabled before
  disabled; (4) stable `command_id` lexicographic order breaks all
  remaining ties. No score floats, no locale collation — an exact golden
  cannot tolerate a non-deterministic ranker. Empty query lists all
  enabled entries in `command_id` order.
- Selecting a `ConsumerState` entry runs it immediately. A `DirectOp`
  entry constructs the op batch and commits. A `Proposal` entry opens the
  proposal review surface. A `TerminalHandoff` entry prefills the terminal
  lane via the existing `TerminalCommandHandoff` conduit (Decision 005) —
  it does not write a shard.
- **Wedge:** the palette is the same op vocabulary as CLI/MCP (§2.3.2);
  a palette `command_id` maps to the same `op_family`/`handoff_id` the CLI
  and MCP expose, so a designer who learns the palette has learned the
  scriptable surface.

Non-goal for v1: palette macro recording / chaining (OQ6).

---

## 10. Interaction State Machines

States and transitions, not prose (master §6.2). Notation: `STATE
--event[guard]/effect--> STATE`. The single `commit()` transition is
marked `[[COMMIT]]`; the cancel transition is marked `[[CANCEL]]` and
guarantees zero committed mutation + zero dirty shard.

### 10.1 Shared tool lifecycle (every tool plugs into this)

```
Idle
  --command(tool.X)/set active_tool=X, init tool consumer-state--> Armed
Armed
  --Esc[[CANCEL]]/clear tool state, active_tool=select--> Idle
  --(tool-specific events; see per-tool machines below)--> (tool states)
```

`active_tool`, all in-progress geometry, snap, and ghost are consumer
state. Only the per-tool `[[COMMIT]]` transition appends to the journal.

### 10.2 Select tool (`tool.select`) — the default

```
SelectIdle
  --hover(move)/recompute hovered via hit-test+filter, update HUD--> SelectIdle
  --click(empty)/clear selected, emphasis=None--> SelectIdle
  --click(obj)[filter pickable]/selected={resolve front-most(overlap)},
       emphasis=HighlightOnly, HUD "1 of m"--> SelectIdle
  --click(same pixel) | Tab[m>1]/selected={next in overlap order},
       HUD "k of m"--> SelectIdle
  --shiftClick(obj)/toggle obj in selected--> SelectIdle
  --dragStart(empty)/begin box, anchor=cursor--> BoxSelecting
  --altDragStart/begin lasso--> LassoSelecting
  --command(select.same_net)[1 obj]/selected=net members, emphasis=DimOthers--> SelectIdle
  --command(select.connected_copper)[1 obj]/selected=copper island, emphasis=DimOthers--> SelectIdle
  --rightClick/open verb menu built from SelectionClass(selected)--> VerbMenu
BoxSelecting
  --drag(move)/update box rect + ghost rect, live-preview enclosed/crossing--> BoxSelecting
  --dragEnd/selected=hits(box, direction, filter), emphasis=HighlightOnly--> SelectIdle
  --Esc[[CANCEL]]/discard box--> SelectIdle
LassoSelecting
  --drag(move)/append polygon point + ghost--> LassoSelecting
  --dragEnd/selected=enclosed(poly, filter)--> SelectIdle
  --Esc[[CANCEL]]/discard lasso--> SelectIdle
VerbMenu
  --pick(verb)/dispatch GuiCommandEntry(verb)--> (that command's machine)
  --Esc[[CANCEL]]/close menu--> SelectIdle
```

No transition in the select tool reaches `commit()`: selection is pure
consumer state (G4). Verbs that mutate dispatch INTO another tool's
machine, which owns the commit.

### 10.3 Route tool (`tool.route.start`) — exemplar mutating tool

```
RouteArmed
  --click(pad/track-end)[snap.target]/
       net = DesignModel.net_of(snap.target_ref)  // identity, not name
       width = rule_width_for(net)                 // from net design rule
       layer = active_layer; start = snap; in_progress=[start]--> Routing
  --click(empty)[no net]/reject (HUD: "no net under cursor")--> RouteArmed
  --Esc[[CANCEL]]/active_tool=select--> SelectIdle
Routing
  --move/recompute snap; ghost=Track(in_progress + cursor, width, net),
       projected_drc; HUD net_name + segment_len + projected_drc--> Routing
  --click(point)[ghost.drc != Violation]/append vertex to in_progress--> Routing
  --click(point)[ghost.drc == Violation]/reject (HUD warns), no append--> Routing
  --key(toggleLayer)[layer change allowed]/place via in ghost (drill/dia
       from net rule), switch active layer, continue net--> Routing
  --key(cycleWidth)/width = next allowed width for net (HUD shows)--> Routing
  --Backspace/drop last in_progress vertex (rubber-band back)--> Routing
  --doubleClick | click(target pad)[snap && same net]/
       build OperationBatch(RoutingOps: segments+vias),
       commit() [[COMMIT]] /clear in_progress--> RouteArmed
  --click(target pad)[snap && DIFFERENT net]/reject (HUD: "net mismatch"),
       no commit--> Routing
  --Esc[[CANCEL]]/discard in_progress + ghost, keep prior commits--> RouteArmed
```

The net is resolved from the start object's identity over the one
`DesignModel` (wedge §2.3.3), never from a refdes/net-name join, and the
default width comes from that net's design rule — so the ghost and HUD
show the real net the moment routing starts (Altium routes on the pad's
net, width from the net rule). One commit per completed route (U2): the
whole multi-segment, multi-via stroke is one `OperationBatch`, one undo
step. The ghost's `projected_drc` blocks committing a violating segment
(G6) — matching Altium online-DRC-while-routing, where the tool refuses
an illegal click. Terminating on a different net is refused, not silently
shorted (a class of error Altium's net-aware routing also prevents).

### 10.4 Place tool (`tool.place.component`)

```
PlaceArmed
  --command(choose item)/load footprint outline into ghost--> PlaceGhosting
  --Esc[[CANCEL]]/active_tool=select--> SelectIdle
PlaceGhosting
  --move/ghost=FootprintOutline at snap(cursor)--> PlaceGhosting
  --key(rotate)/ghost.rotation += step--> PlaceGhosting
  --click[snap]/build OperationBatch(PlacementOps:PlaceFootprint),
       commit() [[COMMIT]]--> PlaceGhosting   // stays armed for next place
  --Esc[[CANCEL]]/clear ghost--> PlaceArmed
```

Repeated placement stays armed (Altium continuous-place); each placed
instance is its own one-step commit.

### 10.5 Move (`edit.move`) — operates on the live selection

```
MoveIdle (entered with non-empty selection)
  --command(edit.move) | dragStart(on selected)/
       ghost=Move(delta=0, refs=selected), pick up--> Moving
Moving
  --move/ghost.delta = snap(cursor) - anchor; guides recompute--> Moving
  --key(rotate)/apply rotation to ghost--> Moving
  --drop | click/build OperationBatch(PlacementOps: MovePlacement[]),
       commit() [[COMMIT]]--> MoveIdle
  --Esc[[CANCEL]]/discard ghost, objects unmoved--> MoveIdle
```

A multi-object move is one batch (U2). Cancel leaves objects exactly where
committed (G2/U5).

### 10.6 Command palette (`view.palette.open`)

```
PaletteClosed
  --command(view.palette.open)/open input, query="", results=all enabled--> PaletteOpen
PaletteOpen
  --type(ch)/query += ch; results = rank(catalog, query, SelectionClass)--> PaletteOpen
  --arrow/move highlight--> PaletteOpen
  --Enter[entry.ConsumerState]/run entry--> PaletteClosed
  --Enter[entry.DirectOp]/build OperationBatch, commit() [[COMMIT]]--> PaletteClosed
  --Enter[entry.Proposal]/open proposal review surface--> PaletteClosed
  --Enter[entry.TerminalHandoff]/prefill terminal via TerminalCommandHandoff--> PaletteClosed
  --Esc[[CANCEL]]/close, discard query--> PaletteClosed
```

### 10.7 Pan/Zoom (`view.*`) — never commits

```
Viewing
  --scroll/zoom about cursor (consumer camera state)--> Viewing
  --middleDrag | space+drag/pan camera--> Viewing
  --command(view.zoom.fit)/camera = fit(bounds)--> Viewing
```

Camera is consumer state (G4); no view action ever touches `commit()`.

---

## 11. Proof Slices

Each names the `datum-test` regression fixture (default) and its gates.
Supervision-reflection precedes interaction (master §4.3): the read-only
HUD/selection display ships before any mutating tool.

- **PS-GRAMMAR-1 (supervision, read-only).** Open `datum-test`. Hover a
  pad; the HUD shows live X/Y and the snapped object's identity; click it;
  the renderer highlights exactly that `object_id`; `select.same_net`
  selects every member of that net resolved over the one `DesignModel`
  and dims the rest. Gates: renders real committed state; no `commit()`
  call; selection is consumer state (assert nothing appended to journal).
- **PS-GRAMMAR-2 (filter + box).** Set the selection filter to pads-only;
  box-select a region; assert only pads were picked despite tracks/vias in
  the box (matches Altium filter). Gates: filter honored by box and
  select-all; consumer state only.
- **PS-GRAMMAR-3 (ghost honesty).** Arm route; draw a segment that
  violates clearance; assert the ghost renders `projected_drc:
  Violation` (red) and the commit transition is refused at that frame;
  reroute legally and double-click to commit. Gates: one
  `OperationBatch` on `commit()`; the committed finding matches the
  ghost's prediction (G6).
- **PS-GRAMMAR-4 (undo durability — the wedge).** Route a 3-segment
  net (one batch), move 5 footprints (one batch); assert exactly two undo
  steps. Undo once, redo once. Close and reopen the project; assert the
  undo cursor and redo tail are restored (U4; Decision 002 recovery
  proof). Gate: durable journal-cursor undo, not an in-memory stack.
- **PS-GRAMMAR-5 (palette = op vocabulary).** Open the palette, run a
  `DirectOp` entry and a `TerminalHandoff` entry; assert the `DirectOp`
  built the same `op_family` the CLI sub-command builds, and the handoff
  prefilled the terminal via `TerminalCommandHandoff` without writing a
  shard (Decision 002 GUI-bypass proof; Decision 005). Gate: no private
  file write; palette action replayable.

---

## 12. Visual-Golden Acceptance

Every grammar surface ships a golden + interaction test in the
`gui-render` harness (master §4.4 / §6.5). Acceptance = the golden
renders real committed state from `datum-test` and the interaction test
exercises the named state-machine transitions.

| Golden | Drives | Asserts | Diff |
|---|---|---|---|
| `grammar_hud_hover.png` | PS-GRAMMAR-1 | HUD readout + hovered object treatment on a real pad | exact |
| `grammar_select_same_net_dim.png` | PS-GRAMMAR-1 | same-net set highlighted, others dimmed (`DimOthers`) | tolerance |
| `grammar_selection_filter_box.png` | PS-GRAMMAR-2 | only pads selected under pads-only filter | exact |
| `grammar_route_ghost_clean.png` | PS-GRAMMAR-3 | clean route ghost over committed copper, distinct treatment | tolerance |
| `grammar_route_ghost_violation.png` | PS-GRAMMAR-3 | violating ghost rendered red (`GhostDrcState::Violation`) | tolerance |
| `grammar_move_ghost_guides.png` | 10.5 | multi-object move ghost with alignment guides | tolerance |
| `grammar_palette_ranked.png` | PS-GRAMMAR-5 | palette open, ranked + greyed-by-selection rows | exact |

Interaction tests (non-image, drive the state machines):
- `it_route_one_batch_one_undo` — §10.3 produces exactly one batch / one
  undo step for a multi-segment route (U2).
- `it_undo_durable_reopen` — PS-GRAMMAR-4 close/reopen restores cursor +
  redo tail (U4).
- `it_cancel_zero_mutation` — every `[[CANCEL]]` leaves zero committed
  mutation and zero dirty shard (G2/U5).
- `it_filter_honored_all_picks` — filter excludes class from click/box/
  lasso/select-all (§5.2).
- `it_palette_op_equiv_cli` — a palette `DirectOp` builds a byte-identical
  `OperationBatch` to the CLI path for the same `op_family` + args (§4
  single op-builder rule / §2.3.2).
- `it_pick_cycle_deterministic` — repeated click / `Tab` on an overlapping
  via+zone+track stack cycles in the fixed front-most order across runs and
  reports a stable "k of m" (§5.1); same pixel always yields the same
  sequence.

---

## 13. Non-Goals

- Owning canvas rendering quality, layer compositing, or per-domain
  geometric tool detail — those belong to
  `GUI_CANVAS_AND_RENDERING.md`. This spec owns the SHARED grammar.
- Push-and-shove / plow interaction semantics (the route tool here is the
  modeless rubber-band exemplar; shove physics is a later canvas-spec
  increment, Decision 002 non-goal).
- The AI proposal review surface itself — this spec only defines the
  shared `GhostPrimitive` it reuses; review lives in
  `GUI_AI_SURFACES.md`.
- The cross-probe MODEL itself (the N-way identity-resolution and per-pane
  emphasis-broadcast coordination across schematic/PCB/3D/BOM/CAM/findings)
  — owned by the dedicated marquee spec `GUI_CROSS_PROBE.md`. Cross-probe
  view composition (tabs/PiP/split) lives in
  `GUI_INFORMATION_ARCHITECTURE.md`. THIS spec defines select-same-net and
  the cross-probe selection IDENTITY (`SelectionState`, `SelectedRef`,
  `EmphasisMode`) — the selection primitives cross-probe builds on.
- Full workbench-profile persistence, multi-monitor, floating windows
  before the core grammar is credible (master §9; 002/012).
- Palette macro recording/chaining, custom user scripting of chords
  beyond rebinding (v1 non-goal; OQ6).
- Live real-time DRC recomputation on every keystroke for the whole board
  (master §9); the route ghost computes LOCAL projected DRC only.
- Editing `specs/PROGRESS.md`, `specs/SPEC_PARITY.md`, `crates/`, or
  `mcp-server` from this authoring track.

---

## 14. Open Questions

1. **Keymap source of truth.** Should the default keymap follow Altium
   conventions as closely as licensing/recognizability allows, a neutral
   modern-pro-app scheme, or fully owner-defined from scratch? (Affects
   the §4.3 default keymap table.)
2. **Destructive deletes.** Does `edit.delete` commit directly or land as
   a Proposal for high-value objects (nets, zones, imported geometry)?
   Ties to master OQ5 / Decision 002 OQ6.
3. **Right-click verb scope.** Does the RMB menu list only commit-valid
   verbs for the exact selection (strict G7), or also show greyed
   not-yet-valid verbs with reasons (more discoverable, more clutter)?
4. **Default emphasis per gesture.** Is `DimOthers` the right default for
   plain click, or only for same-net/cross-probe (with click =
   `HighlightOnly`)? Affects every selection golden.
5. **Selection persistence across projection switch.** When the user
   cross-probes SCH->PCB, does the selection set carry by identity
   automatically (wedge §2.3.3), and does emphasis mode carry with it?
   Coordinate with `GUI_CROSS_PROBE.md` (the cross-probe model + OQ4),
   `GUI_INFORMATION_ARCHITECTURE.md` (view composition), and
   `GUI_CANVAS_AND_RENDERING.md` (cross-probe rendering).
6. **Palette mutating scope for v1.** Should the v1 palette expose
   mutating `DirectOp`/`Proposal` entries at all, or be read-only +
   navigation + `TerminalHandoff` first (supervision-first, master §4.3),
   with direct mutation arriving with the interactive editor phase?
7. **Snap precedence tuning.** §6 pins a default precedence
   (`Pad/Pin > TrackEnd > Via > TrackMidpoint > Guide > ObjectOrigin >
   Grid`) so goldens are deterministic. Open: does the owner want this
   exact order shipped, and should precedence be per-tool (e.g. routing
   prefers TrackEnd, placement prefers Pad)? The default holds until
   overridden; this OQ is a tuning question, not a blocking gap.
8. **Numeric input during a gesture.** Should routing/placement accept
   typed coordinate/length entry mid-gesture (Altium-style numeric entry),
   and if so does that text capture suspend the single-key tool keymap?
