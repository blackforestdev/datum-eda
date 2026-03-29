# Development Plan

## Document Role

This file is the execution plan and sequencing guide.
It is not a live status ledger.

Status source of truth:
- `specs/PROGRESS.md` is the only authoritative status tracker.
- Status changes must be recorded in `specs/PROGRESS.md` first.
- This file may describe planned execution order and priorities, but must not
  introduce independent completion claims.
- If `PLAN.md` conflicts with `specs/PROGRESS.md` or `specs/progress/*.md`,
  treat `PLAN.md` as stale and follow the milestone status/frontier recorded in
  those spec files instead.
- Dirty worktree state is implementation context only. It may help complete an
  already-selected slice, but it must not redefine roadmap priority or the next
  logical contract when milestone docs point elsewhere.
- A landed slice with passing focused proof should normally advance to the
  next capability contract. Paired `...-explain` follow-ons are optional and
  should be chosen only when the milestone docs explicitly require added
  observability before advancement.

Terminology follows `specs/PROGRAM_SPEC.md`:
`Product identity`, `Implementation slice`, `Execution strategy`, and
`Non-goals`.

## Mission Layers

**Product identity**: AI-native EDA platform with deterministic core semantics
and machine-native interfaces.

**Launch wedge**: Best AI/CLI design analysis and automation environment for
Linux PCB projects, validated first through a KiCad-first execution path.

**Core platform**: Canonical design IR plus operation engine with import,
query, ERC/DRC, transformation, and export.

**Full CAD ambition**: Native editing, interactive routing, GUI, advanced
constraints. This is the trajectory, not the launch.

## Stabilization Track (2026-03-26)

Structural debt retirement is tracked in `docs/STABILIZATION_PLAN.md` and runs
in parallel with milestone delivery.
The stabilization track is scoped to non-behavioral decomposition and
guardrails, not feature expansion.

## Active Execution Window

Current planning focus: `M5` deterministic layout-kernel development from
persisted native board state, following the current frontier recorded in
`specs/PROGRESS.md` and `specs/progress/m5_opening.md`.

Near-term execution order:
1. Keep `M3` closed except for regression fixes; do not widen imported-design
   scope without an explicit new milestone need.
2. Keep `M4` closed for scope except for regression fixes or explicit
   documentation/governance maintenance.
3. Use `specs/PROGRESS.md` plus the active milestone shard in
   `specs/progress/` as the source for the next `M5` slice; do not let local
   worktree momentum redefine roadmap priority.
4. Prioritize real capability contracts over adjacent review/checkpoint/query
   expansion once the routing-kernel read surfaces are proven.

## Milestones

### M0: Canonical IR + Foundation
Goal: deterministic core IR, pool foundation, import harness scaffolding.
Deliverable: pool ingestion/query/serialization works deterministically.

### M1: Design Ingestion + Query Engine
Goal: import KiCad/Eagle designs and expose deterministic query surfaces.
Deliverable: DOA2526 + bounded Eagle subset open/query deterministically.

### M2: ERC + DRC + Reporting + MCP/CLI
Goal: useful electrical/physical checking via CLI and MCP.
Deliverable: CI-ready check flows and machine-readable reports.

### R1: Commercial Interop Research Track
Goal: define migration architecture path for commercial tool ecosystems.
Deliverable: implementation brief with fidelity rubric and explicit non-goals.

### M3: Write Operations on Imported Designs
Goal: deterministic, safe imported-design write operations.
Deliverable: board-slice modifications, undo/redo, and save-backed workflow.

### M4: Native Project Creation + Editing
Goal: create designs natively, not only modify imports.
Deliverable: native authoring path from schematic to manufacturing outputs.

### M5: Deterministic Layout Kernel
Goal: unified placement + routing kernel with constraint awareness.
Deliverable: proposal-grade deterministic layout kernel benchmarked on fixtures.

### M6: Layout Strategy + AI Layer
Goal: intent-driven strategy and AI-assisted layout proposal ranking.
Deliverable: explainable placement/routing strategies from user intent.

### M7: GUI + Review Interface
Goal: visual review/editing layer on top of machine-native core.
Deliverable: route review and schematic/board interaction surfaces.

### M8: Professional Features
Goal: advanced constraints, manufacturing, reuse, and enterprise depth.
Deliverable: professional-grade production feature set.

## Critical Path

```
M0 (IR + pool) ──→ M1 (import + query) ──→ M2 (DRC + MCP/CLI)
                                                    │
                                           M3 (write ops) ──→ M4 (native authoring)
                                                                       │
                                              M5 (layout kernel) ──→ M6 (strategy + AI)
                                                                       │
                                                              M7 (GUI) ──→ M8 (pro)
```

Milestone intent:
- M2 is the first useful milestone.
- M3 is the first AI-write milestone.
- M4 is the first no-secondary-tool authoring milestone.
- M5 introduces deterministic placement+routing kernel.
- M6 adds intelligence on top of the proven kernel.
- M7 is the first visual tool milestone.

## Decision Log

| Date | Decision | Rationale |
|------|----------|-----------|
| 2026-03-24 | Study Eagle/Horizon/Altium | Know the domain before building |
| 2026-03-24 | Build from scratch, not fork | Fork inherits unwanted architecture, GPL, GUI-first design |
| 2026-03-24 | Rust engine | Memory safety, modern toolchain, cargo ecosystem |
| 2026-03-24 | Engine-first, GUI-last | Headless is the differentiator; GUI is a consumer |
| 2026-03-24 | Import-first, author-later | Useful before complete; real designs validate the engine |
| 2026-03-24 | MCP as AI interface | Standard protocol, Claude Code integration |
| 2026-03-24 | Altium UX as benchmark | Professional features = domain vocabulary for AI |
| 2026-03-24 | One architect | Community-driven EDA has failed; opinionated direction required |
| 2026-03-24 | No KiCad PNS dependency | AI-proposes/human-reviews routing paradigm removes push-and-shove dependency. Constraint-formalized routing plus review is the path. |
| 2026-03-24 | v1 scope: analysis + automation | Full CAD is the trajectory, not the launch |
