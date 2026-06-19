# M7-IMP-014 Implementation Brief

> **Ticket**: `M7-IMP-014`
> **Stage**: Stage 1 follow-on
> **Track**: Imported board fidelity inside opening `M7`
> **Status**: Ready for implementation
> **Required research input**:
> `research/pcb-text-rendering/PCB_TEXT_RENDERING_RESEARCH.md`
> **Execution plan**:
> `docs/gui/M7_IMP_014_IMPLEMENTATION_PLAN.md`

## Purpose

Define the bounded implementation contract for Datum-owned imported-board text
generation so final rendered text fidelity no longer depends on optional KiCad
`render_cache` presence and no longer piggy-backs on KiCad-authored outline
polygons.

This brief exists so the coding pass does not:
- preserve a representation-dependent hybrid by calling cached KiCad polygons
  "good enough"
- route one board through KiCad-authored glyph geometry and another through a
  weak Datum fallback while claiming a single pipeline
- introduce a KiCad runtime dependency (`pcbnew`, `kicad-cli`, or GUI APIs)
  into the importer

## Problem

Imported KiCad text currently has two geometry sources:
- KiCad-authored `render_cache` polygons when present
- Datum-authored synthetic fallback geometry when `render_cache` is absent

This preserves visible board-to-board quality differences even after the
render-time text-path split is removed.

Research correction:
- `render_cache` is **not** a general parity oracle in the way this brief's
  first draft implied
- KiCad emits `render_cache` only for non-default TrueType /
  `OUTLINE_FONT` text, not for default Newstroke text
- the `DOA2526` vs `datum-test` visible quality gap is therefore not simply
  "same font, cache present vs absent"; it is almost certainly
  TrueType-authored text versus default Newstroke-authored text
- Datum is currently piggy-backing on KiCad's pre-tessellated outline polygons
  for the TrueType-authored case

That violates the imported-board semantic program. The board should not become
visually credible or non-credible based only on optional source
representation.

## Scope

This ticket covers:
- imported KiCad PCB text only
- `gr_text`
- `fp_text`
- footprint `Reference` / `Value` properties and equivalent text fields used
  in imported board review
- the geometry generator used to materialize those items into the review scene
- the contract for how `render_cache` may or may not be used
- Phase 1 Datum-owned Newstroke-equivalent text generation

This ticket does **not** cover:
- native-project authored text generation
- schematic text
- generic UI font rendering
- using KiCad as a runtime dependency
- widening opening `M7` into a full text-engine product initiative outside the
  imported-board slice

## Required Rule

For imported KiCad board review in `M7-IMP-014` Phase 1:
- Datum must generate final text geometry from text semantics, not from
  optional cached source geometry
- Datum must implement a Newstroke-equivalent internal generator and use that
  as the canonical imported-board text path
- cache-present and cache-absent boards must land on the same Datum-owned text
  geometry generation path
- KiCad `render_cache` may be used only as a bounded debug/reference surface,
  not as final rendered truth

Not acceptable:
- rendering KiCad `render_cache` polygons directly in the final scene while
  synthesizing geometry only for cache-absent boards
- introducing `pcbnew`, `kicad-cli`, or any installed KiCad runtime as an
  importer requirement
- allowing cache presence to remain the reason one board looks "good" and
  another looks "bad"

Important Phase 1 expectation:
- if a cache-present board was authored with a non-default TrueType /
  `OUTLINE_FONT`, Datum-owned Newstroke-equivalent generation will not preserve
  that exact TrueType appearance
- in that case the correct Phase 1 outcome is representation-invariant Datum
  text, not visual identity to KiCad's original TrueType rendering
- exact preservation of imported TrueType-authored text is a later follow-on,
  not part of Phase 1 completion

## Required Semantic Inputs

The Datum-owned imported-text generator must consume the authored text
semantics directly:
- text content
- layer ownership
- position / anchor
- rotation
- size / height
- stroke thickness where applicable
- horizontal and vertical justification
- mirror / keep-upright behavior where the imported-board subset requires it
- multiline layout where present in accepted fixtures

The generator must not infer final meaning from cached polygon availability.

## Required Output Contract

For imported KiCad board review:
- all imported text materializes as Datum-owned world-space geometry
- the scene contract receives one canonical imported-text geometry result class
- the renderer does not know whether KiCad originally provided `render_cache`

Phase 1 output rule:
- canonical internal output is stroke/polyline geometry
- polygon tessellation is derived on demand for consumers that need it
- the generator must not be designed as polygon-only cache replacement

It must still be one Datum-owned generator, not a representation split.

## KiCad Dependency Rule

The importer must remain self-contained.

Therefore:
- no `pcbnew`
- no `kicad-cli`
- no shelling out to KiCad
- no "works if KiCad is installed" behavior

If Datum needs KiCad-compatible stroke-font data or glyph rules, those must be
vendored into the repo and used internally.

Per the required research note:
- use an internally vendored CC0 Newstroke-equivalent source
- do not copy KiCad's GPL-wrapped C++ source as the implementation basis
- do not treat the current hand-authored ASCII silkscreen font as sufficient
  for imported-board fidelity

## Cache Policy

`render_cache` may be used only for:
- debugging
- localized fixture analysis
- bounded comparison when the source text was authored with non-default
  TrueType and the cache is the only available outline evidence

`render_cache` may not be used for:
- final imported-board render truth
- deciding whether a board gets a higher-fidelity lane than another board
- general Newstroke parity measurement

Research correction:
- Newstroke parity should be measured against KiCad-rendered snapshots or
  equivalent known-good output, not against `render_cache`, because default
  Newstroke text does not emit `render_cache`
- Phase 1 text-engine research
  (`research/pcb-text-rendering/PCB_TEXT_RENDERING_RESEARCH.md`) confirmed
  that KiCad emits `render_cache` only for non-default TrueType /
  `OUTLINE_FONT` text and not for default Newstroke text
- KiCad issue #17666 further confirms that KiCad does not treat the embedded
  cache as authoritative on load; it re-renders text from the source semantics

Implication:
- the `DOA2526` vs `datum-test` quality gap is not proof that Datum's
  Newstroke synthesis is inherently broken
- it is evidence that one fixture was TrueType-authored with cached outline
  polygons while the other fixture used default Newstroke semantics without a
  cache
- the acceptance target is fixture-internal Datum parity under one owned
  generator, not preserving KiCad's original TrueType appearance for
  cache-present boards

## Minimum Code Surface

The implementation pass must inspect at least:
- `crates/gui-protocol/src/lib.rs`
- `crates/engine/src/export/silkscreen.rs`
- `research/pcb-text-rendering/PCB_TEXT_RENDERING_RESEARCH.md`
- any new internal imported-text geometry module introduced for vendored
  Newstroke-equivalent glyph data or layout semantics
- imported-board tests in `crates/gui-protocol/src/lib.rs`

If the implementation introduces a new internal font/glyph module, that module
must be documented as an imported-board normalization dependency rather than an
ad hoc renderer helper.

## Acceptance Criteria

`M7-IMP-014` is complete only when all of the following are true:
- imported KiCad text no longer uses `render_cache` as final render truth
- cache-present and cache-absent fixtures go through the same Datum-owned
  Newstroke-equivalent text generation path
- no KiCad runtime dependency is introduced
- `datum-test` and `DOA2526` no longer differ in visible text quality solely
  because one fixture had cache polygons and the other did not
- acceptance is based on both fixtures using the same Datum-owned text
  generator, not on `DOA2526` matching its original KiCad TrueType render
- fixture-backed tests prove representation-invariant imported text behavior
- the expected visual change for imported TrueType-authored text is documented
  rather than treated as an unexpected regression
- preserving imported TrueType / `OUTLINE_FONT` appearance remains a later
  text-engine scope item and does not gate `M7-IMP-014` Phase 1 closeout

## Minimum Proof

Implementation proof must use the frozen Stage 0 fixture authority:
- [docs/gui/M7_IMPORTED_BOARD_FIDELITY_FIXTURES.md](/home/bfadmin/Documents/datum-eda/docs/gui/M7_IMPORTED_BOARD_FIDELITY_FIXTURES.md)

At minimum:
- one cache-present board (`DOA2526`)
- one cache-absent board (`datum-test`)

Required proof obligations:
- both fixtures materialize imported text through the same Datum-owned
  Newstroke-equivalent generation path
- visible text semantics remain equivalent across cache-present and
  cache-absent inputs
- if visual quality still differs materially, the ticket is not done
- if a fixture previously relied on KiCad-cached TrueType outlines, any
  resulting visual change under Phase 1 is explained explicitly in the ticket
  closeout and not misclassified as a surprise regression

## Suggested Validation Surface

Add or update tests for:
- cache-present vs cache-absent fixtures landing on the same normalization path
- justify / anchor correctness
- rotation / keep-upright correctness
- multiline imported text when present in accepted fixtures
- Newstroke-equivalent parity checks against KiCad-rendered snapshots or other
  accepted non-cache references
- bounded `render_cache` comparison only where the imported source was
  TrueType-authored and the cache is being used as a local investigation aid

## Deliverable Summary

Expected patch shape:
- one explicit imported-text normalization rule
- one Datum-owned Newstroke-equivalent generator for imported KiCad text
- removal of `render_cache` as final imported-text render truth
- focused fixture-backed tests
- no KiCad runtime dependency
- no hidden dependency on KiCad TrueType outline caches for visual credibility

## Relationship To Existing Tickets

This ticket sharpens and extends:
- `M7-IMP-005` broader silent-fallback audit
- `M7-IMP-013` orientation/keep-upright correctness

`M7-IMP-013` is not the final closeout for imported text fidelity. It closed a
meaning-level rotation bug. `M7-IMP-014` is the stronger architectural follow-on
that removes representation-dependent glyph-source quality from the imported
board path.

Future follow-on:
- imported TrueType / `OUTLINE_FONT` support is a legitimate later slice, but
  it does not gate Phase 1 completion of `M7-IMP-014`
