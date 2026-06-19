# Datum Text Engine Phase 2 Implementation Plan

> **Status**: Active implementation plan
> **Primary research input**:
> `research/pcb-text-rendering/DATUM_TEXT_ENGINE_PHASE_2_RESEARCH.md`
> **Companion brief**:
> `docs/gui/DATUM_TEXT_ENGINE_PHASE_2_BRIEF.md`
> **Outline ownership note**:
> `docs/gui/DATUM_TEXT_ENGINE_OUTLINE_FILL_OWNERSHIP_NOTE.md`
> **Outline attainment note**:
> `docs/gui/DATUM_TEXT_ENGINE_OUTLINE_FILL_ATTAINMENT_NOTE.md`
> **Outline geometry contract note**:
> `docs/gui/DATUM_TEXT_ENGINE_OUTLINE_GEOMETRY_CONTRACT_NOTE.md`
> **Phase 1 predecessor**:
> `docs/gui/M7_IMP_014_IMPLEMENTATION_PLAN.md`

## Purpose

Turn the Phase 2 text-engine research into a concrete landing order for
evolving Datum from a working owned stroke engine into a dual-mode text
platform.

This plan is for execution sequencing. The architecture and product direction
are already settled by the research and the brief.

## Phase 2 Goal

Land a backend-extensible Datum text platform that:
- preserves the existing stroke engine as one technical backend, not as a
  permission slip for lower fidelity
- adds an outline backend without creating a visual caste system between
  manufacturing and non-manufacturing text
- introduces explicit render intent and backend selection
- keeps determinism and fab constraints as first-class requirements

Doctrine:
- all Datum text should target beautiful, high-fidelity output
- manufacturing intent is a validation/export policy layer, not a downgrade
  layer

## Canonical Architecture

### 1. Semantic text model

Add or evolve an engine-owned semantic model to carry:
- content
- family / style id
- family source (`implicit_default` vs `explicit`) so legacy defaults can be
  promoted by intent without destroying an authored Newstroke/stroke-family
  choice
- size
- weight / emphasis flags
- H/V justify
- rotation
- mirror
- keep-upright
- line spacing
- render intent
- optional text-style/class handle

Recommended location:
- `crates/engine/src/text/semantic.rs`

### 2. Layout engine

Keep layout separate from backend-specific glyph generation.

Responsibilities:
- line breaking
- anchors
- multiline stacking
- upright normalization
- mirror semantics
- bounds
- run segmentation by backend / style

Recommended location:
- evolve `crates/engine/src/text/layout.rs`

### 3. Glyph backend layer

Add explicit backend seams:
- `stroke.rs`
- `outline.rs`
- `backend.rs`
- `registry.rs`

Responsibilities:
- backend lookup
- metrics
- geometry generation
- backend-kind introspection

### 4. Geometry output

Keep two canonical owned geometry classes:
- stroke/polyline geometry
- filled outline/polygon geometry

Renderer and export layers choose consumers based on intent and downstream
requirements, not source CAD origin.

Important rule:
- backend diversity is an implementation detail
- user-facing quality doctrine is uniform
- Datum must not normalize toward uglier text for manufacturing-facing content

## Landing Order

### Phase 2A: Formalize semantic model and render intent

Goal:
- introduce explicit `RenderIntent`
- move current text attributes toward the researched semantic contract

Patch surface:
- `crates/engine/src/text/mod.rs`
- new `crates/engine/src/text/semantic.rs`
- `crates/engine/src/board/board_types.rs` or a bounded adapter layer
- `crates/gui-protocol/src/lib.rs`

Required outcome:
- text runs can declare intent without backend knowledge
- Phase 1 behavior remains the default

Current implementation note:
- native `BoardText` now carries `family_source`
- `implicit_default` allows Datum to apply intent defaults such as Inter for
  manufacturing/annotation
- `explicit` preserves the stored family, including `newstroke`
- CLI `--family` placement/edit operations mark the family as explicit

### Phase 2B: Add backend seam without changing behavior

Goal:
- keep current stroke output, but route it through an explicit backend trait

Patch surface:
- new `crates/engine/src/text/backend.rs`
- new `crates/engine/src/text/stroke.rs`
- `crates/engine/src/text/layout.rs`
- `crates/engine/src/export/silkscreen.rs`

Required outcome:
- current Newstroke-equivalent path is now just one backend implementation
- no visible behavior change required yet

### Phase 2C: Add text registry and bundled font asset plumbing

Goal:
- vendor the researched font bundle and add stable family/style ids

Patch surface:
- `crates/engine/assets/fonts/*`
- new `crates/engine/src/text/registry.rs`
- build/provenance docs

Required bundle:
- `Newstroke`
- `Inter`
- `IBM Plex Sans Condensed`
- `Inter Display`
- `JetBrains Mono`

Required outcome:
- no system font dependency
- stable font family selection from semantic text model

### Phase 2D: Determinism gate for outline path

Goal:
- prevent silent fixture drift before outline backend rolls out

Patch surface:
- outline backend test fixtures
- golden flatten output under engine tests

Required outcome:
- one or more tests that parse a vendored font, flatten fixed glyphs at fixed
  tolerance, and byte-compare output on supported CI architecture

This phase is mandatory before outline backend adoption widens.

### Phase 2E: Outline backend extraction path

Goal:
- add outline extraction and flattening

Patch surface:
- new `crates/engine/src/text/outline.rs`

Required library stack:
- `ttf-parser`
- `kurbo`

Required outcome:
- glyph outline extraction from vendored fonts
- flattened contour output at fixed tolerance

No renderer wiring yet beyond tests if needed.

### Phase 2F: Outline fill / tessellation path

Goal:
- turn flattened contours into renderer/export-safe owned geometry

Patch surface:
- `crates/engine/src/text/outline.rs`
- new support helpers if needed
- `crates/gui-render/src/lib.rs`

Required library stack:
- `lyon`
- `i_overlay`

Required outcome:
- filled outline geometry path for annotation / branding intents
- correct hole/counter handling

### Phase 2G: Engineering-mode stroke expansion / fab geometry helpers

Goal:
- support robust engineering text post-processing and future DRC/export needs

Patch surface:
- `crates/engine/src/text/geometry.rs`
- new engineering-geometry helper module if needed
- DRC integration follow-on

Required library stack:
- `cavalier_contours`

Required outcome:
- robust stroke offset helpers
- join/cap policy control
- sub-tolerance polygon cleanup hooks

Constraint:
- this phase must not be used to justify degrading the visible text engine for
  manufacturing intent
- engineering/fab helpers are policy and geometry utilities, not a mandate for
  uglier authored text

### Phase 2H: DRC / fab profile integration

Goal:
- make text manufacturability constraints explicit and profile-driven

Patch surface:
- DRC text rules
- fab profile assets
- validation/reporting surfaces

Required defaults:
- height floor `0.8 mm`
- stroke floor `0.15 mm`
- conservative profile `1.0 mm / 0.20 mm`

Required behavior:
- authored geometry preserved
- DRC warns/errors explicitly
- no silent resize
- no silent backend downgrade because a text object is manufacturing-facing
- beautiful manufacturing fonts remain valid authored intent unless DRC/fab
  policy flags them

### Phase 2I: Style classes and product-facing defaults

Goal:
- provide scalable text UX that matches serious CAD practice

Patch surface:
- engine semantic/style model
- native authoring surfaces
- GUI/editor affordances later as applicable

Required outcome:
- class-based text styles, not only per-object font dropdowns
- sane defaults for:
  - reference designators
  - values
  - assembly/mechanical notes
  - branding / title text

## Concrete File Map

### Create

- `crates/engine/src/text/semantic.rs`
- `crates/engine/src/text/backend.rs`
- `crates/engine/src/text/stroke.rs`
- `crates/engine/src/text/outline.rs`
- `crates/engine/src/text/registry.rs`
- `docs/gui/DATUM_TEXT_ENGINE_PHASE_2_BRIEF.md`
- `docs/gui/DATUM_TEXT_ENGINE_PHASE_2_IMPLEMENTATION_PLAN.md`

### Update

- `crates/engine/src/text/mod.rs`
- `crates/engine/src/text/layout.rs`
- `crates/engine/src/text/geometry.rs`
- `crates/engine/src/export/silkscreen.rs`
- `crates/engine/src/board/board_types.rs`
- `crates/gui-protocol/src/lib.rs`
- `crates/gui-render/src/lib.rs`

## Validation Surface

### Engine tests

- semantic-model round-trip tests
- stroke backend parity tests
- outline flatten determinism tests
- hole/counter handling tests
- mirror / keep-upright / multiline tests across both backends

### Visual tests

- native board text goldens
- imported board text goldens
- branding / annotation / manufacturing intent comparisons
- the native visual fixture manifest is
  [DATUM_TEXT_ENGINE_FIDELITY_FIXTURES.md](/home/bfadmin/Documents/datum-eda/docs/gui/DATUM_TEXT_ENGINE_FIDELITY_FIXTURES.md)
- the GUI visual regression harness design is
  [DATUM_GUI_VISUAL_REGRESSION_HARNESS.md](/home/bfadmin/Documents/datum-eda/docs/gui/DATUM_GUI_VISUAL_REGRESSION_HARNESS.md)
- the stable native projects live under
  `crates/engine/testdata/golden/text/native/`
- screenshot-golden automation must use those fixtures before Phase 2 text
  rendering is considered regression-protected

### DRC tests

- text too small
- stroke too thin
- aspect ratio out of bounds
- per-fab profile override behavior

## Non-Goals

Phase 2 does not require:
- generic path text
- desktop publishing features
- arbitrary complex-script shaping
- OS font discovery
- replacing the stroke backend as Datum's engineering default

## Immediate Recommended Work Order

1. Phase 2A
2. Phase 2B
3. Phase 2D
4. Phase 2C
5. Phase 2E
6. Phase 2F
7. Phase 2G
8. Phase 2H
9. Phase 2I

Rationale:
- lock the architecture first
- keep current behavior stable while introducing seams
- install the determinism gate before outline work spreads
- only then widen into fonts, outline geometry, and product-facing controls
