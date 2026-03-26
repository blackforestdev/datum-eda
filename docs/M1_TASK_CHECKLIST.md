# M1 Task Checklist

> **Status**: Historical `M1` implementation checklist.
> This document records the intended `M1` execution posture when that
> milestone was active. Current repo status lives in `specs/PROGRESS.md`.

## Purpose

Turn the `M1` scope into a concrete, repo-specific checklist:
- what implementation areas are now legitimately open
- what should remain blocked
- what must be true before `M1` is considered complete

---

## 1. M1 Exit Gate Reference

From `specs/PROGRAM_SPEC.md`, `M1` requires:
- KiCad `.kicad_pcb` import
- KiCad `.kicad_sch` import
- Eagle `.brd` / `.sch` import
- schematic connectivity
- board connectivity
- airwire computation
- query API surface
- deterministic import and canonicalization
- corpus-backed golden tests
- fidelity thresholds for KiCad and Eagle

This checklist only covers the repository work needed to get there.

Priority rule for `M1`:
- KiCad is the primary implementation target.
- Eagle remains a bounded secondary migration path.
- Eagle support should not delay KiCad import, DOA2526 correctness,
  connectivity, or query API progress.

---

## 2. M1 Allowed Implementation Areas

The following areas are now open for real implementation work:

- [`crates/engine/src/import/`](../crates/engine/src/import/mod.rs)
- [`crates/engine/src/schematic/`](../crates/engine/src/schematic/mod.rs)
- [`crates/engine/src/board/`](../crates/engine/src/board/mod.rs)
- [`crates/engine/src/connectivity/`](../crates/engine/src/connectivity/mod.rs)
- [`crates/engine/src/api/mod.rs`](../crates/engine/src/api/mod.rs) for read-only query surface growth
- [`crates/test-harness/src/lib.rs`](../crates/test-harness/src/lib.rs) for corpus and golden helpers

Consumer crates were constrained at the time this checklist was authored:
- [`crates/cli/src/main.rs`](../crates/cli/src/main.rs): import/query only
- [`crates/engine-daemon/src/main.rs`](../crates/engine-daemon/src/main.rs): historical stub at the `M1` checkpoint
- [`mcp-server/server.py`](../mcp-server/server.py): historical stub at the `M1` checkpoint

Blocked until later milestones:
- [`crates/engine/src/drc/mod.rs`](../crates/engine/src/drc/mod.rs) (`M2`)
- [`crates/engine/src/ops/mod.rs`](../crates/engine/src/ops/mod.rs) (`M3`)
- [`crates/engine/src/export/mod.rs`](../crates/engine/src/export/mod.rs) (`M3+`)
- [`crates/engine/src/session/mod.rs`](../crates/engine/src/session/mod.rs) beyond minimal read-only design ownership

---

## 3. Recommended Order

1. Define minimal imported board and schematic domain types.
2. Add shared import-report and import-dispatch scaffolding for design files.
3. Implement KiCad schematic import for the initial supported subset.
4. Implement KiCad board import for the initial supported subset.
5. Implement Eagle schematic/board import for a narrow supported subset.
5. Implement schematic connectivity.
6. Implement board connectivity and airwire computation.
7. Implement read-only query summaries and info types.
8. Add corpus-backed golden tests and DOA2526 query assertions.

This order keeps the read-only product wedge honest and avoids dragging
checking or write operations forward.

Decision rule:
- if a question arises whether to spend time on Eagle edge cases or on
  KiCad/DOA2526 import fidelity, prioritize KiCad/DOA2526.

---

## 4. Immediate M1 Deliverables

The next concrete code deliverables should be:

- minimal `board` and `schematic` model types
- a real `Design` owner inside the engine facade
- import result plumbing for design files
- KiCad-first import dispatch
- first read-only query return types:
  - `BoardSummary`
  - `SchematicSummary`
  - `ComponentInfo`
  - `BoardNetInfo`
  - `SchematicNetInfo`

---

## 5. Definition of M1 Code Freeze

`M1` is ready to close when:
- DOA2526 imports successfully
- KiCad design import is correct for the claimed subset
- Eagle design import exists for a bounded claimed subset
- schematic connectivity is deterministic on corpus fixtures
- board connectivity and airwire counts are stable on corpus fixtures
- query APIs return stable structured data
- canonical import output is deterministic across repeated runs
- corpus-backed goldens exist for representative design imports

---

## 6. Immediate Next File Group

The next files that should receive real implementation work are:

- [`crates/engine/src/api/mod.rs`](../crates/engine/src/api/mod.rs)
- [`crates/engine/src/import/mod.rs`](../crates/engine/src/import/mod.rs)
- [`crates/engine/src/import/kicad/mod.rs`](../crates/engine/src/import/kicad/mod.rs)

That is the cleanest entry point into `M1` without violating milestone
boundaries.
