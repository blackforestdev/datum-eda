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
- Governs the Phase-2 GUI build: split-view → nested tiling → real schematic pane →
  cross-probe → zoom → float, sequenced in the Active Frontier.
- Object model to specify (future): `WorkspaceLayout` (the pane tree), `PaneNode`
  (`Leaf`/`Split`), `PaneContent` (`(document, view)`), pane focus/zoom/float state
  — all consumer/workspace state, persisted as preference, never journaled.
- Reference prototype: `docs/gui/prototypes/workspace-panes.html`.

## Open questions (for the spec pass, owner to steer)

- **v1 content types** — schematic + PCB are certain; footprint/symbol editors and
  PDF/report panes land as those surfaces come online. Which are in the first cut?
- **Float mechanism** — PIP-over-the-workspace vs detach-to-OS-window (or both);
  deferred until the tiling+zoom foundation is real.
- **Layout persistence scope** — per-project, per-user-global, or named layouts.
- **Preset layouts** — which named presets ship (Single · Board+Schematic · …).
