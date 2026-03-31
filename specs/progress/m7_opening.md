# M7 Opening Charter

This file records the opening boundary for `M7`. Progress-state updates belong
in `specs/PROGRESS.md`; this shard pins the opening scope, guardrails, and the
first accepted slice.

Concrete opening workspace, first frontend-consumed scene contract, and the
opening architecture spike plan now live in `specs/M7_FRONTEND_SPEC.md`.

## Recommended Opening

Open `M7` as the first visual review milestone, not as a general-purpose GUI
or a placement/routing semantic expansion milestone.

Recommended first `M7` slice:
- one read-only route-proposal review workspace for the completed
  routing-kernel substrate
- consume the existing M5 route proposal, explain, artifact, and selector
  outputs without changing their semantics
- reuse the frozen M6 report/compare/delta evidence vocabulary when strategy
  context is shown
- support review of one selected route proposal or one saved route-proposal
  artifact, with explicit accept/reject handoff remaining in existing machine
  surfaces
- use one bounded authored-board scene contract plus the existing
  `review_route_proposal` contract rather than inventing a parallel review
  model
- acknowledge the integrated terminal lane and AI assistant lane as
  first-class supporting workspace lanes without broadening them into
  independent milestones

Implemented opening slice:
- `project review-route-proposal <dir> --net <uuid> --from-anchor <pad_uuid> --to-anchor <pad_uuid> [--profile <profile>]`
- `project review-route-proposal --artifact <path>`
- MCP parity as `review_route_proposal`
- one deterministic review payload for frontend consumption, reusing existing
  proposal actions and route-proposal artifact segment evidence instead of
  inventing a parallel review model

Specified next accepted frontend slice:
- one route-proposal review workspace defined in `specs/M7_FRONTEND_SPEC.md`
- one deterministic `board_review_scene_v1` contract for authored board
  context plus bounded review/render companions
- one locked viewport-centered three-column shell with bottom dock strip
- one fixed `M7` panel set plus integrated terminal and AI supporting lanes,
  all remaining read-only
- one single-selection model with a separate active review target
- explicit authored/proposed/diagnostic visual-state separation

The opening `M7` slice should explicitly avoid:
- new routing candidate families or selector semantics
- new M6 objectives, profiles, or intent interpretation
- placement editing, placement autorouting, or push-and-shove behavior
- broad schematic editing, 3D review, or general GUI-platform sprawl
- using GUI needs to back-drive engine semantics before a review surface is
  proven
- terminal or AI lanes that invent parallel design truth or bypass engine
  authority

## Entry Conditions

Before coding the first `M7` slice:
1. `M5` is treated as closed for routing-kernel scope in `specs/PROGRESS.md`.
2. `M6` is treated as frozen pending evidence from the current baseline gate.
3. Daemon/runtime panic risk is reduced enough that GUI consumers are not
   exposed to obvious unwrap/expect crashes on ordinary query/review paths.
4. MCP registration/request plumbing is aligned enough that adding a bounded
   visual-review tool does not require fragile catalog/dispatch/runtime
   hand-sync.

## Success Criteria For The First Slice

The first `M7` slice should satisfy all of:
- visual review of one existing route proposal or proposal artifact
- no change to CLI/MCP routing semantics or deterministic selector behavior
- machine-native surfaces remain the authority for apply/export/inspection
- focused proof that the GUI/review layer is a consumer of the engine, not a
  semantic fork of it
- focused proof that canvas, terminal, and AI lanes can share one explicit
  review context without creating competing truth models
- focused proof that the locked shell, panel, selection, and visual-state
  decisions are viable for the opening slice
