# M0 Task Checklist

> **Status**: Non-normative implementation checklist derived from
> `specs/PROGRAM_SPEC.md`, `docs/IMPLEMENTATION_PLAN.md`, and
> `docs/IMPLEMENTATION_GUARDRAILS.md`.
> This document maps the `M0` milestone to the current repository files.
>
> **Phase state**: `M0` is complete as of 2026-03-24.
> This document is retained as a closure record, not as the active milestone
> checklist.

## Purpose

Turn the `M0` scope into a concrete, repo-specific checklist:
- what files already exist and are acceptable
- what files need implementation next
- what tests must exist before `M0` is considered complete

---

## 1. M0 Exit Gate Reference

From `specs/PROGRAM_SPEC.md`, `M0` requires:
- pool types implemented
- pool serialization round-trip
- SQLite pool index
- deterministic serialization
- Eagle `.lbr` import
- Eagle canonicalization round-trip
- deterministic Eagle re-import
- `RuleScope` IR
- UUID v5 import identity
- `.ids.json` sidecar

This checklist only covers the repository work needed to get there.

---

## 2. Current File Status

### Already acceptable for M0

- [`Cargo.toml`](../Cargo.toml)
- [`crates/engine/src/error.rs`](../crates/engine/src/error.rs)
- [`crates/engine/src/api/mod.rs`](../crates/engine/src/api/mod.rs)
- [`crates/engine/src/ir/mod.rs`](../crates/engine/src/ir/mod.rs)
- [`crates/engine/src/ir/geometry.rs`](../crates/engine/src/ir/geometry.rs)
- [`crates/engine/src/ir/ids.rs`](../crates/engine/src/ir/ids.rs)
- [`crates/engine/src/ir/serialization.rs`](../crates/engine/src/ir/serialization.rs)
- [`crates/engine/src/ir/units.rs`](../crates/engine/src/ir/units.rs)
- [`crates/engine/src/rules/mod.rs`](../crates/engine/src/rules/mod.rs)
- [`crates/engine/src/rules/ast.rs`](../crates/engine/src/rules/ast.rs)

### Placeholders to leave alone for now

- [`crates/engine/src/board/mod.rs`](../crates/engine/src/board/mod.rs)
- [`crates/engine/src/connectivity/mod.rs`](../crates/engine/src/connectivity/mod.rs)
- [`crates/engine/src/drc/mod.rs`](../crates/engine/src/drc/mod.rs)
- [`crates/engine/src/export/mod.rs`](../crates/engine/src/export/mod.rs)
- [`crates/engine/src/import/mod.rs`](../crates/engine/src/import/mod.rs)
- [`crates/engine/src/import/eagle/mod.rs`](../crates/engine/src/import/eagle/mod.rs)
- [`crates/engine/src/import/kicad/mod.rs`](../crates/engine/src/import/kicad/mod.rs)
- [`crates/engine/src/ops/mod.rs`](../crates/engine/src/ops/mod.rs)
- [`crates/engine/src/schematic/mod.rs`](../crates/engine/src/schematic/mod.rs)
- [`crates/engine/src/session/mod.rs`](../crates/engine/src/session/mod.rs)
- [`crates/cli/src/main.rs`](../crates/cli/src/main.rs)
- [`crates/engine-daemon/src/main.rs`](../crates/engine-daemon/src/main.rs)
- [`mcp-server/server.py`](../mcp-server/server.py)

### Files that received substantive M0 work

- [`crates/engine/src/pool/mod.rs`](../crates/engine/src/pool/mod.rs)
- [`crates/engine/src/rules/eval.rs`](../crates/engine/src/rules/eval.rs)
- [`crates/engine/src/rules/validate.rs`](../crates/engine/src/rules/validate.rs)
- [`crates/test-harness/src/lib.rs`](../crates/test-harness/src/lib.rs)
- [`crates/engine/src/import/mod.rs`](../crates/engine/src/import/mod.rs)
- [`crates/engine/src/import/eagle/mod.rs`](../crates/engine/src/import/eagle/mod.rs)
- [`crates/cli/src/main.rs`](../crates/cli/src/main.rs)

---

## 3. Implementation Tasks

### Task Group A: Pool Domain Types

Target file:
- [`crates/engine/src/pool/mod.rs`](../crates/engine/src/pool/mod.rs)

Required:
- define `Pin`
- define `Unit`
- define `Gate`
- define `Entity`
- define `Pad`
- define `Package`
- define `PadMapEntry`
- define `Part`

Likely follow-up split after initial implementation:
- `model.rs`
- `storage.rs`
- `index.rs`

Done when:
- types match `specs/ENGINE_SPEC.md` for `M0` scope
- serde round-trip works
- no speculative fields beyond spec

### Task Group B: Pool Serialization Tests

Primary location:
- inline tests in [`crates/engine/src/pool/mod.rs`](../crates/engine/src/pool/mod.rs)
- reusable helpers in [`crates/test-harness/src/lib.rs`](../crates/test-harness/src/lib.rs)

Required:
- struct → JSON → struct round-trip tests
- deterministic key-order serialization tests
- representative sample objects for Unit/Entity/Package/Part

Done when:
- pool round-trip tests pass
- deterministic serialization tests pass on repeated runs

### Task Group C: Rule Validation Surface

Target files:
- [`crates/engine/src/rules/validate.rs`](../crates/engine/src/rules/validate.rs)
- [`crates/engine/src/rules/eval.rs`](../crates/engine/src/rules/eval.rs)

Required in M0:
- structural validation for `RuleScope`
- validation of `Rule` shape
- evaluator stub that explicitly supports only the leaf-node milestone surface

Not required in M0:
- rule application to actual board objects

Done when:
- invalid AST shapes fail clearly
- unsupported nodes return explicit errors, not silent behavior

### Task Group D: SQLite Pool Index

Primary implementation location:
- start in [`crates/engine/src/pool/mod.rs`](../crates/engine/src/pool/mod.rs)
- split to `index.rs` once the shape is clear

Required:
- create index database
- insert core pool records
- keyword search
- parametric-value search

Done when:
- the minimum `M0` program-spec index behavior is testable without importers

### Task Group E: Test Harness Foundation

Target file:
- [`crates/test-harness/src/lib.rs`](../crates/test-harness/src/lib.rs)

Required:
- golden JSON comparison helper
- repeated serialization helper
- fixture path helper

Deferred:
- DRC parity runners
- full corpus loaders for later milestones

Done when:
- engine tests can reuse harness helpers for deterministic JSON assertions

### Task Group F: Import Identity Sidecar

Target areas:
- [`crates/engine/src/ir/ids.rs`](../crates/engine/src/ir/ids.rs)
- likely new helper under `crates/engine/src/import/`

Required:
- `.ids.json` data shape
- read/write helpers
- tests for stable restore behavior

Done when:
- the sidecar contract from `IMPORT_SPEC.md` is represented in code
- repeated imports can be proven to preserve IDs once importer work begins

---

## 4. Closure Summary

Implemented during `M0`:
1. Pool domain types and deterministic pool serialization.
2. Rule AST validation and milestone-gated leaf evaluation.
3. SQLite pool index and search primitives.
4. `.ids.json` sidecar helpers and deterministic import identity.
5. Eagle `.lbr` library importer with 20+ fixture corpus.
6. Canonical golden subset for representative Eagle libraries.
7. Narrow `M0` CLI surface for Eagle `.lbr` import reports and pool search.

---

## 5. Definition of M0 Code Freeze

`M0` is implementation-ready to close when:
- pool types exist in code
- pool serialization is tested
- deterministic serialization is proven through harness helpers
- rule AST validation is in place
- SQLite index basics work
- `.ids.json` helpers exist
- non-`M0` modules remain placeholder-only

---

## 6. Outcome

`M0` closure assessment:
- Exit criteria from `specs/PROGRAM_SPEC.md` are satisfied.
- The Eagle corpus breadth gate is met (`22` `.lbr` fixtures).
- The Eagle golden subset exists and should expand gradually during `M1`,
  not as a condition for keeping `M0` open.

Active work should now move to `M1`.
