# Datum Workspace Architecture Research

> **Status:** In progress — owner discussion and evidence collection
> (`DOC-SYSTEM-SPEC`, `DOC-C01`).
>
> **Purpose:** Preserve the original owner-question ledger, confirmed product
> intent, unresolved decisions, internal evidence, external prior art, and
> visual-research requirements before Datum ratifies Workspace, Design Space,
> Publish Space, or documentation-system mechanisms. `Model Space` and `Paper
> Space` are retained only where quoting the original questions or external
> precedent.

This is a research record, not an implementation specification. An answer is
not a ratified mechanism until the later DOC-C05 owner disposition and DOC-C06
governance reconciliation close. Partial answers remain visibly partial; later
research may refine a mechanism but must not overwrite the owner's intent.

## Method and ownership

- Questions are handled in their original order, one at a time.
- Necessary follow-ups retain the parent number (`1a`, `1b`, and so on) and do
  not replace the next original question.
- Each confirmed answer is recorded before discussion advances.
- Before asking an owner question, reconcile it against the authority matrix
  below. Carry forward ratified decisions and locked prototypes verbatim;
  runtime absence or drift is implementation evidence, never permission to
  reopen product intent.
- Each question packet must state (1) what is already controlled, (2) which
  source controls it, (3) any genuine conflict requiring reconciliation, and
  (4) only the residual owner choice. A broad original question may be closed
  in principle even when a narrower implementation or visual fork remains.
- Internal evidence comes from Datum code, decisions, contracts, research, and
  current prototypes.
- External claims require primary-source research across relevant EDA, CAD,
  publishing, standards, lifecycle, and collaboration systems.
- Codex owns research synthesis, specification, decision packets, roadmap, and
  beads. Claude owns any required visual source-of-truth HTML under
  `docs/gui/prototypes/*.html`; Codex supplies an evidence-backed brief and does
  not edit or stage those artifacts.

## Original 20-question ledger

Status meanings:

- **Open:** the owner has not answered the actual question.
- **Partial:** direction is established, but a named part remains unresolved.
- **Research-gated:** answering now would ratify assumptions that a required
  broader research/specification track must resolve first.
- **Boundary answered:** this specification has settled which authority owns
  the behavior and the invariant it must preserve; detailed mechanism is
  deliberately delegated to that authority's governed specification.
- **Answered in principle:** product intent is clear; research must still
  validate and specify the mechanism.

| # | Original question | Status | Current owner intent and unresolved portion |
|---|---|---|---|
| 1 | Is a Datum “workspace” a top-level professional activity—such as Schematic, Board, Library, and Documentation—or is the entire project one workspace containing switchable editor surfaces? | **Answered in principle** | The Project is the scalable overarching design authority and organizational context. It may contain one or many schematic workspaces, boards, products, variants, assemblies, shared assets, documentation/release packages, and external design references. Datum does not impose one-PCB or one-product walls. Schematic, PCB, Library, Documentation, and related capabilities are surfaces within that project context rather than mandatory isolated project silos. Final UI terminology and navigation presentation remain question 18, not a reopening of this product decision. |
| 2 | Should multiple editor types remain simultaneously visible in panes—for example Board beside Schematic—or does selecting a workspace replace the central editing surface? | **Answered in principle** | Board, Schematic, and future editor workspaces are distinct surfaces that may remain simultaneously visible in independently configurable panes. Users may split, tile, retarget, resize, maximize, or temporarily fullscreen panes according to preference and task speed; selecting one editor does not require replacing the entire central surface. Pane composition is workspace/session state, never design authority. The running Datum split shown in the owner’s 2026-08-22 capture and `docs/gui/prototypes/workspace-panes.html` are existing visual evidence. |
| 3 | Are Symbol Editor and Footprint Editor independent workspaces, or contextual editors entered from the Library/Schematic/Board workflow? | **Answered** | Symbol and Footprint are separate specialist editor surfaces because their authoring contracts are substantial, but access is local and contextual: invoke the relevant editor from the selected symbol/component/package/footprint through the local menu. By default Datum opens a new adjacent pane containing the relevant specialist editor and preserves the invoking editor beside it. A single keystroke closes that transient pane and restores the preceding pane layout. They remain directly openable for library work. This default choreography does not prevent later user-configurable placement. |
| 4 | Does “Model Space” include every authoritative design asset—schematics, boards, symbols, footprints, 3D models, panelization, and BOM data—or should each domain have its own model-space instance? | **Answered** | The original question conflated a space with the assets and editor types it can host. `Workspace` names Datum’s overall configurable working environment. `Design Space` contains authoritative engineering and asset editing through domain-appropriate editors, including spatial canvases and non-spatial editors. Individual domains retain their appropriate editor types and may have one or more independently addressable working contexts. `Model Space` is retired except when discussing external precedent or preserving this original question. Compact UI may label Design Space simply `Design`. |
| 5 | Should Paper Space be one project-wide Documentation workspace capable of referencing every model asset, or should Schematic, Board, and Manufacturing each expose their own paper-space mode? | **Answered in principle** | Datum has one universal `Publish Space` mechanism. Any supported design or artifact can be presented through a viewport; separate per-editor publishing systems are rejected. Compact UI may label Publish Space simply `Publish`. `Paper Space` is retired except when discussing external precedent or preserving this original question. |
| 6 | Existing terminology collides: EDA calls a schematic page a “sheet,” while decision 020 calls a physical publication page a `Sheet`. Would the owner accept distinct terms such as `SchematicPage` and `DrawingSheet`? | **Answered** | Continuous schematic Design Space has no page or sheet object, so no `SchematicPage` term is needed. `Sheet` belongs exclusively to Publish Space: it is an individual publishable page with a page/media definition, zero or more viewports, and publish-owned content. Publish Space manages a user-extensible collection of Sheets rather than being a Sheet itself. A Sheet may begin blank; a user-selected template may populate it. |
| 7 | Is Paper Space strictly a publication/composition surface, with all design editing requiring navigation back to Model Space? | **Answered in principle** | Yes. Authoritative design work occurs in Design Space; Publish Space is how project information is composed, controlled, and published. |
| 8 | Decision 020 proposes edit-through-viewport behavior. Should entering a viewport author through it, or transition the pane to the source Model Space? | **Answered** | A viewport remains a Publish Space projection and annotation target; it never becomes an edit-through aperture into authoritative design objects. Its local right-click menu exposes `Open Source in Design`, following the SolidWorks drawing-to-part/assembly pattern. Double-click is deliberately unassigned rather than treated as an implicit second doorway. Whether the Design editor opens adjacent or retargets a pane remains visual/usability study rather than authority semantics. |
| 9 | Should a viewport permit direct manipulation of projected model objects while remaining in Paper Space, or only manipulation of its frame, crop, scale, visibility, dimensions, and paper annotations? | **Answered in principle** | While remaining in Publish Space, manipulation belongs to viewport presentation and publish-owned content, not projected design objects. |
| 10 | Should model-space annotations and paper-space annotations be separate classes? | **Answered** | Design annotations and Publish annotations have distinct authority. Publish reuse centers on a versioned named `ViewportDefinition` and per-Sheet `ViewportInstance`, not globally shared loose annotations. `New Viewport` creates a clean unique definition; `Use Existing` creates a linked instance such as `board_XYZ_Viewport_01`; ordinary Sheet edits remain instance-local; `Edit Saved Viewport` explicitly changes the shared definition; and `Make Unique` severs only the reuse link while preserving parametric association to Design geometry. Release pinning remains questions 15/16. |
| 11 | Where should engineering dimensions live: as authoritative design intent in Model Space, documentation in Paper Space, or both with distinct authority? | **Answered** | Both spaces support dimensions with distinct, enforced authority. Design Space supports driving dimensions that constrain geometry and reference dimensions that only measure it. Smart Dimension creates a driving dimension by default; if that would over-constrain the design, Datum refuses it and explicitly offers `Create as Reference` rather than silently changing intent. Only Design Space dimensions may drive geometry under the current boundary. Publish dimensions remain associative reference documentation and cannot write back. The existing board-dimension type must be reconciled to this split. |
| 12 | Can one `DrawingSheet` freely mix schematic details, PCB views, 3D views, BOM tables, photographs, fabrication notes, and manufacturing-artifact views, or should templates restrict which source types may coexist? | **Answered in principle** | Free heterogeneous composition is required. Templates assist composition but do not impose source-type walls. |
| 13 | Should Paper Space support arbitrary blank composition, template-driven composition, or both? | **Answered** | Publish Space supports both. A new Sheet may begin blank with its page definition and zero viewports, enabling arbitrary free composition. A user-selected custom template may instead prepopulate viewports, title blocks, company graphics, logos, and other publish-owned content. Templates automate setup but do not restrict later customization. |
| 14 | Is a `SheetSet` a single ordered publication package, or can one project have several sets such as Design Review, Fabrication Release, Assembly, Service Manual, and Customer Documentation? | **Answered** | A scalable Project may contain any number of independently configurable ordered `SheetSet` publication packages. A SheetSet holds ordered references to authoritative Sheets rather than owning duplicate pages, so one unchanged Sheet may appear in several packages. Package-specific divergence uses `Fork Sheet`; inherited named Viewports remain linked until selected ones use `Make Unique`. The user or project template decides initial content, and instances remain freely reconfigurable. Verified customer redaction is tracked separately by `dat-publish-redaction-contract-wzs`. |
| 15 | Should Draft sheets always follow the live model while Released sheets resolve against an immutable `model_revision`? | **Boundary answered** | The Product Revision Engine—not the Publish object model—owns Draft binding, pinning, baselines, and revision allocation. Release resolves a complete approved product/configuration baseline across Design, libraries, rules, Publish definitions/instances, checks, manufacturing plans, artifacts, approvals, and effectivity, never merely one `model_revision`. Exact Draft behavior remains for `dat-product-revision-engine-k9f`. |
| 16 | When the model changes after release, should the released sheet remain frozen until a new document revision is deliberately created? | **Boundary answered** | An existing released configuration and its documents are immutable. Any controlled change in Design Space or Publish Space must be handled by the Product Revision Engine as successor revision work and cannot be released under the unchanged prior revision index. The engine will specify affected-document scope, allocation timing, pending-change identity, approvals, supersession, and regeneration; this documentation specification must not invent those mechanics. |
| 17 | What should Datum implement first after specification: schematic publication, fabrication/assembly drawings, or the general sheet/viewport substrate with one narrow proof template? | **Answered** | Required order is: production-accepted Product Revision Engine foundation; domain-neutral Publish Space foundation; Sheet primitive; Viewport definition/instance and projection primitives; one narrow schematic-publication end-to-end proof; then fabrication and assembly publishing workflows. Publish establishes its authority boundary, revision participation, composition coordinates/units/media, typed operations, Design/artifact references, rendering/export architecture, templates/packages, staleness, validation, and refusals before domain-specific publication drives the architecture. Sheets and Viewports are its first core capabilities rather than a rival pre-Publish subsystem. |
| 18 | Should workspace navigation be document-tab based, persistent-sidebar based, mode/persona based, or a hybrid? | **Answered** | Datum uses the hybrid already established by the visual authority: persistent left Project Navigator, pane-local content identity, recursive panes, focused-pane context, and View-menu tree operations/presets. Single-click selects; double-click or Enter opens the target in a new pane adjacent to the focused pane without evicting existing content. Project discovery remains local: either visually navigate by scrolling or right-click at the relevant Project node/region and choose `Search`. Search then receives keyboard input; each typed character incrementally narrows the matching items and updates the navigator selection. Search UI is invocation-local rather than permanently visible, results remain in the navigator, and no global Project-search surface is introduced. The navigator owns independent vertical scrolling with a right-edge scrollbar and accepts mouse-wheel and two-finger trackpad scrolling while pointed into it; those events never pan or zoom a Design/Publish pane. Design/Publish remain content classifications, never a global pane-evicting mode. |
| 19 | Should pane layouts persist per workspace and per project—for example Board remembering a Board/Schematic split while Documentation remembers sheet composition? | **Retired — obsolete premise** | The question assumes separate Board and Documentation workspaces, which conflicts with Datum’s unified Workspace and recursive pane tree. The settled model restores the user’s pane arrangement for the Project as non-authoritative workspace state; Board, Schematic, Library, Design, and Publish content may coexist in that one tree and do not trigger implicit workspace-specific layout replacement. Explicitly saved layout presets remain a separate optional capability already supported by decision 021, not an answer smuggled into this obsolete question. |
| 20 | Is the desired end state one cohesive Datum window, or may advanced users detach workspaces/sheets into additional native windows? | **Answered — prior false attribution corrected** | Datum has one cohesive native application window. The user may maximize/fullscreen one Design or Publish space or compose any required internal multi-view arrangement through the recursive pane tree, including Schematic, PCB, Publish, and other content together. Workspaces, Sheets, and panes do not detach into additional native windows, and no PiP product path exists. The contrary language introduced by draft decision 007 and the original decision 021 record was never owner-approved and must be removed from operative specifications. |

## Decision and prototype authority matrix

This matrix is the anti-drift control for the 20-question dialogue. “Owner
disposition” means approved direction awaiting DOC-C06 consolidation into the
numbered decisions; it does not outrank an unreconciled conflicting decision
record until that governance transaction lands.

| Q | Carried-forward authority | Controlling evidence | Residual work only |
|---|---|---|---|
| 1 | One scalable Project hosts many coordinated document/editor views; Workspace composition is not source authority. | Decisions 007 and ratified 019; owner scalable-project disposition | DOC-C06 terminology/topology reconciliation |
| 2 | Heterogeneous editor surfaces coexist in recursive, user-configurable panes; focus owns mutation context; layout is consumer state. | Ratified decision 021; `workspace-panes.html`; `board-editor.html`; owner disposition | Extend the content inventory to Publish without changing the pane law |
| 3 | Symbol/Footprint are real editor surfaces supported by pane content; owner selected contextual adjacent entry plus direct library entry. | Decision 021 content inventory; GUI Design Spec; owner disposition | Visual choreography and stable asset target contract |
| 4 | Workspace is UI composition; Design Space is the owner-selected authority term spanning domain editors; `Model Space` is retired product vocabulary. | Decisions 007/019/023; owner disposition | Amend legacy terminology during DOC-C06 |
| 5 | One universal publication mechanism, not per-editor publishing silos; owner-selected name is Publish Space. | Decision 020 universal mechanism; owner disposition | Replace legacy vocabulary and specify the new engine boundary |
| 6 | `Sheet` belongs only to Publish Space; the continuous schematic Design Space has no publication-page object. | Owner disposition; decision 020 is the legacy collision to amend | Engine migration from legacy schematic `Sheet*` names |
| 7 | Design authoring and Publish composition have separate authority. | Decision 020 separation plus owner disposition | Encode the non-write-through boundary in the amended decision |
| 8 | Publish Viewports never become edit-through Design apertures; `Open Source in Design` is the explicit local command. | Owner disposition superseding decision 020’s reach-through paragraph | Visual placement/return choreography only |
| 9 | In Publish, direct manipulation owns Viewport presentation and Publish content, never projected Design objects. | Owner disposition; decision 020 operation inventory to amend | Exact selection/handle/tool contract |
| 10 | Design and Publish annotations are distinct; named versioned ViewportDefinition plus per-Sheet ViewportInstance governs reuse and `Make Unique`. | Owner dispositions; decision 020 associative-annotation substrate | Formal object/operation schema |
| 11 | Design dimensions may drive or reference; Publish dimensions are associative reference documentation and never write back. | Owner dispositions; decisions 020/023 as substrate | Standards-driven dimension semantics and conformance |
| 12 | A Sheet freely mixes supported projections, tables, images, and Publish annotations; templates do not impose domain walls. | Decision 020 content breadth; owner disposition | Supported-type inventory and export proof |
| 13 | Publish supports both blank and template-seeded Sheets; templates remain editable accelerators. | Decision 020 template direction; owner disposition | Template schema and starter-content research |
| 14 | A Project may own many SheetSets; Sheets can be shared; package variants use Fork Sheet and Make Unique; security-grade redaction is separate. | Owner dispositions; decision 020 package substrate; `dat-publish-redaction-contract-wzs` | Formal package/variant/redaction contracts |
| 15 | Draft binding, pinning, and baseline resolution belong to the Product Revision Engine, not a Sheet-local rule. | Owner boundary; `dat-product-revision-engine-k9f` | Revision-engine specification |
| 16 | Existing releases are immutable; controlled Design/Publish changes require successor revision treatment before release. | Owner boundary; Product Revision Engine research | Allocation, impact, approval, supersession, regeneration |
| 17 | Build order is Revision foundation → Publish foundation → Sheet → Viewport → schematic proof → fabrication/assembly. | Owner disposition | DOC-C06 Frontier/dependency ratification |
| 18 | Persistent Project Navigator + document/view panes + View-menu tree control + focused context is the inherited hybrid. Single-click selects; double-click or Enter opens beside; discovery is local through visual scrolling or right-click → Search followed by incremental type-to-select filtering. | Ratified decisions 019/021; GUI Design/Product/Conformance specs; board/schematic/workspace prototypes; owner disposition | DOC-C06 integration and Publish tree content inventory only; the navigation model is closed |
| 19 | Retired: separate Board/Documentation workspace persistence conflicts with the unified recursive-pane Workspace. The user’s Project arrangement may restore as non-authoritative workspace state; content focus never swaps it implicitly. | Decisions 007/021; `workspace-panes.html`; owner disposition | Any named-layout product expansion must be specified separately rather than reopening Q19 |
| 20 | One cohesive native Datum window; arbitrary internal recursive splits plus pane Zoom and Full-Screen Stage; no workspace, Sheet, or pane detachment and no PiP. | Owner disposition; `workspace-panes.html`; corrected decisions 007/021 | DOC-C06 terminology integration only; native-window topology is closed |

### Question 20 provenance correction

The detachment concept did not originate with the owner. Git history traces the
first broad workspace PiP/float/multi-monitor language to draft decision 007 in
commit `22aeebe` (2026-06-19), then traces the false “owner-directed” attribution
and explicit PiP/native-window mechanism to decision 021 in commit `f682f21`
(2026-07-09). The controlling `workspace-panes.html` visual shows the approved
recursive internal pane tree and View-menu operations; it does not establish a
detached-window mechanism. The 2026-08-23 owner disposition corrects the record:
one cohesive native Datum window, arbitrary internal multi-view composition,
pane Zoom, and Full-Screen Stage only.

### Visual-source scope and conflict rule

- `board-editor.html` and `schematic-editor.html` control the editor-shell visual
  composition: persistent left Project tree over a pane-following context panel,
  central pane field, right Inspector, and terminal dock.
- `workspace-panes.html`, ratified in commit `f682f21` with decision 021,
  controls recursive pane tiling, focused-pane ownership, View-menu tree
  operations, fill-focused-pane content classes, and layout presets. Its
  simplified omission of the lower left context panel is not a decision to
  delete Layers/Sheets from the editor shell.
- `DATUM_GUI_CONFORMANCE_SPEC.md` §2.3–2.4 explicitly enforces the left Project
  tree above Layers and pane-following binding. Runtime’s current static Project
  card and generic `PaneContent::{Board,Schematic}` are incomplete realization,
  not competing product authority.
- A future Claude visual study may extend these sources for scalable Project and
  Publish navigation, but its brief must enumerate every carried-forward locked
  element and identify any proposed amendment rather than silently redesigning
  the shell.

## Confirmed intent outside the original 20

### Workspace, Design Space, and Publish Space

`Workspace` is Datum’s overall configurable working environment. It owns the
interaction composition—panes, layout, focus, and open content—but does not
itself determine design authority.

`Design Space` contains authoritative engineering and asset work through the
editor type appropriate to the content: spatial canvases such as Schematic,
PCB, Symbol, Footprint, 3D, or Panelization, as well as non-spatial editors such
as tables, trees, forms, or other project-data views. This does not collapse
those editor types into one universal plane; it establishes their shared side
of the design-versus-publication boundary.

`Publish Space` composes projections of authoritative project information with
publish-owned presentation and documentation content. Publication does not
transfer design authority into the projection. Space-constrained UI may label
the pair simply `Design | Publish`; long-form prose may use `Design Space` and
`Publish Space`.

The owner identified that prior discussion had used `Model Space` and
`Workspace` interchangeably. Both `Model Space` and `Paper Space` are retired
from Datum’s canonical vocabulary. They remain only when quoting an original
question or discussing external CAD precedent; neither names a separate Datum
object class or user-facing mode.

### Publish Space and Sheets

Publish Space is the working environment for composing and managing a
user-extensible collection of `Sheet` objects; it is not itself a Sheet. A
Sheet is one individually publishable page. It owns its page/media definition,
zero or more viewports, and publish-owned annotations and composition content.
The continuous schematic Design Space has no page or Sheet object, eliminating
the legacy EDA terminology collision and the need for `SchematicPage`.

A newly created Sheet may be completely blank apart from its page definition.
A user-selected custom template may automate creation by adding viewports,
title blocks, graphics, annotations, or other publish-owned content. Templates
are accelerators and reusable starting points, not restrictions on subsequent
composition. Detailed template, documentation, and Sheet lifecycle mechanics
remain later questions in this specification pass.

### Sheet Sets and publishing templates

A Project may contain any number of independent ordered `SheetSet` publication
packages, including user-defined review, fabrication, assembly, service, or
customer deliverables. The user or a selected project/publishing template
decides the initial sets, Sheets, Viewports, and publish-owned content.

Template instantiation never locks the result to the template’s original
shape. Users may add, remove, or reorder Sheets; add or remove Viewports on any
Sheet; and otherwise reconfigure the publication package while working. Datum
may ship a small curated starter set—potentially one default template—but the
exact built-in catalog is product-content research, not an architectural
restriction. Saving later changes back into a reusable template must be an
explicit action rather than a side effect of editing an instantiated SheetSet.

SheetSets hold ordered references to authoritative Sheets; they do not own or
implicitly copy those Sheets. One unchanged Sheet may therefore appear in
multiple packages. When a customer, fabrication, service, or other package
needs different content, `Fork Sheet` creates an independently addressable
Sheet variant from the existing composition. Viewport instances in the fork
initially retain their links to the same named definitions so unchanged work
continues to update parametrically.

For a Viewport that must diverge, `Make Unique` creates a new stable
`ViewportDefinition` identity, preserves its parametric association to Design
geometry, breaks only the reuse link to the prior definition, assigns the next
available numbered name (for example `board_XYZ_Viewport_01` to
`board_XYZ_Viewport_02`), and immediately exposes that name for inline editing.
Plain `Rename` remains label-only and never changes identity or linkage.

Customer redaction is not satisfied by hiding layers or annotations: concealed
geometry, metadata, vector structure, attachments, or embedded source could
still escape in an output artifact. `dat-publish-redaction-contract-wzs` is a
mandatory intake tracker for an evidence-backed, allowlisted export and
verification contract. It is related to the current documentation-system spec;
DOC-C06 must determine its exact dependency and Frontier placement before any
implementation authorization. It must not be replaced by ordinary Viewport
visibility controls.

### Tiled editor workspaces

Board, Schematic, and future editor workspaces are distinct surfaces that may
be visible at the same time inside one configurable pane tree. The user can
split, tile, retarget, resize, maximize, or temporarily fullscreen a pane for
speed, much like an expert tiling-window workflow. The focused pane owns its
relevant tools and contextual inspection. This composition is workspace/session
state and does not create, copy, or partition model authority. The current
running Board/Schematic split and `docs/gui/prototypes/workspace-panes.html`
already demonstrate the core disposition; later visual research must extend it
to all intended editor and Publish Space surfaces.

### Local contextual specialist editors

Symbol and Footprint authoring require their own editor personas over Datum’s
shared viewport/tooling services, but entering them should be contextual and
fast. A designer working in Schematic, PCB, or Library can invoke the relevant
asset editor from the selected object’s local menu; Datum opens the specialist
editor in a nearby pane while preserving the parent design context and a quick
return path. Direct opening from Library remains available for asset-first work.

The Footprint editor direction includes fast source-assisted construction,
parametric pad and outline shaping (including operations such as scaling and
corner treatment), standards-engine derivation of dependent manufacturing
features such as mask and stencil/paste apertures, and SolidWorks-like driving
dimensions for physical accuracy. Symbol authoring has similarly specialized
electrical and graphical requirements. These are target research/specification
needs, not claims that the mechanisms already exist.

The default contextual transition opens a new adjacent pane containing the
Symbol or Footprint editor, leaving the invoking editor visible. A single
keystroke closes the transient pane and restores the preceding pane layout, so
entry and exit are both fast. The specialist content lives in its own editor
type within that pane; it is not an editing mode imposed on the invoking
Schematic or Board surface. This is the initial default, not a permanent bar on
user-configurable placement or temporary maximize behavior. Claude’s visual
study must demonstrate the entry, active-editor, dismissal, and restored-layout
states.

Asset ownership (editing an authoritative shared library item versus a
project-local derivative) and the save/commit boundary are separate
library-authority questions and must not be inferred from “local” UI access.

### Continuous schematic workspace

The schematic design surface is effectively unbounded and independent of
physical paper. Primary circuits and subcircuits may coexist spatially on the
same plane. The designer navigates directly across that plane; large projects
must not be divided into publication pages merely to satisfy paper size.

Paper size, orientation, viewport crop and scale, title blocks, logos,
photographs, publication annotations, fabrication notes, DFM/document-control
content, color/monochrome presentation, plotting, and page-oriented export
belong to Publish Space. Required standards and the exact
PDF/PostScript/EPS and plot/output contract remain research questions.

### Dimension authority

Design Space and Publish Space both support dimensions, but they do not share
mutation authority. A Design Space dimension may be a driving constraint over
authoritative geometry, including the SolidWorks-like smart dimensions desired
for accurate Footprint construction. Any resulting geometry change flows
through Datum’s typed Design mutation and constraint-solving path.

A Publish Space dimension is associative documentation. Its anchors may resolve
stable Design geometry through a Viewport so its measured value and placement
remain current, but it cannot constrain, mutate, or write back into that
geometry. This remains true even when the Viewport itself is a linked instance
of a named definition. Publish edits use Publish operations only; authoritative
Design changes require the explicit Design doorway.

Design Space supports both driving and reference dimensions. Smart Dimension
creates a driving dimension by default. If the proposed constraint would
over-constrain the design, Datum refuses that driving mutation and explicitly
offers `Create as Reference`; it never silently converts the user’s intent.
Only Design Space may host driving dimensions under the current approved
boundary. Any future proposal for Publish-driven write-back requires a new
owner decision rather than an incremental interaction shortcut.

The existing engine board-dimension type must later be classified and
reconciled against this authority split rather than treated as precedent.

### Viewport annotation and source navigation

A viewport is a Publish Space object that projects Design Space content. The
user may select projected geometry as an attachment/reference target for
publish-owned dimensions, text, datum symbols, tolerances, leaders, and related
documentation. Those actions annotate the Sheet or viewport presentation; they
do not mutate the projected design source. The exact authority of reference
dimensions versus driving design constraints remains question 11.

When adding a viewport to a Sheet, the user chooses between a new, clean
projection and an existing saved/named Viewport composition. The clean path
starts without inherited Publish annotations or prior presentation edits. The
existing path may select a composition such as `board_XYZ_Viewport_01` and
bring forward its source reference, crop, orientation, scale, visibility,
presentation settings, and associated parametric annotations. This makes the
Viewport composition—not an unscoped global annotation collection—the unit of
intentional Publish reuse.

The approved parametric model has three layers:

1. Design source geometry and stable feature identities.
2. A versioned named `ViewportDefinition` owning its source selector,
   projection/crop/scale/visibility/render properties, and reusable associative
   annotation graph.
3. A per-Sheet `ViewportInstance` owning placement, frame, label placement,
   permitted local presentation overrides, and a reference to the definition.

`New Viewport` creates a clean unique definition and instance. `Use Existing`
creates a linked instance of the chosen named definition. Ordinary Sheet edits
remain instance-local and cannot silently change the shared definition;
`Edit Saved Viewport` is the explicit shared-change doorway and must expose the
affected consumers. `Make Unique` clones the definition for this placement and
severs future definition updates while preserving its parametric associations
to Design geometry. Thus the design-association link and the Publish-reuse link
remain independently controllable.

Every definition and instance has stable identity and revision, and every
change remains a typed journaled operation. Draft-versus-released resolution,
staleness, and revision pinning remain questions 15 and 16 rather than being
inferred here.

Authoritative source editing uses an explicit semantic action, `Open Source in
Design`. The action navigates to the viewport’s referenced Schematic, PCB,
Footprint, Symbol, assembly, or other Design editor instead of making Publish
Space an edit-through surface. It is exposed through the viewport’s local
right-click menu, consistent with Datum’s speed-first contextual interaction
direction. Double-click is deliberately unassigned rather than silently
creating a second navigation doorway. Pane placement and return choreography
remain visual-study work.

This follows the useful boundary in official SolidWorks documentation without
copying its window model. SolidWorks exposes context commands that open a part
or assembly from a drawing and can preserve the drawing view orientation, while
its drawing environment independently supports dimensions, notes, datum
symbols, geometric tolerances, and other annotations:

- [Open Model in Position](https://help.solidworks.com/2026/english/SolidWorks/sldworks/c_open_part_in_position.htm)
- [Drawings Overview](https://help.solidworks.com/2026/English/SolidWorks/sldworks/c_drawings_overview.htm)
- [Inserting Datum Feature Symbols](https://help.solidworks.com/2026/english/swtutorialonline/t_inserting_datum_feature_symbol.htm)
- [SolidWorks Predefined Views](https://help.solidworks.com/2026/English/SolidWorks/sldworks/c_predefined_views.htm)
- [Revit: Apply a View Template](https://help.autodesk.com/cloudhelp/2026/ENU/Revit-Customize/files/GUID-DA062846-32C9-4FFF-9BC1-BBB548868ACA.htm)

The Revit reference is useful only for its explicit one-time-versus-linked
reuse distinction: applying template properties makes an independent result,
while assigning the template preserves a link. Datum uses its own
definition/instance and journal model rather than inheriting Revit’s object or
window architecture.

### Designer-defined organization

Datum does not prescribe circuit categories such as “Power,” “MCU,” “Analog,”
or “Digital I/O.” Organization depends on the design and discipline. Electrical
connectivity is established only through typed electrical constructs—wires,
nets, buses, ports, labels, power symbols, connectors, and related objects—and
is checked by ERC. Intentional violations use governed waiver/deviation
evidence rather than silent suppression.

Cosmetic labeled or colored boundaries are optional and carry no connectivity,
rule, hierarchy, allocation, or publication semantics. Sections may also be
persistently grouped, locked, transformed, copied, and pasted for manipulation;
the detailed persistent-group contract is postponed until its own discussion.

### Scalable Project

A Datum Project is not constrained to one PCB or one complete product. It is a
scalable design authority and organizational container. The designer decides
whether and where to introduce products, boards, assemblies, modules, variants,
references, releases, or other boundaries. Datum supplies scalable containment
without forcing enterprise structure on the simple case.

### Distributed collaboration horizon

How multiple local, remote, air-gapped, intermittent, or high-latency teams
share one logical project is intentionally postponed from this discussion, not
discarded. `dat-distributed-collaboration-architecture-lt1` and Frontier item
`DISTRIBUTED-COLLAB-SPEC` make the research/specification pass mandatory before
multi-user implementation. Candidate direction is Git-like durable local/offline
revision authority plus an optional Google-Docs-like live experience over typed
Datum operations; no mechanism is yet ratified.

### Stable decision locators

The owner approved preserving the existing numbered-decision filenames as
stable historical and governance locators. In particular:

- `PRODUCT_MECHANICS_007_PROJECT_WORKSPACE_MODEL.md` remains correctly named
  under the approved Workspace vocabulary.
- `PRODUCT_MECHANICS_020_PAPER_SPACE_AND_VIEWPORTS.md` retains its legacy path
  so Frontier, manifest, tracker, research, and historical references remain
  stable.
- DOC-C06 must update decision 020’s visible title and operative content to
  `Design Space / Publish Space and Viewports`, explicitly state that its path
  is a preserved legacy locator, and remove legacy terminology from current
  product semantics.

Preserving a pathname does not preserve superseded terminology as product
authority. Current prose, object names, UI labels, and conformance obligations
must use Design Space and Publish Space.

### Product Revision Engine re-entry

Questions 15 and 16 exposed a product-wide configuration-control requirement,
not a local Sheet setting. The owner requires every controlled document to be
revision governed, with changes to schematics, boards, libraries, Footprints,
rules, Publish content, and manufacturing artifacts handled as consequential
engineering changes. The system must be obvious when action is required while
otherwise receding into the background.

`research/documentation-system/PRODUCT_REVISION_ENGINE_RESEARCH.md` and
`dat-product-revision-engine-k9f` establish the mandatory deep-research and
specification track. Git-compatible JSON remains valuable for history and
collaboration, but the Datum Product Revision Engine is the actual revision and
release manager. It must maintain coherent local/offline authority without Git;
an optional Datum-owned Git adapter maps commits, branches, remotes, and signed
tags to engine concepts without making them authoritative. Git commits do not
automatically become engineering release revisions, and external Git changes
must re-enter Datum semantic validation. The owner has approved complete
standalone revision, baseline, approval, release, reproduction, and audit
authority when Git is absent. The engine must also expose traceable audit
evidence against each configured applicable industry-standard authority,
edition, clause, control, disposition, and proof without claiming that software
alone confers certification. The current title-block research claim
that commit batches may seed revision rows must be audited and reconciled
through configuration identification, change control, approval, effectivity,
status accounting, and audit requirements.

No answer to questions 15/16, title-block revision behavior, or release pinning
may be inferred until this track supplies an owner-approved authority model.

### Question 18 evidence packet: navigation layers

The original tabs-versus-sidebar-versus-persona wording conflates distinct
navigation scopes. Question 2 already requires heterogeneous simultaneous
panes, so a global `Design | Publish` application mode that replaces the central
surface would contradict an approved invariant: one pane may need to show a
Design editor while another shows a Publish Sheet or reference surface.

Internal authority currently supports these separate roles:

- the persistent, collapsible Project/Structure navigator discovers the full
  scalable project rather than only currently opened content;
- each leaf in the ratified recursive pane tree owns one focused
  `(document, view)` projection and a header that identifies that content;
- pane split, retarget, close, focus, zoom, and presets are workspace consumer
  state rather than project hierarchy or Design authority;
- the GUI product spec requires document tabs **or** a document switcher, so a
  global flat tab strip is not already ratified;
- general command-palette and focus/history navigation remain available for
  commands and pane movement, but do not create a second Project-search surface.

Owner disposition adds the concrete Project Navigator interaction boundary:

- single-click selects a Project item and updates its contextual information;
- double-click or Enter opens the selected target in a new adjacent pane while
  preserving the existing pane composition;
- the navigator is an independently scrollable region with its vertical
  scrollbar on the right edge;
- mouse-wheel and two-finger trackpad gestures scroll the navigator while the
  pointer is over it and never leak through as pan/zoom input to an editor pane.

External primary-source precedents reinforce the separation rather than one
universal control:

- [VS Code editor groups](https://code.visualstudio.com/docs/editing/userinterface)
  combine Explorer discovery, group-local tabs, recent-history switching,
  keyboard group focus, and optional floating windows. Tabs represent open
  working items, not the full project hierarchy. Datum uses only the discovery
  and group-local distinction here; its optional-window behavior is explicitly
  rejected by question 20.
- [SOLIDWORKS document windows](https://help.solidworks.com/2021/english/Solidworks/sldworks/c_document_windows.htm)
  combine a left manager tree with multiple independently viewable documents
  and multiple views of one document. The tree and document/window navigation
  have different ownership.
- [Blender workspaces](https://docs.blender.org/manual/en/latest/interface/window_system/workspaces.html)
  are named task-oriented arrangements of editor Areas. This supports Datum
  named layout presets, but not treating Design and Publish authority as
  mutually exclusive global personas.
- [AutoCAD Model/Layout tabs](https://help.autodesk.com/cloudhelp/2022/ENU/AutoCAD-Core/files/GUID-DE4888EE-F07D-40D7-94AB-0AD9A1741153.htm)
  provide a direct space/layout switch, but copy poorly into Datum because a
  scalable Project can contain many Design contexts, SheetSets, Sheets, and
  simultaneous heterogeneous panes.
- [SOLIDWORKS FeatureManager filtering](https://help.solidworks.com/2024/english/SolidWorks/sldworks/t_filtering_the_featuremanager_design_tree.htm)
  provides precedent for incremental filtering within a potentially large
  engineering tree and supports matching by names, types, and tags. Datum uses
  this behavior through a local context invocation rather than a permanently
  visible field.
- [JetBrains project-tree speed search](https://www.jetbrains.com/help/idea/speed-search-in-the-tool-windows.html)
  and [Search Everywhere](https://www.jetbrains.com/help/idea/searching-everywhere.html)
  demonstrate both local-tree and keyboard-global search layers. Datum adopts
  the local scoped behavior, improves it by including collapsed descendants,
  and rejects the separate global Project-search layer as slower and less local
  than right-clicking or scrolling in the persistent navigator.

The approved result is therefore a layered hybrid with non-overlapping
semantics: the Project Navigator owns complete-structure discovery; pane-local
identity owns currently visible content; and named workspace layouts own task
arrangements. `Design` and `Publish` classify content/authority and may scope
navigation, but do not become an application-wide mode that evicts other panes.
For large projects, discovery remains deliberately local: the user either
visually navigates with the right-edge scrollbar, mouse wheel, or two-finger
trackpad scrolling, or right-clicks the relevant Project node/region and chooses
`Search`. The local search receives keyboard focus; every typed character
incrementally narrows the matching set and updates the navigator selection,
including matches in collapsed descendants. It is not an always-visible filter.
No global Project-search surface is introduced. Double-click or Enter opens the
selected target beside the focused pane. This closes question 18; later Publish
tree content inventory must extend this model without reopening it.

## DOC-C01 current-state inventory

<!-- EVIDENCE:DOC-SYSTEM-SPEC:DOC-C01-CURRENT-STATE-INVENTORY -->

This inventory records what exists at `c03cfc5`, what is only planned or
visually studied, and where inherited names conflict with the owner-approved
direction. It is descriptive evidence, not permission to preserve current code
shapes or begin implementation.

### Runtime and engine inventory

| Surface or authority | Current implementation | What it proves | DOC-system consequence |
|---|---|---|---|
| Resolved project authority | `crates/engine/src/substrate/mod.rs` owns `DesignModel`, including stable domain objects, component instances, relationships, variants, manufacturing plans, panel projections, output jobs/runs, artifact metadata/runs, checks, proposals, and the transaction journal under one `model_revision`. | Datum already has one resolver-owned authority and technical revision substrate. | Publish objects must join this authority through deterministic shards, stable identity, typed operations, `commit()`, and resolver projection; a second document database or private writer is forbidden. |
| Board Design surface | `crates/engine/src/board/mod.rs` owns `Board`; `board_types.rs` includes `Dimension` and `BoardText`; the native operation enum already creates/sets/deletes board dimensions and text. | Board geometry and annotations are Design data today. | DOC-C02/C03 must distinguish driving/reference Design dimensions from Publish-only associative dimensions. Existing `Dimension` is not precedent for Publish ownership. |
| Schematic Design surface | `crates/engine/src/schematic/mod.rs` owns `Schematic` with `Sheet`, `SheetFrame`, `SheetDefinition`, and `SheetInstance`; `Sheet` owns symbols, wires, ports, text, and drawings. `substrate/operation.rs` exposes `CreateSchematicSheet`, `SetSchematicSheetName`, and many `sheet_id`-scoped authoring operations. | The current engine inherited a page/hierarchy-shaped schematic vocabulary and a paper-era frame. It has real typed authoring identity, not merely display labels. | This is the largest migration collision. Approved intent reserves `Sheet` for Publish Space and makes the schematic Design surface continuous. DOC-C03 must define the replacement electrical hierarchy/addressing model before later implementation renames or migrates these types and operations. `SheetFrame` cannot silently become the Publish title-block object. |
| Library Design assets | `crates/engine/src/pool/symbol.rs`, `footprint.rs`, `package.rs`, and `pool/mod.rs` own Symbol, Footprint, Package, Part, pad/pin mappings, graphics, physical geometry, 3D references, standards basis, and provenance. | Symbol and Footprint are authoritative assets with specialist editing needs, not ad-hoc pane modes. | Publish may project these assets, but editing remains in their Design editors. Contextual adjacent-pane entry is workspace choreography, not asset ownership. |
| Panelization/manufacturing | `crates/engine/src/substrate/artifact.rs` owns `PanelProjection`, `ManufacturingPlan`, `OutputJob`, `OutputJobRun`, and `ArtifactMetadata`; matching typed operation families exist. `PanelProjection` is currently a list of positioned board instances used by production output. | Datum has model-bound manufacturing projections and revision-stamped artifacts, but not a complete interactive panelization editor or publication document. | A panel or generated artifact may be a future Viewport source. It must not be mislabeled as a Publish Viewport or treated as proof that Sheet composition exists. |
| Publish authority | No `Sheet` with page/media composition semantics, `SheetSet`, `ViewportDefinition`, `ViewportInstance`, projected `Table`, Publish annotation family, title-block template authority, controlled-document object, or Publish operation family exists in the engine. | Publish is unimplemented, not hidden under an existing type. | DOC-C02/C03 must specify these objects and their references before implementation. Names from research snippets are hypotheses until ratified. |
| Revision/release authority | The engine has technical `ObjectRevision`/`ModelRevision`, operation guards, journal provenance, and artifact model-revision stamps. It has no complete product baseline, engineering-change, approval, document-revision, release, effectivity, status-accounting, or audit authority. | Existing revision data supports concurrency, provenance, and staleness only. | DOC-C03 may define which Publish objects participate in revision governance and their integration seam, but Product Revision Engine REV-C01..C08 owns release mechanics. A Git commit, journal transaction, or `model_revision` is not an issued revision. |

### GUI and workspace inventory

| Surface | Current implementation/authority | Status and boundary |
|---|---|---|
| Workspace pane tree | `crates/gui-protocol/src/workspace_layout.rs` implements `WorkspaceLayout`, recursive `PaneNode::{Leaf, Split}`, stable `PaneId`, split ratios, focus, and zoom. Decision 021 governs it. | Landed consumer/workspace state. It never advances Design or Publish revision. One cohesive native window is controlling. |
| Pane contents | `PaneContent` contains only `Board` and `Schematic`; its comment explicitly defers Footprint, Symbol, Datasheet, 3D, and CheckReport until real surfaces exist. | Publish is also absent and must be added only with a real surface. A pane is a screen-space `(document, view)` projection, never a paper Viewport. |
| GUI scene/session state | `ReviewWorkspaceState` in `crates/gui-protocol/src/lib.rs` still owns a primary board review scene plus an optional sibling schematic scene and review/supervision state. | This is historical review-shell architecture being evolved, not the target project/document authority. It cannot dictate the documentation object model. |
| Project navigation | The visual authority places a persistent Project tree above the pane-following lower-left context panel. Runtime/conformance currently provides fixture-driven read-only tree content rather than the approved scalable Project Navigator interaction. | The approved navigator law remains: independent vertical scrolling; local right-click Search with incremental type-to-select; single-click select; double-click or Enter opens adjacent without eviction. Publish tree inventory is still missing. |
| Focused context | Pane focus owns tools, menus, Inspector, Layers/Filters, and local feedback. `ApplicationFocus` separately distinguishes editor pane, terminal, and overlay. | Design and Publish content may coexist in the same recursive tree; `Design | Publish` are authority classifications, not global modes that replace the tree. |

### Decision and specification inventory

| Source | Carried-forward authority | Conflict or unfinished work |
|---|---|---|
| Product Mechanics 007 | Project authority is distinct from workspace and session composition; workspace persistence is non-authoritative. | Draft primitives such as tabs, overlays, profiles, and `workspace_revision` exceed landed code and require reconciliation with the scalable Project and the retired Q19 premise. Distributed collaboration remains mandatory later work. |
| Product Mechanics 019 + GUI Product/Design specs | Datum is one manual-first desktop product with document/view surfaces over one `DesignModel`; graphical edits use native typed operations. | `Schematic sheets`, global document/mode switching, and surviving Model/Paper terminology reflect the inherited page model and must be reconciled in DOC-C06. |
| Product Mechanics 021 | Recursive panes, focused-pane ownership, ratio resize, Zoom, Full-Screen Stage, and one native window are ratified. Workspace panes are explicitly not publication viewports. | Its content lists still say model-space and schematic sheet and do not yet include Publish content. These are terminology/inventory repairs, not permission to reopen tiling. |
| Product Mechanics 020 | One universal publication mechanism, heterogeneous projected content, independent Viewport move/resize/scale, paper-scale annotations, and render/export fidelity are the valuable foundation. | Its title and operative prose still use Model/Paper; it treats schematic sheets as model assets, proposes edit-through Viewports, collapses a Viewport to one object rather than definition/instance, and understates complete revision baselines. DOC-C06 must amend rather than layer contradictory prose on top. |
| Shared Tooling Taxonomy / Product Mechanics 023 | Editor personas inherit shared grid, camera, snap, selection, transforms, measurement, constraints, Inspector, and rendering services. | Publish needs a persona/configuration over shared services, while its authored objects and mutation authority remain distinct from Design. |
| Rendering Book §8 | Owner-approved title-block face, visual language, four band/strip configurations, field hierarchy, and per-Viewport scale label. | It is a visual/content contract, not a Sheet, formula, document-control, or release object model. Its remaining “sheet frame design” wording and proportional-size assumptions require standards validation in the later specification. |

### Research and visual-source inventory

| Evidence | Controlled result | Missing proof or required reconciliation |
|---|---|---|
| `docs/gui/prototypes/board-editor.html` and `schematic-editor.html` | Persistent left Project tree, pane field, right Inspector, terminal dock, focused context, and Board/Schematic visual language. | Both still display legacy `Schematic · Sheet` labels. Neither visualizes Publish Space. |
| `docs/gui/prototypes/workspace-panes.html` | Recursive nested tiling, focused-pane context, View-menu tree operations, fill-focused-pane classes, and layout presets. | It is the pane law—not a Publish study. It lacks Publish content, scalable navigator states, local Project search, contextual adjacent specialist-editor entry/return, and the approved continuous schematic naming. |
| `docs/gui/prototypes/title-block-study.html` and `title-block-sizes.html` | Owner-approved title-block compositions, hierarchy, orientations, and scaling study. | They study the block on a page, not blank/template Sheet creation, Viewport definition/instance reuse, SheetSet composition, source navigation, responsiveness, staleness, or release UX. |
| `docs/gui/prototypes/rendering-study.html` and `text-placement-study.html` | Rendering and typography constraints that Publish output must inherit. | They do not define Publish authority or navigation. |
| `research/documentation-system/TITLE_BLOCK_AND_DOC_CONTROL_RESEARCH.md` | Standards perimeter, anchor/field/formula hypotheses, firm customization, and document-control concerns. | Its `DrawingSheet`, revision-row, release-state, and profile sketches are research only. The claim that each commit batch can seed a revision row conflicts with Product Revision Engine separation and must be removed or narrowed during ratification. |
| `research/documentation-system/PRODUCT_REVISION_ENGINE_RESEARCH.md` | Standalone Datum release authority, optional Git adapter, full configuration baseline, immutable releases, and clause-addressable audit requirement. | REV-C01..C08 remain unratified and unimplemented. DOC work must expose integration points without deciding lifecycle mechanics. |
| This workspace research and owner ledger | All 20 original questions are answered, boundary-answered, or retired; it records scalable Project, Workspace/Design/Publish vocabulary, continuous schematic Design, Viewport definition/instance reuse, SheetSet behavior, and build order. | These dispositions remain research evidence until DOC-C05 owner review and DOC-C06 decision/spec reconciliation. |

### Terminology collision register

| Inherited term | Current collision | Required direction |
|---|---|---|
| Workspace | Sometimes means the whole application arrangement and sometimes an editor/domain surface. | Reserve `Workspace` for the overall configurable pane/focus/tool environment. Use editor surface or Design/Publish content for pane targets. |
| Model Space | Used in decision 020 and older GUI prose for both authority and spatial editors. | Retired Datum vocabulary. Use `Design Space`; retain Model Space only for quoted questions or external precedent. |
| Paper Space | Used in decision 020 and older GUI prose for publication composition. | Retired Datum vocabulary. Use `Publish Space`; preserve decision 020's filename only as a stable locator. |
| Sheet | Existing engine type for schematic electrical/page content and proposed type for a physical publication page. | Reserve `Sheet` exclusively for Publish. Replace the schematic page/hierarchy model during a separately specified migration; do not alias both meanings. |
| Viewport | Renderer/camera rectangles, workspace panes, and proposed paper projection windows all use the word informally. | Product vocabulary uses `Pane` for screen tiling and `ViewportDefinition`/`ViewportInstance` for Publish projections. Low-level render viewport may remain an implementation geometry term when clearly scoped. |
| Revision | Used for object/model concurrency, Git history, artifact source stamps, document issue, and product release. | Keep technical, Git, engineering-change, baseline, document/package, and transmittal layers distinct under the Product Revision Engine. |

### DOC-C01 conclusion

The landed substrate can host the future system, but the documentation system
itself does not exist. The next specification work must therefore define an
authority vocabulary before defining structs: one scalable Project; one
non-authoritative Workspace; Design Space editor surfaces; one universal
Publish Space; screen Panes; Publish-only Sheets; and versioned
ViewportDefinitions with per-Sheet ViewportInstances. It must also define the
schematic `Sheet*` migration seam, Publish participation in the Product Revision
Engine, and the missing Claude-owned visual studies without treating any
research sketch as already-ratified mechanism.

## External research state

Primary-source review has begun for AutoCAD layouts/model and paper space,
SolidWorks drawing views, Altium Draftsman, and KiCad schematic hierarchy. It
was paused at the owner’s request before synthesis. External findings will be
added only when the relevant owner question is ready for evidence-backed
discussion; prior art will inform mechanisms without overriding Datum’s stated
product intent.
