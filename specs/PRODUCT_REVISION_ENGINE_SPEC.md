# Datum Product Revision Engine Specification

> **Status:** Governed planning contract ratified by Product Mechanics 034.
> This specification does not authorize implementation, a new dependency,
> certification claims, or Publish Space development.
>
> **Authority:** Product Mechanics 034, the REV-C01 through REV-C07 evidence
> corpus, Product Mechanics 000D/001/010/020/029/033, and the canonical Datum
> operation/commit/journal doctrine.

## 1. Purpose and boundary

The Product Revision Engine is Datum's standalone authority for engineering
configuration, change, baseline, human revision, approval, effectivity, release,
controlled-document issue, package, delivery, standing, status accounting,
reproduction, retention, and standards-audit records.

It extends but never replaces the technical revision system. Every authored
engine mutation uses typed operations through the one canonical `commit()` and
journal path. GUI, CLI, MCP, scripts, import/exchange, and future adapters receive
the same authorization, validation, refusal, and query semantics.

This specification does not select Rust modules, storage layout, cryptographic
algorithms, third-party libraries, multi-writer merge, or Publish composition
implementation.

## 2. Normative laws

1. Technical revisions, transactions, EngineeringRevisions, baselines,
   Releases, document issues, packages, and transmittals are distinct types.
2. Human revision identity is allocated only by an explicit profile-governed
   boundary. Edits, saves, exports, timers, Git events, filenames, labels, and
   deliveries cannot allocate it.
3. Issued records and their covered bytes are immutable. Correction creates a
   successor record or typed standing fact.
4. Every baseline member and release input is exact and resolvable; `latest`,
   floating library bindings, mutable paths, and unqualified digests refuse.
5. Every approval covers an exact canonical target digest. Changed targets
   invalidate applicable prior attestations.
6. Every Affected/Unaffected claim has a typed witness. Incomplete evaluation is
   `ImpactUnknown`, never `Unaffected`.
7. Historical evidence remains valid against its frozen baseline; successor
   difference does not retroactively make it stale.
8. Git and network services are optional adapters and never product authority.
9. Profiles may configure obligation, roles, terminology, and presentation but
   cannot redefine core identity kinds or typed fact meanings.
10. Under any profile that has not explicitly adopted earlier control, the
    engine never blocks, prompts, or delays Design authoring. The only gates are
    release certification/issuance and earlier control explicitly adopted by
    Project policy.
11. Ignoring the revision engine entirely is a supported, first-class workflow,
    not a degraded one.
12. No visual surface, title block, mutable organizer, or audit report becomes a
    rival engineering authority.

## 3. Identity and vocabulary

### 3.1 Two clocks

The technical clock starts with authored work and records every committed
mutation through `Transaction`, `ObjectRevision`, `ModelRevision`, and the
accepted journal tip. The human revision clock starts only at a declared
profile-governed issue boundary.

`ConfigurationRef` is the exact resolution seam:

```text
ConfigurationRef =
  Working { model_revision, accepted_transaction_tip }
  | Baseline { baseline_id, baseline_digest }
```

Git identity is adapter metadata and is never a `ConfigurationRef` variant.

### 3.2 Required identity classes

The engine shall provide non-interchangeable stable IDs for at least:

- `ConfigurationItem`, `ConfigurationBaseline`, `EngineeringChange`,
  `EngineeringRevision`, and `RevisionReservation`;
- `ApprovalAttestation`, `Effectivity`, `ReleaseCandidate`, and `Release`;
- `ControlledDocument`, `DocumentIssue`, `ReleasePackage`, and `Transmittal`;
- `StandardsProfile`, `AuditEvaluation`, retention/hold/disposition records;
- dependency snapshots, impact evaluations, evidence/regeneration records, and
  reproduction manifests/attempts; and
- adapter mappings, exchange receipts, and external-change candidates.

Each governed record carries a stable machine ID, scoped human label where
applicable, scheme-interpreted issue label where applicable, and
algorithm-qualified integrity digest. Labels, paths, and tree positions never
substitute for stable IDs.

### 3.3 Distinct facts

The engine shall preserve these meanings without collapse:

| Fact | Meaning |
|---|---|
| `Changed` | A relevant source identity, revision, or semantic observation differs. |
| `Affected` | A witness proves a consumer's governed meaning, validity, or output can change. |
| `Unaffected` | A witness proves the changed observations are outside the consumer's declared sensitivity. |
| `ImpactUnknown` | Datum cannot prove affected or unaffected. |
| `Stale` | Derived evidence inputs differ from the target configuration/context. |
| `Orphaned` | A required producer or input identity cannot resolve. |
| `NonReproducible` | Required output cannot be reproduced byte-identically under its declared contract. |

`CanonicalEquivalent`, visual equality, and electrical equivalence are
diagnostics and never satisfy `ByteIdentical`.

## 4. Policy and profile resolution

The engine shall accept one explicit resolved Project policy input. Sequential
Alphanumeric (Legacy) is the factory new-Project revision profile. ISO
19650-oriented and future organization/standards profiles use the same core
records. ISO revision and suitability/status are separate fields and axes.

Global Preferences is a later authority. It must define storage, precedence,
provenance, managed policy, migration, synchronization, new-Project seeding, and
accessible presentation. A Project receives a copied policy; later global
default changes cannot silently rewrite it.

Profiles may define revision schemes, reservation rules, applicability,
approval roles/quorum/order/separation, effectivity obligations, audit and
retention requirements, adapter/signature/trusted-time requirements, controlled
terminology, and display detail. Profiles may not:

- turn strings or external identities into core Datum states;
- infer aggregate revision roll-up;
- treat cryptographic validity as scoped authorization;
- erase or reinterpret issued history;
- hide unavailable required evidence by weakening policy; or
- replace the one engine mechanism with a competing lifecycle.

## 5. Authority records

### 5.1 Configuration control

`ConfigurationItem` shall designate one existing Datum authority or explicitly
authored aggregate as a revision-bearing controlled subject without copying it.
CI parentage and product topology are optional and never inferred from storage.
Exactly one CI namespace owns each EngineeringRevision.

`ConfigurationBaseline` shall be an immutable, deterministically ordered
manifest of exact authority references, technical revisions, context, governing
changes, departures, profile references, establishment attestations, provenance,
and digest. `BuildIdentity` may label the baseline but cannot replace it.

`EngineeringRevision` shall identify one CI's approved baseline under one
scheme and namespace, with predecessor, governing changes, allocation policy,
and issuing Release. Successor-work collection alone binds no revision identity.
`RevisionReservation` is separate, provisional, expiring authority and never
released title-block truth.

### 5.2 Engineering change and departure

The engine shall own one `EngineeringChange` with append-only lifecycle:

```text
Draft -> ImpactReview -> Authorized -> Implementing -> Verification -> Closed
          \-> Rejected / Deferred / Cancelled
```

Lifecycle state is projected from typed events. Profiles may label stages or
permit controlled shortcuts; they cannot invent competing record kinds.
Authorization does not prove implementation, verification does not issue, and
closure requires released successor evidence or explicit no-release disposition.

Waivers and accepted deviations shall remain distinct bounded departure records
with exact requirement/finding, scope, authority, validity, effectivity, and
disposition. Existing check waivers/deviations require explicit mapping before
they can satisfy release policy.

In lightweight operation, the first committed divergence from a released
baseline shall quietly open identity-free Draft successor work and collect
linked transactions. Explicit preparation may create the approved provisional
reservation. Project policy may explicitly require authorized Change authority
before Design mutation.

### 5.3 Approval, role, and effectivity

`ApprovalAttestation` shall be append-only and cover a domain-separated,
versioned canonical target statement containing Project, record kind/ID/digest,
intent, role assignment, policy version, actor identity, asserted-time source,
and applicable evidence digests. Revocation or correction creates a typed later
event.

Authorization shall evaluate capability, scope, effective interval, target,
method, order, quorum, independence, and separation of duty. Display role names
confer no authority. Core capabilities include Author, Change coordinator,
Reviewer, Verifier, Configuration authority, Release authority, Auditor, and
Records authority.

`Effectivity` shall use typed predicates and an explicit resolution context.
Unsupported, ambiguous, or unknown selection refuses; it never means universal.
When enumerable, a Release shall retain both the authored expression and exact
resolved population.

### 5.4 Release

`ReleaseCandidate` shall be mutable, revision-guarded preparation authority over
proposed allocations, baseline manifest, changes, departures, effectivity,
evidence, document issues, packages, approval policy, attestations, standards
evaluations, and readiness findings. Covered source/evidence/policy changes mark
the evaluation stale and invalidate changed-target attestations.

`ReleaseConfiguration` shall be atomic. It shall create a separate immutable
Release, baseline, affected EngineeringRevisions, prepared DocumentIssues, and
ReleasePackages only after every refusal is clear. Unchanged baseline members
shall reuse existing issued revisions. It shall never mutate the candidate or
working Design model into released authority.

Issued standing shall be represented only through typed
`SupersessionEstablished`, `AuthorizationWithdrawn`, and
`ObsolescenceDeclared` facts. Profiles govern applicability, authority,
rationale, effectivity, allowed sequences, and displayed terminology.
`Current`/`Historical` are projections. Delivery cannot change standing.

### 5.5 Documents, packages, and delivery

`PublishSet` is an editable ordered Publish selection and owns no document or
release identity. `ControlledDocument` shall provide stable authored-publication
identity and be designated by exactly one revision-bearing CI.

`DocumentIssue` shall be immutable and bind the ControlledDocument, its CI's
EngineeringRevision, baseline `ConfigurationRef`, exact source revisions,
render/generator context, issued output bytes, approvals, issuing Release, and
digest. `ReleasePackage` shall be an immutable ordered manifest of exact issued
records. `Transmittal` shall record the exact package digest, sender, recipients,
delivery/egress policy, time evidence, and delivery/receipt evidence.

Retransmission of unchanged bytes creates a new Transmittal, not a new issue.
Deleting a bound Publish source refuses until every ControlledDocument
dependency is explicitly removed or retargeted.

### 5.6 Standards, audit, and records

`StandardsProfile` shall identify authority, edition, applicability, approved
requirement dispositions, organization policy, effective interval, and digest.
Each requirement disposition shall be typed as Applicable, Tailored,
EquivalentControl, NotApplicable, Unresolved, or BlockedNormativeText and shall
retain its rationale, control/evidence/verification rule, and approval.

`AuditEvaluation` shall freeze profile version, scope, evidence set, engine
version, requirement results, findings, time, and digest. A passing evaluation
claims only satisfaction of that configured matrix. It cannot claim external
certification or acceptance.

Retention policies, holds, and record dispositions shall be typed, effective,
authorized, and auditable. Disposition cannot violate an active hold or silently
erase the historical authority chain.

## 6. Dependency impact, staleness, and regeneration

The engine shall freeze a deterministic dependency snapshot with exact nodes,
typed edges, sensitivities, origins, evaluator identities/revisions,
completeness, unresolved inputs, configuration, and digest.

Semantic comparison shall use stable identity and typed observations. Durable
impact results shall record source/target configuration, dependency snapshot,
graph completeness, per-subject result, witness paths, reason code, required
actions, and optional reviewed disposition. Missing edge/evaluator coverage must
remain visible as `ImpactUnknown`.

Library updates shall produce previewable uptake candidates and never silently
rebind placed Design objects. Adoption is an explicit journaled mutation and
impact input.

Staleness shall compare declared evidence inputs against one explicit target
configuration. `Orphaned` shall identify unresolvable required inputs/producers.
Historical release evidence remains frozen against the historical baseline.

Regeneration plans shall be deterministic and topologically ordered. They may
reuse evidence proven current against the same exact inputs and policy. Execution
creates immutable successor evidence and never edits prior evidence.

Baseline comparison shall align stable identities and report only Added,
Removed, Modified, Retargeted, or Unchanged. Administrative rename is metadata
within Modified. Every controlled difference requires a governing Change,
departure, or accepted administrative disposition before release.

## 7. Reproducibility and authenticity

A `ReproductionManifest` shall bind:

- exact source subjects and dependency snapshot;
- algorithm-qualified producer/build identity;
- effective settings and ordered invocation;
- every influential environment input or explicit exclusion; and
- sorted specified outputs with logical name, kind, byte count, algorithm, and
  digest.

Floating network data, undeclared local files, uncontrolled clocks/randomness,
and `latest` dependencies are prohibited. A `ReproductionAttempt` shall record
the independently resolved environment, produced outputs, comparison evidence,
executor, digest, and exactly one result: ByteIdentical, OutputMismatch,
Unavailable, or ExecutionFailed.

Required reproducibility succeeds only when every specified output is
byte-identical. Later failure creates a new attempt result and never changes the
Release, issued bytes, approval, or standing. Byte equality, signature validity,
actor identity, scoped authorization, policy satisfaction, and trusted time are
separate typed results.

## 8. Standalone local authority and adapters

A Project with no `.git`, Git executable, network, or remote service shall be
able to create, resolve, change, approve, baseline, issue, release, reproduce,
audit, query, back up, restore, exchange, and verify every core authority record.

The logical local authority store shall retain Project and schema identity,
canonicalization versions, Project policy versions, authority records,
append-only events, journal tip, immutable blob manifest, roles/trust evidence,
adapter mappings, exchange receipts, retention/hold records, recovery
checkpoints, and integrity root.

Commit/recovery shall validate model revision, accepted transaction tip,
authority, policy, integrity, and record invariants as one atomic batch. Recovery
shall expose the last complete state or a read-only diagnostic state; it can
never promote a partial Release. Journal order, asserted time, observed receipt
time, trusted timestamp, and peer/server receipt evidence remain distinct.

The optional Git adapter may materialize deterministic Datum state, record
algorithm-qualified mappings, mirror completed Releases, and transport exchange
payloads. Adapter failure cannot partially mutate Datum authority or roll back a
completed local Release. Git ref movement/deletion adds an observation and never
rewrites mapping history.

External content shall enter quarantine. It becomes Datum authority only after
envelope/integrity/schema/Project checks, isolated resolution, semantic
comparison and validation, translation into permitted typed operations, review,
and local commit. Transport need not be Git. Multi-writer semantic merge remains
outside this specification.

## 9. Required operation families

The public mutation surface shall include typed families for:

- CI designation/policy/retirement, standards profiles, retention, and scoped
  role assignment;
- EngineeringChange creation, affected items, impact, effectivity, transaction
  links, review, authorization, implementation, verification, and disposition;
- approvals, attestation supersession/revocation, baseline preparation and
  establishment, relationships, and revision reservation;
- ReleaseCandidate preparation/evaluation/evidence, ControlledDocument,
  DocumentIssue, ReleasePackage, atomic `ReleaseConfiguration`, standing,
  Transmittal, and receipt;
- standards audit, finding disposition, holds, record disposition, and evidence;
- dependency snapshot, semantic delta, impact disposition, library uptake,
  baseline comparison, regeneration, reproduction manifest, and attempts; and
- adapter mappings/divergence/mirroring, authority exchange, external-change
  candidate disposition, trust/credential events, and trusted-time evidence.

Bulk and template actions shall expand to these typed operations and commit
atomically. No public generic JSON-patch or private adapter writer is permitted.

## 10. Required refusals and findings

Every refusal shall carry a stable code, affected identities, controlling
policy/requirement, expected/current facts, witness where applicable, and
actionable remediation. Required families include:

- stale working configuration, unresolvable baseline member, floating release
  dependency, immutable released record, duplicate label, and reservation
  conflict;
- unaccounted difference, incomplete/unknown impact, unauthorized change,
  unverified implementation, changed approval target, unsatisfied approval,
  unresolved effectivity, invalid/expired departure, missing/stale evidence,
  unresolved standards disposition, retention/hold, and egress violation;
- dependency graph incomplete, unassessed library update, missing pinned library,
  orphaned evidence, missing regeneration prerequisite, producer/environment
  mismatch, output mismatch, and missing byte reproducibility;
- local integrity failure, unsupported algorithm, Project/exchange identity or
  prerequisite failure, replay/destination failure, invalid or unauthorized
  external signature, semantic conflict, unrepresentable external change, stale
  Release mirror, required mapping/mirror absence, and missing trusted time; and
- adapter unavailable only when active Project policy explicitly requires it.

Ordinary adapter absence is a supported capability state, not Project
corruption.

## 11. Query contract

Engine, CLI, and MCP shall expose parity for typed read-only queries covering:

- working/baseline resolution, comparison, membership reason, and as-of history;
- Change state, affected items, transactions, evidence, and witness paths;
- CI/document revision history and scheme-aware label resolution;
- baseline membership, relationships, and integrity verification;
- candidate readiness, Release inspection/verification/reproduction/comparison,
  approvals, effectivity, document issues, packages, transmittals, standing,
  status accounting, audit, retention, and disposition history;
- dependency paths, evidence freshness, library uptake preview, regeneration
  planning, and reproduction explanation; and
- local-authority integrity/export, adapter mapping/divergence/mirror status,
  authority exchange, external-change candidates, and attestation trust paths.

Queries shall not reconstruct authority from filenames or Git. Historical
queries shall identify their event/time authority and completeness. Adapter
queries shall return an explicit `AdapterAbsent` result when appropriate.

## 12. Human and accessibility contract

The visual source of truth is:

- `docs/gui/prototypes/revision-ux-shell-study.html`;
- `docs/gui/prototypes/revision-ux-impact-study.html`;
- `docs/gui/prototypes/revision-ux-release-study.html`;
- `docs/gui/prototypes/revision-ux-evidence-study.html`; and
- the non-authoritative cross-file decision index
  `docs/gui/prototypes/revision-ux-study-index.html`.

The approved Q1–Q6 dispositions are normative:

1. Permanent expanded Revision Navigator groups with informative empty states.
2. One cohesive Release pane with nine always-visible semantic summaries;
   unresolved details refuse collapse.
3. Summary-first Impact with canonical witness tree opened beside it and
   permanent Unknown/graph-scope rows.
4. One complete ordered Change hierarchy whose obligation/detail scales by
   profile without changing identity, meaning, position, or discoverability.
5. Pinned in-pane arm-then-confirm issuance with exact consequences visible
   before the second deliberate action and no modal.
6. Verdict-first reproduction over permanent counted evidence categories;
   deficient evidence refuses collapse and immutable manifest detail opens beside.

Existing one-window, recursive tiling, open-beside, focused-pane, output-only
Console, accessibility, responsive, and no-rival-authority laws remain in force.

Visible expanded revision groups are the factory default. Future Global
Preferences shall provide a presentation-only **Hide revision system** option.
Hiding cannot disable records, the journal, policy, findings, release gates, or
recovery of the presentation and cannot require data migration.

## 13. Standards, licensing, and claim boundary

The clause-addressable standards matrix controls source strength, edition,
applicability, licensing, and Datum disposition. Public scope/guidance may
support architecture; exact conformance obligations require lawful
edition-pinned normative text when marked. Audit results shall identify profile,
edition, scope/effectivity, engine version, frozen evidence, exceptions,
tailoring/equivalent controls, unresolved findings, and dispositions.

No result may claim certification, accreditation, regulator approval, customer
acceptance, or blanket standards conformance. No external normative text may be
silently paraphrased into Datum authority.

This specification adds no dependency. Cryptography, signatures, key stores,
Git libraries, PDF/export engines, and external standards content require
separate Product Mechanics 029 decisions and license review.

## 14. Migration and verification requirements

REV-C08 shall schedule bounded implementation and migration slices proving at
least:

1. local/offline end-to-end Change, baseline, approval, Release, standing,
   query, backup/restore, audit export, and integrity verification;
2. journal-tip/fingerprint and crash injection with no partial authority or
   Release visibility;
3. migration or explicit non-equivalence for current labels, CLI `release`
   check profile, proposals, waivers/deviations, Part lifecycle, library review,
   ZoneFill staleness, title-block claims, and GUI technical revision display;
4. independent multi-CI issuance and unchanged-revision reuse;
5. immutable released bytes and standing under successor work;
6. complete/unknown dependency witnesses, explicit library uptake,
   identity-aligned comparison, and topological regeneration;
7. byte-identical reproduction plus mismatch, unavailable producer/environment,
   nondeterminism, and authenticity-separation failures;
8. absent Git, Git mapping/ref divergence, mirror isolation, air-gap exchange,
   quarantine, semantic re-entry, replay/destination, and trust/authorization
   distinctions;
9. all six approved visual behaviors, responsive states, keyboard/accessibility
   semantics, and presentation-only hiding; and
10. operation/CLI/MCP parity, one-writer fences, source health, dependency
    authority, spec parity, evidence traceability, and production acceptance.

No proof-plan wording authorizes execution. Each implementation slice requires
explicit Frontier placement and its stated authorization.

## 15. Deferred authority

- Global Preferences mechanics and UI: `dat-global-preferences-engine-qcv`.
- Simultaneous distributed semantic merge:
  `dat-distributed-collaboration-architecture-lt1`.
- Verified Publish redaction/package variants:
  `dat-publish-redaction-contract-wzs`.
- Exact cryptographic/signature/dependency choices: future numbered decisions.
- Publish Space implementation: blocked until Revision Engine production
  acceptance.

<!-- EVIDENCE:PRODUCT-REVISION-SPEC:REV-C07-APPROVED -->
REV-C07 owner ratification was recorded on 2026-08-25 after the corrected
design-first invariant and first-class ignore workflow were incorporated. This
specification is the normative translation of that approved packet and adds no
new mechanism.
