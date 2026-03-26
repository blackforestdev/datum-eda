# Licensing Strategy

## The Router Question (Resolved)

### KiCad PNS: Not Needed
KiCad's PNS is an interactive push-and-shove router (GPL v3, CERN origin).
It solves the problem: "human drags mouse, router shoves obstacles in
real-time." This is the wrong paradigm for an AI-native tool.

### Why Traditional Autorouter UX Is Not the Right Interface
Every commercial autorouter produces results that need significant manual
cleanup (Altium Situs, Eagle, PADS). They optimize for completion rate
and wire length without adequate signal intent awareness — impedance,
crosstalk, power integrity, thermal constraints. The underlying algorithms
(graph search, constraint propagation, obstacle inflation) are sound.
The problem is the UX paradigm: black-box "push button, get routes" with
no human steering, no formal constraint pipeline, and no review workflow.

### The Replacement: Constraint-Formalized Layout Engine
The layout paradigm for this project is:
**Solver proposes placement + routes → Engine validates → Human reviews.**

Classical algorithms (force-directed placement, A* pathfinding, negotiated
reroute) are the core solvers. They operate on a formal constraint model
derived from the engine's rule system. An AI policy layer handles intent
translation, strategy selection, and review — but is not "the solver."

See docs/LAYOUT_ENGINE.md for full specification. The layout engine covers
both placement and routing as one coupled optimization.

None of this requires KiCad PNS. The GPL question is dissolved.

### Implementation (M5-M6)
- **M5**: Deterministic layout kernel — placement, pathfinding, obstacle
  inflation, multi-net reroute, diff pair prototype. Adaptive graph for
  search (not uniform grid). Benchmarked.
- **M6**: AI strategy layer — circuit recognition, congestion-aware
  floorplanning, impedance resolution, intent-to-constraint translation.

### Manual Trace Placement (M5)
For cases where the designer wants direct control:
- Point-to-point trace placement with clearance-aware pathfinding
- Obstacle avoidance on adaptive routing graph
- Adequate for adjustments to solver proposals

## Engine License Candidates

With routing deferred, the engine license is a free choice:

| License | Pros | Cons |
|---------|------|------|
| MIT | Maximum freedom, commercial-friendly | No copyleft protection |
| Apache 2.0 | Patent grant, commercial-friendly | Slightly more complex |
| MPL 2.0 | File-level copyleft, allows proprietary extensions | Less common in Rust ecosystem |
| AGPL v3 | Network copyleft, prevents SaaS freeloading | Scares away commercial users |
| Dual (MIT + commercial) | Open core model | Requires CLA for contributions |

### Non-recommendation
Do not choose GPL v3 by default. That forecloses options without providing
benefits that the alternatives don't also provide. GPL is appropriate when
you want to prevent proprietary forks. If that's not a priority, a
permissive license gives more flexibility.

### Decision: TBD
License choice should be made at M0, before any code is written and before
any contributors are involved. It should be a deliberate choice, not an
inheritance.

## Third-Party Dependencies

| Dependency | License | Risk |
|-----------|---------|------|
| KiCad PNS router | GPL v3 | High — infects if linked. Mitigate with process isolation or deferral |
| rusqlite | MIT | None |
| serde / serde_json | MIT/Apache 2.0 | None |
| wgpu | MIT/Apache 2.0 | None |
| pyo3 | MIT/Apache 2.0 | None |
| geo (Rust) | MIT/Apache 2.0 | None |
| nlohmann/json | MIT | None (if any C++ interop needed) |
| OpenCASCADE | LGPL v2.1 | Low — LGPL allows dynamic linking without infecting |

The Rust ecosystem is overwhelmingly MIT/Apache 2.0 dual-licensed.
No significant license risk from Rust dependencies.
