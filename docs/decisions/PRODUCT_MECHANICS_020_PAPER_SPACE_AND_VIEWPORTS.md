# PRODUCT_MECHANICS_020 — Paper Space / Model Space Separation & Viewports

> **Status:** Ratified (owner-directed, 2026-07-08). Foundational documentation-model
> decision. Subordinate to the substrate doctrine (model is authority; one mutation
> path; render == CAM fidelity) and pairs with the title-block system (Rendering Book
> §8) and the doc-control research
> (`research/documentation-system/TITLE_BLOCK_AND_DOC_CONTROL_RESEARCH.md`).

## Context / problem

How does Datum produce documentation — schematic sheets, fabrication drawings,
assembly drawings, drill drawings, panelization drawings, cover sheets? Two models:

- **(a) Bespoke generators** (the KiCad/Altium posture): schematic sheets are their own
  surface, the PCB is its own surface, and fab/assembly drawings are separate generated
  outputs. Documentation is fragmented; you cannot freely compose views.
- **(b) Paper space + viewports** (the AutoCAD / SolidWorks mechanical-CAD paradigm):
  the authored design lives in **model space**; the user documents it by placing
  **viewports** — live, scaled windows onto model-space assets — onto **sheets** in
  **paper space**, alongside the title block, dimensions, notes and tables.

No EDA tool offers a *general* paper space with arbitrary scaled viewports of any model
asset on one page (Altium Draftsman is the nearest — a board-documentation space — but
board-only and narrow). This is a genuine gap and a differentiator.

## Decision

**Datum separates model space from paper space.** Documentation is authored in paper
space by placing viewports — live projections of model-space assets — onto sheets that
also carry the title block (Rendering Book §8), sheet frame, dimensions, notes, tables,
and callouts.

- **Model space** = the resolved `DesignModel` and its projections: schematic sheets,
  the board (top / bottom / inner layers), 3D, panelization/array, BOM. These exist
  independent of any sheet and are the authority.
- **Paper space** = a `Sheet` (physical size + orientation) carrying a title block, a
  sheet frame, **viewports**, and annotations.
- **Viewport** = a window on a sheet showing a *view* of a model asset, defined by:
  **source** (which asset + which view), **scale** (1:1, 2:1 detail, or fit-to-window),
  **extent / crop** (a region of the model), **layer / visibility set**, **projection /
  orientation**, and **render intent**. A **detail viewport** is a cropped + scaled
  viewport with a callout linking it to its parent view (the mechanical-drafting detail
  view — new to EDA).

**Every documentation output is a sheet with viewports + title block + annotations** —
schematic sheet sets, fabrication drawings, assembly drawings, drill drawings,
panelization drawings, and cover sheets alike. The per-document-type field additions of
§8 make document types **sheet templates** (prescribed viewports + fields). Gerber/drill
exports are a viewport's content resolved at manufacturing intent.

## How it rides the substrate (why this is Datum-shaped)

- **Model is authority; a viewport is a projection.** A viewport shows a *live* view of
  the resolved model — it copies nothing and mutates nothing (same class as the GUI board
  scene, hover, or a gerber export). Edit the schematic/board and every viewport of it
  updates automatically.
- **Paper-space objects flow through the one mutation path.** Placing a sheet or
  viewport, setting a viewport's scale/crop/layers, adding a dimension or note = typed
  `Operation`s through `commit()`/journal — provenance, diff, undo, AI-proposal — but the
  *content* a viewport shows is projected, never journaled as design data.
- **Interactive vs authored.** Panning/zooming inside a viewport while editing is
  consumer-view state (not journaled); the viewport's authored scale/extent is a
  journaled property. (Interactive behaviours produce operations; they are not
  operations.)
- **render == CAM fidelity (Law 1) extends to paper space.** A viewport renders the real
  geometry through the same engine path, so on-screen == PDF == plot == the fabrication
  deliverable. A panelization scaled to fit 8.5×11 shows the *true* V-score / mousebite /
  route geometry, merely transformed.

## What it unlocks (all of the owner's examples fall out)

- A **cover sheet** with multiple viewports — elevations, a 3D view, board top/bottom —
  on one page.
- **Multiple schematics** (or multiple schematic sheets) composed onto a single sheet.
- A **schematic detail** — a cropped, zoomed viewport of part of a schematic with a
  callout — the mechanical-drafting detail view, unseen in EDA.
- PCB viewports at any scale; a **panelization scaled down to fit Letter** showing the
  V-score / mousebite / route configuration.
- Fab / assembly / drill drawings become **sheet templates** (a board viewport + relevant
  layers + dimensions + notes + title block), not bespoke one-off generators.

## Prior art

- **AutoCAD** — model space + layout tabs + viewports (the canonical paradigm).
- **SolidWorks / mechanical-CAD drawings** — projected / section / detail views at scales
  on sheets.
- **Altium Draftsman** — the nearest EDA analogue (board-documentation space with board
  views, dimensions, callouts) but board-only and limited. **No EDA tool offers a general
  paper space with arbitrary scaled viewports of schematic / PCB / 3D / panelization
  together — Datum's differentiator.**

## Consequences / relationships

- Unifies the documentation system with the title-block work: the **title block is the
  paper-space frame**; **viewports are what it frames**; the **field-formula + doc-control
  layer** (title-block research) attaches to sheets; **sheet sets** provide the `n/N`.
- Object model to specify (future): `Sheet`, `Viewport`
  (source / transform / extent / layers / intent), annotation objects
  (`Dimension` / `Note` / `Table` / `Callout`), `DetailViewport`, `SheetSet` — alongside
  the `DrawingSheet` / `SheetField` / `DocumentControlProfile` from the title-block
  research.
- Import posture unchanged: import remains a one-time converter into model space; paper
  space is native authoring.

## Open questions (for the spec pass, owner to steer)

- **Viewport source vocabulary** — which model views are addressable (schematic sheet,
  board side, inner layer, 3D angle, panel, BOM table).
- **Scale model** — named scales vs free; fit-to-window semantics; scale bar.
- **Cross-references** — detail callouts, sheet/zone references, "SEE SHEET n" resolution.
- **v1 scope** — which comes first: schematic-sheet paper space, or the fab-drawing sheet
  template? (Sequencing to be placed on the Active Frontier.)
