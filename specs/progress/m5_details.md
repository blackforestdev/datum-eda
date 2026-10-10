# M5 routing-kernel checkpoint details

> Detailed evidence relocated from `specs/PROGRESS.md`. The tracker remains the implementation-status entry point; the generated Active Frontier alone selects work. Dated history below does not grant execution or establish a fresh PASS.

## M5 Opening Charter

Status: [~] Routing-kernel scope complete; closure review pending
- Authority for the proposed opening charter and entry criteria:
  `specs/progress/m5_opening.md`
- Recommended M5 focus: deterministic layout-kernel groundwork from persisted
  native board state, opening with one narrow routing/constraint slice rather
  than broad placement/routing ambition.
- Governance narrowing (2026-03-30):
  - for milestone closure, `M5` is now interpreted as the deterministic
    persisted-state routing-kernel milestone recorded in the opening charter
    and current frontier below
  - placement-kernel and placement/routing co-optimization work are deferred
    to a later reopened milestone/slice and are not required for `M5` closure
  - `M6` may open from the completed routing-kernel substrate once closure
    review accepts this narrowed milestone scope
- M5 must not inherit M4 parity work by default; new slices require an
  explicit layout-kernel contract and acceptance criteria.
- MCP implementation parity remains deferred unless explicitly reopened, but
  deferred parity tracking for newer native M4/M5 slices must stay current in
  `specs/MCP_API_SPEC.md`.
- A narrow MCP parity exception is now live for the policy-selected
  authored-copper-graph proposal export surface; broader M5 MCP query/apply
  parity remains deferred.
- That narrow exception now covers the full route-proposal artifact lifecycle
  for the policy-selected authored-copper-graph family: export, inspect, and
  apply.
- Current M5 checkpoint chain (routing-kernel read/query lane):
  - `project query <dir> routing-substrate`
  - `project query <dir> route-preflight --net <uuid>`
  - `project query <dir> route-corridor --net <uuid>`
  - canonical route query surface: `project query <dir> route-path-candidate --net <uuid> --from-anchor <pad_uuid> --to-anchor <pad_uuid> --candidate <accepted_candidate> [--policy <policy>]`
  - canonical route explanation surface: `project query <dir> route-path-candidate-explain --net <uuid> --from-anchor <pad_uuid> --to-anchor <pad_uuid> --candidate <accepted_candidate> [--policy <policy>]`
  - contract-specific `route-path-candidate-*` and
    `route-path-candidate-*-explain` commands are now compatibility wrappers
    around those bounded generic surfaces
  - the remaining legacy wrapper implementations now also dispatch through the
    same shared generic candidate/policy executor path as the canonical
    surfaces, reducing wrapper-only branching ahead of removal
- Current M5 routing expansion (persisted-via reuse lane):
  - fixed-arity via reuse was proven through bounded ordinal slices and is no
    longer the preferred growth model
  - current generalized contract is `project query <dir>
    route-path-candidate --net <uuid> --from-anchor <pad_uuid> --to-anchor
    <pad_uuid> --candidate route-path-candidate-authored-via-chain`
  - current adjacent observability surface is `project query <dir>
    route-path-candidate-explain --net <uuid> --from-anchor <pad_uuid>
    --to-anchor <pad_uuid> --candidate route-path-candidate-authored-via-chain`
  - a same-layer synthesis slice now also exists via `project query <dir>
    route-path-candidate --net <uuid> --from-anchor <pad_uuid> --to-anchor
    <pad_uuid> --candidate route-path-candidate-orthogonal-dogleg`, with the
    paired explanation surface under `route-path-candidate-explain`
  - that same same-layer synthesis lane now also covers `project query <dir>
    route-path-candidate --net <uuid> --from-anchor <pad_uuid> --to-anchor
    <pad_uuid> --candidate route-path-candidate-orthogonal-two-bend`, with the
    paired explanation surface under `route-path-candidate-explain`
  - that same same-layer synthesis lane now also covers `project query <dir>
    route-path-candidate --net <uuid> --from-anchor <pad_uuid> --to-anchor
    <pad_uuid> --candidate route-path-candidate-orthogonal-graph`, with the
    paired explanation surface under `route-path-candidate-explain`
  - a first bounded cross-layer graph-search lane now also exists via
    `project query <dir> route-path-candidate --net <uuid> --from-anchor
    <pad_uuid> --to-anchor <pad_uuid> --candidate
    route-path-candidate-orthogonal-graph-via`, with the paired explanation
    surface under `route-path-candidate-explain`
  - that same bounded cross-layer graph-search lane now also covers `project
    query <dir> route-path-candidate --net <uuid> --from-anchor <pad_uuid>
    --to-anchor <pad_uuid> --candidate
    route-path-candidate-orthogonal-graph-two-via`, with the paired
    explanation surface under `route-path-candidate-explain`
  - that same bounded cross-layer graph-search lane now also covers the
    remaining authored-via graph sequence via `project query <dir>
    route-path-candidate --net <uuid> --from-anchor <pad_uuid> --to-anchor
    <pad_uuid> --candidate
    route-path-candidate-orthogonal-graph-three-via|route-path-candidate-orthogonal-graph-four-via|route-path-candidate-orthogonal-graph-five-via|route-path-candidate-orthogonal-graph-six-via`,
    with the paired explanation surface under `route-path-candidate-explain`
  - the shared orthogonal graph selector now uses an explicit deterministic
    cost rule across that whole family: bend count ascending, then segment
    count ascending, then point-sequence coordinate ascending
  - the same orthogonal graph query and explanation reports now expose that
    selected path cost directly as bend/segment/point counts on the returned
    path or per-segment path data
  - the same orthogonal graph route proposal artifact lane now preserves the
    selected bend count in exported actions and exposes it again through
    artifact inspection/apply reporting
  - `export-route-path-proposal` now also returns the recorded
    orthogonal-graph layer-segment bend/point/track-action breakdown
  - the same-layer orthogonal-graph `route-path-candidate` and
    `route-path-candidate-explain` reports now also return `segment_evidence`
    so direct query output matches the artifact lane vocabulary
  - the same-layer orthogonal-graph candidate and explain surfaces now also
    share one internal graph-search spine, keeping the public reports
    unchanged while consolidating the preflight/search/segment-evidence path
  - the one-authored-via orthogonal-graph candidate and explain surfaces now
    also share one internal via-selection spine that reuses the same
    orthogonal-graph layer-search structure without changing the public
    reports
  - the two-authored-via orthogonal-graph candidate and explain surfaces now
    also share one internal pair-selection spine that reuses the same
    orthogonal-graph layer-search structure without changing the public
    reports
  - orthogonal-graph artifact apply now reports whether stale proposals
    drifted because candidate availability changed, the deterministic ranked
    winner changed, or same-rank geometry changed
  - `inspect-route-proposal-artifact` now also returns the recorded
    orthogonal-graph layer-segment bend/point/track-action breakdown
  - the same lane now also has `project revalidate-route-proposal-artifact
    <dir> --artifact <path>` so callers can read that drift classification
    and live/recorded path summaries without applying
  - that revalidation report now also carries segment-level orthogonal-graph
    evidence so stale proposals can show which layer-side segment changed and
    how its bend/point/track-action facts differ live
- Current M5 existing-copper readback lane:
  - deterministic authored-copper graph path queries now exist in
    increasingly filtered/readback-focused forms recorded in
    `specs/progress/m5_opening.md`
  - that same generalized query surface also covers the accepted
    authored-copper-graph family when `--candidate authored-copper-graph
    --policy <policy>` is selected
  - that same generalized explanation surface also covers the accepted
    authored-copper-graph family when `--candidate authored-copper-graph
    --policy <policy>` is selected
  - accepted bounded policies are `plain`, `zone_aware`, `obstacle_aware`,
    `zone_obstacle_aware`, `zone_obstacle_topology_aware`, and
    `zone_obstacle_topology_layer_balance_aware`
  - the plus-one-gap bridge is now reached canonically through `project query
    <dir> route-path-candidate --net <uuid> --from-anchor <pad_uuid>
    --to-anchor <pad_uuid> --candidate authored-copper-plus-one-gap`
  - a first write-capable bridge now exists for that accepted query contract:
    `project export-route-path-proposal <dir> --net <uuid> --from-anchor
    <pad_uuid> --to-anchor <pad_uuid> --candidate
    authored-copper-plus-one-gap --out <path>`,
    `project inspect-route-proposal-artifact <path>`, and
    `project apply-route-proposal-artifact <dir> --artifact <path>`
  - the same route-proposal artifact lane now also covers the accepted
    completed write-capable family, now canonically via `project
    export-route-path-proposal <dir> --net <uuid> --from-anchor <pad_uuid>
    --to-anchor <pad_uuid> --candidate <accepted_candidate> [--policy
    <policy>] --out <path>`, spanning the accepted single-layer, via-path, and
    authored-copper-graph policy-selected contracts
  - a native direct convenience apply surface now also exists for that same
    accepted single-layer path contract via `project route-apply <dir> --net
    <uuid> --from-anchor <pad_uuid> --to-anchor <pad_uuid> --candidate
    route-path-candidate`
  - a native direct convenience apply surface now also exists for that same
    accepted bounded single-via contract via `project route-apply <dir> --net
    <uuid> --from-anchor <pad_uuid> --to-anchor <pad_uuid> --candidate
    route-path-candidate-via`
  - a native direct convenience apply surface now also exists for that same
    accepted bounded two-via contract via `project route-apply <dir> --net
    <uuid> --from-anchor <pad_uuid> --to-anchor <pad_uuid> --candidate
    route-path-candidate-two-via`
  - a native direct convenience apply surface now also exists for that same
    accepted bounded three-via contract via `project route-apply <dir> --net
    <uuid> --from-anchor <pad_uuid> --to-anchor <pad_uuid> --candidate
    route-path-candidate-three-via`
  - the same direct convenience apply lane now also covers the remaining
    bounded ordinal via contracts via `project route-apply <dir> --net <uuid>
    --from-anchor <pad_uuid> --to-anchor <pad_uuid> --candidate
    route-path-candidate-four-via|route-path-candidate-five-via|route-path-candidate-six-via`
  - that same direct convenience apply lane now also covers the accepted
    deterministic authored-via-chain contract via `project route-apply <dir>
    --net <uuid> --from-anchor <pad_uuid> --to-anchor <pad_uuid> --candidate
    route-path-candidate-authored-via-chain`
  - that same direct convenience apply lane now also covers the accepted
    same-layer orthogonal dogleg contract via `project route-apply <dir> --net
    <uuid> --from-anchor <pad_uuid> --to-anchor <pad_uuid> --candidate
    route-path-candidate-orthogonal-dogleg`
  - that same direct convenience apply lane now also covers the accepted
    same-layer orthogonal two-bend contract via `project route-apply <dir> --net
    <uuid> --from-anchor <pad_uuid> --to-anchor <pad_uuid> --candidate
    route-path-candidate-orthogonal-two-bend`
  - that same direct convenience apply lane now also covers the accepted
    same-layer orthogonal graph contract via `project route-apply <dir> --net
    <uuid> --from-anchor <pad_uuid> --to-anchor <pad_uuid> --candidate
    route-path-candidate-orthogonal-graph`
  - that same direct convenience apply lane now also covers the accepted
    one-authored-via orthogonal graph contract via `project route-apply <dir>
    --net <uuid> --from-anchor <pad_uuid> --to-anchor <pad_uuid> --candidate
    route-path-candidate-orthogonal-graph-via`
  - that same direct convenience apply lane now also covers the accepted
    two-authored-via orthogonal graph contract via `project route-apply <dir>
    --net <uuid> --from-anchor <pad_uuid> --to-anchor <pad_uuid> --candidate
    route-path-candidate-orthogonal-graph-two-via`
  - that same direct convenience apply lane now also covers the remaining
    authored-via orthogonal graph sequence via `project route-apply <dir>
    --net <uuid> --from-anchor <pad_uuid> --to-anchor <pad_uuid> --candidate
    route-path-candidate-orthogonal-graph-three-via|route-path-candidate-orthogonal-graph-four-via|route-path-candidate-orthogonal-graph-five-via|route-path-candidate-orthogonal-graph-six-via`
  - a bounded convenience export surface now also exists for the completed
    write-capable route family via `project export-route-path-proposal <dir>
    --net <uuid> --from-anchor <pad_uuid> --to-anchor <pad_uuid> --candidate
    <accepted_candidate> [--policy <policy>] --out <path>`
  - `project route-apply` now parses `--candidate` from a bounded accepted
    value set instead of a free-form string, and enforces `--policy` only for
    `--candidate authored-copper-graph`
  - a native direct convenience apply surface now also exists for that same
    policy-selected family via `project route-apply <dir> --net <uuid>
    --from-anchor <pad_uuid> --to-anchor <pad_uuid> --candidate
    authored-copper-graph --policy <policy>`
  - the bounded convenience export surface is now exposed through MCP as
    `export_route_path_proposal`
  - the bounded direct route-apply surface is now exposed through MCP as
    `route_apply`
  - the matching generic artifact follow-up surfaces are now exposed through
    MCP as `inspect_route_proposal_artifact`,
    `revalidate_route_proposal_artifact`, and
    `apply_route_proposal_artifact`
  - `specs/PROGRESS.md` tracks only the checkpoint/frontier; detailed per-slice
    history stays in `specs/progress/m5_opening.md`
- Current M5 frontier:
  - deterministic persisted-state layout-kernel/routing queries continue under
    explicit contract selection from `specs/progress/m5_opening.md`
  - a bounded native route selector now exists via `project route-proposal
    <dir> --net <uuid> --from-anchor <pad_uuid> --to-anchor <pad_uuid>`,
    selecting the first successful family from the explicit accepted candidate
    order recorded in `specs/progress/m5_opening.md`
  - that same selector now also feeds a selected-proposal write lane via
    `project export-route-proposal <dir> --net <uuid> --from-anchor
    <pad_uuid> --to-anchor <pad_uuid> --out <path>` and
    `project route-apply-selected <dir> --net <uuid> --from-anchor
    <pad_uuid> --to-anchor <pad_uuid>`
  - that same selector now also has a bounded explanation surface via
    `project route-proposal-explain <dir> --net <uuid> --from-anchor
    <pad_uuid> --to-anchor <pad_uuid>`
  - that same selector now also accepts a bounded deterministic profile set
    via `--profile default|authored-copper-priority`; the default order is
    unchanged, while `authored-copper-priority` prepends the accepted
    authored-copper-graph policy family ahead of the existing default family
    order without introducing scoring
  - a narrow MCP parity reopening now covers the selector lane via
    `route_proposal`, `route_proposal_explain`, `export_route_proposal`, and
    `route_apply_selected`
  - the first artifact/export/apply write lane now covers:
    the accepted plus-one-gap bridge, the accepted single-layer
    `route-path-candidate` contract, and the accepted bounded single-via
    `route-path-candidate-via` contract, and the accepted bounded two-via
    `route-path-candidate-two-via` contract, and the remaining bounded
    `three`/`four`/`five`/`six`-via plus `authored-via-chain` contracts, and
    the accepted same-layer orthogonal dogleg contract, and the accepted
    same-layer orthogonal two-bend contract, and
    the accepted zone-aware and zone-obstacle-aware existing-copper graph
    reuse contracts, and the accepted topology-aware plus layer-balance-aware
    topology-aware zone-obstacle-aware existing-copper graph reuse contracts,
    and the accepted obstacle-aware existing-copper graph reuse contract, and
    the policy-selected authored-copper graph family over the accepted bounded
    policy set
  - new slices must still avoid broad autorouting semantics, invented
    constraints, and untracked MCP drift
  - closure interpretation:
    - this frontier is the intended `M5` closure target under the narrowed
      routing-kernel milestone definition above
    - remaining placement-kernel work is explicitly out of scope for `M5`
      closure
