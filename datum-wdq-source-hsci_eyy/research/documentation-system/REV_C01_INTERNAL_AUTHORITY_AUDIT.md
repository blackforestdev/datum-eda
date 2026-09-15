# REV-C01 Internal Revision-Authority Audit

> **Status:** Committed factual baseline for owner review at REV-C01A.
> **Scope:** Datum repository state at `264b1a2` plus the audit-only governance
> changes that carry this packet. Revised after the owner's first REV-C01A
> review to add disposition, library-state, ZoneFill-staleness, revision-fence,
> derived-cache, and title-block-claim evidence. This document inventories what exists; it does
> not ratify Product Revision Engine vocabulary, lifecycle, policy, or code.

## Question this audit answers

What does Datum currently call a revision, what state does each such mechanism
actually identify, where is it persisted and mutated, and which product-level
revision capabilities are absent or contradicted by existing prose?

The answer is deliberately separated into:

- **Observed fact:** directly demonstrated by cited code, governing prose, or a
  reproducible repository check.
- **Interpretation:** the consequence of those facts for later specification.
- **Recommendation:** a candidate direction for REV-C02 and later work; no
  recommendation in this packet is approved by accepting REV-C01A.

## Executive finding

Datum has a strong **technical change substrate**, but it does not yet have a
Product Revision Engine.

The implemented substrate provides stable object identity, monotonic object
revision fields, a deterministic model fingerprint, optimistic-concurrency
guards, journaled typed operations, proposals, exact library pins, variant
revision fields, waiver/deviation dispositions, bounded ZoneFill staleness,
revision fences, and model-revision-stamped generated evidence. These mechanisms
support replay, bounded staleness detection, diff, undo, and reproducibility.

No implemented Datum authority currently creates or governs configuration
items, approved baselines, engineering changes, document revisions, releases,
effectivity, approval signatures, status accounting, transmittals, or retention.
Git is used by repository tooling and evidence stamps, but there is no engine
integration that maps Git commits or tags to an engineering release.

Several existing names can be mistaken for product revision authority even
though they are not: the CLI `release` check profile, a GUI `rev` label derived
from `source_revision`, free-form manufacturing prefixes, proposal `Accepted`
state, and a model-attachment `approved: bool` field. Existing decision and
research prose also overstates some implemented records. Those collisions must
be resolved before a product mechanism is specified.

## Authority inventory

| Existing concept | Implemented identity and persistence | Mutation / production path | What it actually proves | What it does **not** prove |
|---|---|---|---|---|
| Object revision | `ObjectRevision(u64)` on resolved `DomainObject`; absent JSON fields resolve as `0` ([substrate/mod.rs](../../crates/engine/src/substrate/mod.rs)) | Typed operations can guard and bump object revisions through `commit()` | A particular domain object's technical version and stale-write protection | Approval, release, effectivity, or why the version is fit for use |
| Model revision | `ModelRevision(String)` on `DesignModel`; its computation is SHA-256 of project ID, included shard path/hash pairs, and object ID/revision/source-shard triples | Computed from resolved/authored state when no valid journal exists; when a journal exists, the resolver carries the last valid transaction's recorded `after_model_revision`; selected evidence and sidecars are excluded | Technical identity used for concurrency, journal chaining, and authored-state staleness | An issued engineering revision, approved baseline, document revision, or Git identity; the computation still omits the accepted transaction tip required by decision 000D |
| Transaction / journal lineage | Append-only `.datum/journal/transactions.jsonl`; transaction UUID, batch UUID, before/after model revisions, provenance, diff, operations, inverse operations | The journaled commit path stages shards, appends the record, promotes staged bytes, and updates the cursor | A replayable typed mutation with actor/source/reason, technical before/after identity, and undo/redo linkage | Timestamp, parent transaction DAG, approver/signature, affected shard hashes, validation disposition, baseline membership, release or effectivity |
| Revision-guard surfaces | Journaled commit requires `expected_model_revision`; MCP write/proposal/apply tools require a pinned context fence containing context ID, expected model revision, and accepted transaction tip; daemon native writes reject stale model revisions; CLI undo/redo can guard both model revision and transaction tip; GUI supervision reports accepted tip | Context-fence validation, daemon refusal, commit validation, optional CLI undo/redo guards, and read-only GUI/context projections | Cross-surface stale-context refusal and visibility of both model identity and journal position | A product baseline, approval, or release; carrying both values separately does not make `ModelRevision` incorporate the journal tip |
| Proposal state | Sidecar `Proposal` with `Draft`, `Accepted`, `Deferred`, `Rejected`, or `Applied`, pinned to `prepared_against: ModelRevision` | Proposal create/preview/validate/accept/apply paths; application returns through journaled commit | Reviewed intent and whether a proposed operation batch may be applied to its prepared model | A formal engineering change order, configuration-control-board approval, release approval, or issued revision |
| Library revision pin | `LibraryBinding.pinned_object_revision` and `RevisionedRef { object_id, object_revision }` | Component/library typed operations through the shared mutation path | Exact technical library object revision selected by a project | Library qualification, independent approval, effectivity, or controlled uptake into a released product |
| Pool lifecycle and symbol review annotations | Pool `Part.lifecycle` is `Active`, `Nrnd`, `Eol`, `Obsolete`, or `Unknown`; symbols may carry `LibraryCheckState` and `LibraryObjectProvenance` review fields | Pool/library object authoring | Supply/lifecycle classification and object-local check/reviewer annotations | A governed library-release lifecycle, typed approval role/signature, product effectivity, or baseline qualification |
| Model-attachment review | `ModelReviewState { approved: bool, reviewed_by, reviewed_at }` inside pool data | Pool-object authoring | A small, object-local review annotation | A typed role, immutable signature, formal approval record, baseline, or release authority |
| Variant revision | `VariantOverlay` carries `base_model_revision` and `variant_revision: ObjectRevision` | Variant overlay operations through journaled source shards | Technical identity of a sparse variant overlay against a base model | A released product variant, variant effectivity, or approved option configuration |
| Rules and standards basis | Rule objects carry their own revision counters; check coverage includes strings such as `rule_revision` and `revision_or_profile`; IPC basis `revision` identifies a standards edition | Rule operations bump stored rule revisions; check producers record configured basis | Technical rule evolution and the standards/profile basis a check says it evaluated | Approval of a ruleset, verified standards conformance, or product release |
| Manufacturing plan, panel, and output-job revision | Each authored record carries `object_revision`; names/prefixes are ordinary strings | Manufacturing/output-job typed operations | Technical version of the output recipe or panel definition | That a prefix such as `release-a` or `rev-a` is a governed revision or release |
| Generated artifact/run identity | `OutputJobRun`, `ArtifactProductionProjection`, and `ArtifactMetadata` pin `ModelRevision`; files/projections carry SHA-256 evidence | Generated-evidence persistence paths, intentionally outside authored model-revision changes | Which technical model generated an output and whether recorded bytes validate | Approved configuration membership, document issue, signed release, recipient, effectivity, or retention |
| Waiver and accepted-deviation disposition | `CheckWaiver`; `CheckDeviation { accepted_by, approval_status: Accepted }`; check findings use `active`, `waived`, or `accepted_deviation`; typed create/delete waiver/deviation operations are journaled | Native-write waiver/deviation facade and schematic-root operations; check projection applies dispositions before summary/gating | An authored disposition can prevent a matching finding from counting as an active error in the `release` check gate | A typed approval role/signature, configuration-control approval, effectivity, expiration, or product release |
| Check profile named `release` | Aggregate check profile over relationship, ERC, DRC, standards, and manufacturing domains after waiver/deviation disposition | CLI check runner refuses only when `active` error findings remain; `waived` and `accepted_deviation` findings do not count as active errors | Deterministic checks were run and no undispositioned active error finding blocked that gate | A released baseline or formal human/role approval |
| ZoneFill stale projection | `ZoneFill` pins `source_zone_revision` and `model_revision`; resolver comparison can derive typed `ZoneFillState::Stale`; checks expose `zone_fill_stale` | Generated-evidence resolution and check production | Bounded dependency staleness for persisted copper-fill evidence against its source zone/model | General dependency-impact propagation across Design, libraries, Publish, manufacturing, and released artifacts |
| GUI `rev` presentation | Menubar/status-bar text truncates `scene.source_revision`; schematic import synthesizes a source string from project and sheet UUIDs | Scene import and renderer projection | A short source/scene token is visible | A Datum engineering or document revision |
| Workspace/session state | Pane layout, focus, zoom, filters, console, and related consumer state are expressly never journaled as design operations | Session/UI mutation only | Current user presentation context | Design, document, configuration, or release revision |
| Git repository identity | Repository governance/proof scripts query Git commits; some runtime proof metadata can carry a `source_revision` string | External repository tooling, not the Datum engine | Source-tree or test-evidence identity where explicitly recorded | Datum approval, product baseline, document revision, release, or standalone revision control |

## Exact implemented boundaries

### Technical revisions and shard authority

**Observed fact.** `ObjectRevision` and `ModelRevision` are thin technical
identity types. `DesignModel` collects authored objects plus sidecar and
generated evidence ([substrate/mod.rs](../../crates/engine/src/substrate/mod.rs)).
The resolver classifies project, schematic, board, rules, pool, relationship,
component, variant, manufacturing, panel, and output-job shards as
`AuthoredDesign`; proposals/import maps/forward-annotation reviews as
`SidecarMetadata`; and runs, checks, fills, and artifact metadata as
`GeneratedEvidence`.

**Observed fact.** The model fingerprint excludes artifact metadata, artifact
runs, output-job runs, check runs, zone fills, import maps, proposals, and
forward-annotation reviews. Generated evidence can therefore be recorded or
replayed without changing the authored `ModelRevision`.

**Observed fact.** `ProjectResolver` computes a candidate fingerprint but, when
a valid journal exists, installs the last transaction's recorded
`after_model_revision` as the resolved model revision
([project_resolver.rs](../../crates/engine/src/substrate/project_resolver.rs)).
This preserves journal continuity; it does not change the fact that
`compute_model_revision` itself omits the transaction tip ratified by decision
000D.

**Interpretation.** This is a valuable distinction between authored model state
and evidence about that state. A future configuration baseline cannot simply be
an alias for `ModelRevision`, because the required release evidence is
deliberately outside that fingerprint.

### Commit and journal authority

**Observed fact.** `OperationBatch` contains an optional expected model revision,
provenance, and typed operations. `TransactionRecord` contains transaction and
batch IDs, normal/undo/redo linkage, before/after model revisions, provenance,
optional agent provenance, object diff, operations, and inverse operations
([transaction.rs](../../crates/engine/src/substrate/transaction.rs)).

**Observed fact.** Journaled commit requires an exact expected model revision,
validates optional object-revision guards, stages affected shard bytes, appends
the transaction journal, promotes the staged writes, and updates the journal
cursor ([commit.rs](../../crates/engine/src/substrate/commit.rs)). Repository
write-fence gates pass and preserve the one canonical mutation path, with the
explicit legacy converter exceptions enumerated by the guard.

**Observed fact.** Revision protection is also exposed outside `commit()`. The
MCP context fence requires `context_id`, `expected_model_revision`, and
`accepted_transaction_tip` for guarded write/proposal/apply surfaces
([context_revision_fence.py](../../mcp-server/context_revision_fence.py)); the
engine daemon refuses a stale expected revision
([dispatch.rs](../../crates/engine-daemon/src/dispatch.rs)); CLI undo/redo can
validate both the model revision and exact journal-tip transaction
([journal_mutation.rs](../../crates/cli/src/commands/project/journal_mutation.rs));
and GUI supervision projects the accepted transaction tip
([supervision.rs](../../crates/gui-protocol/src/supervision.rs)).

**Interpretation.** This is strong technical provenance and concurrency control.
It is not, by itself, controlled change authorization or an audit-complete
engineering release record.

### Proposals are not engineering releases

**Observed fact.** A proposal records rationale, affected objects, checks and
finding fingerprints, source, a prepared-against model revision, and lifecycle
state. `Accepted` permits later application; `Applied` links to a transaction
([proposal.rs](../../crates/engine/src/substrate/proposal.rs)).

**Interpretation.** The proposal mechanism is a reusable foundation for change
review, but its current states and fields cannot carry formal change authority,
multi-role approval, effectivity, implementation verification, supersession, or
release.

### Manufacturing evidence is technically pinned, not released

**Observed fact.** Output jobs, manufacturing plans, and panels have ordinary
object revisions. Output runs and artifacts record the source model revision;
artifact files and projections record byte hashes
([artifact.rs](../../crates/engine/src/substrate/artifact.rs)).

**Observed fact.** The CLI profile called `release` is defined only as an
aggregate deterministic check profile, and its gate only tests for active error
findings ([gate.rs](../../crates/cli/src/commands/check/gate.rs),
[run_view.rs](../../crates/cli/src/commands/check/run_view.rs)).

**Interpretation.** Datum can prove which technical state generated bytes and
whether current deterministic checks objected. It cannot yet prove that an
authorized role approved an exact configuration and issued those bytes.

### Finding disposition and bounded staleness

**Observed fact.** `CheckWaiver` and `CheckDeviation` are authored schematic
dispositions; a deviation carries `accepted_by` and the currently single-valued
`DeviationApprovalStatus::Accepted`
([check_disposition.rs](../../crates/engine/src/schematic/check_disposition.rs),
[schematic/mod.rs](../../crates/engine/src/schematic/mod.rs)). Typed
create/delete operations persist both through the journal. Check projection
changes matching findings from `active` to `waived` or `accepted_deviation`, and
the `release` gate counts only `active` errors.

**Interpretation.** Datum therefore has a real disposition authority that
modulates check gating. Its free-form actor fields and one-state approval enum
remain weaker than the controlled roles, attestations, policy, and effectivity
required for formal release approval.

**Observed fact.** Persisted `ZoneFill` evidence records both source-zone and
model revisions. Resolution compares those identities, derives
`ZoneFillState::Stale`, and emits a typed `zone_fill_stale` finding
([zone_fill.rs](../../crates/engine/src/substrate/zone_fill.rs)).

**Interpretation.** Dependency staleness is not wholly absent. One bounded
generated-evidence path is implemented. What remains absent is a general typed
impact graph across library, Design, rules, Publish, manufacturing, documents,
and released configurations.

### Document and Publish authority

**Observed fact.** The governed Publish Space specification reserves
configuration identification, baselines, approvals, revision allocation,
release, effectivity, supersession, withdrawal, status accounting,
reproduction, and audit to the Product Revision Engine. Publish owns its
render/export, staleness determination and display, and refusal substrate; the
Revision Engine supplies the working or immutable-baseline configuration context
that makes that staleness determinable. No corresponding document/release
structs or typed operations exist in the engine today
([PUBLISH_SPACE_SPEC.md](../../specs/PUBLISH_SPACE_SPEC.md)).

**Observed fact.** Earlier title-block research says revision ledgers and status
are projections from the commit journal and suggests auto-appending revision
rows. The current Product Revision Engine research explicitly identifies that
as a conflation: technical commits are evidence, not automatically issued
document revisions ([TITLE_BLOCK_AND_DOC_CONTROL_RESEARCH.md](TITLE_BLOCK_AND_DOC_CONTROL_RESEARCH.md),
[PRODUCT_REVISION_ENGINE_RESEARCH.md](PRODUCT_REVISION_ENGINE_RESEARCH.md)).

**Interpretation.** Title blocks must ultimately project controlled document and
release facts. The transaction journal may support the evidence chain but cannot
be the sole revision-table authority.

### Workspace state

**Observed fact.** GUI workspace layout, focus, camera, filters, and transient
state are consumer/session state and explicitly never enter `commit()` or the
design journal ([workspace_layout.rs](../../crates/gui-protocol/src/workspace_layout.rs)).

**Interpretation.** A product baseline should not capture incidental UI state.
Named workbench/profile persistence may have its own compatibility/schema
version later, but it is not an engineering product revision.

### Git boundary

**Observed fact.** Pool documentation already states that JSON is Git-friendly
while Git is not the product-level revision model
([POOL_ARCHITECTURE.md](../../docs/POOL_ARCHITECTURE.md)). A repository search
finds Git process invocation in governance/test scripts, but none in the engine
for creating, approving, or resolving engineering releases.

**Interpretation.** The planned `GitAdapter` is absent. Standalone Datum release
authority and any optional mapping to commits/tags remain specification work.

## Contradiction and collision matrix

| ID | Evidence in conflict | Observed discrepancy | Required later disposition |
|---|---|---|---|
| REV-GAP-01 | Decision 000D says `ModelRevision` hashes object revisions **plus accepted-transaction tip**; `compute_model_revision` does not hash the journal tip. The resolver carries the last valid transaction's recorded `after_model_revision`, while MCP/context/GUI surfaces track the transaction tip separately | Code and controlling decision describe different technical identities; journal-tail carry-forward and dual revision/tip fences confirm rather than resolve the omission | REV-C03 must preserve the higher authority or deliberately amend it with migration consequences |
| REV-GAP-02 | Decision 000D says transactions include a parent transaction DAG field, affected shard IDs/hashes, validation state, and cache invalidations; `TransactionRecord` has none of those fields | The implemented journal is less complete than the ratified storage model claims | Specify which fields belong in technical journal records versus Product Revision Engine records |
| REV-GAP-03 | Decision 000D says artifact metadata includes timestamp and ties artifact revision to output-job, board/panel, and variant revisions; `ArtifactMetadata` lacks timestamp and exact per-source object revisions | Current artifacts pin the model and hashes but not the complete claimed reproduction tuple | Define release-manifest and artifact-evidence ownership, then reconcile code/spec |
| REV-GAP-04 | Decision 000D says transactions are the sole producer of revisions; resolver materialization accepts missing `object_revision` as `0`, and `ModelRevision` is recomputed directly from shards/objects | “Revision” currently includes both transaction-produced counters and resolver-computed content identity | Narrow the doctrine wording or enforce transaction-only creation/migration semantics |
| REV-GAP-05 | Title-block research proposes journal-derived, auto-appended revision ledgers; current revision research says commits are not issued revisions | Research sources disagree about document revision authority | REV-C03 must make the revision engine authoritative and revise the title-block research before implementation |
| REV-GAP-06 | GUI renders `rev` from `scene.source_revision`; schematic scenes synthesize a project/sheet identity string | User-facing “rev” can label a value that is not an engineering revision | Rename the current chrome or bind it to a future typed revision projection; visual disposition belongs in REV-C06 |
| REV-GAP-07 | CLI calls a deterministic check aggregate `release`, but no release object or transition exists | Passing the gate can be misread as completing release | Rename or explicitly scope the gate when Product Revision Engine vocabulary is ratified |
| REV-GAP-08 | `ModelReviewState.approved: bool`, proposal `Accepted`, symbol check/reviewer annotations, `CheckDeviation.accepted_by`, and waiver/deviation dispositions use approval-like language without controlled roles/signatures; the latter also changes `release`-gate counting | Several local review/acceptance/disposition states can be confused with formal configuration approval, and one has direct gate consequences | Establish typed approval classes and prevent weaker states from satisfying formal release policy without an explicit profile disposition |
| REV-GAP-09 | Manufacturing tests and records permit free-form `rev-*` / `release-*` names and prefixes | A label can look authoritative without any lifecycle relationship | Treat these strings as names only and reserve governed revision projections for engine-owned fields |
| REV-GAP-10 | Generated evidence is excluded from `ModelRevision`, while a release must include exact checks and output bytes | One fingerprint cannot identify the complete released configuration/evidence set | A baseline/release manifest needs its own immutable identity over both controlled inputs and qualifying evidence |
| REV-GAP-11 | Title-block research claims an **existing** six-state PLM machine and a repository `EngineeringChangeOrder`; repository code contains neither authority | Research presents proposed concepts as already implemented and then derives document release/history behavior from them | Correct the research claims and require later document-control design to cite actual Product Revision Engine objects |
| REV-GAP-12 | Decision 000D ratifies persisted derived caches carrying graph revision and every source object revision, with resolver cache-staleness validation; no corresponding general resolved-net/derived-cache record or resolver implementation exists | The ratified revision-keyed cache mechanism is absent; ZoneFill is a bounded generated-evidence staleness path, not the general cache system described | Inventory intended cache classes in REV-C03/C05 and either implement the ratified mechanism or amend 000D explicitly |

## Missing product capabilities

The following are **absent**, not merely hidden behind an unfinished GUI:

| Missing authority | Evidence needed from a future implementation |
|---|---|
| Configuration-item designation and membership | Stable CI identities, scope, relationships, and queryable inclusion rules |
| Configuration baseline | Immutable manifest of exact Design, library, rule, variant, Publish, manufacturing, check, waiver/deviation, artifact, and generator identities |
| Engineering change control | Change identity, rationale, affected/unchanged analysis, disposition, implementation and verification state, supersession, and trace links |
| Typed roles and approvals | Actor identity, role/capacity, intent, time, signature/attestation evidence, policy evaluated, revocation/correction rules |
| Document/package revision | Independent issued identity and lifecycle, not inferred from edit count, transaction count, filename, or title-block text |
| Release transition | Deterministic refusal/commit boundary that freezes an approved baseline and records qualifying evidence |
| Effectivity | Product/variant/serial/lot/build/customer/site/date applicability and successor handling |
| Status accounting | Current and historical queries over CIs, changes, baselines, approvals, releases, discrepancies, and implementation |
| Configuration audit | Clause-addressable requirements, expected evidence, pass/fail/not-applicable/tailored dispositions, and reproducible report identity |
| Transmittal and delivery | What exact package was delivered, to whom, when, by whom, under which classification/export policy |
| Retention and record integrity | Retention policy, immutable evidence, correction/supersession, clock/time authority, export and independent verification |
| Standalone local authority | Durable engine records and workflows that work without Git or a remote service |
| Optional Git adapter | Explicit mapping and divergence state among Datum identities and Git commits/tags without transferring release authority to Git |
| General dependency impact propagation | Beyond implemented ZoneFill revision comparison and artifact/model pins, typed stale/affected/unaffected traversal from changed library/design/rules through Publish, manufacturing, documents, and released artifacts |

## Private-writer and mutation-path result

The audit executed the repository's mutation fences:

```text
python3 scripts/check_schematic_private_writers.py
python3 scripts/check_daemon_write_parity.py
python3 scripts/check_resolver_raw_loads.py
python3 scripts/check_dependency_authority.py
```

All passed. The private-writer guard explicitly enumerated the permitted legacy
KiCad converter and generated-evidence paths rather than reporting a false zero.
No new dependency was introduced. This establishes that the Product Revision
Engine can be designed on top of the existing one-mutation-path substrate; it
does not erase the documented terminal legacy-converter exception.

The first owner-review correction round also ran focused behavioral proof:

```text
python3 -m unittest test_context_revision_fence.py  # from mcp-server/: 13 passed
python3 scripts/run_cargo_guarded.py --workload interactive -- \
  cargo test -p eda-engine zone_fill --lib          # 24 passed
python3 scripts/run_cargo_guarded.py --workload interactive -- \
  cargo test -p eda-engine waiver --lib             # 12 passed
python3 scripts/run_cargo_guarded.py --workload interactive -- \
  cargo test -p datum-eda-cli project_accept_deviation  # 3 passed
```

These tests establish the newly inventoried behavior; they do not prove any
formal product release or standards conformance.

## Reproduction commands

Run from the repository root:

```bash
# Core technical identities and fingerprint membership.
rg -n "ObjectRevision|ModelRevision|compute_model_revision|SourceShardAuthority" \
  crates/engine/src/substrate/mod.rs

# Journal and proposal record fields and enforcement.
rg -n "struct OperationBatch|struct TransactionRecord|struct Proposal|ProposalStatus|journaled commit requires" \
  crates/engine/src/substrate

# Cross-surface revision and journal-tip guards.
rg -n "expected_model_revision|accepted_transaction_tip|expected_tip_transaction" \
  mcp-server crates/engine-daemon crates/cli crates/gui-protocol

# Library, variant, rules, manufacturing, and generated-evidence concepts.
rg -n "pinned_object_revision|variant_revision|rule_revision|struct OutputJob|struct ArtifactMetadata|struct ModelReviewState|LibraryCheckState|LibraryObjectProvenance|enum Lifecycle" \
  crates/engine/src

# Finding dispositions and bounded ZoneFill staleness.
rg -n "CheckWaiver|CheckDeviation|DeviationApprovalStatus|accepted_deviation|ZoneFillState|zone_fill_stale" \
  crates/engine/src crates/cli/src

# Potentially misleading user/product language.
rg -n "ReleaseCheckGateView|profile_id: \"release\"|source_revision|draw_text\(\"rev\"" \
  crates/cli crates/gui-protocol crates/gui-render

# Git integration: repository tooling should appear; engine release integration should not.
rg -n "git (rev-parse|tag|commit)|Command::new\(\"git\"\)|\[\"git\"" \
  crates scripts

# Existing behavioral fences.
python3 scripts/check_schematic_private_writers.py
python3 scripts/check_daemon_write_parity.py
python3 scripts/check_resolver_raw_loads.py
python3 scripts/check_dependency_authority.py
```

## Audit coverage and limits

This packet covers implemented Rust and MCP record types and mutation paths; governing
storage, workspace, Publish, pool, library, and title-block prose; GUI revision
presentation; repository Git invocation; and known write-fence exceptions.

It does **not** decide:

- which standards clauses Datum will claim to implement;
- the final vocabulary, object graph, lifecycle, roles, or signature scheme;
- whether existing technical revision semantics should change;
- the Git adapter mechanism;
- the visual workflow;
- implementation sequencing beyond the already-governed REV-C02..REV-C08 path.

Those decisions require standards evidence and later owner dispositions.

## Recommendations carried forward, not approved here

1. Keep `ObjectRevision`, `ModelRevision`, and the transaction journal as
   technical substrate identities; do not display them as issued revisions.
2. Add a distinct immutable baseline/release manifest that can include exact
   technical inputs and qualifying evidence excluded from `ModelRevision`.
3. Reuse proposal and commit primitives, but introduce explicit engineering
   change, approval, effectivity, and release semantics rather than overloading
   their current statuses.
4. Make title blocks, document registers, status chrome, and output names
   projections of typed revision-engine facts.
5. Keep Git optional and subordinate through a Datum-owned adapter.

## REV-C01A owner decision boundary

### Review history

- **Round 1 — revise (2026-08-23):** the owner required the packet to add
  waiver/accepted-deviation disposition authority; pool lifecycle and symbol
  review annotations; bounded ZoneFill staleness; MCP, daemon, undo/redo, and GUI
  revision-guard surfaces; the false title-block-research claims about an
  existing six-state PLM machine and `EngineeringChangeOrder`; and the absent
  general persisted derived-cache mechanism. The owner also corrected the
  Publish/Revision seam and required the journal-tail `model_revision` behavior
  to be explicit. This revision incorporates all six dispositions. REV-C01A
  remains pending until the owner reviews the corrected committed packet.

Approval means only:

> This packet accurately and sufficiently describes Datum's current revision
> authorities, contradictions, and missing product capabilities as the factual
> baseline for standards research.

Approval does **not** accept the recommendations above, select an architecture,
authorize implementation, or make a standards-conformance claim. A revision
response should identify any missing, incorrect, or unsupported finding so this
packet can be corrected before REV-C02.
