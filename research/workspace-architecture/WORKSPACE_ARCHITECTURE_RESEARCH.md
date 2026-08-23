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
| 15 | Should Draft sheets always follow the live model while Released sheets resolve against an immutable `model_revision`? | **Research-gated** | The question is too narrow: release must resolve a complete approved product/configuration baseline across Design, library, rules, Publish definitions/instances, checks, manufacturing plans, artifacts, approvals, and effectivity—not only one `model_revision`. `research/documentation-system/PRODUCT_REVISION_ENGINE_RESEARCH.md` and `dat-product-revision-engine-k9f` must establish the standards-driven authority model before owner disposition. |
| 16 | When the model changes after release, should the released sheet remain frozen until a new document revision is deliberately created? | **Research-gated** | Released-output immutability, pending-change indication, revision allocation, affected-document scope, supersession, and regeneration are Product Revision Engine decisions. Existing releases must not be silently overwritten, but the exact Draft/change/release lifecycle remains deliberately undecided pending that research. |
| 17 | What should Datum implement first after specification: schematic publication, fabrication/assembly drawings, or the general sheet/viewport substrate with one narrow proof template? | **Open** | No owner disposition yet. |
| 18 | Should workspace navigation be document-tab based, persistent-sidebar based, mode/persona based, or a hybrid? | **Open** | Question 2 settles tiled/split/maximized editor coexistence; the control and navigation model for choosing content still requires owner discussion and Claude’s comparative visual study. |
| 19 | Should pane layouts persist per workspace and per project—for example Board remembering a Board/Schematic split while Documentation remembers sheet composition? | **Open** | Layouts are user-reconfigurable under question 2; persistence scope and ownership are not yet decided. |
| 20 | Is the desired end state one cohesive Datum window, or may advanced users detach workspaces/sheets into additional native windows? | **Open** | Pane fullscreen/maximize inside the shell is established under question 2; detachable native windows remain undecided. |

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
must re-enter Datum semantic validation. The current title-block research claim
that commit batches may seed revision rows must be audited and reconciled
through configuration identification, change control, approval, effectivity,
status accounting, and audit requirements.

No answer to questions 15/16, title-block revision behavior, or release pinning
may be inferred until this track supplies an owner-approved authority model.

## Internal evidence identified for DOC-C01

- `docs/decisions/PRODUCT_MECHANICS_007_PROJECT_WORKSPACE_MODEL.md` already
  separates authoritative project state, persisted workspace composition, and
  volatile session state, but its project and collaboration assumptions require
  reconciliation with the scalable-project intent.
- `docs/decisions/PRODUCT_MECHANICS_019_GUI_PRODUCT_MODEL.md` defines a
  document/view shell over one resolved model but does not settle the current
  workspace terminology questions.
- `docs/decisions/PRODUCT_MECHANICS_020_PAPER_SPACE_AND_VIEWPORTS.md` contains
  the correct universal publication direction but still says schematic
  “sheets” are Model Space assets and currently permits edit-through viewports.
- `docs/decisions/PRODUCT_MECHANICS_021_WORKSPACE_PANE_TILING.md` and
  `docs/gui/prototypes/workspace-panes.html` establish recursive pane tiling and
  focused-pane ownership, not Model/Paper or workspace topology.
- `docs/DATUM_SHARED_TOOLING_TAXONOMY.md` recommends thin editor personas over
  shared services; terminology must be reconciled rather than copied blindly.
- Current `PaneContent` is limited to Board and Schematic, while
  `ReviewWorkspaceState` still reflects historical review-shell ownership.
- The engine currently models schematic `Sheet`, `SheetDefinition`,
  `SheetInstance`, and `SheetFrame`; these combine electrical hierarchy,
  spatial content, and paper-era naming that must be audited against the
  continuous-plane decision.
- Existing title-block prototypes specify title-block visual treatment but no
  complete Paper Space composition or Model/Paper transition. The existing
  workspace-panes prototype specifies pane tiling but not the required universal
  documentation surface. Claude will therefore need an evidence-backed visual
  brief after the authority model is sufficiently resolved.

## External research state

Primary-source review has begun for AutoCAD layouts/model and paper space,
SolidWorks drawing views, Altium Draftsman, and KiCad schematic hierarchy. It
was paused at the owner’s request before synthesis. External findings will be
added only when the relevant owner question is ready for evidence-backed
discussion; prior art will inform mechanisms without overriding Datum’s stated
product intent.
