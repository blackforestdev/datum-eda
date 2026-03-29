# M5 Opening Charter

This file records the recommended opening boundary for `M5`. Progress-state
authority remains `specs/PROGRESS.md`.

## Objective

Open `M5` as the deterministic layout-kernel milestone, but start with one
narrow slice that consumes persisted native board state only and proves a
layout-kernel substrate without jumping directly to broad autorouting or AI
behavior.

## Recommended First Slice

Recommended first `M5` slice:
- deterministic routing/constraint substrate from persisted native board state

Practical shape:
- extract one engine-ready, deterministic view of routing-relevant board state
- include only explicit authored/persisted facts:
  - board outline
  - stackup/layer set
  - keepouts
  - pads/tracks/vias/zones
  - net / net-class constraints already persisted in native state
- use that substrate to support one narrow routing-facing operation, not a full
  router

## Non-Goals

The opening `M5` slice should explicitly avoid:
- multi-net autorouting
- push-shove routing
- AI placement/routing strategy
- impedance solving
- length matching / serpentine generation
- diff-pair routing beyond explicit future contract work
- any geometry invented from missing source fields

## Entry Criteria

Before coding the first `M5` slice:
1. `M4` is treated as closed for scope in `specs/PROGRESS.md`.
2. The `M5` slice must read only persisted native board state.
3. The first slice must define one explicit contract and one clear non-goal
   set before implementation.
4. Acceptance checks must be written up front.

## Acceptance Shape For The First Slice

The first `M5` slice should satisfy all of:
- deterministic output on repeated runs
- no dependence on live import sessions or pool re-resolution
- no invented constraints or geometry
- focused CLI/engine proof coverage
- touched-monolith burn-down remains structural, not compression-based

## Candidate Entry Contracts

Good candidates:
- routing-kernel input extraction report from persisted board state
- single-net route feasibility/preflight against persisted obstacles
- deterministic obstacle/corridor construction for one authored net

Initial contract selected on 2026-03-28:
- routing-kernel input extraction report from persisted native board state via
  `project query <dir> routing-substrate`
- included persisted facts: outline, stackup/layer set, keepouts,
  authored/persisted pads, tracks, vias, zones, nets, and net classes
- explicit non-goal for this slice: no route search, no feasibility verdict,
  no invented corridor geometry

Follow-on slice selected on 2026-03-28:
- single-net route preflight via `project query <dir> route-preflight --net <uuid>`
- input: persisted native board state only
- output:
  - net identity and persisted net-class facts
  - candidate connection anchors from persisted board pads already assigned to
    that net
  - candidate copper layers derived only from persisted stackup and any
    already-persisted routing constraints currently available in native state
  - authored obstacle inventory relevant to that net: keepouts, foreign-net
    tracks/vias/zones, and outside-outline condition
  - explicit status such as `preflight_ready`,
    `blocked_by_authored_obstacle`, or `insufficient_authored_inputs`
- explicit non-goals for this slice:
  - no route search
  - no feasibility corridor synthesis
  - no invented clearances or per-net layer permissions
  - no pool lookup or package re-resolution

Next slice selected on 2026-03-28:
- deterministic single-net corridor geometry via
  `project query <dir> route-corridor --net <uuid>`
- input: persisted native board state only
- output:
  - net identity and persisted net-class facts
  - the same authored anchors already proven by `route-preflight`
  - candidate copper layers derived only from persisted stackup and currently
    available persisted routing facts in native state
  - deterministic authored obstacle geometry relevant to the target net on
    those layers
  - deterministic corridor spans marked available or blocked by authored
    geometry only
  - explicit status such as `corridor_available`, `corridor_blocked`, or
    `insufficient_authored_inputs`
- explicit non-goals for this slice:
  - no pathfinding
  - no A*
  - no route proposal or autorouting
  - no negotiated reroute or push-shove
  - no invented layer permissions, clearances, or path scoring semantics
  - no pool lookup or package re-resolution

Poor candidates for the first `M5` slice:
- full autorouter
- broad placement engine
- AI proposal layer
- anything that bundles several layout semantics into one step
