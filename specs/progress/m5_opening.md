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
- deterministic route candidate family via
  `project query <dir> route-path-candidate --net <uuid> --from-anchor <pad_uuid> --to-anchor <pad_uuid> --candidate <accepted_candidate> [--policy <policy>]`
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
- the canonical route query/explanation entrypoints are now the bounded generic
  `route-path-candidate` and `route-path-candidate-explain` surfaces with
  `--candidate <accepted_candidate> [--policy <policy>]`
- contract-specific `route-path-candidate-*` and
  `route-path-candidate-*-explain` commands now remain only as deprecated
  compatibility wrappers around those generic surfaces
- those remaining compatibility wrappers now dispatch through the same shared
  generic candidate/policy executor path as the canonical surfaces, keeping one
  maintained route query/explanation decision path ahead of wrapper retirement
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
- generalized preferred replacement for further authored-copper-graph suffix growth:
  `project query <dir> route-path-candidate-authored-copper-graph --net <uuid> --from-anchor <pad_uuid> --to-anchor <pad_uuid> --policy <policy>`
  - accepted bounded policy enum:
    - `plain`
    - `zone_aware`
    - `obstacle_aware`
    - `zone_obstacle_aware`
    - `zone_obstacle_topology_aware`
    - `zone_obstacle_topology_layer_balance_aware`
  - suffix contracts remain for compatibility, but no longer define the preferred growth path for this family
- non-goals:
  - no new copper creation
  - no inferred transitions beyond persisted target-net vias
  - no ranking output beyond the selected existing-copper path

Follow-on explanation surface selected on 2026-03-29:
- deterministic existing-authored-copper graph path candidate explanation
  `project query <dir> route-path-candidate-authored-copper-graph-explain --net <uuid> --from-anchor <pad_uuid> --to-anchor <pad_uuid> --policy <policy>`
- input: persisted native board state only
- output:
  - current selected existing-authored-copper path when found
  - or explicit `no_existing_authored_copper_path` when the persisted target-net
    copper graph does not connect the requested anchors
- guardrails:
  - no new routing semantics
  - no blockage inference beyond the accepted authored-copper graph slice
  - no ranking output beyond the selected existing-copper path
  - bounded to the accepted policy set already proven by the generalized
    `route-path-candidate-authored-copper-graph --policy <policy>` surface

Next routing-facing bridge slice selected on 2026-03-29:
- deterministic authored-copper path candidate with exactly one eligible gap
  `project query <dir> route-path-candidate-authored-copper-plus-one-gap --net <uuid> --from-anchor <pad_uuid> --to-anchor <pad_uuid>`
- input: persisted native board state only
- output:
  - one deterministic path composed of authored target-net copper plus exactly
    one eligible same-layer synthetic gap
  - or `no_path_under_current_authored_constraints`
- guardrails:
  - no rip-up/reroute
  - no negotiated routing
  - no push-shove
  - no invented constraints or permissions
  - the synthetic gap must be justified entirely by existing candidate copper
    layer facts and persisted obstacle-truth segment checks

Adjacent write-capable bridge implemented on 2026-03-29:
- deterministic route proposal artifact/export/apply lane for the accepted
  plus-one-gap contract
  - canonical surface: `project export-route-path-proposal <dir> --net <uuid> --from-anchor <pad_uuid> --to-anchor <pad_uuid> --candidate authored-copper-plus-one-gap --out <path>`
  - `project inspect-route-proposal-artifact <path>`
  - `project apply-route-proposal-artifact <dir> --artifact <path>`
- input: persisted native board state only
- output:
  - one versioned route-proposal artifact containing exactly one
    self-sufficient `draw_track` action for the selected synthetic gap segment
  - apply mutates persisted board state by materializing only that selected gap
    segment as one authored target-net track using persisted net-class width
- guardrails:
  - no route search beyond the accepted plus-one-gap query contract
  - no duplication of already-authored copper or vias
  - no invented track width; width comes from persisted net-class facts only
  - apply rejects artifact drift and requires the live deterministic proposal to
    still match the exported action exactly

Adjacent write-capable route-application slice implemented on 2026-03-29:
- deterministic route proposal artifact export for the accepted single-layer
  `route-path-candidate` contract
  - canonical surface: `project export-route-path-proposal <dir> --net <uuid> --from-anchor <pad_uuid> --to-anchor <pad_uuid> --candidate route-path-candidate --out <path>`
  - reuses the same `project inspect-route-proposal-artifact <path>` and
    `project apply-route-proposal-artifact <dir> --artifact <path>` surfaces
- input: persisted native board state only
- output:
  - one versioned route-proposal artifact containing an ordered
    self-sufficient `draw_track` action sequence for the selected
    deterministic single-layer polyline
  - apply mutates persisted board state by materializing that full selected
    path as authored target-net tracks using persisted net-class width
- guardrails:
  - no reranking or re-search beyond the accepted `route-path-candidate`
    query contract
  - no invented track width; width comes from persisted net-class facts only
  - apply rejects artifact drift and requires the live deterministic proposal to
    still match the exported action sequence exactly

Adjacent write-capable route-application slice implemented on 2026-03-29:
- deterministic route proposal artifact export for the accepted bounded
  single-via `route-path-candidate-via` contract
  - canonical surface: `project export-route-path-proposal <dir> --net <uuid> --from-anchor <pad_uuid> --to-anchor <pad_uuid> --candidate route-path-candidate-via --out <path>`
  - reuses the same `project inspect-route-proposal-artifact <path>` and
    `project apply-route-proposal-artifact <dir> --artifact <path>` surfaces
- input: persisted native board state only
- output:
  - one versioned route-proposal artifact containing the ordered
    self-sufficient `draw_track` action sequence for the two selected copper
    segments around the reused authored via
  - apply mutates persisted board state by materializing those selected
    segments only; the chosen via remains reused authored state and is not
    recreated
- guardrails:
  - no via creation
  - no re-search beyond the accepted `route-path-candidate-via` query contract
  - apply drift-check includes the selected reused via UUID as part of the live
    deterministic proposal match

Adjacent write-capable route-application slice implemented on 2026-03-29:
- deterministic route proposal artifact export for the accepted bounded
  two-via `route-path-candidate-two-via` contract
  - canonical surface: `project export-route-path-proposal <dir> --net <uuid> --from-anchor <pad_uuid> --to-anchor <pad_uuid> --candidate route-path-candidate-two-via --out <path>`
  - reuses the same `project inspect-route-proposal-artifact <path>` and
    `project apply-route-proposal-artifact <dir> --artifact <path>` surfaces
- input: persisted native board state only
- output:
  - one versioned route-proposal artifact containing the ordered
    self-sufficient `draw_track` action sequence for the three selected copper
    segments around the reused authored via pair
  - apply mutates persisted board state by materializing those selected
    segments only; the chosen vias remain reused authored state and are not
    recreated
- guardrails:
  - no via creation
  - no re-search beyond the accepted `route-path-candidate-two-via` query
    contract
  - apply drift-check includes the selected reused via pair as part of the
    live deterministic proposal match

Adjacent write-capable route-application slice implemented on 2026-03-29:
- deterministic route proposal artifact export for the remaining bounded
  via-path contracts
  - canonical surfaces:
    `project export-route-path-proposal <dir> --net <uuid> --from-anchor <pad_uuid> --to-anchor <pad_uuid> --candidate route-path-candidate-three-via --out <path>`,
    `... --candidate route-path-candidate-four-via`,
    `... --candidate route-path-candidate-five-via`,
    `... --candidate route-path-candidate-six-via`, and
    `... --candidate route-path-candidate-authored-via-chain`
  - reuses the same `project inspect-route-proposal-artifact <path>` and
    `project apply-route-proposal-artifact <dir> --artifact <path>` surfaces
- input: persisted native board state only
- output:
  - one versioned route-proposal artifact containing the ordered
    self-sufficient `draw_track` action sequence for the selected copper
    segments of each accepted deterministic via-path contract
  - apply mutates persisted board state by materializing those selected
    segments only; every selected via remains reused authored state and is not
    recreated
- guardrails:
  - no via creation
  - no re-search beyond the accepted deterministic via-path query contracts
  - apply drift-check includes the exact selected reused via UUID sequence as
    part of the live deterministic proposal match

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

Adjacent write-capable route-application slice implemented on 2026-03-29:
- deterministic route proposal artifact export for the accepted zone-aware
  existing-authored-copper graph contract
  - canonical surface: `project export-route-path-proposal <dir> --net <uuid> --from-anchor <pad_uuid> --to-anchor <pad_uuid> --candidate authored-copper-graph --policy zone_aware --out <path>`
  - reuses the same `project inspect-route-proposal-artifact <path>` and
    `project apply-route-proposal-artifact <dir> --artifact <path>` surfaces
- input: persisted native board state only
- output:
  - one versioned route-proposal artifact containing the ordered
    self-sufficient `reuse_existing_copper_step` action sequence for the
    selected existing authored-copper path only
  - every action records the reused object kind/UUID and layer span as
    drift-check metadata
  - apply performs live deterministic proposal matching and succeeds as a
    no-op when the selected existing-copper path still matches exactly
- guardrails:
  - no new copper geometry or via creation
  - no re-search beyond the accepted
    `route-path-candidate-authored-copper-graph-zone-aware` query contract
  - apply drift-check includes the selected reused object sequence as part of
    the live deterministic proposal match

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

Follow-on explanation surface selected on 2026-03-29:
- deterministic zone-obstacle-aware existing-authored-copper graph path candidate explanation
  `project query <dir> route-path-candidate-authored-copper-graph-zone-obstacle-aware-explain --net <uuid> --from-anchor <pad_uuid> --to-anchor <pad_uuid>`
- input: persisted native board state only
- output:
  - current selected zone-obstacle-aware existing-authored-copper path when found
  - or explicit `no_existing_authored_copper_path` when authored obstacle

Adjacent write-capable route-application slice implemented on 2026-03-29:
- deterministic route proposal artifact export for the accepted
  zone-obstacle-aware existing-authored-copper graph contract
  - canonical surface: `project export-route-path-proposal <dir> --net <uuid> --from-anchor <pad_uuid> --to-anchor <pad_uuid> --candidate authored-copper-graph --policy zone_obstacle_aware --out <path>`
  - reuses the same `project inspect-route-proposal-artifact <path>` and
    `project apply-route-proposal-artifact <dir> --artifact <path>` surfaces
- input: persisted native board state only
- output:
  - one versioned route-proposal artifact containing the ordered
    self-sufficient `reuse_existing_copper_step` action sequence for the
    selected obstacle-checked existing authored-copper path only
  - every action records the reused object kind/UUID and layer span as
    drift-check metadata
  - apply performs live deterministic proposal matching and succeeds as a
    no-op when the selected existing-copper path still matches exactly
- guardrails:
  - no new copper geometry or via creation
  - no re-search beyond the accepted
    `route-path-candidate-authored-copper-graph-zone-obstacle-aware` query
    contract
  - apply drift-check includes the selected reused object sequence as part of
    the live deterministic proposal match

Next explicit contract selected on 2026-03-29:
- deterministic topology-aware zone-obstacle-aware existing-authored-copper graph path candidate
  `project query <dir> route-path-candidate-authored-copper-graph-zone-obstacle-aware-topology-aware --net <uuid> --from-anchor <pad_uuid> --to-anchor <pad_uuid>`
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
  - order surviving candidate paths by `(step_count,
    topology_transition_count, via_step_count, zone_step_count,
    step_signature_sequence)` ascending
  - select the first path under that explicit whole-path ordering

Follow-on explanation surface selected on 2026-03-29:
- deterministic topology-aware zone-obstacle-aware existing-authored-copper graph path candidate explanation
  `project query <dir> route-path-candidate-authored-copper-graph-zone-obstacle-aware-topology-aware-explain --net <uuid> --from-anchor <pad_uuid> --to-anchor <pad_uuid>`
- input: persisted native board state only
- output:

Adjacent write-capable route-application slice implemented on 2026-03-29:
- deterministic route proposal artifact export for the accepted topology-aware
  zone-obstacle-aware existing-authored-copper graph contract
  - canonical surface: `project export-route-path-proposal <dir> --net <uuid> --from-anchor <pad_uuid> --to-anchor <pad_uuid> --candidate authored-copper-graph --policy zone_obstacle_topology_aware --out <path>`
  - reuses the same `project inspect-route-proposal-artifact <path>` and
    `project apply-route-proposal-artifact <dir> --artifact <path>` surfaces
- input: persisted native board state only
- output:
  - one versioned route-proposal artifact containing the ordered
    self-sufficient `reuse_existing_copper_step` action sequence for the
    selected whole-path-ordered existing authored-copper path only
  - every action records the reused object kind/UUID and layer span as
    drift-check metadata
  - apply performs live deterministic proposal matching and succeeds as a
    no-op when the selected existing-copper path still matches exactly
- guardrails:
  - no new copper geometry or via creation
  - no re-search beyond the accepted
    `route-path-candidate-authored-copper-graph-zone-obstacle-aware-topology-aware`
    query contract
  - apply drift-check includes the selected reused object sequence as part of
    the live deterministic proposal match

Next explicit contract selected on 2026-03-29:
- deterministic layer-balance-aware topology-aware zone-obstacle-aware existing-authored-copper graph path candidate
  `project query <dir> route-path-candidate-authored-copper-graph-zone-obstacle-aware-topology-aware-layer-balance-aware --net <uuid> --from-anchor <pad_uuid> --to-anchor <pad_uuid>`
- input: persisted native board state only
- output:
  - current selected topology-aware zone-obstacle-aware existing-authored-copper path when found
  - or explicit `no_existing_authored_copper_path` when authored obstacle
    filtering leaves no connecting persisted target-net copper path under the
    same whole-path ordering rule

Next explicit contract selected on 2026-03-29:
- deterministic layer-balance-aware topology-aware zone-obstacle-aware existing-authored-copper graph path candidate
  `project query <dir> route-path-candidate-authored-copper-graph-zone-obstacle-aware-topology-aware-layer-balance-aware --net <uuid> --from-anchor <pad_uuid> --to-anchor <pad_uuid>`
- deterministic explanation surface for the current layer-balance-aware topology-aware zone-obstacle-aware existing-authored-copper graph path candidate result
  `project query <dir> route-path-candidate-authored-copper-graph-zone-obstacle-aware-topology-aware-layer-balance-aware-explain --net <uuid> --from-anchor <pad_uuid> --to-anchor <pad_uuid>`
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
  - preserve the topology-aware whole-path ordering
  - refine ties with `layer_balance_score` computed as max-minus-min reused
    step count across candidate copper layers
    filtering leaves no connecting persisted target-net copper path
- guardrails:
  - no new routing semantics
  - no new obstacle interpretation beyond the accepted zone-obstacle-aware graph slice
  - no ranking output beyond the selected existing-copper path

Adjacent write-capable route-application slice implemented on 2026-03-29:
- deterministic route proposal artifact export for the accepted
  layer-balance-aware topology-aware zone-obstacle-aware
  existing-authored-copper graph contract
  - canonical surface: `project export-route-path-proposal <dir> --net <uuid> --from-anchor <pad_uuid> --to-anchor <pad_uuid> --candidate authored-copper-graph --policy zone_obstacle_topology_layer_balance_aware --out <path>`
  - reuses the same `project inspect-route-proposal-artifact <path>` and
    `project apply-route-proposal-artifact <dir> --artifact <path>` surfaces
- input: persisted native board state only
- output:
  - one versioned route-proposal artifact containing the ordered
    self-sufficient `reuse_existing_copper_step` action sequence for the
    selected layer-balance-ordered existing authored-copper path only
  - every action records the reused object kind/UUID and layer span as
    drift-check metadata
  - apply performs live deterministic proposal matching and succeeds as a
    no-op when the selected existing-copper path still matches exactly
- guardrails:
  - no new copper geometry or via creation
  - no re-search beyond the accepted
    `route-path-candidate-authored-copper-graph-zone-obstacle-aware-topology-aware-layer-balance-aware`
    query contract
  - apply drift-check includes the selected reused object sequence as part of
    the live deterministic proposal match

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

Adjacent write-capable route-application slice implemented on 2026-03-29:
- deterministic route proposal artifact export for the accepted
  obstacle-aware existing-authored-copper graph contract
  - canonical surface: `project export-route-path-proposal <dir> --net <uuid> --from-anchor <pad_uuid> --to-anchor <pad_uuid> --candidate authored-copper-graph --policy obstacle_aware --out <path>`
  - reuses the same `project inspect-route-proposal-artifact <path>` and
    `project apply-route-proposal-artifact <dir> --artifact <path>` surfaces
- input: persisted native board state only
- output:
  - one versioned route-proposal artifact containing the ordered
    self-sufficient `reuse_existing_copper_step` action sequence for the
    selected obstacle-checked existing authored-copper path only
  - every action records the reused object kind/UUID and layer span as
    drift-check metadata
  - apply performs live deterministic proposal matching and succeeds as a
    no-op when the selected existing-copper path still matches exactly
- guardrails:
  - no new copper geometry or via creation
  - no re-search beyond the accepted
    `route-path-candidate-authored-copper-graph-obstacle-aware` query
    contract
  - apply drift-check includes the selected reused object sequence as part of
    the live deterministic proposal match

Adjacent write-capable route-application slice implemented on 2026-03-29:
- policy-selected deterministic route proposal artifact export for the
  completed authored-copper-graph family
  - canonical surface: `project export-route-path-proposal <dir> --net <uuid> --from-anchor <pad_uuid> --to-anchor <pad_uuid> --candidate authored-copper-graph --policy <policy> --out <path>`
  - reuses the same `project inspect-route-proposal-artifact <path>` and
    `project apply-route-proposal-artifact <dir> --artifact <path>` surfaces
- input: persisted native board state only
- output:
  - one versioned route-proposal artifact containing the ordered
    self-sufficient `reuse_existing_copper_step` action sequence for the
    selected existing authored-copper path under the requested accepted
    policy
  - the accepted bounded policy set is exactly `plain`, `zone_aware`,
    `obstacle_aware`, `zone_obstacle_aware`,
    `zone_obstacle_topology_aware`, and
    `zone_obstacle_topology_layer_balance_aware`
  - every action records the reused object kind/UUID and layer span as
    drift-check metadata
  - apply performs live deterministic proposal matching under the same policy
    and succeeds as a no-op when the selected existing-copper path still
    matches exactly
- guardrails:
  - no new copper geometry or via creation
  - no new routing semantics beyond the already-accepted authored-copper-graph
    policy query family
  - no re-search beyond the selected
    `route-path-candidate-authored-copper-graph --policy <policy>` query
    contract
  - apply drift-check includes the selected reused object sequence and chosen
    policy as part of the live deterministic proposal match

Adjacent MCP parity slice implemented on 2026-03-29:
- the policy-selected authored-copper-graph export surface is exposed through
  the generic MCP route export tool `export_route_path_proposal`
- input:
  - native project directory path
  - `net_uuid`
  - `from_anchor_pad_uuid`
  - `to_anchor_pad_uuid`
  - accepted bounded `policy`
  - artifact `out` path
- output:
  - the same JSON export report produced by the native CLI
- guardrails:
  - MCP parity is reopened only for this explicit export surface
  - transport remains a thin adapter; no generalized M5 query/apply MCP family
    is introduced by this slice

Adjacent MCP parity slice implemented on 2026-03-29:
- the generic route-proposal artifact follow-up surfaces are now exposed
  through MCP as:
  - `inspect_route_proposal_artifact`
  - `apply_route_proposal_artifact`
- input:
  - `inspect_route_proposal_artifact`: artifact path only
  - `apply_route_proposal_artifact`: native project directory path plus
    artifact path
- output:
  - the same JSON inspection/apply reports produced by the native CLI
- guardrails:
  - MCP parity remains scoped to the artifact lifecycle around the completed
    policy-selected authored-copper-graph proposal surface
  - transport remains a thin adapter; no generalized M5 query parity is
    introduced by this slice

Adjacent MCP parity slice implemented on 2026-03-29:
- the consolidated route convenience surfaces are now exposed through MCP as:
  - `export_route_path_proposal`
  - `route_apply`
- input:
  - `export_route_path_proposal`: native project directory path, anchor/net
    UUIDs, accepted bounded candidate, optional accepted policy when the
    candidate is `authored-copper-graph`, and artifact `out` path
  - `route_apply`: native project directory path, anchor/net UUIDs, accepted
    bounded candidate, and optional accepted policy when the candidate is
    `authored-copper-graph`
- output:
  - the same JSON export/apply reports produced by the native CLI
- guardrails:
  - MCP parity remains a thin adapter over the bounded consolidated native
    route surfaces
  - no generalized M5 query parity is introduced by this slice

Adjacent native convenience-apply slice implemented on 2026-03-29:
- the accepted single-layer deterministic path family now also has a direct
  apply surface:
  - `project route-apply <dir> --net <uuid> --from-anchor <pad_uuid> --to-anchor <pad_uuid> --candidate route-path-candidate`
- input: persisted native board state only
- output:
  - one immediate `route_apply` report built from the current live
    deterministic single-layer full-path proposal
  - no intermediate artifact file is required
- guardrails:
  - no re-search beyond the accepted `route-path-candidate` query contract
  - the direct apply writes only the ordered selected `draw_track` sequence
    from the current live deterministic proposal

Adjacent native convenience-apply slice implemented on 2026-03-29:
- the accepted bounded single-via deterministic path family now also has a
  direct apply surface:
  - `project route-apply <dir> --net <uuid> --from-anchor <pad_uuid> --to-anchor <pad_uuid> --candidate route-path-candidate-via`
- input: persisted native board state only
- output:
  - one immediate `route_apply` report built from the current live
    deterministic single-via full-path proposal
  - no intermediate artifact file is required
- guardrails:
  - no re-search beyond the accepted `route-path-candidate-via` query contract
  - the direct apply writes only the ordered selected `draw_track` sequence;
    the reused via remains reused authored state and is never recreated

Adjacent native convenience-apply slice implemented on 2026-03-29:
- the accepted bounded two-via deterministic path family now also has a
  direct apply surface:
  - `project route-apply <dir> --net <uuid> --from-anchor <pad_uuid> --to-anchor <pad_uuid> --candidate route-path-candidate-two-via`
- input: persisted native board state only
- output:
  - one immediate `route_apply` report built from the current live
    deterministic two-via full-path proposal
  - no intermediate artifact file is required
- guardrails:
  - no re-search beyond the accepted `route-path-candidate-two-via` query
    contract
  - the direct apply writes only the ordered selected `draw_track` sequence;
    the reused via pair remains reused authored state and is never recreated

Adjacent native convenience-apply slice implemented on 2026-03-29:
- the accepted bounded three-via deterministic path family now also has a
  direct apply surface:
  - `project route-apply <dir> --net <uuid> --from-anchor <pad_uuid> --to-anchor <pad_uuid> --candidate route-path-candidate-three-via`
- input: persisted native board state only
- output:
  - one immediate `route_apply` report built from the current live
    deterministic three-via full-path proposal
  - no intermediate artifact file is required
- guardrails:
  - no re-search beyond the accepted `route-path-candidate-three-via` query
    contract
  - the direct apply writes only the ordered selected `draw_track` sequence;
    the reused via triple remains reused authored state and is never recreated

Adjacent native convenience-apply slice implemented on 2026-03-29:
- the remaining bounded ordinal via-path families now also have direct apply
  surfaces:
  - `project route-apply <dir> --net <uuid> --from-anchor <pad_uuid> --to-anchor <pad_uuid> --candidate route-path-candidate-four-via`
  - `project route-apply <dir> --net <uuid> --from-anchor <pad_uuid> --to-anchor <pad_uuid> --candidate route-path-candidate-five-via`
  - `project route-apply <dir> --net <uuid> --from-anchor <pad_uuid> --to-anchor <pad_uuid> --candidate route-path-candidate-six-via`
- input: persisted native board state only
- output:
  - one immediate `route_apply` report built from the current live
    deterministic ordinal-via full-path proposal for each accepted contract
  - no intermediate artifact file is required
- guardrails:
  - no re-search beyond the accepted ordinal-via query contracts
  - the direct apply writes only the ordered selected `draw_track` sequence;
    the full reused authored via chain remains reused state and is never
    recreated

Adjacent native convenience-apply slice implemented on 2026-03-29:
- the accepted deterministic authored-via-chain contract now also has a direct
  apply surface:
  - `project route-apply <dir> --net <uuid> --from-anchor <pad_uuid> --to-anchor <pad_uuid> --candidate route-path-candidate-authored-via-chain`
- input: persisted native board state only
- output:
  - one immediate `route_apply` report built from the current live
    deterministic authored-via-chain full-path proposal
  - no intermediate artifact file is required
- guardrails:
  - no re-search beyond the accepted authored-via-chain query contract
  - the direct apply writes only the ordered selected `draw_track` sequence;
    the reused authored via chain remains reused state and is never recreated

Adjacent native convenience-apply hardening slice implemented on 2026-03-29:
- `project route-apply` now parses `--candidate` from a bounded accepted value
  set instead of a free-form string
- input: CLI request only
- output:
  - parse-time rejection for unsupported direct-apply candidate names
  - explicit policy enforcement: `--policy` is required for `--candidate
    authored-copper-graph` and rejected for the other direct-apply candidates
- guardrails:
  - no hidden fallback from unknown candidate strings into later runtime
    dispatch
  - no silent acceptance of `--policy` on non-policy candidate families

Adjacent native convenience-export slice implemented on 2026-03-29:
- `project export-route-path-proposal <dir> --net <uuid> --from-anchor
  <pad_uuid> --to-anchor <pad_uuid> --candidate <accepted_candidate>
  [--policy <policy>] --out <path>` now provides one bounded convenience
  export surface for the completed write-capable route family
- input: persisted native board state only
- output:
  - one versioned `native_route_proposal_artifact` built from the current live
    deterministic proposal under the selected accepted candidate family
  - no contract-specific export command is required for the common path-family
    flow
- guardrails:
  - no new routing semantics beyond the already-accepted route-path candidate
    families
  - `--policy` is required for `--candidate authored-copper-graph` and
    rejected for the other generic export candidates

Adjacent native compatibility-warning slice implemented on 2026-03-30:
- the legacy `export-route-path-candidate-*` commands were briefly marked as
  deprecated compatibility wrappers before later removal
- input: legacy export command invocation only
- output:
  - help text names the preferred replacement
  - text-mode export output appends a deprecation note pointing users to
    `project export-route-path-proposal ... --candidate ...`
- guardrails:
  - JSON output remains unchanged
  - command behavior remains unchanged; only user guidance is added

Adjacent native convenience-apply slice implemented on 2026-03-29:
- the completed policy-selected authored-copper-graph family now also has a
  direct apply surface:
  - `project route-apply <dir> --net <uuid> --from-anchor <pad_uuid> --to-anchor <pad_uuid> --candidate authored-copper-graph --policy <policy>`
- input: persisted native board state only
- output:
  - one immediate `route_apply` report built from the current live
    deterministic authored-copper-graph proposal under the selected accepted
    bounded policy
  - no intermediate artifact file is required
- guardrails:
  - no new routing semantics beyond the already-accepted authored-copper-graph
    policy query family
  - because the current family reuses existing authored copper only, direct
    apply remains a validated no-op apply surface
  - no generalized direct-apply surface for the broader M5 proposal family is
    introduced by this slice

Next explicit contract selected on 2026-03-30:
- deterministic same-layer orthogonal dogleg path candidate
  `project query <dir> route-path-candidate --net <uuid> --from-anchor <pad_uuid> --to-anchor <pad_uuid> --candidate route-path-candidate-orthogonal-dogleg`
- input: persisted native board state only
- output:
  - current single-net source/target anchor pair
  - explicit status exactly one of:
    - `deterministic_path_found`
    - `no_path_under_current_authored_constraints`
  - when found, exactly one deterministic same-layer path with exactly one
    bend point chosen from the two canonical Manhattan corners between the
    anchors
- deterministic rule:
  - restrict candidates to the requested anchor layer only
  - evaluate the two canonical orthogonal corner orders
    `horizontal_then_vertical` then `vertical_then_horizontal`
  - accept a candidate only when both constituent segments are unblocked under
    the existing authored obstacle checks
  - select the first unblocked candidate in that explicit order
- non-goals:
  - no vias or layer transitions
  - no arbitrary corner search or ranking beyond the explicit two-corner order
  - no rip-up/reroute, negotiated routing, or push-shove
  - no invented constraints beyond the existing persisted obstacle-truth checks

Follow-on explanation surface selected on 2026-03-30:
- deterministic same-layer orthogonal dogleg explanation
  `project query <dir> route-path-candidate-explain --net <uuid> --from-anchor <pad_uuid> --to-anchor <pad_uuid> --candidate route-path-candidate-orthogonal-dogleg`
- input: persisted native board state only
- output:
  - current selected corner and corner order when found
  - or whether failure came from no same-layer dogleg candidate versus all
    dogleg candidates blocked
- guardrails:
  - no new routing semantics
  - no new obstacle interpretation beyond the accepted existing segment checks

Adjacent write-capable route-application slice implemented on 2026-03-30:
- deterministic route proposal artifact export for the accepted same-layer
  orthogonal dogleg contract
  - canonical surface: `project export-route-path-proposal <dir> --net <uuid> --from-anchor <pad_uuid> --to-anchor <pad_uuid> --candidate route-path-candidate-orthogonal-dogleg --out <path>`
  - reuses the same `project inspect-route-proposal-artifact <path>` and
    `project apply-route-proposal-artifact <dir> --artifact <path>` surfaces
- input: persisted native board state only
- output:
  - one versioned route-proposal artifact containing the ordered
    self-sufficient `draw_track` action sequence for the selected two-segment
    dogleg only
- guardrails:
  - no re-search beyond the accepted orthogonal dogleg query contract
  - no invented track width; width comes from persisted net-class facts only

Adjacent native convenience-apply slice implemented on 2026-03-30:
- the accepted deterministic same-layer orthogonal dogleg contract now also
  has a direct apply surface:
  - `project route-apply <dir> --net <uuid> --from-anchor <pad_uuid> --to-anchor <pad_uuid> --candidate route-path-candidate-orthogonal-dogleg`
- input: persisted native board state only
- output:
  - one immediate `route_apply` report built from the current live
    deterministic orthogonal dogleg proposal
  - no intermediate artifact file is required
- guardrails:
  - no re-search beyond the accepted orthogonal dogleg query contract
  - the direct apply writes only the ordered selected `draw_track` sequence

Next explicit contract selected on 2026-03-30:
- deterministic same-layer orthogonal two-bend path candidate
  `project query <dir> route-path-candidate --net <uuid> --from-anchor <pad_uuid> --to-anchor <pad_uuid> --candidate route-path-candidate-orthogonal-two-bend`
- input: persisted native board state only
- output:
  - current single-net source/target anchor pair
  - explicit status exactly one of:
    - `deterministic_path_found`
    - `no_path_under_current_authored_constraints`
  - when found, exactly one deterministic same-layer orthogonal path with
    exactly two bends and three segments
- deterministic rule:
  - restrict candidates to the requested anchor layer only
  - generate detour lines only from persisted board-outline and authored
    obstacle coordinates already present on that layer
  - order candidates by orientation family
    `horizontal_detour` before `vertical_detour`, then detour coordinate
    ascending
  - accept a candidate only when all three constituent segments are unblocked
    under the existing authored obstacle checks
  - select the first unblocked candidate in that explicit order
- non-goals:
  - no vias or layer transitions
  - no arbitrary grid search or invented coordinates
  - no negotiated routing, rip-up/reroute, or push-shove
  - no inferred constraints beyond the existing obstacle-truth checks

Follow-on explanation surface selected on 2026-03-30:
- deterministic same-layer orthogonal two-bend explanation
  `project query <dir> route-path-candidate-explain --net <uuid> --from-anchor <pad_uuid> --to-anchor <pad_uuid> --candidate route-path-candidate-orthogonal-two-bend`
- input: persisted native board state only
- output:
  - current selected path points and detour coordinate when found
  - or whether failure came from no same-layer two-bend candidate versus all
    two-bend candidates blocked
- guardrails:
  - no new routing semantics
  - no new obstacle interpretation beyond the accepted segment checks

Adjacent write-capable route-application slice implemented on 2026-03-30:
- deterministic route proposal artifact export for the accepted same-layer
  orthogonal two-bend contract
  - canonical surface: `project export-route-path-proposal <dir> --net <uuid> --from-anchor <pad_uuid> --to-anchor <pad_uuid> --candidate route-path-candidate-orthogonal-two-bend --out <path>`
  - reuses the same `project inspect-route-proposal-artifact <path>` and
    `project apply-route-proposal-artifact <dir> --artifact <path>` surfaces
- input: persisted native board state only
- output:
  - one versioned route-proposal artifact containing the ordered
    self-sufficient `draw_track` action sequence for the selected three-segment
    path only
- guardrails:
  - no re-search beyond the accepted orthogonal two-bend query contract
  - no invented track width; width comes from persisted net-class facts only

Adjacent native convenience-apply slice implemented on 2026-03-30:
- the accepted deterministic same-layer orthogonal two-bend contract now also
  has a direct apply surface:
  - `project route-apply <dir> --net <uuid> --from-anchor <pad_uuid> --to-anchor <pad_uuid> --candidate route-path-candidate-orthogonal-two-bend`
- input: persisted native board state only
- output:
  - one immediate `route_apply` report built from the current live
    deterministic orthogonal two-bend proposal
  - no intermediate artifact file is required
- guardrails:
  - no re-search beyond the accepted orthogonal two-bend query contract
  - the direct apply writes only the ordered selected `draw_track` sequence

Next explicit contract selected on 2026-03-30:
- deterministic same-layer orthogonal graph path candidate
  `project query <dir> route-path-candidate --net <uuid> --from-anchor <pad_uuid> --to-anchor <pad_uuid> --candidate route-path-candidate-orthogonal-graph`
- input: persisted native board state only
- output:
  - current single-net source/target anchor pair
  - explicit status exactly one of:
    - `deterministic_path_found`
    - `no_path_under_current_authored_constraints`
  - when found, exactly one deterministic same-layer orthogonal multi-segment
    path over persisted coordinate lines only
- deterministic rule:
  - restrict candidates to the requested anchor layer only
  - build x/y coordinate lines only from persisted board-outline vertices,
    requested anchor coordinates, and authored object coordinates already
    present on that layer
  - build a same-layer orthogonal graph from clear spans between those
    persisted-coordinate intersections only
  - rank graph paths by bend count ascending, then segment count ascending,
    then point-sequence coordinate ascending
- non-goals:
  - no vias or layer transitions
  - no invented routing grid or inferred coordinates
  - no negotiated routing, rip-up/reroute, or push-shove
  - no inferred constraints beyond the existing segment obstacle checks

Follow-on explanation surface selected on 2026-03-30:
- deterministic same-layer orthogonal graph explanation
  `project query <dir> route-path-candidate-explain --net <uuid> --from-anchor <pad_uuid> --to-anchor <pad_uuid> --candidate route-path-candidate-orthogonal-graph`

Adjacent write-capable route-application slice implemented on 2026-03-30:
- deterministic route proposal artifact export for the accepted same-layer
  orthogonal graph contract
  - canonical surface: `project export-route-path-proposal <dir> --net <uuid> --from-anchor <pad_uuid> --to-anchor <pad_uuid> --candidate route-path-candidate-orthogonal-graph --out <path>`

Adjacent native convenience-apply slice implemented on 2026-03-30:
- the accepted deterministic same-layer orthogonal graph contract now also has
  a direct apply surface:
  - `project route-apply <dir> --net <uuid> --from-anchor <pad_uuid> --to-anchor <pad_uuid> --candidate route-path-candidate-orthogonal-graph`
- the orthogonal graph report and explanation surfaces now also expose the
  selected path cost explicitly as:
  - `bend_count`
  - `segment_count`
  - `point_count`
- the orthogonal graph route proposal artifact/apply lane now also preserves
  that selected path cost in exported action metadata and inspection/apply
  summaries

Next explicit contract selected on 2026-03-30:
- deterministic one-authored-via orthogonal graph path candidate
  `project query <dir> route-path-candidate --net <uuid> --from-anchor <pad_uuid> --to-anchor <pad_uuid> --candidate route-path-candidate-orthogonal-graph-via`
- input: persisted native board state only
- output:
  - current single-net source/target anchor pair on different copper layers
  - explicit status exactly one of:
    - `deterministic_path_found`
    - `no_path_under_current_authored_constraints`
  - when found, exactly one deterministic cross-layer path that reuses one
    authored target-net via and uses persisted-coordinate orthogonal graph
    search on each side of that via
- deterministic rule:
  - candidate transition set is limited to authored target-net vias only
  - matching vias are ordered by ascending via UUID and must exactly match the
    requested anchor boundary layers
  - each same-layer side reuses the accepted persisted-coordinate orthogonal
    graph search rule without inventing coordinates or creating vias

Follow-on explanation surface selected on 2026-03-30:
- deterministic one-authored-via orthogonal graph explanation
  `project query <dir> route-path-candidate-explain --net <uuid> --from-anchor <pad_uuid> --to-anchor <pad_uuid> --candidate route-path-candidate-orthogonal-graph-via`

Adjacent write-capable route-application slice implemented on 2026-03-30:
- deterministic route proposal artifact export for the accepted
  one-authored-via orthogonal graph contract
  - canonical surface: `project export-route-path-proposal <dir> --net <uuid> --from-anchor <pad_uuid> --to-anchor <pad_uuid> --candidate route-path-candidate-orthogonal-graph-via --out <path>`

Adjacent native convenience-apply slice implemented on 2026-03-30:
- the accepted deterministic one-authored-via orthogonal graph contract now
  also has a direct apply surface:
  - `project route-apply <dir> --net <uuid> --from-anchor <pad_uuid> --to-anchor <pad_uuid> --candidate route-path-candidate-orthogonal-graph-via`

Next explicit contract selected on 2026-03-30:
- deterministic two-authored-via orthogonal graph path candidate
  `project query <dir> route-path-candidate --net <uuid> --from-anchor <pad_uuid> --to-anchor <pad_uuid> --candidate route-path-candidate-orthogonal-graph-two-via`
- input: persisted native board state only
- output:
  - current single-net source/target anchor pair on different copper layers
  - explicit status exactly one of:
    - `deterministic_path_found`
    - `no_path_under_current_authored_constraints`
  - when found, exactly one deterministic cross-layer path that reuses exactly
    two authored target-net vias and uses persisted-coordinate orthogonal graph
    search on each of the three layer-side segments
- deterministic rule:
  - candidate transition set is limited to authored target-net via pairs only
  - matching via pairs are ordered by ascending `(via_a_uuid, via_b_uuid)` and
    must connect the requested anchor layers through one intermediate copper
    layer
  - each layer-side segment reuses the accepted persisted-coordinate
    orthogonal graph search rule without inventing coordinates or creating vias

Follow-on explanation surface selected on 2026-03-30:
- deterministic two-authored-via orthogonal graph explanation
  `project query <dir> route-path-candidate-explain --net <uuid> --from-anchor <pad_uuid> --to-anchor <pad_uuid> --candidate route-path-candidate-orthogonal-graph-two-via`

Adjacent write-capable route-application slice implemented on 2026-03-30:
- deterministic route proposal artifact export for the accepted
  two-authored-via orthogonal graph contract
  - canonical surface: `project export-route-path-proposal <dir> --net <uuid> --from-anchor <pad_uuid> --to-anchor <pad_uuid> --candidate route-path-candidate-orthogonal-graph-two-via --out <path>`

Adjacent native convenience-apply slice implemented on 2026-03-30:
- the accepted deterministic two-authored-via orthogonal graph contract now
  also has a direct apply surface:
  - `project route-apply <dir> --net <uuid> --from-anchor <pad_uuid> --to-anchor <pad_uuid> --candidate route-path-candidate-orthogonal-graph-two-via`

Completed bounded authored-via orthogonal graph family on 2026-03-30:
- the same deterministic persisted-coordinate orthogonal graph contract now
  also spans the remaining bounded authored-via sequence:
  - `route-path-candidate-orthogonal-graph-three-via`
  - `route-path-candidate-orthogonal-graph-four-via`
  - `route-path-candidate-orthogonal-graph-five-via`
  - `route-path-candidate-orthogonal-graph-six-via`
- each contract keeps the same boundary:
  - persisted native board state only
  - reuse exactly the stated authored via count
  - deterministic tuple ordering inherited from the matching ordinal via
    selector
  - one orthogonal graph search per layer-side segment only, with each segment
    using the accepted bend-count then segment-count ranking rule
  - no via creation, no invented coordinate grid, no push-shove
- each report/explanation segment now also records the selected graph-path
  cost explicitly as `bend_count`, `segment_count`, and `point_count`
- the same-layer orthogonal-graph direct query and explain surfaces now also
  expose `segment_evidence`, aligning their reported layer-side path
  structure with export/inspect/revalidate
- the same cost now also survives route proposal export as
  `selected_path_bend_count`, alongside the existing selected path point and
  segment counts
- `export-route-path-proposal` now also returns the recorded
  orthogonal-graph segment breakdown directly in its report, so callers do
  not need to reopen the artifact just to inspect the layer-side path shape
- orthogonal-graph artifact apply now diagnoses ranked-path drift explicitly
  as candidate availability changes, deterministic cost-winner changes, or
  same-rank geometry changes before refusing stale apply
- `inspect-route-proposal-artifact` now also exposes the recorded
  orthogonal-graph segment breakdown, including each layer-side segment's
  bend, point, and track-action counts
- the same artifact lane now also has a non-mutating machine-readable
  revalidation surface:
  - `project revalidate-route-proposal-artifact <dir> --artifact <path>`
  - orthogonal-graph revalidation now also includes segment-level ranked-path
    evidence, so callers can compare each layer-side segment's recorded and
    live bend/point/track-action facts
- each of those contracts now also has:
  - the paired generic explanation surface under
    `route-path-candidate-explain`
  - route proposal artifact export through
    `project export-route-path-proposal`
  - direct native apply through `project route-apply`

Adjacent route-selection slice implemented on 2026-03-30:
- deterministic bounded route proposal selection
  - canonical surface: `project route-proposal <dir> --net <uuid> --from-anchor <pad_uuid> --to-anchor <pad_uuid>`
- input: persisted native board state only
- output:
  - explicit status exactly one of:
    - `deterministic_route_proposal_selected`
    - `no_route_proposal_under_current_authored_constraints`
  - one selected accepted candidate family when available
  - the deterministic candidate order used
  - per-candidate availability or rejection notes for the bounded selector lane
- deterministic rule:
  - evaluate the bounded accepted candidate family order:
    `route-path-candidate`
    `route-path-candidate-orthogonal-dogleg`
    `route-path-candidate-orthogonal-two-bend`
    `route-path-candidate-orthogonal-graph`
    `authored-copper-plus-one-gap`
    `route-path-candidate-via`
    `route-path-candidate-two-via`
    `route-path-candidate-three-via`
    `route-path-candidate-four-via`
    `route-path-candidate-five-via`
    `route-path-candidate-six-via`
    `route-path-candidate-authored-via-chain`
    `route-path-candidate-orthogonal-graph-via`
    `route-path-candidate-orthogonal-graph-two-via`
    `route-path-candidate-orthogonal-graph-three-via`
    `route-path-candidate-orthogonal-graph-four-via`
    `route-path-candidate-orthogonal-graph-five-via`
    `route-path-candidate-orthogonal-graph-six-via`
  - select the first successful family in that exact order
- non-goals:
  - no cross-family scoring beyond the explicit ordered selector
  - no automatic export/apply side effects
  - no MCP parity reopening
  - no broad autorouting or ranking semantics

Adjacent selected-proposal write lane implemented on 2026-03-30:
- deterministic export/apply surfaces for the currently selected bounded route proposal
  - canonical export surface:
    `project export-route-proposal <dir> --net <uuid> --from-anchor <pad_uuid> --to-anchor <pad_uuid> --out <path>`
  - canonical direct-apply surface:
    `project route-apply-selected <dir> --net <uuid> --from-anchor <pad_uuid> --to-anchor <pad_uuid>`
- input: persisted native board state only
- output:
  - reuse the exact accepted selector order from `project route-proposal`
  - export emits one versioned route-proposal artifact for the selected family
  - direct apply materializes the currently selected proposal without requiring
    the caller to restate the chosen candidate family
- deterministic rule:
  - run the accepted `project route-proposal` selector first
  - if a proposal is selected, export/apply that exact family only
  - if no proposal is selected, fail without mutating state
- non-goals:
  - no new ranking or selector heuristics
  - no bypass of artifact drift checks
  - no MCP parity reopening

Poor candidates for the first `M5` slice:
- full autorouter
- broad placement engine
- AI proposal layer
- anything that bundles several layout semantics into one step
