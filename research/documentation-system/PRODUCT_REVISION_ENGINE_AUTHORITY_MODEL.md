# Product Revision Engine Authority Model

> **REV-C03 working model — not ratified.** This document defines the candidate
> vocabulary, objects, identities, relationships, lifecycles, roles, operations,
> invariants, refusals, and queries required by REV-C03. It is constrained by
> the owner-approved REV-C01 baseline and the REV-C02 standards matrix. It does
> not authorize implementation or settle the later Git, impact-engine, UX,
> dependency, or standards-profile decisions.

## Design result

Datum should use one revision substrate with two levels of ceremony:

1. **Technical history is automatic.** Every authored mutation continues to
   produce stable object revisions, one model revision, and an append-only
   transaction record through `commit()`.
2. **Engineering issue is deliberate.** A revision label, baseline, document
   issue, approval, or release exists only because an authorized typed
   operation establishes it over exact immutable inputs.

This keeps ordinary design fast. A user can move, route, edit, undo, and test
without filling out an ECO for every transaction. Once a governed configuration
is issued, later work accumulates as successor work and cannot overwrite or be
released under the prior issue identity. Profiles decide which formal change,
role, approval, effectivity, audit, and retention controls are mandatory.

## Evidence that shapes the model

### Datum evidence

- `ObjectRevision`, `ModelRevision`, guarded typed operations, the journal,
  exact library pins, proposal records, generated-evidence hashes, check
  dispositions, and ZoneFill staleness are implemented technical substrate;
  none is a product release.
- Decision 001 requires all authored changes to converge on `commit()` and the
  journal. Revision-engine operations must therefore extend that path rather
  than add a PLM side door.
- Decision 000D's unimplemented transaction-tip and journal-record claims must
  be reconciled explicitly; a Product Revision Engine must not conceal those
  technical gaps behind a new product revision label.
- Publish Space owns composition, render/export, and staleness presentation.
  The Revision Engine owns the `ConfigurationRef`, baselines, changes,
  approvals, issue allocation, release, effectivity, status, and audit.
- A Datum Project is a scalable owner-selected container. Configuration items,
  products, assemblies, release scopes, and packages are optional authored
  organization, never a mandatory project topology.

### Standards evidence

The clause-addressable controls are in
`PRODUCT_REVISION_ENGINE_STANDARDS_MATRIX.md`. They require a model capable of
unique controlled identity, exact baselines, authorized and impact-assessed
change, applicability/effectivity, status accounting, verification, immutable
released information, approvals, records, delivery, and retention. The matrix
also proves these obligations are profile- and contract-dependent.

### External product precedents

- [SOLIDWORKS PDM](https://help.solidworks.com/2024/English/EnterprisePDM/FileExplorer/c_Versions_and_Revisions.htm)
  creates a version for each check-in but advances revision only at a meaningful
  workflow state. This supports Datum's transaction-versus-issued-revision
  separation.
- [Autodesk Vault](https://help.autodesk.com/cloudhelp/2026/ENU/Vault-Have-You-Tried/files/GUID-A225776F-F9E8-43B7-911F-16800E34E07E.htm)
  separates versions, lifecycle states, and revision milestones, and preserves
  released versions. Datum should preserve those concepts but avoid making a
  centralized vault mandatory.
- [Aras Express Change Management](https://www.aras.com/community/documentationlibrary/Innovator/35/Content/Innovator%2024%20Docs/Aras%20PE%2014%20-%20User%27s%20Guide/Express%20Change%20Management.htm)
  separates the change object and its roles/workflow from the affected Parts or
  Documents, locks the old released revision, and releases a new revision after
  review. Its configurable simple/full workflows validate profile-scaled
  ceremony, while its fixed enterprise roles are not adopted as Datum core.

These are behavioral references, not dependencies or authorities.

## Canonical vocabulary

| Term | Exact Datum meaning | Explicitly not |
|---|---|---|
| `Technical revision` | Immutable identity of a committed object/model state used for concurrency, replay, diff, provenance, and staleness. | An issued engineering revision, approval, or release. |
| `Transaction` | One append-only record of an authored operation batch committed through the canonical path. | An ECO, release, or revision-table row. |
| `Controlled subject` | Existing Datum authority selected for configuration control: project, design object/aggregate, library object, ruleset, variant, Publish object, manufacturing plan, output job, generator, or other registered kind. | A copied PLM shadow object. |
| `Configuration Item` (`CI`) | Stable designation that places one controlled subject or explicit aggregate under defined change authority and policy. | A mandatory decomposition of every Project. |
| `Working configuration` | Resolver view of current authored state at an exact model revision and transaction tip. | A baseline or mutable object stored as competing design truth. |
| `Engineering revision` | Human-governed issue identity allocated to an approved configuration of one revision-bearing CI. | A transaction count, Git commit, filename suffix, mutable counter, or release-scope identity. |
| `Configuration baseline` | Immutable manifest of exact controlled-subject revisions and qualifying context established by authority at one point. | A branch, tag, workspace, or alias for `ModelRevision`. |
| `Engineering change` | Governed intent, impact, authorization, implementation, and verification record relating predecessor and successor controlled configurations. | The operation batch itself or an automatically approved proposal. |
| `Successor work` | Mutable technical work based on a released/baselined predecessor and destined for a possible successor issue. | A mutation of the predecessor release. |
| `Approval attestation` | Immutable actor/role/intent/time/policy statement over an exact target digest. | An editable boolean, free-text name, or implicit trust from login/Git. |
| `Release candidate` | Mutable, invalidation-aware assembly of proposed baseline, evidence, approvals, document issues, packages, effectivity, and profile evaluation. | A released object or authority after its inputs change. |
| `Release` | Atomic authorized event establishing an immutable baseline and its issued engineering/document/package identities with qualifying evidence. | Passing the CLI check profile currently named `release`. |
| `Controlled document` | Stable identity for authored publication meaning governed by revision policy. | A Sheet, PDF byte stream, or title-block field alone. |
| `Document issue` | Immutable issued revision of a ControlledDocument resolved from an exact configuration and rendering/generator context. | The editable Publish source or arbitrary export. |
| `Release package` | Immutable named manifest of exact issued documents/artifacts selected for a release purpose. | A `PublishSet` body or a recipient delivery event. |
| `Transmittal` | Immutable record that an exact ReleasePackage was delivered to identified recipients under a delivery/egress policy. | The package itself or proof that the recipient accepted it. |
| `Effectivity` | Typed expression and resolved population to which a change, revision, departure, or release applies. | Unvalidated free text. |
| `Status accounting` | Historical/current projection derived from authoritative events and immutable records. | A second mutable lifecycle database. |
| `Audit evaluation` | Immutable evaluation of an edition-pinned requirements profile against a frozen evidence set. | Certification or a prose-only report. |

## Identity system

Every governed record has four non-interchangeable identity facets:

| Facet | Purpose | Rule |
|---|---|---|
| Stable ID | Machine reference across rename, reordering, and storage movement. | UUID-based Datum identity; never derived from label/path/revision. |
| Human number/name | Search, drawing, package, and organization vocabulary. | Scoped, uniqueness-policy controlled, rename rules explicit. |
| Issue/revision label | Human milestone label interpreted only through its assigned revision scheme. | Allocated only by policy; unique within its revision namespace; never parsed as technical ordering without the scheme. Alphabetic and numeric labels elsewhere in this working model are illustrations, not defaults. |
| Content/record digest | Exact integrity and independent verification. | Deterministic digest over the canonical immutable payload and referenced identities. |

`ObjectRevision` and `ModelRevision` remain existing technical identity types.
The following new IDs are candidates: `ConfigurationItemId`, `BaselineId`,
`EngineeringChangeId`, `EngineeringRevisionId`, `ApprovalAttestationId`,
`ReleaseCandidateId`, `ReleaseId`, `ControlledDocumentId`, `DocumentIssueId`,
`ReleasePackageId`, `TransmittalId`, `EffectivityId`, `StandardsProfileId`, and
`AuditEvaluationId`.

The type system must reject substituting one ID class for another even when all
use the same UUID wire representation.

## Authority objects and relationships

### `ConfigurationItem`

Designates an existing controlled subject without copying it.

```text
ConfigurationItem {
  id, human_number, name,
  subject: AuthorityRef,
  kind, parent_ci?, relationships[],
  control_policy_ref,
  revision_scheme_ref?,
  owning_authority,
  designation_event,
  retirement_event?
}
```

An aggregate CI owns only membership/relationship meaning; its member data
remains in the original Datum authorities. A subject can be controlled without
being assigned to a product/assembly tree. CI parentage is optional and cannot
be inferred from Project filesystem layout.

### `ConfigurationRef`

The existing Publish seam becomes a revision-engine tagged union:

```text
ConfigurationRef =
  Working { model_revision, accepted_transaction_tip }
  | Baseline { baseline_id, baseline_digest }
```

Both variants resolve through one engine interface. `Working` is exact at query
time but not immutable for later reuse unless its values are retained.
`Baseline` resolves only the frozen manifest. Git identity is optional adapter
metadata and is never a union variant.

### `ConfigurationBaseline`

```text
ConfigurationBaseline {
  id, human_number?, baseline_type,
  scope: ReleaseScope,
  members: sorted Vec<BaselineMember>,
  source_model_revision,
  accepted_transaction_tip,
  governing_changes[],
  departures[],
  standards_profile_refs[],
  establishment_attestations[],
  established_by, established_at,
  predecessor_baselines[],
  digest
}

BaselineMember {
  authority_ref,
  exact_technical_revision,
  source_shard_digest?,
  role,
  inclusion_reason
}
```

The baseline is immutable when established. “Current,” “superseded,”
“withdrawn,” or “historical” are status projections from later relationship
events; no operation rewrites the manifest. Generated evidence is not silently
folded into `ModelRevision`; exact release evidence is referenced separately by
the Release.

### `EngineeringRevision`

```text
EngineeringRevision {
  id,
  configuration_item_id,
  revision_namespace,
  revision_scheme_ref,
  revision_label,
  baseline_id,
  predecessor_revision?,
  governing_change_ids[],
  allocated_by_policy,
  issued_by_release_id
}
```

An EngineeringRevision cannot exist in issued form without a baseline and
Release. No revision scheme or default label sequence is ratified by this
working model. A profile may reserve a scheme-governed label during successor
work, but reservation is a separate expiring `RevisionReservation`, not an
issued revision and never appears as released title-block truth. Merely
collecting successor work binds no revision identity or reservation.

The revision namespace is owned by exactly one revision-bearing
`ConfigurationItem`. That CI may designate one controlled subject or an
explicitly authored aggregate Product/System; aggregate membership never rolls
member revisions into a coincident product label. An arbitrary `ReleaseScope`
cannot own a competing EngineeringRevision. Exact multi-CI composition remains
the authority of `ConfigurationBaseline`, an optional `BuildIdentity` labels
that baseline, and `Release` coordinates issuance.

### `EngineeringChange`

One scalable object supports quiet and regulated profiles rather than separate
ECR/ECO architectures:

```text
EngineeringChange {
  id, human_number?, title, rationale,
  source, change_class?, urgency?,
  predecessor_baselines[],
  affected_items: Vec<AffectedItem>,
  impact_evaluations[],
  effectivity_ref?,
  implementation_plan?,
  transaction_links[],
  verification_evidence[],
  lifecycle_events[],
  approval_policy_ref,
  attestations[]
}

AffectedItem {
  ci_or_authority_ref,
  action: Add | Modify | Retire | NoChange,
  before_ref?, proposed_after_ref?, released_after_ref?,
  impact_disposition,
  effectivity_override?
}
```

Profiles may present this as Change, ECR/ECO, DCO, deviation workflow, or an
organization term, but the engine keeps one semantic record. A lightweight
profile may create a background `EngineeringChange` only when work diverges
from a released baseline, auto-collect linked transactions, and ask for
rationale/impact only at release preparation. A regulated profile may require
explicit initiation and authorization before implementation. Neither path
lets an unreviewed transaction become an issued revision.

### `ApprovalAttestation` and policy

```text
ApprovalAttestation {
  id,
  target: GovernedRecordRef,
  target_digest,
  actor_identity,
  role_assignment_ref,
  intent: Approve | Reject | Acknowledge | Verify | Authorize | Release,
  policy_ref,
  method,
  asserted_at,
  credential_evidence?,
  supersedes_attestation?
}
```

Attestations are append-only. A mistaken or compromised approval is corrected
by a typed superseding/revocation event; a released record is never edited to
erase history. `ApprovalPolicy` defines required intents, roles, quorum/order,
separation of duty, credential method, and scope. Role names are profile data,
not hard-coded enterprise hierarchy.

### `Effectivity`

```text
Effectivity {
  id,
  expression: typed predicate tree,
  selector_kinds,
  resolved_population_snapshot?,
  resolution_context,
  valid_from?, valid_until?,
  authority,
  digest
}
```

Initial selector families can include product/assembly identity, CI, variant,
board/subassembly, serial range, lot/batch, build, customer, site, contract,
and date. Unsupported selectors refuse; unknown does not mean “all.” A release
records both the authored expression and, when enumerable, the population
resolved under its exact context.

### Documents, packages, and delivery

```text
ControlledDocument {
  id, document_number, name, document_kind,
  source_ref, revision_scheme_ref,
  owning_authority, title_block_definition_ref?
}

DocumentIssue {
  id, controlled_document_id, engineering_revision_id,
  configuration_ref: Baseline,
  source_revision_refs[], render_context,
  output_artifact_refs[], approval_attestations[],
  issued_by_release_id, digest
}

ReleasePackage {
  id, package_number?, name, purpose,
  release_id, ordered_member_refs[],
  package_metadata, manifest_digest
}

Transmittal {
  id, transmittal_number?, release_package_id,
  exact_package_digest, sender, recipients[],
  delivery_policy_ref, classification_egress_context?,
  sent_at, delivery_evidence[], receipt_evidence[]
}
```

Publish `Sheet`, `ViewportDefinition`, `ViewportInstance`, and `PublishSet`
remain Publish objects. A `ControlledDocument` references their authored
publication meaning; a `DocumentIssue` records an exact issued result. One
Sheet or PublishSet may feed multiple controlled documents only through
explicit definitions; no title-block text manufactures identity. A Package
can be retransmitted without creating a new issue if its exact bytes and
policy remain unchanged; the new Transmittal records the new delivery event.

Exactly one revision-bearing CI designates each revision-controlled
`ControlledDocument`. Its `DocumentIssue.engineering_revision_id` references
that CI's issued `EngineeringRevision`; an issue has no independent revision
counter. Renaming, reordering, or otherwise editing the source PublishSet can
affect a future issue and staleness analysis but cannot mutate an existing
DocumentIssue. Deleting a bound Sheet or PublishSet refuses until every
ControlledDocument dependency is explicitly removed or retargeted.

### `ReleaseCandidate` and `Release`

```text
ReleaseCandidate {
  id, scope, proposed_revision_allocations[],
  working_configuration_ref,
  proposed_baseline_manifest,
  governing_changes[], departures[], effectivity_refs[],
  required_evidence_plan,
  evidence_refs[], proposed_document_issues[],
  proposed_packages[], approval_policy_ref,
  attestations[], standards_evaluations[],
  readiness_findings[], last_evaluated_context
}

Release {
  id, release_number?, released_at,
  release_authority_attestation,
  baseline_id, engineering_revision_ids[],
  document_issue_ids[], package_ids[],
  governing_changes[], departures[], effectivity_refs[],
  qualifying_evidence_refs[], audit_evaluation_refs[],
  policy_evaluation_digest,
  engine_version, record_digest
}
```

The candidate is mutable sidecar authority guarded by model revision and
journal tip. Any covered source/evidence/policy change marks its readiness
evaluation stale and invalidates attestations whose target digest changed.
`Release` is created atomically with the immutable baseline and issued records
only after every refusal is clear. Release never writes or freezes the working
DesignModel; successor work continues from the released predecessor while the
baseline remains independently resolvable.

### Standards and records objects

```text
StandardsProfile {
  id, name, version,
  authority_editions[], applicability,
  requirement_dispositions[], organization_policy_refs[],
  approval, effective_interval, digest
}

RequirementDisposition {
  requirement_id, authority, edition, clause,
  disposition: Applicable | Tailored | EquivalentControl |
               NotApplicable | Unresolved | BlockedNormativeText,
  rationale, control_ref?, evidence_rule?, verification_rule?,
  approval_attestation?
}

AuditEvaluation {
  id, profile_ref, scope, frozen_evidence_set,
  engine_version, evaluated_at,
  requirement_results[], findings[], digest
}

RetentionPolicy {
  id, record_classes[], triggers[], periods[], holds[],
  disposition_authority, policy_approval, effective_interval
}

RecordDisposition {
  id, record_ref, policy_ref, action,
  due_at, hold_resolution?, authorization, evidence
}
```

Standards text is not embedded as mutable UI prose. Profiles refer to
edition-pinned clause identities and licensed-text access status. Audit
evaluation reports satisfaction of the configured matrix, never certification.

## Lifecycle model

Lifecycle state is a projection of append-only typed events. The current state
may be indexed for speed, but the event chain remains authority.

### EngineeringChange lifecycle

```text
Draft
  -> ImpactReview
  -> Authorized
  -> Implementing
  -> Verification
  -> Closed
```

Side exits are `Rejected`, `Deferred`, and `Cancelled`. Rework returns through
a typed event to the appropriate earlier state; it never deletes a decision.
Profiles may collapse stages (for example Draft directly to Authorized) only
through an explicit policy transition. Authorization does not prove
implementation; verification does not issue the release; closure requires the
released successor or an explicit no-release disposition.

### ReleaseCandidate lifecycle

```text
Preparing -> ReadyForReview -> InApproval -> ReadyToRelease
```

Any changed covered digest projects `Stale`, from which the candidate returns
to `Preparing` after reevaluation. `Rejected` and `Cancelled` are terminal
candidate dispositions. Successful release creates a separate immutable
`Release`; it does not mutate the candidate into the release authority.

### Issued-record status

Baseline, EngineeringRevision, DocumentIssue, Release, and ReleasePackage are
immutable records, not editable lifecycle objects. Their current standing is
derived from later events/relationships:

```text
Current | Superseded | Withdrawn | Obsolete | Historical
```

These words do not rewrite the record and do not all apply to every kind.
Withdrawal does not erase prior effectivity or transmittal history.

## Role and authority model

`ActorIdentity` identifies a person, service, organization, or controlled
automation principal. `RoleAssignment` binds that actor to a role, scope,
authority source, and effective interval.

Core capabilities—not mandatory job titles—are:

| Capability | Permitted authority |
|---|---|
| Author | Prepare technical work and change content. |
| Change coordinator | Maintain change scope, impact, assignments, and progression. |
| Reviewer | Record review findings or acknowledgement. |
| Verifier | Attest that implementation/evidence meets specified checks. |
| Configuration authority | Designate CIs, establish baselines, and disposition controlled change as policy permits. |
| Release authority | Perform the final release attestation and atomic release operation. |
| Auditor | Evaluate frozen evidence without gaining mutation/release authority. |
| Records authority | Approve holds and record disposition under retention policy. |

One actor may hold several capabilities in a lightweight profile. Regulated
profiles can require separation, quorum, order, independence, or customer
authority. The core engine evaluates capabilities and scope, never display
role strings.

## Typed operation catalog

All authored revision-engine changes use typed operations and `commit()`.
Immutable records are created atomically; no generic JSON patch is public.

### Control and profile

- `DesignateConfigurationItem`
- `UpdateConfigurationItemPolicy`
- `RetireConfigurationItemDesignation`
- `CreateStandardsProfile`
- `AddRequirementDisposition`
- `ApproveStandardsProfileVersion`
- `CreateOrUpdateRetentionPolicy`
- `AssignScopedRole` / `RevokeScopedRole`

### Change

- `CreateEngineeringChange`
- `SetChangeRationaleAndClassification`
- `AddOrUpdateAffectedItem`
- `RecordImpactEvaluation`
- `SetChangeEffectivity`
- `LinkImplementationTransaction`
- `SubmitChangeForReview`
- `AuthorizeChange` / `RejectChange` / `DeferChange`
- `BeginChangeImplementation`
- `RecordImplementationVerification`
- `CloseChange` / `CancelChange`

### Approval and baseline

- `RequestApproval`
- `RecordApprovalAttestation`
- `SupersedeOrRevokeAttestation`
- `PrepareBaselineManifest`
- `EstablishConfigurationBaseline`
- `RecordBaselineRelationship`
- `ReserveRevisionLabel` / `ReleaseRevisionReservation`

### Release, document, package, and delivery

- `CreateReleaseCandidate`
- `RefreshReleaseCandidateEvaluation`
- `AddCandidateEvidence`
- `DefineControlledDocument`
- `PrepareDocumentIssue`
- `PrepareReleasePackage`
- `ReleaseConfiguration`
- `WithdrawReleaseStanding`
- `RecordTransmittal`
- `RecordDeliveryReceipt`

### Audit and records

- `RunStandardsAuditEvaluation`
- `RecordAuditFindingDisposition`
- `PlaceLegalOrPolicyHold`
- `AuthorizeRecordDisposition`
- `RecordDispositionEvidence`

Bulk/template commands expand to these operations and commit atomically. A GUI,
CLI, MCP client, script, importer, or future Git adapter receives the same
authorization and refusal behavior.

## Invariants

1. **One mutation path.** No revision-engine source record changes outside
   typed operations and `commit()`.
2. **No identity collapse.** Technical revisions, transactions, engineering
   revisions, baselines, document issues, releases, packages, and transmittals
   are distinct typed identities.
3. **No released mutation.** Immutable issued records and their covered bytes
   are never edited; correction creates a successor or standing event.
4. **Exact baseline resolution.** Every baseline member resolves to an exact
   registered authority and technical revision or establishment refuses.
5. **No floating dependencies in release.** A release cannot contain “latest,”
   unpinned library bindings, unresolved generated inputs, or mutable paths.
6. **Approval covers bytes/meaning.** Every attestation targets a canonical
   digest; a changed target cannot retain the prior approval.
7. **Authority is scoped and effective.** An actor must hold the required
   capability for the target scope and time; display names confer nothing.
8. **Issued label uniqueness.** A revision/document/release label is unique in
   its scheme namespace and cannot be reused after cancellation or withdrawal
   unless the governing scheme explicitly permits an auditable reservation
   release before issue.
9. **Change accounting is complete.** Released successor differences trace to
   governing changes or explicit accepted administrative dispositions.
10. **Effectivity is resolvable.** Unknown/ambiguous applicability cannot be
    coerced to universal applicability.
11. **Evidence is frozen.** Release and audit cite immutable evidence identity,
    producer/engine version, configuration context, and integrity digest.
12. **Departure is bounded.** Waiver/deviation scope, authority, effectivity,
    validity, and affected requirement/finding are explicit.
13. **Profile truth is edition-pinned.** Re-evaluation under a new standard or
    organization profile creates a new audit context; history is not rewritten.
14. **Workspace is excluded.** Pane, camera, selection, terminal, and layout
    state never enter a product baseline unless a future separately governed
    record class explicitly requires it.
15. **Git is non-authoritative.** No commit, branch, or tag creates Datum
    approval, revision, baseline, or release without the engine operation.

## Required typed refusals

| Refusal | Trigger |
|---|---|
| `StaleWorkingConfiguration` | Expected model revision or accepted transaction tip differs. |
| `UnresolvableBaselineMember` | A manifest reference/revision cannot resolve exactly. |
| `FloatingReleaseDependency` | Candidate contains latest/unpinned/mutable input. |
| `ReleasedRecordImmutable` | Any operation attempts to edit an issued record. |
| `DuplicateIssuedLabel` | Revision/release/document label already exists in namespace. |
| `RevisionReservationConflict` | Active reservation conflicts or policy forbids reservation. |
| `UnaccountedControlledDifference` | Candidate differs from predecessor without governing disposition. |
| `ImpactAnalysisIncomplete` | Required affected subjects/consumers lack dispositions. |
| `ChangeNotAuthorized` | Profile requires authorization before linked implementation/release. |
| `ImplementationNotVerified` | Required verification is missing/stale/failed. |
| `ApprovalTargetChanged` | Attested target digest differs from current candidate digest. |
| `ApprovalPolicyUnsatisfied` | Missing role, order, quorum, independence, method, or authority. |
| `EffectivityUnresolved` | Selector invalid, ambiguous, unsupported, or population resolution failed. |
| `DepartureInvalidOrExpired` | Waiver/deviation lacks authority/scope or is outside validity. |
| `RequiredEvidenceMissing` | Check, artifact, document, audit, signature, or reproduction proof absent. |
| `EvidenceStale` | Evidence configuration/generator/policy context differs from candidate. |
| `StandardsDispositionUnresolved` | Applicable release policy contains unresolved/blocked requirement. |
| `RetentionOrHoldViolation` | Requested disposition conflicts with schedule or active hold. |
| `EgressPolicyViolation` | Package/transmittal recipient or handling violates policy. |
| `NonReproducibleRelease` | Baseline, generator, environment, or output digest cannot be reproduced/verified as required. |
| `ExternalAdapterUnavailable` | Only when the active policy explicitly requires adapter synchronization/signing. |

Every refusal returns stable code, affected IDs, controlling policy/requirement,
current versus expected identity, and actionable remediation. The GUI may
narrate it, but prose is never the only machine interface.

## Query contract

The engine must answer without reconstructing meaning from filenames or Git:

- `configuration resolve --working|--baseline <id>`
- `configuration compare <ref-a> <ref-b>`
- `configuration why-member <baseline> <authority-ref>`
- `configuration as-of <event-or-time>`
- `change show|list|status|affected|transactions|evidence <id>`
- `change impact <id> --explain-paths`
- `revision history <ci-or-document>`
- `revision resolve-label <namespace> <label>`
- `baseline show|members|predecessors|successors|verify <id>`
- `release readiness <candidate> --profile <id>`
- `release show|verify|reproduce|compare <id>`
- `approval required|recorded|valid <target>`
- `effectivity resolve <id> --context <configuration>`
- `document issues <document>` and `document reproduce <issue>`
- `package members|verify <id>`
- `transmittal show|recipients|delivery-evidence <id>`
- `status configuration|changes|approvals|departures|releases --as-of ...`
- `audit evaluate|show|verify <id>`
- `records retention-status|holds|due|disposition-history`

Each query emits typed JSON through engine/CLI/MCP parity and may also provide a
human view. Historical queries state the event/time authority and completeness;
an unavailable clock/signature source is explicit.

## Reconciliation of REV-C01 contradictions

| Gap | REV-C03 disposition |
|---|---|
| REV-GAP-01 | Preserve model revision and accepted transaction tip as a required pair in `ConfigurationRef::Working`. Whether the tip is also folded into `ModelRevision` remains a technical migration decision; product authority never assumes it is today. |
| REV-GAP-02 | Transaction parent/shard hash/validation/cache fields remain technical-journal debt. Product change/baseline/release records reference transactions but do not duplicate or pretend to supply missing crash/replay fields. |
| REV-GAP-03/10 | Release owns a separate immutable manifest over baseline plus qualifying evidence/artifact identities; it is not `ModelRevision`. |
| REV-GAP-04 | Product issue operations are transaction-produced, but technical resolver identity behavior must be reconciled independently rather than restating “all revisions” imprecisely. |
| REV-GAP-05/11 | ControlledDocument/DocumentIssue/Release replace the false journal-derived revision-table and nonexistent PLM/ECO claims. The title-block research requires correction before ratification. |
| REV-GAP-06 | GUI `rev` may display only a typed technical or engineering revision with its class clear; visual wording is deferred to REV-C06. |
| REV-GAP-07 | The existing CLI `release` check profile is a readiness input, not `ReleaseConfiguration`; naming migration belongs in implementation planning. |
| REV-GAP-08 | Existing proposal acceptance, review booleans, and check dispositions cannot satisfy `ApprovalPolicy` without explicit migration/equivalent-control disposition. |
| REV-GAP-09 | Free-form `rev-*`/`release-*` strings remain labels only. |
| REV-GAP-12 | General dependency/cached-derived-state design is deferred to REV-C05; ZoneFill remains the bounded implemented precedent. |

## Deliberately deferred seams

- **REV-C04:** local record persistence/recovery, Git mapping, external shard
  ingestion, signatures across remotes, concurrent/distributed exchange.
- **REV-C05:** complete dependency graph, affected/unaffected proof, staleness,
  library uptake, regeneration, reproduction environment.
- **REV-C06:** quiet-default UX, role/profile setup, candidate/readiness flow,
  title-block/status projections, accessibility, and visual source of truth.
- **REV-C07:** owner ratification of names, mechanisms, standards claims,
  dependency posture, and visual contract.

## Questions requiring explicit owner disposition before REV-C07

These are not approval requests yet; each needs a proof-backed decision packet:

1. Should Datum expose one user-facing `Change` object with profile-specific
   labels/stages, or distinct ECR/ECO/DCO object types that share a substrate?
   This model recommends one semantic object to avoid forcing enterprise method.
2. Should a lightweight project automatically open background successor Change
   work after the working configuration first diverges from a released
   baseline, or require an explicit “Begin revision” action? This question is
   blocked on the revision-identity prerequisite: collection must not imply,
   reserve, or allocate a revision identity.
3. Is `EngineeringRevision` allocated per CI, per explicit release scope, or
   both under separate namespaces? Resolved below as per revision-bearing CI
   only, including an optional explicitly authored aggregate Product/System CI.
4. May a Release contain several independently numbered engineering revisions
   coordinated by one baseline/release event? Resolved below as yes, while
   unchanged baseline members retain their previously issued revisions.
5. Is `ControlledDocument` the correct stable authority between editable
   Publish sources and immutable DocumentIssues, or should PublishSet directly
   carry document identity? Resolved below as V9-A: the stable authority remains
   separate and PublishSet never becomes a release state machine.
6. Should withdrawn/obsolete/superseded be universal standing events or only
   profile-enabled vocabulary? This model recommends universal typed standing
   relationships with profile-selected allowed terms and transition rules.

## Owner disposition ledger

### REV-C03-Q1 — EngineeringChange identity and lifecycle

**Approved 2026-08-24.** The reviewed authority is `EngineeringChange`, lines
213–250, and `EngineeringChange lifecycle`, lines 414–431, as frozen in commit
`f473ffe`.

Datum has one engine-level `EngineeringChange` identity with the append-only
lifecycle `Draft -> ImpactReview -> Authorized -> Implementing -> Verification
-> Closed` and the `Rejected`, `Deferred`, and `Cancelled` exits. Profiles may
label stages as ECR, ECO, DCO, or local terms without creating competing engine
objects. Waivers and deviations remain distinct bounded-departure records.

<!-- EVIDENCE:PRODUCT-REVISION-SPEC:REV-C03-Q1-APPROVED -->

### REV-C03-Q2 prerequisite history — completed

**Owner revision 2026-08-24.** Q2 is not a choreography choice until Datum's
revision-identity scheme and complex-product composition rules are ratified.
Automatic successor-work collection is acceptable only when fully decoupled
from presumed next-in-sequence identity. Q2 and Q3 were deferred pending a
dedicated evidence-backed decision packet and concrete owner-reviewed title-
block and UI renderings; all six identity dispositions are now recorded below.

The sequencing/origin synthesis is recorded in
`REVISION_SEQUENCING_AND_ORIGIN_RESEARCH.md`; it supplies candidate evidence,
not a standards-profile mapping or owner disposition. Its strongest normative
claims remain subject to the REV-C02 source-strength and licensed-text gates.

The prerequisite packet must decide:

1. a profile-policy scheme registry covering linear alphabetic, linear numeric,
   ISO 19650 status plus revision, and organization-custom behavior, with no
   unratified default;
2. per-CI revision namespaces plus baseline-manifest identity as the primary
   complex-product composition mechanism, with product-level labels optional;
3. that successor-work collection binds no revision identity and allocation is
   a scheme-governed designer/release-time action.

The visual decision packet must additionally resolve human-identity origin,
enabled minting-event kinds, optional phase-build identity, suitability/status
exposure, and prototype-to-production sequence transition before Q2 or Q3.

<!-- EVIDENCE:PRODUCT-REVISION-SPEC:REV-C03-Q2-PREREQUISITE -->

### REV-C03-ID-Q1 — Revision profile registry and factory preference

**Approved 2026-08-24.** Datum retains one Product Revision Engine with a
profile-policy registry; it does not implement competing revision engines. The
v1 registry exposes the researched sequential numeric, sequential alphabetic,
issue/revision, ISO 19650-oriented, and organization-custom behaviors. The
factory system preference is `Sequential Alphanumeric (Legacy)`. ISO 19650 is a
selectable professional profile, and the registry remains extensible to later
standards without changing engine identity.

The Global Preferences engine is not implemented. Until its separately tracked
specification and implementation land, Revision code receives an explicit
resolved policy in tests and bounded execution. A future system preference
seeds new Project policy only; that Project-owned copy is the governed authority.
Changing a system preference cannot silently alter an existing Project, a CI
namespace, or any issued identity. Alphabetic/numeric token configuration and
human-identity origin remain scheme policy; this disposition allocates no token
and does not answer REV-C03-ID-Q2.

**Follow-on:** `dat-global-preferences-engine-qcv`, blocked on completion of the
Product Revision Engine specification.

<!-- EVIDENCE:PRODUCT-REVISION-SPEC:REV-C03-ID-Q1-APPROVED -->

### REV-C03-ID-Q2 — Human-identity origin before issue

**Approved 2026-08-24.** The factory `Sequential Alphanumeric (Legacy)` profile
uses the visually reviewed V2 Candidate P treatment: before issue, a governed
Publish drawing may carry an honestly marked provisional `RevisionReservation`
rather than presenting the vague `UNALLOCATED` state as its primary human-facing
revision communication. The reservation is visibly provisional and not issued.
It is not an `EngineeringRevision`, baseline, approval, or Release, and it can
never project as released title-block truth. Technical transaction history
continues independently throughout the provisional state. Only a later
profile-authorized boundary can create an issued `EngineeringRevision`.

The ASME initial-release dash remains a separate standards-profile behavior,
not a provisional-mark synonym and not part of this approval. Its exact meaning
remains subject to the licensed-clause review already recorded by the evidence
packet.

A future Global Preferences presentation setting may let the user project a
provisional watermark across the entire drawing. The setting's default, allowed
wording, template interaction, print/export behavior, and whether a governed
profile may require it are deliberately deferred to
`dat-global-preferences-engine-qcv`; this approval does not silently invent that
unbuilt preference authority.

<!-- EVIDENCE:PRODUCT-REVISION-SPEC:REV-C03-ID-Q2-APPROVED -->

### REV-C03-ID-Q3 — Profile-owned issuance transitions

**Approved 2026-08-24.** Revision-identity minting policy belongs to the active
Revision Profile; Datum has no universal list of boundary events that mint
identities independently of profile policy. The future Global Preferences
selection seeds a new Project with a resolved profile, and the Project-owned
copy remains authoritative thereafter.

The factory `Sequential Alphanumeric (Legacy)` profile uses the approved V2-P
reservation treatment and converts the reserved sequential identity into an
issued identity only through an explicit `Issue/Release` operation. The
ISO 19650 profile maps its own WIP, Shared, and Published transitions without
conflating suitability/status with revision identity. Other registry profiles
may define different explicit transition maps without changing the engine.

Manufacturing, delivery, review, certification, baseline, and phase-build
events remain typed boundary context. They mint identity only when the resolved
profile explicitly maps that event to an issuance transition. Editing, saving,
exporting, elapsed time, transaction count, and retransmission never implicitly
mint identity. Retransmitting unchanged released material creates a
`Transmittal` against the existing Release. Phase-build naming and release-scope
identity remain undecided until REV-C03-ID-Q4.

The resolved policy input must therefore carry an editioned transition map from
typed boundary context to reservation/allocation/issuance behavior. Historical
identities retain the profile-policy version that interpreted their transition.

<!-- EVIDENCE:PRODUCT-REVISION-SPEC:REV-C03-ID-Q3-APPROVED -->

### REV-C03-ID-Q4 — Optional phase/build hierarchy presentation

**Approved 2026-08-24.** Datum supports an optional first-class `BuildIdentity`
over one immutable `ConfigurationBaseline` manifest. The factory presentation is
the visually reviewed V4a treatment: a quiet build-and-baseline Inspector with
no mandatory phase hierarchy. A future Global Preferences selection may instead
seed new Projects with the V4b enterprise Phase -> Build -> Baseline Navigator
hierarchy, and a governed enterprise profile may require that hierarchy. The
Project-owned copied policy remains authoritative thereafter.

V4a and V4b are presentations over one data model, not competing storage or
identity mechanisms. V4b reveals typed organization already present in the
same `BuildIdentity` relationships; enabling it does not migrate, duplicate, or
reinterpret baseline data. Projects that do not use build identities incur no
required hierarchy or visible workflow.

Phase, Build, baseline, and member revision/spin remain distinct typed
identities. A Build identity never becomes composition authority, forces member
labels to match, or causes automatic revision roll-up. Naming and successor
policy are governed by the resolved organization/profile policy, and both
Sequential Alphanumeric and ISO 19650 profiles may use the capability.

<!-- EVIDENCE:PRODUCT-REVISION-SPEC:REV-C03-ID-Q4-APPROVED -->

### REV-C03-ID-Q5 — Profile-owned suitability/status axis

**Approved 2026-08-25; owner clarification recorded the same day.** The visually
reviewed V6a behavior and V3d title-block projection are jointly authoritative:
suitability/status exposure belongs to profiles that define that axis. The
factory `Sequential Alphanumeric (Legacy)` profile does not expose ISO
suitability codes, while ordinary Datum lifecycle truth such as Provisional,
Not Issued, Issued, and Released remains visible and must not be mislabeled as
ISO status.

The ISO 19650 profile stores revision identity and suitability/status as
separate authoritative fields that may change independently. A formatter may
display both compactly, but the engine refuses any profile, operation, or
projection that conflates them. Its full title-block projection follows V3d:
`REVISION` and `STATUS` occupy separately labeled cells, with container,
namespace, and lifecycle state also remaining distinct. Organization-custom and
future standards profiles may define their own typed status policies.

Future Global Preferences seeds the selected Revision Profile for new Projects;
the copied Project policy remains authoritative. Interfaces omit a meaningless
empty suitability field when the active profile defines no such axis.

<!-- EVIDENCE:PRODUCT-REVISION-SPEC:REV-C03-ID-Q5-APPROVED -->

### REV-C03-ID-Q6 — Prototype-to-production identity transition

**Approved 2026-08-25.** Datum supports both visually reviewed V6b transition
policies. Candidate A is the factory behavior: prototype and production work
continue in one CI namespace and one revision sequence. Candidate B is an
enterprise/organization-profile option: an explicit governed transition closes
the prototype namespace, creates a separately governed production identity,
and records immutable supersession lineage between them.

Candidate B is never a counter reset inside one namespace. Closed namespaces
remain queryable, historical identities retain their original meanings, and no
transition may erase, reuse, or reinterpret an allocated token. Global
Preferences may seed the new-Project policy; the copied Project policy remains
authoritative. Selecting ISO 19650 does not itself force Candidate B because
ISO provisional/contractual sequencing and product prototype/production
re-identification are distinct concerns.

Every transition is an explicit typed, policy-authorized operation. No phase
change, release, label edit, or profile switch silently changes identity.

<!-- EVIDENCE:PRODUCT-REVISION-SPEC:REV-C03-ID-Q6-APPROVED -->

### REV-C03-Q2 — Automatic lightweight successor collection

**Approved 2026-08-25.** In the lightweight profile, the first committed
divergence from a released baseline atomically opens a quiet Draft
`EngineeringChange` and begins collecting successor transactions against the
exact predecessor baseline. Datum does not interrupt the designer with a
mandatory `Begin revision` action before allowing the technical work.

Automatic collection creates no `RevisionReservation`, `EngineeringRevision`,
baseline, approval, `Release`, or `DocumentIssue`. Its user-facing Change or
Inspector wording is `Successor work — revision identity not yet reserved`, not
the vague `UNALLOCATED` title-block treatment. When the designer later performs
explicit revision preparation, the selected profile may create the already
approved V2-P provisional reservation; only the profile's explicit issuance
transition creates an issued revision.

The designer may name, split, merge, or reassign the collected work while the
append-only transaction history and predecessor relationship remain intact.
Release requires complete governing-Change or accepted administrative-
disposition coverage for every successor difference. Governed profiles may
instead require an explicit authorized Change before mutation.

The current prototype V5 engine state remains semantically valid, but its
literal `UNALLOCATED` user wording requires Claude-owned visual reconciliation
to the approved text above before the visual contract closes.

<!-- EVIDENCE:PRODUCT-REVISION-SPEC:REV-C03-Q2-APPROVED -->

### REV-C03-Q3 — EngineeringRevision ownership

**Approved 2026-08-25.** Select visual study V8-A. An
`EngineeringRevision` is allocated only within the namespace of one
revision-bearing `ConfigurationItem`. A CI may designate one controlled subject
or an explicitly authored aggregate Product/System. Aggregate designation is
optional and never inferred from Project containment, baseline membership, or
matching member labels.

`ConfigurationBaseline` remains the exact composition authority. Optional
`BuildIdentity` labels a baseline without becoming a member revision, and
`Release` coordinates the atomic issuance event. Arbitrary `ReleaseScope`
cannot own a competing EngineeringRevision; this rejects V8-B's two
revision-shaped identities and V8-C's loss of independently issued member
revisions.

Profiles may decide which designated CIs require revision control and how their
labels are projected. They cannot redefine the owner kind, collapse a baseline
or build label into an EngineeringRevision, or create a scope-owned parallel
revision namespace. This is core identity architecture, not a Global
Preferences option.

<!-- EVIDENCE:PRODUCT-REVISION-SPEC:REV-C03-Q3-APPROVED -->

### REV-C03-Q4 — Coordinated multi-revision Release

**Approved 2026-08-25.** One `Release` may atomically issue one or several
independently numbered `EngineeringRevision` records belonging to different
revision-bearing `ConfigurationItem` records. Each revision retains its own CI
namespace, scheme, predecessor, and governing changes; labels are not required
to match.

The Release issues only affected successor revisions. An unchanged CI appearing
in the new `ConfigurationBaseline` reuses its already-issued revision and is not
silently reminted. The baseline records the complete exact composition,
including reused revisions, while the Release records the coordinated
authorization and issuance event. This permits a board, assembly, controlled
document, and optional aggregate Product CI to become effective together
without fragmenting one governed change across artificial release events.

<!-- EVIDENCE:PRODUCT-REVISION-SPEC:REV-C03-Q4-APPROVED -->

### REV-C03-Q5 — ControlledDocument authority

**Approved 2026-08-25.** Select visual study V9-A. `PublishSet` remains an
editable, user-named ordered selection of Sheet references. It never owns a
document number, EngineeringRevision, release state, immutable issue, package,
or delivery identity.

`ControlledDocument` is the separate stable authority for authored publication
meaning, document number, purpose, and control policy. A revision-bearing CI
designates it; the CI's `EngineeringRevision` owns the human revision identity.
Immutable `DocumentIssue` references that EngineeringRevision and freezes the
exact baseline, source revisions, render context, output bytes, approvals, and
digest. `ReleasePackage` selects exact issues and `Transmittal` records delivery
of exact package bytes.

One Sheet or PublishSet composition may explicitly source multiple
ControlledDocuments with distinct purposes, numbers, and policies without
duplicating the editable composition. Later source edits cannot mutate released
issues. The existing referenced-object law controls deletion: deleting a bound
Sheet or PublishSet refuses until dependent ControlledDocuments are explicitly
removed or retargeted. V9's contrary deletion wording and its new source-health
overflow are presentation defects tracked by
`dat-revision-v9-prototype-conformance-p7q`; neither changes V9-A.

<!-- EVIDENCE:PRODUCT-REVISION-SPEC:REV-C03-Q5-APPROVED -->
