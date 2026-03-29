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

Current next slice selected on 2026-03-28:
- deterministic single-layer path candidate via
  `project query <dir> route-path-candidate --net <uuid> --from-anchor <pad_uuid> --to-anchor <pad_uuid>`
- input: persisted native board state only
- output:
  - target net identity
  - one authored source/target anchor pair only
  - candidate copper layers considered under the existing corridor/span model
  - explicit status exactly one of:
    - `deterministic_path_found`
    - `no_path_under_current_authored_constraints`
  - when found, exactly one deterministic geometric path as an ordered
    polyline taken directly from the first unblocked matching corridor span in
    corridor report order (candidate copper layer order, then pair index)
    polyline on a single candidate layer only
- explicit non-goals for this slice:
  - no layer transitions
  - no alternatives or ranking
  - no rip-up/reroute, negotiated routing, or push-shove
  - no autorouting or AI/proposal framing
  - no invented constraints or permissions

Immediate follow-on review/debug slice selected on 2026-03-28:
- deterministic path-candidate explanation via
  `project query <dir> route-path-candidate-explain --net <uuid> --from-anchor <pad_uuid> --to-anchor <pad_uuid>`
- input: persisted native board state only
- output:
  - current `route-path-candidate` status
  - explicit deterministic selection rule
  - selected corridor span when found
  - otherwise whether failure is due to no matching corridor span or all
    matching spans blocked
  - blocked-span reasons only from existing corridor blockage facts
- non-goals:
  - no new routing semantics
  - no layer transitions
  - no multi-path inventory
  - no ranking or scoring

Current checkpoint state on 2026-03-28:
- accepted deterministic read-only kernel chain:
  - `routing-substrate`
  - `route-preflight`
  - `route-corridor`
  - `route-path-candidate`
  - `route-path-candidate-explain`
- this is an intentional M5 checkpoint, not an implied launch point for the
  next contract
- the next planning decision must be explicit and separate:
  - stop M5 here temporarily, or
  - open one new tightly-scoped contract in a later planning step

Next explicit contract selected on 2026-03-29:
- deterministic single-via path candidate via
  `project query <dir> route-path-candidate-via --net <uuid> --from-anchor <pad_uuid> --to-anchor <pad_uuid>`
- input: persisted native board state only
- output:
  - current single-net source/target anchor pair
  - candidate copper layers from persisted stackup and current persisted routing facts
  - explicit status exactly one of:
    - `deterministic_path_found`
    - `no_path_under_current_authored_constraints`
  - when found, exactly one deterministic geometric path reusing one already-authored persisted via only
- deterministic rule:
  - select the first authored target-net via in ascending via UUID order whose boundary layers exactly match the requested anchor layers and whose source-to-via and via-to-target segments are both unblocked
- non-goals:
  - no creating vias
  - no choosing among many vias beyond the explicit deterministic rule
  - no multilayer search beyond one existing via transition
  - no invented transition permissions

Follow-on explanation surface selected on 2026-03-29:
- deterministic single-via path candidate explanation
  `project query <dir> route-path-candidate-via-explain --net <uuid> --from-anchor <pad_uuid> --to-anchor <pad_uuid>`
- input: persisted native board state only
- output:
  - current single-net source/target anchor pair
  - current `route-path-candidate-via` status
  - explicit deterministic via selection rule
  - selected via when found, or whether failure came from no matching authored
    via versus all matching vias blocked
- non-goals:
  - no new routing semantics
  - no added via-choice heuristics
  - no transition inference beyond the accepted via slice

Next explicit contract selected on 2026-03-29:
- deterministic two-via path candidate
  `project query <dir> route-path-candidate-two-via --net <uuid> --from-anchor <pad_uuid> --to-anchor <pad_uuid>`
- input: persisted native board state only
- output:
  - current single-net source/target anchor pair
  - explicit status exactly one of:
    - `deterministic_path_found`
    - `no_path_under_current_authored_constraints`
  - when found, exactly one deterministic geometric path reusing exactly two
    already-authored persisted vias only
- deterministic rule:
  - select the first unblocked matching authored via pair in ascending
    `(via_a_uuid, via_b_uuid)` order whose layer sequence connects the
    requested anchor layers through one intermediate copper layer
- non-goals:
  - no creating vias
  - no inferred transition permissions
  - no free multilayer or multi-via search beyond the explicit pair rule

Follow-on explanation surface selected on 2026-03-29:
- deterministic two-via path candidate explanation
  `project query <dir> route-path-candidate-two-via-explain --net <uuid> --from-anchor <pad_uuid> --to-anchor <pad_uuid>`
- input: persisted native board state only
- output:
  - current single-net source/target anchor pair
  - current `route-path-candidate-two-via` status
  - explicit deterministic via-pair selection rule
  - selected via pair when found, or whether failure came from no matching
    authored via pair versus all matching via pairs blocked
- non-goals:
  - no new routing semantics
  - no added via-pair choice heuristics
  - no transition inference beyond the accepted two-via slice

Next explicit contract selected on 2026-03-29:
- deterministic three-via path candidate
  `project query <dir> route-path-candidate-three-via --net <uuid> --from-anchor <pad_uuid> --to-anchor <pad_uuid>`
- input: persisted native board state only
- output:
  - current single-net source/target anchor pair
  - explicit status exactly one of:
    - `deterministic_path_found`
    - `no_path_under_current_authored_constraints`
  - when found, exactly one deterministic geometric path reusing exactly three
    already-authored persisted vias only
- deterministic rule:
  - select the first unblocked matching authored via triple in ascending
    `(via_a_uuid, via_b_uuid, via_c_uuid)` order whose layer sequence connects
    the requested anchor layers through two intermediate copper layers
- non-goals:
  - no creating vias
  - no inferred transition permissions
  - no free multilayer or multi-via search beyond the explicit triple rule

Follow-on explanation surface selected on 2026-03-29:
- deterministic three-via path candidate explanation
  `project query <dir> route-path-candidate-three-via-explain --net <uuid> --from-anchor <pad_uuid> --to-anchor <pad_uuid>`
- input: persisted native board state only
- output:
  - current single-net source/target anchor pair
  - current `route-path-candidate-three-via` status
  - explicit deterministic via-triple selection rule
  - selected via triple when found, or whether failure came from no matching
    authored via triple versus all matching via triples blocked
- non-goals:
  - no new routing semantics
  - no added via-triple choice heuristics
  - no transition inference beyond the accepted three-via slice

Next explicit contract selected on 2026-03-29:
- deterministic four-via path candidate
  `project query <dir> route-path-candidate-four-via --net <uuid> --from-anchor <pad_uuid> --to-anchor <pad_uuid>`
- input: persisted native board state only
- output:
  - current single-net source/target anchor pair
  - explicit status exactly one of:
    - `deterministic_path_found`
    - `no_path_under_current_authored_constraints`
  - when found, exactly one deterministic geometric path reusing exactly four
    already-authored persisted vias only
- deterministic rule:
  - select the first unblocked matching authored via quadruple in ascending
    `(via_a_uuid, via_b_uuid, via_c_uuid, via_d_uuid)` order whose layer
    sequence connects the requested anchor layers through three intermediate
    copper layers
- non-goals:
  - no creating vias
  - no inferred transition permissions
  - no free multilayer or multi-via search beyond the explicit quadruple rule

Follow-on explanation surface selected on 2026-03-29:
- deterministic four-via path candidate explanation
  `project query <dir> route-path-candidate-four-via-explain --net <uuid> --from-anchor <pad_uuid> --to-anchor <pad_uuid>`
- input: persisted native board state only
- output:
  - current single-net source/target anchor pair
  - current `route-path-candidate-four-via` status
  - explicit deterministic via-quadruple selection rule
  - selected via quadruple when found, or whether failure came from no matching
    authored via quadruple versus all matching via quadruples blocked
- non-goals:
  - no new routing semantics
  - no added via-quadruple choice heuristics
  - no transition inference beyond the accepted four-via slice

Next explicit contract selected on 2026-03-29:
- deterministic five-via path candidate
  `project query <dir> route-path-candidate-five-via --net <uuid> --from-anchor <pad_uuid> --to-anchor <pad_uuid>`
- input: persisted native board state only
- output:
  - current single-net source/target anchor pair
  - explicit status exactly one of:
    - `deterministic_path_found`
    - `no_path_under_current_authored_constraints`
  - when found, exactly one deterministic geometric path reusing exactly five

Adjacent thin slice implemented on 2026-03-29:
- deterministic five-via explanation surface
  `project query <dir> route-path-candidate-five-via-explain --net <uuid> --from-anchor <pad_uuid> --to-anchor <pad_uuid>`
- input: persisted native board state only
- output:
  - current selected via quintuple when found
  - or whether failure came from no matching authored via quintuple versus all
    matching via quintuples blocked
- guardrails:
  - no new routing semantics
  - no new via selection logic beyond the accepted five-via slice

Next explicit contract selected on 2026-03-29:
- deterministic six-via path candidate
  `project query <dir> route-path-candidate-six-via --net <uuid> --from-anchor <pad_uuid> --to-anchor <pad_uuid>`
- input: persisted native board state only
- output:
  - current single-net source/target anchor pair
  - explicit status exactly one of:
    - `deterministic_path_found`
    - `no_path_under_current_authored_constraints`
  - when found, exactly one deterministic geometric path reusing exactly six
    already-authored persisted vias only

Adjacent thin slice implemented on 2026-03-29:
- deterministic six-via explanation surface
  `project query <dir> route-path-candidate-six-via-explain --net <uuid> --from-anchor <pad_uuid> --to-anchor <pad_uuid>`
- input: persisted native board state only
- output:
  - current selected via sextuple when found
  - or whether failure came from no matching authored via sextuple versus all
    matching via sextuples blocked
- guardrails:
  - no new routing semantics
  - no new via selection logic beyond the accepted six-via slice

Next explicit contract selected on 2026-03-29:
- deterministic authored-via-chain path candidate
  `project query <dir> route-path-candidate-authored-via-chain --net <uuid> --from-anchor <pad_uuid> --to-anchor <pad_uuid>`
- input: persisted native board state only
- output:
  - current single-net source/target anchor pair
  - explicit status exactly one of:
    - `deterministic_path_found`
    - `no_path_under_current_authored_constraints`
  - when found, exactly one deterministic geometric path reusing only
    already-authored persisted target-net vias
- deterministic rule:
  - enumerate authored via chains whose layer sequence connects the requested
    anchor layers and whose segment geometry is explainable entirely from
    existing corridor/span facts plus the reused authored vias
  - order candidate chains by `(via_count, via_uuid_sequence)` ascending
  - select the first unblocked matching chain under that explicit order
- non-goals:
  - no creating vias
  - no inferred transition permissions
  - no rip-up/reroute, negotiated routing, or push-shove
  - no ranking output or fallback inventory beyond the selected chain

Adjacent thin slice implemented on 2026-03-29:
- deterministic authored-via-chain explanation surface
  `project query <dir> route-path-candidate-authored-via-chain-explain --net <uuid> --from-anchor <pad_uuid> --to-anchor <pad_uuid>`
- input: persisted native board state only
- output:
  - current selected authored via chain when found
  - or whether failure came from no matching authored via chain versus all
    matching via chains blocked
- guardrails:
  - no new routing semantics
  - no new via-chain selection logic beyond the accepted generalized chain slice

Next explicit contract selected on 2026-03-29:
- deterministic existing-authored-copper graph path candidate
  `project query <dir> route-path-candidate-authored-copper-graph --net <uuid> --from-anchor <pad_uuid> --to-anchor <pad_uuid>`
- input: persisted native board state only
- output:
  - current single-net source/target anchor pair
  - explicit status exactly one of:
    - `deterministic_path_found`
    - `no_path_under_current_authored_constraints`
  - when found, exactly one deterministic geometric path made only of
    already-authored persisted target-net tracks and vias
- deterministic rule:
  - build the existing authored-copper graph from persisted target-net tracks
    and vias only
  - select the first path found by breadth-first traversal after sorting graph
    edges by `(step_kind, object_uuid, destination_anchor)`, yielding a
    deterministic minimum-step path with lexicographic tie-breaks
- non-goals:
  - no new copper creation
  - no inferred transitions beyond persisted target-net vias
  - no ranking output beyond the selected existing-copper path

Follow-on explanation surface selected on 2026-03-29:
- deterministic existing-authored-copper graph path candidate explanation
  `project query <dir> route-path-candidate-authored-copper-graph-explain --net <uuid> --from-anchor <pad_uuid> --to-anchor <pad_uuid>`
- input: persisted native board state only
- output:
  - current selected existing-authored-copper path when found
  - or explicit `no_existing_authored_copper_path` when the persisted target-net
    copper graph does not connect the requested anchors
- guardrails:
  - no new routing semantics
  - no blockage inference beyond the accepted authored-copper graph slice
  - no ranking output beyond the selected existing-copper path

Next explicit contract selected on 2026-03-29:
- deterministic zone-aware existing-authored-copper graph path candidate
  `project query <dir> route-path-candidate-authored-copper-graph-zone-aware --net <uuid> --from-anchor <pad_uuid> --to-anchor <pad_uuid>`
- input: persisted native board state only
- output:
  - current single-net source/target anchor pair
  - explicit status exactly one of:
    - `deterministic_path_found`
    - `no_path_under_current_authored_constraints`
  - when found, exactly one deterministic geometric path made only of
    already-authored persisted target-net tracks, vias, and zone-supported
    on-layer continuity
- deterministic rule:
  - build the existing authored-copper graph from persisted target-net tracks
    and vias only
  - add on-layer target-net zone edges between authored anchor, track-end, and
    via-end graph points that lie within the same persisted target-net zone
  - select the first path found by breadth-first traversal after sorting graph
    edges by `(step_kind, object_uuid, destination_anchor)`, yielding a
    deterministic minimum-step path with lexicographic tie-breaks
- non-goals:
  - no new copper creation
  - no inferred transitions beyond persisted target-net vias
  - no ranking output beyond the selected existing-copper path

Follow-on explanation surface selected on 2026-03-29:
- deterministic zone-aware existing-authored-copper graph path candidate explanation
  `project query <dir> route-path-candidate-authored-copper-graph-zone-aware-explain --net <uuid> --from-anchor <pad_uuid> --to-anchor <pad_uuid>`
- input: persisted native board state only
- output:
  - current selected zone-aware existing-authored-copper path when found
  - or explicit `no_existing_authored_copper_path` when the persisted target-net
    track/via/zone graph does not connect the requested anchors
- guardrails:
  - no new routing semantics
  - no blockage inference beyond the accepted zone-aware graph slice
  - no ranking output beyond the selected existing-copper path

Next explicit contract selected on 2026-03-29:
- deterministic zone-obstacle-aware existing-authored-copper graph path candidate
  `project query <dir> route-path-candidate-authored-copper-graph-zone-obstacle-aware --net <uuid> --from-anchor <pad_uuid> --to-anchor <pad_uuid>`
- input: persisted native board state only
- output:
  - current single-net source/target anchor pair
  - explicit status exactly one of:
    - `deterministic_path_found`
    - `no_path_under_current_authored_constraints`
  - when found, exactly one deterministic geometric path made only of
    already-authored persisted target-net tracks, vias, and zone-supported
    continuity whose reused graph edges are unblocked under the current
    authored obstacle checks
- deterministic rule:
  - build the existing authored-copper graph from persisted target-net tracks,
    vias, and zone continuity only
  - exclude any track, via, or zone-connection edge whose reused geometry is
    blocked under the current authored obstacle checks
  - select the first path found by breadth-first traversal after sorting graph
    edges by `(step_kind, object_uuid, destination_anchor)`, yielding a
    deterministic minimum-step path with lexicographic tie-breaks
- non-goals:
  - no new copper creation
  - no inferred transitions beyond persisted target-net vias
  - no ranking output beyond the selected existing-copper path

Next explicit contract selected on 2026-03-29:
- deterministic obstacle-aware existing-authored-copper graph path candidate
  `project query <dir> route-path-candidate-authored-copper-graph-obstacle-aware --net <uuid> --from-anchor <pad_uuid> --to-anchor <pad_uuid>`
- input: persisted native board state only
- output:
  - current single-net source/target anchor pair
  - explicit status exactly one of:
    - `deterministic_path_found`
    - `no_path_under_current_authored_constraints`
  - when found, exactly one deterministic geometric path made only of
    already-authored persisted target-net tracks and vias whose reused
    geometry is unblocked under the current authored obstacle checks
- deterministic rule:
  - build the existing authored-copper graph from persisted target-net tracks
    and vias only
  - exclude any track or via edge whose reused geometry is blocked under the
    current authored obstacle checks
  - select the first path found by breadth-first traversal after sorting graph
    edges by `(step_kind, object_uuid, destination_anchor)`, yielding a
    deterministic minimum-step path with lexicographic tie-breaks
- non-goals:
  - no new copper creation
  - no inferred transitions beyond persisted target-net vias
  - no ranking output beyond the selected existing-copper path

Follow-on explanation surface selected on 2026-03-29:
- deterministic obstacle-aware existing-authored-copper graph path candidate explanation
  `project query <dir> route-path-candidate-authored-copper-graph-obstacle-aware-explain --net <uuid> --from-anchor <pad_uuid> --to-anchor <pad_uuid>`
- input: persisted native board state only
- output:
  - current selected obstacle-aware existing-authored-copper path when found
  - or explicit `no_existing_authored_copper_path` when authored obstacle
    filtering leaves no connecting persisted target-net copper path
- guardrails:
  - no new routing semantics
  - no new obstacle interpretation beyond the accepted obstacle-aware graph slice
  - no ranking output beyond the selected existing-copper path

Poor candidates for the first `M5` slice:
- full autorouter
- broad placement engine
- AI proposal layer
- anything that bundles several layout semantics into one step
