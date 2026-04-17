# M7 Imported Board Fidelity Checklist

> **Status**: Active execution checklist for the imported-board fidelity track
> inside the opening `M7` milestone.
> This checklist derives from
> `docs/gui/M7_IMPORTED_BOARD_FIDELITY_PLAN.md`.

## Purpose

Turn the imported-board fidelity plan into concrete issue-sized work items the
team can pick up, sequence, and close.

This checklist covers:
- fixture freeze
- KiCad PCB import truth preservation
- board review scene-contract expansion
- renderer semantic readability
- regression coverage

This checklist does **not** broaden `M7` into:
- editing or apply flows
- schematic review
- 3D review
- new routing semantics

## Working Rules

- Keep work grouped by `import`, `scene-contract`, and `renderer`.
- Close Stage 0 before treating later visual feedback as authoritative.
- Close Stage 1 before claiming imported KiCad board review is trustworthy.
- Do not use renderer polish to mask missing import or scene truth.
- Apply the delivery/testability gate in
  `docs/gui/M7_DELIVERY_GATES.md` before treating any user-facing slice as
  "done enough" to move on.
- When a slice touches standards-bound PCB semantics such as land-pattern
  geometry, solder mask expansion, stencil/paste aperture reduction, or other
  manufacturing-facing observables, use the relevant research note as an
  explicit design input rather than silently inheriting an EDA-package default.
  Current required reference:
  `research/ipc-compliance/IPC_COMPLIANCE_RESEARCH.md`
- Update `specs/PROGRESS.md` when stage-level status changes.
- Keep the active defect inventory in
  `docs/gui/M7_IMPORTED_BOARD_FIDELITY_ISSUES.md`.

## Stage 0: Freeze The Truth Set

### Fixture Freeze

- [x] `M7-FIX-001` Create and check in the imported-board fidelity fixture
  manifest.
  Owner: `engine/import` + `gui-protocol`
  Output:
  - canonical half-routed Datum board
  - multilayer KiCad board
  - pad-shape fidelity board
  - outline/zone edge-case board within supported ownership rules
  Working doc:
  - `docs/gui/M7_IMPORTED_BOARD_FIDELITY_FIXTURES.md`
  Acceptance note:
  - satisfied by the current manifest with the canonical `datum-test` board
    explicitly accepted as a stable external/local fixture until it is
    vendored

- [x] `M7-FIX-002` Define the required reference artifacts for each fixture.
  Owner: `gui-protocol` + `gui-render`
  Output:
  - source `.kicad_pcb`
  - Datum launch path or scene payload path
  - KiCad screenshot reference when useful
  - Datum screenshot baseline target name
  Working doc:
  - `docs/gui/M7_IMPORTED_BOARD_FIDELITY_ARTIFACTS.md`
  Acceptance note:
  - satisfied by the current artifact matrix and naming rules for the accepted
    local canonical fixture plus the repo-native fallback fixtures

- [x] `M7-FIX-003` Create the first issue inventory grouped by `import`,
  `scene-contract`, and `renderer`.
  Owner: project lead / architect
  Output:
  - one tracked list with severity and owning crate
  Working doc:
  - `docs/gui/M7_IMPORTED_BOARD_FIDELITY_ISSUES.md`
  Acceptance note:
  - satisfied by the current issue inventory with stage mapping, severity,
    evidence, and linked implementation/decsion briefs

Exit for Stage 0:
- the team is using the same boards, the same screenshots, and the same defect
  groupings

## Stage 1: Stop Silent Import Corruption

### Layer Identity

- [x] `M7-IMP-001` Remove silent fallback that collapses unresolved imported
  KiCad layer identities onto default copper layers.
  Owner: `crates/engine/src/import/kicad`
  Minimum proof:
  - supported multilayer fixture retains expected layer identities
  - unsupported layer-table cases fail explicitly or remain clearly bounded
  Acceptance note:
  - completed with explicit unknown-layer import errors, audited call-site
    propagation, and six focused layer-resolution tests including inner-layer
    table coverage
  Validation maintenance note:
  - pre-existing `PlacedPad.drill` fallout in engine tests was resolved as
    mechanical build maintenance so the Stage 1 validation surface could run;
    this was not counted as semantic scope for `M7-IMP-001`

- [ ] `M7-IMP-002` Add fixture-backed tests for KiCad layer-table parsing across
  normal spacing/format variants.
  Owner: `crates/engine/src/import/kicad`
  Minimum proof:
  - tests cover non-tab-indented layer tables and inner-layer cases

### Outline Ownership

- [ ] `M7-IMP-003` Document and enforce supported KiCad board-outline ownership
  rules for imported board review.
  Owner: `crates/engine/src/import/kicad`
  Minimum proof:
  - top-level `Edge.Cuts` support is explicit
  - unsupported footprint-embedded outline tricks do not silently claim success

- [ ] `M7-IMP-004` Add fixture-backed outline tests for supported imported
  boards.
  Owner: `crates/engine/src/import/kicad`
  Minimum proof:
  - expected outline survives import on canonical supported boards

### Explicit Bounding

- [ ] `M7-IMP-005` Audit KiCad PCB import paths for other silent fidelity
  degradation cases and convert them to explicit bounded behavior where needed.
  Owner: `crates/engine/src/import/kicad`
  Minimum proof:
  - unsupported cases do not silently produce materially wrong board meaning

Exit for Stage 1:
- imported supported cases stop silently producing wrong board truth
  Current read:
  - partially satisfied
  - `M7-IMP-001` complete
  - Stage 1 still blocked on `M7-IMP-003`

## Stage 2: Raise Pad And Footprint Fidelity

### Pad Semantics

- [ ] `M7-IMP-006` Preserve supported pad shape semantics for `circle`, `rect`,
  `oval`, and `roundrect`.
  Owner: `crates/engine/src/import/kicad`
  Minimum proof:
  - imported board state retains shape kind correctly for canonical fixtures

- [ ] `M7-IMP-007` Preserve supported pad width/height/drill semantics without
  collapsing to coarse approximations.
  Owner: `crates/engine/src/import/kicad`
  Minimum proof:
  - fixture-backed checks for exact imported dimensions on supported pads

- [ ] `M7-IMP-008` Preserve or explicitly bound supported pad rotation
  semantics.
  Owner: `crates/engine/src/import/kicad`
  Minimum proof:
  - rotated non-circular pad fixtures no longer silently distort geometry

- [ ] `M7-IMP-009` Define the supported roundrect / radiused-corner contract
  for imported board review.
  Owner: `crates/engine/src/import/kicad` + `gui-protocol`
  Minimum proof:
  - the frontend receives enough data to distinguish roundrect pads from plain
    rectangles on supported boards

### Footprint Display Companions

- [ ] `M7-SCN-001` Decide the minimum footprint-native display primitives
  required for credible imported-board review.
  Owner: `crates/gui-protocol`
  Output:
  - explicit decision on package-body / silkscreen / mechanical companion data
    needed in the board review scene

- [ ] `M7-SCN-002` Expand the board review scene contract with the accepted
  footprint display companions.
  Owner: `crates/gui-protocol`
  Minimum proof:
  - the canonical fixture no longer reads only as coarse component boxes plus
    generic pad dots

Exit for Stage 2:
- supported imported pads and footprint companions are credible enough for
  review

## Stage 3: Add Unrouted Connectivity As A First-Class Scene Lane

### Scene Contract

- [x] `M7-SCN-003` Define explicit scene primitives for imported-board unrouted
  connectivity / ratsnest state.
  Owner: `crates/gui-protocol`
  Minimum proof:
  - authored, unrouted, proposed, and diagnostic lanes are separate categories
  Acceptance note:
  - completed with explicit `unrouted_primitives` in
    `board_review_scene_v1`, a dedicated `UNROUTED` filter lane, and
    engine-backed identity fields carried through the imported-board path

- [x] `M7-SCN-004` Populate unrouted connectivity from engine-backed imported
  board truth rather than renderer inference.
  Owner: `crates/gui-protocol` + engine query surface
  Minimum proof:
  - canonical half-routed board exposes remaining unrouted state deterministically
  Acceptance note:
  - completed with `Board::unrouted()` endpoint positions flowing into
    imported-board scene primitives; the canonical `datum-test` board now
    exposes deterministic ratsnest spans from engine truth rather than
    renderer-side guesses

- [x] `M7-SCN-005` Add visibility control and stable identity rules for the new
  unrouted lane.
  Owner: `crates/gui-protocol` + `crates/gui-app`
  Minimum proof:
  - visibility toggles and selection can address unrouted primitives cleanly
  Acceptance note:
  - completed with the `UNROUTED` toggle in the shell filter model, stable
    object ids on unrouted primitives, and renderer/app wiring that treats
    unrouted as a first-class scene lane rather than an inferred overlay

Exit for Stage 3:
- the half-routed board clearly distinguishes authored routed copper, remaining
  unrouted connectivity, and proposal overlay geometry
  Current read:
  - satisfied on the accepted canonical half-routed fixture after the
    renderer/world-dash fix

## Stage 4: Lock Semantic Rendering

### Renderer Contract

- [x] `M7-REN-001` Write down the renderer semantic contract for authored,
  unrouted, proposed, and diagnostic states.
  Owner: `crates/gui-render`
  Output:
  - one explicit rendering-vocabulary note linked from the fidelity plan
  Working note:
  - `docs/gui/M7_RENDER_SEMANTIC_CONTRACT.md`
  Acceptance note:
  - completed with one explicit vocabulary/stack note defining the intended
    authored vs unrouted vs proposed vs diagnostic reading for the opening `M7`
    viewport

- [ ] `M7-REN-006` Define and enforce layer/material render discipline for
  authored board geometry.
  Owner: `crates/gui-render`
  Minimum proof:
  - authored imported geometry inherits visibility and base appearance from the
    scene layer/material model by default
  - any remaining special-case render passes are explicit, product-justified,
    and documented as bounded exceptions
  - layer ordering follows a declared render-stack rule: layer type first, then
    front/back side, then stable tie-breakers
  Working note:
  - `docs/gui/M7_RENDER_LAYER_DISCIPLINE_MEMO.md`

- [x] `M7-REN-002` Render unrouted connectivity with a visual grammar that does
  not read like copper.
  Owner: `crates/gui-render`
  Minimum proof:
  - canonical half-routed board matches the intended authored vs unrouted
    distinction
  Acceptance note:
  - completed with engine-backed solid ratsnest lines, subtle endpoint
    anchoring, contrast under-stroke, and deterministic per-net color identity
    on the canonical half-routed fixture; the lane no longer reads as copper
    or as fake via/drill markers

- [ ] `M7-REN-003` Keep proposed overlays copper-like in both selected and
  non-selected states without drifting back toward airwire-like linework.
  Owner: `crates/gui-render`
  Minimum proof:
  - selected-state review still reads as proposed copper plus emphasis, not
    generic path nodes

- [ ] `M7-REN-004` Ensure footprint and board context remain readable under dim
  unrelated and review focus.
  Owner: `crates/gui-render`
  Minimum proof:
  - canonical fixture remains understandable to a PCB reviewer under active
    review focus

Exit for Stage 4:
- the viewport is semantically readable without side-by-side explanation

## Stage 5: Regression Coverage

### Structural And Visual Regression

- [ ] `M7-REG-001` Add fixture-backed import tests covering the supported
  imported KiCad PCB fidelity subset.
  Owner: `crates/engine/src/import/kicad`

- [ ] `M7-REG-002` Add scene-contract tests covering authored / unrouted /
  proposed / diagnostic category presence and identity stability.
  Owner: `crates/gui-protocol`

- [ ] `M7-REG-003` Add screenshot or image-based regression checks for the
  canonical imported-board review states.
  Owner: `crates/gui-render` + `crates/gui-app`

- [ ] `M7-REG-004` Add one standing human-review checklist for the canonical
  half-routed board.
  Owner: project lead / architect
  Output:
  - quick yes/no review prompts for authored copper, unrouted connectivity,
    proposal overlay, and footprint-context readability

Exit for Stage 5:
- imported-board fidelity is protected by both structural and visual checks

## Completion Rule

This checklist is complete only when:
- Stage 1 through Stage 5 exits are met
- the acceptance gates in
  `docs/gui/M7_IMPORTED_BOARD_FIDELITY_PLAN.md` are met
- `specs/PROGRESS.md` is updated to reflect the resulting milestone status

## Standards-Bounded Addition

The standards/compliance program does not broaden opening `M7` into a full IPC
footprint-authoring milestone.

It does tighten what counts as acceptable imported-board review work:
- standards-relevant imported observables already exposed in the review surface
  must preserve source truth
- the team may not silently derive manufacturability-relevant geometry from a
  host-EDA default where source data exists
- bounded import-audit diagnostics are in-scope when they report delta without
  mutating imported source geometry

For the current opening `M7` track, this especially applies to:
- copper pad geometry
- drill and annular ring
- solder-mask aperture policy
- paste-aperture policy
- thermal-pad and thermal-via treatment where present

This does **not** make the following part of opening `M7` by default:
- full IPC footprint generation
- full-library standards enforcement
- general compliance claims
