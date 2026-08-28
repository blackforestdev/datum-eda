# REV-I00 Product Revision Engine Execution-Authorization Packet

> **Status:** owner review required.
>
> **Boundary:** approval authorizes REV-I01 only. It does not authorize the full
> Revision program, Global Preferences or Publish implementation, a dependency,
> a provider, a GUI slice, or concurrent execution.

## 1. Findings first

### Finding 1 — the former policy blocker is closed

Product Mechanics 037 now supplies the exact Revision seam: Preferences resolves
only a new-Project `ProjectPolicySeed` snapshot; Project mutation authority
copies it atomically with a durable itemized receipt; existing Projects never
live-follow later preference changes. Presentation, Capability, and
WorkflowDefault cannot cross. Revision remains sole authority for revision
identity, policy after genesis, records, gates, and lifecycle.

Evidence: `docs/decisions/PRODUCT_MECHANICS_037_GLOBAL_PREFERENCES_ENGINE.md:20-33,56-66`;
Product Mechanics 034 at
`docs/decisions/PRODUCT_MECHANICS_034_PRODUCT_REVISION_ENGINE.md:43-64,188-207`.

### Finding 2 — REV-I01 is integrity work, not product workflow

The current engine already has guarded `OperationBatch` commit, technical
`ModelRevision`, a Project resolver, JSONL journal/cursor, undo/redo, native-
write, CLI/daemon/MCP fences, and broad source-shard replay. It does not yet have
the REV-I01 algorithm-qualified integrity root, canonical immutable authority
records/blobs, atomic staged authority writes tied to one journal commit point,
complete last-good/read-only recovery, or independent backup/restore
verification. Those missing facts are why REV-I01 precedes every new revision
record type.

Evidence: `specs/PRODUCT_REVISION_ENGINE_IMPLEMENTATION_PLAN.md:35-75,109-134`;
`research/documentation-system/REV_C01_INTERNAL_AUTHORITY_AUDIT.md:94-142,250-285`;
current modules `crates/engine/src/substrate/transaction.rs`,
`journal.rs`, `journal_io.rs`, `project_resolver.rs`, and `replay.rs`.

### Finding 3 — existing monolith debt constrains ownership

Source-health passes, but `substrate/journal.rs` and `substrate/replay.rs` are
grandfathered oversized modules. REV-I01 may not grow them or hide work in
`substrate/mod.rs`, `operation.rs`, `operation_application.rs`, GUI protocol, or
CLI dispatch. Product Mechanics 022 requires cohesive extraction and downward
ratcheting whenever legacy debt is touched.

The selected ownership is a new cohesive engine-only
`crates/engine/src/revision/` domain with bounded modules for canonical records,
integrity, store staging/promotion, recovery/verification, and test support.
Integration with the canonical transaction path occurs through a narrow commit
hook; it does not create a second mutation facade or writer.

Evidence: Product Mechanics 022 normal limits and burn-down law at
`docs/decisions/PRODUCT_MECHANICS_022_SOURCE_HEALTH_GOVERNANCE.md:42-88`;
current measured files: `substrate/journal.rs` 803 lines,
`substrate/replay.rs` 809, `substrate/mod.rs` 654,
`substrate/operation.rs` 646, and `substrate/operation_application.rs` 708.

### Finding 4 — the first persistence choice can remain dependency-free

REV-I01 selects a Datum-owned Project-local authority store under `.datum/`:
versioned canonical JSON immutable records/manifests, content-addressed blobs,
algorithm-qualified digests, staged temporary generations, and one atomic head
promotion coordinated with the accepted transaction. A complete manifest is
the unit of verification, backup, restore, and recovery. Partial generations
are never visible as Release authority. The resolver exposes last complete or
typed read-only diagnostic state.

This is a bounded initial implementation choice under the logical
`RevisionAuthorityStore`; it does not select SQLite, a database, a crypto
library, Git, a network client, signature provider, PDF engine, or external
service. Digest algorithms use only already-authorized workspace facilities;
any missing algorithm or library returns to PM-029 before code changes.

Evidence: Product Mechanics 034 standalone/local integrity law at
`docs/decisions/PRODUCT_MECHANICS_034_PRODUCT_REVISION_ENGINE.md:120-141` and
REV-I01 at `specs/PRODUCT_REVISION_ENGINE_IMPLEMENTATION_PLAN.md:109-134`.

### Finding 5 — proof fixtures are adequate for REV-I01, not yet for REV-I12

REV-I01 uses deterministic working copies of the checked-in
`native_authored_baseline_v1` Project (library, schematic, board, and rules) and
the valid `profile-divergence-authored-copper` and `via-available` native
Projects. Fault injection wraps those real product paths; it does not replace
them with a revision-only toy store. The current `/tmp` DOA2526 import is useful
interactive evidence but is not durable and cannot be the sole acceptance
fixture.

Before REV-I12 authorization, the owner must approve a durable broader real-
project corpus covering actual manufacturing and Publish dependencies. REV-I00
does not pretend that later corpus already exists.

Evidence: `crates/test-harness/testdata/library/native_authored_baseline_v1/`;
`crates/test-harness/testdata/quality/native_project_validation_manifest_v1.json`;
real-project law at `specs/PRODUCT_REVISION_ENGINE_IMPLEMENTATION_PLAN.md:461-472`.

### Finding 6 — policy gates are green; the parallel test baseline has two
tracked terminal flakes

The committed packet records the exact gate and guarded Cargo outcomes run at
the packet commit. Any later code-start claim must rerun them from its own head;
these results are evidence, not a lease or waiver.

Dependency authority, cargo-resource policy, source health, specification
governance/parity, progress and project-state consistency, schematic private-
writer, daemon write parity, resolver raw-load fences, all seven native Project
fixtures, and strict locked/offline all-target Clippy passed. Evidence
traceability is green after registering the four Claude-owned agent-authority
renders that landed concurrently with this packet.

The guarded locked/offline workspace suite is not recorded as clean. Its first
sandbox run reached three accessibility socket tests that require credentials
unavailable in the managed sandbox. Two escalated runs then each exposed a
different pre-existing terminal timing failure:
`vsusp_bg_fg_continues_after_suspend_and_resume` did not observe `BG-OK`, and
`termination_cancels_backpressured_input_without_reader` observed a busy input
queue. Each exact failing test passed when rerun in isolation. The independent
failures are tracked as `dat-terminal-job-control-workspace-flakes-lby`; they
are not waived. REV-I01 closure requires a clean full guarded suite without a
retry, in addition to focused proof.

### Finding 7 — approval is deliberately narrow

Approval creates execution authority only for REV-I01 technical integrity and
atomic authority substrate. REV-I02 typed product authority, Project policy,
Change, impact, Release, reproduction, adapters, CLI/MCP expansion, GUI,
enterprise workflow, standards/migration closure, production acceptance,
Publish, and every new dependency remain unauthorized. REV-I01 completion must
return to a fresh Frontier authorization state; it cannot roll into REV-I02.

## 2. Exact REV-I01 slice proposed for authorization

REV-I01 may:

1. reconcile accepted transaction-tip identity without equating it to
   EngineeringRevision;
2. create the bounded `engine::revision` integrity/store modules described in
   Finding 3;
3. implement versioned canonical envelopes, algorithm-qualified digests,
   integrity roots, immutable staged records/blobs, a complete-generation
   manifest, and one head promotion tied to the canonical commit path;
4. expose last-complete recovery or typed read-only diagnostics, never partial
   authority;
5. implement complete backup/export manifests, restore, and independent
   verification for this substrate;
6. retain expected-model-revision and accepted-tip fences through all existing
   mutation surfaces; and
7. add only the operations/queries needed to prove this substrate, without
   defining REV-I02 product record kinds.

It may not create ConfigurationItem, EngineeringChange, EngineeringRevision,
Release, ControlledDocument, or other REV-I02+ authority; change user-visible
Revision UX; implement Preferences or Publish; introduce a private writer; or
add a dependency.

## 3. REV-I01 proof gates

Acceptance requires committed, addressable evidence for:

- crash interruption before/after every stage, manifest, journal, and head-
  promotion boundary;
- corrupt/truncated manifest, missing blob, wrong digest, stale expected model
  revision, stale accepted tip, and concurrent writer refusal;
- last-complete recovery and explicit read-only diagnostic fallback, with no
  partially visible release/authority state;
- canonical round trip, deterministic ordering, algorithm qualification,
  backup completeness, restore equivalence, and independent verification;
- native-write, CLI, daemon, MCP, proposal, undo/redo fence preservation and no
  private writer;
- unchanged ordinary Design mutation semantics on the three named checked-in
  Projects; and
- all standing non-compiling gates plus guarded locked/offline workspace tests
  and strict all-target Clippy.

## 4. Dependency, migration, and visual posture

- **Dependency:** none requested. PM-029 remains a hard stop.
- **Migration:** REV-I01 adds versioned empty/initial authority-store bootstrap
  and safe read-only fallback only. It does not reinterpret free-form revision
  strings, proposals, approvals, waivers, deviations, Part state, library
  provenance, ZoneFill staleness, or the CLI `release` check profile; those stay
  assigned to later slices.
- **Visual:** REV-I01 has no new visible workflow. If recovery or corruption
  produces user-visible behavior beyond existing typed diagnostics, execution
  pauses for a bounded Claude-owned render before clauses or UI are added.
- **Project policy:** REV-I01 stores no preference source and consumes no live
  global preference. The PM-037 copy-once seam remains for REV-I03.

## 5. Owner boundary

<!-- OWNER:PRODUCT-REVISION-ENGINE:REV-I00:REVISION-ENGINE-EXECUTION -->

The exact question is:

> Does this committed baseline preserve Product Mechanics 034, Product
> Mechanics 037's copy-once Project-policy seam, Product Mechanics 022 module
> ownership, PM-029's no-new-dependency boundary, honest real-project evidence,
> and the listed proof gates closely enough to authorize REV-I01—and REV-I01
> only?

**Recommended response:** approve only the bounded integrity substrate because
it closes prerequisites without defining product revision records or starting
Preferences, Publish, GUI, or later Revision slices.

**Exact response format:**

```text
REVISION-ENGINE-EXECUTION: approve
```

or

```text
REVISION-ENGINE-EXECUTION: revise — <specific scope, dependency, migration, ownership, fixture, or proof correction>
```

<!-- EVIDENCE:PRODUCT-REVISION-ENGINE:REV-I00-PACKET -->
