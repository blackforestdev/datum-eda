# Product Mechanics 023: Universal Editor-Interaction and Viewport Toolkit

Status: ratified doctrine

## Decision

Every Datum drawing surface — schematic, board, and every future editor
(footprint, symbol, and any later canvas) — shares ONE consumer-side
editor-interaction and viewport backbone. Grid, camera, coordinate/hit-test,
snap, stroke weight, hover, selection, tool-mode, the local (context/marking)
menu, coordinate readout, and per-surface layer visibility are single, shared
mechanisms that each surface **configures**, never reimplements.

A drawing surface differs from another only by a `ViewportProfile` — a small
bundle of per-surface configuration (which object types are hittable/snappable,
its grid pitch table and colors, its tool set and keymap, its menu-key
namespace, its layer set). A surface MUST NOT carry its own grid, camera, snap,
hit-test, selection, menu, or readout implementation.

This decision is controlling for consumer-side editor interaction. Operational
documents, the governed toolkit spec, manifests, and gates may implement or
strengthen it, but may not weaken it. It is subordinate to, and does not
restate, the higher decisions it builds on (§ Relationship).

## Why This Is Required

The schematic grid was implemented a second time, in a different coordinate
space with different weight behavior, so it thickened on zoom while the board
grid did not. Investigation showed the grid was only the visible symptom: the
**entire per-viewport interaction class** (tool-mode, hover, selection, marquee,
the context menu, coordinate readout, cursor, snap, per-editor keybinding) was
board-only or absent for the schematic, funneling through two board-only
chokepoints, with the schematic pane coded "non-interactive." Two divergent
implementations of the same capability is exactly the "a tool per
object-class... a redundant tool is a defect" failure the project's Lean ethos
forbids, reached by accretion rather than by decision.

The correction is not to fix the grid twice-over but to make the shared backbone
a governed requirement, so no future editor can fork the interaction rules again
and every editor inherits the same behavior for free. The field precedent is
Horizon EDA, whose single canvas is inherited by every editor and varies only by
which object types register as snap/hit targets — the model this decision
ratifies for Datum.

## Normative Rules

The key words **MUST**, **MUST NOT**, **REQUIRED**, **SHOULD**, and **MAY** are
normative.

### UVT-001: One shared backbone, configured per surface

The shared interaction/viewport mechanisms MUST exist exactly once and be
consumed by every drawing surface through a per-surface `ViewportProfile`. A new
editor is created by authoring a profile, not by adding a grid, camera, snap,
hit-test, selection, menu, or readout implementation. Adding a second
implementation of any of these capabilities is a defect under this decision.

### UVT-002: Consumer-side, engine-free

The toolkit MUST live in the GUI/consumer layer and MUST NOT introduce any
dependency from the engine, daemon, protocol, or persisted project formats onto
it. Camera, grid, snap, hover, selection, and menu are view concerns; the engine
owns only units, coordinates, stable IDs, and typed operations. This specializes
the decision-014 layout-solver fence for the interaction toolkit: the boundary
MUST be a compile-time dependency edge, not a convention.

### UVT-003: Render-approach law (retained world vs immediate overlay)

Authored, fab-bearing geometry MUST be rendered from the retained world (nm)
buffer. Grid, selection, hover, cursor, snap feedback, and menu MUST be rendered
as an immediate screen-space overlay driven by the live camera. This law applies
to every surface identically. Baking presentation chrome (grid, selection) into
a surface's world buffer — the defect that made the schematic grid scale with
zoom — is prohibited. This preserves `render == CAM` (Law 1): only presentation
channels live in the overlay; authored geometry stays retained and unaltered.

### UVT-004: The `EditorViewport` keystone

Screen↔world projection and hit-testing MUST be resolved for the **focused**
surface in that surface's own camera and coordinate space, through one
`EditorViewport` abstraction. No interaction capability may hard-code a single
surface's viewport or camera as the sole coordinate authority. Every surface,
including every non-board surface, MUST be able to map a screen point to world
and hit-test its own geometry.

### UVT-005: Stroke weight classes

Every renderable primitive MUST declare exactly one weight class:

- **A — ScreenConstant:** a fixed device-pixel weight that never scales with
  zoom (grid, selection, cursor, hover, marquee, and other presentation chrome).
- **B — WorldWidthWithMinClamp:** a true world (nm) width that scales with zoom
  but never renders thinner than a minimum device-pixel clamp (authored copper,
  pads, vias, silk, and drawn schematic wires/buses/pins — real geometry).
- **C — AuthoredConstantNm:** a fixed nm nominal width sourced from an authored
  literal; renders as class B.

Class B/C exists because fab-bearing geometry has real width and MUST scale with
zoom (`render == CAM`). Class A exists because presentation chrome MUST stay
readable at any zoom. The primitive→class assignment is specified by the
governed toolkit spec.

### UVT-006: Snap gesture is not an operation; the committed result is

Snapping-in-progress (quantization preview, engaged-target highlight,
rubber-band, live readout, cursor recolor) is consumer-only view state and MUST
NOT be journaled. On commit, the final snapped coordinate MUST be an exact
i64-nm argument to a typed operation through the one `commit()` + journal, with
provenance, diff, and undo. This specializes the decision-020 / operation-model
boundary for snap. "Quantize a selection onto the active grid" MUST be a
parameter of the existing align verb (`reference: grid`), never a new sibling
tool.

### UVT-007: The local menu is contextual and verb-firing on every surface

The local (context/marking) menu MUST resolve its content per surface from the
focused `EditorViewport`'s hit-test result, the current selection (multi-select
= intersection of per-type menus), and the surface's menu-key namespace.
Selecting a leaf MUST fire that leaf's bound typed verb — on every surface — as
one of the tri-modal ways (marking-menu preset / tool-inspector form / AI
intent) to reach the same operation. A menu that opens only over one surface, or
whose leaves do not fire verbs, does not satisfy this rule.

### UVT-008: Naming discipline

The consumer interaction surface is named `EditorViewport` (or `ViewportSurface`)
and MUST NOT be called a bare "viewport." It is distinct from the decision-020
**paper-space viewport** (a journaled sheet property) and from the decision-021
**workspace pane** (session layout state). Camera and interaction state are
consumer/session state and MUST NOT be journaled.

### UVT-009: Blocking governance and staged delivery

The governed toolkit spec MUST carry an honest check disposition
(ENFORCED / TO-ENFORCE / HUMAN) for each claim, and MUST be woven into the
Active Frontier with its dependencies. Delivery MAY be staged in slices, but
each slice MUST preserve the board's byte-identical visual-parity golden (or
record a deliberate, owner-blessed re-bless) and MUST honor source-health
burn-down (decision 022) when it touches an oversized module. A slice MUST NOT
introduce a per-surface fork of a capability this decision requires to be shared.

## Relationship to Existing Decisions

This decision builds on and is subordinate to:

- **The operation model + Lean ethos (`CLAUDE.md`, PM-000 series):** one mutation
  path; capabilities are parameters of a small verb set. UVT-001/006/007 apply
  that ethos to interaction tooling.
- **Decision 014 (UI layout system):** UVT-002 specializes the engine-free fence
  for the interaction toolkit.
- **Decision 020 (paper-space and viewports):** UVT-006 reuses its
  gesture-vs-journaled-operation boundary; UVT-008 keeps the paper-space viewport
  distinct.
- **Decision 021 (workspace pane tiling):** UVT-004 follows the per-pane warm
  camera as its precedent; UVT-008 keeps the workspace pane distinct.
- **Decision 022 (source-health governance):** UVT-009 requires burn-down when a
  slice touches a monolith.

It does not amend any of them; on conflict the higher decision wins and this
document is the one to fix.

## Consequences

Every editor Datum ever ships inherits grid, camera, snap, hit-test, selection,
menu, and readout behavior identically, for the cost of one `ViewportProfile`.
The schematic becomes a real interactive editor and the grid bug is fixed as a
by-product of unification rather than as a second grid. The cost is that the
first editors pay to extract the shared backbone out of the board-only code, and
each extraction slice carries the source-health obligation of decision 022. That
cost is paid once; the alternative — a divergent interaction stack per editor —
is the failure this decision exists to prevent.
