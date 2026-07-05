# Product Mechanics Decision 018: Genesis Provenance Boundary

> **Status**: Ratified.
> **Date**: 2026-07-05.
> **Scope**: Native project genesis, t=0 provenance, mutation journal boundary.
> **Depends on**:
> `PRODUCT_MECHANICS_000D` (storage and versioning model),
> `PRODUCT_MECHANICS_001` (canonical edit model),
> `docs/CANONICAL_IR.md`,
> `specs/NATIVE_FORMAT_SPEC.md`.

## Problem

Native project genesis creates the four root shards required before a design
history exists: `project.json`, `schematic/schematic.json`, `board/board.json`,
and `rules/rules.json`. Phase 2 moved that bootstrap into the engine-owned
native-write facade and locked it with byte-identity, idempotence, resolver
roundtrip, and empty-journal tests. The remaining open question was whether
genesis should also produce a t=0 provenance record.

Adding a genesis record to the mutation journal would blur the boundary the
substrate relies on: journal transactions are created by typed operation
batches, produce revisions, participate in undo/redo, and are counted from the
first committed mutation. Genesis has no prior mutable model revision and no
operation batch to replay.

## Decision

Genesis remains engine-owned, deterministic, atomic, resolver-validated, and
deliberately non-journaled.

There is no `TransactionKind::Genesis`, no zero-operation `TransactionRecord`,
and no reserved t=0 entry in `.datum/journal/transactions.jsonl`. Mutation
provenance starts at the first committed typed operation batch after project
creation. Repeat genesis over existing root ids remains byte-identical and
continues to leave the mutation journal empty.

If product requirements later need visible genesis evidence, add a dedicated
genesis evidence sidecar outside the mutation journal. That sidecar must be
excluded from transaction counts, undo/redo, object-revision production,
model-revision production, and journal replay. It must also preserve repeat
bootstrap byte identity for the four canonical root shards.

## Invariants

1. `project new` writes exactly the four canonical root shards through
   `bootstrap_native_project`.
2. Genesis writes are atomic per shard and immediately resolver-validated.
3. Re-running genesis with the same root ids is byte-identical.
4. A fresh native project has an empty mutation journal.
5. Every post-create authored source mutation must use a typed
   `OperationBatch` and the journaled commit path.

## Verification

The focused proof is `cargo test -p eda-engine api::native_write::genesis`.
The standard drift battery additionally runs `scripts/check_spec_governance.py`
and `scripts/check_schematic_private_writers.py`, which keep the decision
classified and keep project bootstrap delegated to engine genesis rather than
CLI-private JSON writers.
