# M7-IMP-010 Implementation Brief

> **Ticket**: `M7-IMP-010`
> **Stage**: Stage 1 follow-on (directly observed fidelity bug on canonical fixture)
> **Track**: Imported board fidelity inside opening `M7`
> **Status**: Ready for implementation
> **Product Decision**: Option A chosen (parser-only fix, no IR schema change)

## Purpose

Define the bounded implementation contract for removing the silent pad-layer
misclassification shortcut in imported KiCad PCB boards, and for recognizing
the real KiCad pad-layer encodings explicitly.

This brief exists so the coding pass does not:
- widen the board IR schema under the cover of a fidelity bugfix
- preserve the current silent fallback to the footprint's placement layer
- invent pad-layer semantics that are not present in the source `.kicad_pcb`
- touch downstream DRC, routing, rendering, or export surfaces

## Problem

The current pad-layer extractor only recognizes the literal names `"F.Cu"` and
`"B.Cu"` in a pad's `(layers ...)` list. For any other encoding — including
the KiCad-standard through-hole wildcard `"*.Cu"`, the `"F&B.Cu"` variant, or
inner-copper names — it returns `None`. The call site then silently falls back
to the footprint's placement layer.

Current evidence:
- [skeleton.rs parse_pad_copper_layer_anywhere](/home/bfadmin/Documents/datum-eda/crates/engine/src/import/kicad/skeleton.rs:835)
- [skeleton.rs pad construction call site](/home/bfadmin/Documents/datum-eda/crates/engine/src/import/kicad/skeleton.rs:673)

Observed behavior on canonical `datum-test`:
- through-hole Mill-Max pins use `(layers "*.Cu" ...)`
- the extractor returns `None`, import silently assigns them the footprint's
  placement layer (typically F.Cu), so the pad layer identity in the IR does
  not reflect that the pad spans all copper layers
- the GUI review surface cannot truthfully distinguish single-layer SMD pads
  from multi-layer through-hole pads from their IR layer assignment

This violates the plan rule:
- supported cases must preserve board truth
- unsupported cases must fail explicitly, not silently misclassify

## Scope

This ticket covers:
- imported KiCad pad `(layers ...)` parsing
- the shortcut helper that resolves a pad's primary copper layer
- the call site that currently falls back to the package layer
- fixture-backed proof on the frozen Stage 0 truth set

This ticket does **not** cover:
- changing `PlacedPad` schema (no new `layers: Vec<LayerId>` field)
- DRC, routing, or rendering changes that consume pad-layer semantics
- export paths (Gerber, drill, PnP) that assume single-layer pads
- pad rotation (`M7-IMP-008`) or roundrect (`M7-IMP-009`)
- broader pad-layer membership expressiveness in the IR

The explicit non-goal is important: full pad-layer membership semantics in
`PlacedPad` would require a schema change with broad downstream ripple, and
that belongs to a separate IR-expansion ticket if DRC / routing / rendering
later need true layer-set semantics.

## Required Change

Replace the narrow literal-name recognition with an explicit bounded parser
over the accepted KiCad pad-layer encodings.

### Accepted Encoding Set

For this ticket, the parser must recognize:
- `"F.Cu"` — SMD on the top copper layer, primary layer = F.Cu
- `"B.Cu"` — SMD on the bottom copper layer, primary layer = B.Cu
- `"*.Cu"` — all copper layers (through-hole), primary layer canonicalized to
  F.Cu
- `"F&B.Cu"` — front and back copper (through-hole variant), primary layer
  canonicalized to F.Cu
- explicit inner-copper names present in the PCB's own `(layers ...)` table
  (e.g. `"In1.Cu"`, `"In2.Cu"`): primary layer = that inner layer
- any layer name that resolves via `resolve_layer_id` against the parsed
  layer table

If the pad's `(layers ...)` list contains multiple copper entries (e.g.
`"F.Cu" "In1.Cu" "B.Cu"`), the primary layer is the most-top-side recognized
copper layer; the multi-layer nature is signalled implicitly by the pad's
`drill > 0` (see limitation note below).

### Primary-Layer Rule

- SMD pads: primary layer is the actual copper layer listed.
- Through-hole / all-copper pads (`*.Cu`, `F&B.Cu`, or any copper-list with
  `drill > 0`): primary layer canonicalized to F.Cu.
- Inner-layer-only pads (only inner-copper names present, no outer-copper
  reference): primary layer is that explicit inner layer.

### Silent-Fallback Rule

The call site must no longer resolve a missing/unparsed pad layer to the
package (footprint placement) layer. Unknown or unresolvable pad-layer
encoding must return an explicit `EngineError::Import` naming the offending
encoding and the owning pad.

### Multilayer Signal

For this bounded fix, the multi-layer nature of through-hole pads is carried
by the pad's `drill` field (non-zero drill + accepted wildcard handling) and
the primary-layer canonicalization to F.Cu. Full explicit pad-layer
membership in the IR is deferred (see limitation note).

## Minimum Code Surface To Audit

The implementation pass must inspect at least:
- `crates/engine/src/import/kicad/skeleton.rs` — `parse_pad_copper_layer_anywhere`
  (the shortcut) and its call site in pad construction
- `crates/engine/src/import/kicad/parser_helpers.rs` — `resolve_layer_id` (the
  existing canonical layer-name resolver from `M7-IMP-001`)

Each path must be confirmed to either:
- produce the correct primary layer for a supported pad encoding
- surface an explicit import error for unsupported encoding
- never fall back to the footprint's placement layer silently

## Supported Behavior Rule

After this ticket lands:
- pads whose `(layers ...)` list matches the accepted encoding set import with
  the correct primary copper layer
- through-hole pads are no longer misclassified as single-layer package-layer
  pads
- unrecognized or ambiguous pad-layer encodings fail import with an
  actionable diagnostic

## Fixture Proof

Implementation proof must use the frozen Stage 0 fixture authority:
- [docs/gui/M7_IMPORTED_BOARD_FIDELITY_FIXTURES.md](/home/bfadmin/Documents/datum-eda/docs/gui/M7_IMPORTED_BOARD_FIDELITY_FIXTURES.md)

Minimum proof obligations:
- canonical `m7-datum-test-half-routed` through-hole Mill-Max pads (using
  `*.Cu`) import with primary layer = F.Cu and `drill > 0`
- SMD pads on `datum-test` import with their actual copper layer as primary
- `simple-demo-board` continues to import correctly for existing SMD pads
- at least one repo-native fixture exercises a non-trivial pad-layer encoding
  if available in the frozen set

If the frozen fixture set does not contain a case the implementation needs
(e.g. inner-layer-only pad, `F&B.Cu` encoding), the implementing engineer
must:
- stop before closing the ticket
- ask the user to provide a real exported `.kicad_pcb` containing the needed
  case
- not fabricate synthetic KiCad text fixtures to fill the gap

## Acceptance Criteria

`M7-IMP-010` is complete only when all of the following are true:
- `F.Cu`, `B.Cu`, `*.Cu`, `F&B.Cu`, and supported inner-copper names all
  resolve to the correct primary layer
- the call-site `unwrap_or(package_layer)` silent-fallback is removed
- unsupported pad-layer encodings fail import with an explicit
  `EngineError::Import` that names the offending encoding
- canonical `datum-test` through-hole pads import with primary layer = F.Cu
  and the multi-layer nature is represented by `drill > 0`
- existing KiCad import tests still pass
- added tests use real fixtures or pure helper unit tests only — no synthetic
  KiCad text fixtures

## Suggested Test Additions

Allowed additions:
- pure helper unit tests over the pad-layer encoding recognizer on
  `Vec<String>` inputs representing the `(layers ...)` token set
- real-fixture integration tests against `datum-test` verifying that the
  Mill-Max through-hole pads resolve to F.Cu primary with `drill > 0`
- real-fixture integration tests against `simple-demo-board` verifying
  continued SMD pad layer assignment

Not allowed:
- synthetic `.kicad_pcb` strings in test bodies

## IR Limitation Note

This ticket intentionally does not add explicit multi-layer membership to
`PlacedPad`. The bounded canonical-primary-layer rule plus `drill > 0` is the
multilayer signal for now.

If any future consumer (DRC, routing, rendering, export) needs to know the
full set of copper layers a pad spans, the correct response is:
- open a separate IR-expansion ticket
- land the `PlacedPad.layers: Vec<LayerId>` (or equivalent) change with the
  downstream consumer that needs it

Do NOT retroactively expand this ticket's scope to add such a field.

## Deliverable Summary

Expected patch shape:
- pad-layer encoding recognizer in the KiCad importer
- call-site change removing the silent package-layer fallback
- focused real-fixture and helper tests
- no unrelated import-scope broadening
- no schema change to `PlacedPad`

## Follow-On Relationship

This ticket should land after:
- `M7-IMP-001` (layer identity without silent fallback) — required because
  the pad-layer extractor depends on `resolve_layer_id` for inner-copper name
  resolution

This ticket does not remove the need for:
- `M7-IMP-005` broader silent-fallback audit
- `M7-IMP-008` pad rotation semantics
- `M7-IMP-009` roundrect corner semantics
- future IR-expansion work if explicit multi-layer pad membership is required
