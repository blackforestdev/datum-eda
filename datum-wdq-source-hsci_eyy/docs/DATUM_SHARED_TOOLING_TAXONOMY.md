# Datum Shared-Tooling Taxonomy

Status: governed reference (active)

## Purpose

The controlling catalogue of Datum's **shared editor "language/tooling"** — the
capabilities that must exist **once**, in a shared backbone, and be *configured*
by each editor (schematic, board, footprint, symbol, library, rules,
manufacturing, and future editors), never reimplemented per editor. It answers, in
one tracked place: *what should be shared across all Datum editors, what already
is, what is forked or absent, and what could be shared as the program develops.*

This document is a **research-derived catalogue and sequencing reference**, not a
ratified mechanism. It **recommends**; each shared capability that ratifies
mechanism gets its own numbered decision record + governed spec when it reaches
buildable definition (decision `023` + `DATUM_UNIVERSAL_VIEWPORT_TOOLING_SPEC.md`
is the first such component — the Layer-1 interaction backbone — and re-seats
under this taxonomy). It is the parent that the Phase-2 integration work builds on.

It was produced by a four-domain investigation (look inside + look outside):
Datum's own architecture; EDA tools (KiCad, Horizon EDA, Altium, Eagle); mechanical/
parametric CAD (SolidWorks, Fusion 360, FreeCAD, Creo, Siemens NX); and creative/
multi-context tools (Affinity, Adobe, Figma, QGIS, Bitwig/Ableton). Sources are
cited in `§7`.

Authority: subordinate to `CLAUDE.md` and `docs/DATUM_PRODUCT_MECHANICS.md`; it
organizes and sequences shared-tooling work but does not override doctrine or any
decision record.

---

## 1. The controlling finding

**The field converged on one architecture, independently, under four names** —
FreeCAD *workbench*, Fusion *workspace*, Siemens NX *application*, Affinity
*persona*: a **shared services kernel below + a thin editor "persona" above**.
Switching editors swaps only the *toolset* and *one contextual inspector panel*
(and optionally the read/write posture); the document, canvas, camera, selection,
snapping/grid, history/undo, measure, and appearance are **shared services the
switch is forbidden to touch**. FreeCAD states it literally ("the tools change but
the contents of your scene doesn't change"); Bitwig's global transport/grid bar
lives above every view and is never re-implemented per view; Affinity's persona
"owns almost nothing."

The load-bearing mechanism is FreeCAD's strict **App/Gui (MVC) split**: a headless
document (`App::Document`, no GUI dependency) that the GUI only *observes* and never
mutates directly. That is **exactly Datum's posture** — a headless engine + one
canonical `DesignModel`, with the GUI as a consumer that emits typed `Operation`s
through one `commit()` and holds no private write path.

**Datum's specific finding (look inside):** the split is *diagnostic*. Datum's
**authority/substrate layer is already genuinely unified** — one `Operation` enum
(~140 variants, `substrate/operation.rs:7`), one durable `commit()`+journal that
undo and proposal-apply re-enter (`substrate/commit.rs:135`), one `ProjectResolver`
(`project_resolver.rs:73`), one `ObjectId`/`ComponentInstance` identity space, one
coordinate/units model (i64 nm), one query/check/artifact namespace, and the
single-source verb registry (`crates/verb-registry`). But the **consumer/interaction
layer is board-only-forked** — grid, camera, coordinate/hit-test, snap,
stroke-weight, hover, selection, tool-mode, context menu, coordinate readout, and
layer-visibility are a second-or-absent implementation the schematic never got. That
gap is exactly what decision `023` exists to reverse.

**Datum's structural advantage.** Because Datum has *one resolved `DesignModel` +
stable identity*, several capabilities other tools must *build* become **emergent**:
cross-probe/cross-select, schematic↔board sync, and much of query/rule sharing fall
out of the single model + `ObjectId`, exactly as Horizon EDA needs no cross-probe
bus and Altium derives cross-probe from its unified data model. KiCad, by contrast,
still runs **two** data models joined by a UUID mail bus and had to retrofit
align/distribute and scripting from PCB-only outward — the retrofit tax Datum's
single model is designed to avoid.

---

## 2. The Datum Editor-as-Persona model (recommended)

Every Datum editor **should be a thin persona descriptor over the shared kernel**,
not a place where interaction mechanism lives. Concretely, an editor =

- a **declared verb-set** — which typed operations from the one verb registry it
  exposes (Datum already has the registry; an editor is a manifest that selects
  from it — the FreeCAD workbench / Fusion workspace mechanism);
- **view-providers** — how its object classes render into the retained world scene
  and register hit/snap targets (KiCad's `PAINTER`+`VIEW_ITEM`, Horizon's
  `render()` overload + `Selectables`);
- **one contextual inspector** — a data-model-driven property panel that renders the
  selected object's declared typed properties (KiCad `PROPERTY_MANAGER`, Horizon
  auto-generated widgets, SolidWorks `IPropertyManagerPage2`) — **zero bespoke
  panel code per object class**;
- a **read/write posture** — some editors/modes are inspect-only (Figma Dev Mode is
  the precedent);

and **inherits every shared service by reference**: coordinate/hit, camera, grid,
snap, stroke-weight, hover/cursor, selection, tool-mode dispatch, context menu,
coordinate readout, layer-visibility, undo/history, measure. This is captured
operationally as the `ViewportProfile` in `DATUM_UNIVERSAL_VIEWPORT_TOOLING_SPEC.md`
(Layer 1); Layer-2 services (below) extend the same profile.

**The onboarding test (convergent across every tool surveyed):** adding a new
editor (footprint, symbol, a rules canvas) must require only — register the document
type, make its objects drawable, register its typed properties, register its verbs,
and (if geometric) declare its snap targets. If it forces re-implementing selection,
undo, snapping, navigation, or the property panel, **the kernel boundary is
misplaced.** KiCad's post-GAL rule is the benchmark: a new editor implements only a
`PAINTER`, a `SELECTION_TOOL`, and its object subtree; everything else is inherited.

**The 3D-viewer boundary (a deliberate exception).** KiCad's 3D viewer joins the
model/identity/bus layer (gets board data + cross-select) but opts out of the shared
2D canvas — its own render target. Lesson for Datum's future 3D/panelization views:
**the sharing boundary is the data + identity + dispatch layer, not necessarily the
pixels.** Share the model, identity, selection, and command vocabulary; a
fundamentally different render target may legitimately not share the 2D canvas.

---

## 3. The taxonomy (five layers)

Each row: **capability | Datum status | which editors | field precedent | share
now/later | governing spec**. Status: **unified** (one shared impl exists),
**forked** (board-only + a rival/absent elsewhere), **absent**, **spec** (specified,
not built).

### Layer 0 — Authority / substrate (already unified — the foundation)

| Capability | Status | Editors | Field precedent | Governing |
|---|---|---|---|---|
| Typed `Operation` vocabulary + one `commit()`+journal | unified | ALL | Eagle "no private write path" (ULP emits commands); Altium ECO as typed batch | CANONICAL_IR, substrate |
| Undo/redo as inverse-commit (not a separate mechanism) | unified | ALL | KiCad abstract `COMMIT`; Horizon `HistoryManager` | substrate |
| One projection engine (`ProjectResolver`) → one `DesignModel` | unified | ALL | Horizon one pool+model; NX master-model | resolver |
| Stable identity as its own layer (`ObjectId`/`ComponentInstance`) | unified | ALL | KiCad `KIID`, Horizon UUID, Altium Unique ID — the universal join key | CANONICAL_IR |
| Coordinate/units (i64 nm, edge conversion) | unified | ALL | KiCad `EDA_UNITS`/`UNIT_BINDER` | CANONICAL_IR |
| One query / check / artifact namespace | unified | ALL | Altium one Query engine under SCH+PCB | AI_CLI_MCP_TOOL_SURFACE |
| Single-source verb registry | unified | ALL surfaces | KiCad `ACTION`/`TOOL_ACTION` catalog; Eagle one command language | verb-registry |
| Provenance / proposal lifecycle (AI proposes, never silently applies) | unified (routing landed) | ALL | Altium validate-then-execute ECO | AI_CLI_MCP_TOOL_SURFACE |

### Layer 1 — Shared interaction/viewport services (decision 023 — being unified)

| Capability | Status | Editors | Field precedent | Now/Later | Governing |
|---|---|---|---|---|---|
| `EditorViewport` keystone (per-pane screen↔world + hit-test) | forked/absent | board→all | Horizon one `CanvasGL`; every editor hit-tests its own space | now | 023 §1.1 |
| Grid engine (screen-constant weight, adaptive LOD) | forked (the bug) | board correct, sch broken | Horizon 1px grid, ×2 coarsen at 20px | now | 023 §5 |
| Camera / navigation | shared math, forked routing | board+sch | KiCad `VIEW_CONTROLS`; universal | now | 023 §2 |
| Snap + quantize (2-tier resolver, SnapTarget registry) | spec | board→all | Horizon grid→object-override; Altium unified cursor-snap | now | 023 §3 |
| Stroke weight-classes (A/B/C) | absent as a model | ALL | KiCad IU/stroke; render==CAM | now | 023 §4 |
| Hover / cursor / crosshair | shared typed state + overlay mechanism | board+schematic | universal | landed S4; extend profiles with new surfaces | 023 §2 |
| Selection + marquee (viewport gesture) | forked/absent | board only | KiCad `SELECTION`; Horizon `Selectables` | now | 023 §2 |
| Tool-mode engine + per-editor keymap | forked/board-only | board only | KiCad `TOOL_MANAGER`; Fusion workspace | now | 023 §2 |
| Context / marking menu (contextual, verb-firing) | forked/board-only | board only | Fusion marking menu (shared grammar, context payload) | now | 023 §6 |
| Coordinate readout / status-field ownership | absent | board only | KiCad X/Y/dx/dy/units status bar | now | 023 §7 |
| Layer / visibility (per surface) | absent | ALL | KiCad `APPEARANCE_CONTROLS`; note: shared in open tools, per-editor in Altium | now | 023 §2 |

### Layer 2 — Shared editor services (next tier — mostly forked/absent)

| Capability | Status | Editors | Field precedent | Now/Later | Governing (future) |
|---|---|---|---|---|---|
| **Selection identity model** (multi-select, per-type sets — the sharpest lever) | forked/thin (single-target review enum; absent from engine) | ALL | FreeCAD app-wide `SelectionSingleton` (tree+canvas), observer bus, `SelectionGate` | now (highest fan-out) | new DR/spec |
| **Property / inspector (data-model-driven)** | shared shell, board-populated only | ALL | KiCad reflection `PROPERTY_MANAGER`; SolidWorks PMP; FreeCAD typed `App::Property` | share-later (shell exists) | new spec |
| **Measurement / dimension (one Measure)** | forked/pcb-only | ALL | FreeCAD Std Measure ("replaces several per-workbench measures", all workbenches) | share-later | new spec |
| **Align / distribute + transform** (rotate/flip/mirror as one verb) | forked/pcb-only | board→all | KiCad `ALIGN_DISTRIBUTE_TOOL` (PCB-first *anti-pattern*, issue #6769) | share-later | PARAMETRIC_TOOLING + new |
| **Markers / check-overlay + report-item** | partly unified (report model) + per-view overlay | rules over all | KiCad `MARKER_BASE`+`RC_ITEM` (one overlay for two rule engines) | mostly now | CHECKING_ARCHITECTURE |
| **Appearance / theme / styling** | absent (mutation) | ALL | QGIS Map Themes (shared named state, opt-out lock); *decide deliberately: one service or per-surface* | share-later | new spec |

### Layer 3 — Shared geometry / compute (library boundary)

| Capability | Status | Editors | Field precedent | Now/Later | Governing (future) |
|---|---|---|---|---|---|
| **One geometry/constraint solver library** (align, DRC geometry, routing geometry, footprint constraints, zone-fill, future sketch/impedance) | scattered / to-consolidate | ALL geometric | **D-Cubed 2D DCM: one solver embedded across AutoCAD + SolidWorks + Creo, and across sketch/assembly/CAM/CAE**; planegcs/SolveSpace prove clean library extraction | later (highest-value de-dup) | new DR/spec |
| **Shared parameter-schema registry** (the tri-modal single source) | spec (per-verb JSON today) | ALL authoring | SolidWorks add-ins render identical PMP; one schema, three renderers | later (high leverage) | PARAMETRIC_TOOLING |
| **Scripting / plugin object model** (same objects the editors expose, uniform) | partial (IPC/registry) | ALL | Eagle UL, Altium PME, KiCad IPC — Eagle *enforces no-private-write-path through it* | later | MCP/API specs |

### Layer 4 — Forward-looking (could become shared as Datum grows)

| Capability | Recurs across | Field precedent | Governing (future) |
|---|---|---|---|
| Parametric array / pattern placement | board, footprint, symbol grids | SolidWorks patterns; one op over populations | PARAMETRIC_TOOLING |
| Conversion sub-wheels as one parameterized op (Change Layer/Width/Size/Net-Class ▸) | board + schematic | Altium properties; one verb many types | CONTEXT_MENU_CONTENT |
| Silk/courtyard/body drawing primitives (line/rect/circle/poly/arc/text) | footprint, package, symbol, board, schematic | shared drawable-item interface (KiCad `EDA_ITEM`, Eagle DTD) | new spec |
| Footprint & symbol editors | footprint, symbol | must be `ViewportProfile`s / personas, no new mechanism | 023 + profiles |
| Diff-pair / length-match / controlled-impedance as net-class attributes + kernel | board routing, rules | not dedicated tools — attributes + shared solver | PCB/RULES contracts |
| Zone-fill pour solver (derived-geometry engine) | board, manufacturing, rules | shared geometry library (Layer 3) | NATIVE_FORMAT/PCB |
| Panelization / 3D / STEP-IDF-ODB++ export | manufacturing, 3D | **3D-viewer boundary: share model/identity/dispatch, not the 2D canvas** (KiCad) | standards stubs |
| Sync / one-change coherence (sch↔board) | schematic, board | **emergent** from one model + identity (vs Altium ECO / Eagle F-B annotation) | resolver (intrinsic) |

---

## 4. Cross-domain design rules (recommended Datum policy)

Distilled from the four domains; each becomes a review lens for shared-tooling work:

1. **Shared services below, thin personas above.** An editor owns only its verb-set,
   view-providers, one inspector, and read/write posture. It must never grow a
   private selection set, undo stack, snapping, or navigation — in Datum's terms that
   *is* a private write path / rival authority, already forbidden.
2. **Build shared from day one.** KiCad's PCB-only align/distribute (issue #6769) and
   PCB-only SWIG bindings are documented retrofit debt. A capability that will span
   editors is built as one cross-editor service the first time, not per-editor.
3. **The sharing boundary is data + identity + dispatch, not necessarily pixels.**
   A different render target (3D) may legitimately not share the 2D canvas while
   still sharing the model, identity, selection, and verbs.
4. **One selection service is the highest-leverage primitive.** It is the app-wide,
   observer-fed singleton every tool, inspector, and menu joins; Datum's is currently
   a thin single-target review enum absent from the engine — the biggest single gap.
5. **One geometry/constraint solver library.** The most error-prone, highest-value
   code to never duplicate; D-Cubed proves one solver serves many editors and many
   domains. Datum's geometric reasoning belongs behind one engine-owned boundary.
6. **Data-model-driven property inspector.** Declare typed property metadata on each
   object class; one generic inspector renders and binds it — a new object type
   touches no panel code.
7. **No private write path, enforced even through scripting.** Eagle's ULP is
   read-only and must emit commands into the one engine — the historical proof of
   Datum's "every change through one `commit()`."
8. **Re-specialization is legitimate — name it.** A shared verb grammar
   ("constrain"/"align"/"measure") with editor-scoped solvers (2D sketch relations
   vs 3D mates; schematic vs board align) is correct, not a failure of unification.
9. **Same tool, consistent feel, context-specific payload.** Standardize the
   *grammar and framework* (how selection/measure/property-edit/menu feel); let each
   editor supply the *contextual command payload* (Fusion marking menu; SolidWorks
   CommandManager tabs).

---

## 5. Sequencing (how the layers map onto the roadmap)

- **Layer 0** — done; the foundation the rest rides on.
- **Layer 1** — decision `023` + `DATUM_UNIVERSAL_VIEWPORT_TOOLING_SPEC.md`; build
  is the S0–S11 slices in the Active Frontier (step 2c). The `EditorViewport`
  keystone + selection gesture here is the precondition for Layer 2.
- **Layer 2** — the selection-identity model is the recommended **first Layer-2
  spec** (it sits under the most Layer-1/2/3 rows and unblocks multi-select,
  context-menu intersection, and the inspector). Property-inspector, one-Measure,
  align/transform, and appearance follow, each as its own decision record + governed
  spec when it reaches buildable definition.
- **Layer 3** — the one-geometry-solver-library boundary and the shared
  parameter-schema registry are engine-side consolidations, sequenced when a second
  consumer (schematic authoring / zone-fill / footprint constraints) makes the
  duplication concrete.
- **Layer 4** — folded in as each roadmap capability reaches buildable definition,
  each as a persona/profile or attribute over the shared services, never new
  mechanism.

Ordering intent lives in the **Active Frontier** (`specs/PROGRESS.md`); this section
names layers, not a rival next-step ordering.

---

## 6. Governance

- **Class:** governed reference; tracked in `specs/spec_governance_manifest.json`
  (`entries` + `tracked_docs`) and woven into the Active Frontier (step 2c) as the
  parent catalogue under which decision `023` is the Layer-1 component.
- This catalogue **recommends**; each shared capability that ratifies mechanism
  requires its own numbered decision record + governed spec + parity registration
  when it reaches buildable definition. Nothing here authorizes building Layers 2–4
  without that per-capability governance and owner authorization.
- **Not a rival roadmap:** the ordered next-step sequence stays in the Active
  Frontier; this document classifies and layers, and links down to detail.

---

## 7. Sources

**Internal:** `CLAUDE.md`; `docs/DATUM_PRODUCT_MECHANICS.md`; `docs/CANONICAL_IR.md`;
`docs/contracts/AI_CLI_MCP_TOOL_SURFACE.md`; `docs/gui/DATUM_GUI_PARAMETRIC_TOOLING.md`;
`docs/gui/DATUM_GUI_CONTEXT_MENU_CONTENT.md`; decision `023` +
`docs/gui/DATUM_UNIVERSAL_VIEWPORT_TOOLING_SPEC.md`; code:
`crates/engine/src/substrate/{operation,commit,undo_redo,project_resolver,mod}.rs`,
`crates/engine/src/api/native_write/{mod,context,guards,ids,registry}.rs`,
`crates/verb-registry/src/{lib,catalog}.rs`,
`crates/gui-protocol/src/{lib,context_envelope}.rs`,
`crates/gui-render/src/side_panels/render_inspector.rs`.

**EDA:** KiCad Doxygen + dev-docs (`VIEW`/`PAINTER`/`GAL`, `TOOL_MANAGER`/
`ACTION_MANAGER`/`TOOL_ACTION`, `COMMIT`, `PROPERTY_MANAGER`/`PROPERTIES_PANEL`,
`MARKER_BASE`/`RC_ITEM`, `KIWAY`/`KIFACE`, `EDA_UNITS`/`UNIT_BINDER`, IPC API;
align/distribute issue #6769); Horizon EDA source + author blog (`ImpBase`,
`CanvasGL`, `Core`/`IDocument`, `HistoryManager`, `Selectables`, headless DRC split);
Altium TechDocs (Unified Design Environment, Properties panel, Query Language, ECO,
Cross Probe, unified cursor-snap, PME); Autodesk Eagle official help + DTD (GRID/
WINDOW/GROUP/CHANGE commands, User Language, `<layers>`/`<grid>` grammar, F/B
annotation).

**Mechanical/parametric CAD:** SolidWorks help (CommandManager, PropertyManager
`IPropertyManagerPage2`, FeatureManager, Measure, Selection Filters, Smart Dimension);
Autodesk Fusion 360 help + API (single `.f3d`, workspaces, marking menu, ViewCube);
FreeCAD SourceDoc/GitHub/DeepWiki (workbench architecture, App/Gui MVC split,
`SelectionSingleton`, Std Measure, planegcs); PTC Creo + Siemens NX (Creo modes/
TOOLKIT; NX master-model + Application switch + NX Open); Siemens D-Cubed 2D DCM;
SolveSpace / planegcs (solver-as-library).

**Creative / multi-context:** Affinity (Persona model, StudioLink); Adobe
(Workspaces, Smart Guides); Figma (Design/Dev/Prototype modes over one canvas);
QGIS (shared layers + Map Themes + lock); Bitwig/Ableton (global transport/grid bar,
shared quantize; the idiom Datum's GUI borrows).

Full per-domain source URL lists are retained in the research record and available
on request.
