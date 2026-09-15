# Datum Publish Space Specification

> **Status:** Governed planning contract. Owner architecture approved through
> DOC-C05; reconciled by DOC-C06 on 2026-08-23. This specification does not
> authorize implementation.
>
> **Authority:** Product Mechanics 020, Product Mechanics 034, the Datum
> substrate doctrine, the owner-approved workspace-architecture research, the
> reviewed Publish Space visual study, and
> `specs/PRODUCT_REVISION_ENGINE_SPEC.md`. Product revision and release
> mechanics remain exclusively owned by the Product Revision Engine.

## 1. Purpose and product boundary

Datum provides one scalable Project, one configurable Workspace, and two
non-exclusive authority classifications inside it:

- **Design Space** contains authoritative engineering and reusable-asset
  authoring through domain-specific editor surfaces.
- **Publish Space** contains authoritative composition and documentation
  authoring over projections of Design objects and generated artifacts.

Design and Publish surfaces may coexist in any recursive Pane arrangement.
Neither is a global mode, separate application, or replacement for the other.
Publish Space is universal: schematic, board, symbol, footprint, panel,
manufacturing, table, image, and future supported sources use one composition
system rather than per-editor documentation generators.

The stable decision filename
`docs/decisions/PRODUCT_MECHANICS_020_PAPER_SPACE_AND_VIEWPORTS.md` is retained
as a historical locator. `Model Space` and `Paper Space` are not current Datum
product terms.

## 2. Controlling principles

1. Project scope is designer-selected. One Project may contain one circuit or
   many products, boards, subassemblies, variants, libraries, and packages.
2. Workspace and Pane state are consumer state. They never become Design,
   Publish, document, or release authority.
3. Design and Publish source objects use stable identity, deterministic shards,
   typed operations, expected-revision guards, `commit()`, and the journal.
4. A Publish projection never copies or mutates its Design source.
5. Resolved imagery, table cells, title-block values, and output are projections;
   their definitions and bindings are authored source.
6. Publish dimensions and annotations may associate with stable Design features
   but never drive Design geometry.
7. The Product Revision Engine supplies working or immutable-baseline
   configuration context. Publish objects do not implement a rival release
   state machine.
8. Rendering and controlled output resolve the same geometry, styling, fields,
   and configuration context.

## 3. Canonical vocabulary and authority

| Object | Meaning and authority | Must never mean |
|---|---|---|
| `Project` | Scalable authority/container whose authored structure is chosen by the designer. | One PCB, one product, or one imposed hierarchy. |
| `DesignModel` | Resolver-owned canonical Design and Publish source graph. | A GUI scene, Git checkout, or release baseline. |
| `Workspace` | One configurable working environment in one native Datum window. | A Design domain or publication package. |
| `Pane` | One leaf of the recursive screen tile tree showing one content/view projection. | A Publish Viewport or source copy. |
| `Design Space` | Classification for authoritative engineering authoring. | One universal canvas or application mode. |
| `Editor Surface` | Schematic, Board, Symbol, Footprint, and future domain persona over shared tooling. | A separate authority or native application. |
| `Publish Space` | Universal authoring domain for documentation composition and publishing. | A Sheet, output file, or editor-specific generator. |
| `PublishCatalog` | Project-owned root/index of Publish identities and optional organization. | A mandatory product hierarchy. |
| `Sheet` | One stable, individually publishable physical page and ordered composition. | A schematic page, Pane, package, or output file. |
| `ViewportDefinition` | Named reusable view meaning and reusable annotation graph. | A screen camera or copied Design object. |
| `ViewportInstance` | One placement of a ViewportDefinition on one Sheet. | A second Design object. |
| `ProjectedTable` | Authored table projection definition whose resolved cells come from exact sources. | A shadow database. |
| `PublishAnnotation` | Publish-owned documentation, static or associatively anchored. | A driving Design constraint. |
| `SheetSet` | User-named ordered publication selection containing Sheet references. | The authoritative home of Sheet bodies, a controlled-document identity, or a release state machine. |
| `PublishTemplate` | Reusable seed graph that creates ordinary editable Publish objects. | A restriction on later composition. |
| `ConfigurationRef` | Revision-engine-owned working or immutable-baseline resolution context. | A Sheet-local Draft/Released flag. |

## 4. Workspace and navigation contract

Datum uses one cohesive native window. It supports one Pane or recursively
nested horizontal and vertical splits; Design, Publish, terminal, and auxiliary
content may coexist. A focused Pane may be maximized or enter the existing
full-screen Stage. Panes and Sheets do not detach into additional native windows.

The persistent Project Navigator is the authoritative discovery surface:

- `Design` and `Publish` classify project content without evicting open Panes.
- Under `Publish`, `All Sheets`, `Publish Sets`, and `Saved Viewports` are
  sibling groups.
- `All Sheets` is the sole authoritative visual home for each Sheet.
- Publish Sets show visibly linked, set-local ordered references rather than
  duplicate-looking Sheet bodies.
- Single-click selects. Double-click or Enter opens beside the focused Pane.
- The Navigator has a visible right-edge scrollbar and supports wheel and
  two-finger scrolling.
- Local right-click `Search`/`Search Within` followed by typing incrementally
  filters the selected scope, including collapsed descendants. There is no
  separate global Project-search surface in this contract.

Contextual Symbol and Footprint authoring opens a new adjacent Pane by default,
preserves the invoking context, and closes/restores the prior arrangement with
one shortcut. This is a default choreography, not an authority boundary.

Pane titles are type-first:

- Design: `Editor Type · Local Design Identity`, such as
  `Schematic · sensor-node` or `Board · main`.
- Publish: `Publish · Sheet Number Sheet Title`, such as
  `Publish · FAB-01 Assembly Top`.

`Schematic · Sheet n/N` and `Board · Layout` are legacy labels to migrate.

## 5. Publish object model

### 5.1 Shared identity and coordinates

Every Publish source object has a stable `ObjectId`, guarded technical
`ObjectRevision`, deterministic JSON-shard representation, and human name where
applicable. References use stable IDs and typed roles, never filenames, labels,
tree positions, or pixels.

Sheet geometry uses signed integer nanometers in a Sheet-local Cartesian system
with origin at the lower-left media corner, +X right, and +Y up. Angles use the
project angular convention. Persisted scales are reduced positive rationals;
floating-point values are not authority.

### 5.2 Sheet and media

A Sheet owns identity, name, object revision, `PageMedia`, ordered `SheetItem`
composition, metadata/bindings, and optional template provenance. `PageMedia`
supports standards-profile media identifiers and Custom width/height,
orientation, margins/printable guidance, and entry units.

The closed initial `SheetItem` family is:

```text
ViewportInstance | ProjectedTable | PublishAnnotation |
TitleBlockInstance | PublishGraphic | ImageInstance
```

A Sheet may be blank or template-seeded, may mix supported items freely, and
may contain zero or many Viewports. Overflow is visible and validated, never
silently cropped or rescaled. Sheet owns no mutable release label or baseline.

### 5.3 Viewport definition and instance

`ViewportDefinition` owns reusable meaning: typed source and subview selector,
projection/orientation, crop/extent, exact or deterministic fit scale, visibility
selectors, style/render intent, color policy, optional clip, and reusable
associative annotations.

Initial source families are Schematic Design context, Board side/layer view,
Symbol, Footprint, Panel projection, and generated Artifact view. New source
families require an existing owning authority and renderer.

`ViewportInstance` owns Sheet placement: definition reference, frame position
and size, label placement, lock, clip/frame presentation, item-list z-order,
explicit permitted property overrides, and instance-local annotations.
Effective properties resolve as instance override then definition value.

- Move changes placement only.
- Resize changes frame only and reveals more or less at the current scale.
- Scale changes projected content scale only.
- Fit-to-frame is an authored deterministic scale mode.
- Lock prevents accidental presentation edits; it does not freeze source state.

`New Viewport` creates a unique definition. `Use Existing` places a linked
instance. Ordinary placement edits are local. `Edit Saved Viewport` explicitly
edits the shared definition with consumer-impact preview. `Make Unique` creates
the next named definition identity, materializes effective properties, preserves
Design associations, clears materialized overrides, retargets the instance, and
offers inline rename. Plain Rename changes only the label.

Healthy linked/override/unique state is fully available in Inspector and appears
as a glyph-plus-word chip on the Viewport only on hover or selection. Broken or
stale state remains visible until resolved. These editing overlays never print
or export and never rely on color alone.

### 5.4 Tables, annotations, graphics, and title blocks

`ProjectedTable` owns a typed source/query kind, columns, deterministic filter/
sort/group policy, style, placement, pagination/overflow, and optional row-to-
feature references. BOM, drill/hole, layer-stack, net/pin, Sheet-index, and
drawing-register tables use this shared mechanism. Resolved rows are derived
from the supplied ConfigurationRef and are not persisted as rival truth.

Publish annotations include Note, reference Dimension, Leader, Balloon,
datum/tolerance symbol, Callout, and ViewReference. Each has paper-scale style
and Sheet-local, definition-reusable, or instance-local scope. Associative
anchors use the ViewportInstance plus stable Design object/feature identity and
typed geometric role. Missing or ambiguous targets produce typed findings;
Datum never fuzzy-retargets or silently converts them to static geometry.

`TitleBlockDefinition` is a reusable governed asset containing anchored graphics,
field slots/bindings, images, page-scope rules, and standards metadata.
`TitleBlockInstance` places it and owns permitted overrides. Revision, status,
approval, and release fields are revision-engine projections, never free text
claims or values inferred from Git/commit counts.

### 5.5 Publish Sets, duplication, and templates

A `SheetSet` (user-facing `Publish Set`) owns identity, user name, ordered
`SheetUse` references, authoring metadata, and explicit source bindings to
separate revision-engine `ControlledDocument` definitions. It never owns a
document number, EngineeringRevision, release state, immutable DocumentIssue,
ReleasePackage, or Transmittal. `SheetUse` owns set-local order and numbering but
never the Sheet body. One Sheet may be referenced by multiple Publish Sets, and
one Publish Set may explicitly source multiple ControlledDocuments with
distinct purpose, number, and policy.

`Duplicate as New Sheet` is a rare contextual action creating a new Sheet
identity and editable composition copy while preserving Design associations and
shared ViewportDefinition links. It immediately requests a meaningful name;
mechanical `(copy)` naming must not persist. Divergence of a linked Viewport uses
`Make Unique`, not implicit severing.

Templates may create blank or prepopulated Sheets, Publish Sets, Viewports,
title blocks, graphics, and bindings in one atomic batch. Created objects are
ordinary editable project objects. Templates never force customer/fabrication
taxonomy or prevent adding/removing/reordering Sheets and Viewports.

## 6. Mutation and refusal contract

Publish uses explicit typed operations for create/delete/rename, media and
metadata, item ordering, definition properties, instance placement/overrides,
annotations and anchors, tables, governed assets, Publish Set membership/order,
duplication, Make Unique, and template instantiation. Generic JSON patch is not
a public mutation mechanism.

Multi-object actions are atomic and carry complete inverse data. Deletion of a
referenced object refuses until dependents are explicitly removed or retargeted.
Completed gestures emit one authored operation; drag previews, selection, hover,
camera movement, Pane operations, and `Open Source in Design` are consumer/
navigation state.

Minimum typed findings/refusals cover wrong-kind or missing source, lost or
ambiguous feature, cycles, invalid scale/crop/clip, media overflow, unsupported
render intent, unresolved mandatory field, stale artifact, invalid Publish Set
reference/order, attempted Publish-to-Design write-through, immutable-baseline
mutation, and output blocked by checks or egress policy.

## 7. Design doorway and revision seam

A Viewport is never an edit-through aperture. Its local right-click menu exposes
`Open Source in Design`, which opens the source in a new adjacent Pane while
preserving Publish context. Double-click is deliberately unassigned. Closing the
adjacent source Pane restores the prior arrangement.

Every Publish render receives exactly one `ConfigurationRef`: working for normal
authoring or immutable baseline for reproduction. Baseline views use restrained
lock, word, and exact-identity cues and refuse mutation. A later Design change
may produce an impact finding and navigation to analysis, but Publish never
allocates a revision or rewrites released content.

The Product Revision Engine exclusively owns configuration identification,
change control, baselines, approvals, revision allocation, release, effectivity,
supersession, withdrawal, status accounting, reproduction, and audit. Existing
releases are immutable, and controlled Design or Publish changes cannot be
released under an unchanged prior revision index.

Visibility, crop, omission, duplication, and Make Unique do not prove redaction.
Verified redaction remains a separate contract (`dat-publish-redaction-contract-wzs`).

## 8. Conformance obligations

An implementation slice is incomplete until its applicable obligations have
machine-readable evidence:

| ID | Required proof |
|---|---|
| `PUB-CONF-01` | Stable identity, deterministic shard round-trip, guarded typed operations, journal/undo, and no private writer. |
| `PUB-CONF-02` | Pane/Workspace actions cannot mutate Design or Publish source without a distinct typed operation. |
| `PUB-CONF-03` | Sheet media, coordinates, ordered items, blank/template creation, and overflow behavior are deterministic. |
| `PUB-CONF-04` | Definition/instance inheritance, overrides, Edit Saved Viewport, Make Unique, Rename, and deletion/refusal semantics are exact. |
| `PUB-CONF-05` | Move, resize, scale, fit, crop, lock, and source navigation remain non-conflated. |
| `PUB-CONF-06` | Projection cannot mutate Design; associative failure is explicit; Publish dimensions cannot drive geometry. |
| `PUB-CONF-07` | All Sheets authority, linked Publish Set references, set-local ordering, and Duplicate as New Sheet identity are exact. |
| `PUB-CONF-08` | Working/baseline ConfigurationRef resolution, immutable-baseline refusal, and impact findings have no rival release state. |
| `PUB-CONF-09` | On-screen and PDF output resolve identical geometry, style, fields, and configuration inputs with reproducible provenance. |
| `PUB-CONF-10` | Project Navigator, pane titles, adjacent opening, single-window tiling, local search, scrolling, and accessibility match the approved prototypes. |
| `PUB-CONF-11` | Broken/stale/error states are persistent, non-color-only, accessible, and absent from controlled output. |
| `PUB-CONF-12` | Dependency authority, source-health, spec governance, evidence traceability, GUI conformance, and owner visual acceptance gates pass. |

## 9. Sequencing and bounded first proof

Publish implementation is not yet authorized. The required development order is:

1. complete and production-accept the Product Revision Engine foundation;
2. establish the domain-neutral PublishCatalog, operation, resolution, coordinate,
   render/export, staleness, and refusal substrate;
3. implement Sheet and PageMedia primitives;
4. implement ViewportDefinition/ViewportInstance and projection primitives;
5. prove one narrow Schematic-to-Sheet vertical slice;
6. extend to fabrication, assembly, tables, richer sources, and output breadth.

The narrow proof contains one PublishCatalog and working ConfigurationRef, one
blank or template Sheet with one supported ISO/ANSI media choice, one Schematic
ViewportDefinition and instance, move/resize/scale/crop/lock, Open Source in
Design, one title block, one Note, one associative reference Dimension, one
Publish Set, deterministic screen/PDF output, and an immutable-baseline
reproduction witness.

Fabrication/assembly templates, arbitrary 3D, verified redaction, PS/EPS/plotter
breadth, and all projected-table variants are explicitly outside that first
proof. Exact implementation packages and execution authorization must be placed
by later Frontier transactions after Revision Engine production acceptance.

## 10. Evidence and traceability

Controlling evidence chain:

- `research/workspace-architecture/WORKSPACE_ARCHITECTURE_RESEARCH.md`
- `docs/gui/prototypes/workspace-panes.html`
- `docs/gui/prototypes/publish-space-study.html`
- `docs/gui/workspace-and-documentation-specification/PUBLISH_SPACE_VISUAL_STUDY_BRIEF.md`
- `docs/decisions/PRODUCT_MECHANICS_020_PAPER_SPACE_AND_VIEWPORTS.md`
- `research/documentation-system/PRODUCT_REVISION_ENGINE_RESEARCH.md`
- `docs/decisions/PRODUCT_MECHANICS_034_PRODUCT_REVISION_ENGINE.md`
- `specs/PRODUCT_REVISION_ENGINE_SPEC.md`

<!-- EVIDENCE:DOC-SYSTEM-SPEC:DOC-C06-GOVERNED-RECONCILIATION -->
This specification is the DOC-C06 governed reconciliation of the owner-approved
architecture. Product Mechanics 034 and the Product Revision Engine
specification now govern its `ConfigurationRef`, issue, release, and impact
seam. Publish implementation remains blocked until that engine foundation
reaches production acceptance.
