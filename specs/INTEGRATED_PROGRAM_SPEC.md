# INTEGRATED_PROGRAM_SPEC.md

Status: Draft scaffold for full integrated program specification write.

## 1. Purpose

This document is the integration layer above individual specs in `specs/`.
It defines:

- naming and identity policy for the project
- source-of-truth precedence across specs
- cross-spec contracts that must remain consistent
- freeze gates for promoting this scaffold into the full integrated program
  specification

It does not restate detailed type/API schemas already controlled elsewhere.
It does not own live implementation status claims.

## 2. Naming Policy (Controlling)

- Product name: `Datum EDA`
- Machine identifier form: `datum-eda`
- Prohibited: legacy placeholder project names in code, fixtures, docs, tests,
  MCP registration, sockets, and automation paths

## 3. Source-of-Truth Precedence

When documents conflict, precedence is:

1. `specs/INTEGRATED_PROGRAM_SPEC.md` (integration contracts only)
2. Domain specs:
   - `specs/PROGRAM_SPEC.md`
   - `specs/ENGINE_SPEC.md`
   - `specs/MCP_API_SPEC.md`
   - `specs/IMPORT_SPEC.md`
   - `specs/ERC_SPEC.md`
   - `specs/CHECKING_ARCHITECTURE_SPEC.md`
   - `specs/SCHEMATIC_CONNECTIVITY_SPEC.md`
   - `specs/SCHEMATIC_EDITOR_SPEC.md`
   - `specs/NATIVE_FORMAT_SPEC.md`
3. Progress tracking authority: `specs/PROGRESS.md`
4. Rationale documents in `docs/` (non-controlling unless promoted to `specs/`)

Status authority rule:
- `specs/PROGRESS.md` is the only source of truth for implementation status.
- This file may define acceptance evidence and defer contracts, but must not
  introduce independent milestone completion state.

## 4. Integrated Contract Baseline

This section defines contract wiring only:
- milestone gate definitions are controlled by `specs/PROGRAM_SPEC.md`
- integrated acceptance mapping is controlled by Sections 12-13 in this file
- implementation status for those gates is controlled by `specs/PROGRESS.md`

## 5. Cross-Spec Contracts (Must Stay Aligned)

### 5.1 Milestone Contracts

- Milestone gate definitions: `specs/PROGRAM_SPEC.md`
- Concrete implementation status: `specs/PROGRESS.md`
- Plan sequencing: `PLAN.md`

Contract:
- Milestone completion state must be recorded in `specs/PROGRESS.md` only.
- Supporting docs may summarize status but may not introduce independent
  completion claims.
- Fresh execution or takeover must anchor first to `specs/PROGRESS.md` and the
  active milestone shard docs in `specs/progress/`. `PLAN.md` is sequencing
  guidance only, and uncommitted worktree state must not override the recorded
  milestone frontier when they conflict.

### 5.2 Engine/API/MCP Contract Split

- Engine behavior surface: `specs/ENGINE_SPEC.md`
- MCP wire schemas and method catalog: `specs/MCP_API_SPEC.md`
- Program-level exit gates and tool expectations: `specs/PROGRAM_SPEC.md`

Contract:
- Each exposed tool/method must map 1:1 across engine capability, daemon/MCP
  schema, and gate language (or be explicitly deferred in all three).

### 5.3 Checking Contracts

- Rule semantics: `specs/ERC_SPEC.md` and `specs/CHECKING_ARCHITECTURE_SPEC.md`
- Execution/reporting exposure: `specs/PROGRAM_SPEC.md` and `specs/MCP_API_SPEC.md`

Contract:
- Violation identity, severity, waiver targeting, and report shape remain
  coherent across engine, CLI, MCP, and progress gates.

### 5.4 Schematic/Board Parity Contracts

- Schematic connectivity and editor semantics:
  `specs/SCHEMATIC_CONNECTIVITY_SPEC.md`, `specs/SCHEMATIC_EDITOR_SPEC.md`
- Board + shared engine model: `specs/ENGINE_SPEC.md`

Contract:
- Board and schematic capability growth remains parity-tracked in milestones and
  does not allow one side to become structurally underspecified.
- Native CLI/engine surface growth beyond current daemon/MCP implementation is
  allowed only when the same change updates deferred MCP parity tracking in
  `specs/MCP_API_SPEC.md`; silent MCP drift is not acceptable even when MCP
  code parity is intentionally deferred.
- Repeated ordinal contract families (`n=1,2,3...`) must not grow by
  mechanical succession indefinitely; once the pattern is proven, development
  must either declare a stopping boundary or replace the family with one
  generalized contract.
- Repeated qualifier/suffix contract families (`foo-aware`,
  `foo-aware-bar-aware`, `...-explain`, and similar stacked variants) must not
  grow by mechanical adjective or suffix accretion once the underlying pattern
  is proven.
- When qualifier/suffix stacking starts describing policy combinations more
  than new milestone semantics, development must stop and either:
  - replace the family with one bounded generalized contract that exposes the
    accepted policy dimensions explicitly, or
  - declare a stopping boundary and defer further expansion.
- Further qualifier/suffix-family growth is not acceptable until one of those
  two actions has been taken.
- Generalization must be evidence-based within the same contract family.
  A family may be collapsed into a bounded policy/parameter surface only when
  multiple accepted variants already exist inside that family and those
  variants differ along a real, bounded semantic axis.
- Cross-family analogy is not sufficient justification for generalization.
  A successful `--policy` or parameterized surface in one family does not, by
  itself, justify introducing the same abstraction into a neighboring family.
- Each new contract family or generalized surface must carry an explicit
  stopping boundary or generalization trigger. If neither can be stated
  clearly, the family is not ready for repeated adjacent expansion.
- Adjacent-slice growth must justify itself by one of three values only:
  real capability increase, real observability/debuggability increase, or real
  simplification/generalization of an already-proven family. Mere adjacency is
  not enough.
- Paired `...-explain` or other observability surfaces are optional, not
  mandatory. They should be added only when they unlock materially needed
  debugging/review value for the current frontier; symmetry with an earlier
  family is not enough.
- A landed slice with passing focused proof does not imply that a paired
  observability surface is the default next step. Unless milestone docs
  explicitly require observability before advancement, the next slice should
  default to the next capability contract.
- When a generalized surface replaces a repeated family, the same change must
  record which surface is now preferred and whether the older family is frozen,
  compatibility-only, or scheduled for later removal.
- Contract naming complexity is bounded. If a new surface name is growing by
  repeated qualifiers or chained semantics to explain what it does, that is a
  signal to generalize or stop rather than keep extending the name.
- Completion reporting must be diff-grounded. The declared implemented
  contract in a handoff/report must match the actual landed core files and
  tests for that slice; prior narrative or previously declared intent is not
  authoritative if the diff says otherwise.
- Dirty worktree momentum is not roadmap authority. Uncommitted or partially
  implemented files may inform the implementation plan for an already-selected
  slice, but they must not decide what the next contract should be when the
  milestone docs point elsewhere.
- Audit/review/checkpoint work is exceptional, not the default adjacent slice.
  After a contract lands cleanly, the next slice should normally be the next
  thin capability contract unless a test/gate failure, a real defect, or a
  genuine contract ambiguity requires review work.
- Touched-monolith burn-down must move real ownership into coherent companion
  shards; it must not be satisfied by facade-shell proliferation.
- Root modules may be thin, but they must remain readable module roots with
  visible ownership boundaries. One-line `include!` trampolines, root-to-root
  forwarding shells, and extraction layers that consist primarily of
  registration/re-export churn are not acceptable as structural burn-down.
- Companion shards created for burn-down must own real behavior, types,
  helpers, or tests. Pure import/re-export/include indirection does not count
  as architectural decomposition.

## 6. Promotion Criteria to Full Integrated Spec

Promote this scaffold to the full integrated program spec when:

- all domain specs include explicit “current vs target” contract language where
  needed
- cross-spec contract checklist in Section 5 has no unresolved conflicts
- `PLAN.md` remains execution-only and free of independent status claims
- open deferrals for the next milestone phase are explicit and mutually
  referenced

## 7. Next Write Order (Full Spec Authoring)

1. Lock `M2` baseline snapshot (done)
2. Write integrated contracts for `M3` operation model boundary
3. Write integrated contracts for `M4` native format + schematic editor
4. Add verification appendix mapping each integrated contract to an executable
   check or explicit manual review step

## 8. Contract Verification Matrix

This matrix is the minimum verification layer for integrated-spec freeze.

| Contract Area | Verification Method | Command / Evidence | Owner | Frequency |
|---------------|---------------------|--------------------|-------|-----------|
| Naming policy (`Datum EDA` / `datum-eda`) | Automated scan | `rg -n "horizon-ai|Horizon-ai|horizon ai|Horizon AI|horizon_ai" -g '!target/**' -g '!.git/**' /home/bfadmin/Documents/datum-eda` must return no matches | release owner | pre-merge for spec-affecting changes |
| Workspace health | Automated test run | `cargo test -q` | platform owner | pre-freeze and CI |
| M2 quality gates | Automated harness | `cargo run -p eda-test-harness --bin m2_quality -- --json` with `"pass": true` | checking owner | pre-freeze and CI |
| M2 performance baseline | Automated harness | `cargo run -p eda-test-harness --bin m2_perf -- --iterations 3 --compare-baseline crates/test-harness/testdata/perf/m2_doa2526_baseline.json` | checking owner | pre-freeze |
| MCP server registration health | Config + self-test | `python3 mcp-server/server.py --self-test` and active MCP host config contains `mcpServers["datum-eda"]` | tooling owner | after MCP changes |
| Engine/API/MCP surface parity | Manual cross-check + tests | `specs/PROGRAM_SPEC.md`, `specs/ENGINE_SPEC.md`, `specs/MCP_API_SPEC.md` reviewed together; parity tests in engine-daemon + mcp-server pass | platform owner | per milestone gate |
| Daemon/MCP transport smoke | Automated transport smoke in unrestricted environment | `cargo test -q -p eda-engine-daemon handle_client_round_trips_open_project_and_get_check_report -- --ignored` | platform owner | unrestricted CI and release candidates |
| Checking report/waiver consistency | Manual contract audit + targeted tests | `specs/ERC_SPEC.md` + `specs/CHECKING_ARCHITECTURE_SPEC.md` + `specs/MCP_API_SPEC.md` reviewed; ERC/DRC explanation/report tests pass | checking owner | per milestone gate |
| Schematic/board parity | Manual contract audit | parity section in `specs/PROGRAM_SPEC.md` and `specs/INTEGRATED_PROGRAM_SPEC.md` updated in same change as any scope shift | architecture owner | per milestone planning pass |

Transport note:
- Socket transport proof is tracked separately from behavioral surface proof.
- Sandbox-compatible audits (`check_alignment.py`, M3 preflight, workspace health)
  must not require live Unix socket listeners or socket-pair IPC.
- Unix socket transport is instead validated in unrestricted CI and release
  candidate checks.

### 8.1 Freeze Rule

Integrated spec freeze requires all matrix rows to be either:

- passing with reproducible evidence, or
- explicitly deferred with a dated defer note in both
  `specs/INTEGRATED_PROGRAM_SPEC.md` and `specs/PROGRESS.md`.

## 9. M3 Integrated Boundary Contract (Imported-Design Writes)

### 9.1 Scope Boundary

`M3` is restricted to deterministic write operations on imported designs with
KiCad board write-back. Imported-schematic editing remains out of scope.

In scope:
- board-side authored modifications listed in `specs/PROGRAM_SPEC.md` (`M3`)
- operation execution, batch execution, undo/redo semantics
- derived data recomputation after each committed write operation
- CLI/MCP write exposure for M3-listed operations

Out of scope:
- native project creation/editing semantics (`M4`)
- routing/placement solver behavior (`M5+`)
- GUI-authoring semantics (`M7+`)

### 9.2 Controlling Specs for M3

- Exit gates: `specs/PROGRAM_SPEC.md` (`M3`)
- Engine write semantics and API contracts: `specs/ENGINE_SPEC.md`
- MCP write wire contracts: `specs/MCP_API_SPEC.md`
- History/context gate and fidelity-claim framing:
  `docs/R1_G0_FOUNDATION.md`
- Progress state: `specs/PROGRESS.md`

### 9.3 M3 Non-Negotiable Invariants

- Every write operation is transactionally represented and undoable.
- Undo/redo does not mutate operation meaning across runs.
- Same operation sequence on same baseline input yields identical persisted
  output.
- Derived connectivity/airwire/checking recomputation is deterministic and
  operation-bounded.
- Exported KiCad output preserves unmodified authored objects exactly where
  contract requires.
- `M3` evidence must be scoped to the implemented KiCad imported-board slice
  and must not be generalized into broader interop-completeness claims.
- Fidelity claims for `M3` outputs must remain classifiable as `exact`,
  `approximated`, `preserved-as-metadata`, or `unsupported` per
  `docs/R1_G0_FOUNDATION.md`.

### 9.4 M3 Verification Evidence

At minimum:
- operation determinism tests
- undo/redo round-trip tests
- import → modify → save → reimport fidelity checks
- CLI exit semantics for write flows
- MCP/daemon write-method parity tests for M3 surface
- explicit evidence that accepted lossiness remains bounded and documented,
  rather than inferred from successful file opening alone

## 10. M4 Integrated Boundary Contract (Native Authoring + Export)

### 10.1 Scope Boundary

`M4` introduces first-class native project authoring and manufacturing export.
It is the parity milestone where schematic and board authoring surfaces must be
equally spec-strong and scriptable.

In scope:
- native file format and schema-versioning contracts
- schematic operation surface from `specs/SCHEMATIC_EDITOR_SPEC.md`
- board operation surface listed under `M4` in `specs/PROGRAM_SPEC.md`
- forward-annotation ECO flow with review/accept controls
- Gerber/Excellon/BOM/PnP export contracts

Out of scope:
- automated placement/routing strategy engines (`M5+`)
- GUI interaction models and tooling (`M7+`)
- advanced collaboration semantics beyond defined single-writer assumptions

### 10.2 Controlling Specs for M4

- Exit gates: `specs/PROGRAM_SPEC.md` (`M4`)
- Native persistence: `specs/NATIVE_FORMAT_SPEC.md`
- Schematic authoring model and operations: `specs/SCHEMATIC_EDITOR_SPEC.md`
- Engine type/API contracts: `specs/ENGINE_SPEC.md`
- MCP/CLI tool contracts: `specs/MCP_API_SPEC.md`
- History/context gate and migration-boundary framing:
  `docs/R1_G0_FOUNDATION.md`
- Progress state: `specs/PROGRESS.md`

### 10.3 M4 Parity Invariants (Board vs Schematic)

- Schematic and board authored objects both use stable UUID identity and
  deterministic serialization.
- Schematic operation catalog is not treated as advisory; it is gate-level.
- Query parity for schematic topology/state is exposed with the same rigor as
  board query surfaces.
- ECO boundaries (schematic→board and board→schematic where applicable) are
  explicit and reviewable, never implicit side effects.
- Native persistence must not absorb vendor-specific migration semantics into
  canonical state without either a stable cross-tool mapping or structured
  metadata preservation.
- Schema/version evolution must support explicit migration reporting rather
  than silent semantic flattening.

### 10.4 M4 Verification Evidence

At minimum:
- native project open/save deterministic round-trip tests
- schematic operation acceptance tests (minimum operation set)
- board operation acceptance tests (minimum operation set)
- ECO proposal generation + apply/reject tests
- manufacturing artifact validation checks against reference viewers/tooling

## 11. Integrated Defer Discipline

Any contract intentionally deferred beyond the active milestone must include:

- defer scope (exactly what is deferred)
- defer reason (why it is outside current milestone)
- target milestone for re-entry
- cross-spec references where the defer is mirrored

A defer is invalid unless it appears in both:
- `specs/INTEGRATED_PROGRAM_SPEC.md` and
- `specs/PROGRESS.md`.

## 12. M3 Acceptance Table (Gate-to-Evidence Mapping)

This table is the controlling acceptance map for `M3` integrated review.
Documentation-parity note:
- This section normalizes gate naming to `specs/PROGRAM_SPEC.md` only.
- It does not reopen `M3` implementation scope or alter `M3 overall` status in
  `specs/PROGRESS.md`.
Primary executable hook:
`cargo run -p eda-test-harness --bin m3_acceptance_gate -- --json`  
Current behavior: returns structured `passed` status only when the current
base determinism, replacement determinism, undo/redo, replacement undo/redo,
board round-trip fidelity, sidecar-backed round-trip fidelity, and
engine/daemon/MCP/CLI write-surface parity hooks all pass together.
Companion hook for save determinism:
`cargo run -p eda-test-harness --bin m3_op_determinism -- --json`
Current behavior: returns structured `passed` status for the current save-backed
`move_component`/`delete_track`/`delete_via`/`delete_component`/`rotate_component`/`set_value`/`set_reference`/`set_design_rule`/`assign_part`/`set_package`/`set_net_class` KiCad-board slices and should fail if save determinism regresses.
Companion hook for replacement-family save determinism:
`cargo run -p eda-test-harness --bin m3_replacement_op_determinism -- --json`
Current behavior: returns structured `passed` status for the current
`set_package_with_part`/`replace_component`/`replace_components`/`apply_component_replacement_plan`/`apply_component_replacement_policy`/`apply_scoped_component_replacement_policy`/`apply_scoped_component_replacement_plan` save-backed replacement slice and should fail if replacement save determinism regresses.
Companion hook for stack behavior:
`cargo run -p eda-test-harness --bin m3_undo_redo_roundtrip -- --json`
Current behavior: returns structured `passed` status for the current
`delete_track`/`delete_via`/`delete_component`/`move_component`/`rotate_component`/`set_value`/`set_reference`/`set_design_rule`/`assign_part`/`set_package`/`set_net_class` undo/redo slice and should fail if round-trip stack behavior regresses.
Companion hook for replacement-family stack behavior:
`cargo run -p eda-test-harness --bin m3_replacement_undo_redo_roundtrip -- --json`
Current behavior: returns structured `passed` status for the current
`set_package_with_part`/`replace_component`/`replace_components`/`apply_component_replacement_plan`/`apply_component_replacement_policy`/`apply_scoped_component_replacement_policy`/`apply_scoped_component_replacement_plan` undo/redo slice and should fail if replacement transaction round-trip behavior regresses.
Companion hook for pure board-write artifact fidelity:
`cargo run -p eda-test-harness --bin m3_board_roundtrip_fidelity -- --json`
Current behavior: returns structured `passed` status for unmodified KiCad-board identity plus current
`delete_track`/`delete_via`/`delete_component`/`move_component`/`rotate_component`/`set_value`/`set_reference` save→reimport→save artifact stability and should fail if pure board-write round-trip fidelity regresses.
Companion hook for sidecar-backed save fidelity:
`cargo run -p eda-test-harness --bin m3_sidecar_roundtrip_fidelity -- --json`
Current behavior: returns structured `passed` status for the current
`set_design_rule`/`assign_part`/`set_package`/`set_package_with_part`/`replace_component`/`replace_components`/`apply_component_replacement_plan`/`apply_component_replacement_policy`/`apply_scoped_component_replacement_policy`/`apply_scoped_component_replacement_plan`/`set_net_class` save→reimport→save artifact-stability slice and should fail if KiCad-board or sidecar round-trip fidelity regresses.
Companion hook for CLI/MCP/daemon write-surface readiness:
`cargo run -p eda-test-harness --bin m3_write_surface_parity -- --json`
Current behavior: returns structured `passed` status for the current
engine/daemon/MCP/CLI `move_component`/`rotate_component`/`set_value`/`set_reference`/`assign_part`/`set_package`/`set_package_with_part`/`replace_component`/`replace_components`/`apply_component_replacement_plan`/`apply_component_replacement_policy`/`apply_scoped_component_replacement_policy`/`apply_scoped_component_replacement_plan`/`set_net_class`/`delete_component`/`delete_track`/`delete_via`/`set_design_rule`/`undo`/`redo`/`save`
slice and should fail if current write-surface parity regresses.

| M3 Gate (`specs/PROGRAM_SPEC.md`) | Evidence Type | Required Evidence Hook |
|-----------------------------------|---------------|------------------------|
| Operations implemented | Automated | Engine operation API tests + MCP write method tests covering each listed op |
| Undo/redo | Automated | Transaction replay/undo test suite with per-operation round-trip assertions |
| Operation determinism | Automated | Determinism harness: same op sequence on same input yields byte-identical save |
| KiCad write-back | Automated + manual spot-check | import→modify→save→reimport tests; open output in KiCad without load errors |
| Round-trip fidelity | Automated | Golden diff tests for unchanged objects across write-back |
| MCP write tools | Automated | Daemon + MCP tests for `move_component`, `rotate_component`, `set_value`, `set_reference`, `assign_part`, `set_package`, `set_package_with_part`, `replace_component`, `replace_components`, `apply_component_replacement_plan`, `apply_component_replacement_policy`, `apply_scoped_component_replacement_policy`, `apply_scoped_component_replacement_plan`, `set_net_class`, `delete_component`, `set_design_rule`, `delete_track`, `delete_via`, `undo`, `redo`, `save` |
| Derived data update | Automated | Post-op connectivity/airwire/DRC recompute assertions |
| CLI modify command | Automated | CLI integration tests for `tool modify ...` with exit semantics |

## 13. M4 Acceptance Table (Gate-to-Evidence Mapping)

This table is the controlling acceptance map for `M4` integrated review.

R1 dependency rule:
- `M3 overall` and `M4 overall` completion state in `specs/PROGRESS.md`
  remains gated by `R1-G0 Foundation Gate`.

| M4 Gate (`specs/PROGRAM_SPEC.md`) | Evidence Type | Required Evidence Hook |
|-----------------------------------|---------------|------------------------|
| Native format | Automated | Native save/load/save byte-stability tests + schema version migration tests |
| Schematic operations | Automated | Operation acceptance tests for all required schematic operations from `specs/SCHEMATIC_EDITOR_SPEC.md` |
| Board operations | Automated | Operation acceptance tests for M4 board operation set |
| Schematic query parity | Automated | Engine/daemon/MCP/CLI parity tests for labels/buses/bus_entries/noconnects/hierarchy/fields |
| Forward annotation | Automated + manual | ECO proposal generation tests + apply/reject flow tests with deterministic diff output |
| Gerber export | Automated + manual | Export test fixtures + gerbv validation with zero warnings |
| Drill export | Automated + manual | Export test fixtures + gerbv drill validation |
| BOM export | Automated | Format/schema tests + deterministic output tests |
| PnP export | Automated | Output schema and deterministic ordering tests |
| Gerber comparison | Automated + manual | Layer/alignment/aperture comparison script + manual visual diff audit |

### 13.1 Evidence Wiring Rule

For each row in Sections 12-13, the milestone cannot be marked complete until:

- the evidence hook exists in repository code/tests or documented manual
  procedure, and
- the corresponding status row in `specs/PROGRESS.md` is synchronized.
