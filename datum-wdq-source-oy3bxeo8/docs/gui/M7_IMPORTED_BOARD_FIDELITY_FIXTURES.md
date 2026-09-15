# M7 Imported Board Fidelity Fixture Manifest

> **Status**: Historical -- `M7-FIX-001` fixture manifest, M7 spike closed-for-scope; retained as historical evidence.
> This document freezes the imported-board fidelity fixture set used by the
> opening `M7` post-spike correction track.

## Purpose

Define the specific boards the team must use when evaluating imported KiCad PCB
board-review fidelity.

This manifest exists to prevent:
- proving fixes against one convenient board only
- changing the acceptance surface mid-fix
- mixing repo-native fixtures and local boards without explicit tracking

This manifest is the fixture authority for:
- `docs/gui/M7_IMPORTED_BOARD_FIDELITY_PLAN.md`
- `docs/gui/M7_IMPORTED_BOARD_FIDELITY_CHECKLIST.md`
- `docs/gui/M7_IMPORTED_BOARD_FIDELITY_ISSUES.md`

## Fixture Set

The imported-board fidelity track uses the following fixture classes.

### F1. Canonical Half-Routed Review Board

ID:
- `m7-datum-test-half-routed`

Role:
- canonical semantic-readability fixture
- proves authored copper vs unrouted connectivity vs proposal overlay
- carries the strongest current evidence for import, scene, and renderer
  discussion

Source of truth:
- local external KiCad board currently referred to as `datum-test`

Required properties:
- intentionally half-routed
- multiple remaining airwires in KiCad
- representative package and pad diversity for route-review work

Current status:
- external/local fixture, not yet checked into the repo

Tracking rule:
- until vendored, this fixture remains part of the accepted truth set and must
  be referenced by stable ID in issue discussion and screenshot review

### F2. Repo-Native Unrouted Board

ID:
- `airwire-demo-board`

Source:
- [crates/engine/testdata/import/kicad/airwire-demo.kicad_pcb](/home/bfadmin/Documents/datum-eda/crates/engine/testdata/import/kicad/airwire-demo.kicad_pcb)

Role:
- repo-native unrouted / airwire baseline
- engine/query-side proof for unrouted extraction and deterministic review data

Why included:
- already present in the repo
- already used by import/query determinism and query goldens

### F3. Repo-Native Partial Route Board

ID:
- `partial-route-demo-board`

Source:
- [crates/engine/testdata/import/kicad/partial-route-demo.kicad_pcb](/home/bfadmin/Documents/datum-eda/crates/engine/testdata/import/kicad/partial-route-demo.kicad_pcb)

Role:
- repo-native partially routed board for imported board-review checks
- fallback/companion semantic-readability fixture until the canonical
  `datum-test` fixture is vendored

Why included:
- already present in the repo
- already recognized in broader quality evidence

### F4. Repo-Native Layer / Coverage Board

ID:
- `drc-coverage-demo-board`

Source:
- [crates/engine/testdata/import/kicad/drc-coverage-demo.kicad_pcb](/home/bfadmin/Documents/datum-eda/crates/engine/testdata/import/kicad/drc-coverage-demo.kicad_pcb)

Role:
- broader board-feature fixture
- supporting board for layer, zones, and general imported-board review checks

### F5. Repo-Native Simple Board

ID:
- `simple-demo-board`

Source:
- [crates/engine/testdata/import/kicad/simple-demo.kicad_pcb](/home/bfadmin/Documents/datum-eda/crates/engine/testdata/import/kicad/simple-demo.kicad_pcb)

Role:
- control fixture
- sanity baseline for deterministic import and board scene extraction

## Required Coverage By Fixture Class

The track must cover the following acceptance concerns.

| Concern | Primary fixture | Supporting fixtures |
|---------|-----------------|---------------------|
| authored vs unrouted semantic separation | `m7-datum-test-half-routed` | `partial-route-demo-board`, `airwire-demo-board` |
| imported airwire / unrouted extraction | `airwire-demo-board` | `m7-datum-test-half-routed` |
| layer identity preservation | fixture to be added or nominated during Stage 1 if current repo-native set proves insufficient | `drc-coverage-demo-board`, `simple-demo-board` |
| pad / footprint fidelity | `m7-datum-test-half-routed` | fixture to be added or nominated during Stage 2 if current repo-native set proves insufficient |
| outline ownership behavior | `m7-datum-test-half-routed` for the current footprint-embedded trick case | repo-native top-level-outline fixtures |

## Freeze Rules

- Do not remove a fixture from this manifest while a ticket depending on it is
  still open.
- Do not replace the canonical `m7-datum-test-half-routed` fixture with a
  repo-native board unless the architect explicitly re-accepts the semantic
  review target.
- Adding a new fixture is allowed only when:
  - a currently accepted gap is not exercised by the existing set
  - the new fixture is named here before it is used as proof of a fix

## Current Gaps In The Manifest

The following fixture needs remain open but do not block Stage 0 freeze:
- one explicitly nominated multilayer board if the current accepted set proves
  insufficient for `M7-IMP-001`
- one explicitly nominated pad-shape stress board if the current accepted set
  proves insufficient for `M7-IMP-008` / `M7-IMP-009`

Rule:
- those additions must be recorded here before the corresponding tickets are
  considered closed

## Stage 0 Completion Read

`M7-FIX-001` is satisfied when:
- this manifest is the agreed fixture authority for the imported-board
  fidelity track
- the team is using stable fixture IDs in issue discussion
- the canonical local `datum-test` board and the current repo-native fallback
  boards are all explicitly named here

Current read:
- satisfied
- caveat retained: `m7-datum-test-half-routed` remains an accepted
  external/local fixture until it is vendored
