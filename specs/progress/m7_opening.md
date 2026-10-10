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

## Tracker Checkpoint History

> Detailed evidence relocated from `specs/PROGRESS.md`. The tracker remains the implementation-status entry point; the generated Active Frontier alone selects work. Dated history below does not grant execution or establish a fresh PASS.

## M7 Opening Charter

Status: [~] Opened narrowly as a read-only route-proposal review layer
- Authority for the opening charter and entry criteria:
  `specs/progress/m7_opening.md`
- Concrete workspace/contract/spike definition:
  `specs/M7_FRONTEND_SPEC.md`
- Active imported-board fidelity execution plan:
  `docs/gui/M7_IMPORTED_BOARD_FIDELITY_PLAN.md`
- Active board-review fidelity diagnosis:
  `docs/gui/M7_BOARD_REVIEW_FIDELITY_GAP.md`
- Active M7 focus: one narrow visual review layer on top of the closed M5
  routing-kernel substrate and the frozen M6 strategy-reporting/evidence
  stack.
- Entry conditions satisfied before opening:
  - daemon result serialization no longer unwrap-panic on response encoding in
    `crates/engine-daemon/src/dispatch.rs`; serialization failures now return
    structured internal JSON-RPC errors instead of crashing the daemon
  - one remaining imported-connectivity runtime unwrap in
    `crates/engine/src/connectivity/mod.rs` was removed from the hierarchical
    link resolver path
  - `mcp-server/server_runtime.py` now generates the plain daemon-backed MCP
    request/call wrappers from one shared `DAEMON_CLIENT_METHOD_SPECS` table
    instead of duplicating those wrappers by hand across request builders and
    call helpers
  - `mcp-server/daemon_client_request_tests.py` now sanity-checks that the
    generated daemon client wrappers are installed and preserve required fixed
    parameter shapes such as `rotate_component`
  - governance/docs no longer treat `M5` as the active execution window; M5 is
    closed and M6 is frozen pending evidence
- Implemented opening slice:
  - new native CLI review surface:
    `project review-route-proposal <dir> --net <uuid> --from-anchor <pad_uuid> --to-anchor <pad_uuid> [--profile <profile>]`
  - the same command also supports saved-artifact review with:
    `project review-route-proposal --artifact <path>`
  - the review payload reuses the existing selected route-proposal and route
    proposal artifact data structures, exposing deterministic proposal actions,
    segment evidence, selected contract metadata, and source identity for one
    frontend-consumable review object
  - the same review surface is exposed through MCP as
    `review_route_proposal`
  - apply/export remain in the existing machine-native CLI/MCP surfaces; M7 is
    review-only in this opening slice

- Next accepted frontend slice for `M7`:
  - one fixed route-proposal review workspace defined in
    `specs/M7_FRONTEND_SPEC.md`
  - one deterministic `board_review_scene_v1` contract for the opening board
    review context
  - one locked viewport-centered three-column shell with bottom dock strip
  - one fixed panel taxonomy: `Project`, `Filters`, `Inspector`, `Review`
  - one single-selection read-only interaction model with a separate active
    review target
  - one fixed panel set plus integrated terminal and AI supporting lanes, all
    remaining subordinate to engine authority
  - one explicit authored/proposed/diagnostic visual-state model
  - one adopted interim frontend layout system: Taffy solves GUI shell/panel
    logical-pixel rectangles behind the retained `wgpu` renderer while engine
    and protocol crates remain layout-solver free

- Accepted post-spike correction track inside opening `M7`:
  - one bounded imported-board fidelity program defined in
    `docs/gui/M7_IMPORTED_BOARD_FIDELITY_PLAN.md`
  - scope split:
    - import fidelity for KiCad PCB truth preservation
    - scene-contract fidelity for explicit authored / unrouted / proposed /
      diagnostic lanes
    - renderer fidelity for PCB-native semantic readability
  - sequencing rule:
    - execute after the architecture spike proves the
      `gui-app` / `gui-protocol` / `gui-render` boundary
    - execute before broadening imported-board review claims or generalizing
      the opening `M7` workflow beyond the current bounded review surface
  - current correction-track read:
    - Stage 3 unrouted-lane work is functionally landed on the canonical
      half-routed fixture
    - previously recorded Stage 1 / Stage 2 blockers around outline ownership,
      pad rotation, roundrect semantics, outline layer-id carriage, and outline
      visibility gating have materially landed in code; the fidelity docs are
      now being reconciled to that newer repo state
    - `M7-INT-001` closed its first slice (2026-06-09): authored-object
      selection ownership, switch-clears-prior, and hover-preview-only
      behavior are regression-locked in
      `crates/gui-render/tests/selection_ownership.rs`
    - `M7-REN-006` closed (2026-06-09): the render-stack policy now has a
      single code encoding (`RenderStage` declaration order, shared
      `POST_COPPER_STAGES` walk), copper-layer appearance is constructed
      material-first, the bounded exception set is documented at the
      retained-geometry pass header, and contract regression tests lock the
      declared stage ladder
    - `M7-REN-003` closed (2026-06-12): proposed overlays verified
      copper-like at world-true width in selected and non-selected states;
      the diagnostic per-vertex marker violation ("generic path nodes" over
      proposed copper) was removed and regression-locked
    - `M7-REN-004` closed (2026-06-12): filled-zone copper is a declared
      derived shade of the layer material so pad/teardrop/pour boundaries
      read correctly (teardrop tangency criterion verified on DOA2526), and
      dim-unrelated readability is verified on the canonical fixture with
      regression locks in `render_contract_tests.rs`
    - active next implementation frontier is `M7-REG-001..003` fixture-backed
      import/scene/visual regression coverage (owner-ordered 2026-06-11)
    - the renderer semantic contract note now lives in
      `docs/gui/M7_RENDER_SEMANTIC_CONTRACT.md`
  - standards amendment for the opening slice:
    - opening `M7` remains review-focused and does not expand into a full IPC
      authoring/validation milestone
    - standards-relevant imported observables already exposed in the review
      surface must preserve source truth
    - bounded import-audit diagnostics are in-scope where they report delta
      without mutating imported geometry

- Acceptance checks met for the opening slice:
  - deterministic repeated output on unchanged persisted state
  - no pool re-resolution or live import-session dependence
  - no invented routing geometry or inferred constraints beyond persisted facts
  - engine + CLI test coverage for sorted/deterministic extraction

- Acceptance checks required for the next frontend slice:
  - deterministic repeated `board_review_scene_v1` output on unchanged
    persisted state
  - stable authored and review companion identities on unchanged state
  - no frontend-owned semantic geometry or review inference beyond explicit
    machine contracts
  - explicit authored vs proposed vs diagnostic visual separation in the
    opening workspace
  - terminal and AI supporting lanes consume explicit selection/review context
    without becoming parallel design truth
  - the opening route review starts from the first proposal action in
    deterministic review order

- Acceptance checks required for the imported-board fidelity track:
  - supported KiCad PCB layer identities do not silently collapse to fallback
    copper layers
  - supported imported pads preserve the physical dimensions and shape
    semantics required for board review
  - unsupported imported-board cases fail explicitly or remain clearly bounded
    instead of silently producing materially wrong board meaning
  - the accepted fixture set can visually distinguish authored copper,
    unrouted connectivity, proposed overlay geometry, and board-context
    primitives reliably enough for a PCB user to trust the review
  - fixture-backed tests plus screenshot or image-based review cover the
    canonical half-routed board and the supporting imported-board edge cases

---
