# Implementation Plan

> **Status**: Non-normative implementation planning derived from the
> controlling specifications in `specs/`.
> This document sequences implementation work and assigns module ownership.
> If it conflicts with a formal specification, the spec wins.

## Purpose

Translate the frozen specification set into an executable development plan:
- workspace and crate ownership
- module boundaries
- implementation order
- test-harness priorities
- milestone-ready coding gates

This document is the bridge between specification and code.

---

## 1. Current Workspace

The existing workspace already contains the right top-level shape:

```text
Cargo.toml
crates/
  engine/
  cli/
  engine-daemon/
  test-harness/
mcp-server/
docs/
specs/
```

This structure should be preserved for `M0-M2`.

Do not add a `gui` crate yet.
The GUI is a consumer that begins at `M7` and should not shape the core.

---

## 2. Crate Ownership

### `crates/engine`

Owns:
- canonical domain types
- invariants and validation
- import identity helpers
- serialization
- pool model and search surface
- schematic connectivity
- ERC
- board connectivity
- DRC
- operation model
- public `Engine` facade

Internal module structure:

```text
engine/src/
  lib.rs
  error.rs
  api/
  ir/
  pool/
  board/
  schematic/
  rules/
  import/
  connectivity/
  erc/
  drc/
  ops/
```

Rules:
- `api/` is the only public consumer-facing boundary inside the crate
- internal modules must not define parallel public APIs
- no CLI, socket, or MCP concerns in `engine`

### `crates/cli`

Owns:
- command parsing
- output formatting
- CLI-to-engine request mapping
- exit code behavior

Must not own:
- import logic
- checking logic
- persistence logic

### `crates/engine-daemon`

Owns:
- engine process lifecycle
- JSON-RPC over Unix socket
- session management for one loaded design

Must not own:
- business logic duplicated from `engine`

### `crates/test-harness`

Owns:
- corpus loading
- golden-file comparison
- deterministic serialization tests
- benchmark runners
- fixture normalization

### `mcp-server/`

Owns:
- Python MCP adapter
- mapping MCP tools to daemon RPC calls

Must remain thin.

---

## 3. Implementation Sequence

### Phase A: M0 Foundation

Goal:
- compile the canonical types
- prove deterministic serialization
- prove import identity
- stand up the test harness

Order:
1. `engine/ir`
2. `engine/rules` AST and support matrix
3. `engine/pool` core types
4. deterministic JSON serializer
5. import UUID and `.ids.json` helpers
6. `test-harness` golden utilities

First code deliverables:
- `Point`, `Rect`, `Polygon`, `Arc`
- shared enums and reference types
- pool types from `ENGINE_SPEC.md`
- `RuleScope` and rule validation
- canonical JSON serializer
- UUID v5 import identity helpers

Tests required before moving on:
- serialization byte-stability
- UUID determinism
- pool round-trip JSON tests
- rule AST parse/serialize tests

### Phase B: M1 Import + Query

Goal:
- ingest KiCad/Eagle designs into canonical IR
- resolve connectivity
- answer read-only queries

Current status:
- This phase is in progress and materially ahead of the original minimal plan.
- Implemented now:
  - Eagle `.lbr` import into the pool
  - KiCad `.kicad_pro` metadata import
  - KiCad `.kicad_sch` canonical import slice
  - KiCad `.kicad_pcb` canonical import slice
  - board and schematic summaries
  - board/schematic net info
  - imported footprint pads and pad-derived board net pins
  - importer-backed board airwire computation via `get_unrouted`
  - board components
  - schematic labels, symbols, ports, buses, hierarchy, and no-connect queries
  - schematic connectivity diagnostics
  - board structural diagnostics including partially-routed-net reporting
  - early ERC prechecks
  - CLI read/check surface
  - daemon JSON-RPC read/check surface
  - Python MCP stdio read/check surface

Remaining work to close `M1`:
- broaden DOA2526-anchored KiCad corpus coverage
- finish deterministic connectivity confidence on richer hierarchy cases
- complete remaining read-only query families still listed in the formal exit gate
- prove board connectivity/airwire correctness on the target corpus, not just
  on focused fixtures
- validate board partial-route behavior and airwire counts against DOA2526 and
  companion KiCad boards
- keep Eagle design import bounded and secondary to KiCad correctness

Priority:
- KiCad first: this is the primary live ecosystem and the main product wedge
- Eagle second: bounded migration support and regression coverage only
- Do not let Eagle parity delay KiCad import, connectivity, or query work

Order:
1. import scaffolding and reports
2. Eagle library import
3. KiCad schematic import
4. KiCad board import
5. Eagle schematic/board import for a narrow supported subset
6. schematic connectivity engine
7. board connectivity and airwire computation
8. query API surface
9. corpus and golden hardening on DOA2526 and companion fixtures

First code deliverables:
- `ImportReport`
- import mappers for minimal supported features
- `SchematicConnectivityGraph`
- board net/pad connectivity graph
- summaries and query info types
- `CheckReport`
- daemon JSON-RPC read/check dispatcher
- MCP stdio read/check tool host

Tests required before moving on:
- corpus import succeeds
- deterministic re-import
- schematic connectivity golden tests
- board connectivity golden tests
- DOA2526 query correctness, including airwire/unrouted behavior
- daemon read/check method tests
- MCP stub read/check self-tests

Implementation note:
- If a tradeoff appears between "more Eagle edge-case coverage" and
  "faster KiCad/DOA2526 correctness," choose KiCad/DOA2526.

### Phase C: M2 ERC + DRC + CLI/MCP

Goal:
- make the product wedge real
- expose checking and query surfaces to CLI and MCP

Current status:
- The CLI, daemon, and MCP transport shells are already present.
- The remaining `M2` burden is no longer “stand up surfaces”; it is:
  - expand ERC from the current precheck/rule subset to the specified corpus-backed set
  - implement DRC
  - finish CLI/MCP catalog parity
  - harden CI behavior and corpus-backed confidence

Order:
1. ERC engine
2. DRC engine
3. shared checking report plumbing
4. complete `cli` read/check surface
5. complete `engine-daemon` session shell
6. complete `mcp-server` tool adapter

First code deliverables:
- `ErcReport`, `DrcReport`
- waiver matching
- `tool drc`
- remaining daemon methods matching `MCP_API_SPEC.md`
- MCP registration and smoke tests against the real daemon

Tests required before moving on:
- ERC corpus checks
- DRC corpus checks
- CLI exit code tests
- daemon JSON-RPC method tests
- MCP end-to-end smoke test against DOA2526

---

## 4. First Coding Gates

These are stop/go gates, not aspirations.

### Gate 1: “M0 Ready”

Do not begin importer work until:
- canonical serialization is deterministic
- UUID import identity is proven
- pool types compile cleanly
- `test-harness` can run golden comparisons

### Gate 2: “M1 Ready”

Do not begin ERC/DRC until:
- import produces canonical IR reliably
- schematic connectivity is deterministic
- board connectivity and airwires are correct on corpus samples
- query API returns stable results

Note: early ERC prechecks and partial read/check transport may exist before this
gate closes, but they do not replace the formal `M1` exit criteria.

### Gate 3: “M2 Ready”

Do not begin M3 write operations until:
- CLI and MCP can drive read/check flows cleanly
- waiver matching is deterministic
- corpus-based ERC/DRC confidence is established

---

## 5. Module Ownership Details

### `engine/ir`
- units
- geometry
- ids
- serialization

### `engine/pool`
- Unit, Gate, Entity, Package, Part, Padstack, Symbol
- pool storage helpers
- pool query helpers

### `engine/schematic`
- authored schematic model
- summaries and query info types

### `engine/board`
- authored board model
- summaries and query info types

### `engine/import`
- source readers
- source-to-IR mapping
- `.ids.json` handling

### `engine/connectivity`
- schematic connectivity
- board connectivity
- airwires

### `engine/erc`
- pin semantics
- rule execution
- waiver application

### `engine/drc`
- rule evaluation
- geometric checks
- connectivity checks

### `engine/ops`
- operation trait
- diffs
- transactions
- undo/redo

---

## 6. Test Harness Priorities

Priority order:
1. deterministic serialization
2. import identity
3. schematic connectivity
4. board connectivity
5. query API correctness
6. ERC correctness
7. DRC correctness
8. CLI/MCP end-to-end smoke tests

Minimum corpus split:
- Eagle libraries for `M0`
- imported KiCad/Eagle designs for `M1`
- known-bad and known-clean schematic/board cases for `M2`

Golden artifacts should include:
- canonical JSON
- query snapshots
- schematic connectivity snapshots
- ERC reports
- DRC reports

---

## 7. Guardrails

- Do not add behavior not already defined in `specs/`.
- Do not let `cli`, `engine-daemon`, or `mcp-server` bypass `engine` APIs.
- Do not persist derived data as canonical state.
- Do not let GUI concerns enter the engine before `M7`.
- Do not begin `M3` or `M4` work while `M2` acceptance gaps remain open.

---

## 8. Immediate Next Tasks

1. Finalize `crates/engine` module tree to match this plan.
2. Freeze the allowed implementation surface for `M0`.
3. Implement deterministic serialization and import identity first.
4. Build `test-harness` before importer breadth expands.
5. Track progress against `specs/PROGRAM_SPEC.md`, not ad hoc TODOs.
