# REV-C07 Product Revision Engine ratification packet

> Status: **candidate for owner ratification; not yet a decision or implementation
> authorization**. This packet consolidates the completed REV-C01 through REV-C06
> work for adversarial review. Approval authorizes a numbered Product Mechanics
> decision and governed Product Revision Engine specification; it does not
> authorize Rust implementation, a third-party dependency, or Publish Space work.

## 1. The one decision

Datum proposes to ratify a Datum-owned Product Revision Engine that provides
local/offline configuration, change, baseline, revision, release, approval,
effectivity, status-accounting, evidence, and audit authority. Git remains an
optional, subordinate history/transport adapter. The engine uses typed authority
and immutable issued records, supports a low-ceremony factory profile and
standards-driven regulated profiles over the same model, and presents the
owner-approved REV-C06 visual contract.

The owner is not being asked to choose among new candidates in this packet. The
question is whether the consolidation below faithfully carries forward the
already-reviewed evidence and dispositions without widening their claims.

## 2. Evidence map reviewed for this packet

| Subject | Exact reviewed authority |
|---|---|
| Current implementation baseline and contradictions | `REV_C01_INTERNAL_AUTHORITY_AUDIT.md:1-286` |
| Standards/source-strength contract | `PRODUCT_REVISION_ENGINE_STANDARDS_MATRIX.md:10-64` |
| Clause-addressable requirement/disposition matrix | `PRODUCT_REVISION_ENGINE_STANDARDS_MATRIX.md:66-131` |
| Licensed-text limits and Datum-owned product choices | `PRODUCT_REVISION_ENGINE_STANDARDS_MATRIX.md:133-186` |
| Canonical vocabulary and identity separation | `PRODUCT_REVISION_ENGINE_AUTHORITY_MODEL.md:75-119` |
| Authority objects and relationships | `PRODUCT_REVISION_ENGINE_AUTHORITY_MODEL.md:120-427` |
| Lifecycles, standing, roles, operations, invariants, refusals, queries | `PRODUCT_REVISION_ENGINE_AUTHORITY_MODEL.md:428-674` |
| REV-C03 owner disposition ledger | `PRODUCT_REVISION_ENGINE_AUTHORITY_MODEL.md:727-1033` |
| Local/offline authority and durability | `REV_C04_OFFLINE_GIT_INTEGRATION_CONTRACT.md:104-204` |
| Attestation/trust boundary | `REV_C04_OFFLINE_GIT_INTEGRATION_CONTRACT.md:205-265` |
| Optional Git adapter, semantic re-entry, and air-gap exchange | `REV_C04_OFFLINE_GIT_INTEGRATION_CONTRACT.md:266-450` |
| REV-C04 operations, refusals, invariants, profiles, and proofs | `REV_C04_OFFLINE_GIT_INTEGRATION_CONTRACT.md:451-576` |
| Impact, staleness, uptake, comparison, and regeneration | `REV_C05_IMPACT_STALENESS_REPRODUCIBILITY_CONTRACT.md:87-357` |
| Reproducible release evidence | `REV_C05_IMPACT_STALENESS_REPRODUCIBILITY_CONTRACT.md:358-436` |
| REV-C05 operations, invariants, and verification | `REV_C05_IMPACT_STALENESS_REPRODUCIBILITY_CONTRACT.md:438-541` |
| REV-C06 interaction doctrine and twelve required frames | `REV_C06_UX_VISUAL_STUDY_BRIEF.md:90-306` |
| REV-C06 surface routing and accessibility basis | `REV_C06_UX_VISUAL_STUDY_BRIEF.md:308-344` |
| REV-C06 Q1-Q6 dispositions | `REV_C06_UX_VISUAL_STUDY_BRIEF.md:345-494` |
| Visual decision index | `docs/gui/prototypes/revision-ux-study-index.html:40-65` |
| Shell, Navigator, Change, and design-first preference | `docs/gui/prototypes/revision-ux-shell-study.html:113-271` |
| Impact, library uptake, and baseline comparison | `docs/gui/prototypes/revision-ux-impact-study.html:113-287` |
| Regeneration, readiness, and atomic issuance | `docs/gui/prototypes/revision-ux-release-study.html:113-355` |
| Reproduction, adapter, responsive, and accessibility behavior | `docs/gui/prototypes/revision-ux-evidence-study.html:113-290` |

Line references identify the reviewed 2026-08-25 corpus. Evidence traceability
digests remain the machine-enforced drift authority when later edits move lines.

## 3. Proposed ratified mechanism

### 3.1 Two clocks and separate identities

Datum keeps technical history and human engineering revision identity separate.
Transactions, journal tips, object/model revisions, Git commits, revision
reservations, EngineeringRevisions, ConfigurationBaselines, DocumentIssues,
Releases, ReleasePackages, and Transmittals are distinct typed identities.

The technical clock begins with authored work and records every committed
mutation. Human revision identity is allocated only by an explicit,
profile-governed boundary; edits, saves, exports, timers, Git commits, and
retransmittals do not mint it.

### 3.2 Profile registry and defaults

One engine mechanism supports a registry of revision schemes. Sequential
Alphanumeric (Legacy) is the factory new-Project profile. ISO 19650-oriented and
future organization/standards profiles remain selectable without redefining core
record kinds. ISO revision and suitability/status remain separate axes and title-
block fields.

Global Preferences does not yet exist. The Revision Engine consumes an explicit
resolved-policy input as though that boundary exists; the separately tracked
Global Preferences specification must define storage, precedence, provenance,
new-Project seeding, migration, managed policy, and accessible presentation.
Existing Projects retain their copied Project policy rather than silently
following later global-default changes.

### 3.3 Configuration and composition authority

A `ConfigurationItem` is an explicitly designated revision-bearing authority.
`EngineeringRevision` identity is allocated within that CI's namespace. A CI may
designate a single controlled subject or an explicitly authored aggregate
Product/System; aggregate identity is never inferred by roll-up.

A `ConfigurationBaseline` is the exact immutable composition authority. It may
contain independently revised boards, assemblies, controlled documents,
firmware, libraries, rules, and evidence. Optional `BuildIdentity` labels a
baseline but never replaces its manifest identity.

### 3.4 Change authority

Datum owns one engine-level `EngineeringChange` with an append-only lifecycle:

```text
Draft -> ImpactReview -> Authorized -> Implementing -> Verification -> Closed
          \-> Rejected / Deferred / Cancelled
```

Profiles may label stages ECR/ECO/DCO or organization terms and may collapse
permitted transitions, but cannot create competing semantic objects or arbitrary
string states. Waivers and accepted deviations remain separate, bounded
departure records with explicit scope, authority, validity, and effectivity.

In the lightweight profile, first committed divergence from a released baseline
quietly opens identity-free Draft successor Change work. It reserves no revision
identity. Explicit preparation may later create the approved V2-P provisional
reservation; Issue/Release creates issued identity. A regulated Project profile
may require authorized Change authority before mutation.

### 3.5 Documents, packages, and delivery

`PublishSet` stays an editable, user-named ordered selection. It never owns a
document number, engineering revision, release state, or delivery identity.
`ControlledDocument` is the stable authored-publication authority designated by
a revision-bearing CI. Immutable `DocumentIssue` binds an EngineeringRevision,
frozen configuration, and output bytes. `ReleasePackage` selects exact issues;
`Transmittal` records delivery. One editable composition may explicitly source
several ControlledDocuments with different purposes and policies.

### 3.6 Atomic Release and immutable standing

One `Release` may atomically issue one or several independently numbered
EngineeringRevisions. Only affected successor CIs receive new revisions;
unchanged baseline members reuse their existing issued revisions without
reminting. Successful issuance creates a separate immutable Release, baseline,
revisions, document issues, and packages. It never mutates the ReleaseCandidate
into release authority or freezes the working Design model.

Issued records never mutate. Later standing is represented by typed
`SupersessionEstablished`, `AuthorizationWithdrawn`, and
`ObsolescenceDeclared` facts. Profiles govern applicability, required authority,
rationale, transition rules, effectivity, and displayed terminology, but never
rewrite the issued record or reduce the typed facts to free-form strings.

### 3.7 Impact, staleness, uptake, and regeneration

`Changed`, `Affected`, `Unaffected`, `ImpactUnknown`, `Stale`, `Orphaned`, and
`NonReproducible` are different facts. Every Affected/Unaffected result requires
a typed witness path over a frozen dependency snapshot; incomplete graph coverage
produces `ImpactUnknown`, never an optimistic `Unaffected`.

Staleness compares declared evidence inputs to an explicit target
configuration. Historical evidence remains valid evidence for its frozen
baseline even when successor work changes. Library updates never silently rebind
placed Design objects; uptake is explicit and impact-bearing. Regeneration is
topologically ordered, reuses proven-current evidence, and creates successor
evidence without editing prior evidence.

### 3.8 Reproduction and authenticity

Release reproduction binds exact source subjects, producer/build identity,
invocation/settings, environment, and specified output bytes. Success means
`ByteIdentical` for every specified output. `CanonicalEquivalent` is a diagnostic
only and cannot satisfy reproducibility. A later mismatch, unavailable producer,
or failed execution creates a new attempt result without changing the Release,
issued bytes, approvals, or standing. Digest equality and authenticity/signer
authority remain separate typed facts.

### 3.9 Standalone authority, Git, and exchange

Every engine capability—including approval, baseline establishment, issuance,
standing, audit, and reproduction—works without Git or network service. Canonical
local records are append-only/digest-linked, crash-recoverable, and queryable.

The optional Git adapter may materialize deterministic records, associate
algorithm-qualified commit/tag references, mirror completed releases, and carry
exchange bundles. Git cannot authorize, issue, withdraw, supersede, or rewrite
Datum authority. External changes enter quarantine and become Datum authority
only after integrity/compatibility checks, semantic comparison, validation,
translation into typed operations, review, and local commit. Full simultaneous
multi-writer semantic merge remains reserved to
`dat-distributed-collaboration-architecture-lt1`.

## 4. Proposed ratified human contract

The visual source of truth is the four-file REV-C06 study plus its decision
index. The six approved dispositions are:

| Decision | Ratified candidate | Consequence |
|---|---|---|
| Q1 | A-amended (`21a1cc1`) | Revision Navigator groups are visible and expanded from Project creation with informative empty states. |
| Q2 | A-amended (`213b5d6`) | One cohesive Release pane keeps nine semantic summaries visible; unresolved facts refuse collapse. |
| Q3 | A-amended (`0ce0760`) | Impact is summary-first; the canonical witness tree opens beside it; Unknown/scope rows are permanent. |
| Q4 | revised (`e41a15c`) | Lightweight and regulated profiles use one complete ordered Change hierarchy; policy changes obligation/detail only. |
| Q5 | B-amended (`6873456`) | Atomic issuance uses an in-pane arm-then-confirm bar; exact consequences remain visible; no modal is used. |
| Q6 | A-amended (`188276b`) | Verdicts lead over permanent counted evidence categories; deficiencies refuse collapse; immutable manifest detail opens beside. |

The interaction laws remain one native Datum window, recursive user-controlled
tiling, open-beside navigation, focused-pane ownership, output-only Console, and
no visual surface gaining rival product authority.

## 5. Design-first, never-blocks invariant

This invariant is proposed for explicit ratification because it was owner-
directed after the six-question brief and rendered in
`revision-ux-shell-study.html:236-271` (commit `e3112f7`):

1. Under the factory lightweight profile, the Revision Engine never blocks,
   prompts, or delays ordinary Design authoring.
2. Gates occur when the user asks Datum to certify/issue a Release, or when the
   Project has explicitly adopted a regulated profile that requires earlier
   control.
3. The engine always maintains technical history, identity, and audit truth even
   when revision presentation is hidden.
4. The future Global Preferences specification must provide a **Hide revision
   system** presentation preference. It may hide Navigator groups and revision
   projections only; it cannot disable the journal, discard records, weaken
   Project policy, bypass release gates, or require migration to restore the UI.
5. Permanent visible/expanded Revision groups remain the factory default approved
   by Q1. The hide control is an optional later preference, not a contradiction
   or retroactive change to Q1.

## 6. Standards and licensing boundary

Datum's standards matrix is an auditable applicability/disposition system, not a
claim of blanket certification. Public primary sources support architectural
principles. Exact clause obligations, controlled vocabulary, label rules, record
retention, and conformance profiles require edition-pinned licensed text where
the matrix says so. A green Datum audit proves only the recorded profile,
applicability, evidence, and dispositions; it does not claim regulator,
customer, or certification-body acceptance.

No new third-party implementation dependency is approved by this ratification.
Cryptographic algorithms/libraries, signature providers, PDF/export engines,
Git libraries, and external standards content remain dependency-029 and/or
licensed-source decisions. Git CLI availability cannot become a hidden core
dependency because standalone local authority is mandatory.

## 7. Reconciliations and migrations required before production acceptance

Ratification acknowledges rather than erases the REV-C01 gaps:

- journal fingerprint/tip and crash-recovery authority must be completed without
  equating `ModelRevision` with product revision;
- existing free-form `rev-*` / `release-*` labels remain non-authoritative;
- the current CLI `release` check profile must be renamed or clearly separated
  from atomic `ReleaseConfiguration`;
- existing proposal approvals, check waivers/deviations, Part lifecycle, library
  provenance/review annotations, and ZoneFill staleness require explicit mapping
  or migration—none silently satisfy a stronger engine record;
- title-block research claims about an existing six-state PLM machine and
  EngineeringChangeOrder implementation remain corrected as non-existent;
- GUI technical `rev` reporting must identify its revision class and cannot be
  presented as an EngineeringRevision.

REV-C08 must turn these into bounded implementation/migration slices and proof
gates. This packet does not select their Rust module layout or authorize work.

## 8. Deliberately deferred and excluded

- Global Preferences storage, precedence, managed policy, synchronization, and
  UI implementation → `dat-global-preferences-engine-qcv`.
- Simultaneous multi-writer/distributed semantic merge →
  `dat-distributed-collaboration-architecture-lt1`.
- Publish Space implementation remains blocked until the Revision Engine
  foundation reaches production acceptance.
- Verified redaction/package variants remain `dat-publish-redaction-contract-wzs`.
- Exact cryptographic/signature dependencies and licenses require separate
  numbered dependency authority.
- Implementation decomposition, migrations, proof fixtures, standards-profile
  witnesses, and production acceptance are REV-C08—not this owner decision.

## 9. What approval will create

After owner approval, Codex may create only the following governance artifacts:

1. `docs/decisions/PRODUCT_MECHANICS_034_PRODUCT_REVISION_ENGINE.md`, recording
   the ratified mechanism and its non-authorization/dependency limits.
2. `specs/PRODUCT_REVISION_ENGINE_SPEC.md`, turning this authority into testable
   normative requirements without inventing new mechanism.
3. Reconciled Publish/GUI/roadmap/spec-governance references.
4. REV-C08 implementation and proof planning on the Active Frontier, still
   planning-only until separately authorized.

If the owner revises any mechanism, REV-C07 remains open and the affected source,
visual, and packet claims must be corrected before decision/spec creation.

## 10. Adversarial review checklist

Before owner signature, verify that this packet:

- never turns Git, filenames, labels, timestamps, or UI state into authority;
- never mutates or reinterprets an issued record;
- never lets a profile redefine core identity kinds or typed fact meanings;
- never equates Changed, Affected, Stale, Orphaned, or NonReproducible;
- never claims `CanonicalEquivalent` satisfies byte reproducibility;
- never hides an unresolved/Unknown/blocking fact by default;
- never lets **Hide revision system** disable engine truth or regulated policy;
- never makes lightweight authoring ceremony a universal requirement;
- never weakens licensed-source, dependency-029, or certification boundaries;
- preserves every Q1-Q6 amended visual disposition exactly; and
- leaves implementation authorization to REV-C08 and later owner gates.

## 11. Owner response

<!-- OWNER:PRODUCT-REVISION-SPEC:REV-C07:REV-C07-RATIFICATION -->

After Claude's findings-first adversarial review, respond with exactly one:

```text
REV-C07-RATIFICATION: approve
```

or

```text
REV-C07-RATIFICATION: revise — <specific contradiction, omission, or boundary correction>
```

Approval means the packet faithfully consolidates the mechanism and authorizes
the governance artifacts in section 9. It does **not** authorize implementation,
new dependencies, certification claims, or Publish Space development.
