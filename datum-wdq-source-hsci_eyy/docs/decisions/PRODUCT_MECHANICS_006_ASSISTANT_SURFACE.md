# Product Mechanics Decision 006: Assistant Surface

Status: draft hypothesis + implementation mechanism woven 2026-06-19.
Date: 2026-06-19

Driven by:
- `docs/DATUM_PRODUCT_MECHANICS.md`
- `docs/audits/scope-integration/DATUM_PRODUCT_MECHANICS_REVIEW_AGENDA.md`
- `docs/decisions/PRODUCT_MECHANICS_000_UNIFIED_DESIGN_MODEL.md`
- `docs/decisions/PRODUCT_MECHANICS_000D_STORAGE_AND_VERSIONING_MODEL.md`
- `docs/decisions/PRODUCT_MECHANICS_000F_LIVE_PRODUCTION_PROOF_MODEL.md`
- `docs/decisions/PRODUCT_MECHANICS_001_CANONICAL_EDIT_MODEL.md`
- `docs/decisions/PRODUCT_MECHANICS_004_AI_TOOLING_CONTRACT.md`
- `docs/decisions/PRODUCT_MECHANICS_005_EMBEDDED_TERMINAL.md`

## Decision Scope

Define whether Datum should have a built-in assistant surface and, if so, what
role it plays relative to manual tools, the real embedded terminal, external
agents, and the canonical proposal/transaction model.

## Product Intent

The assistant surface is optional and secondary.

Datum must be fully usable without AI and without a built-in assistant panel.
The primary product architecture is manual EDA tooling plus deterministic
Datum primitives: structured context, stable object IDs, CLI/MCP/API tools,
proposals, diffs, checks, artifacts, transactions, and provenance.

The assistant panel may help users understand and operate Datum, but it must
not become the primary AI architecture, a fake terminal, or the only route to
core functionality. If the assistant cannot be clearly bounded, it should be
deferred until the terminal and proposal/transaction model are product-real.

## Decision

If Datum includes an assistant surface, it shall be a bounded
read/proposal/apply UI over the same tool contract used by CLI, MCP,
terminal-launched agents, scripts, importers, checkers, routers, and manual
proposal review.

The assistant has no private editing API. It may read through
`DatumQueryTool`, run checks through `DatumCheckTool`, generate or compare
artifacts through `DatumArtifactTool`, draft proposals through
`DatumProposalTool`, and apply approved proposals through `DatumCommitTool`.
It must not write source shards, projection caches, artifacts-as-source, or the
transaction journal directly.

Assistant-originated changes become design state only when an approved
`Proposal` or allowed policy-scoped `OperationBatch` calls `commit()` and
produces a journaled `TransactionRecord` with assistant provenance. For
proposal-mediated assistant changes, acceptance is durable through the applied
`Proposal` whose `source` is assistant-originated and whose
`applied_transaction_id` matches the committed transaction.

## User-Visible Behavior

If present, the assistant should be visibly distinct from the terminal:
- Terminal: real PTY shell sessions where users run normal commands and launch
  whatever tools they choose.
- Assistant: optional Datum-aware conversation, explanation, review, and
  proposal surface.

The assistant may show current project context, selected objects, active
projection, recent transactions, open checks, generated artifacts, and relevant
Datum commands. It may answer questions, explain findings, suggest next steps,
or draft proposals.

The assistant must clearly label when it is:
- explaining current state
- running a read-only query
- creating an uncommitted proposal
- validating a proposal
- requesting approval to apply a proposal
- applying an approved proposal through the canonical commit path
- summarizing results from checks or artifacts
- handing the user off to the terminal, CLI, or manual editor

It must not present AI-generated changes as already applied unless those
changes became committed transactions through the canonical model.

## Assistant Session Model

### AssistantSession

`AssistantSession` is optional conversation and review state scoped to a
project or workspace session.

Required fields:
- `assistant_session_id`
- `project_id` and project root when project-scoped
- active `DatumContextEnvelope` snapshot
- actor identity and model/provider metadata where applicable
- visible conversation state
- referenced object IDs, findings, artifacts, proposals, and transactions
- capability set: read, check, artifact, propose, or apply-approved
- persistence policy: ephemeral, local session, or project-linked metadata

Assistant conversation text is not design authority. Project history records
assistant-created proposals, accepted transactions, artifacts, and checks, not
arbitrary chat content unless the user explicitly saves it as an artifact or
note.

### AssistantContext

`AssistantContext` is a UI-selected view of the shared
`DatumContextEnvelope`.

The assistant reads only the engine-resolved `DesignModel` assembled by
`ProjectResolver` at a known `model_revision`. `AssistantContext` and the
shared `DatumContextEnvelope` are views over that resolved model, never over
source shards or on-disk files directly.

It may include:
- active project and model revision
- active projection and editor context
- selected object IDs and relationship IDs
- visible checks and findings
- relevant artifacts and output jobs
- recent transactions and proposal lineage
- user-visible assumptions and unresolved questions

The assistant must refresh context before validating or applying proposals.
If the current model revision differs from the proposal base revision, the
assistant must mark the proposal stale and revalidate or ask the user to
discard it.

## Bounded Tool Use

The assistant may only use these product capabilities:
- `Read`: use `DatumQueryTool` to inspect the `ProjectResolver`-assembled
  `DesignModel` and its projection state at a known `model_revision`.
- `Check`: use `DatumCheckTool` to run deterministic checks or retrieve
  findings.
- `Artifact`: use `DatumArtifactTool` to generate, compare, or explain
  revisioned artifacts.
- `Propose`: use `DatumProposalTool.create|validate|reject|defer` to create
  and manage uncommitted operation batches.
- `Apply`: use `DatumProposalTool.apply`, which delegates to
  `DatumCommitTool` and `commit()`.

The assistant may not:
- directly call low-level shard writers
- write journal entries
- mutate derived caches as source state
- drive hidden GUI tools to create edits without operations
- treat generated manufacturing outputs as source authority
- use terminal text parsing as a design mutation path

## Manual Workflow Requirements

No core EDA workflow may depend on the assistant.

Users must be able to manually:
- create, edit, and navigate schematic projections
- create, edit, and navigate PCB projections
- edit symbols, footprints, parts, rules, constraints, and manufacturing
  outputs
- run checks and inspect findings
- create and review proposals from non-AI tools
- apply or reject proposals
- inspect diffs, transaction history, artifacts, and provenance
- use the terminal and CLI without the assistant

The assistant may reduce friction, explain concepts, and compose commands, but
it cannot be the product mechanism that makes an incomplete manual workflow
appear complete.

## Optional AI And Tooling Behavior

The assistant may:
- read structured project context through the same query tools available to
  CLI/MCP/external agents
- explain selected objects, nets, rules, checks, artifacts, or transactions,
  surfacing electrical-to-physical context through `ComponentInstance` as the
  canonical symbol-to-package join key rather than reference designators or
  names
- summarize ERC/DRC findings and suggest review order
- draft bounded proposals backed by typed operations
- request user approval before applying proposals

Assistant-drafted relationship proposals may only alter authored
`RelationshipKind` and authored intent (`LayoutDeviation`, `BoardOnlyObject`,
`SchematicOnlyObject`, `ReverseEngineered`, accepted-deviation). They may
never propose changes to resolver-derived status (`Implemented`,
`PendingImplementation`, `UnresolvedMismatch`) or derived
`NotApplicableForVariant`, which are recomputed on load and are not authored
source state.
- run deterministic checks through supported tooling
- suggest terminal commands for the user to run
- hand off to a terminal-launched agent when the user wants a full shell-based
  workflow

The assistant must:
- use the canonical proposal/transaction path for mutations
- make assumptions and unresolved questions visible
- preserve provenance for assistant-originated proposals and transactions
- respect the same permission and automation policies as external agents
- remain optional and dismissible
- show stale/conflict state when model revision changes under a proposal

The assistant must not:
- bypass the operation model
- mutate project files directly
- become a disguised terminal without PTY behavior
- hide diffs, check failures, or relationship-state changes
- silently waive standards or manufacturing findings
- replace the user's ability to run external agents in the real terminal

## Assistant UI Mechanics

The assistant UI should be structured around explicit cards rather than a
single opaque chat transcript:
- Context card: active project, projection, selection, model revision, and
  visible findings.
- Query result card: read-only output with source query and revision.
- Check card: deterministic check run, findings, checked revision, and stale
  state.
- Proposal card: rationale, affected objects, operation batch summary, diff,
  assumptions, risks, checks, stale/conflict state, and actions.
- Apply card: approval path, commit result, transaction ID, journal tip, and
  invalidated derived state/artifacts.
- Handoff card: terminal command, CLI command, or manual editor action when
  the task is better handled outside the assistant.

Apply controls must never be hidden in generated text. The user approves a
proposal through a product UI action or configured automation policy, and that
approval path is exposed as UI/API metadata while durable acceptance is proven
by the applied proposal's transaction link.

## Standards And Compliance Impact

The assistant may explain standards-sensitive findings, but it must not become
an unreviewed standards authority.

Assistant behavior that touches standards or manufacturing must:
- identify the check, rule, process basis, or standard being discussed
- preserve imported/source provenance where applicable
- create proposals for corrections, waivers, or process deviations
- show affected objects and diffs before approval
- run or recommend relevant checks after accepted changes
- record assistant provenance on proposals and accepted transactions

Assistant explanations should distinguish between deterministic Datum
findings, source data, owner-approved rules, and model-generated
interpretation.

## Private Mutation Ban

The assistant is explicitly forbidden from creating design-state changes by:
- writing project files
- editing source shards
- editing derived projection caches
- editing the transaction journal
- invoking hidden GUI commands that bypass operation generation
- applying generated artifacts as source authority
- auto-waiving checks or standards findings

All assistant-created mutations must be visible as proposals or as
policy-allowed direct operation batches and must end in `commit()` plus a
journaled transaction before they become design state.

## Explicit Non-Goals

This decision does not define:
- a required assistant launch in the first product slice
- a required model vendor or hosted model
- a complete conversation memory system
- a replacement for terminal-launched agents
- a replacement for CLI/MCP tooling
- a private assistant editing API
- complete prompt or UX copy
- a compliance sign-off authority

## First Proof Slice

The first proof slice should be intentionally narrow and bounded:
- open an optional assistant panel from a project
- show active project, projection, selected-object context, and model revision
- answer one read-only question using structured Datum queries
- summarize one check finding with affected object IDs
- create one uncommitted proposal with typed operations, rationale, diff,
  checks, assumptions, risks, stale/conflict state, and provenance
- require explicit user approval before applying the proposal
- apply the proposal through `DatumProposalTool.apply` and `DatumCommitTool`
- show the resulting transaction ID, journal tip, invalidated derived state,
  and assistant provenance

If the real PTY terminal and proposal/transaction model are not ready, the
assistant proof slice should stop at read-only explanation and proposal draft
creation rather than committing edits.

## Open Owner Questions

1. Should the assistant surface be deferred until the embedded terminal and
   proposal/transaction model are product-real?
2. Should the first assistant be read-only plus proposal drafting, or should it
   be allowed to apply approved proposals in the first slice?
3. What assistant contexts are safe and useful to expose first: selection,
   active projection, checks, transaction history, artifacts, or all of these?
4. Should assistant conversations be persisted in the project, in local
   session state, or not persisted initially?
5. How should the UI prevent users from confusing the assistant panel with a
   terminal session?
6. Which actions should always be handed off to the terminal or manual editor
   instead of being performed from the assistant panel?
