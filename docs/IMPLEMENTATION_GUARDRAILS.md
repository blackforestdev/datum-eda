# Implementation Guardrails

> **Status**: Historical pre-`M2` implementation control snapshot.
> This document is retained for milestone-history context only.
> It no longer describes the current repository state.
> Current status and milestone truth live in `specs/PROGRAM_SPEC.md`,
> `specs/PROGRESS.md`, and `specs/INTEGRATED_PROGRAM_SPEC.md`.

## Purpose

Define the exact allowed implementation surface for the current repository
state so development does not outrun the frozen specification set.

This document answers:
- what code is allowed to change now
- what scaffolding is safe to keep
- what modules are placeholders only
- what work is explicitly blocked until later milestones

---

## 1. Historical Readiness Assessment

This section records the repository shape when `M0` had closed and `M1`
implementation was opening. It is not a statement about the current repo.

Code that was already safe and aligned with completed `M0` work in that
snapshot:
- `crates/engine/src/error.rs`
- `crates/engine/src/api/mod.rs` as a narrow read-only/import facade
- `crates/engine/src/ir/geometry.rs`
- `crates/engine/src/ir/ids.rs`
- `crates/engine/src/ir/serialization.rs`
- `crates/engine/src/ir/units.rs`
- `crates/engine/src/rules/ast.rs`
- `crates/engine/src/rules/validate.rs`
- `crates/engine/src/rules/eval.rs`
- `crates/engine/src/pool/`
- `crates/engine/src/import/`
- `crates/test-harness/`
- `crates/cli/`

Code that was still scaffolding only in that snapshot:
- `crates/engine/src/board/`
- `crates/engine/src/connectivity/`
- `crates/engine/src/drc/`
- `crates/engine/src/export/`
- `crates/engine/src/ops/`
- `crates/engine/src/schematic/`
- `crates/engine/src/session/`
- `crates/engine-daemon/`
- `mcp-server/`

---

## 2. Current Allowed Surface (`M1`)

The following implementation areas are open for real work in `M1`:

### `crates/engine/src/ir/`

Allowed:
- geometry primitives
- unit conversion
- import identity helpers
- deterministic serialization

Not allowed:
- domain-specific geometry algorithms beyond trivial helpers
- importer-specific path construction logic beyond identity helpers
- persistence caches or file IO policies

### `crates/engine/src/rules/`

Allowed:
- `RuleScope` AST
- `Rule`, `RuleType`, `RuleParams`
- serialization of rule objects
- structural validation of the AST shape

Not allowed:
- full evaluator behavior tied to board/pool objects
- natural language rule parsing
- rule application against geometry or nets

### `crates/engine/src/pool/`

Allowed:
- spec-defined pool types
- JSON serialization of pool types
- SQLite index scaffolding and search primitives
 - import population from supported importers

Not allowed:
- manual authoring UX
 - future manufacturing/export logic

### `crates/engine/src/import/`

Allowed in `M1`:
- shared import-report plumbing
- Eagle design import
- KiCad design import
- Eagle library import maintenance
- source-format-to-canonical mapping for the supported subset

Not allowed:
- write-back
- source-format preservation features beyond explicit spec scope

### `crates/engine/src/schematic/`

Allowed in `M1`:
- imported schematic domain model
- supporting authored/derived distinctions needed for import and query

Not allowed:
- schematic write operations
- native authoring-only features from `M4`

### `crates/engine/src/board/`

Allowed in `M1`:
- imported board domain model
- supporting authored/derived distinctions needed for import and query

Not allowed:
- write operations
- routing behavior

### `crates/engine/src/connectivity/`

Allowed in `M1`:
- schematic connectivity
- board connectivity
- airwire computation
- deterministic recomputation for imported designs

### `crates/test-harness/`

Allowed:
- deterministic serialization checks
- golden file utilities
- corpus manifest plumbing for `M0`

Not allowed:
- fake DRC/ERC logic to unblock tests

---

## 3. Placeholder-Only Modules

The following modules may exist, but they must not accumulate real milestone
logic yet.

### `drc`
Blocked until:
- `M2`

### `ops`
Blocked until:
- `M3`

### `export`
Blocked until:
- `M3` write-back for source formats
- `M4` native/manufacturing export

### `session`
Blocked until:
- engine-held multi-project/session behavior becomes real in `M2`

---

## 4. Consumer Crates

### `crates/cli`

Allowed now:
- `M0` import and pool-search behavior
- import/query command growth that maps to real engine APIs

Blocked:
- real command behavior before matching engine APIs exist
- direct file parsing in the CLI

### `crates/engine-daemon`

Allowed in that snapshot:
- daemon shell only in the pre-`M2` repository state

Blocked:
- JSON-RPC implementation before `M2`
- session behavior beyond placeholder messaging

### `mcp-server`

Allowed in that snapshot:
- stub entrypoint in the pre-`M2` repository state
- comments and TODOs referencing `MCP_API_SPEC.md`

Blocked:
- real tool registration
- fallback design logic
- direct file parsing

---

## 5. Required Coding Order

Before any ERC/DRC work:
1. implement design importers
2. implement schematic connectivity
3. implement board connectivity
4. implement query summaries and info types
5. prove deterministic corpus behavior

Before any write operations:
1. prove `M2` query/check surface through CLI and MCP

---

## 6. Current Scaffold Risks

The current scaffold has one structural risk:
- `crates/engine/src/lib.rs` publicly exposes many later-milestone modules,
  which can encourage implementation to spread too early

Policy response:
- module exposure is tolerated for now as workspace scaffolding
- real logic must still obey milestone boundaries from this document

The current scaffold does **not** need a structural rewrite before `M0`
work begins.

---

## 7. Stop Conditions

Stop and update specs or plans before coding if:
- a needed type is not defined in `specs/`
- a file format detail is missing from the controlling spec
- a command/method name is not present in the controlling API surface
- a change would introduce a new subsystem or crate
- a change would make `docs/` the effective source of truth again

---

## 8. Immediate M0 Tasks

1. Implement spec-defined pool types in `crates/engine/src/pool/`
2. Implement pool JSON round-trip tests
3. Implement SQLite pool index scaffolding
4. Add deterministic golden-file helpers in `crates/test-harness/`
5. Keep all non-`M0` crates and modules in stub form
