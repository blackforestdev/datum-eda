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

## Viewport interaction & authored properties

A viewport is directly manipulable on the sheet, and each manipulation sets an
**authored, journaled property** (a typed `Operation` through `commit()`), distinct from
the transient drag/gesture that produces it (interaction produces operations; it is not
an operation). **Three distinct controls — do not conflate them:**

- **Move** — drag the viewport **anywhere on the sheet**; sets its **position**. (Journaled.)
- **Resize** — **corner-drag sizes the viewport window only** — it **never** changes the
  content scale. A larger window simply reveals *more of the model at the current scale*;
  the frame is a window, not a zoom. (Journaled.)
- **Scale** — the **only** control for scale: the viewport's **local (marking / context)
  menu** — preset content scales **2:1 · 1:1 · 1:2 · 1:4 · …**, a **custom** scale, and
  **fit-to-window**. Scale **zooms the content** (independent of the window size). Sourced
  from the `menu_model` per-object content (decision 019 context-menu system); the chosen
  scale is a journaled viewport property. Corner-drag is never a scale gesture.

So: *move* = where on the sheet · *resize* = how big the window · *scale* = how zoomed the
content. All three are journaled (undo, provenance, diff, AI-proposal); the live drag is not.

## Scale lives on the viewport, not the title block

Because a sheet can carry **many viewports at different scales**, there is **no single
sheet-wide scale** — so scale is **omitted from the title-block on-face set** (Rendering
Book §8) **by design**. Instead, **each viewport carries its own scale label** (a
mechanical-drawing view label, e.g. `SCALE 2:1`, shown at the viewport). This is the
reason scale was deliberately left off the title block.

- *Exception (single-viewport sheets):* a fab/assembly sheet with one dominant viewport
  MAY surface that viewport's scale as a **bound field** in the title block (a
  Ref/Computed field that reads the sole viewport's scale — the field-formula layer) — a
  convenience, not a second source of truth. **The viewport is the authority for scale.**

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

## Additional capabilities (in scope for the spec)

Each of these is a projection of the model or a journaled paper-space property — coherent
with the substrate, not a bolt-on:

- **Live tables — model-projected content alongside viewports.** BOM, drill/hole table,
  layer stackup, netlist/pin tables, fabrication notes, and a **sheet index / drawing
  list** are **live projections of the model** placed on sheets like viewports: they
  auto-update and render == CAM. A cover-sheet drawing list and a fab-drawing drill table
  are the *same mechanism* as `n/N` — derived, never hand-typed.
- **Associative annotations.** Dimensions, leaders, balloons and GD&T are **bound to model
  features** (board outline, pad-to-pad, hole positions, a component for an assembly
  balloon) so they follow the model when it changes; balloon numbers tie to the BOM table.
  Static notes are allowed; associative is the default where a feature exists.
- **Annotation paper-scale.** Annotations render at a consistent **paper size** regardless
  of a viewport's content scale — a dimension in a 2:1 viewport and one in a 1:4 viewport
  both print at the same text/arrow size (annotative scale, per mechanical CAD).
- **Per-viewport display controls.** Each viewport carries a **display/render style**
  (full-colour · fab-monochrome · assembly · x-ray · dimmed-inactive), **layer/visibility
  overrides**, a **lock** (freeze position/size/scale against stray edits), and an
  optional **non-rectangular clip** boundary. This is why the *same* board reads as a
  colour review on screen and a clean fab-mono view on a fab sheet — one geometry, styled
  per viewport (render == CAM holds; style is presentation).
- **View references that track the sheet set.** Detail callouts, section marks and
  "SEE SHEET n / DETAIL A" references are **projections of the sheet set** — they stay
  correct automatically as sheets renumber or reorder (same class as `n/N`).
- **Edit-in-place — the viewport is a portal.** Double-click into a viewport to author the
  schematic/board **through** it (typed ops on the model), then back out to paper space. A
  viewport is a live window you can reach through, not a dead snapshot.
- **Released sheets are frozen to a model revision (doc-control × substrate).** A sheet in
  a **Released** state resolves its viewports, tables, and associative annotations against
  the released **`model_revision`** — a stable, provenance-backed snapshot — while
  **Draft** sheets stay live. This extends the live-vs-frozen rule (title-block dates) to
  the whole sheet and makes a released drawing package reproducible.
- **Output — plot / export / drawing package.** Sheets plot/export to **PDF / plot at true
  paper size**; a whole **sheet set exports as one package**, which *is* a doc-control
  **transmittal** (title/revision/approval blocks per IPC-D-325). render == CAM: the
  exported sheet is byte-identical to the on-screen sheet.
- **Placement discipline.** Viewports, tables and annotations snap to a sheet grid and get
  the same **align / distribute** discipline as board objects (the parametric-tooling verb
  set), and can be locked — so a multi-viewport cover sheet composes cleanly by
  construction.

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
- Object model to specify (future): `Sheet` (+ `ReleaseState` freezing a `model_revision`),
  `Viewport` (source / transform / extent / layer-overrides / display-style / lock /
  clip / intent), `Table` (model-projected: BOM / drill / stackup / netlist / sheet-index),
  associative annotation objects (`Dimension` / `Leader` / `Balloon` / `Note` / `Callout` /
  `ViewReference`), `DetailViewport`, `SheetSet` — alongside the `DrawingSheet` /
  `SheetField` / `DocumentControlProfile` from the title-block research.
- Import posture unchanged: import remains a one-time converter into model space; paper
  space is native authoring.

## Open questions (for the spec pass, owner to steer)

- **Viewport source vocabulary** — which model views are addressable (schematic sheet,
  board side, inner layer, 3D angle, panel, BOM table).
- **Scale model** — presets (2:1 · 1:1 · 1:2 · 1:4 · …) + custom + fit-to-window, set from
  the viewport local menu, scale label on each viewport (resolved above). Open finer
  points: the exact preset ladder, whether a **scale bar** graphic is offered, and
  rounding for "fit".
- **Cross-references** — detail callouts, sheet/zone references, "SEE SHEET n" resolution.
- **Freeze-on-release policy** — is a released sheet auto-frozen to its `model_revision`,
  or explicitly, and can a user "refresh" a released sheet (issuing a new revision)?
- **Annotation-scale defaults** — the paper text/arrow sizes (tie to the §8/ISO 3098
  ladder) and whether any annotation may opt into model-scale.
- **Edit-in-place UX** — how far double-click-into-a-viewport authoring goes vs jumping to
  the full editor; how it reads for AI/CLI (the op targets the model, not the viewport).
- **v1 scope** — which comes first: schematic-sheet paper space, or the fab-drawing sheet
  template? (Sequencing to be placed on the Active Frontier.)

## Governed specification pass

The next work is an owner-led architecture and specification discussion, not an
implementation authorization. It must distinguish Datum's application workspaces
and editor surfaces from the Model Space/Paper Space doctrine so that “workspace,”
“pane,” “viewport,” “model asset,” and “sheet” cannot be used interchangeably.

<!-- REQ:DOC-SYSTEM-SPEC:DOC-C01 -->
DOC-C01 inventories current code, decisions, research, and GUI prototypes for
Board, Schematic, Symbol, Footprint, panelization, and documentation surfaces.

<!-- EVIDENCE:DOC-SYSTEM-SPEC:DOC-C01-CURRENT-STATE-INVENTORY -->
DOC-C01 is complete at the current-state boundary. The detailed inventory is
`research/workspace-architecture/WORKSPACE_ARCHITECTURE_RESEARCH.md` under
“DOC-C01 current-state inventory.” It proves that the shared resolver, typed
operation substrate, recursive pane Workspace, Design assets, manufacturing
projections, and visual studies exist, while Publish authority and operations do
not. It also records the blocking legacy collisions: schematic `Sheet*` and
`sheet_id` operations versus Publish-only `Sheet`, workspace Pane versus Publish
Viewport, retired Model/Paper terminology, and technical revision versus issued
product/document revision. This evidence authorizes DOC-C02 specification only;
it does not authorize implementation.

<!-- REQ:DOC-SYSTEM-SPEC:DOC-C02 -->
DOC-C02 establishes the architectural vocabulary and authority map: what is a
workspace or editor surface, what lives in Model Space, what lives in Paper
Space, and how panes and paper-space viewports project those authorities.

<!-- EVIDENCE:DOC-SYSTEM-SPEC:DOC-C02-VOCABULARY-AUTHORITY-MAP -->
DOC-C02 is complete as a planning authority map in
`research/workspace-architecture/WORKSPACE_ARCHITECTURE_RESEARCH.md`. The map
defines Project, DesignModel, Workspace, Pane, Design Space, Editor Surface,
Design Asset, Publish Space, Sheet, ViewportDefinition, ViewportInstance,
Projected Table, Publish Annotation, SheetSet, Publish Template, Artifact, and
Product Revision Engine without overlapping ownership. It fixes the mutation
boundary: Workspace/Panes are consumer state; Design and Publish source use
distinct typed operation families through the one commit/journal authority;
Publish projections cannot write through to Design; and release meaning belongs
to the Product Revision Engine. This evidence advances object-model planning
only and does not ratify or authorize implementation.

<!-- REQ:DOC-SYSTEM-SPEC:DOC-C03 -->
DOC-C03 specifies the object and mutation model for Sheet, Viewport, projected
Table, annotation, and SheetSet, including live versus released revision
resolution and the edit-through-viewport boundary.

<!-- EVIDENCE:DOC-SYSTEM-SPEC:DOC-C03-OBJECT-MUTATION-CONTRACT -->
DOC-C03 is complete as an owner-reviewable planning contract in
`research/workspace-architecture/WORKSPACE_ARCHITECTURE_RESEARCH.md`. It defines
PublishCatalog, Sheet/PageMedia/SheetItem, ViewportDefinition/Instance,
ProjectedTable, PublishAnnotation/anchors, title-block definition/instance,
SheetSet/SheetUse, forks, templates, stable references, exact coordinate/scale
rules, typed operation families, atomicity, staleness, refusal states, and the
bounded schematic proof. Publish resolution consumes a Product Revision Engine
working configuration or immutable baseline; it does not own a rival release
state or pin one `model_revision`. `Open Source in Design` remains non-mutating
navigation and no Viewport permits Design write-through. This evidence supplies
the semantic brief for DOC-C04 visual studies and does not authorize code.

<!-- REQ:DOC-SYSTEM-SPEC:DOC-C04 -->
DOC-C04 creates or updates HTML visual studies for navigation, composition,
viewport manipulation, Model/Paper transitions, responsive states, and release
state. The owner reviews these studies before mechanism is ratified.

<!-- EVIDENCE:DOC-SYSTEM-SPEC:DOC-C04-PUBLISH-SPACE-VISUAL-STUDY -->
DOC-C04 is complete as an owner-disposition-ready visual contract in
`docs/gui/prototypes/publish-space-study.html` (commit `cfe7a60`). Frames A-H
cover Project Navigator discovery and local search, coexisting Design and
Publish panes, blank/template Sheet creation, Viewport manipulation and reuse,
annotation/source navigation, SheetSet sharing and forks, working/released
context, and responsive/accessibility states. The study preserves the approved
single-window recursive-pane shell, Direction-B title block, output-only Datum
Console, and Design/Publish authority boundary. OR-1, OR-2, OR-3, and OR-8 have
received owner disposition but await DOC-C06 ratification; the visual-only
presentations in OR-5..OR-7 and two pane-title wording candidates remain
explicitly unratified pending DOC-C05 owner disposition. The prototype is
registered as a source in exactly the
`prototype-rendering-and-publish` evidence route; this evidence neither
authorizes implementation nor defines Product Revision Engine mechanics.

<!-- REQ:DOC-SYSTEM-SPEC:DOC-C05 -->
<!-- OWNER:DOC-SYSTEM-SPEC:DOC-C05:DOC-C05 -->
DOC-C05 records the owner's choices for vocabulary, workspace topology,
viewport behavior, v1 document type, release semantics, and visual disposition.

<!-- EVIDENCE:DOC-SYSTEM-SPEC:DOC-C05-OR-1-OWNER-APPROVED -->
On 2026-08-23 the owner approved Frame A's sibling-group structure with the
naming and identity clarification recommended during review: `Publish Sets`,
`All Sheets`, and `Saved Viewports` are sibling groups beneath Publish. `All
Sheets` is the sole authoritative project-wide Sheet collection. Publish Sets
contain visibly distinct references to those Sheets rather than additional
Sheet rows or authoritative homes. Creation and search remain on the relevant
local node menu. This is a DOC-C05 owner disposition for later DOC-C06
ratification, not implementation authorization.

<!-- EVIDENCE:DOC-SYSTEM-SPEC:DOC-C05-OR-2-OWNER-APPROVED -->
On 2026-08-23 the owner approved Frame C's Sheet-creation choreography. `New
Blank Sheet` asks for media and orientation and creates a genuinely empty Sheet
with no Viewports, title block, or template residue. `New Sheet from Template`
creates a Sheet prepopulated with the selected template's title block and
starter content. The result remains freely editable; a template accelerates
creation but imposes no lasting structural restriction. This disposition does
not reopen the already settled support for both blank and template entry, and
does not authorize implementation.

<!-- EVIDENCE:DOC-SYSTEM-SPEC:DOC-C05-OR-3-CARRIED-FORWARD -->
The owner corrected the DOC-C05 packet on 2026-08-23 because Frame E's source
navigation had already been answered: `Open Source in Design` opens the
referenced Design editor in a new adjacent pane and preserves the invoking
Publish context. The Viewport's local right-click menu is the primary doorway,
double-click remains unassigned, and the transient adjacent pane retains the
approved quick-close/restore behavior. DOC-C05 carries this disposition forward
rather than treating it as a new decision.

<!-- EVIDENCE:DOC-SYSTEM-SPEC:DOC-C05-OR-8-CARRIED-FORWARD -->
The Direction-B title-block composition was already locked in the Rendering
Book, and Claude's DOC-C04 commit `cfe7a60` records the owner's 2026-08-23 review
and approval of its compact Publish-context renditions in Frames B, C, and G.
DOC-C05 therefore carries OR-8 forward and does not reopen the locked title
block or ask for duplicate approval.

<!-- EVIDENCE:DOC-SYSTEM-SPEC:DOC-C05-OR-4-OWNER-APPROVED -->
On 2026-08-23 the owner approved both explicit parts of Frame D's amended OR-4
study (prototype commit `92d5fdd`). D4's linked, linked-with-overrides, and
unique vocabulary uses a glyph plus a word and never color alone. For D4b
appearance timing, Candidate D (`on demand`) is selected and Candidate P
(`persistent`) is rejected because permanent healthy chips consume Sheet real
estate and add visual noise. The Inspector always exposes complete state; hover
reveals the compact healthy chip; selection reveals the chip and handles while
detail remains in the Inspector. Broken-source and stale-reference findings
remain visibly persistent until resolved. These are editor overlays and never
print or export. The underlying ViewportDefinition/ViewportInstance mechanics
remain unchanged, and this visual disposition does not authorize
implementation.

<!-- EVIDENCE:DOC-SYSTEM-SPEC:DOC-C05-OR-5-OWNER-APPROVED -->
On 2026-08-23 the owner approved Frame F6's single-home reference presentation
for OR-5. `All Sheets` owns every Sheet exactly once. A user-named `Publish Set`
is an ordered selection of visibly linked references to authoritative Sheets;
the same Sheet may be referenced by multiple Publish Sets, while numbering and
ordering remain local to each set and never alter Sheet identity. Selecting a
reference resolves to the one authoritative Sheet, whose Inspector may show all
Publish Set memberships. Datum imposes no customer, fabrication, review, or
other package taxonomy.

Creating an independent composition is a separate, rare contextual action
labelled `Duplicate as New Sheet…`; it creates a new stable Sheet identity and
immediately requests a meaningful user-authored name. Any mechanically generated
name is provisional only, and lasting document-control identity must never use
`(copy)` terminology. This action is not how Publish Sets are assembled. The
internal `SheetSet`/`SheetUse` vocabulary remains an implementation candidate
for DOC-C06 reconciliation; the approved user-facing term is `Publish Set`.
This visual and naming disposition does not authorize implementation.

<!-- REQ:DOC-SYSTEM-SPEC:DOC-C06 -->
DOC-C06 reconciles the approved architecture into a dedicated governed spec,
decision 020, research, prototypes, conformance obligations, beads, and the
Active Frontier. It may select a bounded implementation successor but does not
authorize implementation by itself.
