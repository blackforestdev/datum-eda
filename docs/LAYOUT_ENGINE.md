# Layout Engine — Constraint-Formalized Placement and Routing

## Thesis

Placement and routing are one coupled optimization problem, not two
sequential stages. The industry treats them as separate tools because
legacy architectures lacked a unified constraint model. In practice,
designers iterate between placement and routing continuously — placement
determines routability, routing reveals placement problems.

This engine unifies placement and routing as two modes of the same
constraint solver, sharing the same design space, constraint model,
objective function, and proposal/review workflow.

The differentiator is not novel algorithms. Classical techniques remain
the core solvers — force-directed placement, A* pathfinding, negotiated
reroute. The differentiator is:

1. A unified constraint model for both placement and routing, derived
   from the engine's rule system — not bolted on after.
2. Co-optimization: the placement solver considers routability, the
   routing solver can request placement adjustments.
3. An AI policy layer for intent translation, strategy selection,
   circuit recognition, and review — not "AI is the solver."
4. A proposal/review workflow where no change commits without explicit
   acceptance, enabling human-in-the-loop repairability.

Traditional algorithms are not replaced. They are subsumed into a
properly-formalized constraint pipeline with placement/routing
co-optimization and an AI strategy layer on top.

Current accepted M5 kernel boundary stays narrower than the long-term thesis:
persisted-state `routing-substrate`, `route-preflight`, `route-corridor`, and
single-layer `route-path-candidate` are deterministic read-only
routing-kernel primitives, not full router behavior. `route-path-candidate`
selects the first unblocked matching corridor span in corridor report order
(candidate copper layer order, then pair index) and emits that selected span
geometry directly. `route-path-candidate-explain` is the paired read-only
explanation surface for that contract, reporting selected-span or blocked/no-
match reasons without adding new path semantics. This accepted chain is the
current intentional M5 checkpoint boundary; any next routing-semantic contract
should be opened only by a separate explicit planning decision.
`route-path-candidate-via` is the next reopened M5 slice and stays narrow as
well: it may reuse one already-authored persisted via under an explicit
ascending-UUID selection rule, but it does not create vias, infer transition
permissions, or open general multilayer routing.
`route-path-candidate-two-via` extends that same contract family by allowing
exactly two already-authored persisted vias under an explicit ascending
`(via_a_uuid, via_b_uuid)` rule when they connect the requested anchor layers
through one intermediate copper layer. It still does not create vias, infer
transition permissions, or open free multilayer search.
`route-path-candidate-two-via-explain` is the paired read-only explanation
surface for that two-via contract, reporting the selected via pair when found
or whether failure came from no matching authored via pair versus all matching
via pairs blocked, using only existing two-via/path facts without adding new
routing semantics.
`route-path-candidate-via-explain` is the paired read-only explanation surface
for that via contract, reporting the selected via when found or whether failure
came from no matching authored via versus all matching vias blocked, using only
existing via/path blockage facts without adding new transition semantics.

---

## 1. Problem Formulation

### Design Space
```
Ω = Board_Area × Layers

Where:
  Board_Area: 2D polygon (board outline minus keepouts)
  Layers: ordered set of copper layers from stackup

Coordinate system: nanometers (i64) for storage and final geometry.
Search algorithms use adaptive representations — NOT the nanometer
grid directly. See §2.1 for discretization strategy.
```

### What the Solver Controls

The solver optimizes two coupled sets of variables:

```
Placement variables (per component):
  position: (x, y)    — center of footprint
  rotation: angle      — 0, 90, 180, 270 (or continuous)
  layer: Top | Bottom  — component side

Routing variables (per net):
  paths: Vec<TrackSegment>  — copper traces
  vias: Vec<Via>            — layer transitions
  layer_assignment: per-segment layer choice
```

These are coupled because pad positions (routing inputs) are determined
by component positions (placement outputs). Moving a component invalidates
its connected routes. The solver can adjust both simultaneously.

### Boundary Conditions
```
Fixed:
  Board outline (non-negotiable geometry)
  Mounting holes (fixed positions)
  Connectors (typically edge-constrained)
  Pre-placed components (user-locked positions)

Semi-fixed:
  Components the user has roughly placed but not locked
  (solver can refine position within a tolerance)

Free:
  Unplaced components (solver determines position)
  All routing (solver determines paths)
```

### Required Connectivity (Net Topology)
```
For each net N:
  pins(N) = { pad_1, pad_2, ..., pad_k }

  Requirement: all pins in N must be connected by
  continuous copper paths (tracks + vias).

  This is a Steiner tree problem per net:
  find the minimum-cost tree spanning all pins,
  where "cost" is defined by the objective function.
```

### Constraint Set
Every constraint is derived from the engine's rule system. The solver
consumes resolved constraints — the rule query engine has already
evaluated which rules apply to which objects.

#### Placement Constraints

```
Physical (hard):
  no_overlap(comp_a, comp_b): courtyard clearance between components
  within_outline(comp): component must be inside board boundary
  assembly_clearance(comp_a, comp_b): pick-and-place machine minimum gap

Grouping (soft → hard based on priority):
  group_proximity(group, max_spread):
    Components in group must fit within bounding box of max_spread.
    Only applies to authoritative groups (explicit or hierarchical).
    Advisory groups generate proximity suggestions, not constraints.

  group_separation(group_a, group_b, min_distance):
    Centroids of two groups must be at least min_distance apart.
    Typical use: power vs. analog, digital vs. RF.

Pin Proximity:
  pin_proximity(comp_a, pin_a, comp_b, pin_b, max_distance):
    Specific pin-to-pin distance constraint.
    Typical use: decoupling cap pad to IC power pin.
    Can be auto-generated from netlist (caps on power nets → nearest IC).

Thermal:
  thermal_separation(hot_comps, sensitive_comps, min_distance):
    Components with high power dissipation must be separated from
    temperature-sensitive components.
    Hot components identified from parametric data (power rating).

Orientation:
  uniform_orientation(group):
    All components in group have the same rotation.
    Typical use: passive arrays for wave solder.

  symmetric_placement(comp_a, comp_b, axis):
    Two components placed symmetrically about an axis.
    Typical use: matched transistor pairs (DOA2526 Q1/Q2).

Signal Flow:
  ordered_placement(group, direction):
    Components placed in netlist-order along a direction.
    Typical use: filter stages, signal chain.
```

#### Routing Constraints

```
Per-Net Constraints:
  width_min(net, layer): i64        — minimum track width
  width_preferred(net, layer): i64  — preferred track width
  width_max(net, layer): i64        — maximum track width
  layer_set(net): Set<LayerId>      — allowed layers for this net
  via_allowed(net): bool            — whether vias are permitted
  via_drill(net): i64               — via drill diameter
  via_diameter(net): i64            — via pad diameter

Per-Net-Pair Constraints (binary):
  clearance(net_a, net_b, layer): i64  — minimum copper-to-copper distance
  clearance(net, keepout): i64         — minimum distance to keepout region

Impedance Constraints:
  impedance_target(net, layer): Option<f64>  — target Z₀ in ohms
  impedance_tolerance(net): f64              — acceptable deviation
  → Resolved to width by stackup + material properties BEFORE routing

Differential Pair Constraints:
  diffpair_gap(pair, layer): i64     — edge-to-edge spacing
  diffpair_skew_max(pair): i64      — maximum length mismatch within pair

Length Matching Constraints:
  length_target(group): Option<i64>  — target routed length
  length_tolerance(group): i64       — acceptable deviation from target
  length_reference(group): NetId     — reference net (match to its length)

Global Constraints:
  max_vias(net): Option<u32>         — via budget per net
  no_route_regions: Vec<Polygon>     — keepout areas per layer
  preferred_direction(layer): Option<Axis>  — X or Y routing preference
```

### Unified Objective Function

Placement and routing share one objective. Placement minimizes
*estimated* routing cost. Routing minimizes *actual* routing cost.
The same function drives both, evaluated at different fidelity levels.

```
Minimize:
  α₁ · total_wire_length        (estimated during placement, actual during routing)
  + α₂ · total_via_count        (estimated during placement, actual during routing)
  + α₃ · placement_constraint_penalty  (grouping, separation, proximity, thermal)
  + α₄ · routing_constraint_penalty    (clearance, width, impedance, length)
  + α₅ · congestion_penalty     (routing density exceeding capacity)
  + α₆ · crosstalk_coupling     (parallel run length of sensitive nets)

Subject to (hard — must satisfy):
  No component overlap (courtyard clearance)
  All components within board outline
  All connectivity requirements satisfied
  All clearance rules satisfied
  All width rules satisfied

Subject to (soft — penalized, not fatal):
  Impedance within tolerance
  Length matching within tolerance
  Group proximity targets
  Group separation targets
  Orientation uniformity

Where:
  α₁..α₆ are tunable weights.
  Default profiles: "general", "high_speed", "power", "mixed_signal"
  User can adjust per design.
```

During placement, wire length and congestion are estimated from a
connectivity-weighted distance metric (half-perimeter wire length
of net bounding box — standard HPWL estimate from VLSI placement).
During routing, they are measured from actual geometry.

---

## 2. Solver Architecture

### Overview: Three Phases with Feedback

```
  ┌──────────────┐     ┌──────────────┐     ┌──────────────┐
  │   Phase 1    │────→│   Phase 2    │────→│   Phase 3    │
  │  Placement   │     │   Global     │     │  Detailed    │
  │              │←────│   Routing    │←────│   Routing    │
  └──────────────┘     └──────────────┘     └──────────────┘
        ↑                                         │
        └─────────────────────────────────────────┘
                    feedback loop

  Phase 1 produces component positions.
  Phase 2 assigns nets to layers and estimates paths.
  Phase 3 computes exact track geometry.

  Feedback: if Phase 3 fails (congestion, clearance), it can
  request Phase 2 to reassign layers or Phase 1 to adjust
  placement. This is the co-optimization loop.
```

### 2.1 Search Discretization

Nanometers are the canonical coordinate unit, but not a search lattice.
A 100mm × 100mm board at nanometer resolution is 10^14 cells — not
searchable. The solver operates on a separate routing representation:

```
Three geometry layers:

1. Canonical geometry (nm, i64)
   - Storage, DRC, final output
   - Exact coordinates, no approximation

2. Routing graph (adaptive, coarse-to-fine)
   - Global phase: coarse grid (e.g., 100μm cells) for congestion
     and layer assignment. Standard global routing technique.
   - Local phase: obstacle-aware graph construction around relevant
     pads, vias, and existing tracks. Graph nodes at pad centers,
     obstacle corners, Steiner candidate points. Graph edges are
     candidate path segments.
   - NOT a uniform grid. Resolution adapts to local density.

3. Continuous geometry reconstruction
   - After solving on the routing graph, convert discrete paths
     back to canonical nm coordinates.
   - Straighten segments, remove grid artifacts, enforce angles.
   - Verify clearance on the final geometry (not the graph proxy).
```

This matches how VLSI global/detailed routers work: coarse grid for
global assignment, obstacle-aware graph for detailed routing, exact
geometry for final verification.

### Phase 1: Placement

#### 1a. Component Grouping
```
Identify groups from multiple sources with explicit authority levels:

  Authoritative (persisted, consumed by constraint evaluator):
  - Explicit: user-defined groups in the constraint system
  - Hierarchical: schematic sub-sheets → one group per sheet

  Advisory (transient, used by solver for proposals only, NOT persisted,
  NOT consumed by constraint evaluator or DRC):
  - Topological: netlist clustering (strongly-connected subgraphs)
  - AI-inferred: circuit function recognition from topology + values
    ("U1 + R1 + R2 + C1 = inverting amplifier feedback network")

  Advisory groups inform the solver but do not affect deterministic
  engine behavior. A designer promotes an advisory group to explicit
  (authored data) by accepting the proposal.

Output: set of ComponentGroups with authority level and inter-group
  relationships (must-be-close, must-be-far, signal-flow-ordering)
```

#### 1b. Floorplanning (Group → Region Assignment)
```
Partition board area into regions.
Assign groups to regions considering:
  - Separation constraints (power vs. analog)
  - Signal flow (input → output spatial ordering)
  - Connector locations (fixed, constrain nearby groups)
  - Thermal zones (hot components in ventilated regions)

Method: recursive bisection or simulated annealing on region graph.
Output: per-group target region (bounding polygon).
```

#### 1c. Component Placement (Within-Group)
```
For each group, place components within assigned region:

Method: force-directed placement (analytic + iterative)
  - Net springs: connected components attract (spring constant ∝ pin count)
  - Overlap repulsion: courtyard boundaries repel
  - Boundary forces: components pushed away from region edges
  - Constraint forces: pin proximity pulls caps toward IC power pins
  - Orientation alignment: torque toward uniform rotation

Iterate until convergence (forces below threshold).
Snap to grid. Legalize (resolve any remaining overlaps).

Output: (x, y, rotation, layer) per component.
```

#### 1d. Routability Estimation
```
Before committing placement:
  - Compute HPWL (half-perimeter wire length) for all nets
  - Run congestion estimation (coarse grid channel capacity)
  - Identify bottlenecks ("BGA U1 east side: 47 nets, 12 channels")
  - Score placement quality (objective function value)

If routability score is below threshold:
  - Adjust placement (spread components in congested regions)
  - Re-run estimation
  - Iterate until acceptable or flag for human review

Output: PlacementProposal with quality score and warnings.
```

### Phase 2: Global Routing (Layer + Via Assignment)

Traditional routing is sequential: route net 1, then net 2, then net 3.
This is greedy and produces suboptimal results because early routing
decisions block later nets.

The solver uses two phases:

#### 2a: Assignment + Topology
Determine the routing strategy without computing exact paths.

```
For each net:
  1. Layer assignment: which layers will this net use?
     Input: allowed layers, impedance requirements, via budget
     Method: constraint propagation + cost estimation

  2. Via placement: where do layer transitions occur?
     Input: pad layers, assigned routing layers, board geometry
     Method: Steiner point estimation on the 3D pad graph

  3. Routing order: which nets route first?
     Input: constraint criticality (tightest constraints first),
            connectivity complexity, spatial congestion estimation
     Method: topological sort on constraint dependency graph

  4. Congestion estimation: where will routing be dense?
     Input: pad locations, net assignments, board geometry
     Method: global routing on a coarse grid (standard technique
             from VLSI — partitioned routing regions with capacity)
```

Global phase output: per-net routing plan (layers, approximate via
locations, routing order, congestion map). No exact paths yet.

### Phase 3: Detailed Routing — Path Computation
Compute exact track geometry that satisfies all constraints.

```
For each net (in order from Phase 1):
  1. Decompose net into 2-pin connections (Steiner tree decomposition)

  2. For each connection:
     a. Compute constraint corridor:
        - Obstacle map (existing copper, keepouts, other nets' clearance halos)
        - Width requirement for this net on this layer
        - Impedance-derived width if applicable

     b. Find path through corridor:
        - A* pathfinding on the routing graph (see §2.1)
        - Cost function incorporates wire length, bends, proximity to obstacles
        - Clearance satisfaction guaranteed by corridor construction
        - Reconstruct continuous geometry from graph path

     c. Optimize path:
        - Remove redundant vertices
        - Straighten segments where possible
        - Minimize corners (each corner is an impedance discontinuity)

  3. Validate complete net:
     - All pins connected
     - Length within tolerance
     - Impedance within tolerance (recompute from actual geometry)

  4. If validation fails:
     - Adjust path (reroute with tighter constraints)
     - If unresolvable: flag for human review
```

### Feedback: Cross-Phase Conflict Resolution

#### Within-Routing Iteration
After all nets are routed, run a refinement pass:

```
While (improvement > threshold):
  1. Identify worst constraint violations
  2. Rip up the most-violating nets
  3. Update obstacle map
  4. Reroute ripped-up nets with updated constraints
  5. Evaluate global objective function

This is the negotiate-and-reroute pattern used in VLSI:
nets compete for routing resources, conflicts are resolved
by iterative rip-up and reroute with updated priorities.
```

#### Cross-Phase Feedback
When detailed routing fails or produces poor results:

```
Routing → Global Routing feedback:
  "Net CLK0 cannot route on layer 2 — reassign to layer 3"
  → Phase 2 re-runs layer assignment for affected nets

Routing → Placement feedback:
  "BGA U1 east side congestion: 47 nets, 12 channels available"
  → Phase 1 shifts U1 or surrounding components to open space

Placement → Routing feedback:
  "Component C12 moved 2mm — rip up and reroute connected nets"
  → Phase 3 re-routes only affected nets incrementally

This feedback loop is the co-optimization. It does not run
indefinitely — maximum iteration count is configurable (default: 3).
Each iteration improves the objective function or the solver stops
and flags remaining issues for human review.
```

---

## 3. Constraint Solver Details

### Clearance as Minkowski Sum
Clearance checking during routing is equivalent to obstacle inflation:

```
For net N with clearance C to obstacle O:
  Inflated_obstacle = Minkowski_sum(O, circle(C))

The track for net N must not intersect any inflated obstacle.
This transforms clearance checking into simple geometric intersection,
which is computationally efficient.
```

### Impedance-to-Width Resolution
Before routing, impedance constraints are resolved to width constraints:

```
Given:
  Target impedance Z₀
  Layer index → stackup properties (Dk, thickness, copper weight)
  Structure type (microstrip, stripline, coplanar)

Compute:
  Required trace width W for Z₀ on this layer

Method:
  Closed-form approximation (IPC-2141 / Wadell equations) for v1.
  Full field solver (Method of Moments 2D) for v2+.

Result:
  width_preferred(net, layer) = W
  This is a pre-routing computation, not a routing-time computation.
```

### Length Matching
Length matching is a post-routing constraint:

```
1. Route all nets in the matching group
2. Measure routed lengths
3. Identify shortest and longest
4. For nets shorter than target:
   - Add serpentine/accordion tuning pattern
   - Pattern inserted in uncongested segments
   - Pattern amplitude and spacing respect clearance rules
5. Re-measure, iterate if needed
```

This is the same approach every EDA tool uses for length matching.
The difference is that the tuning pattern generation is automated,
not interactive (no human dragging meander patterns).

### Differential Pair Routing
Diff pairs are routed as a single entity, not two independent nets:

```
1. Compute coupled path (single centerline)
2. Generate parallel traces at ±(gap/2 + width/2) from centerline
3. At bends: compute coupled arc geometry to maintain gap
4. At via transitions: place via pair with correct stagger
5. Validate: intra-pair skew within tolerance
```

---

## 4. AI Integration Layer

The solver is algorithmic. The AI layer wraps it at three points:
before (intent → constraints), during (strategy selection), and
after (analysis + review).

### Intent Translation (LLM-driven, pre-layout)
```
User: "This is a DDR4 interface between U1 and U3"

AI resolves BOTH placement and routing constraints:

  Placement:
  - Group DQ byte lanes with their DQS pairs
  - Place DDR4 IC and memory symmetrically
  - Decoupling caps within 2mm of power pins
  - Signal flow: controller → termination → memory

  Routing:
  - DDR4 impedance targets (40Ω SE, 80Ω differential)
  - Length matching groups (DQ bytes, DQS-to-CLK, ADDR/CMD)
  - Max skew tolerances per JEDEC spec
  - Layer assignment (signals on inner stripline)

Output: unified constraint set for placement AND routing.
```

```
User: "Keep the analog section isolated from the switching regulator"

AI resolves:
  Placement:
  - group_separation(analog_group, power_group, 20mm)
  - thermal_separation(U5_regulator, U1_opamp, 15mm)

  Routing:
  - No routing of power nets through analog region
  - Ground plane continuity under analog section
```

### Strategy Suggestion (heuristic / learned, pre-layout)
```
Given a netlist and board outline:
  - Suggest component grouping and floorplan
  - Identify critical nets and their placement implications
  - Suggest routing order (critical first)
  - Suggest layer assignment
  - Identify high-congestion areas
  - Flag impossible constraints before attempting
  - Recommend placement patterns from library:
    "U5 (LM3671) matches buck converter pattern:
     Cin → VIN pin, L1 adjacent, Cout near VOUT, FB divider compact"

Method: rule-based heuristics initially.
Future: trained on corpus of successfully-designed boards.
```

### Review + Refinement (post-layout)
```
AI analyzes the complete layout (placement + routing):
  - "C12 is 5mm from U3 VIN pin — move within 2mm for decoupling"
  - "Q1 and Q2 are 8mm apart — reduce for thermal matching"
  - "Net CLK0 has 3 unnecessary vias — reassign to single layer"
  - "DQ byte 2 is 1.2mm over length target — add tuning here"
  - "Analog ground plane has a split under R7 — reroute digital trace"
  - "Estimated impedance on CLK1 segment 3 is 48Ω (target 50Ω)"

Human reviews, accepts or adjusts, AI learns from the decisions.
```

---

## 5. Incremental Operations

Not every operation is "layout the entire board." The common case
after initial layout is local changes — move one component, route
one net, adjust one rule. The solver handles these incrementally.

### Placement Operations
```
place_component(comp, position, rotation)  — manual placement
suggest_placement(comps)                    — AI suggests positions for unplaced components
refine_placement(group)                     — re-optimize one group's placement
```

### Routing Operations
```
route_net(net_id)           — route one unrouted net
reroute_net(net_id)         — rip up and reroute one net
route_nets(net_ids)         — route a group (e.g., one bus)
tune_length(net_id)         — add/adjust tuning pattern
add_track_manual(points, net, width, layer)  — user draws a track
delete_track(track_id)                        — remove a segment
```

### Reactive Re-Layout
When a component moves or a rule changes:
```
1. Identify affected nets (connectivity to moved/changed component)
2. Optionally: re-optimize nearby component positions (if unlocked)
3. Rip up affected tracks
4. Reroute affected nets with updated constraints
5. Report all changes as a proposal for human review
```

This is where co-optimization matters most — a component move
doesn't just invalidate routes, it may improve or worsen placement
of nearby components. The solver considers both.

---

## 6. Outputs

The layout engine produces Proposals, not committed geometry.
This is critical — nothing changes the design without explicit acceptance.

```rust
/// Unified proposal for placement, routing, or both
pub struct LayoutProposal {
    /// What this proposal covers
    pub scope: ProposalScope,  // PlacementOnly, RoutingOnly, Full

    /// Proposed component positions (placement)
    pub placements: Vec<ProposedPlacement>,

    /// Proposed tracks and vias (routing)
    pub tracks: Vec<ProposedTrack>,
    pub vias: Vec<ProposedVia>,

    /// Constraint satisfaction report
    pub report: ConstraintReport,

    /// Warnings (constraints met but margins are tight)
    pub warnings: Vec<LayoutWarning>,

    /// Failures (constraints not met — human must decide)
    pub failures: Vec<LayoutFailure>,

    /// Objective function score (lower is better)
    pub score: f64,
}

pub struct ProposedPlacement {
    pub component: Uuid,
    pub position: Point,   // new position
    pub rotation: i32,     // tenths of degree
    pub layer: LayerId,    // top or bottom
    pub previous: Option<(Point, i32, LayerId)>,  // for diff display
}

pub struct ConstraintReport {
    pub placement_overlap: bool,
    pub clearance_met: bool,
    pub width_met: bool,
    pub impedance_met: bool,
    pub length_met: bool,
    pub via_budget_met: bool,
    pub group_proximity_met: bool,
    pub group_separation_met: bool,
    pub routability_score: f64,  // 0.0 = unroutable, 1.0 = easy
    pub details: Vec<ConstraintDetail>,
}
```

The human (or AI agent) reviews the proposal and either:
- `accept(proposal)` — commits all changes as Operations
- `reject(proposal)` — discards, optionally adjusts constraints
- `accept_partial(proposal, items)` — commits selected placements/routes
- `refine(proposal, adjustments)` — modify and re-evaluate

No placement or route is committed without explicit acceptance. This is
the "AI proposes, engine validates, human reviews" paradigm.

---

## 7. Implementation Phases

These map to PLAN.md milestones M5, M6, M7.

### M5: Deterministic Layout Kernel (engineering milestone)

Current opening slice (2026-03-28):
- implemented contract: deterministic routing-substrate extraction report from
  persisted native board state only
- included facts: outline, stackup/layer set, keepouts, authored/persisted
  pads, tracks, vias, zones, nets, and net classes
- explicit non-goals for the opening slice: route search, feasibility
  decisions, corridor synthesis, autorouting, and any geometry inferred beyond
  persisted state

Current follow-on slice (2026-03-28):
- implemented contract: deterministic single-net route preflight from
  persisted native board state only
- authored anchors: persisted board pads already assigned to the target net
- candidate copper layers: derived only from persisted stackup plus currently
  available persisted routing facts in native state
- explicit non-goals for the follow-on slice: pathfinding, obstacle inflation,
  corridor synthesis, or any inferred per-net layer permissions not already
  present in persisted state

Current corridor slice (2026-03-28):
- implemented contract: deterministic single-net route corridor from persisted
  native board state only
- corridor primitive: deterministic anchor-to-anchor spans evaluated on the
  candidate copper layers already proven by `route-preflight`
- obstacle geometry: only persisted keepouts, foreign-net tracks/vias/zones,
  and board-outline escape conditions relevant to those spans
- explicit non-goals for the corridor slice: pathfinding, A*, route proposals,
  negotiated reroute, push-shove, or any invented constraints beyond the
  persisted native board state already present on disk

Placement:
- Component grouping (explicit + schematic hierarchy + netlist clustering)
- Force-directed placement within groups
- Overlap resolution and legalization
- Routability estimation (HPWL + coarse congestion)
- Placement proposal output with constraint report

Routing:
- Adaptive graph construction from board geometry
- A* pathfinding with obstacle-aware graph, Steiner decomposition
- Clearance-aware corridor computation (Minkowski inflation)
- Via rule enforcement during search
- Manual trace placement (point-to-point, clearance-aware)
- Multi-net: negotiated rip-up and reroute
- Diff pair prototype (coupled centerline, gap enforcement)
- Copper pour (polygon fill, thermal relief)
- Route proposal output with constraint satisfaction report

Co-optimization:
- Routing → placement feedback (congestion triggers placement adjustment)
- Placement → routing feedback (component move triggers reroute)
- Bounded iteration (max 3 cross-phase cycles)

**Benchmarks** (both placement and routing):
- Placement: HPWL, congestion score, constraint satisfaction, runtime
- Routing: completion rate, via count, total length, clearance violations
- Combined: runtime, stability under repeated runs with same seed

### M6: Strategy + AI Layer (intelligence milestone)
- Circuit recognition from netlist topology + part values
  ("R1 + R2 + U1 = feedback amplifier" → group + proximity constraints)
- Component group inference beyond schematic hierarchy
- Congestion-aware floorplanning
- Net ordering optimization (constraint criticality)
- Layer assignment (impedance-aware from stackup)
- Impedance-to-width resolution (closed-form, IPC-2141/Wadell)
- Length matching (automated serpentine/accordion insertion)
- Diff pair skew matching, via stagger patterns
- AI intent translation: "DDR4 interface" → formal constraint set
  including both placement groups and routing constraints
- AI strategy suggestion: placement regions, routing order, layer budget
- AI post-layout analysis: weak points, improvements
- Design pattern library: known-good placement patterns for common circuits
  (LDO, buck converter, USB, crystal oscillator, op-amp stage)
- Proposal ranking: score candidates, present best with rationale

### M7: Review UI (interaction milestone)
- Placement proposal review (accept/reject/adjust per component)
- Route proposal review (accept/reject/adjust per net)
- Net length gauge, route state visualization, clearance display
- Component group visualization (highlight groups, show constraints)
- Command line placement + routing commands
- Keyboard-centric parameter cycling during manual operations

---

## 8. Comparison with Existing Approaches

| Approach | Method | Weakness |
|----------|--------|----------|
| Maze router (Lee 1961) | BFS on grid | No constraint awareness, sequential |
| Line router (Hightower 1969) | Line-probe | Incomplete, can't handle complex obstacles |
| Topological router (1980s) | Rubber-band on topology graph | Poor completion rate on dense boards |
| Push-and-shove (KiCad PNS) | Interactive, human-driven | Requires human to provide global strategy |
| Altium ActiveRoute | Guided sequential | Can't place vias, no true optimization |
| Commercial autorouter (Situs, etc.) | Sequential with heuristics | Geometric only, no electrical awareness |
| **This engine** | **Constraint-formalized routing with AI policy layer** | **Unproven integration, needs validation** |

### Honest Assessment
The underlying algorithms (graph search, obstacle inflation, constraint
propagation, negotiated reroute) are well-established in VLSI and
existing PCB tools. The novel contribution is not a new routing algorithm
but a properly-formalized constraint pipeline with an AI strategy/review
layer.

The risk is integration, not invention: can the constraint formalization,
strategy layer, and proposal/review workflow produce results that are
competitive with human-driven push-and-shove on real designs?

Mitigation: the solver's output is always a proposal, never auto-committed.
Even a solver that handles 80% of nets correctly and flags the remaining
20% for human attention is more useful than an autorouter that silently
produces signal integrity violations. The proposal/review workflow turns
solver weakness into human-in-the-loop repairability.
