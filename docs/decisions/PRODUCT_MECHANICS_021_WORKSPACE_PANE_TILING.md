# PRODUCT_MECHANICS_021 — Workspace Pane Tiling (the editor viewport model)

> **Status:** Ratified (owner-directed, 2026-07-09). Foundational GUI-workspace
> mechanism. Deepens the "Workspace & Mode Model" sketch in
> `docs/gui/DATUM_GUI_DESIGN_SPEC.md` into a governed model. Numbered 021 (021 was
> never previously created — the integrated-terminal/command-console idea once
> floated as 021 was folded into the design spec's "Command Surfaces" section, not
> a numbered record). Visual reference: `docs/gui/prototypes/workspace-panes.html`.

## Context / problem

Datum is a single unified workspace, not a set of separate editor windows. The
user must be able to see and work on more than one thing at once — schematic
next to PCB, a footprint editor beside the board, a datasheet PDF while routing —
without leaving the workspace. The M7 spike had a single central viewport; the
first Phase-2 slice hard-coded a fixed two-pane Board|Schematic split. Neither is
the product: the owner wants **SolidWorks-style splittable viewports** with
**tmux-style recursive nesting** (a split whose child is itself a split), managed
from the **View menu**, where any pane can hold whatever the task needs — and,
when a task calls for it, the ability to **maximize** a pane or **float** one over
the others.

The tension the owner weighed: pure tiling is simple and predictable (the tiling-
WM ideal), but sometimes you want to get the other panes out of the way, or
overlap a small reference window while working. The resolution below gives both
without a floating window-manager free-for-all.

## Decision

**The workspace viewport is a recursive tile tree, tile-first, with two bounded
overlay modes layered on top.**

- **A binary split tree.** A node is either a **Leaf** (one pane = a
  `(document, view)` pair) or a **Split** (`orientation: Horizontal | Vertical`, a
  `ratio`, and two child nodes). Binary + nesting produces *every* layout a tiling
  WM can make — including a split whose child is another split (Schematic |
  [Footprint / Board]) — with the simplest possible structure. Default is a single
  Leaf: the document you opened.
- **Panes hold `(document, view)` pairs.** Model-space documents (schematic sheet,
  PCB/board layer-or-3D, footprint editor, symbol editor) resolve over the one
  `DesignModel`; auxiliary read views (datasheet/PDF, check/DRC report,
  manufacturing output, 3D) are consumer surfaces. The **focused** leaf owns the
  Inspector, Layers, mode-tools, and the active-editor menus (context-follows-
  focus).
- **The View menu manages the tree** (the SolidWorks pattern): **Split Vertical**,
  **Split Horizontal**, **Close Pane**, **Focus Next/Previous Pane**, **Fill
  focused pane with →** (Board / Schematic / Footprint / Symbol / 3D / Datasheet /
  Check Report), and **Layout presets** (Single · Board+Schematic · …). Splitting
  divides the focused pane; closing collapses it and the sibling reclaims the space.
- **Split ratios resize by dragging the divider gutter** (direct manipulation,
  complementing the View-menu tree ops). The 1px gutter between a split's two
  children is a grab handle (widened by a small grab margin): pressing on it and
  dragging re-apportions the split — its `ratio` — live, **clamped** so neither
  child ever collapses to nothing (`PANE_RATIO_MIN..MAX`). Each split is addressed
  by its **root-to-node path** (the sequence of first/second descents), so nested
  splits resize independently and a divider drag never disturbs a sibling split. The
  ratio is workspace view state — never journaled — exactly like which panes exist,
  focus, and zoom. Dragging a gutter resizes the split; it does **not** change focus
  or run pane content interaction.
- **Tile is the foundation; two overlay modes are deliberate escape hatches, not
  the default:**
  1. **Zoom / maximize a pane** — temporarily fill the whole workspace with the
     focused pane and hide the rest, then restore the exact layout. This is the
     "get the others out of the way / minimize" need; it never destroys the tree
     (it is a transient view state over it). (tmux `zoom`; VS Code "maximize editor
     group".)
  2. **Float / detach a pane** — an explicit "detach this pane" action floats it as
     a picture-in-picture over the others (or pops it to its own OS window) — e.g.
     a datasheet hovering over the board while routing. **Opt-in and deliberate,
     never the ambient behavior**, so day-to-day work stays clean tiling.

**Build order (governed sequencing):** the tiling tree + View-menu control is the
foundation and is built first (it is what Phase-2 split-view grows into). **Zoom**
is a small addition that covers the owner's minimize instinct. **Float/detach** is
a later escape hatch, added when tiling+zoom demonstrably do not cover a real
working need — not gold-plated up front.

## How it rides the substrate (why this is Datum-shaped)

- **Panes are projections, not authorities.** A pane shows a live `(document,
  view)` over the resolved `DesignModel`; it copies nothing and mutates nothing.
  Editing in one pane updates every pane showing related model objects.
- **The pane layout is consumer view state, NOT a journaled design operation.**
  Which panes exist, their split ratios, focus, zoom, and float are **workspace/
  session state** — the same class as window layout, hover, or selection.
  Interactive/view behaviors produce operations but are not operations and are
  never journaled (CLAUDE.md ethos). The layout persists as a per-user workspace
  preference; it does not enter `commit()`/the design journal and is not design
  data.
- **Cross-probe falls out for free.** Because every pane projects the one
  `DesignModel`, selecting an object highlights its counterparts in every other
  pane that shows related objects (schematic symbol ↔ board footprint ↔ net) —
  Altium's cross-probe / Horizon's message bus, for free.
- **Mode-gated tools per pane.** Each focused pane owns its header tool strip and
  active-editor menus (schematic: wire/symbol/label/bus; PCB: route/via/zone/place;
  footprint & symbol: their drawing tools) — the gating
  `docs/gui/DATUM_GUI_MENU_BINDINGS.md` already assumes.

## Not the same as decision 020 (naming discipline)

**Workspace panes ≠ paper-space viewports.** Decision 020 viewports are fixed-scale
**projection windows onto documentation sheets** (paper space — output/
documentation; move/resize/scale as authored, journaled sheet properties).
**Workspace panes** (this decision) are the **interactive editor tiling** (screen
space — follows focus; layout is consumer state, not journaled). Same word
("viewport") historically overloaded; the specs name them distinctly —
**paper-space viewport** vs **workspace pane** — and never conflate them.

## Prior art

- **tmux** — recursive pane tiling + `zoom` (maximize) + popup overlays. The
  purist tiling reference, and even it has the two escape hatches.
- **VS Code** — tiled editor grid, "maximize editor group," and detach-to-window.
- **Altium Designer / SolidWorks** — split viewports (H/V), tiled documents/panels,
  dockable-or-floating panels. SolidWorks' split-viewport is the owner's reference.
- **Blender / Maya** — tiling "areas" with pop-out-to-floating-window.

## Consequences / relationships

- Supersedes the hard-coded fixed Board|Schematic split (first Phase-2 slice) — that
  slice becomes the **first implementation** of this model (single-pane default →
  one split → the full tree).
- Governs the Phase-2 GUI build: split-view → nested tiling → **divider-drag
  resize** → real schematic pane → cross-probe → zoom → float, sequenced in the
  Active Frontier.
- Object model (built): `WorkspaceLayout` (the pane tree), `PaneNode`
  (`Leaf`/`Split` with `ratio`), `PaneContent` (`(document, view)`), pane
  focus/zoom state, and `SplitChild`/`set_ratio_at_path` (root-to-node split
  addressing for divider-drag resize). Float state remains future — all consumer/
  workspace state, persisted as preference, never journaled.
- Reference prototype: `docs/gui/prototypes/workspace-panes.html`.

## Open questions (for the spec pass, owner to steer)

- **v1 content types** — schematic + PCB are certain; footprint/symbol editors and
  PDF/report panes land as those surfaces come online. Which are in the first cut?
- **Float mechanism** — PIP-over-the-workspace vs detach-to-OS-window (or both);
  deferred until the tiling+zoom foundation is real.
- **Layout persistence scope** — per-project, per-user-global, or named layouts.
- **Preset layouts** — which named presets ship (Single · Board+Schematic · …).

## Amendment: Full-Screen Stage (owner-directed, 2026-08-14)

<!-- REQ:VIEWPORT-FULLSCREEN:FS-LAW -->

Maximum working space is a **two-tier ladder over the same pane tree**, and it
is universal: every viewport content — board editor, schematic, footprint
editor, symbol editor, model-space, paper-space, and future pane contents —
gets both tiers by construction, because they are pane behaviors, never
per-editor features.

- **Tier 1 — Zoom (ratified above, landed in P2.1):** the focused pane
  temporarily fills the whole *workspace*; application chrome (menu bar,
  dock, Inspector, panels) remains.
- **Tier 2 — Full-Screen Stage (this amendment):** the focused pane fills the
  entire *application window* and **all chrome hides** — menu bar, dock,
  Inspector, panels, pane headers. Nothing persistent overlays the canvas;
  the viewport's own immediate overlays (selection, marquee, readout) render
  normally. The field pattern is Blender's maximize-vs-fullscreen-area pair
  and VS Code's maximize-group-vs-zen-mode pair: two distinct reaches for
  space, both transient view states.

**Invocation surfaces (three, one verb each — Lean):**

1. **Pane-header button:** every pane header carries a maximize affordance —
   click toggles Tier-1 Zoom; `Shift`+click toggles Tier-2 Stage. In Stage
   there are no headers: exit is by hotkey or `Escape`.
2. **Hotkeys:** `Shift+F11` toggles Zoom; `F11` toggles Full-Screen Stage
   (the universal fullscreen key). Both act on the focused pane and are
   instant round-trips — press again to return to the exact prior layout.
3. **View menu:** `Zoom Pane` and `Full-Screen Stage` entries in the
   data-driven menu model, same verbs, gated like every menu entry.

**Laws:**

- Both tiers are **consumer/workspace view state over the unchanged tile
  tree** — the split layout, ratios, and focus survive the round-trip
  exactly; entering and leaving is never journaled, persisted only as
  workspace preference alongside zoom/focus state.
- `Escape` in Stage exits Stage before performing any other Escape meaning
  (selection-clear ordering defers to the S5 contract once a selection
  exists: Escape clears an active gesture, then Stage, then selection).
- Stage never changes engine state, camera framing, or selection — the same
  scene, camera, and selection render at the larger size.
- OS-level window fullscreen remains the platform's own affordance and is
  orthogonal; Stage works identically in a windowed or OS-fullscreen window.

Build rider: tracked as a pane-system rider (`dat-viewport-fullscreen-stage`
bead); Tier-1 invocation surfaces and the Tier-2 stage land together on the
landed P2.1 zoom substrate.
