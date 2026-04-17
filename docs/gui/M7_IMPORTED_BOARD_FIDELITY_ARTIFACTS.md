# M7 Imported Board Fidelity Reference Artifacts

> **Status**: Active reference-artifact definition for `M7-FIX-002`.
> This document defines the required proof artifacts for each imported-board
> fidelity fixture.

## Purpose

Freeze what counts as acceptable proof for imported-board fidelity work before
Stage 1 implementation begins.

This document exists to ensure that each accepted fixture has:
- a stable source path or source identity
- a stable Datum launch path
- a stable screenshot naming convention
- enough artifact structure that fixes can be compared without re-debating the
  evidence format

## Artifact Types

Each fixture should eventually have the following artifact set.

### A. Source Artifact

Required:
- the source `.kicad_pcb` file path or the stable external fixture identity if
  the board is not yet vendored

Purpose:
- preserve the KiCad-side truth being compared

### B. Datum Launch Artifact

Required:
- one of:
  - a stable `--board <path>` launch path
  - a stable project-root launch path
  - a checked `board_review_scene_v1` payload if the review is fixture-driven

Purpose:
- guarantee the team is loading the same Datum-side review surface

### C. KiCad Reference Screenshot

Required when useful:
- one checked or archived screenshot showing the KiCad-side semantic truth

Minimum naming convention:
- `<fixture-id>.kicad.reference.png`

Purpose:
- preserve expected authored copper / airwire / footprint readability

### D. Datum Baseline Screenshot

Required:
- one Datum screenshot target for the fixture once the track stabilizes

Minimum naming convention:
- `<fixture-id>.datum.review.png`
- for selected-state checks:
  - `<fixture-id>.datum.review.selected.png`

Purpose:
- make visual regressions reviewable and discussable

### E. Notes Artifact

Required:
- short notes on what the fixture is expected to prove

Minimum naming convention:
- `<fixture-id>.notes.md` or an equivalent section in this document

Purpose:
- prevent screenshot-only interpretation drift

## Per-Fixture Artifact Matrix

| Fixture ID | Source artifact | Datum launch artifact | KiCad reference screenshot | Datum baseline screenshot | Notes |
|-----------|------------------|-----------------------|----------------------------|---------------------------|-------|
| `m7-datum-test-half-routed` | local external KiCad board (`datum-test`) | stable `--board` launch path on the local board | required | required | required |
| `airwire-demo-board` | [airwire-demo.kicad_pcb](/home/bfadmin/Documents/datum-eda/crates/engine/testdata/import/kicad/airwire-demo.kicad_pcb) | stable imported-board launch path | optional | required once Stage 3 starts | required |
| `partial-route-demo-board` | [partial-route-demo.kicad_pcb](/home/bfadmin/Documents/datum-eda/crates/engine/testdata/import/kicad/partial-route-demo.kicad_pcb) | stable imported-board launch path | optional | required once Stage 3 starts | required |
| `drc-coverage-demo-board` | [drc-coverage-demo.kicad_pcb](/home/bfadmin/Documents/datum-eda/crates/engine/testdata/import/kicad/drc-coverage-demo.kicad_pcb) | stable imported-board launch path | optional | optional until directly used as proof | required |
| `simple-demo-board` | [simple-demo.kicad_pcb](/home/bfadmin/Documents/datum-eda/crates/engine/testdata/import/kicad/simple-demo.kicad_pcb) | stable imported-board launch path | optional | optional until directly used as proof | required |

## Notes By Fixture

### `m7-datum-test-half-routed`

Must prove:
- authored copper vs unrouted KiCad airwires are both present in the source
  truth
- Datum can eventually distinguish authored copper, remaining unrouted state,
  and proposal overlay geometry clearly
- footprint/pad fidelity concerns remain visible on a realistic board

### `airwire-demo-board`

Must prove:
- imported unrouted extraction remains deterministic
- repo-native fallback evidence exists for Stage 3 even before screenshot
  baselines are complete

### `partial-route-demo-board`

Must prove:
- imported partially routed state can be reviewed without depending only on the
  local canonical board

### `drc-coverage-demo-board`

Must prove:
- broader imported board geometry can survive scene extraction without
  collapsing into a toy case

### `simple-demo-board`

Must prove:
- the simplest imported control case remains stable while richer fixtures are
  being fixed

## Completion Rule

`M7-FIX-002` is satisfied when:
- this document is the accepted artifact authority for Stage 0
- every fixture in
  `docs/gui/M7_IMPORTED_BOARD_FIDELITY_FIXTURES.md`
  has a defined source artifact and Datum launch artifact
- the canonical local fixture and the repo-native fallback fixtures all have
  explicit screenshot/notes expectations

Current read:
- satisfied for the current accepted fixture set
- caveat retained: the canonical `m7-datum-test-half-routed` source artifact
  is still an accepted local external board rather than a vendored repo asset
