# Product Mechanics Decision 004: AI Tooling Contract

Status: draft hypothesis + implementation mechanism woven 2026-06-19.
Date: 2026-06-19

Driven by:
- `docs/DATUM_PRODUCT_MECHANICS.md`
- `docs/audits/scope-integration/DATUM_PRODUCT_MECHANICS_REVIEW_AGENDA.md`
- `docs/decisions/PRODUCT_MECHANICS_000_UNIFIED_DESIGN_MODEL.md`
- `docs/decisions/PRODUCT_MECHANICS_000D_STORAGE_AND_VERSIONING_MODEL.md`
- `docs/decisions/PRODUCT_MECHANICS_000F_LIVE_PRODUCTION_PROOF_MODEL.md`
- `docs/decisions/PRODUCT_MECHANICS_001_CANONICAL_EDIT_MODEL.md`

## Decision Scope

Define the product contract for Datum's AI-facing tooling.

This decision covers structured context, command surfaces, proposal mechanics,
safety boundaries, and provenance rules that make AI collaboration native to
Datum without creating hidden editing powers or a second model authority.

## Product Intent

AI in Datum is optional.

Datum must be fully usable without AI. Every core EDA workflow must remain
available through manual tools, CLI commands, scripts, project files, and
deterministic checks.

AI-native means Datum exposes EDA-native capabilities to optional agents:
stable object IDs, structured project context, schematic and PCB queries,
typed proposals, diffs, checks, artifact generation, and provenance. It does
not mean an agent receives private access to source shards or a hidden model of
the design.

The primary integration model is:
- real terminal sessions that can launch `codex`, `claude`, `aider`, shell
  scripts, Datum CLI commands, and other user-selected tools
- Datum CLI/MCP/API tools that expose the same canonical `DesignModel`
  primitives available to manual workflows
- proposal and transaction mechanics that keep user approval, diffing, and
  auditability explicit

## Decision

Datum shall expose one implementation contract to CLI commands, MCP tools,
terminal-launched agents, optional assistant surfaces, scripts, importers,
checkers, and routers.

That contract has three hard boundaries:
- Reads go through resolved model/query/projection services owned by the
  engine, never direct interpretation of source shards as authority.
- Candidate mutations are typed `OperationBatch` values attached to
  `Proposal` records unless an explicit trust policy allows direct commit.
- All accepted mutations flow through `commit()`, append exactly one
  `TransactionRecord` to the journal, and receive provenance identifying the
  actor, tool, session, acceptance path, model revision, and affected objects.

No CLI command, MCP method, assistant action, terminal process, importer,
checker, router, or script may mutate source shards, projections, caches, or
artifacts as a private design-state path. Shards are persistence partitions;
the resolved in-memory `DesignModel` is the authority; the journal is the
auditable commit record.

## User-Visible Behavior

Users should be able to open a Datum project, launch a terminal or configured
agent, and give that tool enough structured project context to inspect,
analyze, and propose changes without screen-scraping the GUI.

Visible behavior should include:
- project-aware discovery of project root, project ID, model revision, active
  projection, selected objects, open checks, and recent transactions
- stable object queries for schematic, PCB, library, rules, checks,
  manufacturing projections, artifacts, relationship states, and provenance
- proposals that show rationale, affected objects, diff/preview, checks run,
  unresolved assumptions, risks, and apply/reject/defer actions
- deterministic checks that can be run before and after a proposal
- generated artifacts with model revision, command/tool provenance, and
  validation state
- clear labels for whether work originated from a user, CLI command, MCP tool,
  script, importer, checker, router, assistant, or external AI agent

AI work must not appear as unexplained geometry or file edits. If a tool
changes design state, the user must be able to inspect what changed, why, from
what source, against which revision, and under which approval path.

## Manual Workflow Requirements

AI tooling must not define the minimum usable product.

Manual workflows must be possible without starting any model or assistant:
- create and open projects
- edit electrical intent in schematic projections
- edit physical implementation in PCB projections
- edit library symbols, footprints, part bindings, and process metadata
- edit rules and constraints
- run ERC/DRC and inspect findings
- navigate objects and relationship states
- generate manufacturing outputs
- review diffs, proposals, transactions, artifacts, and provenance

Any AI-visible operation must have a corresponding deterministic tool or data
contract that can be exercised without AI. If an agent can perform a core edit
but a user cannot perform the same workflow manually or through supported
non-AI tooling, the workflow is incomplete.

## Contract Classes

These classes are product contracts, not final Rust module names. They define
the implementation shape that CLI, MCP, assistant, and terminal-launched tools
must share.

### DatumToolSession

`DatumToolSession` is the authenticated, project-scoped capability envelope
for a tool invocation.

Required fields:
- `session_id`: stable ID for the terminal tab, assistant session, MCP client,
  CLI process, script run, importer, checker, or router
- `actor_type`: `User`, `Cli`, `Mcp`, `Script`, `Importer`, `Checker`,
  `Router`, `Assistant`, or `ExternalAgent`
- `actor_identity`: user/tool identity when available
- `project_id` and `project_root`
- `model_revision` read at session/context creation
- `projection_id`, `selection`, and `cursor_context` when launched from a UI
  context
- `capabilities`: read, check, artifact, propose, apply-approved, or
  direct-commit-by-policy
- `provenance_seed`: command, executable, MCP method, assistant action, or
  importer/checker source

A session grants capabilities to Datum tools only. It does not grant write
permission to project shards, derived caches, or journal files.

### DatumContextEnvelope

`DatumContextEnvelope` is the stable context Datum injects into terminal,
assistant, CLI, and MCP sessions.

Required fields:
- project root, project ID, storage layout version, and schema version
- current `model_revision` and accepted transaction tip
- active projection and editor context IDs
- selected object IDs and relationship IDs
- visible check finding IDs and artifact IDs
- local discovery endpoint or CLI discovery command
- context expiry and refresh command

Context is a snapshot. Tools must call `datum-eda context refresh` or the MCP
equivalent before proposing or applying changes if their envelope revision is
stale.

### DatumQueryTool

`DatumQueryTool` is read-only structured access to resolved `DesignModel`
state and derived projections.

Query families:
- `project.get_context`
- `object.get`
- `object.neighbors`
- `selection.describe`
- `schematic.query`
- `pcb.query`
- `library.query`
- `rules.query`
- `relationships.query`
- `checks.query`
- `manufacturing.query_projection`
- `artifacts.query`
- `transactions.query`
- `provenance.query`

Queries return stable IDs, object revisions, projection provenance, and source
relationship metadata where relevant. They do not expose shard bytes as
authority and they never mutate state.

Cross-domain relations returned by `object.neighbors`, `relationships.query`,
`schematic.query`, and `pcb.query` are keyed on `ComponentInstance` (and stable
`ObjectId`), never on reference designator, name, path, or sheet/board ID. The
AI contract surfaces `ComponentInstance` explicitly as the canonical
electrical-to-physical join binding `PlacedSymbol` and `PlacedPackage`, so an
agent must never reconstruct that join by string matching. Imported object
identity and provenance resolve through the Import Map keyed by `import_key`
(not `source_hash`); `provenance.query` exposes that `import_key`-keyed source
provenance together with unknown-basis markers.

### DatumCheckTool

`DatumCheckTool` runs deterministic ERC, DRC, standards, process,
manufacturing, or project validation against a known model/projection revision.

Required inputs:
- project ID
- check family and scope
- model revision or projection revision
- selected object IDs or output job IDs when scoped

Required outputs:
- check run ID
- findings with stable IDs and affected object IDs
- checked revision and generator/checker version
- pass/fail/error/stale state

Check results may be persisted as analysis/check records through `commit()`
when they become project state. Persisted check records remain revision/hash-
keyed derived state, keyed to the checked model/projection revision and
checker version, and are invalidated by `model_revision` movement; they are
never an alternate authority. Ephemeral check previews must be explicitly
marked as session state.

### DatumArtifactTool

`DatumArtifactTool` generates or compares artifacts from live projections.

Artifact families include Gerber, drill, soldermask, paste, BOM, PnP,
assembly, panel, reports, and analysis outputs.

Generated artifacts must record:
- project ID and model revision
- output job, variant, manufacturing plan, and projection revision
- generator version and command/tool provenance
- validation state and equivalence result when compared to a previous export

Artifacts are revisioned snapshots of projections. A generated artifact is not
source authority unless a user explicitly imports it as production evidence or
reverse-engineering input through a proposal/transaction workflow.

### DatumProposalTool

`DatumProposalTool` creates, updates, validates, rejects, defers, and applies
proposals.

Required proposal creation inputs:
- `OperationBatch` with typed operations and stable object targets
- rationale
- affected object IDs
- expected result
- model revision the proposal was prepared against
- assumptions, risks, and checks requested or already run
- `DatumToolSession` provenance

Required proposal outputs:
- stable proposal ID
- machine-readable diff/preview
- validation result
- required acceptance policy
- stale/conflict status relative to the current model revision

Applying a proposal does not write files directly. It calls the same
`commit()` primitive used by manual edits and produces one journaled
`TransactionRecord`.

### DatumCommitTool

`DatumCommitTool` is the only mutation gateway exposed to tools. It accepts:
- an approved `Proposal`
- an `OperationBatch` from a manual GUI edit
- an `OperationBatch` from a CLI/MCP/script only when policy allows
  direct-commit-by-policy

Commit behavior is inherited from Decision 001:
`OperationBatch -> commit()` applies in memory, stages shard bytes and fsyncs,
appends a `TransactionRecord` to the journal and fsyncs it as the commit
point, then performs atomic rename plus directory fsync.

`DatumCommitTool` must reject:
- stale base revisions unless the operation batch is explicitly rebased and
  revalidated
- operations targeting unknown or mismatched object revisions
- writes to derived projection caches as if they were source state
- attempts to bypass proposal policy for AI/assistant/checker/import repair
  work
- missing provenance or acceptance path

### DatumJournalTool

`DatumJournalTool` exposes transaction history, proposal lineage, durable
undo/redo cursors, and provenance.

It is read-only except for undo/redo requests, which are represented as
compensating operation batches and committed through `DatumCommitTool`.

## CLI And MCP Shape

Datum CLI and MCP must be isomorphic over the same contract classes.

Required CLI groups:
- `datum-eda context get|refresh`
- `datum-eda query ...`
- `datum-eda check run|show`
- `datum-eda artifact generate|compare|show`
- `datum-eda proposal create|show|validate|reject|defer|apply`
- `datum-eda journal list|show|undo|redo`

CLI naming note: `datum-eda` is the canonical CLI executable. `eda` and bare
`datum` command examples are legacy/noncanonical; `datum.*` remains reserved
for MCP tool family names only.

Required MCP tool families:
- `datum.context.get`
- `datum.query.*`
- `datum.check.*`
- `datum.artifact.*`
- `datum.proposal.*`
- `datum.journal.*`

CLI output must have a stable machine-readable JSON mode. MCP outputs must be
the same schemas, not assistant-specific summaries. Human-readable summaries
are presentation layers over these schemas.

## Optional AI And Tooling Behavior

Agents and assistants may:
- inspect project metadata, active context, selections, and object graphs
- run structured schematic, PCB, library, rule, and manufacturing queries
- run ERC/DRC/check commands and summarize findings
- generate analyses, reports, and artifacts
- create bounded proposals backed by typed operations
- explain proposal rationale, risks, and assumptions
- request approval to apply proposals
- apply proposals only through `DatumProposalTool.apply` and
  `DatumCommitTool` or an explicitly configured safe automation policy, where
  every accepted AI mutation reduces to a typed `OperationBatch` and calls the
  single `commit()` primitive, appending exactly one journaled
  `TransactionRecord` with provenance identifying actor, tool, session,
  acceptance path, and model revision

Agents and assistants must not:
- mutate design state through private file writes or hidden GUI automation
- bypass operation, proposal, diff, check, transaction, and provenance records
- silently repair imports or inferred standards issues
- silently waive ERC/DRC/process findings
- redefine electrical intent from physical edits without an explicit proposal
  or relationship-state transition
- become required for any core EDA workflow

The initial AI tool contract should favor small, explicit, inspectable tools
over broad "do anything" commands.

## Standards And Compliance Impact

AI tools may help detect, explain, or propose fixes for standards and process
issues, but they must not become silent compliance authorities.

Standards-sensitive behavior must:
- identify the relevant standard, process assumption, rule, or check basis
- preserve imported source metadata through the Import Map keyed by
  `import_key` (not `source_hash`), retaining unknown-basis markers
- present corrections as proposals unless a user has explicitly configured a
  safe automation policy
- record provenance when a standards-related edit is accepted
- distinguish authored design intent from inferred repair
- keep generated compliance artifacts tied to model revision and check state

Examples include IPC footprint/process corrections, soldermask or paste
expansion changes, clearance/creepage findings, drill constraints, fab notes,
BOM/PnP validation, and manufacturing output checks.

## Private Mutation Ban

This decision explicitly forbids private mutation paths.

Forbidden paths include:
- writing source shards from a CLI command, MCP tool, assistant, agent, or
  script without `commit()`
- writing the transaction journal directly
- treating generated Gerber, drill, BOM, PnP, assembly, or report files as
  source authority
- editing derived caches as a substitute for changing source objects
- driving hidden GUI actions to avoid proposals or provenance
- applying check/import/standards repairs without a proposal or configured
  direct-commit policy

Datum may expose file export, artifact generation, and report generation
commands, but those commands must be classified as artifact/output operations
and tied to source model revisions.

## Explicit Non-Goals

This decision does not define:
- a required built-in model provider
- a required chat UI
- prompt text or assistant personality
- complete permission policy for every automation level
- remote execution, cloud syncing, or hosted agent infrastructure
- replacement of manual schematic, PCB, library, rules, or manufacturing tools

## First Proof Slice

The first proof slice should demonstrate an optional external or built-in tool
using Datum-native context without private edit access:
- discover active project root, project ID, model revision, active projection,
  selected objects, and Datum discovery endpoint
- query selected objects with stable IDs, object revisions, and
  domain-specific properties
- run one deterministic check and return machine-readable findings
- create one proposal with a typed `OperationBatch`, rationale, affected
  object list, diff/preview, assumptions, checks, and provenance
- reject stale proposals when the model revision has moved
- accept or reject the proposal through `datum-eda proposal apply` or the MCP
  equivalent
- show the resulting journaled transaction and artifact/check invalidation
  state

Suggested first scenario:
- select a PCB object with an associated check finding
- have a tool propose a bounded physical correction
- review the diff and check result
- apply the proposal into a transaction
- verify provenance identifies the tool, session, model revision, and approval
  path

## Open Owner Questions

1. What is the smallest initial CLI/MCP query family that proves AI-native EDA
   context without overbuilding the agent API?
2. Should the first slice expose a local session socket, CLI discovery only,
   or both?
3. Which object categories need stable IDs in the first proof slice beyond
   selected PCB objects, findings, proposals, and transactions?
4. What automation policies, if any, are allowed to apply proposals without
   per-proposal approval?
5. Should AI-created analyses and reports become first-class artifacts from
   the first slice?
6. How much assistant/agent activity should be persisted in project history
   versus local session history when it does not produce proposals,
   transactions, artifacts, or checks?
