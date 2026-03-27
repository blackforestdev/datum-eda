# Decomposition Backlog (PR-Ready)

Status: active.
Mode: structural-only refactors; no behavior changes.

## Operating Lanes

- `m4-feature` lane: functional M4 work only.
- `decomp` lane: file/module decomposition only.
- Never mix both lanes in one PR.

## PR Template Rules (Decomposition Lane)

Required in every decomposition PR description:
- Scope: structural only, no behavior changes.
- Write set: exact files/modules touched.
- Evidence: tests/checks run.
- Budget impact: before/after line counts for touched oversized files.

Required checks:
- `python3 scripts/check_file_size_budgets.py`
- `python3 scripts/check_decomposition_coverage.py`
- `python3 scripts/check_touched_monolith_growth.py`
- `python3 scripts/check_test_file_sizes.py --max-lines 700`
- `python3 scripts/check_alignment.py`
- `python3 scripts/check_progress_coverage.py`

Touched-monolith explicit policy:
- If a PR touches a known monolith file, that file must not grow.
- Structural PRs should reduce touched monolith line counts.
- Baselines are enforced in `scripts/check_touched_monolith_growth.py` and must
  be ratcheted downward after decomposition merges.

Structural-only rule:
- Decomposition lane PRs are structural-only by policy.
- They must not intentionally change CLI/API behavior, payload schemas, or
  milestone semantics.
- Any behavior change discovered during decomposition must be moved to a
  separate feature-lane PR.

## Tripwire Response Playbook

When a tripwire fails, use this runbook:

1. `check_touched_monolith_growth.py` fails:
- Action: split/move newly added logic into shards until touched monolith
  returns to baseline or below.
- Merge status: blocked until fixed.

2. `check_decomposition_coverage.py` fails:
- Action: add explicit budget entry in `scripts/check_file_size_budgets.py` and
  add/refresh the file in decomposition planning docs.
- Merge status: blocked until budget + plan coverage exists.

3. `check_file_size_budgets.py` fails:
- Action: decompose to get under cap, or reduce cap debt in the same PR.
- Exception policy: cap increases are discouraged; if unavoidable, require
  explicit written justification, owner signoff, and follow-up decomposition
  item in this backlog.
- Merge status: blocked unless exception is approved.

4. Mixed-lane PR (feature + decomposition) detected:
- Action: split into two PRs (`m4-feature` and `decomp`).
- Merge status: blocked until split.

## Sequenced Backlog

### Wave 1: CLI Entry-Point Decoupling

PR-D01:
- Goal: split `crates/cli/src/main.rs` test imports and command registration into dedicated modules.
- Write set:
  - `crates/cli/src/main.rs`
  - `crates/cli/src/main_tests.rs` (and existing `main_tests_*` modules only as wiring)
  - new `crates/cli/src/main_entry/` module tree
- Constraint: no command behavior changes.
- Exit: `main.rs` pre-test LOC reduced by at least 20%.

PR-D02:
- Goal: split `crates/cli/src/command_exec.rs` into domain dispatch modules.
- Write set:
  - `crates/cli/src/command_exec.rs`
  - new `crates/cli/src/command_exec/`:
    - `project.rs`
    - `query.rs`
    - `modify.rs`
    - `plan.rs`
    - `import_check.rs`
- Constraint: dispatch mapping unchanged.
- Exit: top-level `command_exec.rs` becomes routing facade.

### Wave 2: Project Command Monolith Decomposition

PR-D03:
- Goal: carve out native project query/read surfaces from `command_project.rs`.
- Write set:
  - `crates/cli/src/command_project.rs`
  - new `crates/cli/src/command_project/query/`
- Constraint: preserve existing JSON/text payload shapes.

PR-D04:
- Goal: carve out schematic mutation surfaces.
- Write set:
  - `crates/cli/src/command_project.rs`
  - new `crates/cli/src/command_project/schematic_ops/`
- Constraint: operation semantics and error strings unchanged.

PR-D05:
- Goal: carve out board mutation surfaces.
- Write set:
  - `crates/cli/src/command_project.rs`
  - new `crates/cli/src/command_project/board_ops/`
- Constraint: no CLI contract drift.

PR-D06:
- Goal: carve out forward-annotation artifact/review/apply surfaces.
- Write set:
  - `crates/cli/src/command_project.rs`
  - new `crates/cli/src/command_project/forward_annotation/`
- Constraint: keep action-id determinism and artifact schema behavior unchanged.

PR-D07:
- Goal: carve out export/compare/validate surfaces.
- Write set:
  - `crates/cli/src/command_project.rs`
  - new `crates/cli/src/command_project/export/`
- Constraint: no output format changes.

PR-D08:
- Goal: final pass to convert `command_project.rs` into orchestration facade.
- Write set:
  - `crates/cli/src/command_project.rs`
  - `crates/cli/src/command_project/mod.rs`
- Exit:
  - `command_project.rs` <= 1500 LOC
  - submodules own all heavy logic paths.

### Wave 3: CLI Args Schema Decomposition

PR-D09:
- Goal: split `cli_args.rs` by command family.
- Write set:
  - `crates/cli/src/cli_args.rs`
  - new `crates/cli/src/cli_args/`:
    - `project_args.rs`
    - `query_args.rs`
    - `modify_args.rs`
    - `plan_args.rs`
    - `import_check_args.rs`
- Constraint: clap interface remains backward compatible.

### Wave 4: Engine Concentration Reduction

PR-D10:
- Goal: split `crates/engine/src/export/mod.rs` by artifact family.
- Write set:
  - `crates/engine/src/export/mod.rs`
  - new `crates/engine/src/export/`:
    - `gerber.rs`
    - `drill.rs`
    - `bom.rs`
    - `pnp.rs`
    - `shared.rs`
- Constraint: exported artifact content unchanged.

PR-D11:
- Goal: split `crates/engine/src/board/mod.rs` into facade + helpers.
- Write set:
  - `crates/engine/src/board/mod.rs`
  - new `crates/engine/src/board/` helper modules:
    - `queries.rs`
    - `diagnostics.rs`
    - `connectivity_hooks.rs`
    - `mutations.rs`
- Constraint: board API and diagnostics/check outputs unchanged.

## Merge Order and Parallelization

Recommended order:
1. D01 -> D02
2. D03, D04, D05 in sequence (same write root)
3. D06 -> D07 -> D08
4. D09
5. D10 and D11 (can run in parallel if write sets remain disjoint)

Parallel-safe set:
- `m4-feature` PRs can merge between decomposition PRs.
- Decomposition PRs rebase onto latest `main` before merge.

## Ratchet Policy

After each merged decomposition PR touching oversized modules:
- lower corresponding freeze cap in `scripts/check_file_size_budgets.py`
- never increase a cap unless there is written exception approval
- keep `scripts/check_decomposition_coverage.py` green

## Completion Criteria

- `crates/cli/src/command_project.rs` <= 1500 LOC
- `crates/cli/src/command_exec.rs` <= 1200 LOC
- `crates/cli/src/cli_args.rs` <= 1200 LOC
- `crates/cli/src/main.rs` pre-test <= 1200 LOC
- `crates/engine/src/export/mod.rs` pre-test <= 350 LOC
- `crates/engine/src/board/mod.rs` pre-test <= 450 LOC
- no new oversized source module without explicit budget and plan entry
