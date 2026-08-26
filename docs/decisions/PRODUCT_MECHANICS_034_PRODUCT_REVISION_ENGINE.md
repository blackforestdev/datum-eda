# Product Mechanics 034: Datum Product Revision Engine

Status: ratified doctrine

## Context

Datum already owns technical change history through stable identities, typed
operations, `commit()`, object/model revisions, and the transaction journal.
Those mechanisms cannot answer the different engineering question: which exact
configuration was reviewed, approved, issued, delivered, and remains applicable?
Git commits, filenames, title-block strings, mutable Publish Sets, and technical
revision counters likewise cannot supply that authority.

The owner requires one system that remains simple for an unconstrained designer,
scales to standards-driven configuration control, works completely offline, and
is independently auditable. REV-C01 through REV-C06 established the internal
baseline, standards/source-strength matrix, authority model, offline/Git seam,
impact and reproducibility contract, and owner-reviewed visual behavior. The
corrected REV-C07 packet was approved on 2026-08-25.

## Decision

Datum owns one **Product Revision Engine**. It is the authority for configuration
items, engineering changes, baselines, human engineering revisions, approvals,
effectivity, releases, controlled-document issues, release packages,
transmittals, standing, status accounting, reproducibility, retention, and audit.
It uses the canonical Datum mutation path and remains fully capable without Git,
a network, or a remote service.

The engine keeps two clocks separate:

- the technical clock records every accepted authored mutation through the
  journal and object/model revision system; and
- the human revision clock advances only at an explicit profile-governed
  boundary.

An edit, save, export, timer, Git commit, retransmission, filename, or UI label
never mints an EngineeringRevision.

## Configuration and revision identity

A `ConfigurationItem` explicitly designates one controlled subject or authored
aggregate as revision-bearing authority. An `EngineeringRevision` is allocated
only inside that CI's namespace. Aggregate Product/System identity is permitted
but never inferred by rolling up member labels.

A `ConfigurationBaseline` is the exact immutable composition authority. It may
contain independently revised boards, assemblies, controlled documents,
firmware, libraries, rules, plans, and evidence. An optional `BuildIdentity`
labels a baseline without replacing its manifest identity. One atomic `Release`
may issue several affected CIs' independent EngineeringRevisions; unchanged
members reuse existing issued revisions without reminting.

The factory new-Project revision profile is Sequential Alphanumeric (Legacy).
ISO 19650-oriented and future organization/standards profiles share the same
engine types. ISO revision and suitability/status remain separate axes. Global
Preferences will seed explicit policy into new Projects; existing Projects keep
their copied Project policy until deliberately changed.

## Change, release, and standing

Datum has one engine-level `EngineeringChange` whose append-only lifecycle is:

```text
Draft -> ImpactReview -> Authorized -> Implementing -> Verification -> Closed
          \-> Rejected / Deferred / Cancelled
```

Profiles may label or collapse allowed stages but cannot create competing
semantic objects or arbitrary string states. Waivers and accepted deviations
remain distinct bounded departures.

Lightweight post-release divergence quietly opens identity-free successor
Change work. It reserves no revision label. Explicit preparation may create the
approved provisional reservation; Issue/Release creates issued identity. A
Project profile may explicitly adopt earlier control and require authorized
Change authority before mutation.

`ReleaseCandidate` is mutable preparation state. Successful
`ReleaseConfiguration` atomically creates separate immutable Release, baseline,
EngineeringRevision, document-issue, and package records; it never converts the
candidate into authority or freezes the working Design model.

Issued records never mutate. Later standing uses typed
`SupersessionEstablished`, `AuthorizationWithdrawn`, and
`ObsolescenceDeclared` facts. Current/Historical are projections. Delivery does
not change standing.

## Controlled documents and Publish boundary

`PublishSet` remains an editable, user-named ordered selection. It owns no
document number, human revision, release state, delivery identity, or issued
bytes. `ControlledDocument` is stable authored-publication authority designated
by a revision-bearing CI. Immutable `DocumentIssue` binds its issued
EngineeringRevision, baseline configuration, render context, and exact output
bytes. `ReleasePackage` selects exact issued records; `Transmittal` records an
exact delivery event.

Publish owns composition and the display/substrate for staleness. The Revision
Engine supplies exact `ConfigurationRef` context, impact facts, issued identity,
and release projections. Neither side implements a rival lifecycle.

## Impact and reproducibility

`Changed`, `Affected`, `Unaffected`, `ImpactUnknown`, `Stale`, `Orphaned`, and
`NonReproducible` are distinct typed facts. Affected/Unaffected requires a
machine-readable witness over a frozen dependency snapshot; incomplete
coverage yields `ImpactUnknown`, never optimistic `Unaffected`.

Library uptake is explicit and impact-bearing. Regeneration is topologically
ordered, may reuse proven-current evidence, and creates successor evidence
without editing historical evidence. Baseline comparison aligns stable identity
and reports Added, Removed, Modified, Retargeted, and Unchanged; an
administrative rename is metadata within Modified, not delete/add.

Release reproduction binds exact source, producer/build, invocation,
environment, and specified output bytes. Success is `ByteIdentical` for every
specified output. Canonical, visual, or electrical equivalence is diagnostic
only. Digest equality and authenticity/authorization remain separate facts.

## Standalone authority and optional Git

Every approval, baseline, release, standing, status, reproduction, and audit
workflow has a non-Git, non-network execution path. Local records are
append-only/digest-linked, crash-recoverable, independently verifiable, and
transportable through a verified authority exchange.

An optional Datum-owned Git adapter may materialize deterministic shards,
associate algorithm-qualified commits/tags, mirror completed Releases, and
transport exchange payloads. Git cannot approve, allocate, issue, withdraw,
supersede, or rewrite Datum authority. External changes remain quarantined until
integrity and compatibility checks, semantic comparison, validation, translation
to typed operations, review, and local commit. Simultaneous multi-writer merge
remains reserved to `dat-distributed-collaboration-architecture-lt1`.

## External enterprise authority posture

<!-- EVIDENCE:PRODUCT-REVISION-SPEC:ENTERPRISE-AUTHORITY-POSTURE -->
Datum remains the revision, configuration, and release authority when PLM, PDM,
supplier, customer, or external configuration-management systems participate.
No external-system-master mode is adopted. External systems integrate only as
subordinate, failure-isolated adapters: they may receive mirrored immutable
records and submit exchange envelopes, signed attestations, or candidate
changes, but inbound material remains quarantined until Datum validates,
translates, authorizes, and commits it through the canonical mutation path.

An external configuration manager may hold scoped Configuration or Release
capability **inside Datum** through `ActorIdentity` and `RoleAssignment`, and may
exercise it remotely through signed attestation exchange. The human or
organization may be external; the authoritative role, target statement,
authorization evaluation, accepted attestation, baseline, revision, and Release
remain Datum records. A future PLM/PDM adapter may mirror outward like the Git
adapter; it cannot allocate, approve, issue, supersede, withdraw, obsolete, or
rewrite Datum authority on its own.

## Human contract

The approved REV-C06 visual contract is normative:

- Revision Navigator groups are visible and expanded from Project creation,
  with informative empty states.
- One cohesive Release pane keeps all nine semantic summaries visible; only
  clean detail may collapse, and unresolved facts refuse collapse.
- Impact opens summary-first and its canonical witness tree opens beside it;
  Unknown and graph-scope rows remain permanently visible.
- Lightweight and regulated profiles expose the same complete ordered Change
  hierarchy; policy changes obligation and detail, never meaning or discovery.
- Atomic issuance uses an in-pane arm-then-confirm bar that shows exact
  consequences before the second deliberate action; no modal is used.
- Reproduction verdicts lead over permanent counted evidence categories;
  deficient evidence refuses collapse and immutable manifest detail opens beside.

Under any profile that has not explicitly adopted earlier control, the Revision
Engine never blocks, prompts, or delays Design authoring; its only gates are
release certification/issuance and earlier control explicitly adopted by Project
policy. Ignoring the revision engine entirely is a supported, first-class
workflow, not a degraded one.

Engine truth remains maintained when its presentation is hidden. A future
**Hide revision system** preference may hide Navigator groups and projections
only; it cannot disable the journal, discard records, weaken Project policy,
bypass gates, or require migration to restore presentation. Visible expanded
groups remain the factory default.

## Standards, evidence, and dependency boundary

Profiles map edition-pinned requirements and applicability to controls,
evidence, and dispositions. A Datum audit proves only the configured profile,
scope, evidence, and recorded dispositions. It does not claim regulator,
customer, accreditation, or certification-body acceptance. Licensed normative
text remains gated wherever the standards matrix says so.

This decision selects no cryptographic algorithm, signature provider, Git
library, PDF/export engine, external standards content, or other dependency.
Every such choice remains subject to Product Mechanics 029 and applicable
license authority. Git CLI availability cannot become a hidden core dependency.

## Required reconciliation and sequencing

Before production acceptance, implementation must reconcile journal-tip/crash
recovery, free-form revision labels, the existing CLI `release` check-profile
name, current proposal/check departure records, Part lifecycle, library
provenance, ZoneFill staleness, false legacy PLM/ECO claims, and ambiguous GUI
technical `rev` wording without pretending those surfaces already implement this
engine. It must also implement and owner-review enterprise role workflow UX for
assignment, bounded delegation, actionable queues, authority/provenance
visibility, and the external configuration-manager exchange path.

REV-C08 must place bounded implementation, migration, audit-witness,
reproduction, and production-acceptance slices on the Frontier. This decision
does not authorize implementation or Publish Space development. Publish Space
implementation remains blocked until the Revision Engine foundation reaches
production acceptance. Global Preferences specification is the ordered
architecture follow-on after REV-C08 planning.

## Owner evidence

<!-- EVIDENCE:PRODUCT-REVISION-SPEC:REV-C07-RATIFIED -->
The owner approved `REV-C07-RATIFICATION` on 2026-08-25 after requiring the
design-first invariant to apply to every profile that has not explicitly adopted
earlier control and declaring that ignoring the revision engine is a supported,
first-class workflow. All other consolidated claims were verified faithful
against the disposition ledger and drawn evidence.

On 2026-08-25 the owner explicitly retained Datum-authoritative enterprise
integration: no PLM/PDM or other external system may become revision authority;
future adapters remain subordinate through quarantine/exchange and outward
mirroring. The owner also directed enterprise role-workflow UX and a documented
external configuration-manager scenario into pre-production scope.

## Dependency and licensing impact

None. This decision grants no dependency or license exception under Product
Mechanics 029.
