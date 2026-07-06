# Datum GUI Design Spec — The "How"

> **Status**: Governed active design spec.
> **Authority**: Realizes the *visual/interaction execution* ("how") of the GUI
> that `docs/gui/DATUM_GUI_PRODUCT_SPEC.md` (the "what") and decision 019 leave
> open, on the systems ratified by decisions **014** (layout geometry) and **015**
> (Design Book tokens). Design authored by the project owner; this doc records the
> ratified design decisions and points at the controlling visual prototype.
> **Reference prototype**: `docs/gui/prototypes/board-editor.html` (design pass 3 — split PCB|Schematic view).
> **Scope**: The concrete visual composition, density, and interaction craft of
> the Datum desktop GUI — starting with the board editor.

## Why this exists

The product spec, menu bindings, and even the Design Book specify *what* the GUI
contains and *why* — a menu tree, a component checklist, a token palette. None of
them is a **design**: the composition, visual hierarchy, density rhythm, and
interaction choreography that decide whether the interface is clean, attractive,
and intuitive. That craft is the "how," and in UI it is often more decisive than
the what/why. This spec captures it, developed together with a live HTML
prototype (the vehicle) so the two directly convey the intended interface.

## Design thesis (the pinned direction)

Datum's GUI is a **professional instrument in the pro-audio idiom** — the visual
language of Bitwig / Ableton (and plugins in that lineage: Vital, iZotope RX,
FabFilter). This is not an aesthetic accident: pro-audio apps render everything
custom (no OS-native widgets) — the exact architecture Datum chose with wgpu — so
they are Datum's real peer class. The governing rules:

1. **Flat, dark, restrained chrome. Zero decoration.**
2. **Color is meaning, never decoration.** Chrome is the Design Book's monochrome
   ramp; saturated color appears *only* on canvas content (copper/nets/layers)
   and as the single `#CE5A92` magenta selection accent. (Ableton's gray chrome +
   colored clips; Bitwig's semantic modulation color.)
3. **The canvas is the protagonist**, and controls act *on* it (Altium's Properties
   Panel + direct manipulation — the anti-dialog-per-object thesis).
4. **Dense, not cramped** — ruthless hierarchy and uniform rhythm, generous where
   it counts.

## Reference prototype

`docs/gui/prototypes/board-editor.html` is the **controlling visual reference**
for the board editor. It is built entirely from the Design Book chrome/content
tokens. This document narrates the decisions it embodies; when they disagree, the
prototype wins for visuals and this doc wins for rationale/rules — reconcile in
the same change.

## Locked decisions — board editor v1

- **Shell composition** (left→right, top→bottom): menu bar · tool rail (icon
  accelerators) · left column (Project tree over Layers) · **central board canvas
  (protagonist)** · right column (Inspector) · bottom dock (Terminal/Assistant
  tabs) · status bar. Approximate widths: rail ~46px, left ~228px, right ~300px.
- **Color-application law**: chrome uses only `bg/surface.01–03/border/text`
  tokens; the only chrome color allowed onto the canvas is `--acc` (#CE5A92) as
  selection. Copper/nets/pads/vias/ratsnest use the content tokens.
- **Selection → Inspector binding**: single selection is the primary model;
  selecting an object (magenta outline + glow on canvas) drives the Inspector and
  cross-highlights panel ↔ canvas.
- **Inspector = Properties Panel**: context-sensitive to the selection;
  inline-editable rows (no dialogs); collapsible grouped sections (Identity /
  Placement / Nets / Checks); key-value rows with mono tabular values for
  coordinates.
- **Layers panel**: Ableton colored-track-row idiom — swatch + name + visibility,
  active layer accented.
- **Type & density**: dense but legible — ~25px rows, chrome text ~13.5–14.5px,
  uppercase section labels ~11.5px with letter-spacing, `tabular-nums` on all
  coordinates/IDs. **Production UI font = IBM Plex Sans** (the prototype uses a
  system stand-in; the sandbox can't embed the binary).
- **Icon scale/style**: line/stroke glyphs — tool-rail ~20px, tree/panel ~15px;
  every tool carries a single-key accelerator; tooltips mandatory on icon-only
  controls; never icon-only for critical actions.
- **Status bar**: cursor X/Y (mm) · active tool · selection · grid · active layer
  · DRC count · model revision.

## Workspace & Mode Model

Datum is a **single unified viewport**, not a set of separate editor windows. The
user opens a document from the Project pane and the viewport enters that
document's **mode**; the mode carries its own toolset and menus.

- **Documents / modes**: schematic, PCB/board, footprint editor, symbol editor
  (library-object modes), plus read views (rules/check report, manufacturing).
  Selecting `project → Schematic` / `Board` / a library `Footprint` / `Symbol`
  switches the mode.
- **Mode-specific tools (the SolidWorks pattern)**: each mode swaps the tool rail
  and the active-editor-gated menus (schematic: wire / symbol / label / bus /
  junction; PCB: route / via / zone / place; footprint & symbol: their drawing
  tools) — exactly the gating `DATUM_GUI_MENU_BINDINGS.md` already assumes.
- **Tiling ("tmux for EDA")**: the viewport splits into panes; each pane is a
  **(document, view) pair**. This one abstraction covers both "2D + 3D of the same
  board" and "schematic here, PCB there, footprint in a PIP." Panes tile or float
  (picture-in-picture).
- **Context follows focus**: the focused pane owns the tool rail and menus, and
  the Inspector / Layers / Filters panels bind to the focused pane's document and
  selection.
- **Cross-probe over one model**: selection is engine-level, so selecting an
  object in one pane highlights its counterparts in every other pane showing
  related objects (schematic symbol ↔ board footprint ↔ net). This falls out of
  tiled panes over one shared `DesignModel` — Altium's cross-probe / Horizon's
  message bus, for free.
- **Open sub-decisions**: whether tools live in a per-pane header strip (as pass 3
  shows) vs. a global left rail that retargets on focus; PIP vs. tile-only.

Reference: `docs/gui/prototypes/board-editor.html` (pass 3) shows a PCB|Schematic
split with per-pane mode tools, context-follows-focus, and U1 cross-probed across
both panes.

## Context Menu System (right-click) — the speed surface

Designed 2026-07-06 from a two-part research pass: EDA right-click systems
(Altium / KiCad / Horizon / Eagle) for *content*, and marking/radial menus
(Autodesk Maya, Sketchbook Pro, Blender, Fluxbox; Kurtenbach & Buxton HCI work)
for *form*.

**Paradigm** — **select-first, object-verb** (KiCad / Horizon), NOT Eagle's modal
verb-first. The menu is **filtered to only the actions valid for the current
selection** (Horizon's anti-bloat discipline). Every item **emits a typed
`Operation` through the one commit path** — so the context menu is also the
natural home for AI **"Propose…"** variants and undo/provenance affordances (kept
in one submenu to preserve the short-menu discipline).

**Form — hybrid marking menu + linear overflow:**
- Right-click opens a per-object-type **marking menu** (radial). *Same gesture,
  two modes*: **hold ~280 ms → labeled wheel draws** (novice); **flick immediately
  in a direction → command fires, nothing draws** (expert, eyes stay on the
  board). The novice drag *is* the expert flick — continuous skill transfer.
- **≤8 items per wheel; the 4 cardinal (N/E/S/W) carry the most-frequent verbs**
  (only cardinals are reliably reproducible blind); diagonals carry secondary
  actions. **Never place a destructive/irreversible action on a diagonal — keep
  Delete cardinal.**
- **Nesting ≤2 levels**; any sub-wheel is **4-wide** (cardinal-only).
- **Compound marks — the exponential speedup.** Selecting *through* a submenu is
  one continuous stroke: flick toward the parent verb, then toward the child —
  e.g. up-then-right traces an inverted "L". Experts draw the whole shape blind,
  and it becomes a single muscle-memory gesture (Kurtenbach & Buxton hierarchic
  marking menus; Zhao & Balakrishnan simple-vs-compound marks). To support it, **a
  submenu spawns as a new radial at the location of the parent wedge you flicked
  toward** (offset outward in that direction — the Sketchbook Pro model), *not*
  re-centered. The parent wheel stays put (faded/blurred), so the compound stroke
  flows outward and the gesture path stays constant per object type → the whole
  drill-down becomes one memorized flick-shape rather than a slow menu descent.
- One wedge is **"More…" → a conventional linear, scannable list** (the
  magenta-group-delineated menu already mocked) for the long tail — parameterized
  actions (net classes, track widths, value pickers). Support **tear-off** so a
  chosen list floats as a palette during repetitive work.
- **Every item carries a keyboard accelerator** (manual-first hedge; a third
  eyes-free path).
- **Frozen geometry**: slot positions are constant per object type; unavailable
  actions grey **in place**, never reflow; never auto-sort by recency.
  **User-editable** (hoist your own verbs onto cardinals).

**Adoption is progressive — the gesture is never required.** The floor stays as
easy as any CAD package: a beginner *holds* to get a normal labeled menu, or uses
the menu bar / command palette / keyboard — nothing forces the flick. The flick
is an opt-in speed *reward* that pays off with repetition, so the ceiling is
faster than any competitor. This deliberately optimizes for intermediate-to-
advanced users (Datum's target segment) without a beginner penalty — the concern
"it might take getting used to" is real but opt-in, never a wall.

**Linear ordering** (the overflow and any non-radial fallback), top→bottom with
magenta group separators: object-special (frequent-first) → transform block →
grouped submenus (`Select ▸` / `Net ▸` / `Convert ▸`) → clipboard/lifecycle →
lock/visibility toggles → **Properties last (`E`)**.

**Empty-canvas menu is different** — space-level only: Paste, Select All,
`Grid ▸`, `View ▸`, `Place ▸`, global toggles (router/ratsnest/highlight). No
object verbs.

**Multi-select** — menu = the **intersection** of actions valid for all selected
objects; singular labels become "…Selected…"; Properties opens a multi-edit.

**Per-object cardinal proposals** (owner-tunable, user-editable):

| Object | N | E | S | W | diagonals |
|---|---|---|---|---|---|
| Component | Move | Rotate | Delete | Properties | Flip · Lock · Find in schematic · More… |
| Track | Route/Continue | Change Width | Delete | Change Layer | Drag · Highlight Net · Add Via · More… |
| Via | Change Size | Assign Net | Delete | Properties | Move · Highlight Net · More… |
| Net | Highlight | Hide Ratsnest | Assign Netclass | Assign Color | Select All · More… |
| Zone | Fill | Unfill | Edit Border | Properties | Repour · Add Cutout · Duplicate to Layer · More… |
| Empty | Place ▸ | Paste | Grid ▸ | View ▸ | — |

**Still to prove:** the marking-menu interaction in the prototype (delayed popup,
expert flick drawing nothing), and per-object cardinal tuning with the owner.
Reference visual: `docs/gui/prototypes/context-menu-marking-menu.html` (contiguous
semi-transparent wedges over a blurred board, auto-scaling, nested radial submenus
with parent-layer fade/blur). The AI dock-vs-overlay comparison is illustrated in
`docs/gui/prototypes/open-decisions.html`.

## Open design decisions (resolve before broader build-out)

1. **AI surface — RESOLVED (2026-07-06): both, by role.** Proposed *changes*
   render as an **inline ghost overlay on the canvas** (dimmed geometry in place,
   `Tab` accept / `Esc` dismiss) — never a chat panel stealing the canvas. The
   **conversational agent lives in the terminal/assistant lane** — the "code-agent
   for EDA" model: converse, or `Esc` and redirect, exactly like driving Claude
   Code — reconciling decisions 004/006 (assistant surface) + 005 (terminal). The
   two are complementary, not alternatives: overlay = proposal presentation on the
   canvas; terminal lane = the agent conversation. Both ship. (Note the AI need not
   be one large model — e.g. a small GPU-resident routing model handles routing;
   the surfaces above are model-agnostic.)
2. **Fonts**: embed IBM Plex Sans; choose the data-mono face (Design Book leaves
   it open).
3. **Context menus — RESOLVED (2026-07-06)** into the "Context Menu System"
   section above (hybrid marking menu + linear overflow, select-first object-verb,
   per-object cardinal verbs). Remaining: validate the marking-menu interaction in
   the prototype (delayed popup, expert flick, mark trail) and tune the per-object
   cardinal assignments with the owner.
4. **Datum visual identity**: reference frames, origins, fiducials, measurement
   styling — no Datum-specific identity yet (generic dark theme). Owner call.

## Working method

Spec ⇄ prototype, co-developed with tight back-and-forth until the pair directly
conveys the interface. The prototype is the visual source of truth; this spec is
the ratified rationale + rules. Each approved pass updates both in the same
change. Next surfaces after the board editor: schematic editor, library browser.

## Governance

Tracked in `specs/spec_governance_manifest.json` (docs/gui enforced glob) and in
the `specs/PROGRESS.md` Active Frontier (step 1's first concrete deliverable).
The prototype under `docs/gui/prototypes/` is referenced here so it is not an
orphan. Update this spec + the prototype together; never let one drift.
