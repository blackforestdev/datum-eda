# M7-IMP-014 Implementation Plan

> **Ticket**: `M7-IMP-014`
> **Status**: Active implementation plan
> **Primary design input**:
> `research/pcb-text-rendering/PCB_TEXT_RENDERING_RESEARCH.md`
> **Companion brief**:
> `docs/gui/M7_IMP_014_IMPORTED_TEXT_NORMALIZATION_BRIEF.md`
> **Phase 2 successor**:
> `docs/gui/DATUM_TEXT_ENGINE_PHASE_2_BRIEF.md`

## Purpose

Turn the PCB text rendering research and the revised `M7-IMP-014` brief into a
 concrete execution plan with explicit module boundaries, landing order,
 validation surfaces, and non-goals.

This plan is for implementation sequencing, not policy debate. The research and
 brief already settle the product direction:
- Datum owns imported-board text geometry
- Phase 1 uses a Datum-owned Newstroke-equivalent generator
- `render_cache` is not final render truth
- no KiCad runtime dependency is allowed

After Phase 1 lands and stabilizes, the next text-engine expansion work is
tracked under:
- `docs/gui/DATUM_TEXT_ENGINE_PHASE_2_BRIEF.md`
- `docs/gui/DATUM_TEXT_ENGINE_PHASE_2_IMPLEMENTATION_PLAN.md`

## Phase 1 Goal

Land a self-contained Datum-owned imported-board text engine that:
- produces Newstroke-equivalent stroke geometry from imported text semantics
- routes both `datum-test` and `DOA2526` through the same generation path
- removes `render_cache` from final imported-board text rendering
- documents the expected visual change for imported TrueType-authored text

This phase does **not** preserve KiCad TrueType / `OUTLINE_FONT` visual
 identity. It normalizes imported text onto Datum-owned Newstroke-equivalent
 output.

## Canonical Architecture

### 1. Core text module

Add a Datum-owned text module under the engine:

- `crates/engine/src/text/mod.rs`
- `crates/engine/src/text/newstroke_data.rs`
- `crates/engine/src/text/layout.rs`
- `crates/engine/src/text/geometry.rs`

Responsibilities:

- `newstroke_data.rs`
  - vendored CC0 Newstroke-equivalent glyph data
  - codepoint lookup
  - glyph metrics (left/right bounds, raw stroke points)

- `layout.rs`
  - convert semantic text attributes into positioned glyphs
  - apply line breaking, multiline stacking, justification, tab/space policy,
    keep-upright normalization, mirror semantics, italic shear if supported in
    Phase 1

- `geometry.rs`
  - convert positioned glyphs into canonical stroke/polyline geometry
  - expose deterministic helpers for later polygon derivation

- `mod.rs`
  - stable API surface used by import, GUI scene build, and export

### 2. Semantic input model

Extend the engine-owned text semantic model instead of keeping the current
 under-specified `BoardText`.

Minimum additional fields for Phase 1:
- `h_justify`
- `v_justify`
- `mirrored`
- `keep_upright`
- `multiline`
- `line_spacing_ratio`

Recommended location:
- evolve `crates/engine/src/board/board_types.rs` `BoardText`

If changing `BoardText` directly causes excessive blast radius, introduce a
 parallel internal layout struct in the text module first, then fold it back
 into `BoardText` in a follow-on.

### 3. Import adapter

The KiCad import/scene layer remains an adapter:

- parse KiCad text semantics
- map them into Datum text semantics
- call the engine text module
- emit Datum-owned geometry only

This work primarily lands in:
- `crates/gui-protocol/src/lib.rs`

### 4. Renderer contract

The renderer consumes only Datum-owned geometry primitives for imported-board
 text.

Phase 1 renderer rule:
- no special imported-text branch based on `cached_polygons`
- no dependence on `component_texts` for imported KiCad text
- imported-board text should already be materialized as geometry before render

## Canonical Output

Phase 1 canonical output is:
- stroke/polyline geometry

Derived outputs later:
- polygon tessellation for export consumers that need filled outlines

Do not design Phase 1 as a polygon-cache replacement.

## Landing Order

### Phase 1A: Engine text module

Goal:
- create the reusable Datum-owned Newstroke-equivalent generator

Patch surface:
- add `crates/engine/src/text/*`
- export a small public API from the engine crate

Suggested API shape:

```rust
pub enum TextHAlign { Left, Center, Right }
pub enum TextVAlign { Top, Center, Bottom }

pub struct TextAttributes {
    pub position: Point,
    pub rotation_degrees: i32,
    pub height_nm: i64,
    pub stroke_width_nm: i64,
    pub h_align: TextHAlign,
    pub v_align: TextVAlign,
    pub mirrored: bool,
    pub keep_upright: bool,
    pub line_spacing_ratio_ppm: i32,
}

pub struct StrokeSegment {
    pub from: Point,
    pub to: Point,
    pub width_nm: i64,
}

pub fn layout_text_strokes(text: &str, attrs: &TextAttributes) -> Vec<StrokeSegment>;
```

Notes:
- keep this API small
- point/segment ownership should match engine geometry types where practical
- if `BoardText` can be upgraded cleanly, `layout_text_strokes(&BoardText)` is
  acceptable

### Phase 1B: Replace the toy ASCII font

Goal:
- retire the hand-authored ASCII-only generator in
  `crates/engine/src/export/silkscreen.rs`

Patch surface:
- `crates/engine/src/export/silkscreen.rs`
- use the new engine text module internally

Required outcome:
- export and imported-board generation converge on the same Newstroke-equivalent
  text engine

This is important because otherwise import and export remain two different text
 systems under different names.

### Phase 1C: Extend KiCad semantic parsing

Goal:
- parse enough real text semantics to feed the engine cleanly

Patch surface:
- `crates/gui-protocol/src/lib.rs`

Required parsing coverage:
- text content
- layer
- `at`
- font size / thickness
- justify
- keep-upright for footprint-owned text
- mirror semantics where the imported-board subset requires it
- multiline when present in accepted fixtures
- hide/visibility handling for `Reference`/`Value`

### Phase 1D: Remove `render_cache` as final imported truth

Goal:
- `render_cache` no longer determines final imported-board text geometry

Patch surface:
- `crates/gui-protocol/src/lib.rs`
- maybe minor cleanup in `crates/gui-render/src/lib.rs`

Required change:
- cached and non-cached imported text both call the same Datum text generator
- `cached_polygons` stop being a final-render path for imported-board text

Allowed remaining use of `render_cache`:
- debug-only
- test/reference utilities
- bounded local investigation of TrueType-authored fixtures

### Phase 1E: Validation and fixture proof

Goal:
- prove both canonical fixtures land on the same path

Patch surface:
- `crates/gui-protocol/src/lib.rs` tests
- possibly new engine text tests under `crates/engine/src/text/tests.rs`

Required proof:
- `datum-test`
- `DOA2526`
- no KiCad runtime dependency in the production path

## Concrete File Map

### Create

- `crates/engine/src/text/mod.rs`
- `crates/engine/src/text/newstroke_data.rs`
- `crates/engine/src/text/layout.rs`
- `crates/engine/src/text/geometry.rs`
- `docs/gui/M7_IMP_014_IMPLEMENTATION_PLAN.md`

### Update

- `crates/engine/src/lib.rs`
- `crates/engine/src/board/board_types.rs`
- `crates/engine/src/export/silkscreen.rs`
- `crates/gui-protocol/src/lib.rs`
- `crates/gui-render/src/lib.rs` only if cleanup remains after scene changes

## Data Strategy

Phase 1 data source:
- vendored CC0 Newstroke-equivalent source per the research note

Do:
- store source as static data inside the repo
- validate glyph decoding at build/test time
- document provenance clearly

Do not:
- scrape KiCad at runtime
- shell out to external tools
- copy KiCad's GPL-wrapped C++ file as the implementation basis

## Acceptance Gates By Phase

### Gate A: module exists

- engine text module builds
- glyph lookup works for ASCII + required board punctuation
- missing glyph behavior is explicit

### Gate B: export convergence

- `render_silkscreen_text_strokes` uses the new module
- old toy ASCII glyph table is removed or reduced to a temporary shim

### Gate C: import convergence

- imported text no longer branches on `render_cache` for final geometry
- `component_texts` no longer carries imported KiCad text semantics as a
  fallback lane

### Gate D: fixture proof

- `datum-test` and `DOA2526` both use the same Datum-owned path
- the expected TrueType-authored visual change is documented in closeout

## Non-Goals For Phase 1

- imported TrueType / `OUTLINE_FONT` fidelity preservation
- arbitrary system font support
- RTL shaping
- full Unicode parity beyond what the vendored Newstroke-equivalent source
  provides
- decorative/logo text quality above PCB-review fidelity needs

## Phase 2+ Follow-Ons

### Phase 2: Better semantic completeness

- fuller mirror handling
- superscript/subscript if fixture-relevant
- explicit tab semantics if fixture-relevant

### Phase 3: Polygon derivation utilities

- stroke-to-polygon helper for export and any polygon-only consumers
- keep strokes canonical

### Phase 4: TrueType / `OUTLINE_FONT`

- detect imported non-default font usage explicitly
- resolve imported TrueType if available through Datum-owned parsing
- no KiCad runtime dependency

This is the phase that can restore fixtures like `DOA2526` closer to their
 original KiCad-authored TrueType look.

## Recommended Immediate Patch Order

1. Add engine text module with vendored Newstroke-equivalent data
2. Repoint `crates/engine/src/export/silkscreen.rs` to the new module
3. Extend imported text semantics parsing in `crates/gui-protocol/src/lib.rs`
4. Remove `render_cache` as final imported text geometry
5. Rewrite tests so both `datum-test` and `DOA2526` assert the same Datum path

## Completion Definition

`M7-IMP-014` Phase 1 is done when:
- the repo has one Datum-owned imported-board text engine
- export and imported-board generation use the same core text module
- the production path has no KiCad runtime dependency
- `render_cache` is no longer the reason a fixture looks credible
- docs and closeout explicitly note the TrueType-authored visual tradeoff
