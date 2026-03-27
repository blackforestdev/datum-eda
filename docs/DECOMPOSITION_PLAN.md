# Structural Decomposition Plan

Status: active structural-only stabilization track.
Scope: decomposition and governance only. No behavior changes.

## 1. Purpose

This plan closes monolith risk across the repository by:
- auditing oversized modules
- defining decomposition shards and sequencing
- enforcing non-growth governance in CI

This plan is intentionally non-functional. It must not expand product scope,
change milestone contracts, or alter runtime semantics.

## 2. Discovery Sweep (2026-03-27)

Largest high-risk source modules identified by repo sweep:

| File | Size | Risk |
|------|------|------|
| `crates/cli/src/command_project.rs` | 9244 LOC | Critical monolith: multi-domain command logic + serialization + flow control |
| `crates/cli/src/main.rs` | 3030 pre-test LOC | Critical entry-point concentration |
| `crates/cli/src/command_exec.rs` | 2595 LOC | High dispatch concentration |
| `crates/cli/src/cli_args.rs` | 2262 LOC | High command-schema concentration |
| `crates/engine/src/export/mod.rs` | 928 pre-test LOC | High export-domain concentration |
| `crates/engine/src/board/mod.rs` | 716 pre-test LOC | High board-domain concentration |

Cross-cutting risk vectors:
- command schema, dispatch, and execution are concentrated in a few CLI files
- decomposition governance was incomplete for newly oversized modules
- budget coverage lag allowed large files to grow without explicit freeze caps

## 3. Structural Governance Rules

The following governance is now required:
- Every oversized source module must be explicitly budgeted in
  `scripts/check_file_size_budgets.py`.
- Oversized means pre-test source lines > 700.
- New oversized modules are blocked unless budgeted and tracked.
- Existing oversized modules are non-growth frozen via explicit caps.
- Caps must trend downward over time as decomposition lands.

CI enforcement:
- `scripts/check_file_size_budgets.py`
- `scripts/check_decomposition_coverage.py`
- `scripts/check_test_file_sizes.py`

## 4. Decomposition Sequencing

### Phase A: CLI monolith decomposition (highest priority)

Targets:
- `crates/cli/src/command_project.rs`
- `crates/cli/src/command_exec.rs`
- `crates/cli/src/cli_args.rs`
- `crates/cli/src/main.rs`

Shard strategy:
- split by command family and lifecycle:
  - `project/new_inspect`
  - `project/query`
  - `project/schematic_ops`
  - `project/board_ops`
  - `project/forward_annotation`
  - `project/export`
- isolate text/json rendering from mutation/query logic
- keep argument schemas grouped by command family, not one mega enum block

Exit criteria:
- no single CLI source file > 2000 LOC total
- no CLI entry-point pre-test block > 1200 LOC
- behavior parity maintained by existing test shards

### Phase B: engine export + board concentration reduction

Targets:
- `crates/engine/src/export/mod.rs`
- `crates/engine/src/board/mod.rs`

Shard strategy:
- split `export` by artifact family (`gerber`, `drill`, `bom`, `pnp`, shared)
- split `board` by read-surface, diagnostics/connectivity hooks, mutation helpers

Exit criteria:
- `export/mod.rs` becomes orchestrator only
- `board/mod.rs` retains API facade; heavy logic moved into submodules

### Phase C: budget normalization + ratcheting

Targets:
- all oversized modules in budget allowlist

Actions:
- reduce freeze caps in small increments after each decomposition merge
- remove freeze entries when file size is below normal limits

Exit criteria:
- no critical monoliths (>3000 LOC)
- no high monoliths (>2000 LOC) in CLI core modules
- sustained CI compliance without cap increases

## 5. Anti-Drift Operating Rules

- Structural-only PRs: no functional behavior changes.
- One decomposition PR should own one shard family and one write scope.
- Do not combine milestone feature work with structural decomposition in one PR.
- Any size-cap increase requires explicit written justification and reviewer signoff.
- Prefer adding modules over growing existing monoliths.

## 6. Reporting

Status reporting source of truth remains `specs/PROGRESS.md`.
This plan tracks decomposition governance and sequencing only.

Execution backlog:
- see `docs/DECOMPOSITION_BACKLOG.md` for PR-ready shards, write-set ownership,
  and merge order.
