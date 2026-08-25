# Datum Product Revision Engine Research

> **Status:** In progress — internal baseline approved and clause-addressable
> standards matrix drafted
> (`dat-product-revision-engine-k9f`). No object model, lifecycle, revision
> scheme, standards-conformance claim, or implementation is yet ratified.

## Why this research exists

Datum cannot reduce engineering revision control to Git commits, a title-block
counter, or a frozen PDF. A released electronic product is a governed
configuration spanning schematic and PCB authority, library definitions,
rules, variants, manufacturing plans, checks, waivers, Publish documents,
generated artifacts, approvals, and effectivity. A moved connector or changed
Footprint can invalidate the relationship between the physical product and its
released documentation even when files remain syntactically valid.

The owner requires revision integrity to be universal and obvious when action
is needed, while otherwise receding into the background. Every controlled
document must participate in revision governance. The simple Datum project must
remain simple; regulated aerospace, defense, medical, and other organizations
must be able to apply stronger profiles without a second product architecture.

Questions 15 and 16 in the workspace/documentation owner ledger are closed at
their authority boundary: this engine owns their detailed behavior. This
research must define the release and baseline concepts those questions assumed
without delaying completion of the remaining workspace questions.

## Controlling posture

- The Datum Product Revision Engine is the revision, configuration, change,
  approval, baseline, and release authority. It must remain coherent in local
  and offline operation without requiring a Git repository or remote service.
- The owner has ratified complete standalone authority: without Git, Datum must
  still support revision, baseline, approval, release, reproduction, status
  accounting, and audit rather than falling back to a reduced revision model.
- Git-compatible JSON is a persistence, history, exchange, and collaboration
  substrate reached through an optional Datum-owned adapter; Git is not by
  itself Datum's engineering-release authority.
- Every committed Datum mutation has technical revision and provenance.
- An issued engineering revision identifies an approved configuration, not a
  count of edits or Git commits.
- A released configuration is never silently rewritten by later Design,
  library, Publish, rule, or generator changes.
- A controlled Design Space or Publish Space change after release must become
  successor revision work; the changed configuration may not be released under
  the unchanged prior revision index. Allocation timing and affected-document
  propagation remain research questions rather than being inferred here.
- Any divergence from a released baseline must be visible, impact-analyzed,
  and traceable to a controlled change disposition.
- Title-block revision/status fields are projections of governed document and
  release authority, not editable claims and not raw commit metadata.
- Datum specifies configurable policy and evidence mechanisms; it must not
  imply third-party certification merely because a profile is selected.

## Standards-audit contract

The owner requires the Product Revision Engine to be fully auditable against
applicable industry standards. This is a stronger requirement than retaining a
generic activity log and a narrower claim than automatic certification.

For every enabled standards or organization profile, Datum must preserve a
machine-queryable and human-reviewable chain from:

1. the exact authority, edition, clause, and applicability decision;
2. through the Datum control, invariant, role, workflow, or required record;
3. to the exact baseline, change, approval, check, artifact, and release
   evidence that satisfied or failed it;
4. including justified non-applicable, tailored, equivalent-control, waiver,
   deviation, and unresolved states;
5. with immutable provenance sufficient for an independent auditor to
   reproduce the disposition without relying on mutable UI text.

Profiles must distinguish normative requirements from guidance, Datum product
policy, and organization-specific procedure. They must identify licensed or
otherwise unavailable normative text rather than silently paraphrasing it as
authority. An audit report must state the profile and editions evaluated,
scope and effectivity, evidence set, exceptions, unresolved findings, and the
Datum engine version that performed the evaluation. Passing a Datum audit
means the recorded evidence satisfies that configured requirements matrix; it
does not assert accreditation, regulator acceptance, or product certification.

## Initial internal audit

Datum already has useful substrate pieces:

- `model_revision`, stable object identity/revision, typed operations, the
  commit journal, and revision guards provide technical change identity.
- `docs/POOL_ARCHITECTURE.md` explicitly states that pool JSON is designed for
  Git while Git is not the product-level revision model; library approval and
  same-identity resolution remain Datum semantics.
- `docs/LIBRARY_ARCHITECTURE.md` requires placed component bindings to pin Part,
  Symbol, Package, Footprint, and PinPadMap revisions.
- decision 007 distinguishes model, workspace, artifact, and stale-projection
  revisions and separately forces distributed-collaboration research.
- decision 020 proposes live Draft views and model-revision-pinned released
  Sheets, but it does not define a complete product configuration baseline.
- `research/documentation-system/TITLE_BLOCK_AND_DOC_CONTROL_RESEARCH.md`
  already proposes controlled documents, drawing registers, release states,
  revision history, and transmittals.

The title-block research also contains a material conflation to correct: the
commit journal may supply change evidence, but a commit or operation batch must
not automatically become an issued document revision or revision-table row.
Configuration identification, change authority, approval, effectivity, and
release must mediate that transition.

## Preliminary vocabulary to test, not yet ratified

- **Technical revision:** immutable identity of a committed object/model/file
  state used for concurrency, provenance, replay, diff, and staleness.
- **Configuration Item (CI):** an owner/profile-designated product, asset,
  document, rule set, plan, or other unit placed under formal control.
- **Engineering change:** proposed and dispositioned intent describing why a
  controlled configuration changes, its affected identities, impact, checks,
  approvals, and effectivity.
- **Configuration baseline:** approved manifest of exact CI revisions and
  release evidence at a point in the product lifecycle.
- **Document revision:** controlled issue identity for a document or package;
  distinct from its technical edit history.
- **Release:** role-authorized act that freezes a baseline, resolves title-block
  facts, generates/verifies artifacts, and records issue/transmittal evidence.
- **Effectivity:** scope in which an approved change applies, such as product
  variant, serial/lot range, build, customer, site, or date.
- **Status accounting:** queryable current and historical state of baselines,
  changes, approvals, affected CIs, releases, and implementation.

The research must determine which concepts remain universal Datum primitives,
which are profile-enabled, and which terminology changes for lightweight use.

## Revision layers that must not collapse

1. Object and model technical revisions advance with committed mutations.
2. Git commits capture deterministic storage snapshots and collaborative
   history; optional signed tags may mirror important Datum releases.
3. Engineering changes govern movement away from an approved baseline.
4. Configuration baselines identify the exact product/document/artifact set.
5. Document/package revisions identify issued communication to a recipient.
6. Transmittals record what was delivered, to whom, when, and under what
   classification or egress policy.

## Datum core and Git adapter boundary

The revision engine is not implemented as a thin wrapper that delegates product
meaning to Git. Datum owns the semantic state machine and durable records for
technical revisions, engineering changes, CI membership, baselines, approvals,
document revisions, releases, and status accounting. A project without Git
must still be able to author, inspect, compare, approve, release, reproduce, and
audit its governed Datum configurations using local engine authority.

An optional `GitAdapter` may:

- materialize deterministic Datum shards into commits;
- associate a Datum technical revision or release with a commit identity;
- create or verify annotated/signed tags as release mirrors;
- exchange branches/remotes and report repository divergence;
- ingest externally changed shards as candidate technical changes for Datum
  resolution, semantic validation, impact analysis, and explicit acceptance.

The adapter may not infer approval from commit existence, infer an engineering
revision from commit order, treat a branch or tag name as a Datum lifecycle
transition, or bypass typed operations and the journal. Git failure, absence,
or remoteness must not corrupt Datum revision state. An organization profile
may require successful Git synchronization/signing before a release, but that
is a release policy evaluated by Datum rather than Git becoming the authority.

The distributed-collaboration research must later settle semantic merge,
concurrent operations, remote signatures, and exchange policy. This revision
track owns the product-level meaning of the states being exchanged.

## Primary-source standards baseline

The following sources establish the research perimeter. Exact requirements
from paywalled standards must be verified from lawfully accessible normative
copies before Datum claims a conforming profile.

| Source | Initial relevance | Status |
|---|---|---|
| [ISO 10007:2017](https://www.iso.org/standard/70400.html) | Configuration-management planning, identification, change control, status accounting, and audit across the product lifecycle | Official scope reviewed; normative text pending lawful access |
| [MIL-HDBK-61B](https://quicksearch.dla.mil/WMX/Default.aspx?token=5764667) | DoD configuration-management guidance for hardware/software, digital artifacts, identification, change control, accounting, and audits | Official public source identified; full extraction pending |
| [NASA NPR 7123.1D Change 2 §3.2.15](https://nodis3.gsfc.nasa.gov/displayDir.cfm?Internal_ID=N_PR_7123_001D_&page_name=Chapter3) | Current NASA requirements for tailored CM process, configuration identification/change control, integrity, traceability, security, and lifecycle records | Official public current requirement reviewed clause-by-clause; replaces the historical 1B perimeter citation |
| [NASA Systems Engineering Handbook §6.5](https://www.nasa.gov/reference/6-0-crosscutting-technical-management/) | Baselines, change authority, unique CI/document identifiers, release, and product integrity | Official guidance reviewed at overview level |
| [ECSS-M-ST-40C Rev.1](https://ecss.nl/standard/ecss-m-st-40c-rev-1-configuration-and-information-management/) | Public normative configuration and information management requirements, including identification, control, status, verification, approvals, delivery, and retention | Official complete public text reviewed clause-by-clause for the ECSS profile |
| [MIL-STD-31000B (2018-10-31)](https://quicksearch.dla.mil/WMX/Default.aspx?token=5754451) | Contract-tailored technical data package identity, content, metadata, approval, and index controls | Official public normative text reviewed for invoked TDP profiles |
| [ASME Y14.35-2025](https://www.asme.org/codes-standards/find-codes-standards/revision-of-engineering-drawings-and-associated-documents) | Identification and recording of engineering product-definition and associated-document revisions | Official scope reviewed; normative text pending lawful access |
| [Git tag documentation](https://git-scm.com/docs/git-tag.html) | Annotated/signed release-point identity available to an outer Git integration | Official behavior reviewed; not an engineering CM standard |

Standards families requiring deliberate follow-up include ASME Y14.34/Y14.100,
SAE/EIA-649, AS9100-series quality/configuration obligations, MIL-STD-31000
technical data packages, ISO 9001 documented-information controls, relevant
IPC product/documentation records, and applicable ECSS configuration/change
control. Edition, status, licensing, scope, and normative availability must be
verified before use.

The clause-level source register, requirement/disposition rows, licensing gates,
and coverage proof are maintained in
`PRODUCT_REVISION_ENGINE_STANDARDS_MATRIX.md`. That artifact distinguishes
normative profile requirements, official guidance, publisher scope, and Datum
product policy; its summaries never substitute for the cited authority.

## Research and specification workstreams

<!-- REQ:PRODUCT-REVISION-SPEC:REV-C01 -->
### REV-C01 — Internal authority inventory

Inventory every existing technical, object, model, library, variant, rules,
artifact, document, workspace, and Git revision concept; identify collisions,
missing identities, stale-state paths, and private writers.

<!-- REQ:PRODUCT-REVISION-SPEC:REV-C01A -->
<!-- OWNER:PRODUCT-REVISION-SPEC:REV-C01A:REV-C01A -->
### REV-C01A — Owner acceptance of the proven baseline

Before REV-C02 begins, present the owner with a committed proof packet that
contains clickable code/spec evidence, reproducible search or test commands,
an authority inventory, a contradiction and missing-capability matrix, and a
clear separation of observed fact from interpretation and recommendation. The
owner approves or revises whether that packet is a sufficiently accurate and
complete factual baseline for standards research. This gate does not approve a
Product Revision Engine mechanism or implementation.

<!-- EVIDENCE:PRODUCT-REVISION-SPEC:REV-C01A-APPROVED -->
**Owner review — approved 2026-08-24.** After one explicit revision round, the
owner approved the corrected REV-C01 factual baseline with
`REV-C01-BASELINE: approve`. The accepted packet is
`research/documentation-system/REV_C01_INTERNAL_AUTHORITY_AUDIT.md`, evidenced
by commits `1a4a041` and `009f45c`; commit `d324dce` registered the correction
as Frontier evidence. This approval establishes only the current-authority,
contradiction, and missing-capability baseline for REV-C02. It does not approve
vocabulary, lifecycle, standards conformance, architecture, dependencies, or
implementation.

<!-- REQ:PRODUCT-REVISION-SPEC:REV-C02 -->
### REV-C02 — Standards matrix

Build a requirement/disposition matrix for configuration identification,
baselines, change control, status accounting, audits, drawing revision,
approval/signature, effectivity, records, transmittal, and retention. Separate
normative requirements, organization policy, and Datum product choices. Record
exact authority, edition, clause, applicability, Datum control, required
evidence, verification method, and unresolved or tailored disposition so the
matrix can drive repeatable audits rather than remain explanatory prose.

<!-- REQ:PRODUCT-REVISION-SPEC:REV-C03 -->
### REV-C03 — Authority and operation model

Define the exact objects, identities, relationships, lifecycle states, roles,
typed operations, refusal states, invariants, and query surfaces. Establish the
one path from technical change through approved release without making every
edit an issued revision.

The evidence-constrained working model is maintained in
`PRODUCT_REVISION_ENGINE_AUTHORITY_MODEL.md`. Its candidate mechanisms and
explicit owner questions remain unratified until the later owner-decision gate.
The owner's REV-C03-Q2 sequencing objection exposed revision identity as a
prerequisite to Q2 and Q3. The evidence and visual-review contract for that
prerequisite are maintained in
`REVISION_SEQUENCING_AND_ORIGIN_RESEARCH.md` and
`REV_C03_REVISION_IDENTITY_DECISION_PACKET.md`. ID-Q1 now ratifies the profile
registry and factory `Sequential Alphanumeric (Legacy)` preference. ID-Q2
ratifies its honestly marked provisional `RevisionReservation` treatment while
preserving the rule that only a profile-authorized boundary creates an issued
`EngineeringRevision`. ID-Q3 ratifies profile-owned issuance transitions:
Sequential Alphanumeric uses explicit Issue/Release, ISO 19650 uses its
profile-defined WIP/Shared/Published transitions, and typed boundary context
never mints independently of the selected profile. ID-Q4 ratifies optional
first-class `BuildIdentity` over a baseline manifest, with quiet V4a as the
factory presentation and V4b phase/build hierarchy as a
Global Preferences or governed-enterprise-profile selection over the same data
model. ID-Q5 jointly ratifies the V6a profile-owned status model and the V3d
ISO title-block projection: Sequential Alphanumeric exposes no ISO suitability
axis, while ISO 19650 keeps revision and suitability/status independently
authoritative and renders them in separately labeled title-block cells. Reset
policy is resolved by ID-Q6: one continuing namespace is the factory behavior,
while an enterprise/profile policy may perform an explicit prototype-to-
production namespace transition with immutable supersession lineage. Allocation
choreography outside these identity dispositions remains unratified.
Original REV-C03-Q2 now ratifies automatic lightweight successor collection:
the first post-release divergence opens a quiet Draft `EngineeringChange`
without reserving revision identity; explicit revision preparation later enters
the approved V2-P provisional state. Governed profiles may require an authorized
Change before mutation. REV-C03-Q3 selects V8-A: every issued
`EngineeringRevision` belongs to a revision-bearing CI, including an optional
explicit aggregate Product/System CI. Baselines own exact composition,
`BuildIdentity` optionally labels a baseline, and arbitrary `ReleaseScope`
cannot own a competing revision namespace. Profiles may select revision-bearing
CIs and presentation, but cannot redefine these identity kinds. REV-C03-Q4
permits one atomic `Release` to issue several affected CIs' independently
numbered successor revisions. Unchanged baseline members reuse their existing
issued revisions; the baseline, not label coincidence, proves the complete
configuration. REV-C03-Q5 selects V9-A: mutable Publish Sets remain authored
selection/order objects, while a CI-designated `ControlledDocument` carries
stable document meaning and its EngineeringRevision. Immutable DocumentIssue
references that revision and exact rendered evidence; ReleasePackage and
Transmittal retain exact package and delivery identity.

REV-C03-Q6 selects V10-C. The core stores typed supersession, authorization-
withdrawal, and obsolescence facts; profiles govern per-kind applicability,
authority, rationale, effectivity, allowed sequences, and display terms without
reducing the facts to strings. Issued history remains immutable, Current and
Historical are projections, and delivery never changes standing or bypasses
egress authorization. This completes the REV-C03 owner-question ledger.
A proposed full-drawing provisional-watermark preference is deferred to
`dat-global-preferences-engine-qcv`; no current Preferences implementation is
assumed.

The owner-approved rollout deliberately separates policy consumption from the
not-yet-built Global Preferences authority. Revision specification continues
against an explicit resolved `RevisionPolicy` input. The factory system
preference is `Sequential Alphanumeric (Legacy)`; ISO 19650 and later standards
remain selectable profiles. A future Preferences engine seeds new Project
policy but never remains a live authority over an existing Project or issued
identity. Its specification is tracked by `dat-global-preferences-engine-qcv`
and follows completion of this specification; no current code may claim that
general Preferences storage, precedence, migration, or GUI exists.

<!-- REQ:PRODUCT-REVISION-SPEC:REV-C04 -->
### REV-C04 — Git and distributed/offline integration

The candidate contract is
[`REV_C04_OFFLINE_GIT_INTEGRATION_CONTRACT.md`](REV_C04_OFFLINE_GIT_INTEGRATION_CONTRACT.md).
It specifies complete local authority, durable recovery semantics,
algorithm-qualified Git mapping receipts, transport-neutral air-gapped
exchange, canonical attestations, honest clock/trust evidence, external-change
quarantine and semantic re-entry, and failure-isolated release mirroring.

Git remains optional and subordinate. A Project policy may require a verified
pre-release Git mapping receipt, or block distribution until a post-release
mirror succeeds, but the local atomic Datum `Release` remains the only issuance
event. A clean textual merge is only candidate input until isolated resolution,
semantic comparison/validation, and typed commit succeed. The exact
multi-writer merge, replica, live-session, permission, and server mechanisms
remain reserved to `dat-distributed-collaboration-architecture-lt1` rather than
being smuggled into the revision engine.

<!-- REQ:PRODUCT-REVISION-SPEC:REV-C05 -->
### REV-C05 — Impact, staleness, and reproducibility

The candidate contract is
[`REV_C05_IMPACT_STALENESS_REPRODUCIBILITY_CONTRACT.md`](REV_C05_IMPACT_STALENESS_REPRODUCIBILITY_CONTRACT.md).
It specifies dependency traversal across Design, library, rules, checks,
Publish, manufacturing, and artifacts; durable affected/unaffected/unknown
proofs; precise stale and orphan states; explicit pinned-library uptake;
identity-aligned baseline comparison; topological immutable regeneration; and
byte-reproducible release evidence.

The contract deliberately separates changed, affected, stale, orphaned,
standing, and non-reproducible facts. A missing impact path is not proof of no
impact; a newer library object never silently rebinds a placed instance;
historical evidence stays valid against its frozen baseline; and reproducible
means byte-identical specified outputs from exact source, producer, invocation,
and environment evidence. Informative Reproducible Builds, SLSA, and in-toto
patterns strengthen provenance without being misrepresented as EDA conformance
authority. No implementation or dependency is authorized.

<!-- REQ:PRODUCT-REVISION-SPEC:REV-C06 -->
### REV-C06 — Human experience and visual contract

The Codex-owned visual-study authority is
[`REV_C06_UX_VISUAL_STUDY_BRIEF.md`](REV_C06_UX_VISUAL_STUDY_BRIEF.md). It
preserves the approved revision identity/authority visuals and requires a
Claude-owned extension covering local shell entry, quiet successor work,
Change lifecycle, impact explanation, library uptake, baseline comparison,
regeneration, release readiness, approvals, atomic issuance, reproduction,
offline/Git posture, responsive behavior, and accessibility.

The four-file Claude study renders all twelve coverage frames and maps them to
six bounded owner questions. `REV-C06-Q1` is owner-approved as rendered
candidate `Q1-A-amended`: the Navigator exposes permanent, expanded Revision
groups with informative empty-state rows from Project creation. Reveal on first
record and collapse-by-default are rejected; any later hiding/collapse control
belongs only to the future Global Preferences presentation policy. `REV-C06-Q2`
is owner-approved as `Q2-A-amended`: one cohesive Release pane keeps all nine
section summaries visible, permits only user-requested collapse of clean detail,
and refuses collapse for unresolved or invalidated facts. `REV-C06-Q3` is
owner-approved as `Q3-A-amended`: summary-first assessment retains permanent
Unknown/scope rows, while row drill opens the canonical witness tree beside the
summary without replacing it. `REV-C06-Q4` is owner-approved as `Q4-revised`:
lightweight and regulated profiles retain one complete, stable Change hierarchy;
profiles change obligation and detail without changing identity, meaning,
position, or discoverability. Q5–Q6 remain pending and must be presented exactly
one at a time with exact source and prototype ranges, consequences,
recommendation, and response syntax. No implementation is authorized.

<!-- REQ:PRODUCT-REVISION-SPEC:REV-C07 -->
<!-- OWNER:PRODUCT-REVISION-SPEC:REV-C07:REV-C07 -->
### REV-C07 — Owner disposition and governed ratification

Resolve owner questions, reconcile affected research and decisions, and ratify
mechanism only in numbered decision/spec governance with licensing and
standards claims explicit.

<!-- REQ:PRODUCT-REVISION-SPEC:REV-C08 -->
### REV-C08 — Frontier placement and proof contract

Place bounded implementation slices, migration, conformance gates, fixtures,
standards-profile audit witnesses, evidence export/reproduction checks, and
production acceptance on the Active Frontier. Research completion must not
implicitly authorize implementation.

## First owner decisions to develop slowly

1. Does the first post-release change allocate the next document revision
   immediately, or retain the last released revision plus a conspicuous pending
   change identity until approval?
2. Is revision assignment package-wide, per controlled document, per Sheet, or
   profile-selectable with a baseline manifest composing mixed revisions?
3. Which project objects are always CIs, which may be designated as CIs, and
   which remain technical history only?
4. What constitutes an affected document when source, library, rule, template,
   generator, or metadata changes?
5. What is the minimum no-configuration user experience, and which profiles
   activate review boards, signatories, effectivity, classifications, and
   formal audits?
6. How do release correction, withdrawal, supersession, rollback, branch/merge,
   and emergency deviation work without rewriting history?

These are discussion prompts, not an invitation to answer them in one batch.
The owner dialogue remains one question at a time.
