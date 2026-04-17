# M7-IMP-001 Implementation Brief

> **Ticket**: `M7-IMP-001`
> **Stage**: Stage 1
> **Track**: Imported board fidelity inside opening `M7`
> **Status**: Ready for implementation

## Purpose

Define the bounded implementation contract for removing silent KiCad layer
fallback corruption from the imported-board path.

This brief exists so the coding pass does not:
- fix the symptom in one helper while leaving silent fallback at call sites
- prove the fix on one board only
- accidentally widen `M7` scope into unrelated import work

## Problem

Unknown KiCad layer names currently collapse silently to layer `0` / `F.Cu`.

Current evidence:
- [parser_helpers.rs](/home/bfadmin/Documents/datum-eda/crates/engine/src/import/kicad/parser_helpers.rs:687)

Current behavior:
- `kicad_layer_name_to_id()` hard-falls through to `_ => 0`
- import continues without surfacing an error or explicit bounded result
- imported board meaning can be materially corrupted while still looking
  superficially valid

This is unacceptable for Stage 1 because it violates the plan rule:
- supported cases must preserve board truth
- unsupported cases must fail explicitly or remain clearly bounded

## Scope

This ticket covers:
- imported KiCad PCB layer identity resolution
- the helper/API surface that currently allows silent fallback
- every call site that currently assumes a layer ID always exists
- fixture-backed proof on the frozen Stage 0 truth set

This ticket does **not** cover:
- pad rotation
- roundrect ratio semantics
- outline ownership policy
- renderer appearance
- scene-contract expansion

## Required Change

Replace the silent fallback contract with an explicit bounded contract.

Acceptable implementation directions:
1. return `Result<LayerId, EngineError>` from the relevant resolution path
2. return `Option<LayerId>` and force every caller to handle absence explicitly

Not acceptable:
- keeping `_ => 0`
- replacing it with a different silent fallback
- logging-only behavior while continuing import with guessed copper identity

## Minimum Code Surface To Audit

The implementation pass must inspect at least:
- `crates/engine/src/import/kicad/parser_helpers.rs`
- `crates/engine/src/import/kicad/skeleton.rs`

Known current call paths using resolved layer IDs include:
- footprint placement layer
- track layer
- via start/end layers
- zone layer(s)
- board text layer

The pass must confirm that each path now does one of:
- succeeds with a supported resolved layer
- fails explicitly with a bounded import error
- remains explicitly omitted by documented supported behavior

## Supported Behavior Rule

After this ticket lands:
- supported KiCad boards with recognized layer identity must import with the
  correct layer IDs
- unsupported or unresolved layer-table cases must not silently produce
  `F.Cu` geometry

If a board cannot be imported faithfully because layer identity is unresolved,
the importer should stop with an explicit, actionable error.

## Fixture Proof

Implementation proof must use the frozen Stage 0 fixture authority:
- [docs/gui/M7_IMPORTED_BOARD_FIDELITY_FIXTURES.md](/home/bfadmin/Documents/datum-eda/docs/gui/M7_IMPORTED_BOARD_FIDELITY_FIXTURES.md)

Minimum proof obligations:
- canonical `m7-datum-test-half-routed` board still imports under the accepted
  supported subset or fails explicitly for a different bounded reason
- `simple-demo-board` still imports correctly
- at least one supporting board from the manifest is exercised to prove the
  fix is not datum-test-only

If the current frozen set proves insufficient for a realistic multilayer proof,
the implementing engineer must:
- stop before closing the ticket
- nominate the additional fixture in the manifest
- then continue using the updated accepted set

## Acceptance Criteria

`M7-IMP-001` is complete only when all of the following are true:
- no unknown KiCad layer name silently maps to `F.Cu`
- the importer has an explicit bounded response to unresolved layer identity
- all affected call sites have been audited and updated
- the accepted fixture set provides proof that supported boards still import
  correctly
- tests cover at least one nontrivial layer-table case beyond a trivial control
  board

## Suggested Test Additions

The implementation should add or update tests for:
- non-tab-indented KiCad layer tables
- inner-layer or non-default copper-layer naming where supported
- explicit failure on unknown/unresolved layer names

## Deliverable Summary

Expected patch shape:
- helper/API contract change for layer resolution
- call-site updates in board import
- focused tests
- no unrelated import-scope broadening

## Follow-On Relationship

This ticket should land before:
- `M7-IMP-003`
- any claim that Stage 1 import truth is restored

This ticket does not remove the need for:
- `M7-IMP-003` outline ownership enforcement
- `M7-IMP-005` broader silent-fallback audit
