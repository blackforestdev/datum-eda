# Datum Universal Viewport & Editor-Interaction Toolkit Spec

Status: governed spec (active)

Governed by **decision `PRODUCT_MECHANICS_023_UNIVERSAL_VIEWPORT_TOOLING`** (the
ratified law). This document is the *how*: it may strengthen but not weaken 023.
It **extends** — and does not restate — the existing surface it plugs into:
`docs/gui/DATUM_GUI_DESIGN_SPEC.md` (Application Status Bar readout), `DATUM_RENDERING_BOOK.md`
(stroke hierarchy, grid colour, filled-outline text, selection-is-screen-only),
`DATUM_GUI_PARAMETRIC_TOOLING.md` (tri-modal verbs, align `reference: grid`),
`DATUM_GUI_CONTEXT_MENU_CONTENT.md` + `research/gui-context-menus/CONTEXT_MENU_RESEARCH.md`
(the menu content/form), `docs/gui/menu_model.json` (the data-driven, gated menus),
and decisions 014/020/021/022.

Each normative claim below carries an honest **check disposition** per the
`DATUM_GUI_CONFORMANCE_SPEC.md` discipline: **ENFORCED** (an existing gate/test/
golden already locks it), **TO-ENFORCE** (a named gate to add with the implementing
slice), or **HUMAN** (reference-image / eyeball review).

---

## 1. Architecture

### 1.1 The `EditorViewport` keystone

Every drawing surface is an `EditorViewport` (never a bare "viewport" —
decision-021 workspace pane and decision-020 paper-space viewport are distinct).
Screen↔world projection and hit-testing resolve in the target surface's own
camera/space. Pointer preview, wheel zoom, and `Space`+primary-button drag-pan
target the pane containing the pointer; keyboard/menu commands and active-tool
gestures target the focused pane. A focus-changing click continues dispatch in
that pane. Camera state is
keyed by `(PaneId, surface/document identity)`, so duplicate views remain
independent and content replacement cannot inherit another coordinate space.

`Space`+primary-button drag is the exclusive 2D pan activation on every drawing
surface. `Space` MUST already be held when the primary button is pressed; it MUST
NOT steal an in-progress selection or authoring drag. Right-click/right-drag is
reserved exclusively for the local/marking menu (§6), and the middle button is
reserved for 3D-view rotation rather than 2D pan. Pan ends on primary-button
release, `Space` release, `Escape`, focus/window deactivation, or pointer-capture
loss; every termination clears capture and suppresses click, selection, and tool
commit for that pan gesture. Text-entry focus owns `Space` normally and MUST NOT
arm pan. A `Space` tap without a primary drag remains available to the focused
active tool (including route-corner rotation); key repeat MUST NOT synthesize a
tap command.

*Disposition: TO-ENFORCE — routing/lifecycle tests with distinct surface bounds,
duplicate panes, content replacement, missing scenes, and same-click dispatch;
board frame stays byte-identical (visual-parity).*

### 1.2 Render-approach law (UVT-003)

Authored, fab-bearing geometry renders from the retained world (nm) buffer.
Grid, selection, hover, cursor, snap feedback, and menu render as an **immediate
screen-space overlay** driven by the live camera — on every surface. Baking
presentation chrome into a surface's world buffer is prohibited (that is the
defect that made the schematic grid scale with zoom). Preserves `render == CAM`.

*Disposition: ENFORCED for board and schematic grid/hover/cursor. Interaction
chrome has a dedicated post-world overlay on both surfaces, and the retention
regression asserts pointer refresh does not resolve retained geometry or disturb
the schematic grid buffer. HUMAN remains for zoom/readability evaluation.*

### 1.3 `ViewportProfile` (the per-surface config)

A surface is one `ViewportProfile`, bundling small config structs — never new
mechanism:

```
ViewportProfile {
    grid:      GridConfig,        // pitch table, mode (square/rect), origin, colours
    camera:    CameraConfig,      // warm-camera slot, bounds source
    snap:      SnapConfig,        // registered SnapTarget kinds, running-snap defaults
    stroke:    StrokeConfig,      // primitive -> weight-class map (see §4)
    hover:     HoverConfig,       // hoverable classes
    select:    SelectionConfig,   // selectable classes, selection visuals
    tools:     ToolSet,           // per-surface tool enum + keymap
    menu:      MenuKeyNamespace,  // "pcb.*" | "schematic.*" (already in menu_model.json)
    readout:   ReadoutConfig,     // units, precision, polar
    layers:    LayerSet,          // the surface's layer/visibility set
}
```

*Disposition: PARTIAL — grid, stroke, hover, and cursor configuration plus the
shared interaction state mechanism live in `gui-viewport`; camera, snap,
selection, tool, menu, readout, and layer configuration remain staged work.*

### 1.4 Crate boundary (UVT-002)

The shared mechanism lives in a new consumer-side crate **`gui-viewport`**
(depends on `gui-protocol` i64-nm types; the engine, daemon, protocol, and
persisted formats never depend on it — a compile-time fence, decision-014
precedent). Input-event wiring stays in `gui-app`; render batching stays in
`gui-render`.

*Disposition: TO-ENFORCE — a dependency-direction check (engine/daemon/protocol
have no edge to `gui-viewport`).*

---

## 2. The engine set (mechanism vs config)

One shared mechanism each; the per-surface variation is only the `…Config`.

| Engine | Shared mechanism | Per-surface config |
|---|---|---|
| **CoordinateHit** (keystone) | per-pane screen↔world + hit-test over world hit-regions; point-in-poly/polyline predicates | hit-region set, bounds |
| **GridEngine** | adaptive-LOD screen-space axis rects via live camera; CROSS/DOT/LINES as one `mark_size` knob; origin (§5) | pitch table, mode, colours |
| **CameraEngine** | existing `CameraState`+`Projection`; zoom-to-cursor, fit=zoom-to-selection, drag-pan; one routing path | warm-camera slot, bounds source |
| **StrokeWeightModel** | the three weight classes + projection/floor math (§4) | primitive→class map |
| **HoverEngine + cursor** | per-surface hover from CoordinateHit; crosshair; snapped-cursor glyph | hoverable classes |
| **Selection + Marquee** | per-surface single + rubber-band selection | selectable classes, visuals |
| **ToolModeEngine** | per-focused-editor active tool/mode; toolbar hit regions; per-editor keymap routing | tool set + keymap |
| **ContextMenuEngine** | per-surface content from hit+selection+profile; multi-select intersection; overflow + nested wheels; **verb execution on leaf** (§6) | menu-key namespace |
| **CoordinateReadout** | cursor X/Y→units, dx/dy vs settable origin; focused-editor status fields (§7) | units, precision, polar |
| **SnapEngine** | 2-tier ordered-scan resolver; SnapTarget registry; SnapFilter (§3) | Target kinds, defaults |
| **LayerVisibility** | per-surface layer toggle → world-range filter | layer set |

### 2.1 Work budgets and hit eligibility

- Grid generation MUST inverse-project the visible pane, use overflow-safe
  iteration, and emit at most **16,384 marks/lines per pane per frame**.
- CoordinateHit MUST use a retained spatial index. Pointer queries MUST examine a
  deterministically bounded candidate set; full O(n) scans are prohibited.
- Schematic hit metadata MUST be typed, not inferred from identifier prefixes,
  for symbols, pins, wires, buses, labels, junctions, and no-connect markers.
  Hittability is distinct from whether the active tool permits selection.
- Timing benchmarks on representative large designs supplement deterministic
  work gates; wall-clock timing alone MUST NOT be the CI correctness oracle.

### 2.2 S5 selection and marquee contract (owner-ratified, design in progress)

This subsection is the durable working authority for S5. It records only
owner-ratified behavior while the section-by-section design review is in
progress; unresolved behavior remains explicitly open until the final review.
S5 implementation MUST NOT begin from this incomplete subsection before that
review closes it.

Research basis and derived classification:
`research/gui-compound-selection/GUI_COMPOUND_SELECTION_RESEARCH.md` →
`DATUM_SELECTION_COMPOUND_EDITING_GUIDANCE.md`. DNP in that material is one
illustrative compound attribute, not a privileged or exhaustive edit surface.

#### 2.2.1 Normal selection and clearing

- Primary-clicking an eligible object selects it. Primary-clicking another
  eligible object replaces the selection with that object.
- A primary click in an unfocused pane focuses that pane and performs the
  selection in the same click.
- Primary-clicking empty canvas preserves the selection. Datum MUST NOT require
  the user to find or reveal blank canvas in order to leave selection.
- `Escape` is the sole explicit clear-selection command. If a temporary gesture
  or operation is active, the first `Escape` cancels that operation while
  preserving the prior selection; a subsequent `Escape`, with no gesture or
  operation active, clears the complete selection.
- Selection references stable authored identity rather than an incidental
  rendered primitive identity and drives the Inspector and cross-probe
  projection.

#### 2.2.2 Add, remove, and region gesture grammar

Datum has one modifier meaning per selection action; synonymous modifier paths
are prohibited:

- plain primary click replaces with one object;
- `Shift`+primary click adds one object; applying it to an already-selected
  object leaves that object selected;
- `Ctrl`+primary click removes one object; applying it to an unselected object
  is a no-op;
- `Shift`+primary drag opens an additive selection region. With no prior
  selection it creates the selection; otherwise it extends the selection;
- `Ctrl`+primary drag opens a subtractive selection region and removes matching
  members from the prior selection; and
- plain primary drag does not open a selection region; it is **reserved** for
  direct object manipulation under the S5B movement contract (§2.2.12) — in
  read-only S5A the gesture binds no object-manipulation behavior.

`Shift`+`Ctrl` is not a third selection operation. `Escape` clears the complete
selection rather than removing one member at a time.

Selection-region activation uses a **4 physical-device-pixel** movement
threshold. Below the threshold the input remains a modified click; at or beyond
the threshold it becomes a region gesture. The initial direction locks the
region shape for the gesture: rightward motion creates a rectangle and leftward
motion creates a freeform lasso. The shape MUST NOT switch mid-gesture. Releasing
primary commits the region result. `Escape`, focus loss, capture loss, pane
closure, or content replacement cancels the gesture and preserves the prior
selection.

`Space` held before primary press owns pan and prevents region activation.
Pressing `Space` after a region gesture begins MUST NOT steal it. A region
gesture remains owned by its originating pane.

#### 2.2.3 Region feedback and selection auto-pan

An active rectangle or lasso renders a temporary high-contrast animated dashed
"dancing ants" boundary in the immediate screen-space overlay. The boundary is
one physical device pixel, is clipped to its originating pane, has no persistent
occluding fill, and disappears immediately on commit or cancellation. Under a
reduced-motion preference the boundary remains dashed but does not animate.
Additive and subtractive gestures MUST be visibly distinguishable without
depending on animation alone. Region feedback is consumer/session state: it is
never journaled, persisted, exported, or emitted into manufacturing output.

Dragging an active region into the **24 physical-device-pixel** edge band starts
selection auto-pan. Speed increases toward and beyond the edge; exact
acceleration and maximum speed are implementation tuning values, not alternate
gesture grammar. Auto-pan moves only the originating pane's camera, supports
diagonal motion at corners, stops when the pointer returns inside the edge band,
and does not transfer focus or gesture ownership to an adjacent pane. The
rectangle/lasso remains anchored in world space so geometry revealed by
auto-pan participates in the final result. Cancellation stops auto-pan and
preserves the prior selection.

#### 2.2.4 Region qualification and workspace granularity

Region qualification uses a strict **greater-than-50-percent** rule. Exactly
50 percent is a non-selection. This is the fixed S5-v1 rule, not a preference;
later tuning requires usability evidence and a governed spec change.

- A PCB footprint with pads qualifies only when a strict majority of its pad
  center anchors lie inside the rectangle/lasso. Thus an SOIC-8 requires five
  pads; three or four do not qualify. A padless footprint falls back to its
  placement anchor. Silkscreen, courtyard, fabrication graphics, and
  reference/value text do not enlarge this test.
- A schematic symbol with pins qualifies only when a strict majority of its pin
  connection anchors lie inside the rectangle/lasso. Thus a 14-pin symbol
  requires eight pins. A pinless symbol falls back to its placement anchor;
  symbol graphics and text do not enlarge this test.
- In the board workspace, clicking a pad selects its parent footprint and pad
  anchors contribute to parent-footprint region selection. In the footprint
  editor workspace, a pad is independently selectable.
- **Component-owned graphics (owner-resolved, S5-C01A / OPEN-2, 2026-08-14):**
  in the board workspace, clicking a footprint's owned silkscreen, courtyard,
  or fabrication graphic selects the **parent footprint** — the same ownership
  collapse as pads (and graphics still never enlarge the region test).
  Footprint-contributed **Edge.Cuts geometry carries no hit region**: board
  outline clicks always resolve to the board-level outline authority, never to
  a component. Child-level graphic selection is definition-editor authority.
- In the schematic workspace, clicking a pin selects its parent symbol and pin
  anchors contribute to parent-symbol region selection. In the symbol editor
  workspace, a pin is independently selectable.
- **Definition-editor child qualification and composition (owner-resolved,
  S5-C01A / OPEN-4, 2026-08-14):** in the Footprint Editor a pad region-
  qualifies when its **center anchor** lies inside the rectangle/lasso; in the
  Symbol Editor a pin region-qualifies when its **connection anchor** lies
  inside — one anchor qualifies the complete compound pin subject
  (stub/terminal/name/number). Both follow the anchor family: no per-class
  area tests, and name/number text never enlarges the test. **Pad geometry
  plus pad-number text project as one pad subject** (the RB §2.7 pin
  construction analogy); pad-number text is therefore excluded from the
  owned-text class, exactly as pin name/number text is.
- **Non-authored subjects (owner-resolved, S5-C01A / OPEN-12, 2026-08-14):**
  proposal actions, review evidence, and check findings are acquired by
  **explicit pointing only** — their overlay hit geometry, lane rows, or
  checks/Inspector surfaces. **Region gestures (rectangle/lasso) and Ctrl+A
  never acquire non-authored subjects**: a region is a spatial query over
  authored design geometry, and its result must be a function of design
  content, never of which checks ran or which proposals are pending. Mixed
  compounds (non-authored + authored members) are deferred until each
  channel has complete typed scene authority and the S5-C03 lifetime rules
  exist — transient subjects (findings die on recheck, proposals on
  commit/discard) cannot yet be bookkept by the §2.2.10 stable-identity
  compound model.
- **Definition-editor owned graphics (owner-resolved, S5-C01A / OPEN-5,
  2026-08-14):** inside the Footprint and Symbol Editors, owned graphics —
  body strokes, silk artwork, lines/arcs, and filled shapes — are
  **independent authored targets** qualified by the existing generic rules:
  the line/path anchor rule for strokes and the filled-graphic rule (100%
  enclosure; direct click; Select-menu entry) for filled shapes, including
  the symbol body fill, which is the authored object in its editor. In
  board/schematic workspaces owned graphics remain parent projections
  (OPEN-2), completing one uniform law: every child class is a parent
  projection in placement workspaces and an independent target in its
  definition editor. Standalone schematic drawings carry the same generic
  construction with no cross-pane twin.
- Copper zones do not qualify from a partial rectangle/lasso overlap; they
  qualify only when **100 percent** of their authored filled area (including all
  islands) is enclosed. Otherwise a zone is selected by direct primary click or
  explicitly through right-button drag → `Select` → the user-legible zone entry
  (name/net/layer); conflict-menu `Select All` may include it. Zone outline,
  fill, islands, and thermal geometry project one authored zone identity rather
  than independent selectable objects. Connected/global electrical selection
  may include the zone because that is a logical expansion, not geometric
  region acquisition.
- Independently selectable filled graphics use the same rule: no partial
  rectangle/lasso acquisition, but complete enclosure of **100 percent** of the
  authored filled area qualifies. They also remain available through direct
  primary click when topmost or an explicit user-legible entry (and optional
  `Select All` membership) under right-button drag → `Select`. Their outline and
  fill project one authored graphic identity.
- Vias, junctions, no-connect markers, and other point-like objects qualify
  when their center or electrical connection anchor lies inside the region.

Line/path qualification follows authored topology anchors rather than a
rendered-length percentage:

- a straight authored section qualifies only when both endpoints lie inside the
  rectangle/lasso (the strict majority of two anchors is two);
- a curved authored section qualifies when at least two of its start, authored
  midpoint, and end anchors lie inside;
- a connected multi-section run is not region-tested as one path: every
  authored section qualifies independently, while double/triple click provide
  the explicit connected-run/global-net expansion; and
- rendered stroke width, selection halo, viewport clipping, and dash phase do
  not inflate or otherwise change the anchor test.

This endpoint rule follows the researched EDA/CAD containment precedent while
retaining Datum's strict-majority contract without an unfamiliar or visually
ambiguous path-length integration.

Standalone text and labels qualify when more than 50 percent of their oriented
layout rectangle lies inside the rectangle/lasso; exactly 50 percent does not.
The test uses actual rotated layout bounds rather than an axis-aligned expansion
or per-glyph ink. Glyph outlines, inter-glyph whitespace, font changes, and
selection halos do not change qualification. Schematic net labels follow their
visible layout bounds; their electrical connection anchor does not independently
force region selection.

Reference/value text owned by a footprint or symbol is neither independently
selected nor counted toward its parent's qualification in board/schematic
workspaces. In footprint/symbol editor workspaces that owned text becomes an
independent authored target and follows the same strict-majority oriented-layout
rule. Direct primary click and the local `Select` menu remain the precise paths
for difficult text cases.

**All owned text kinds (owner-resolved, S5-C01A / OPEN-3, 2026-08-14):** the
definition-editor independence above applies to **every** owned text kind —
reference/value, user text, fabrication notes, and additional symbol fields —
under the same strict-majority oriented-layout rule; there is one text
qualification law, never a per-kind variant. In board/schematic workspaces all
owned text kinds alike remain parent projections (not selectable, not
counted). Hidden-field editor behavior is governed by the OPEN-13 resolution;
the field table (key add/delete/reorder) remains a dedicated tool — canvas
selection governs the placed text subject.

Filled graphics here means authored non-zone shapes with an interior (for
example PCB logo/documentation polygons, rectangles, or circles); schematic
symbol-body fills remain projections of their parent symbol.

#### 2.2.5 Progressive electrical selection scope

Electrical geometry uses progressive click depth:

- single click selects the local authored section under the pointer;
- double click replaces that with the physically connected run containing the
  section; and
- triple click replaces that with every occurrence carrying the same resolved
  logical net identity across the complete design.

On schematics, the global scope includes disconnected and cross-sheet
occurrences joined by the same resolved label. On boards, it includes all
conductive/connective geometry assigned to the net throughout the board. This
behavior applies to every net, not only power nets. Net classes are not a click
depth. Parent footprints and symbols may render related context but are not
silently added to an electrical selection.

Click-depth expansion is progressive and immediate: section, then connected
run, then global net. Pointer movement beyond the click threshold or a changed
hit target starts a new sequence.

**Ladder origins (owner-resolved, S5-C01A / OPEN-1, 2026-08-14).** The click
ladder originates on **directly-selectable conductive geometry** — track/wire
sections, vias, and zone/pour copper: geometry where the single click selects
the conductive object itself, so each deeper tier widens the same subject
(the run is the physically continuous copper/wire connected component through
the origin). **Pads and labels/ports are excluded as origins** and remain
object-only: a pad's single click is ratified to select the parent footprint
(§2.2.4), so ladder tiers cannot nest from it; a label's double click is
reserved for future edit-in-place (S5B). Neither loses net acquisition — the
explicit `Select Net` verb in the right-button-drag local Select menu and the
marking menu provides it from every net member. This is a gesture-scoping
rule, not a capability boundary.

#### 2.2.6 Overlap resolution through the local menu

Normal primary click remains fast and selects the deterministic topmost eligible
candidate. Datum MUST NOT introduce a separate automatic ambiguity popup or
click-cycling path. Right-button drag invokes the governed local/marking menu;
its `Select` branch exposes every eligible conflicting candidate under the
pointer using user-legible reference, pad/pin, net, layer, and type labels.
Candidate hover pre-highlights that candidate. The menu offers both individual
selection and `Select All`; releasing outside or pressing `Escape` dismisses it
without changing selection. Candidate order is deterministic by visible layer,
object priority, then stable identity.

#### 2.2.7 Locked objects

Locked objects remain selectable and inspectable but MUST NOT be modified. They
have a persistent visible distinction (for example subdued/greyed authored
geometry); selection retains the normal highlight while also preserving an
unambiguous lock indication. A refused mutation writes one concise `stdout`
message identifying the locked object and command. A mutation over a mixed
locked/unlocked selection fails as a whole rather than silently applying a
partial edit. Selection never unlocks an object implicitly.

#### 2.2.8 Hidden geometry and layer visibility

- Geometry on a hidden layer cannot be newly selected by primary click,
  rectangle, lasso, conflict-menu `Select All`, or electrical selection
  expansion. Objects excluded by an active object-class selection filter are
  likewise ineligible.
- Dimmed but still-visible geometry remains eligible unless its object class is
  filtered out. Hidden and locked are distinct states: locked objects remain
  available for inspection, while hidden objects cannot be acquired from the
  canvas.
- Hiding an already-selected object preserves its selection identity; a
  visibility change MUST NOT implicitly clear or remove selection. Its canvas
  projection and selection highlight disappear with the hidden geometry, while
  the Inspector and approved contextual/command output report that selected
  members are hidden. This does not depend on or decide the deferred Application
  Status Bar.
  Restoring visibility restores their selection projection.
- Hidden selected objects cannot normally be manipulated or modified. A
  mutation whose ordinary/local selection contains hidden members is refused as
  a whole rather than silently editing only visible members. The explicit
  `Ctrl+A` global-selection scope in §2.2.9 is the sole exception. Hidden members
  may be removed explicitly through a non-canvas selection consumer, or the
  complete selection may be cleared with `Escape`.

**Field visibility and definition-child lock (owner-resolved, S5-C01A /
OPEN-13, 2026-08-14):** field visibility is an **authored, journaled design
property**, distinct from this section's layer/class visibility, which is
consumer view state and never journaled; the two mechanisms never conflate.
The governing acquisition rule is **canvas acquisition requires rendered
geometry**: an invisible field renders no glyphs and is therefore not
canvas-acquirable by construction — it is found and managed through the field
table and Inspector. If a definition editor later renders invisible fields
ghosted (a view option), they become acquirable automatically because they
are rendered; no rule change is needed. **Definition children (pads, pins,
owned text, owned graphics) carry no lock semantics in S5A** — the locked-
object law applies only where a typed lock exists; the universal lock
vocabulary arrives with S5B and is not faked before its substrate.

#### 2.2.9 Explicit global Select All

`Ctrl+A` replaces the prior selection with every authored object in the focused
workspace document. It is the sole global-selection authority and deliberately
supersedes ordinary canvas eligibility: it includes hidden objects, objects
excluded by object-class filters, copper pours/zones, filled graphics, and all
other authored workspace geometry. Pads and pins still collapse to their parent
footprint/symbol in board/schematic workspaces. Grid, page chrome, hover,
findings, proposals, and other non-authored overlays are not authored objects
and are excluded.

The explicit global scope permits manipulation of hidden selected members.
Locked members are included and visibly distinguished but remain immutable; any
mutation over the global selection is refused as a whole until every affected
locked object is explicitly unlocked or removed. Datum MUST NOT silently leave
locked members behind while moving the remainder.

This exception belongs only to `Ctrl+A`. A zoomed-out rectangle/lasso remains a
local geometric selection and follows §2.2.4, including the 100-percent rule for
zones and filled areas. It MUST NOT acquire global-selection authority merely
because it visually surrounds the complete design.

#### 2.2.10 Selection membership, focus, and compound Inspector subject

The selection is a typed stable-identity set, not a scalar target or an
internally ordered list with implicit authority. Deterministic iteration and
serialization use stable identity ordering only for reproducibility; that order
MUST NOT choose a privileged member. The set has an optional explicit **focus
member**:

- plain primary click creates a one-member set and focuses that member;
- `Shift`+primary click adds and focuses the clicked member;
- an individual local-menu `Select` choice focuses that member;
- `Ctrl`+primary click removes the member; removing the focus member leaves the
  remaining set with no focus rather than promoting an arbitrary member;
- rectangle/lasso preserves an existing focus member only while it remains in
  the resulting set, and a region begun without a prior selection creates a
  group with no focus; and
- `Ctrl+A` creates a global group with no focus member.

Focus is consumer/session state and does not give its member additional mutation
authority. Any command that genuinely needs a reference object must obtain an
explicit reference through that command's interaction rather than choosing the
first UUID, render candidate, or set member.

With exactly one member, the Inspector subject is that authored object. With
multiple members, the Inspector subject is a first-class **temporary compound
selection**, presented with a user-legible label such as
`Compound Selection — 14 objects`. It exposes compound attributes rather than
pretending that one focus member represents the whole set. The attribute surface
includes at least member count and types, combined bounds, group reference and
position, workspace/layer/net coverage, common-versus-mixed state, hidden and
locked counts, and an expandable member inventory. An optional focus member may
be identified within that compound without replacing the compound Inspector
subject.

The compound exists only as selection/session state and disappears when the
selection is cleared; creating it is never journaled and does not silently add
an authored project group. An explicit future S5B `Group` command may convert
the current compound into a named persistent authored `Group XXX` through the
normal typed operation/journal path; ordinary multi-selection never performs
that conversion implicitly, and S5A exposes no such mutation.

The compound Inspector's S5A baseline is read-only. It presents these future
compatible capabilities only as unavailable/explanatory extension seams:

- **Position X/Y:** editing either coordinate translates every member by the
  displayed delta as one atomic Move operation;
- **Rotation:** rotates every compatible member atomically around the displayed
  group reference;
- **Mirror horizontal / vertical:** available in the schematic editor for a
  compatible symbol selection and applied atomically within that schematic
  workspace;
- **Lock / unlock:** one explicit atomic group command with the affected-member
  count; and
- **Group:** creates the persistent authored `Group XXX` described above.

Derived bounds, counts, and coverage remain legible even when not editable.
The Inspector provides an explicit scope selector without changing canvas
selection: `All N` followed by per-type scopes such as `Parts 8`, `Traces 4`, or
`Symbols 6`. `All N` exposes only typed semantic properties applicable to every
member. A per-type scope deliberately narrows the target set and may expose
additional compatible properties for that type; every proposed edit states its
scope and affected count (for example, `Set DNP on 8 of 12 selected objects`).
Returning to `All N` restores the compound view with the same selection.

A common value renders normally; divergent values render explicitly as
`Mixed`, never as blank or an arbitrary member's value. Replacing `Mixed` with a
concrete value is an intentional request to assign that value to every member of
the declared scope. Compatibility is based on typed semantic property identity,
value domain, units, and mutation verb—not a coincidentally shared display
label. A field is unavailable when the complete declared scope cannot accept it.

Every later S5B batch edit preflights its complete declared scope and commits as one typed
atomic operation with undo/redo. Locked, stale, incompatible, constrained, or
otherwise invalid members cause an explained whole-operation refusal; Datum
MUST NOT silently skip them. DNP remains semantically distinct from Exclude from
BOM, Exclude from Board, Exclude from Simulation, and variant fitted/not-fitted
state.

Further compatible batch attributes are an intended extension point rather than
an excuse for partial mutation. The broader PCB, schematic, library,
variant/manufacturing, text/graphics, connectivity, and persistent-group
research inventory and classification are preserved in
`GUI_COMPOUND_SELECTION_RESEARCH.md`; individual domain surfaces still require
owner ratification before entering an implementation boundary.

#### 2.2.11 Cross-workspace projection and GUI mutation authority

A selection is project-workspace state and projects into every open workspace
that can resolve a related representation. Selecting PCB geometry therefore
also renders the corresponding schematic selection projection, and selecting
schematic geometry renders the corresponding PCB projection. For example,
triple-clicking a resolved net in the schematic displays that global-net
selection on the board as well as the schematic. This is one shared design
selection projected across editors, not two unrelated private selections.

Exactly one GUI workspace is active at a time. Its governed magenta workspace
frame is the visible mutation-authority indicator. Selection may remain visibly
projected in inactive workspaces, but GUI gestures, hotkeys, the action console,
and local authoring commands MUST NOT modify an inactive workspace. The user
must navigate/focus the board before editing the board projection, or focus the
schematic before editing the schematic projection. Changing active workspace
does not by itself clear the shared selection.

This active-workspace restriction governs GUI authority only. It does not
replace engine validation or redefine explicit CLI/MCP operations, whose target
documents and authority remain part of their typed command contracts.

**Definition↔instance projection (owner-resolved, S5-C01A / OPEN-6,
2026-08-14):** a library **definition** (pad, pin, owned text/field, owned
graphic, footprint, symbol) and its **placed instances** are different
identity domains joined by a library binding — never one same-identity
subject. Selecting a definition in a Footprint/Symbol Editor projects placed
instances as **merely related** under the RB §2.3 law: exact authored
appearance, no selection accent, no glow, not counted as selected. Full
same-identity cross-pane projection remains reserved for the same resolved
design object viewed through multiple panes (e.g. the Global Net subject).
The Inspector and the explicit relationship view carry usage instead
("used by N instances"; mild unrelated-dim only). This mirrors the ratified
merely-related treatment of component↔placed-symbol cross-probe.

#### 2.2.12 Future S5B moving contract: one verb, four invocation surfaces

This section is a future S5B contract and is unavailable in read-only S5A. It
defines the extension seam without authorizing implementation in S5A.

Movement requires an existing selection. The fast direct-manipulation sequence
is: primary click to select, then a subsequent primary press-and-drag on a
visible selected member to move the complete selection. Datum MUST NOT collapse
an initial selection click into an accidental move. The movement drag begins
only after the pointer crosses the governed 4-physical-pixel threshold.

The same move verb is exposed through four deliberate interaction surfaces:

1. primary-drag a visible selected member;
2. enter `Move` in the future action console;
3. press the `M` hotkey; or
4. invoke right-button drag → `Move Selection` in the local menu.

These are projections of one command, not independent move implementations.
They MUST share selection input, snapping, preview, validation, cancellation,
commit, and journal behavior. The local-menu route remains available even when
hidden global-selection members make direct dragging impractical, and the
action-console vocabulary must not create private mutation semantics.

If any selected member is locked, no invocation begins a partial move. The
selection is preserved, locked members remain visibly distinguished, the local
menu action is disabled where preflight state is available, and an attempted
keyboard/console/direct invocation reports the atomic refusal to `stdout`. The
user must explicitly unlock or remove every affected locked member before the
move can begin.

Movement uses one immediate reference-point rule and does not prompt for an
additional AutoCAD-style base point:

- direct drag uses the primary-press world position;
- `M` and action-console `Move` use the current canvas cursor world position;
- local-menu `Move Selection` uses the menu invocation world position; and
- the reference need not lie on visible geometry, so `Ctrl+A` selections with
  hidden members remain movable.

Movement begins immediately. Snapping applies to that reference point; primary
click commits the destination and `Escape` cancels, restoring every original
position. Every invocation resolves the same translation:
`new_position = original_position + (destination - reference)`.

#### 2.2.13 Visual-state precedence **[LOCKED]**

**Selection wins over hover.** The selected state remains the authoritative
visual whenever selection and hover target the same object. Hover MUST NOT
recolor, obscure, thicken, weaken, replace, or otherwise visually compete with
the selection highlight. Hovering an already-selected object changes only the
cursor or interaction affordance needed for the available gesture; it does not
add a second object halo. Hovering a different eligible object continues to show
the governed lighter hover preview without altering the selected set.

This precedence is screen-space consumer state and MUST NOT invalidate or
rebuild retained authored geometry. Locked, cross-workspace, focus-member,
related-highlight, and diagnostic overlap rules are locked by the Rendering
Book §§2.1–2.8. Research inventory and integration guidance:
`research/selection-visual-language/SELECTION_VISUAL_LANGUAGE_RESEARCH.md` →
`DATUM_SELECTION_VISUAL_LANGUAGE_GUIDANCE.md`; tracked by
`dat-s5-selection-visual-contract-zid`.

An actual shared selection uses the same full governed selection treatment in
every active or inactive workspace where it resolves. Pane inactivity MUST NOT
dim or restyle selected objects. The magenta pane frame, focus dot/header, and
tool enablement alone communicate which workspace has GUI mutation authority.
Focus change preserves selection membership/appearance. Hover may still follow
the pointer-containing inactive pane, but selection wins on the same object.
Merely related/cross-probed geometry is not an actual member and uses the locked
subordinate related-context role in the Rendering Book §2.3.

A triple-click Global Net selection is one semantic selection subject. Its full
selection projection includes every visible resolved electrical representation
of that net across workspaces—wires, **junction dots (owner-resolved, S5-C01A /
OPEN-7, 2026-08-14: authored conductive geometry, an owned member with full
member treatment)**, labels/ports, pin terminals/stubs, tracks,
vias, connected pad regions, net-owned zone boundary/fill, and relevant
airwire/ratsnest geometry. Parent symbol bodies and footprints are not selected
by connectivity alone; they remain merely related. Hidden members remain hidden
and are summarized by the net Inspector rather than expanded into an enormous
primitive selection set.

Bus selection has its own hierarchy: single click selects the local authored
section, double click the physically connected bus run, and triple click the
semantic bus identity across schematic hierarchy. Its projection includes the
visible bus spine, owned name/label, and attached entry geometry as one subject.
Scalar member wires/nets remain independent; selecting them follows normal net
click depth and does not select the parent bus. Entry-level independent selection
exists only in a workspace/tool with that editing authority. Inspector lists
member nets and hidden/cross-sheet occurrences without glowing every member net.

**Bus region qualification and hierarchy projection (owner-resolved, S5-C01A /
OPEN-8, 2026-08-14):** bus sections region-qualify by the generic line/path
anchor rule (straight: both endpoints inside; curved: ≥2 of
start/authored-midpoint/end), each section independently. **Entries never
enlarge a bus section's region test**, and a region containing only entry
geometry acquires nothing — entries are owned member geometry, never
origination geometry. The semantic-bus tier projects **Global-Net-style across
all schematic sheets and panes** — spine runs, bus name/labels, and attached
entries as one subject with hidden/cross-sheet occurrences summarized by
Inspector counts — carried by **bus-distinct typing** (never a disguised
scalar-net list). **A bus has no board-side projection**: buses resolve to
individual nets on the board, so copper participates only through a member
net's own ladder, keeping Bus and Global Net subjects distinct.

Merely related geometry retains its exact authored baseline and receives no
accent, internal glow, recoloring, or luminance lift. In an explicit
relationship/cross-probe context, unrelated geometry may dim slightly so the
unchanged related baseline remains legible by figure/ground. Related objects are
not selected, have no transform handles, and are not counted as selection
members. Direct selection promotes them normally. The current ungoverned
via-coloured `AUTHOR_RELATED` presentation is migration debt, not the target.

Within a compound selection, the optional focus member has no persistent extra
canvas styling: every member uses the same full selection treatment. Focus is
identified in Inspector/session state only and grants no mutation authority.
Commands needing a geometric reference display a temporary command-owned marker
while armed. Focus removal leaves no focus and never promotes another member by
identity/render order.

Locked is orthogonal to selection: the authored presentation remains slightly
desaturated/greyed, full selection remains intact, transform handles are absent,
and a small screen-space padlock appears at the selected/hovered authored anchor.
Dense compounds limit repeated glyphs while Inspector reports locked count.
The padlock is blocked on visual-system governance: `icon_set.json` declaration,
the Rendering Book 24px/1.7-stroke/round-cap language, icon-contact-sheet entry,
and HUMAN prototype review are prerequisites; no Unicode/one-off fallback.

Proposal, diagnostic, and selection are orthogonal channels. Authored geometry
is the base; proposed geometry retains its ghost/dual stroke; selection remains
an identifiable object-shaped brightening/glow/cue; diagnostic marker shape and
semantic hue render topmost. Selecting a proposal or diagnostic adds selection
without erasing uncommitted/severity identity. No channel may recolor another
into plain selection magenta, and proposal overlap cannot erase an authored
selection boundary.

Standalone text selection follows actual rendered glyph geometry—no persistent
generic bbox—with edit handles only under the text tool and a low-zoom minimum
cue at the authored anchor. Point selections preserve semantic cores: junction
wire-coloured center + accent ring/glow; no-connect complete X/flag; via material
and drill void + silhouette ring/glow. Symbol Editor pin selection treats the
complete stub/terminal/name/number without siblings/body. Tiny/high-contrast
fallbacks are crisp screen-space rings/outlines that never alter authored/hit
geometry.

Dense/global/panelized selection uses a deterministic per-pane LOD. Cull hidden,
offscreen, and clipped projections; simplify sub-2-physical-pixel objects to one
minimum cue; emit at most **65,536 detailed overlay primitives per pane/frame**.
Overflow switches the entire pane to an exact screen-space union mask of all
visible selected silhouettes—never partial truncation or a bbox. High screen
coverage reduces only soft-glow radius/opacity while preserving crisp cue and
authored colours. Inspector reports complete count + aggregate LOD. A 100k-object
fixture gates deterministic fallback, warm-buffer reuse, and bounded capacity.

**Dense law over non-authored channels (owner-resolved, S5-C01A / OPEN-14,
2026-08-14):** the selection-cue budget and fallback above are **one per-pane
mechanism wherever the cue draws** — including over proposal, review, and
diagnostic geometry; there is never a second parallel LOD system, so the
65,536 bound stays a hard guarantee under maximal collision. The union-mask
fallback replaces the **selection cue channel only**: proposal
ghost/dual-stroke, diagnostic marker shape and severity hue, and review
evidence keep rendering in their compositing positions and are never
flattened, absorbed, or recolored by the mask — the no-channel-flattening law
extended to the degraded regime. The 100k-object fixture MUST include a
maximal-collision variant (dense authored selection + active proposals +
findings) gating this identity preservation (S5-C09/S5-C10 evidence scope).

#### 2.2.14 S5A/S5B delivery boundary

S5 is split so the selection experience does not claim mutation authority the
engine does not possess:

- **S5A — selection and compound inspection:** the ratified acquisition,
  lifecycle, membership, projection, visibility, region, compound subject,
  scope, Common/`Mixed`/Unavailable presentation, and read-only derived/blocker
  reporting contract.
- **S5B — selection authority substrate:** persistent authored Group identity
  and operations; universal lock vocabulary or an explicit capability matrix;
  typed field-level batch patch/operation contracts; atomic preflight; and the
  ratified compatible translation/rotation/schematic-mirror transforms.
- **Later domain tools:** topology-, rule-, library-, variant-, manufacturing-,
  hierarchy-, annotation-, padstack-, zone-, track/via-, production-, and
  copy/delete-closure surfaces classified by the research-derived guidance.

`Compound Selection` is reserved for ephemeral selection/session state;
`Group <name>` is reserved for a persistent authored object created explicitly.
S5A MUST retain typed extension seams for S5B/later without presenting those
later capabilities as landed. S5 execution remains unauthorized until final
owner review and a numbered selection-identity decision ratify this mechanism.

<!-- EVIDENCE:UVT-S5-SPEC:S5-C05-BOUNDARY -->
**Boundary reconciliation (S5-C05, 2026-08-14).** The complete §2.2 corpus is
reconciled to this boundary under one **global reading rule**: any wording in
§2.2 that names moving, dragging-to-move, rotation, mirroring, lock/unlock
commands, `Group` creation, editable/batch fields, transform handles, or the
text-edit tool specifies **S5B-or-later** behavior — S5A ships acquisition,
lifecycle (§2.2.18), projection, and read-only inspection/outputs (§2.2.19)
only, and claims no mutation authority anywhere. Specifics fixed or confirmed
by the sweep:

- §2.2.2 plain primary drag: rewritten — the gesture is *reserved* for the
  S5B movement contract and binds no S5A behavior.
- §2.2.7/2.2.8/2.2.9 locked/hidden mutation-refusal language (whole-refusal,
  no implicit unlock, hidden-member manipulation): these are **laws
  constraining future mutations**, not S5A capability claims; their atomic
  semantics are S5-C06's contract.
- §2.2.10 compound capability rows (rotation, lock/unlock, batch fields):
  confirmed already presented as unavailable/explanatory extension seams —
  the required S5A rendering is *visible, disabled, with reason*, exactly as
  Phase 1 renders `not_built` menu items.
- §2.2.12: confirmed titled and scoped as the future S5B moving contract,
  unavailable in S5A.
- §2.2.13/§2.2.16 references to transform handles and the text-edit tool
  (text rows, RB §2.7 language): the tools and handles are S5B-or-later; in
  S5A no handle geometry renders for any selection state.
- §2.2.16 matrix: every row's mutation-adjacent cell (lock behavior, batch
  fields, dedicated tools) reads under the global rule; OPEN-13 already
  ratifies no S5A child-lock vocabulary.

**Boundary assertions (TO-ENFORCE; consumed by S5-C10):**

- **B1 no mutation entry** — the S5A build exposes no code path from any
  selection surface (canvas, Inspector, console, envelope) to a design
  `Operation` (`boundary_no_mutation_entry`).
- **B2 seams render disabled-with-reason** — S5B seam capabilities render
  visible, disabled, and explained; never active, never hidden
  (`boundary_seams_disabled_with_reason`).
- **B3 no handle geometry** — no transform-handle overlay renders in S5A
  for any selection state (`boundary_no_handles_in_s5a`).

<!-- EVIDENCE:UVT-S5-SPEC:S5-C06-ATOMIC -->
**Atomic refusal law (S5-C06, 2026-08-14).** One law governs every later
operation invoked over a selection, from any surface (S5B transforms,
quantize/align, parametric deep verbs, batch fields, domain tools):

- **Preflight, then all-or-nothing.** Before emitting any `Operation`, the
  shared batch guard preflights every member. A member that is **locked**,
  **stale** (fails §2.2.18 re-validation), **incompatible** (the operation
  is not typed for its class), **constrained** (a rule or solver constraint
  forbids the result), or **invalid** (fails engine validation) causes the
  **whole operation to refuse**. No member is silently skipped, no partial
  mutation is committed, and no journal entry is produced by a refusal.
- **Explained, never mute.** A refusal names the blocking members and the
  blocker kind per member, using the same blocker vocabulary the outputs
  contract already disclosed (§2.2.19 O4) — a refused operation is never a
  surprise the Inspector did not already show. The explanation states how
  to proceed (unlock explicitly, remove members, narrow scope).
- **No implicit repair.** Refusal never auto-unlocks, auto-drops, or
  auto-narrows the selection to make the operation succeed; the user (or
  agent) modifies the selection or the blockers explicitly and re-invokes.
  The sole ratified exception remains §2.2.9 `Ctrl+A` hidden-member
  manipulation, which is explicit global scope by definition.
- **Same law for agents.** Operations arriving via CLI/MCP over a
  selection-derived id set refuse identically with the same typed blocker
  report — no surface gets a lenient partial-application mode.

Reconciled wording: §3.3 quantize (widened from locked-only to the full
five-blocker enumeration); `DATUM_GUI_PARAMETRIC_TOOLING.md` (already
carries the full enumeration — confirmed conformant); §2.2.7/2.2.9/2.2.10
whole-refusal clauses (confirmed as instances of this law).

**Atomicity assertions (TO-ENFORCE; consumed by S5-C10):**

- **R1 whole refusal** — one blocked member of any of the five kinds
  refuses the entire operation with zero journal effect
  (`refusal_whole_operation_no_journal`).
- **R2 blocker disclosure parity** — the refusal report matches the
  §2.2.19 blocker vocabulary and names every blocking member
  (`refusal_report_matches_outputs`).
- **R3 no implicit repair** — refusal never mutates selection membership
  or lock state (`refusal_no_implicit_repair`).
- **R4 surface parity** — GUI, CLI, and MCP invocations over the same
  selection state produce identical refusal reports
  (`refusal_surface_parity`).

#### 2.2.15 Canonical S5 specification-closure requirements

These stable IDs are exhaustive for the remaining S5 contract. The structured
Frontier completion plan orders them and beads mirrors the IDs; neither may
publish a smaller rival checklist. Completion authorizes only the
specification/ratification transaction, never S5A implementation.

<!-- REQ:UVT-S5-SPEC:S5-C01 -->
- **S5-C01 — identity/class matrix.** Reconcile one exhaustive PCB, Footprint
  Editor, Schematic, and Symbol Editor matrix covering selectable subject,
  parent/child ownership, click/region qualification, section/run/global scope,
  owned versus related projection, hidden/locked/filter behavior, overlay and
  accessibility/LOD treatment, Inspector projection, and explicit unsupported
  classes. Include authored geometry plus selectable proposal/review/diagnostic
  identities; absence of typed scene authority must be stated, never omitted.
  Mandatory rows are: PCB Board footprint, track section/run, via, zone, text,
  filled graphic, line/arc/outline, dimension, and Global Net; Footprint Editor
  pad and owned text/graphics; Schematic symbol, wire section/run, Global Net,
  bus section/run/semantic bus, label/port, junction, no-connect, text, drawing,
  and hierarchical sheet representation; Symbol Editor pin and owned
  text/graphics; plus proposal, review, and diagnostic subjects. Each row must
  be supported or explicitly marked unsupported/deferred.

<!-- REQ:UVT-S5-SPEC:S5-C01A -->
- **S5-C01A — matrix owner-choice gate.** Resolve and record every disposition
  in the S5-C01 `OPEN-1` through `OPEN-14` register before any dependent
  specification, prototype, or conformance step begins. The approved choices
  become inputs to C02–C10; those steps propagate them into their owning
  contracts and evidence. C11 is final review of an already resolved corpus,
  not the first point at which matrix choices are made. This step is an explicit
  owner boundary: agents present the register and stop without choosing,
  claiming specification work, or advancing until owner dispositions are
  recorded.

<!-- REQ:UVT-S5-SPEC:S5-C02 -->
- **S5-C02 — bounded region queries.** Specify deterministic rectangle/lasso
  candidate bounds, exhaustion result/fallback, auto-pan revealed geometry, no
  unbounded scan, and exact future assertions.

<!-- REQ:UVT-S5-SPEC:S5-C03 -->
- **S5-C03 — lifetime.** Specify selection/focus behavior across model revision,
  update/replacement/deletion, document or pane replacement, stale identities,
  and partial cross-pane resolution.

<!-- REQ:UVT-S5-SPEC:S5-C04 -->
- **S5-C04 — compound outputs.** Define single/multi Inspector subjects,
  `All N`/per-type scopes, Common/`Mixed`/Unavailable, counts/bounds/coverage,
  hidden/locked/blocker reporting, and a stable-ID payload for terminal,
  action-console, and AI context without depending on the deferred status bar.

<!-- REQ:UVT-S5-SPEC:S5-C05 -->
- **S5-C05 — S5A/S5B boundary.** Reconcile all wording to S5A acquisition,
  lifecycle, projection, and read-only inspection. Move/rotate/mirror/lock/group
  and editable fields remain S5B/later seams with no claimed mutation authority.

<!-- REQ:UVT-S5-SPEC:S5-C06 -->
- **S5-C06 — atomic refusal.** Reconcile quantize and parametric-tooling wording
  so locked, stale, incompatible, constrained, or invalid members refuse the
  whole later operation; no member is silently skipped.

<!-- REQ:UVT-S5-SPEC:S5-C07 -->
- **S5-C07 — identity and cross-probe.** Supersede current-use singleton wording,
  define object/compound/run/Global-Net/Bus/proposal/review/diagnostic subjects,
  distinguish same-identity projections from merely-related mappings, and align
  P2.3 as dependent on completed S5A.

<!-- REQ:UVT-S5-SPEC:S5-C08 -->
- **S5-C08 — overlay law.** Reconcile §4 and retained-selection wording to one
  class-A immediate overlay: exact 2-physical-pixel crisp object cue,
  subordinate internal glow and owned-geometry lift, with retained bytes,
  static buffers, CAM/export geometry, and hit bounds unchanged.

<!-- REQ:UVT-S5-SPEC:S5-C09 -->
- **S5-C09 — prototype/accessibility evidence.** Confirm or update fixed-size
  Rendering Study and Schematic Editor references; add the governed padlock
  asset/contact-sheet entry; cover compound, locked, maximal collision,
  active/inactive panes, Global Net/Bus, text/points, and dense fallback under
  normal, high-contrast, CVD, reduced-motion, zoom, and scale review.

<!-- REQ:UVT-S5-SPEC:S5-C10 -->
- **S5-C10 — conformance disposition.** Give every S5 machine claim an exact
  future gate/test file and assertion and every HUMAN claim an exact reference,
  committed golden, and review record. Build-dependent checks remain honestly
  TO-ENFORCE until they land with S5A; specification-phase evidence closes now.

<!-- REQ:UVT-S5-SPEC:S5-C11 -->
<!-- OWNER:UVT-S5-SPEC:S5-C11:S5-FINAL-REVIEW -->
- **S5-C11 — owner review.** Review C01–C10, record explicit approval with no
  remaining S5 choice, and leave the Application Status Bar deferred.

<!-- REQ:UVT-S5-SPEC:S5-C12 -->
<!-- OWNER:UVT-S5-SPEC:S5-C12:S5-RATIFICATION -->
- **S5-C12 — selection-identity decision.** Allocate the next available decision
  number at creation time, cite the complete contract/evidence, ratify Layer-2
  identity and the S5A/S5B boundary, and keep implementation separately authorized.

<!-- REQ:UVT-S5-SPEC:S5-C13 -->
- **S5-C13 — synchronized closure.** Update governance/parity where applicable,
  traceability/status prose, Frontier plus generated projection, and the bead;
  close only with specification evidence. S5A becomes unblocked but is neither
  automatically execution-authorized nor canonical—the selector is explicitly
  advanced in the same governance transaction. Run project-state, governance,
  conformance, and source-health gates.

#### 2.2.16 S5-C01 selection identity/class matrix (reconciled)

<!-- EVIDENCE:UVT-S5-SPEC:S5-C01-MATRIX -->

This is the exhaustive cross-editor selection identity/class matrix required by
S5-C01: every mandatory selectable class of the PCB board editor, Footprint
Editor, Schematic editor, and Symbol Editor, plus the cross-cutting
proposal/review/diagnostic subjects. It reconciles the ratified rules of
§2.2.1–2.2.14, the Rendering Book selection law, the compound/Inspector
guidance, and the **code truth** of typed scene authority. Where the governing
corpus is silent, the cell says `OPEN-n` and the open-reconciliation register
at the end of this section carries the candidate resolution — silence is
recorded, never papered over. Code citations (file:line) describe the substrate
at commit `d598bd8` and are evidence, not normative contract; document
citations are normative.

**Verdict predicate (applies to every row).** A row closes with two
independent verdicts, so "supported" can never blur specification with
implementation:

- `spec:` **ratified** (every cell closed by governing text) / **partial**
  (one or more cells `OPEN-n`) / **silent** (the corpus defines no selection
  rules for the class).
- `substrate:` **live** (typed scene identity with a working hit/selection
  path) / **typed-only** (typed identity exists; acquisition unwired or hit
  region absent) / **absent** (no typed scene identity at all).

A row is *supported* when `spec` is ratified or partial AND the class is an
S5A acquisition subject; *unsupported* or *deferred* rows say so explicitly
with the owning closure item. Schematic click selection is deliberately
unwired pending S5A (`resolve_schematic_primary_click` returns false,
`crates/gui-app/src/runtime_camera_pane.rs:525-542`); `typed-only` on
schematic rows reflects that gate, not an accident.

**Citation key.** UVT §n = this spec; RB §n = `DATUM_RENDERING_BOOK.md`;
CONF = `DATUM_GUI_CONFORMANCE_SPEC.md` S5 rows; P2 = `DATUM_GUI_PHASE_2_SPEC.md`;
COMP = `DATUM_SELECTION_COMPOUND_EDITING_GUIDANCE.md` (+
`research/gui-compound-selection/GUI_COMPOUND_SELECTION_RESEARCH.md`);
SVL = `DATUM_SELECTION_VISUAL_LANGUAGE_GUIDANCE.md` (+ its research file).

**Shared baselines.** Every row inherits these four baselines; cells state
only class deltas. A baseline never silently closes a cell the corpus left
open — such cells carry `OPEN-n` markers regardless.

- **B-CLICK** (UVT §2.2.6): click selects the deterministic topmost eligible
  candidate; overlap resolves through the right-button-drag local Select menu
  with user-legible labels; no ambiguity popup, no click-cycling.
- **B-HL** (UVT §2.2.7/2.2.8/2.2.9): hidden-layer or class-filtered geometry
  is ineligible for NEW selection by click, region, menu Select All, or
  electrical expansion; hiding an already-selected object preserves selection
  identity and the Inspector reports hidden members; locked objects stay
  selectable + inspectable, never modifiable — slight neutral greying, full
  selection treatment retained, no transform handles, whole-selection refusal
  on mixed mutation with one concise explanation; the anchor padlock glyph is
  governance-blocked pending `icon_set.json` declaration (RB §2.5); Ctrl+A
  supersedes eligibility filters (UVT §2.2.9).
- **B-OV** (RB §2.1; UVT §2.2.13): the universal three-part treatment on the
  actual visible silhouette — slight luminance lift on every owned primitive,
  `#CE5A92` internal soft glow, crisp object-shaped 2.0-physical-px
  screen-space cue; never a generic bounding box; identical full strength in
  every resolving pane, active or inactive; selection wins over hover, never
  pulses, never color-alone; high-contrast mode substitutes crisp
  rings/outlines for glow; screen-only — retained buffers, CAM/export
  geometry, and hit bounds never change.
- **B-LOD** (RB §2.8; UVT §2.2.13): sub-2px member projections collapse to
  one minimum screen-space cue; 65,536 detailed-overlay-primitive cap per
  pane/frame, then atomic whole-pane exact visible-silhouette union-mask
  fallback — never partial/bbox truncation; the 100k-object fixture gates
  determinism.
- **B-INS** (CONF; P2; UVT §2.2.10; COMP): single subject → Inspector title
  ref + kind + SELECTED chip with Identity/Placement/Checks sections;
  compound subject → "Compound Selection — N objects" with member count/types,
  combined bounds, hidden/locked/incompatible counts with exact blocker
  explanation, `All N` + per-type scopes, Common/`Mixed`/Unavailable field
  rendering; S5A Inspector is read-only.

##### PCB board editor

- **Board footprint (placed component instance).**
  *Ownership:* parent owns children — a click on any owned pad selects the
  parent footprint; pad anchors feed parent region qualification; owned
  reference/value text is neither independently selectable nor counted; Ctrl+A
  collapses pads to the parent (UVT §2.2.4/2.2.9). Child independence in the
  Footprint Editor workspace is ratified for pads and owned reference/value
  text only (UVT §2.2.4); board-workspace clicks on owned graphics select the
  parent footprint and component Edge.Cuts geometry carries no hit region
  (OPEN-2 resolved, §2.2.4). *Qualification:* B-CLICK; region qualifies on a strict majority
  (>50%) of pad center anchors inside (padless → placement anchor);
  silkscreen, courtyard, fab graphics, and ref/value text never enlarge the
  test (UVT §2.2.4). *Scope:* object-only; never silently added to electrical
  selections — under Global Net the body is merely related (UVT §2.2.5/2.2.13;
  RB §2.2); pad clicks never originate the click ladder — tier 1 is the parent
  footprint, so tiers cannot nest; the explicit `Select Net` verb covers net
  acquisition from a pad (OPEN-1 resolved, §2.2.5 origins). *Projection:* whole coherent presentation lifts as one subject;
  connected traces NOT selected (RB §2.1); component↔symbol cross-pane is a
  merely-related mapping — exact authored appearance, no accent, Inspector
  explains (RB §2.3; P2; CONF). *Hidden/locked:* B-HL. *Overlay:* B-OV +
  B-LOD. *Inspector:* B-INS; compound `Parts N`; candidate common fields
  value/rotation/lock; side/flip, DNP/variant, annotation, fields go to
  dedicated authority (COMP). *Scene authority:* `ComponentBounds`
  (`crates/gui-protocol/src/lib.rs:234`, object_kind `component`), rect hit
  region + sub-graphics hit-mapping to the component id
  (`crates/gui-render/src/render/retained.rs:743/:754/:814/:827`); selected as
  flat `SelectionTarget::AuthoredObject(String)` (`lib.rs:440-445`) — no typed
  object class; `ComponentTextPrimitive` has no hit region.
  *Verdict:* spec **ratified** (OPEN-1/OPEN-2 resolved); substrate
  **live**.
- **Track section / connected run.**
  *Ownership:* the authored section is its own subject; run and net are
  progressive scopes, not ownership containers — a run is never region-tested
  as one path (UVT §2.2.4/2.2.5). *Qualification:* B-CLICK; region path rule —
  straight section needs BOTH endpoints inside; curved needs ≥2 of
  start/authored-midpoint/end; every section qualifies independently; stroke
  width, halo, clipping, dash phase never alter the test (UVT §2.2.4).
  *Scope:* the ratified ladder — single = section, double = physically
  connected run, triple = design-wide resolved net; every net; net classes are
  not a click depth; pointer movement past threshold restarts (UVT §2.2.5).
  *Projection:* exact authored path per member section, no giant bbox
  (RB §2.1/2.2); at net depth the same semantic identity projects into the
  schematic pane (UVT §2.2.11; P2). *Hidden/locked:* B-HL. *Overlay:* B-OV +
  B-LOD; the current retained-world selected-recolor path is migration debt —
  S5 moves selection to the typed post-world overlay (SVL). *Inspector:*
  B-INS; compound `Traces N`; net/topology/cleanup via dedicated tools
  (COMP). *Scene authority:* `TrackPrimitive` (`lib.rs:285`), polyline hit
  region (`retained.rs:694`), click wired
  (`crates/gui-app/src/main.rs:2648-2691/:2738`); ABSENT: any typed run/net
  scope subject — `AuthoredObject(String)` holds one flat id, no click-depth
  machinery. *Verdict:* spec **ratified**; substrate **live** (section) /
  **absent** (run/net scope subjects).
- **Via.**
  *Ownership:* independent point-like subject; Global Net member without
  becoming a separately counted selection (UVT §2.2.13; RB §2.2).
  *Qualification:* B-CLICK; region point rule — center/connection anchor
  inside (UVT §2.2.4). *Scope:* full ladder origin — directly-selectable
  conductive geometry (OPEN-1 resolved, §2.2.5 origins): double = the
  physically connected copper run through the via, triple = the resolved
  net. *Projection:* own silhouette; full member treatment
  under a Global Net subject (RB §2.2). *Hidden/locked:* B-HL. *Overlay:*
  B-OV + B-LOD; semantic core preserved — via material colour + drill void
  with silhouette ring; must stay distinguishable from junction/terminal/
  no-connect while selected (RB §2.7). *Inspector:* B-INS; span/type, net,
  tenting, padstack via dedicated authority (COMP). *Scene authority:*
  `ViaPrimitive` (`lib.rs:297`), circle hit region (`retained.rs:707`).
  *Verdict:* spec **ratified** (OPEN-1 resolved); substrate **live**.
- **Zone / copper pour.**
  *Ownership:* outline, fill, islands, and thermal geometry project ONE
  authored zone identity — none independent (UVT §2.2.4); net-owned
  boundary/fill is a Global Net member (UVT §2.2.13). *Qualification:*
  B-CLICK (direct when topmost, or explicit Select-menu entry with
  name/net/layer label); region — NO partial qualification; 100% enclosure of
  the authored filled area including islands (UVT §2.2.4). *Scope:*
  object-only geometric acquisition; logical inclusion via electrical
  expansion only (UVT §2.2.4); zone/pour copper is a full ladder origin
  (OPEN-1 resolved, §2.2.5 origins).
  *Projection:* one zone identity — boundary treated, layer fill retained
  (RB §2.1); same-identity Global Net participation (UVT §2.2.13).
  *Hidden/locked:* B-HL. *Overlay:* B-OV + B-LOD; large-coverage glow
  radius/opacity reduces deterministically while the crisp cue and material
  colours remain (RB §2.1/2.8). *Inspector:* B-INS; fill is derived geometry —
  inspectable, never written back; net/layer/priority/refill via dedicated
  tools (COMP). *Scene authority:* `ZonePrimitive` (`lib.rs:311`), polygon
  hit region (`retained.rs:775`). *Verdict:* spec **ratified** (OPEN-1
  resolved); substrate **live**.
- **Standalone board text.**
  *Ownership:* own subject; component-owned ref/value text belongs to the
  footprint row's child rule (UVT §2.2.4). *Qualification:* B-CLICK + Select
  menu as the precise path; region — >50% of the ORIENTED layout rectangle;
  exactly 50% fails; rotated layout bounds, never axis-aligned expansion or
  per-glyph ink (UVT §2.2.4). *Scope:* object-only. *Projection:* selection
  follows actual rendered glyph geometry; the layout rectangle is
  hit/qualification geometry only, never a persistent visual (RB §2.7).
  *Hidden/locked:* B-HL. *Overlay:* B-OV + B-LOD; glyph brightening + glow,
  NO persistent layout rectangle; handles only while the text-edit tool is
  active; low zoom → minimum cue at the authored anchor (RB §2.7); the current
  loose bbox halo is fallback-only implementation debt (SVL). *Inspector:*
  B-INS; common fields style/height/stroke/alignment/visibility where typed;
  content formulas dedicated (COMP). *Scene authority:* `BoardTextPrimitive`
  (`crates/gui-protocol/src/board_text_primitives.rs:5`), rect hit region
  (`retained.rs:842`); glyph meshes hit only via the sibling rect;
  `ComponentTextPrimitive` rendered with NO hit region and suppresses the
  parent fallback rect (`retained.rs:735-752`). *Verdict:* spec **ratified**;
  substrate **live**.
- **Filled graphic (standalone authored shape with interior).**
  *Ownership:* outline + fill project one authored identity (UVT §2.2.4);
  symbol/footprint-owned fills are parent projections, excluded here.
  *Qualification:* B-CLICK (direct when topmost, or explicit Select-menu
  entry); region — no partial acquisition; 100% enclosure of the authored
  filled area (UVT §2.2.4). *Scope:* object-only; Ctrl+A includes it
  (UVT §2.2.9); no electrical membership. *Projection:* one identity, board-
  local. *Hidden/locked:* B-HL. *Overlay:* B-OV + B-LOD. *Inspector:* B-INS;
  NO candidate common fields until authored graphic identities converge
  (COMP). *Scene authority:* `BoardGraphicPrimitive` (`lib.rs:131`), polyline
  hit region only (`retained.rs:856`); ABSENT: a filled-graphic class distinct
  from strokes and any interior hit shape — the ratified interior-click/
  enclosure semantics have no substrate. *Verdict:* spec **ratified**;
  substrate **live** (outline hit only; interior hit + filled identity
  absent).
- **Line / arc / outline graphic (stroke-only, incl. board outline).**
  *Ownership:* standalone graphics are their own subjects. Component-owned
  graphics: clicks select the parent footprint and component Edge.Cuts
  geometry carries no hit region (OPEN-2 resolved, §2.2.4 — ratifies the
  code behavior at `retained.rs:814/:827/:790-798`). *Qualification:* B-CLICK; region path rule
  as for track sections (UVT §2.2.4). *Scope:* object-only. *Projection:*
  exact authored path, board-local. *Hidden/locked:* B-HL. *Overlay:* B-OV +
  B-LOD. *Inspector:* B-INS; no common fields until identities converge
  (COMP). *Scene authority:* `BoardGraphicPrimitive` (`lib.rs:131`) +
  `OutlinePolyline` (`lib.rs:217`), polyline hit regions
  (`retained.rs:856/:869`). *Verdict:* spec **ratified** (OPEN-2 resolved);
  substrate **live**.
- **Dimension (board measurement annotation).**
  **Deferred from S5A by owner ratification (OPEN-10 resolved,
  2026-08-14).** The class is not an S5A selection subject: the engine type
  exists (`crates/engine/src/board/dimension.rs`) but is never projected
  into any scene — not rendered, no primitive, no object_kind, no hit
  region — and the Rendering Book defines no dimension visual construction,
  so acquisition rules would specify selection for undesigned geometry and
  could carry no honest check disposition. Unrendered geometry cannot
  region-qualify by construction; no special-casing needed. **Re-entry
  condition:** author dimension selection rules when the Rendering Book
  defines the dimension construction AND typed scene projection exists —
  expected alongside the decision-020 documentation-system spec pass, which
  may re-home dimensions in paper space. Inspector guidance retained for
  that future pass: measured value is derived, never written back;
  endpoints/units/precision via dedicated tools (COMP). *Verdict:* spec
  **ratified deferral**; substrate **absent** — **unsupported in S5A by
  design**.
- **Global Net (PCB projection).**
  *Ownership:* ONE semantic subject, not a compound of primitives (RB §2.2);
  members are all conductive/connective geometry on the resolved net —
  tracks, vias, connected pad regions, net-owned zone boundary/fill, and
  ratsnest projection (UVT §2.2.5/2.2.13); parent footprints/symbol bodies
  are never selected by connectivity — merely related. *Qualification:*
  triple-click on net geometry only (third ladder tier, UVT §2.2.5); region
  gestures NEVER acquire net/global scope (UVT §2.2.9). *Scope:* IS the
  global ladder tier; every net; net classes are not a click depth.
  *Projection:* same-identity across panes — every visible resolved
  electrical representation glows with full member treatment; Escape clears
  the whole subject as one selection (UVT §2.2.11/2.2.13; RB §2.2; P2).
  *Hidden/locked:* B-HL — hidden members stay hidden, summarized by Inspector
  count, never materialized (UVT §2.2.13; CONF). *Overlay:* B-OV + B-LOD —
  the dense-LOD law is the governing scale mechanism for this subject.
  *Inspector:* net-centric Net/Members/Checks(ERC) view; per-kind and
  hidden-member counts; netclass values display-only (P2; CONF; COMP).
  *Scene authority:* ABSENT — `SelectionTarget::AuthoredObject(String)`
  carries one flat id; no net-subject type, no ladder wiring (board click
  emits single-object select only, `main.rs:2648-2691`); ratsnest
  `UnroutedPrimitive` (`lib.rs:148`) rendered with NO hit region
  (`retained.rs:444-493`). *Verdict:* spec **ratified**; substrate **absent**
  — the largest spec-vs-substrate gap in this matrix; blocks P2.3.
- **Airwire / ratsnest geometry.**
  Ratified here to close the class explicitly: derived ratsnest geometry is
  **never an independent selection subject** — it has no authored identity to
  select. It participates in selection ONLY as projected member geometry of a
  Global Net subject (UVT §2.2.13), where it receives member treatment under
  B-OV/B-LOD. Click/region gestures never acquire an airwire directly;
  Ctrl+A excludes it as non-authored (UVT §2.2.9). *Scene authority:*
  `UnroutedPrimitive` (`lib.rs:148`), rendered, no hit region — consistent
  with this ratification. *Verdict:* spec **ratified** (this row); substrate
  **typed-only**; **unsupported as an independent subject by design**.

##### Footprint Editor

- **Pad (independent authored target).**
  *Ownership:* independently selectable in this workspace (UVT §2.2.4) — the
  definition editor's authority; no parent collapse. Pad geometry + pad-number
  text project as ONE pad subject; pad-number text is excluded from the
  owned-text class (OPEN-4 resolved, §2.2.4). *Qualification:* B-CLICK with
  pad-pin labels in the Select menu; region — the pad center anchor inside
  the rectangle/lasso qualifies (OPEN-4 resolved, §2.2.4 anchor family).
  *Scope:*
  object-only — no resolved-net ladder in a library-definition context.
  *Projection:* definition→placed-instance mapping (same-identity vs
  merely-related): placed instances project as merely related — exact
  authored appearance, no accent (OPEN-6 resolved, §2.2.11). *Hidden/locked:* B-HL; many families carry no lock attribute
  today — universal lock vocabulary is S5B (COMP). *Overlay:* B-OV + B-LOD;
  pad silhouette with copper colour + drill void preserved (RB §2.1/2.7).
  *Inspector:* B-INS; `Pads N`; common fields later typed
  net/mask/paste/rotation; shape/padstack/drill/number via dedicated tools
  (COMP). *Scene authority:* ABSENT — no Footprint Editor surface exists:
  `PaneContent` and `SceneSurface` are exactly { Board, Schematic }
  (`crates/gui-protocol/src/workspace_layout.rs:174-177`,
  `crates/gui-render/src/render/types.rs:80-83`); the only pad type is the
  board-review `PadPrimitive` (`lib.rs:249`). *Verdict:* spec **partial**
  **ratified** (OPEN-4/OPEN-6 resolved); substrate **absent**.
- **Owned text (all kinds).**
  *Ownership:* every footprint-owned text kind — reference/value, user text,
  fab notes — is an independent authored target in this workspace (OPEN-3
  resolved, §2.2.4 all-owned-text clause); in the board workspace all owned
  text alike is a parent projection, not selectable, not counted
  (UVT §2.2.4). *Qualification:* strict-majority oriented-layout
  rule, ratified for owned text by the §2.2.4 Footprint Editor clause;
  B-CLICK. *Scope:* object-only. *Projection:* definition↔instance text
  mapping: merely related, never same-identity (OPEN-6 resolved,
  §2.2.11). *Hidden/locked:* B-HL; field-visibility vs
  layer-visibility semantics resolved: distinct mechanisms; canvas
  acquisition requires rendered geometry (OPEN-13, §2.2.8). *Overlay:* B-OV +
  B-LOD; RB §2.7 text law (glyph geometry, no persistent rectangle, edit-tool
  handles only). *Inspector:* B-INS; style/height/stroke/alignment/visibility
  where typed (COMP). *Scene authority:* ABSENT — no editor surface; nearest
  type `ComponentTextPrimitive` (`lib.rs:182`) is board-side, no hit region.
  *Verdict:* spec **ratified** (OPEN-3/OPEN-6/OPEN-13 resolved); substrate
  **absent**.
- **Owned graphics (strokes and filled shapes).**
  *Ownership:* independent authored targets in this workspace (OPEN-5
  resolved, §2.2.4 definition-editor owned-graphics clause); board-workspace
  sub-graphics remain parent projections (OPEN-2 resolved).
  *Qualification:* the existing generic rules — path anchor rule for
  strokes/arcs, filled-graphic rule (100% enclosure; direct click;
  Select-menu entry) for filled shapes (UVT §2.2.4). *Scope:* object-only.
  Graphics are not uniformly backed by authored engine identities (COMP),
  which blocks stable identity bookkeeping until that converges.
  *Hidden/locked:* B-HL. *Overlay:* generic B-OV + B-LOD (no class-specific
  RB text). *Inspector:* B-INS; no common fields until identities converge
  (COMP). *Scene authority:* ABSENT — no editor surface; board-side
  `ComponentGraphicPrimitive` (`lib.rs:168`) hit-maps to the parent
  component. *Verdict:* spec **ratified** (OPEN-5 resolved); substrate
  **absent**.

##### Schematic editor

- **Symbol.**
  *Ownership:* symbol is the parent unit — a pin click selects the parent;
  pin anchors feed parent qualification; owned ref/value text not
  independently selectable; body fills remain parent projections; Ctrl+A
  collapses pins to parent (UVT §2.2.4/2.2.9); pin independence exists only
  in the Symbol Editor. *Qualification:* B-CLICK; region — strict >50%
  majority of pin connection anchors (pinless → placement anchor); graphics/
  text never enlarge the test (UVT §2.2.4). *Scope:* object-only; under
  Global Net the body is merely related (UVT §2.2.5/2.2.13). *Projection:*
  whole-symbol coherent subject — body stroke, pin stubs, terminal dots, ALL
  symbol text lift together; "a half-highlighted symbol is a defect"; dark
  body fill stays dark; attached nets keep normal wire colour (RB §2/2.1);
  component↔symbol cross-pane is merely-related (RB §2.3; P2; CONF).
  *Hidden/locked:* B-HL. *Overlay:* B-OV + B-LOD. *Inspector:* B-INS;
  `Symbols N`; value/transform-display where typed; reference uniqueness/
  bindings/units/variants via dedicated tools; mirror is S5B (COMP;
  UVT §2.2.10). *Scene authority:* `SchematicHitKind::Symbol`
  (`crates/gui-protocol/src/schematic_scene_import/mod.rs:27-35`, object_kind
  `schematic_symbol`), rect hit region via `push_schematic_hit_regions`
  (`crates/gui-render/src/render/coordinate_hit.rs:203-234`); pins tagged
  `schematic-symbol-pin:{uuid}:{idx}`; selection flattens to
  `AuthoredObject(String)`; click unwired pending S5A. *Verdict:* spec
  **ratified**; substrate **typed-only**.
- **Wire (section / connected run).**
  *Ownership:* independent authored sections; run/net are click-depth scopes,
  not ownership (UVT §2.2.4/2.2.5). *Qualification:* B-CLICK; region path
  rule (both-endpoints / ≥2-of-3 anchors) (UVT §2.2.4). *Scope:* ratified
  ladder — section / physically connected run / global resolved net including
  disconnected cross-sheet occurrences joined by the same resolved label
  (UVT §2.2.5). *Projection:* exact authored path; same-identity Global Net
  member; named P2.3 cross-probe highlight target (RB §2.1/2.2; P2).
  *Hidden/locked:* B-HL — electrical expansion cannot newly select hidden
  geometry (UVT §2.2.8). *Overlay:* B-OV + B-LOD. *Inspector:* B-INS; wires
  carry NO generic common properties — topology-aware operations are
  dedicated tools (COMP); net-tier selection projects the Net/Members/Checks
  view (P2). *Scene authority:* `SchematicHitKind::Wire`, polyline hit
  region (`coordinate_hit.rs:203-234`); ABSENT: typed section/run/net scope
  identity or click-depth machinery. *Verdict:* spec **ratified**; substrate
  **typed-only** (section) / **absent** (scope machinery).
- **Global Net (schematic projection).**
  As the PCB Global Net row, from the schematic side: ONE semantic subject
  owning schematic wires, matching labels/ports, and pin connection
  terminals/stubs plus the board members; acquired by triple-click on a wire
  section; never region-acquirable; Escape clears the whole subject; hidden
  members summarized by count; parent bodies merely related
  (UVT §2.2.5/2.2.11/2.2.13; RB §2.2; CONF). The P2.3 worked example of
  same-identity cross-probe (P2). *Inspector:* Net/Members/Checks(ERC) view
  (P2). *Scene authority:* ABSENT — no net-subject type, no ladder, no
  cross-pane projection type anywhere in gui-protocol/gui-render/gui-app.
  *Verdict:* spec **ratified**; substrate **absent** — S5A must land it
  before the P2.3 cross-probe build.
- **Bus (section / run / semantic hierarchical bus).**
  *Ownership:* the bus owns spine + owned name/label + attached bus-entry
  geometry as ONE subject; scalar member wires/nets remain independent
  subjects — member selection never selects the parent bus and members never
  glow via bus membership; entries are not separately counted
  (UVT §2.2.13; RB §2.2). *Qualification:* B-CLICK = local authored bus
  section; region — generic path rule per section; entries never enlarge
  the test and entry-only regions acquire nothing (OPEN-8 resolved,
  §2.2.13). *Scope:* own three-tier ladder:
  section / physically connected run / semantic bus identity across the
  hierarchy (UVT §2.2.13); typed Bus projection is distinct from scalar
  Global Net (CONF). *Projection:* joint spine+name+entries subject;
  semantic tier projects Global-Net-style across all schematic sheets/panes
  with bus-distinct typing; no board-side projection — buses resolve to
  individual nets on copper (OPEN-8 resolved, §2.2.13).
  *Hidden/locked:* B-HL; Inspector carries member AND hidden counts (CONF).
  *Overlay:* B-OV + B-LOD on the joint subject; members stay at authored
  baseline. *Inspector:* member nets listed, not glowed; bus
  name/members/segments via dedicated tools (UVT §2.2.13; COMP). *Scene
  authority:* `SchematicHitKind::Bus`, polyline hit region; ABSENT: bus
  entries remain untyped `schematic_graphic` with no hit region; no
  run/semantic scope identity. *Verdict:* spec **ratified** (OPEN-8
  resolved); substrate **typed-only** (spine) / **absent** (entries, scopes).
- **Label / port.**
  *Ownership:* independent authored targets; a bus-owned name/label belongs
  to the bus subject; labels/ports become owned Global Net members when the
  net is selected (UVT §2.2.13). *Qualification:* region — visible layout
  bounds under the strict >50% oriented-rectangle rule; the connection anchor
  does NOT independently force selection (UVT §2.2.4); B-CLICK. *Scope:*
  object-only — labels never originate the ladder; double click is reserved
  for S5B edit-in-place and the `Select Net` verb covers net acquisition
  (OPEN-1 resolved, §2.2.5 origins).
  *Projection:* label/port geometry itself (glyphs + pill, dark fill
  retained, RB §2.1); same-identity Global Net member; named P2.3 highlight
  target (P2). *Hidden/locked:* B-HL. *Overlay:* B-OV + B-LOD. *Inspector:*
  B-INS; rename/kind/direction via dedicated tools with hierarchy validation
  (COMP). *Scene authority:* `SchematicHitKind::Label`, rect hit region;
  port is typed as `Label` — the port class survives only in the
  `schematic-port:{uuid}` id prefix and must be properly typed during S5A.
  *Verdict:* spec **ratified** (OPEN-1 resolved); substrate **typed-only**
  (port class untyped).
- **Junction.**
  *Ownership:* independent point-like authored object; selecting it never
  selects attached wires (UVT §2.2.4). *Qualification:* point rule —
  center/connection anchor inside (UVT §2.2.4); B-CLICK. *Scope:*
  object-only. *Projection:* the junction dot alone — wire-coloured filled
  center preserved + accent ring; junction/pin-terminal/via/no-connect stay
  mutually distinguishable while selected (RB §2.7); the junction dot is an
  owned member of the Global Net projection with full member treatment
  (OPEN-7 resolved, §2.2.13 member list) while remaining an independent
  object-only subject for direct acquisition — the same dual role as vias.
  *Hidden/locked:* B-HL. *Overlay:* B-OV + B-LOD tiny-object
  law (RB §2.7). *Inspector:* position/status display only;
  connectivity-aware placement/removal are dedicated tools (COMP). *Scene
  authority:* `SchematicHitKind::Junction`, polygon hit region. *Verdict:*
  spec **ratified** (OPEN-7 resolved); substrate **typed-only**.
- **No-connect marker.**
  *Ownership:* independent point-like object; never selects the parent
  symbol (RB §2.7). *Qualification:* point rule; B-CLICK. *Scope:*
  object-only — it marks absence of connectivity. *Projection:* the complete
  X/flag semantic core as one subject (UVT §2.2.13; RB §2.7).
  *Hidden/locked:* B-HL. *Overlay:* B-OV + B-LOD tiny-object law.
  *Inspector:* position/status display only (COMP). *Scene authority:*
  `SchematicHitKind::NoConnect`, polyline hit region. *Verdict:* spec
  **ratified**; substrate **typed-only**.
- **Text (standalone).**
  *Ownership:* independent authored target; symbol-owned ref/value text is a
  parent projection here (UVT §2.2.4). *Qualification:* >50% oriented layout
  rectangle; B-CLICK + Select menu for difficult cases (UVT §2.2.4).
  *Scope:* object-only. *Projection:* rendered glyph geometry; layout rect is
  hit geometry only (RB §2.7). *Hidden/locked:* B-HL. *Overlay:* B-OV +
  B-LOD; RB §2.7 text law. *Inspector:* B-INS; compound schematic text
  mirrors the PCB text field set where typed — style, height, stroke,
  alignment, visibility; content stays dedicated (OPEN-9 resolved; one
  text vocabulary program-wide, folded into S5-C04). *Scene authority:* ABSENT — free schematic
  text flows through `SchematicTextSink` into `board_texts`
  (`schematic_scene_import/mod.rs:278-292`) but schematic hit regions are
  built only from `board_graphics`
  (`crates/gui-render/src/render/scene.rs:464-474`); `SchematicHitKind` has
  no Text variant. *Verdict:* spec **ratified** (OPEN-9 resolved); substrate
  **absent**.
- **Drawing / graphic.**
  *Ownership:* standalone drawings are independent targets; symbol-body
  fills are explicitly excluded — parent projections (UVT §2.2.4).
  *Qualification:* generic rules govern — path rule for strokes,
  filled-graphic rule (100% enclosure) for filled shapes (UVT §2.2.4).
  *Scope:* object-only. *Projection:* filled shapes project outline + fill
  as one identity; sheet-local documentation geometry with no cross-pane
  twin (OPEN-5 resolved). *Hidden/locked:* B-HL. *Overlay:* B-OV + B-LOD —
  the generic construction is the ratified treatment (OPEN-5 resolved).
  *Inspector:* B-INS; no common fields until
  authored identities converge (COMP). *Scene authority:* ABSENT —
  schematic drawings emit as untyped `schematic_graphic`;
  `schematic_hit_kind()` returns None → no hit regions. *Verdict:* spec
  **ratified** (OPEN-5 resolved); substrate **absent**.
- **Hierarchical sheet representation.**
  **Deferred from S5A by owner ratification (OPEN-11 resolved,
  2026-08-14).** Sheets render today (plain `schematic_graphic` rects, id
  `schematic-sheet-instance:{uuid}`, `schematic_scene_import/mod.rs:293-310`)
  but carry no hit kind and no typed identity, and the selection subject is
  undesigned in three directions: no Rendering Book sheet construction, no
  typed sheet-pin children, and cross-sheet instance identity unaddressed.
  Sheet interaction is navigation-dominated — the descend/breadcrumb model
  belongs to the schematic-surface design pass, which precedes selection
  rules. Visible-but-inert sheet bodies are acceptable in read-only S5A; the
  Sheets panel (P2) provides list navigation. **Gesture reservation
  (ratified now):** double click on a sheet body is reserved for
  descend-into-sheet and no future step may bind it otherwise. **Re-entry
  condition:** author sheet selection rules when the schematic-surface
  design pass defines the sheet construction and navigation model AND typed
  sheet + sheet-pin scene authority exists. *Verdict:* spec **ratified
  deferral**; substrate **absent** — **unsupported in S5A by design**.

##### Symbol Editor

- **Pin (independent child subject: stub + terminal + name + number as ONE
  subject).**
  *Ownership:* workspace granularity flips ownership — schematic workspace:
  pin click selects the parent symbol; Symbol Editor: the pin is
  independently selectable and the subject is the complete
  stub/terminal/name/number, explicitly without sibling pins or the body
  (UVT §2.2.4/2.2.13; RB §2.7); name/number text are children of the pin
  subject. *Qualification:* B-CLICK (pad-pin labels); region — the pin
  connection anchor inside the rectangle/lasso qualifies the complete
  compound subject; name/number text never enlarges the test (OPEN-4
  resolved, §2.2.4 anchor family). *Scope:*
  object-only — no electrical expansion from a pin *definition*.
  *Projection:* stub/terminal/name/number as one owned identity; siblings
  and body at rest (RB §2.7); projection onto placed instances unstated
  : merely related, never same-identity (OPEN-6 resolved, §2.2.11).
  *Hidden/locked:* B-HL; whether a pin definition carries a lock
  attribute: none in S5A — lock vocabulary arrives with S5B (OPEN-13
  resolved, §2.2.8). *Overlay:* B-OV + B-LOD; pin
  terminal stays distinct from junction/via/no-connect while selected
  (RB §2.7). *Inspector:* B-INS; library pins read placed projection; pin
  table / library authoring are dedicated tools (COMP). *Scene authority:*
  ABSENT — no Symbol Editor surface, scene kind, or hit vocabulary exists
  anywhere (`PaneContent`/`SceneSurface` = { Board, Schematic }); nearest is
  schematic-surface `SchematicHitKind::Pin`, which resolves to the parent
  symbol. *Verdict:* spec **ratified** (OPEN-4/OPEN-6 resolved); substrate
  **absent**.
- **Owned text (all kinds).**
  As the Footprint Editor owned-text row, for symbol-owned text: every owned
  text kind — reference/value and all additional fields — is an independent
  authored target in this workspace under the strict-majority oriented-layout
  rule (OPEN-3 resolved, §2.2.4 all-owned-text clause); pin name/number text
  is NOT this class — it belongs to the pin subject (RB §2.7).
  Definition↔instance field projection unstated
  resolved as merely related (OPEN-6, §2.2.11); field-visibility vs
  layer-visibility semantics unreconciled
  resolved: distinct mechanisms; canvas acquisition requires rendered
  geometry (OPEN-13, §2.2.8). *Overlay:* B-OV + B-LOD; RB §2.7 text law. *Inspector:*
  B-INS; symbol fields expose visibility/position after typed ops; the field
  table is a dedicated tool (COMP). *Scene authority:* ABSENT — no editor
  surface; no owned-text selection identity exists anywhere. *Verdict:* spec
  **ratified** (OPEN-3/OPEN-6/OPEN-13 resolved); substrate **absent**.
- **Owned graphics (body strokes, lines/arcs, filled shapes).**
  As the Footprint Editor owned-graphics row: independent authored targets
  in this workspace under the generic path + filled-graphic rules (OPEN-5
  resolved, §2.2.4) — **including the symbol body fill, which is the
  authored object in its editor**; on the schematic the body fill remains a
  parent projection and the RB dark-body-fill retention rule continues to
  describe whole-symbol selection, not editor-side fill selection. Authored
  engine identity for graphics is not uniform (COMP). *Scene authority:*
  ABSENT — no editor surface; symbol drawings emit as untyped
  `schematic_graphic`, no hit region. *Verdict:* spec **ratified** (OPEN-5
  resolved); substrate **absent**.

##### Cross-cutting non-authored subjects (all editors)

- **Proposal object (uncommitted route/production proposal, ghost/dual-stroke).**
  *Ownership:* a click owns the whole proposal ACTION, not a geometric
  primitive — every overlay primitive carries `proposal_action_id` and
  resolves to one `HitTarget::ReviewAction` (code truth); per-primitive
  qualification does not exist; spec-side ownership is declared SILENT by
  §2.2; resolved: the whole proposal ACTION is the subject, acquired by
  explicit pointing only (OPEN-12 resolved, §2.2.4). *Qualification:* click live
  today via screen-space `HitRegion` rects
  (`crates/gui-render/src/render/overlay.rs:243/:259`); region qualification
  resolved: region gestures NEVER acquire proposal subjects — regions are
  spatial queries over authored geometry only (OPEN-12 resolved, §2.2.4). *Scope:* one action = one
  subject; ladder inapplicable; explicit-only acquisition (OPEN-12
  resolved). *Projection:* compositing law —
  authored base → proposal ghost/dual-stroke → selection cue → topmost
  diagnostic; selecting ADDS the cue without erasing uncommitted identity;
  no channel recolors another into selection magenta (RB §2.6;
  UVT §2.2.13); cross-pane projection deferred with mixed-compound
  semantics (OPEN-12); production
  proposals have no world-scene identity at all (data-panel summary +
  chrome hit only). *Hidden/locked:* excluded from Ctrl+A as non-authored
  (UVT §2.2.9) and from region acquisition (OPEN-12 resolved);
  hidden/locked semantics ride the channel lifecycle, not §2.2.7/2.2.8.
  *Overlay:*
  ghost/dual-stroke identity retained under selection (RB §2.6); B-LOD
  one shared budget/fallback with channel identity preserved (OPEN-14
  resolved, §2.2.13 dense clause). *Inspector:*
  review lane projects the active action; compound membership for
  non-authored subjects is deferred until the channel has complete typed
  scene authority and S5-C03 lifetime rules exist (OPEN-12 resolved). *Scene
  authority:* `ProposalOverlayPrimitive` (`lib.rs:322-334`);
  `SelectionTarget::ReviewAction(String)` + `SessionCommand::SelectReviewAction`
  (`lib.rs:440-445/:628-661`); ABSENT: retained-world hit regions; any typed
  world identity for production proposals. *Verdict:* spec **partial**
  **ratified** (OPEN-12 resolved: explicit-only acquisition; mixed
  compounds deferred); substrate
  **live** (route-proposal action selection end-to-end).
- **Review subject (dashed evidence geometry of a review action).**
  *Ownership:* parent = the review action; evidence polylines are typed
  children (`ReviewPrimitive.evidence_key`) and are never independently
  selectable — no hit region of any kind; the owning action is reachable
  only through review-lane chrome rows — the ratified explicit-only path
  (OPEN-12 resolved); evidence geometry is never independently acquirable.
  *Projection:* dashed evidence keyed to the ACTIVE action; RB §2.6
  orthogonal-channel law binds any selection cue over it. *Hidden/locked:*
  excluded from Ctrl+A (UVT §2.2.9) and region acquisition (OPEN-12
  resolved). *Overlay:*
  B-LOD: one shared budget/fallback, channel identity preserved (OPEN-14
  resolved, §2.2.13 dense clause). *Scene authority:*
  `ReviewPrimitive` (`lib.rs:337-342`), rendered
  (`overlay.rs:265-291`); ABSENT: any hit region for evidence geometry.
  *Verdict:* spec **ratified** (OPEN-12 resolved: lane-only explicit
  acquisition); substrate **typed-only**.
- **Diagnostic / finding marker (ERC/DRC check finding).**
  *Ownership:* the finding is its own selection identity (check-finding
  fingerprint), NOT a child of the diagnosed authored object — target
  cross-resolution (`check_finding_scene_target_object_id`,
  `crates/gui-protocol/src/check_runs.rs:194-202`) is a fit/hover
  navigation aid, not ownership; click ownership otherwise SILENT
  ; resolved: the finding is acquired by explicit pointing only — marker,
  lane, or checks surface (OPEN-12 resolved). *Qualification:* dead today — `HitTarget::CheckFinding`
  exists and is handled (`types.rs:43`; `main.rs:2747`) but is constructed
  nowhere; region gestures never acquire findings (OPEN-12 resolved).
  *Projection:* marker shape +
  semantic severity hue render TOPMOST; selecting ADDS selection without
  erasing severity; shape, not hue alone, carries distinction (RB §2.6;
  UVT §2.2.13). *Hidden/locked:* excluded from Ctrl+A (UVT §2.2.9);
  target-hidden behavior deferred with mixed-compound semantics (OPEN-12).
  *Overlay:* B-LOD application
  one shared budget/fallback with marker shape/severity preserved (OPEN-14
  resolved, §2.2.13 dense clause); no marker is rendered in-scene today.
  *Inspector:*
  renders an already-selected finding; compound membership for mixed
  finding+authored selections deferred until typed scene authority and
  S5-C03 lifetime rules exist (OPEN-12 resolved). *Scene authority:*
  `SelectionTarget::CheckFinding(String)` + `SessionCommand::SelectCheckFinding`
  (`lib.rs:440-445/:641`); ABSENT: any scene marker primitive and any live
  pointer path. *Verdict:* spec **ratified** (OPEN-12 resolved:
  explicit-only acquisition; mixed compounds deferred); substrate
  **typed-only** — canvas marker rendering and the pointer path remain
  unbuilt.

#### 2.2.17 S5-C02 bounded region query contract

<!-- EVIDENCE:UVT-S5-SPEC:S5-C02-CONTRACT -->

This contract makes every rectangle/lasso membership query (§2.2.2 gestures,
§2.2.4/§2.2.16 per-class qualification) deterministic, bounded, and atomic.
It governs the query engine, not the gestures: §2.2.2/§2.2.3 own how a region
is drawn; this section owns how its membership is computed.

**1. Exactness law.** Committed region membership is an **exact, total
function of four inputs**: the resolved design revision, the final
world-space region geometry, the per-class qualification rules, and the
eligibility state (layer/class visibility per §2.2.7/2.2.8, OPEN-12
authored-only scope). It is NEVER a function of pan path, frame timing,
preview budgets, camera/zoom during the gesture, screen visibility of
candidates, or check/proposal state. There is no such thing as a partial or
truncated region selection: budgets bound **work per frame**, never **result
correctness**.

**2. Candidate bounds — no unbounded scan.** Candidate enumeration runs
through the shared spatial index (`SpatialHitIndex`,
`crates/gui-viewport/src/hit.rs` — the S3 AABB hierarchy): the region's
world-space AABB prunes the tree, and only AABB-intersecting candidates are
examined. Examined-candidate count is proportional to geometry intersecting
the region's AABB, never to total design size, for any sub-scene region. The
query tests **per-class qualification anchors** (pad centers, pin connection
anchors, path endpoints/authored midpoints, placement anchors, oriented text
layout rectangles, authored zone filled areas) as world geometry — never
rendered pixels.

**3. Lasso geometry.** The lasso polygon closes automatically (last vertex
to first) on evaluation. Point-anchor tests use the **even-odd rule**
point-in-polygon. The oriented-text >50% rule evaluates the area fraction of
the oriented layout rectangle inside the region exactly (polygon clipping),
for both rectangle and lasso regions; exactly 50% remains a non-selection.
Zone/filled-graphic 100%-enclosure requires the complete authored filled
area including islands inside the region polygon. A gesture below the 4px
activation threshold is not a region (§2.2.2); a degenerate zero-area world
region selects nothing.

**4. Evaluation and exhaustion.** Two budgets with distinct roles, one
shared principle — feedback may degrade, membership may arrive late,
correctness never changes:

- **Preview budget (per frame):** live marquee candidate feedback is
  time-sliced. On per-frame exhaustion the preview degrades
  deterministically — region boundary plus a running count, with dense
  preview following the §2.2.13 dense law (including union-mask fallback,
  OPEN-14). Preview degradation is presentation state and never journaled.
- **Query completion (on release):** membership evaluation runs to
  completion, spanning frames when the candidate set exceeds the per-frame
  examination bound (cf. the S3 point-query budget
  `DEFAULT_HIT_QUERY_BUDGET = 4096`, `crates/gui-viewport/src/hit.rs`; the
  region examination bound is its own governed constant in the same
  module). While evaluation spans frames the preview shows an explicit
  evaluating state. Commit yields the exact set — **all or nothing**:
  exhaustion never produces a silently partial selection, and cancellation
  (`Escape`, focus/capture loss, pane closure, content replacement —
  §2.2.2) discards the evaluation and preserves the prior selection.

**5. Auto-pan inclusion.** The region is anchored in world space (§2.2.3);
membership is computed from the **final** world-space region geometry.
Geometry revealed by auto-pan — or never rendered onscreen at any point
during the gesture — participates identically. Two gestures ending with the
same final region geometry over the same design revision produce identical
membership regardless of pan path, pan speed, or zoom changes mid-gesture.

**6. Future assertions (TO-ENFORCE; consumed by S5-C10).** Exact assertion
contract for the S5A build; test homes are indicative (`gui-viewport` region
module) and finalized at build time:

- **A1 determinism** — same revision + same final region ⇒ identical
  membership set across runs (`region_membership_deterministic`).
- **A2 pruned enumeration** — a small region on the 100k-object fixture
  examines only AABB-intersecting candidates; examined count is asserted
  against the index-pruned bound, never total object count
  (`region_query_prunes_by_aabb`).
- **A3 oracle exactness** — rect and lasso membership equal a brute-force
  per-class oracle on the 100k fixture (`region_membership_matches_oracle`).
- **A4 auto-pan equivalence** — a panned gesture and a directly-specified
  equal final region produce identical membership
  (`region_autopan_membership_equivalence`).
- **A5 atomicity under pressure** — commit under budget pressure yields the
  exact set; cancellation mid-evaluation preserves the prior selection
  (`region_commit_atomic_under_budget`).
- **A6 per-class qualification** — majority-anchor, both-endpoints/2-of-3,
  100%-enclosure, oriented-rect >50% (including exactly-50% rejection), and
  point-anchor cases per §2.2.4/§2.2.16
  (`region_class_qualification_cases`).
- **A7 non-authored exclusion** — regions never acquire proposal, review,
  or diagnostic subjects regardless of overlay density
  (`region_excludes_non_authored`).

#### 2.2.18 S5-C03 selection and focus lifetime contract

<!-- EVIDENCE:UVT-S5-SPEC:S5-C03-CONTRACT -->

This contract defines what happens to a selection — and its optional compound
focus member — when the world changes underneath it. One principle governs
every rule: **selection is consumer state holding stable authored identities;
it is a projection over the current resolved model, never a snapshot of past
geometry, and it never resolves ambiguity by guessing.**

**1. Identity anchoring.** A selection member is a stable identity
(`ObjectId`-backed authored id; resolved net/bus identity for semantic
subjects; action/finding identity for non-authored subjects) — never an
index, a coordinate, or a geometry snapshot. Selection state records the
`model_revision` against which it last resolved.

**2. Re-resolution on every revision.** Every `model_revision` change — from
any surface: GUI commit, CLI, MCP, agent, undo/redo — triggers re-resolution
of the selection against the new resolved model:

- an identity that still resolves **stays selected**, projecting the new
  geometry/properties (identity-preserving edits keep selection by
  substrate law: mutating operations preserve `ObjectId`);
- an identity that no longer resolves is **dropped** — silently on canvas,
  **reported** in the Inspector (membership counts update; a compound
  subject reports members removed by revision); no phantom members ever
  survive re-resolution;
- **no substitution and no resurrection:** delete-and-recreate produces a
  new identity that is never auto-selected in place of the old one, and an
  identity restored by undo does **not** re-enter a selection it was
  dropped from — undo restores model state, never consumer selection
  state. Selection resurrection would couple consumer state to the journal
  and make selection content depend on edit history rather than on the
  four-input exactness law (§2.2.17).

**3. Semantic subjects re-derive wholly.** A Global Net or Bus subject
stores the resolved net/bus identity only; its member projection is derived
fresh at each revision and never cached across revisions. If the identity
survives (rename with stable id: survives), membership follows the new
resolution; if the net/bus ceases to resolve, the subject dissolves to an
empty selection, reported. Members are never individually dropped from a
semantic subject — the projection is always exactly the current resolution.

**4. Compound focus.** Focus is a reference to a current member. If the
focused member is dropped by re-resolution (or the member leaves the
selection), focus clears to none — **no auto-promotion** of another member
(consistent with the ratified conformance row: removing focus promotes
none). Focus never survives the member it referenced.

**5. Pane and document lifecycle.** Selection is project-workspace state
(§2.2.11), not per-pane state:

- closing a pane never clears or reduces the selection — remaining panes
  keep projecting it;
- replacing a pane's **content** (board↔schematic, sheet navigation)
  cancels any active gesture in that pane (§2.2.2) but preserves the
  committed selection; the new content projects whatever members it
  resolves;
- replacing the **project/document** drops the selection entirely —
  selection identities are scoped to one resolved project and never leak
  across projects; there is no cross-project selection state.

**6. Stale identities never fault.** A selection identity arriving from any
consumer (saved workspace state, terminal/AI context payload, cross-session
restore) that does not resolve against the current model is dropped with a
reported count — never an error, never a crash, never a partial-resolution
guess, never a fuzzy match. The only lawful responses to an unresolvable
identity are *drop and report*.

**7. Partial cross-pane resolution.** Each pane projects exactly the members
its content resolves; a pane that resolves none of the selection shows no
projection — this is normal state, not an error. The Inspector reports the
selection's **total** membership independent of any single pane's
resolution, with hidden/unresolved-in-view members carried as counts
(§2.2.13 hidden-member law). The selection's identity and membership are
global; only its *projection* is per-pane.

**8. Non-authored subject lifetime.** Per OPEN-12: a proposal-action
selection dissolves when its action commits or is discarded; a check-finding
selection dissolves when a check run invalidates its fingerprint (a new run
reproducing the same fingerprint preserves selection). Dissolution clears to
empty and is reported. Non-authored lifetimes are bound to their producing
artifact's lifecycle, never to the design journal.

**9. Future assertions (TO-ENFORCE; consumed by S5-C10).** Test homes
indicative, finalized at build time:

- **L1 revision re-resolution** — surviving identities stay selected with
  updated projection; dropped identities disappear with Inspector report
  (`lifetime_reresolution_on_revision`).
- **L2 no phantom members** — post-revision selection never contains an
  unresolvable identity (`lifetime_no_phantom_members`).
- **L3 identity-preserving edits** — property/geometry mutation preserves
  selection membership (`lifetime_edit_preserves_selection`).
- **L4 no resurrection** — undo of a deletion does not restore dropped
  members; recreate does not substitute (`lifetime_no_resurrection`).
- **L5 semantic re-derivation** — net/bus subjects derive membership fresh
  per revision; dissolve-to-empty when unresolvable
  (`lifetime_semantic_subject_rederivation`).
- **L6 focus no-auto-promotion** — dropping the focused member clears
  focus to none (`lifetime_focus_clears_no_promotion`).
- **L7 pane lifecycle** — pane close preserves selection; content
  replacement cancels gestures but preserves committed selection; project
  replacement drops it (`lifetime_pane_document_lifecycle`).
- **L8 stale identity safety** — unresolvable incoming identities drop
  with report, no fault, no fuzzy match (`lifetime_stale_identity_drop`).
- **L9 partial projection** — per-pane projection equals per-pane
  resolution while Inspector totals stay global
  (`lifetime_partial_pane_projection`).

#### 2.2.19 S5-C04 compound output contract

<!-- EVIDENCE:UVT-S5-SPEC:S5-C04-CONTRACT -->

This contract defines everything a selection projects **out** — to the
Inspector, to the action console's textual echo, and to the typed context
payload consumed by the terminal back-door and AI collaborators. §2.2.10
owns how membership and focus are formed; this section owns what consumers
see. It depends on no part of the deferred Application Status Bar.

**1. One projection, three consumers — shared by construction.** The
Inspector view, the console echo, and the context payload all derive from
**one typed selection-projection structure**. No consumer computes its own
membership summary; divergent counts between surfaces are a defect by
definition. (The status bar, if later retained, becomes a fourth consumer
of the same structure — never a rival computation.)

**2. Single-subject projection.** Exactly one member: the Inspector subject
is that object — identity header (reference, kind, `SELECTED` chip) with
Identity/Placement/Checks sections and the class-specific rows ratified in
the §2.2.16 matrix. Semantic subjects project their own views: Global Net →
Net/Members/Checks(ERC) with per-kind and hidden-member counts; Bus →
member nets and hidden/cross-sheet occurrence counts (§2.2.13). Non-authored
subjects project their producing artifact (review action, check finding)
per OPEN-12.

**3. Compound projection.** Multiple members form the temporary compound
subject (`Compound Selection — N objects`, §2.2.10) with:

- **Scopes:** `All N` plus one per-type scope per member class present
  (`Parts 6`, `Traces 14`, …). Scope selection is a **view filter** over the
  compound — it never changes selection membership. Per-type scopes expose
  the class field tables ratified in the matrix (including OPEN-9 schematic
  text); `All N` exposes only class-independent aggregates.
- **Field states, exact semantics:** **Common** — every in-scope member has
  the same typed value; the value is shown. **Mixed** — the field is typed
  for every in-scope member but values differ; shown as `Mixed`, NEVER an
  arbitrary member's value. **Unavailable** — the field is not typed for
  every in-scope member (or is blocked); shown with the reason. S5A renders
  all field states read-only.
- **Aggregates:** member count and types; combined world-space bounds of
  member projections; workspace/layer/net coverage; and the expandable
  stable-identity member inventory (§2.2.10).
- **Blocker reporting:** hidden count, locked count, and
  incompatible/blocker count each carry an **exact explanation** — which
  members and why — matching the §2.2.7/2.2.8 whole-refusal law so a later
  refused mutation is never a surprise the outputs failed to disclose.

**4. Stable-ID context payload.** The typed envelope
(`DatumSelectionContext`, `crates/gui-protocol/src/context_envelope.rs` —
today a singleton `{kind, id}`) extends to the full subject vocabulary as
**one serializable payload shared verbatim by the terminal back-door, the
action console, and AI context**:

- `subject_kind`: `none` | `authored_object` | `compound` | `global_net` |
  `bus` | `review_action` | `check_finding`;
- `subject_id`: the semantic/artifact identity where the kind has one;
- `model_revision`: the revision the projection was resolved against
  (§2.2.18 anchoring);
- `summary`: total member count, per-type counts, hidden and locked counts;
- `focus_id`: the optional focus member;
- `member_ids`: the full stable-identity list in deterministic
  stable-identity order (§2.2.10) up to a **governed cap of 256 ids**;
  beyond the cap the list is omitted, `truncated: true` is set, and
  consumers rely on `summary` or query explicitly — the payload is bounded
  by contract, never by silent cutoff.

The payload is a read-only observation: carrying it grants no mutation
authority; any consumer acting on its identities goes through typed
operations whose targets re-validate at commit time under the §2.2.18
stale-identity law (drop-and-report, never fuzzy match).

**5. Future assertions (TO-ENFORCE; consumed by S5-C10).** Test homes
indicative:

- **O1 single-source projection** — Inspector, console echo, and envelope
  derive from one structure; counts can never diverge
  (`outputs_single_projection_source`).
- **O2 field-state oracle** — Common/Mixed/Unavailable computed correctly
  against a brute-force oracle over mixed compounds
  (`outputs_field_state_semantics`).
- **O3 scope filtering** — per-type scopes expose exactly the in-scope
  class fields; scope switching never mutates membership
  (`outputs_scope_view_filter`).
- **O4 blocker disclosure** — hidden/locked/incompatible counts carry exact
  member/reason explanations consistent with later whole-refusal
  (`outputs_blocker_reporting`).
- **O5 envelope bound + determinism** — stable ordering, 256-id cap with
  `truncated` marker, serialization round-trip
  (`outputs_envelope_bounded_deterministic`).
- **O6 status-bar independence** — every output is complete with no status
  bar present (`outputs_no_status_bar_dependency`).
- **O7 read-only surface** — S5A Inspector/envelope expose no mutation
  entry point (`outputs_readonly_boundary`).

#### 2.2.20 S5-C07 selection identity and cross-probe contract

<!-- EVIDENCE:UVT-S5-SPEC:S5-C07-IDENTITY -->

This contract defines what a selection **is** — the Layer-2 selection-identity
model the numbered decision (S5-C12) ratifies. It supersedes every singleton
reading in §2.2: wherever older wording says "the selected object", it now
means "the selection subject" as defined here. The substrate's current
singleton `SelectionTarget` (`None` / `AuthoredObject(String)` /
`ReviewAction` / `CheckFinding`, `crates/gui-protocol/src/lib.rs:440-445`) is
the acknowledged predecessor this vocabulary replaces in the S5A build.

**1. Subject vocabulary.** The project-workspace selection is exactly one
subject at a time, drawn from a closed typed vocabulary:

- **None.**
- **Object** — one stable authored identity of any §2.2.16 matrix class,
  carrying its typed class (the flat-string era ends: port≠label,
  track≠graphic at the type level).
- **Compound** — a set of two or more stable authored identities with
  optional focus (§2.2.10). Membership is enumerated, not derived.
  Non-authored identities are not members (OPEN-12).
- **Run** — a derived connectivity scope: the physically continuous
  connected component (copper run, wire run, or bus run) through an origin
  section, per the OPEN-1 origin classes. Identified by origin kind +
  resolved derivation; membership is **derived per revision**, never an
  enumerated set (§2.2.18.3 re-derivation law applies).
- **Global Net** — the resolved net identity; membership derived per
  revision across all workspaces (§2.2.13).
- **Bus** — the semantic hierarchical bus identity (§2.2.13, OPEN-8);
  bus sections are Object subjects and bus runs are Run subjects with a bus
  origin — three tiers, three subject kinds, one ladder.
- **Proposal** — a review-action identity; **Review** — the same action
  addressed through its evidence surface (evidence children are never
  independent subjects); **Diagnostic** — a check-finding fingerprint.
  All three are non-authored: explicit-only acquisition (OPEN-12),
  lifecycle-bound (§2.2.18.8).

Enumerated (Object/Compound) and derived (Run/Global Net/Bus) subjects
differ in exactly one way: derived subjects store an identity and re-derive
membership at each revision; enumerated subjects store identities and drop
what stops resolving. Both obey the same §2.2.18 lifetime law.

**2. Same-identity vs merely-related — the formal distinction.**

- **Same-identity projection:** two pane renderings project the same
  subject iff they resolve the **identical subject identity** at the same
  `model_revision` — one identity, two cameras. Every resolving pane
  renders the full selection treatment at identical strength (§2.2.11;
  RB §2.1).
- **Merely-related mapping:** a relation between **different identities**
  — two identities, one relation. Related geometry keeps its exact
  authored baseline: no accent, no glow, no lift, not counted, no handles;
  direct selection promotes it normally (RB §2.3).

The known mappings, classified once:

| Mapping | Classification |
|---|---|
| Net member geometry across panes (tracks/wires/vias/pads/junctions/labels of one resolved net) | same-identity (one Global Net subject) |
| Semantic bus spine/name/entries across sheets | same-identity (one Bus subject) |
| Component ↔ placed symbol (cross-probe) | merely-related (binding relation) |
| Library definition ↔ placed instances | merely-related (OPEN-6) |
| Bus ↔ scalar member nets | merely-related (§2.2.13) |
| Net ↔ parent footprint/symbol bodies | merely-related (§2.2.13) |
| Check finding ↔ its target authored object | merely-related (navigation aid, §2.2.16) |
| Proposal ↔ the authored geometry it would produce | merely-related (uncommitted overlay) |

No mapping may be reclassified by a build; adding a mapping requires adding
it here.

**3. P2.3 alignment.** The Phase-2 cross-probe slice (P2.3, "one selection
identity projected into both panes") is the first build consumer of this
model and **depends on completed S5A** — the typed subject vocabulary, the
schematic selection substrate (S3+S5A), and the same-identity projection
law — not merely on this specification. P2.3's board↔schematic behavior is
exactly: same-identity subjects project fully in both panes; the
component↔symbol mapping renders related-context only, with the Inspector
explaining the relation (`DATUM_GUI_PHASE_2_SPEC.md` P2.3 reads under this
contract).

**4. Substrate obligations (S5A build).** The typed successor of
`SelectionTarget` must represent: class-typed object identities (restoring
the port class lost to `Label` typing and freeing `AuthoredObject` from flat
strings), compound sets with focus, derived Run/Global-Net/Bus subjects,
and the non-authored kinds — matching the §2.2.19 envelope `subject_kind`
vocabulary one-to-one. The §2.2.16 scene-authority column enumerates the
per-class gaps this replaces.

**5. Future assertions (TO-ENFORCE; consumed by S5-C10).**

- **I1 vocabulary round-trip** — the typed subject enum, the envelope
  `subject_kind`, and this section's vocabulary are one-to-one; no consumer
  invents a subject kind (`identity_vocabulary_roundtrip`).
- **I2 same-identity equality** — a subject resolving in N panes renders N
  full-strength projections of one identity; deselecting in any pane clears
  all (`identity_same_projection_equality`).
- **I3 related never selected** — merely-related geometry receives no
  selection token/treatment under any mapping in the table
  (`identity_related_baseline_only`).
- **I4 derived determinism** — Run/Global-Net/Bus membership derivation is
  deterministic per (identity, revision) (`identity_derived_membership`).
- **I5 P2.3 gate** — the cross-probe build consumes the typed vocabulary
  (structural: no P2.3 code path predates the S5A subject types)
  (`identity_p23_consumes_vocabulary`).

##### Open-reconciliation register (S5-C01)

<!-- EVIDENCE:UVT-S5-SPEC:S5-C01A-RESOLVED -->
**GATE CLOSED (owner session, 2026-08-14): all fourteen OPEN choices are
RESOLVED below** — OPEN-1 revised; OPEN-2 through OPEN-9 and OPEN-12 through
OPEN-14 approved as recommended; OPEN-10 and OPEN-11 ratified deferrals with
recorded re-entry conditions. Each entry carries its resolution, reason, and
normative recording; the named propagation steps (C02–C09) reconcile the
approved choices into their contracts and evidence. S5-C11 only performs
final review and cannot introduce a new choice after C02–C10 are complete.

Every `OPEN-n` below is an explicit, tracked reconciliation decision. Each
carries a research-grounded candidate resolution; none is silently resolved.

<!-- OWNER:UVT-S5-SPEC:S5-C01A:OPEN-1 -->
- **OPEN-1 — ladder origins.** §2.2.5 defines the section→run→net ladder for
  clicks on track/wire sections only; via / pad(footprint) / zone-fill /
  label click-origins are unstated. *Candidate:* ratify ladder origins as
  track/wire sections exclusively; clicks on other net members stay
  object-only (matches the §2.2.5 wording; keeps one predictable origin
  class). *Decision:* S5-C01A. *Propagation:* S5-C07.
  *RESOLVED (owner, 2026-08-14): revised* — the ladder originates on all
  **directly-selectable conductive geometry**: track/wire sections, vias,
  and zone/pour copper (recorded as the §2.2.5 origins clause). *Reason:*
  the run tier is well-defined from any continuous-copper origin (the
  physically connected component through the click point), so the
  track/wire-only candidate was needlessly restrictive; the owner's intent
  is "triple-click any net copper selects the whole net". **Pads and
  labels/ports stay excluded** for structural, not stylistic, reasons: a
  pad's tier-1 click is ratified as the parent footprint (§2.2.4), so
  ladder tiers cannot nest from it; a label's double click is reserved for
  S5B edit-in-place. Both keep net acquisition through the explicit
  `Select Net` verb — a gesture-scoping rule, not a capability boundary.
<!-- OWNER:UVT-S5-SPEC:S5-C01A:OPEN-2 -->
- **OPEN-2 — board-workspace component-owned graphics.** Ownership of
  component-owned graphics in the board workspace is spec-silent.
  *Candidate:* ratify the current code behavior — sub-graphics hit-map to
  the parent component; Edge.Cuts component graphics excluded from hit
  regions. *Decision:* S5-C01A. *Propagation:* S5-C07/S5-C08.
  *RESOLVED (owner, 2026-08-14): approved as recommended* (recorded as the
  §2.2.4 component-owned-graphics clause). *Reason:* the parent collapse is
  the field consensus (KiCad/Altium select the owning footprint) and extends
  the already-ratified §2.2.4 ownership model — on the board, the footprint
  is the object; child graphic editing is definition-editor authority. The
  Edge.Cuts hit exclusion protects board-level outline authority: a
  footprint contributing outline geometry must never capture outline clicks.
  Useful consequence: silk-dense, pad-sparse parts become clickable by
  their silk while the region test stays pad-anchor-based.
<!-- OWNER:UVT-S5-SPEC:S5-C01A:OPEN-3 -->
- **OPEN-3 — owned text beyond reference/value.** The editors' owned-text
  independence is ratified for reference/value only; user/fab owned text is
  silent. *Candidate:* same independent-target + oriented-rect rule for all
  owned text kinds in the definition editors. *Decision:* S5-C01A.
  *Propagation:* S5-C04/S5-C07.
  *RESOLVED (owner, 2026-08-14): approved as recommended* (recorded as the
  §2.2.4 all-owned-text clause). *Reason:* manual-first makes definition-
  editor selectability non-optional — unreachable-by-hand text cannot be
  authored; one text qualification law avoids per-kind divergence (Lean);
  KiCad/Altium library editors select all owned text without role
  distinction. Board/schematic parent-projection behavior is unchanged for
  every kind. Hidden-field semantics stay with OPEN-13; the field table
  stays a dedicated tool.
<!-- OWNER:UVT-S5-SPEC:S5-C01A:OPEN-4 -->
- **OPEN-4 — Footprint/Symbol Editor child region rules + composition.**
  Pad region qualification, pin-compound region qualification, and
  pad-number-text composition are unstated. *Candidate:* point-like
  center/connection-anchor rule for pads; anchor-qualifies-the-compound for
  pins; pad geometry + pad-number text as one subject (RB §2.7 pin analogy).
  *Decision:* S5-C01A. *Propagation:* S5-C02/S5-C07.
  *RESOLVED (owner, 2026-08-14): approved as recommended* (recorded as the
  §2.2.4 definition-editor child clause). *Reason:* §2.2.4's architecture is
  deliberately anchor-based — area tests exist only as ratified exceptions
  (zones/filled at 100%, text oriented-rect) — so pad center-anchor and pin
  connection-anchor keep the one qualification family and stay deterministic
  at density; the connection anchor is the pin's only unambiguous single
  point and text bounds must never drive selection; pad-number text is the
  pad's identity rendering (pin construction analogy) and is excluded from
  the owned-text class. Known trade accepted: a large pad whose center lies
  outside a region does not qualify.
<!-- OWNER:UVT-S5-SPEC:S5-C01A:OPEN-5 -->
- **OPEN-5 — editor-workspace owned graphics.** Whether the generic
  path/filled rules extend to Footprint/Symbol Editor owned graphics (and
  schematic drawing cross-pane/overlay specifics). *RESOLVED (owner,
  2026-08-14): approved as recommended* (recorded as the §2.2.4
  definition-editor owned-graphics clause). *Reason:* manual-first — silk
  artwork and symbol bodies cannot be authored without selecting them;
  reuses the ratified generic predicates instead of inventing per-editor
  rules; completes the uniform child law (parent projection in placement
  workspaces, independent target in the definition editor); the symbol-body
  fill is the authored object in its editor while the schematic-side
  dark-fill law continues to describe whole-symbol selection. Substrate
  remains honestly absent (no editor surfaces; graphics identity
  convergence pending per COMP). *Candidate was:* extend the
  generic §2.2.4 rules as independent authored targets in definition
  editors. *Decision:* S5-C01A. *Propagation:* S5-C02/S5-C07/S5-C08.
<!-- OWNER:UVT-S5-SPEC:S5-C01A:OPEN-6 -->
- **OPEN-6 — definition↔instance projection.** Whether selecting a
  pad/text/graphic/pin *definition* in a library editor projects onto placed
  instances. *Candidate:* merely-related under RB §2.3 (exact authored
  appearance, no accent), never same-identity. *Decision:* S5-C01A.
  *Propagation:* S5-C07.
  *RESOLVED (owner, 2026-08-14): approved as recommended* (recorded as the
  §2.2.11 definition↔instance clause). *Reason:* definitions and instances
  are different identity domains joined by a library binding — same-identity
  projection is reserved for one resolved design object in multiple panes;
  the tighter component↔placed-symbol coupling is already ratified
  merely-related; a selection cue on instances would falsely imply instance
  mutation authority (definition edits propagate through library-update
  flows, not selection); class-wide glow (every 0402 pad instance) is
  Global-Net-scale noise for a local edit. Usage lives in the Inspector
  ("used by N instances") and the explicit relationship view. S5-C07 folds
  this into the formal same-identity vs merely-related definition.
<!-- OWNER:UVT-S5-SPEC:S5-C01A:OPEN-7 -->
- **OPEN-7 — junction Global-Net membership.** Junctions are absent from the
  enumerated net-member list. *Candidate:* include the junction dot as an
  owned member of the Global Net projection (it is resolved conductive
  geometry). *Decision:* S5-C01A. *Propagation:* S5-C07/S5-C08.
  *RESOLVED (owner, 2026-08-14): approved as recommended* (recorded in the
  §2.2.13 member enumeration). *Reason:* the member list's criterion is
  visible resolved electrical representation, and the authored junction dot
  qualifies plainly — the list already includes derived ratsnest geometry
  and pin terminals/stubs, which are equally rendering conventions of
  connectivity; an unglowed dot amid glowing wires reads as a rendering
  defect against the whole-subject coherence law. The junction keeps its
  ratified object-only direct-acquisition behavior — dual role as with
  vias; tiny-object and dense-LOD laws cover the cost.
<!-- OWNER:UVT-S5-SPEC:S5-C01A:OPEN-8 -->
- **OPEN-8 — bus region rule + hierarchy projection.** Region qualification
  for spine/entries and member-pane specifics of semantic-bus projection.
  *Candidate:* generic path rule for spine and entries; hierarchy projection
  follows the Global Net cross-pane law with bus-distinct typing. *Decision:*
  S5-C01A. *Propagation:* S5-C02/S5-C07/S5-C08.
  *RESOLVED (owner, 2026-08-14): approved as recommended* (recorded as the
  §2.2.13 bus clause). *Reason:* pure rule-reuse — path rule per section;
  entries follow the ratified owned-geometry pattern (never enlarge the
  test, never originate acquisition); the semantic tier reuses the Global
  Net projection shape (all schematic sheets/panes, Inspector count
  summarization) under bus-distinct typing so a Bus subject is never a
  disguised scalar-net list. Clarified: no board-side bus projection —
  buses resolve to individual nets on copper, keeping Bus and Global Net
  subjects distinct.
<!-- OWNER:UVT-S5-SPEC:S5-C01A:OPEN-9 -->
- **OPEN-9 — schematic-text compound fields.** The typed compound field
  table covers PCB text only. *Candidate:* mirror the PCB text row
  (style/height/stroke/alignment/visibility where typed). *Decision:* S5-C01A.
  *Propagation:* S5-C04.
  *RESOLVED (owner, 2026-08-14): approved as recommended.* *Reason:* text
  is one model program-wide (Rendering Book typography + engine text
  registry), so the Inspector field vocabulary must not fork per editor —
  the shared-backbone law; the field set carries no PCB-specific semantics
  (layer/side consequences were already dedicated); "where typed" keeps the
  vocabulary honest until typed ops exist; content editing stays a
  dedicated tool on both surfaces. Folded into the S5-C04 compound-outputs
  contract.
<!-- OWNER:UVT-S5-SPEC:S5-C01A:OPEN-10 -->
- **OPEN-10 — dimension class.** Spec-silent, no scene projection.
  *Candidate:* author path-rule qualification for extension/dimension lines
  + oriented-rect for dimension text, or formally defer the class from S5A.
  *Decision:* S5-C01A. *Propagation:* S5-C02/S5-C04/S5-C08.
  *RESOLVED (owner, 2026-08-14): deferred from S5A* (recorded in the
  dimension matrix row as a ratified deferral). *Reason:* the class is not
  even rendered — no scene projection, and no Rendering Book dimension
  construction exists — so acquisition rules would specify selection for
  undesigned geometry and could carry no honest S5-C10 check disposition
  (no golden, no state test possible); sequencing-by-product-readiness
  forbids speccing ahead of design. Re-entry condition: Rendering Book
  dimension construction + typed scene projection, expected with the
  decision-020 documentation-system spec pass (which may re-home
  dimensions in paper space).
<!-- OWNER:UVT-S5-SPEC:S5-C01A:OPEN-11 -->
- **OPEN-11 — hierarchical sheet representation.** Spec-silent, untyped.
  *Candidate:* filled-graphic-style rect qualification for the sheet body
  with sheet pins as children, or formally defer from S5A. *Decision:* S5-C01A.
  *Propagation:* S5-C02/S5-C04/S5-C07/S5-C08.
  *RESOLVED (owner, 2026-08-14): deferred from S5A* (recorded in the sheet
  matrix row as a ratified deferral). *Reason:* the subject is undesigned in
  three directions — no Rendering Book sheet construction, no typed
  sheet-pin children, cross-sheet instance identity unaddressed — and sheet
  interaction is navigation-dominated, belonging to the schematic-surface
  design pass; ratifying selection gestures before the navigation model is
  designed invites OPEN-1-class gesture collisions. Sheets render today, so
  visible-but-inert bodies are acceptable in read-only S5A. Ratified now:
  double click on a sheet body is reserved for descend-into-sheet.
  Re-entry: schematic-surface design pass + typed sheet/sheet-pin scene
  authority.
<!-- OWNER:UVT-S5-SPEC:S5-C01A:OPEN-12 -->
- **OPEN-12 — non-authored subject semantics.** Region/scope/hidden/locked/
  compound rules for proposal, review, and diagnostic subjects are declared
  SILENT by §2.2. *Decision:* S5-C01A. *Propagation:* S5-C02/S5-C03/S5-C04/
  S5-C07/S5-C08.
  *RESOLVED (owner, 2026-08-14): approved as recommended* (recorded as the
  §2.2.4 non-authored-subjects clause). *Reason:* explicit-only acquisition
  keeps the live proposal path intact and matches how workflow objects are
  addressed; region/Ctrl+A exclusion preserves the determinism S5-C02
  requires — a spatial query's result must be a function of design content,
  never of which checks ran or which proposals are pending; mixed-compound
  deferral is a lifetime problem — transient subjects cannot be bookkept by
  the §2.2.10 stable-identity model until typed scene authority and the
  S5-C03 lifetime rules exist. Nothing usable today is removed.
<!-- OWNER:UVT-S5-SPEC:S5-C01A:OPEN-13 -->
- **OPEN-13 — field visibility vs layer visibility.** Text-field visibility
  as a typed property vs §2.2.8 hidden-selection semantics; and whether
  definition-editor children carry lock attributes. *Decision:* S5-C01A.
  *Propagation:* S5-C03/S5-C05.
  *RESOLVED (owner, 2026-08-14): approved as recommended* (recorded as the
  §2.2.8 field-visibility/lock clause). *Reason:* field visibility is an
  authored, journaled design property while layer visibility is
  never-journaled consumer view state — conflating them violates the
  one-mutation-path doctrine in one direction or the other; the ratified
  formulation "canvas acquisition requires rendered geometry" makes
  invisible fields unacquirable by construction and self-maintains if a
  ghost-view editor option later renders them; definition children carry no
  lock semantics in S5A because the universal lock vocabulary is S5B's
  delivery boundary and is not faked before its substrate.
<!-- OWNER:UVT-S5-SPEC:S5-C01A:OPEN-14 -->
- **OPEN-14 — dense/tiny fallback for non-authored channels.** RB §2.8 is
  stated for selection overlay over authored geometry; application to
  proposal/review/diagnostic channels is unstated. *Decision:* S5-C01A.
  *Propagation:* S5-C08/S5-C09.
  *RESOLVED (owner, 2026-08-14): approved as recommended* (recorded as the
  §2.2.13 dense-over-non-authored clause). *Reason:* the 65,536 cap is a
  hard per-pane guarantee only if there is exactly one selection-cue budget
  wherever the cue draws — a second LOD system would allow combined
  overdraw past the bound in the worst case the cap was built for; the
  union-mask fallback replaces the selection cue channel only, never
  flattening proposal ghost, marker shape/severity, or review identity
  (the no-channel-flattening law extended to the degraded regime); the
  100k fixture gains a mandatory maximal-collision variant gated in
  S5-C09/S5-C10.

---

## 3. Snap & Quantize

Snap and quantize are two faces of one idea: mapping a free coordinate onto a
disciplined one. **Snap** is an interactive gesture — it shapes the cursor point
that becomes an argument to a typed Operation, and is itself never journaled
(UVT-006). **Quantize** is the committed batch form — a journaled Operation that
rounds already-placed geometry onto the grid. They share the same grid model and
connectivity rules; they differ only in whether the result is a live cursor or a
diff.

### 3.1 Snap-priority model (v1) — Horizon two-tier `grid → object override`

1. **Tier 1 — grid.** Round `(cursor − grid_origin)` to `grid_spacing`, per axis.
   A fine-grid modifier divides spacing by `div = 10` while held.
2. **Tier 2 — nearest object Target (overrides tier 1).** Find the nearest
   registered `SnapTarget` within `snap_radius` **screen pixels**; if one exists
   it replaces the grid point (object-snap beats grid within radius — Horizon;
   Altium "object hotspot snapping overrides a snap grid"). The cursor recolours
   on engage.

The resolver is a single ordered scan over ranked Target providers, so deferred
tiers (electrical grid, snap guides) slot in as new providers, not a rewrite:

```
resolve_snap(cursor, providers) =
    candidate ← grid_point(cursor)                 // rank 0
    for provider in providers sorted by rank desc: // higher rank overrides
        t ← provider.nearest(cursor, provider.radius_px)
        if t.is_some(): candidate ← t; break
    candidate
```

**v1 params (consumer-side, user-configurable):** `snap_radius` = **10 px**;
fine-grid `div` = **10**; grid-snap suppressor + object-snap suppressor both exist
as momentary modifiers (bindings are keymap, not spec-frozen). All gesture state,
never journaled.

**SnapFilter:** exclude the object(s) under the gesture; gate to visible layers
(+ current-layer-only toggle); a per-`SnapTargetKind` type mask (v1 all-on) — the
seam where AutoCAD-style per-type running-snaps land later.

*Disposition: TO-ENFORCE — unit tests for the ordered-scan override, screen-px
radius invariance across zoom, and SnapFilter exclusion of the moved object.*

### 3.2 Object-snap Target registry (per surface)

`SnapTarget` is a projection of resolved model truth (never an authority):

```
SnapTarget { point: Point<i64 nm>, object: ObjectId, kind: SnapTargetKind,
             surface: SurfaceRef, vertex: Option<u32> }
```

`point` is exact i64 nm; screen-px only governs eligibility radius, never the
stored value. `object` is the UUID a resulting Operation references.

- **Board (`SnapTargetKind::Pcb`):** `PadCenter`, `ViaCenter`, `TrackEndpoint`,
  `TrackVertex`, `Junction`, `PadOnGrid`.
- **Schematic (`SnapTargetKind::Sch`):** `PinEndpoint`, `WireEndpoint`,
  `WireVertex`, `Junction`, `BusConnection`, `LabelAnchor`, `NoConnect`.

*Disposition: TO-ENFORCE — a test that each surface's registry is non-empty and
Target points are exact nm; HUMAN — snapping visibly engages pins/pads.*

### 3.3 Quantize-to-grid — anchor-rounding, connectivity-preserved, no new verb

- **Anchor, not bbox, not per-vertex.** Quantize rounds each selected object's
  **placement anchor/origin** to the nearest grid node and translates it rigidly.
  Bbox-rounding shifts the anchor by a non-grid delta (misleading); per-vertex
  rounding deforms polylines and tears connected geometry (deferred as an opt-in
  `granularity`).
- **Connectivity survives because Datum nets are UUID/net-addressed, not
  coordinate-coincident.** Rounding a component's anchor drags its pins; attached
  wire endpoints ride along via the same connectivity-preserving re-solve a normal
  move performs; the wire endpoint is not independently quantized. Quantize never
  severs a connection to satisfy the grid.
- **It is an argument value, not a new verb or Operation.** Quantize =
  `datum.pcb.align_components` (and its schematic mirror) with **`reference: grid`**
  — already listed in `DATUM_GUI_PARAMETRIC_TOOLING.md`; the verb today
  (`verbs_pcb.rs:258`) exposes only `mode`. Align's `axis` param doubles as the
  quantize-axis selector (`horizontal` = round X only, `vertical` = round Y only,
  omitted = both).

The journaled Operation is the **same guarded position batch align already emits**;
`reference: grid` resolves to concrete `new_point`s at the verb/facade edge, taking
the grid as explicit already-quantized nm args so the op is replayable without live
UI state:

```
align_components { path, components: [uuid,…], op: align,
    reference: grid,                       // NEW enum value on the existing param
    axis: horizontal | vertical | <both>,  // reused as the quantize-axis selector
    grid_origin: Point<i64 nm>, grid_spacing: Vector<i64 nm> }  // recorded in provenance
```

Any locked, stale, incompatible, constrained, or invalid member causes an
explained whole-operation refusal by the shared batch guard (§2.2.14 atomic
refusal law); quantize never skips a selected member or partially succeeds.

*Disposition: TO-ENFORCE — verb-registry parity (the new `reference: grid` value +
menu verb) and a test that quantize preserves net connectivity; owner sign-off on
semantics before the slice lands (UVT-006).*

### 3.4 Deferred behind the interface (additive, no rework)

Altium electrical grid (a Target provider at its own rank + radius); snap guides
(a higher-rank provider); dual-axis snap distance (`snap_radius` → `{x,y}`);
AutoCAD per-type running-snap toggles (the SnapFilter type mask); per-vertex/bbox
quantize `granularity`. Each was checked against §3.1–§3.3 to confirm it is a later
addition, not a redesign.

---

## 4. Weight-Class Policy

> Resolves the "strokes thicken on zoom-in" defect: schematic grid, wires, and
> symbol strokes are baked in world-nm against a fixed reference projection and
> then re-scaled by the live camera. A **second** latent defect: `world_stroke_nm`
> (`geometry.rs:263`) floors `.max(1.0)` in **nanometres** — a no-op (1 nm is
> invisible) — so the intended min-width clamp never fires. This section assigns
> every primitive a weight class and fixes the widths, floors, and the single LOD
> threshold, cross-checked against `render==CAM` Law 1 and grid readability.

### 4.1 The three classes

- **A — `ScreenConstant(px)`** — fixed device-pixel weight, resolved every frame
  against the live camera, never emitted into a retained world buffer. Chrome only.
- **B — `WorldWidthWithMinClamp(nm, min_px)`** — true per-object world width;
  scales with zoom (physically correct) but its **projected** width is floored at
  `min_px` device px so a thin object never vanishes zoomed out.
- **C — `AuthoredConstantNm(nm, min_px)`** — a house/importer nominal nm literal;
  renders identically to B. B vs C is a **provenance** distinction (user-owned vs
  document-default width), not a render-behaviour one.

**Invariant:** class A is the *only* zoom-invariant class. Everything representing
real document/fab geometry is B/C and must scale. The grid thickening bug is a
class-A primitive mis-implemented as world geometry; the wire thickening is a B/C
primitive frozen against the reference (not live) projection.

### 4.2 Primitive → class table

`1 mil = 25 400 nm`. Widths: device px for A (exact), nm for B/C (nominal) + `min_px`.

| Primitive | Class | Width / nominal | `min_px` | Notes |
|---|---|---|---|---|
| Grid minor line | A | 1.0 px | — | hairline; differentiate by tone, not weight |
| Grid major line | A | 1.0 px | — | heavier tone, same stroke |
| Grid axis / origin | A | 1.5 px | — | accent, still zoom-invariant |
| Grid dot / cross | A | 1.0 px | — | KiCad parity |
| Selection highlight | A | 2.0 px crisp object-shaped cue + subordinate internal glow | — | slight owned-geometry lift; semantic/material hue retained |
| Hover pre-highlight | A | 1.5 px | — | lighter than selection |
| Cursor crosshair | A | 1.0 px | — | |
| Snapped-cursor glyph | A | 1.5 px | — | at the snapped point |
| Marquee rectangle | A | 1.0 px dashed | — | |
| Copper trace | B | per-object | 1.0 px | scaling is correct (Law 1) |
| Pad / via | B | filled area | — | LOD-hide sub-pixel, no clamp |
| Board silk line | C | 150 000 nm | 1.0 px | |
| Board/silk text | B | filled-outline glyph | — | §5 RENDERING_BOOK; LOD-hide < ~6 px cap |
| Edge.Cuts / outline | C | 100 000 nm | 1.0 px | |
| Schematic wire | C | 152 400 nm (6 mil) | 1.0 px | KiCad default |
| Schematic bus | C | 304 800 nm (12 mil) | 1.5 px | top of hierarchy (§2) |
| Bus-entry | C | 152 400 nm | 1.0 px | |
| Symbol body outline | C | 127 000 nm (5 mil) | 1.0 px | below wire |
| Pin line / stub | C | 101 600 nm (4 mil) | 1.0 px | thinnest |
| Pin terminal dot | C | 300 000 nm dia | 3.0 px | symbol-stroke colour |
| Junction dot | C | 400 000 nm dia | 3.0 px | wire colour, larger |
| RefDes/Value/label text | B | filled-outline, 1.27 mm | — | LOD-hide < ~6 px cap |
| Pin-name / pin-number text | B | filled-outline (Plex Mono for numbers) | — | |
| Net-label pill | C | border 127 000 nm + fill | 1.0 px | |
| No-connect marker | C | 152 400 nm (X) | 1.0 px | |
| Power-symbol glyph | C | 127 000 nm | 1.0 px | |

**Text is never a class-A stroke** — per RENDERING_BOOK §5 all on-canvas/silk text
is **filled-outline geometry** (class B fill), governed by LOD hide-below-cap, not
a min-px clamp (which would smear a sub-pixel glyph).

### 4.3 `min_px` floor reconciliation (three roles, not one constant)

- **Grid = 1.0 px exact, class A** — an exact width, not a floor.
- **Real geometry (copper/silk/wire/pin/outline) = 1.0 px floor, class B/C** —
  applied **in device px after live projection**: `width_px = (nominal_nm ×
  live_scale).max(min_px)`. This fixes the nm-floor no-op in `world_stroke_nm`.
- **Attention overlays (selection/proposal/hover) = 2.0 px floor, class A** —
  deliberately heavier for emphasis; not the geometry floor, never applied to copper.
- Junction/terminal **dots = 3.0 px floor** (a sub-3-px disc reads as a stray pixel).

*Disposition: TO-ENFORCE — every §4.2 primitive has an assignment/consumer gate;
class-B/C width floors in device px against the live projection; HUMAN — zoom
test, grid + selection weight constant. Model-only scaffolding is not LANDED.*

### 4.4 LOD threshold (unified, one rule for both panes)

Replace the two ad-hoc `px_per_mm` cutoffs (`pads_and_layers.rs:1212`) with one
threshold on **on-screen grid spacing** `S_px = pitch_nm × 1e-6 × px_per_mm`:

- **Coarsen knee — `S_px < 20`:** advance one tier (drop minor, promote major, ×2
  pitch). Horizon's coarsen point.
- **Re-fine knee — `S_px > 80`:** step one tier finer. The 4× gap is deliberate
  hysteresis to kill tier-flicker.
- **Hide-grid floor — major `S_px < 10`:** draw no grid (KiCad `m_gridMinSpacing`).

The existing `≥8` Normal cutoff already *is* the 20-px knee (2.5 mm × 8 = 20);
retune Fine to `px_per_mm ≥ 16` (1.25 mm × 16 = 20) so both boundaries share the
one rule and the schematic pane inherits it from its own pitches.

*Disposition: TO-ENFORCE — tests that both panes hit the same 20-px knee, 80-px
re-fine hysteresis, 10-px hide floor, visible-extent clipping, overflow safety,
and the 16,384-emission budget.*

---

## 5. Grid engine

Screen-space axis rects (class A, §4) drawn per-frame against the live camera;
CROSS/DOT/LINES as one continuous `mark_size` knob (Horizon); origin marker; LOD
per §4.4. Config: pitch table (board metric 2.5/5/10 mm; schematic imperial
1.27/2.54 mm), mode SQUARE/RECTANGULAR, colours (`#141821` line grid from
RENDERING_BOOK). The board grid already renders this way; the fix is to route the
schematic grid through the same engine and stop baking it into the world buffer
(`scene.rs:357`).

*Disposition: ENFORCED (board golden byte-identical); HUMAN + TO-ENFORCE for the
schematic weight-constant-on-zoom check.*

---

## 6. Context menu (local/marking menu) build contract

The content/form is already designed — build to it, do not re-author:
`DATUM_GUI_CONTEXT_MENU_CONTENT.md` (per-object content, both surfaces),
`CONTEXT_MENU_RESEARCH.md` (HCI form: cardinal-4, ≤8/level, depth ≤2, "More…"
overflow, mark-ahead), and the CI-validated data model `menu_model.json`
(`pcb.*` AND `schematic.*` menus, gated by `check_menu_model.py`).

The runtime (`ContextMenuEngine`) must add what is missing today:
1. **Per-surface content** resolved from the focused `EditorViewport`'s hit-test +
   selection + `MenuKeyNamespace` — replacing the board-only `pcb.*` key function
   (`main.rs:3431`) and the remaining board-coordinate gate. The former
   schematic right-click fallback to pan is removed: right-click is now reserved
   for this per-surface menu even before schematic menu content lands.
2. **Multi-select = intersection** of per-type menus (`workspace().selection`,
   ignored today).
3. **Verb execution on leaf-select** — replacing `MarkingMenuItem => dismiss`
   (`main.rs:2982`). Each leaf fires its bound typed verb (tri-modal). View/read
   verbs fire immediately; **authoring verbs ride the GUI→engine write path**
   (Frontier write-path step); until then an authoring leaf is disabled/queued, not
   silently inert.
4. **Overflow list + nested `▸` sub-wheels** (unbuilt; `marking_menu.rs:114` draws a
   static "MORE…").

*Disposition: ENFORCED (`check_menu_model.py` locks the data + slot invariants);
TO-ENFORCE (a test that a schematic right-click opens a schematic menu and a leaf
fires its verb); HUMAN (radial matches `context-menu-marking-menu.html`).*

---

## 7. Coordinate readout & Application Status Bar field ownership

Add the currently-absent readout: cursor X/Y → display units, dx/dy vs a settable
origin. Its global-bar placement is **REOPENED**, not implied by the prototype:
`DATUM_APPLICATION_STATUS_BAR_GUIDANCE.md` requires fast-changing pointer/snap/
gesture feedback near the pointer-containing pane and persistent editor state
near its pane. Keyboard/tool authority still follows the focused editor. If any
pane-derived value is also mirrored globally, it must name its PaneId/surface and
must not become the sole feedback path. v1 readout vocabulary = X/Y + dx/dy +
grid + units; deferred = Z/dist/polar/Space-to-zero (KiCad full readout).

*Disposition: HUMAN + TO-ENFORCE after owner placement decision — containing-
pane projection/units, focus-versus-pointer ownership, split/duplicate-pane
ambiguity, and no-global-only feedback tests.*

---

## 8. Surface profiles (v1)

- **Board** — Pcb Target kinds; metric grid; board tool set; `pcb.*` menu; full
  board layer set. (Repointed onto the shared engines only once schematic-proven —
  schematic-first rollout.)
- **Schematic** — Sch Target kinds; imperial grid; schematic tool set (new — extend
  the tool enum, `geometry.rs:1154`); `schematic.*` menu; net-role layer set.
- **Footprint / Symbol (future)** — authored as profiles when those editors land;
  no new mechanism.

---

## 9. Research traceability

| Open question | Resolved in | Implementing slice |
|---|---|---|
| Weight-class per primitive | §4.2 | S0 (model) + S1 (grid) + per-primitive as surfaces repoint |
| min-px floor unit bug | §4.3 | S0 / S1 |
| Adaptive-LOD threshold | §4.4 | S1 |
| Snap-priority model | §3.1 | S10 |
| Object-snap Target registry | §3.2 | S10 |
| Quantize-to-grid semantics + op shape | §3.3 | S11 |
| Context-menu (already designed) | §6 | S7 |
| Coordinate-readout scope | §7 | S8 |
| Deferred richness (named/polar/electrical/…) | §3.4, §8 | post-v1, same interfaces |

---

## 10. Slice map

Delivery is staged per the campaign plan; the ordered, dependency-aware position
lives in the **Active Frontier** (`specs/PROGRESS.md`), not restated here. Spine:
S0 crate + StrokeWeightModel → S1 GridEngine (fixes the bug) → S2 CameraEngine →
**S3 CoordinateHit keystone** → {S4 hover, S5 selection+marquee, S6 tool-mode,
S7 context-menu, S8 readout, S9 layer-vis} → S10 SnapEngine → S11 quantize verb.
Each slice keeps the board visual-parity golden green (or a deliberate re-bless)
and honours source-health burn-down (decision 022).
