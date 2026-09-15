# M7 Opening Charter

This file records the opening boundary for `M7`. Progress-state updates belong
in `specs/PROGRESS.md`; this shard pins the opening scope, guardrails, and the
first accepted slice.

> **Product-identity note**: This is historical/frontend milestone scope. It is
> subordinate to
> `docs/decisions/PRODUCT_MECHANICS_016_PRODUCT_NORTH_STAR.md`. M7 imported-board
> review does not define Datum's roadmap center; native governed library and
> schematic capture do.

Concrete opening workspace, first frontend-consumed scene contract, and the
opening architecture spike plan now live in `specs/M7_FRONTEND_SPEC.md`.
Imported-board correction planning for the post-spike fidelity track now lives
in `docs/gui/M7_IMPORTED_BOARD_FIDELITY_PLAN.md`.
Delivery/testability gating for the opening slice now lives in
`docs/gui/M7_DELIVERY_GATES.md`.

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
- interim adoption of Taffy as the frontend shell/panel layout solver behind
  the retained `wgpu` renderer, governed by
  `docs/decisions/PRODUCT_MECHANICS_014_UI_LAYOUT_SYSTEM.md`

Specified post-spike correction track inside opening `M7`:
- one bounded imported-board fidelity program defined in
  `docs/gui/M7_IMPORTED_BOARD_FIDELITY_PLAN.md`
- keep imported KiCad PCB review inside the opening `M7` milestone, but treat
  credibility of imported board review as a separate acceptance gate from
  crate-boundary / shell / interaction proof
- execute this work after the opening architecture spike proves the
  `gui-app` / `gui-protocol` / `gui-render` boundary, and before broadening
  imported-board review claims or adding broader GUI workflows
- split the correction work into import-fidelity, scene-contract, and
  renderer-semantics passes rather than letting ad hoc renderer polish hide
  engine/import truth problems

The opening `M7` slice should explicitly avoid:
- new routing candidate families or selector semantics
- new M6 objectives, profiles, or intent interpretation
- placement editing, placement autorouting, or push-and-shove behavior
- broad schematic editing, 3D review, or general GUI-platform sprawl
- using GUI needs to back-drive engine semantics before a review surface is
  proven
- terminal or AI lanes that invent parallel design truth or bypass engine
  authority

## Delivery Rule

Opening `M7` work may not advance on a "low resolution but technically
implemented" basis.

For this milestone, a user-facing slice is only allowed to count as complete
enough to move on when:
- the tester can intentionally trigger it in the current shell
- the tester can observe and understand the resulting state
- the minimum substrate needed for the claimed behavior is already in place

For opening `M7`, that substrate explicitly includes:
- selection ownership
- hit-testing ownership
- focus / relatedness behavior
- layer / visibility semantics
- render-state consistency

If one of those pillars is missing, the required repair work is not scope
creep.
It is prerequisite completion for the slice already being claimed.

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

## Standards Amendment

Opening `M7` is now also constrained by the standards/compliance posture in
`specs/STANDARDS_COMPLIANCE_SPEC.md`.

This does **not** promote opening `M7` into a full IPC footprint-authoring or
general standards-validation milestone.

It does mean:
- `M7` may not silently replace imported manufacturability-relevant geometry
  with a host-EDA default or an inferred standards result
- `M7` must preserve standards-relevant imported observables faithfully where
  the review surface already exposes them
- `M7` may add bounded import-audit diagnostics for recognized standards-aware
  observables when the rule basis is explicit enough to support review findings

For opening `M7`, the immediate standards-facing observables are:
- copper pad geometry
- drill and annular ring
- solder-mask aperture policy
- paste-aperture policy
- thermal-pad / thermal-via review truth where present
- courtyard and clearance semantics only when and where the review surface
  already exposes them explicitly

What remains out of scope for opening `M7`:
- full IPC footprint wizard / generator
- full-library standards enforcement
- broad compliance claims
- automatic geometry healing toward an inferred IPC result

The standards-aware addition to opening `M7` should therefore stay bounded to:
- import truth preservation
- review-surface fidelity for standards-relevant observables
- structured import-audit diagnostics, where implemented, that report delta
  without mutating imported source geometry
