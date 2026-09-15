# M7-IMP-003 Implementation Brief

> **Ticket**: `M7-IMP-003`
> **Stage**: Stage 1
> **Track**: Imported board fidelity inside opening `M7`
> **Status**: Ready for implementation
> **Product Decision**: Option A chosen
>
> Authority for the decision:
> [docs/gui/M7_IMP_003_OUTLINE_OWNERSHIP_DECISION.md](/home/bfadmin/Documents/datum-eda/docs/gui/M7_IMP_003_OUTLINE_OWNERSHIP_DECISION.md)

## Purpose

Define the bounded implementation contract for making imported KiCad PCB
board-outline extraction explicit, complete, and free of the current silent
placeholder fallback.

This brief exists so the coding pass does not:
- broaden outline ownership beyond the accepted Option A subset
- guess outline geometry from arbitrary footprint graphics
- preserve the silent `10mm x 10mm` placeholder box as an acceptable result
- mix this ticket with scene-contract or renderer work

## Problem

The current importer derives board outline only from top-level `gr_line` and
`gr_arc` blocks whose `(layer ...)` is `Edge.Cuts`.

Current evidence:
- [parser_helpers.rs](/home/bfadmin/Documents/datum-eda/crates/engine/src/import/kicad/parser_helpers.rs:466)
- [skeleton.rs](/home/bfadmin/Documents/datum-eda/crates/engine/src/import/kicad/skeleton.rs:618)

Observed behavior:
- `outline_from_edge_cuts()` iterates only top-level `gr_line` and `gr_arc`
  blocks
- the canonical `m7-datum-test-half-routed` fixture encodes its outline as
  `fp_line` / `fp_arc` on `Edge.Cuts` inside a footprint; these are silently
  ignored
- when nothing is found, skeleton construction falls back to a default
  `10mm x 10mm` placeholder box without error
- review downstream silently operates on an outline that does not match the
  imported design

This is unacceptable for Stage 1 because it violates the plan rules:
- supported cases must preserve board truth
- unsupported cases must fail explicitly or remain clearly bounded
- imported-board review must not silently operate on placeholder geometry

## Scope

This ticket covers:
- imported KiCad PCB board-outline extraction semantics
- the helper/API surface that currently composes the outline from Edge.Cuts
  geometry
- the call site that currently silently replaces missing outline with the
  default placeholder
- fixture-backed proof on the frozen Stage 0 truth set

This ticket does **not** cover:
- silkscreen / courtyard / fab / assembly geometry extraction
- zone outline semantics
- footprint-embedded `fp_poly`, `fp_rect`, or `fp_circle` contributors (those
  may follow in a later ticket if fixture evidence requires it)
- any change to the scene contract, renderer appearance, or GUI behavior

## Required Change

Extend board-outline extraction to explicitly support footprint-embedded
`Edge.Cuts` geometry under Option A, and remove the silent placeholder
fallback.

### Accepted Primitive Set

For this ticket:
- top-level `gr_line` on `Edge.Cuts` — authoritative
- top-level `gr_arc` on `Edge.Cuts` — authoritative
- footprint-embedded `fp_line` on `Edge.Cuts` — valid contributor under
  footprint transform
- footprint-embedded `fp_arc` on `Edge.Cuts` — valid contributor under
  footprint transform

All other forms on `Edge.Cuts` remain out-of-subset for this ticket.

### Transform Rule

For each footprint-embedded contributor:
- start from the footprint's local coordinate system
- apply the footprint `(at x y [rot])` placement: rotate the local points
  about the local origin by the footprint rotation, then translate by the
  footprint position
- emit the resulting world-space points into the outline pipeline

### Composition Rule

The imported outline is the union of:
- all accepted top-level contributors
- all accepted per-footprint contributors after their transforms are applied

If the resulting contributor set assembles into a closed polygon via the
existing assembly logic, that polygon is the imported outline.

### Failure Rule

If no accepted contributor is present, or the contributor set cannot be
assembled into a closed outline under the existing rules, import must fail
explicitly with an actionable diagnostic.

Not acceptable:
- returning the default `10mm x 10mm` placeholder box silently
- succeeding with a partial or arbitrary outline
- promoting non-`Edge.Cuts` footprint graphics into the outline

## Minimum Code Surface To Audit

The implementation pass must inspect at least:
- `crates/engine/src/import/kicad/parser_helpers.rs` — `outline_from_edge_cuts`
  and its helpers
- `crates/engine/src/import/kicad/skeleton.rs` — the site that currently
  `.unwrap_or_else(default_outline)` the extracted outline
- the footprint iteration already present in skeleton for package extraction,
  so the transform context is obtained consistently

Known current call paths producing or consuming board outline:
- `outline_from_edge_cuts(contents) -> Option<Polygon>`
- the `Board { outline, .. }` field populated in the skeleton
- any consumer of `Board.outline` that assumes a non-placeholder polygon

Each path must be confirmed to either:
- produce a supported imported outline
- surface an explicit import error
- remain explicitly omitted under the documented supported subset

## Supported Behavior Rule

After this ticket lands:
- boards whose outline is expressible under Option A must import with the
  correct real-world outline geometry
- boards with no accepted outline contributors must fail import explicitly
- the default `10mm x 10mm` placeholder must no longer be a silent success
  surface

## Fixture Proof

Implementation proof must use the frozen Stage 0 fixture authority:
- [docs/gui/M7_IMPORTED_BOARD_FIDELITY_FIXTURES.md](/home/bfadmin/Documents/datum-eda/docs/gui/M7_IMPORTED_BOARD_FIDELITY_FIXTURES.md)

Minimum proof obligations:
- canonical `m7-datum-test-half-routed` imports with its authored outline
  (4 straight edges + 4 fillet arcs forming a rounded rectangle), assembled
  from footprint-embedded `fp_line` and `fp_arc` contributors under the
  footprint transform
- `simple-demo-board` continues to import correctly (top-level contributors
  only)
- at least one additional repo-native fixture exercises either a non-trivial
  footprint transform or a composition of top-level and footprint-embedded
  contributors

If the frozen fixture set is insufficient to prove the rule comprehensively,
the implementing engineer must:
- stop before closing the ticket
- nominate the additional fixture in the manifest
- then continue using the updated accepted set

## Acceptance Criteria

`M7-IMP-003` is complete only when all of the following are true:
- top-level `gr_line` / `gr_arc` on `Edge.Cuts` continue to contribute
  correctly
- footprint-embedded `fp_line` / `fp_arc` on `Edge.Cuts` contribute correctly
  after applying the footprint `(at x y rot)` transform
- composition of top-level and footprint-embedded contributors produces the
  intended imported outline
- the silent `default_outline` placeholder path is removed; missing outline is
  an explicit bounded import error
- existing outline-related tests still pass
- new tests cover at least the cases listed below

## Suggested Test Additions

- footprint-embedded outline with pure translation (rot == 0) assembles
  correctly
- footprint-embedded outline with non-zero rotation assembles correctly
- composition of top-level and footprint-embedded contributors on one board
- explicit import failure when a board has no accepted outline contributor
- the `m7-datum-test-half-routed` canonical outline is recovered faithfully

## Deliverable Summary

Expected patch shape:
- outline extraction function change (footprint iteration + transform)
- call-site change removing the silent placeholder fallback
- focused fixture-backed tests
- no unrelated import-scope broadening

## Follow-On Relationship

This ticket should land after:
- `M7-IMP-001` (layer identity without silent fallback) — required so the
  outline extractor's layer check itself is no longer ambiguous

This ticket does not remove the need for:
- `M7-IMP-005` broader silent-fallback audit
- `M7-SCN-*` scene-contract work once Stage 2 begins
