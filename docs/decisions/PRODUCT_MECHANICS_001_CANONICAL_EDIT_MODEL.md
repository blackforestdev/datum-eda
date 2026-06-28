# Product Mechanics Decision 001: Canonical Edit Model

Status: draft hypothesis + how/mechanism woven 2026-06-18; identity,
ComponentInstance, journal, and M7 sequencing ratified; remaining forks open.
Date: 2026-06-14

Driven by:
- `docs/decisions/PRODUCT_MECHANICS_000_UNIFIED_DESIGN_MODEL.md`
- `docs/decisions/PRODUCT_MECHANICS_000D_STORAGE_AND_VERSIONING_MODEL.md`
- `docs/decisions/PRODUCT_MECHANICS_000E_MULTI_SHEET_MULTI_BOARD_MODEL.md`
- `docs/decisions/PRODUCT_MECHANICS_000F_LIVE_PRODUCTION_PROOF_MODEL.md`
- `docs/audits/scope-integration/DATUM_PRODUCT_MECHANICS_RESEARCH_SYNTHESIS.md`
- `docs/DATUM_PRODUCT_MECHANICS.md`
- `docs/audits/scope-integration/DATUM_PRODUCT_MECHANICS_REVIEW_AGENDA.md`
- `docs/audits/scope-integration/NORTH_STAR_PROJECT_AUDIT.md`

## Decision Scope

Define the single mutation model for Datum's unified `DesignModel`.

This decision controls how design-state changes are created, previewed,
committed, undone, audited, and exposed across:
- manual GUI tools
- CLI commands
- MCP tools
- terminal-launched agents
- optional assistant surfaces
- imports and import repair
- ECO flows
- DRC/ERC correction proposals
- scripts and automation

## Product Intent

Datum must not become a collection of disconnected editing systems.

Manual users, CLI users, scripts, external agents, built-in assistants,
importers, checkers, and routers must all operate through the same inspectable
mutation primitives.

The model must support both:
- fast direct manual editing
- safe review/apply workflows for automation and AI

## Decision

Datum shall use a canonical edit model built around:
- `DesignModel`
- `Operation`
- `OperationBatch`
- `Proposal`
- `Transaction`
- `Diff`
- `Provenance`

Every design-state mutation, from every surface, must resolve to typed
operations and commit through a single `commit()` primitive that appends to the
project journal. There is no second commit path and no private byte-writer.

Operations target the canonical `DesignModel` — the single in-memory authority
assembled by the engine-owned `ProjectResolver` from segmented source shards on
disk — not isolated schematic, PCB, or library files. Shards are persistence
partitions, never authorities. Schematic, PCB, symbol, footprint, rules,
manufacturing, and analysis surfaces are derived projections over the model.

Automation, AI, ECO, import repair, DRC/ERC correction suggestions, route
planning, and batch scripts should produce proposals by default unless an
explicit trust/commit policy allows direct transaction creation.

Manual GUI edits may commit directly when the action is local, visible,
undoable, and does not imply hidden cross-domain changes. Manual edits that
imply schematic/PCB authority changes, standards deviations, destructive batch
changes, or import repair should become proposals.

No GUI, CLI, MCP, assistant, terminal, script, or importer may mutate design
state through a private path that bypasses this model.

## Core Primitives

### DesignModel

The `DesignModel` is the canonical authority that operations mutate.

It contains electrical intent, physical implementation, library bindings,
rules/constraints, standards/process basis, analysis records, manufacturing
state, artifacts, proposals, transactions, and provenance.

Operations may affect one domain object or several related domain objects, but
they always commit against the design model.

### Operation

An operation is the smallest typed edit Datum understands semantically.

Examples:
- `PlaceSymbol`
- `MoveSymbol`
- `DrawWire`
- `DeleteWire`
- `PlaceComponent`
- `MoveComponent`
- `RouteTrack`
- `PlaceVia`
- `SetRule`
- `AssignPart`
- `SetPadProcessAperture`
- `ApplyEcoChange`
- `AcceptLayoutDeviation`

Required properties:
- typed
- validated before commit
- targets stable object identities where applicable
- identifies the projection/editor context that produced it when relevant
- deterministic enough for diff/check/audit use
- does not silently perform unrelated edits

Compound user actions emit operation batches.

### OperationBatch

An operation batch is an ordered set of operations that should be reviewed or
committed together.

Examples:
- drag component with attached routing adjustments
- apply ECO changes for several components
- accept a DRC pad-process correction across selected pads
- route a net with multiple track and via operations

### Proposal

A proposal is an uncommitted candidate operation batch.

Expected sources:
- AI agent
- assistant
- DRC/ERC correction suggestion
- standards/process-audit repair
- ECO comparator
- route planner
- import repair workflow
- CLI preview mode
- script review mode

Required fields:
- stable proposal ID
- operation batch
- rationale
- affected objects
- expected result
- diff/preview
- checks run
- unresolved assumptions
- risks
- provenance
- apply/reject/defer status

### Transaction

A transaction is a committed operation batch, recorded as one append-only
`TransactionRecord` in the project journal.

Expected sources:
- manual GUI direct edit
- accepted proposal
- CLI command in commit mode
- MCP operation with approval
- script with configured trust policy
- import/open conversion step

Required fields:
- stable transaction ID
- ordered operations
- changed objects
- before/after diff or equivalent undo data
- provenance (`actor`, commit `source`, and reason); proposal-mediated
  acceptance is proven by the applied `Proposal` record whose
  `applied_transaction_id` matches this transaction ID
- timestamp
- check state when relevant
- undo/redo data
- parent-txn DAG field (single parent in the first cut; reserved for the
  multi-writer case, which is not built)

Every transaction — regardless of source surface — is produced by exactly one
engine commit primitive. `OperationBatch -> commit()` applies the batch in
memory, then stages shard bytes and fsyncs them, then appends the
`TransactionRecord` to the journal and fsyncs it (this append is THE COMMIT
POINT), then performs an atomic rename plus directory fsync. The journal is the
persisted, replayable, auditable history of the project; transactions are its
sole producer. The first-cut precondition for this commit path is
single-writer-per-project: one writer holds the project, and the parent-txn DAG
collapses to a linear chain.

Transactions are the sole producer of revisions. A commit advances
`object_revision` (a monotonic `u64` per touched object) and recomputes
`model_revision` (a sha256 over the canonically sorted set of object revisions
plus the accepted-transaction tip). Downstream derived state — connectivity,
zone fill, manufacturing projections, variant views — keys its staleness on
these revisions; transactions are the only thing that moves them.

### Diff

A diff is the machine-readable change description attached to a proposal or
transaction.

Minimum diff categories:
- object created
- object deleted
- property changed
- geometry changed
- rule/check state changed
- artifact generated or invalidated

### Provenance

Provenance records where a change came from and how it was accepted.

Minimum provenance fields:
- actor type: user, AI, assistant, CLI, MCP, script, importer, checker, router
- actor identity when available
- source command/tool/import/check/proposal
- timestamp
- direct/proposal classification, either from `CommitProvenance.source` or from
  the applied proposal's transaction link
- source file or external reference where applicable

## Direct Commit Versus Proposal

Direct commit is allowed when:
- the user performs a local manual edit
- the action is immediately visible
- the action is undoable
- the action does not imply hidden electrical/physical relationship changes
- the action does not silently repair imported data
- the action does not waive or deviate from standards/check results

Proposal required when:
- an AI or assistant suggests a design mutation
- a DRC/ERC/check finding suggests a correction
- an import repair would change source-derived geometry
- an ECO or relationship-state change affects electrical/physical authority
- a batch command affects many objects
- a standards deviation or waiver is created
- the change is destructive or difficult to inspect

## Manual Workflow Requirements

Manual tools must feel direct. The canonical edit model must not turn every
click into a modal approval flow.

Expected behavior:
- simple placement, movement, routing, property edits, and deletion can commit
  immediately as transactions
- undo/redo works consistently and durably: undo and redo are not volatile
  in-memory stacks but positions (cursors) in the durable transaction journal,
  so history survives across sessions; an undo is itself recorded as a
  compensating transaction so the journal remains append-only and auditable
- transaction history is inspectable
- property/inspector edits use the same transaction model as canvas tools
- cross-domain or high-risk actions become proposals

## Optional AI And Tooling Behavior

AI is not a hidden authority.

Agents and assistants may:
- inspect project state
- query selected objects
- run checks
- create proposals
- explain proposals
- apply proposals only through explicit approval or configured policy

Agents and assistants may not:
- mutate design files behind Datum's back
- bypass transactions
- create untracked geometry changes
- silently repair imports
- silently waive standards/check findings

## Implementation Surfaces Affected

This decision affects:
- engine operation APIs
- CLI project mutation commands
- GUI canvas tools
- design model persistence
- projection and editor-context surfaces
- inspector/property editing
- undo/redo stack
- native project persistence
- importers
- DRC/ERC/checking
- ECO/annotation
- MCP tools
- assistant bridge
- embedded terminal environment
- scripts and automation

## Storage And Revision Reconciliation

Transactions must align with the storage/versioning model.

Required transaction revision behavior:
- every committed transaction advances the resolved `model_revision`
- affected source objects receive `object_revision` updates where applicable
- generated artifacts are invalidated or linked to the model/output-job
  revisions that produced them
- library, variant, manufacturing-plan, and output-job edits record their own
  revision context
- transaction provenance records the storage shards touched by the operation
  batch when practical

### The single commit path is the enforcement mechanism

Operations, proposals, and transactions converge on one commit primitive. A
manual edit that qualifies for direct commit and a proposal accepted by a user
both call the identical engine commit path; direct commits are classified by
`CommitProvenance.source`, while proposal-mediated commits are classified by the
durable proposal-to-transaction join
(`Proposal.applied_transaction_id == TransactionRecord.transaction_id`) plus the
proposal's own `Proposal.source`. This convergence is what makes the rule that
no surface may mutate state through a private path enforceable rather than
aspirational: there is exactly one place where shard bytes change and exactly
one place where the journal grows, so an audit reduces to "did this byte change
arrive through `commit()`?" rather than an open-ended search for side doors.

Storage shards are persistence partitions. They do not get private mutation
paths. A direct file edit, importer repair, CLI helper, or GUI utility that
changes a shard without producing a transaction is outside the canonical model
and is a bug or a time-boxed migration exception, never a sanctioned shortcut.

### Retired bypasses stay retired

The former board-text editing helpers in the GUI substrate
(`crates/gui-protocol/src/board_text_mutations.rs`) were a concrete violation
of this rule: they read the board document via `board_path`, mutated parsed JSON
in place, and wrote it back with `write_json_file` with no operation, no
transaction, and no undo. That helper module has been removed. Selected
board-text GUI edits now prefill the project PTY with journaled
`datum-eda project edit-board-text "$DATUM_PROJECT_ROOT" --text <uuid> ...`
commands, and the drift gate fails if `gui-app` reintroduces those retired
private writer helper names.

This is the governance pattern for every similar exception: either migrate it
to a typed operation and commit it, or delete the shortcut. A retired bypass
must not remain as a dormant public helper because that preserves a side door
around durable undo, machine-readable diffs, provenance, and the shared
atomic/journaled core used by CLI, MCP, GUI, importers, and agents.

### The format-duality landmine (an ordering decision, not a free fold-in)

Folding this GUI path in is not a drop-in replacement, because the two ends
speak different formats. The GUI helper writes native board JSON, while
`Engine::save` patches KiCad s-expression text through the sidecar path. The
real split-truth bug lives at the KiCad save end, where authored intent and
emitted text can diverge. Converging the GUI bypass therefore requires picking
one of two orderings deliberately:

1. target the KiCad save path first — where the split-truth defect actually
   lives — so the converged commit path emits the authoritative format; or
2. first stand up a native-board `DesignModel` shard, so the GUI helper's
   typed operation commits against a native authority that the resolver owns.

Either is defensible; doing neither and naively routing native-JSON writes
through `commit()` would paper over the duality. This is an explicit ordering
decision for the owner (see Open Owner Questions), not a free fold-in. It also
sits behind the M7 sequencing: the identity, resolver, and commit modules are
built concurrently with M7, but the authority flip and the from-shards KiCad
emitter are deferred until M7 closes and the emitter passes a
byte-identical-when-unmodified fidelity gate on the datum-test fixture, run as
dual-write during a deprecation window.

## Relationship And Multi-Board Reconciliation

Operations must target explicit electrical, physical, and relationship
objects, addressed by stable surrogate identity (`ObjectId = Uuid`,
persist-on-create), never by name, reference designator, or position.

The cross-domain join between a schematic symbol and the board package that
realizes it is a `ComponentInstance`: a per-instance stable surrogate (`Uuid`)
that both the placed symbol and the placed package reference, distinct from the
library `part` field. It is created and linked at import and at forward
annotation, and it replaces the reference-designator string that informally
serves as the electrical<->physical join today.

The authored binding between domains is a `Relationship` (its own surrogate id)
carrying a `RelationshipKind` of `ImplementedBy`, `BoardOnly`, `SchematicOnly`,
`ReverseEngineered`, `Pending`, or `Mismatch`. A relationship's status is split
into a derived axis and an authored-intent axis: the derived status is one of
`Implemented`, `PendingImplementation`, or `UnresolvedMismatch` (recomputed by
the resolver, never authored); the authored intent is one of `LayoutDeviation`,
`BoardOnlyObject`, `SchematicOnlyObject`, `ReverseEngineered`, or an
accepted-deviation. Net identity is a stable `NetId` (lowest-`NetId`-wins on
merge) bound to derived connectivity groups through a `NetAnchor` authored hint.

Examples:
- `DrawWire` mutates electrical intent within an electrical scope.
- `RouteTrack` mutates physical implementation within a board or physical
  scope.
- `BindPhysicalToElectrical` records, through a `ComponentInstance`, how a board
  object implements electrical intent.
- `SetRelationshipStatus` records authored intent such as `LayoutDeviation`,
  `BoardOnlyObject`, `SchematicOnlyObject`, or `ReverseEngineered`. It does not
  write the derived statuses (`Implemented`, `PendingImplementation`,
  `UnresolvedMismatch`), which the resolver computes, nor a population's
  `NotApplicableForVariant`, which is a derived variant-resolution outcome and
  is never stored (see the variant model).
- `CreateBoardFromElectricalScope` creates physical implementation state
  without making the board a separate authority.

Multi-sheet and multi-board work must not bypass the edit model. A sheet edit,
board edit, module edit, variant edit, or board-first reconstruction step is a
normal operation or proposal against the resolved project model.

## Manufacturing And Artifact Reconciliation

Manufacturing projections and exported artifacts must also use the canonical
edit model.

Examples:
- editing an output job is a transaction
- changing panel rails, tabs, mouse bites, V-scores, fiducials, tooling holes,
  coupons, labels, or panel notes is a manufacturing-domain transaction
- generating Gerber, NC drill, soldermask, paste, BOM, PnP, assembly, or panel
  artifacts records artifact metadata and provenance
- exported files are snapshots of model, manufacturing-plan, variant, and
  output-job revisions

Panelization operations must not silently mutate source board geometry.
If a user wants to convert a manufacturing/panel change into a board design
change, that conversion must be explicit and reviewable.

## Standards And Compliance Impact

Standards-aware checks and corrections must use this model.

Example:
- DRC finds `pad_mask_expansion_missing`
- checker creates or references a correction proposal
- user reviews the affected pads and process basis
- accepted proposal commits `SetPadProcessAperture` operations
- transaction records provenance that this was a standards/process correction
  from imported or authored geometry

Datum must not silently mutate source geometry toward an inferred IPC result.

Standards-aware checks may create proposals. They may not directly repair
design state unless the user has explicitly accepted the proposal or configured
a narrowly scoped automation policy.

Examples of proposal-producing checks:
- pad/mask/paste aperture correction
- footprint courtyard or assembly clearance correction
- library metadata completion
- imported unknown-basis remediation
- waiver or deviation creation

## Explicit Non-Goals

This decision does not define:
- the complete list of all operations
- the final on-disk transaction history format
- the full GUI transaction-history UI
- the full agent permission model
- a complete ECO implementation
- a complete standards sign-off workflow

Those decisions depend on this model but should be documented separately.

## First Proof Slice

The first proof slice should demonstrate, through the single `commit()` path:
- one electrical projection edit represented as an operation and committed as a
  transaction (advancing `object_revision` and `model_revision`)
- one physical projection edit represented as an operation and committed as a
  transaction
- one relationship represented in the design model, including a derived
  `UnresolvedMismatch` distinct from an authored deviation
- one proposal created from a non-manual source and accepted into a transaction,
  with `Proposal.applied_transaction_id` linking it to the committed
  transaction and `Proposal.source` distinguishing it from a direct edit
- consistent durable undo/redo as journal cursors, with undo recorded as a
  compensating transaction
- machine-readable diff
- provenance visible in test output or debug query

Suggested minimal objects:
- electrical projection: place or move symbol, draw wire
- physical projection: draw track or move component
- proposal: DRC pad-process correction or ECO placement proposal

Seeding (Slice 0): the first proof seeds a board-first, all-`BoardOnly`
reverse-engineered model. Now that `ComponentInstance` is approved, the
immediate follow-on is the `ImplementedBy` seed, which exercises the
electrical<->physical join end to end.

This slice maps onto the consolidated proof plan whose full set lives in
`PRODUCT_MECHANICS_000D_STORAGE_AND_VERSIONING_MODEL.md`. The gates this
document is most directly accountable to are PG-IDENTITY-SUBSTRATE (1),
PG-COMMIT-ATOMIC+DURABLE-UNDO (3), and PG-PROPOSAL-PARITY (5); all gates are
wired into `run_drift_gates.sh` under PG-HARNESS-WIRING (10).

## Resolved Mechanism (was open; ratified 2026-06-18)

These prior open questions are now answered by ratified mechanism and are no
longer forks:

- Transaction-history persistence (was Q5). Resolved: the journal is required
  source state from the first cut. History is durable, not session-only;
  undo/redo are cursors into the journal and undo is a compensating transaction.
- Identity wire type and import representation (was Q3, in part). Resolved:
  `ObjectId = Uuid`, persist-on-create. Native objects keep a v4 generated once
  and persisted; imported objects keep v5 demoted to an import SEED, persisted
  on first import into a net-new Import Map shard keyed by `import_key` (not
  `source_hash`). An import is therefore a source-load event that seeds the
  Import Map shard, with native conversion expressed as ordinary operations
  against the resolved model — not an opaque blob.
- Object/model revision persistence (part of Q7). Resolved: `object_revision`
  (monotonic `u64` per object) and `model_revision` (sha256 over canonically
  sorted object revisions + accepted-txn tip) persist from the first slice;
  transactions are their sole producer.

## Open Owner Questions

1. Are CLI commands allowed to commit directly by default, or should preview be
   the default for mutating commands?
2. Which manual GUI edits require proposal review instead of direct commit?
3. Format-duality ordering for retiring the GUI board-text bypass: target the
   KiCad save path first (where the split-truth bug lives), or stand up a
   native-board `DesignModel` shard first? This is the ordering decision called
   out in Storage And Revision Reconciliation.
4. What minimum diff format is acceptable for geometry changes?
5. Allocated integer `ObjectId(u64)` as a diff-size optimization is deferred:
   should the eventual allocator be global or project-local? (Deferred, not
   blocking the first cut.)
6. What is the first operation family to move behind the canonical model:
   electrical projection, physical projection, rules/checks, or library?
7. Beyond model/object/transaction revisions (now ratified to persist), which
   remaining revision classes must persist in the first slice: artifact,
   library, variant, or output-job?
8. Which authored relationship intents should require proposals rather than
   direct manual transactions?
9. Should generated artifact metadata be transaction history, artifact state,
   or both?
10. What minimum transaction data is needed for useful git review without
    turning every edit into noisy storage churn?
