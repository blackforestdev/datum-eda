# Datum Product Revision Engine Implementation and Production-Acceptance Plan

> **Status:** Governed planning contract; implementation is not authorized.
>
> **Tracker:** `dat-product-revision-engine-build-18f`.
>
> **Authority:** Product Mechanics 034 and
> `specs/PRODUCT_REVISION_ENGINE_SPEC.md`. This plan decomposes that ratified
> mechanism; it cannot change it.

## 1. Purpose and authorization boundary

This plan converts the Product Revision Engine specification into bounded,
ordered implementation and proof slices. It exists so later execution cannot
silently choose a storage authority, merge technical and human revisions, add a
dependency, skip migration, build GUI-first, or declare production readiness
from happy-path demonstrations.

The complete program is blocked until:

1. REV-C08 closes `dat-product-revision-engine-k9f` with this plan and Frontier
   placement;
2. `dat-global-preferences-engine-qcv` ratifies the policy-resolution boundary
   that seeds new Projects while preserving copied Project policy; and
3. the owner explicitly authorizes `REV-I00` after reviewing the then-current
   dependency, migration, source-health, and proof posture.

No later slice inherits execution authorization merely because its dependency
is complete. No new third-party dependency is permitted without an exact
Product Mechanics 029 decision naming the dependency and license obligations.
Publish Space implementation remains separately blocked until `REV-I12`
production acceptance.

## 2. Current substrate and required reconciliation

Implementation shall extend the existing one-mutation-path substrate rather
than create a PLM sidecar application:

- stable object IDs/revisions, `ModelRevision`, typed `OperationBatch`,
  `commit()`, and the journal already own accepted technical mutations;
- `ProjectResolver` already assembles one `DesignModel` from authored,
  sidecar, and generated-evidence shards;
- native-write, daemon, CLI, MCP, proposal, undo/redo, and GUI supervision
  already expose revision/tip guards in different bounded forms;
- check waivers/deviations, Part lifecycle, library provenance/review, artifact
  model pins, and ZoneFill stale findings are real narrow precedents; and
- no ConfigurationItem, EngineeringChange, ConfigurationBaseline,
  EngineeringRevision, Release, ControlledDocument, DocumentIssue,
  ReleasePackage, Transmittal, general dependency graph, revision audit, or Git
  adapter exists today.

The implementation must resolve, not hide, REV-GAP-01 through REV-GAP-12 from
`REV_C01_INTERNAL_AUTHORITY_AUDIT.md`. Existing labels, review booleans,
proposal acceptance, check disposition, technical `rev` chrome, and the CLI
check profile named `release` cannot be promoted by type alias or wording alone.

## 3. Architecture and source-health law

The engine crate owns semantics and persistence. GUI, CLI, MCP, and adapters are
consumers. The required ownership bands are:

```text
revision authority types + canonical encoding
    -> revision authority store/resolver
        -> typed native-write operations + commit hooks
            -> read/query service
                -> daemon/CLI/MCP parity
                    -> GUI projections and workflows
```

The implementation shall introduce cohesive modules before feature growth. It
must not append the system to `substrate/mod.rs`, `operation.rs`, GUI protocol
`lib.rs`, renderer monoliths, or CLI dispatch monoliths. A shared type or service
used by several domains is implemented once and configured by callers.
Forwarding-only `include!` splits do not satisfy Product Mechanics 022.

The physical shard/database layout remains an implementation decision inside
the logical `RevisionAuthorityStore` contract. Whichever layout is selected must
remain deterministic, Git-reviewable where JSON is used, locally complete,
atomic with the journal, independently verifiable, and migration-versioned.

## 4. Ordered implementation program

<!-- REQ:PRODUCT-REVISION-ENGINE:REV-I00 -->
<!-- OWNER:PRODUCT-REVISION-ENGINE:REV-I00:REV-I00 -->
### REV-I00 — Execution authorization and baseline lock

**Kind:** owner decision. **Authorization:** none until explicit approval.

Before code changes, produce one committed baseline packet containing:

- the completed Global Preferences policy-resolution specification and its exact
  seam into Project revision policy;
- current source-health ceilings and intended new module ownership;
- the proposed local authority persistence/canonicalization approach;
- the exact initial no-new-dependency posture and any separately requested
  Product Mechanics 029 decisions;
- current results for mutation fences, resolver fences, dependency authority,
  spec parity, locked/offline checks, and a guarded workspace proof baseline;
- real-project fixtures selected for every later proof; and
- a scope statement showing that Publish implementation and distributed merge
  remain excluded.

Owner response must be recorded as
`REVISION-ENGINE-EXECUTION: approve` or a specific revision. Approval authorizes
only the selected first execution slice, not the whole program concurrently.

<!-- REQ:PRODUCT-REVISION-ENGINE:REV-I01 -->
### REV-I01 — Technical integrity and atomic authority substrate

Resolve the prerequisites on which every immutable product record depends:

- reconcile `ModelRevision` and accepted transaction-tip authority with Product
  Mechanics 000D without equating either to EngineeringRevision;
- add the required parent/affected-shard/hash/validation/cache-invalidations or
  formally reconcile their exact technical-journal ownership;
- provide staged authority-record/blob writes with one journal commit point;
- make recovery expose the last complete state or read-only diagnostic state,
  never a partial Release;
- add versioned canonicalization, algorithm-qualified digests, integrity root,
  backup/export completeness manifest, restore, and independent verification;
- retain expected model revision plus accepted-tip fences through native-write,
  daemon, CLI, MCP, undo/redo, and proposal application; and
- prove per-Project single-writer/TOCTOU safety through the separately required
  write-ownership decision if it is still unresolved.

**Exit evidence:** crash injection at every staged/promote/journal boundary;
corruption/truncation/missing-blob tests; backup/restore equivalence; no private
writer; and unchanged Design mutation semantics on real Projects.

<!-- REQ:PRODUCT-REVISION-ENGINE:REV-I02 -->
### REV-I02 — Typed authority core and local resolution

Implement versioned, non-interchangeable IDs, canonical records, events, and
resolution for:

- ConfigurationItem, ConfigurationRef, ConfigurationBaseline,
  EngineeringRevision, RevisionReservation, and BuildIdentity;
- EngineeringChange, ApprovalAttestation, Effectivity, ReleaseCandidate,
  Release, and typed standing facts;
- ControlledDocument, DocumentIssue, ReleasePackage, and Transmittal;
- StandardsProfile, RequirementDisposition, AuditEvaluation, retention, hold,
  and disposition records; and
- dependency, impact, evidence, regeneration, reproduction, adapter-mapping,
  exchange, and external-candidate identities required by later slices.

This slice establishes persistence and exact read-only resolution, not business
workflow completion. Unknown schema, ID-kind substitution, mutable issued-body
attempts, unresolved references, and invalid canonical payloads must fail with
typed diagnostics. No generic public JSON patch is allowed.

**Exit evidence:** deterministic round-trip and digest goldens; type-confusion
compile/runtime refusals; unknown-version read-only recovery; stable rename/path
movement; and independent resolution with Git absent.

<!-- REQ:PRODUCT-REVISION-ENGINE:REV-I03 -->
### REV-I03 — Project policy, roles, attestations, and effectivity

Implement the explicit resolved-policy input and Project-owned policy versions,
using the Global Preferences specification only for new-Project seeding. Existing
Projects retain copied policy. Implement:

- Sequential Alphanumeric (Legacy) factory behavior, ISO-oriented separate
  revision/status axes, custom scheme registry, and namespace rules;
- scoped ActorIdentity/RoleAssignment capability checks and effective intervals;
- canonical ApprovalAttestation statements, changed-target invalidation,
  supersession/revocation, quorum/order/independence/separation rules;
- typed Effectivity expressions and resolved-population snapshots; and
- profile-controlled terminology/presentation without redefining core facts.

Cryptographic methods remain pluggable policy capabilities. This slice cannot
select a signature algorithm, key store, provider, or library without separate
dependency authority.

**Exit evidence:** lightweight multi-capability actor; regulated separated-role
flow; changed-target invalidation; unauthorized-but-valid-signature distinction
through a test provider; effectivity ambiguity refusal; existing-Project policy
stability after global-default change; and offline operation.

<!-- REQ:PRODUCT-REVISION-ENGINE:REV-I04 -->
### REV-I04 — EngineeringChange and bounded departures

Implement the one append-only EngineeringChange lifecycle, identity-free quiet
successor collection, explicit V2-P reservation preparation, affected-item
accounting, implementation transaction links, verification, closure, and typed
Rejected/Deferred/Cancelled exits.

Map existing proposal, review, CheckWaiver, and accepted CheckDeviation facts
honestly:

- proposal acceptance remains proposal authority;
- review booleans and free-form actors cannot satisfy controlled approval;
- waivers/deviations remain bounded departures with exact scope, authority,
  validity, effectivity, and governing requirement/finding; and
- migration records whether an existing fact is mapped, retained as legacy
  evidence, or rejected as insufficient.

Under profiles without explicitly adopted earlier control, no operation in this
slice may block, prompt, or delay ordinary Design authoring. A policy that has
explicitly adopted earlier control may refuse mutation with exact authority and
remediation.

**Exit evidence:** first divergence opens one quiet Draft Change without
revision identity; repeated edits collect without duplicate Changes; provisional
reservation is explicit; regulated pre-authorization refusal; complete
append-only lifecycle/rework; and undo/proposal parity before any issued record.

<!-- REQ:PRODUCT-REVISION-ENGINE:REV-I05 -->
### REV-I05 — Dependency, impact, baseline, and library uptake

Implement deterministic DependencySnapshot, SemanticDelta, ImpactEvaluation,
evidence-input freshness, explicit library-uptake preview/adoption, stable-ID
baseline comparison, and topological regeneration planning.

Required edge coverage crosses Design, library, rules, checks, Publish source,
manufacturing, generated artifacts, controlled documents, packages, and policy.
An incomplete graph/evaluator produces permanent `ImpactUnknown`. Every
Affected/Unaffected result cites a machine-readable witness. Administrative
rename is Modified metadata, never delete/add.

`EstablishConfigurationBaseline` shall freeze exact members, technical
revisions, journal tip, governing Changes/departures, profiles, attestations,
dependency snapshot, and digest. No floating reference is allowed.

**Exit evidence:** transitive affected paths; sensitivity-disjoint Unaffected
witnesses; missing-edge Unknown; ZoneFill compatibility; pinned-library missing
and explicit uptake; Added/Removed/Modified/Retargeted/Unchanged comparison;
unaccounted-difference refusal; and immutable historical baseline resolution.

<!-- REQ:PRODUCT-REVISION-ENGINE:REV-I06 -->
### REV-I06 — Atomic multi-CI release and document authority

Implement ReleaseCandidate readiness/invalidation and atomic
`ReleaseConfiguration` over one or several independently revised CIs. The
transaction shall create the exact baseline, affected EngineeringRevisions,
Release, prepared DocumentIssues, and ReleasePackages; unchanged members reuse
existing revisions. Failure leaves none visible.

Implement V9-A document authority:

- mutable PublishSet remains source composition only;
- one CI-designated ControlledDocument owns stable document identity;
- immutable DocumentIssue binds EngineeringRevision, baseline, render context,
  exact outputs, approvals, and Release;
- ReleasePackage selects exact issues/artifacts; and
- Transmittal records an exact delivery without minting revision or changing
  standing.

Implement typed supersession, withdrawal, and obsolescence without issued-body
mutation. Retarget/remove dependencies before deleting a bound Publish source.
Rename the existing CLI check profile or make its readiness-only meaning
unambiguous before exposing atomic release commands.

**Exit evidence:** single- and multi-CI atomic issuance; unchanged-revision
reuse; injected failure at every commit phase; stale candidate/approval refusal;
duplicate-label and floating-input refusal; immutable standing; retransmittal
without reminting; and title-block fields projected only from engine authority.

<!-- REQ:PRODUCT-REVISION-ENGINE:REV-I07 -->
### REV-I07 — Immutable evidence, regeneration, and byte reproduction

Implement executable regeneration plans and ReproductionManifest/Attempt
authority. Manifests bind exact source subjects, dependency snapshot,
producer/build, invocation, influential environment, variance policy, and
specified output bytes. Regeneration creates successor evidence and can reuse
only evidence proven current under identical declared inputs.

`ByteIdentical` requires equality for every specified output. Mismatch,
Unavailable, and ExecutionFailed remain distinct. Canonical/visual/electrical
equivalence is diagnostic only. Later attempts never alter Release, issued
bytes, approval, or standing.

**Exit evidence:** successful reproduction from independent disk locations and
declared-irrelevant locale/timezone/path variation; controlled timestamp,
randomness, ordering, locale, and path-leak failures; missing producer/environment;
external-generator exclusion/refusal; retained original bytes; and digest versus
authenticity separation.

<!-- REQ:PRODUCT-REVISION-ENGINE:REV-I08 -->
### REV-I08 — Offline audit exchange and optional Git adapter

Implement the transport-neutral authority exchange, external-change quarantine,
semantic re-entry, adapter mapping receipts, divergence observations, and
failure-isolated Release mirroring specified by REV-C04.

Every core workflow and historical query must pass with no `.git`, no Git
executable, and no network. Git support may use already-authorized system
process boundaries only if a controlling decision permits; adding a Git crate,
signature library, network client, or helper dependency requires its own
Product Mechanics 029 decision.

External state becomes authoritative only after envelope/schema/Project/integrity
checks, isolated resolution, semantic comparison and validation, translation to
typed operations, review, and local commit. This slice does not implement
simultaneous multi-writer merge.

**Exit evidence:** full and incremental air-gap exchange; missing prerequisite,
replay, wrong destination, corruption, and unknown schema; Git ref move/delete
without history mutation; clean textual but semantic-invalid quarantine;
pre-release mapping versus post-release mirror policy; adapter crash/retry; and
completed local Release surviving mirror failure.

<!-- REQ:PRODUCT-REVISION-ENGINE:REV-I09 -->
### REV-I09 — Engine, native-write, daemon, CLI, and MCP parity

Expose the ratified operations, queries, refusals, and machine-readable evidence
through one engine service and the existing native-write/verb registry path.
CLI and MCP remain thin surfaces; neither authors private operations or
reconstructs authority from files/Git. Read-only queries must cover current and
as-of configuration, Changes, baselines, revisions, approvals, effectivity,
releases, documents, packages, transmittals, standing, impact, evidence,
reproduction, audits, retention, adapters, exchanges, and trust paths.

Public inventory additions shall update verb/parity manifests and include
permission, stale-context, unavailable-adapter, deterministic JSON, and human
view tests. Proposal twins are required for any appropriate authorable operation;
release/approval authority cannot be simulated through a proposal-only shortcut.

**Exit evidence:** generated operation/query inventory parity; CLI/daemon/MCP
round trips over the same real Project; identical refusal codes/payloads;
historical query completeness; context-fence enforcement; and no private writer.

<!-- REQ:PRODUCT-REVISION-ENGINE:REV-I10 -->
### REV-I10 — Approved GUI contract

Implement only the owner-approved REV-C06 visual behavior over engine truth:

1. permanent expanded Revision Navigator groups and informative empty states;
2. one cohesive Release pane with nine always-visible summaries and
   unresolved-detail collapse refusal;
3. summary-first Impact with its canonical witness tree opened beside;
4. one stable Change hierarchy whose obligations/detail scale by profile;
5. pinned in-pane arm-then-confirm issuance with exact consequences; and
6. verdict-first counted reproduction evidence with immutable manifest opened
   beside.

Preserve one native window, recursive tiling, open-beside navigation,
focused-pane ownership, output-only Console, responsive layouts, AT-SPI
semantics, non-color-only states, and focus/keyboard behavior. No GUI state owns
revision truth. The future Hide revision system preference is a presentation
seam only; actual global storage/UI remains owned by the Global Preferences
program unless already implemented and separately authorized.

**Exit evidence:** functional behavior tests, accessibility inspection,
pixel-exact approved-state goldens wired into standing gates, running-app owner
visual review, narrow/HiDPI/focus/terminal-open layouts, and proof that Design
authoring remains unblocked outside explicitly adopted earlier control.

<!-- REQ:PRODUCT-REVISION-ENGINE:REV-I10A -->
### REV-I10A — Enterprise role-workflow UX

Implement and owner-review the enterprise human workflow over the same typed
authority used by REV-I03 and REV-I09:

- role assignment and bounded delegation with capability, scope, effective
  interval, rationale, provenance, revocation, and no privilege escalation;
- role-derived personal/team queues that project pending actions, blockers,
  separation/quorum obligations, and handoff state without becoming authority;
- always-reachable authority visibility showing who may act, why, under which
  policy/assignment, and whether that authority is active, expired, revoked,
  unavailable, or blocked;
- the external configuration-manager walkthrough from the normative spec,
  including sent, quarantined, verified, signature-valid, authorized, locally
  committed, refused, and outward-mirrored states; and
- keyboard, assistive-technology, narrow-pane, offline, unavailable-adapter,
  expired-delegation, and concurrent-stale-target behavior.

No PLM/PDM UI or adapter state may appear to allocate, approve, issue, or own a
Datum revision. Queue actions resolve to typed operations/proposals through the
canonical mutation path. If visual design is not already owner-approved, this
slice requires a Claude-owned prototype study and explicit owner disposition
before implementation acceptance.

**Exit evidence:** assignment/delegation/refusal tests; queue-to-authority
traceability; separation/quorum and stale-target proofs; signed external
attestation quarantine/admission walkthrough; adapter-failure isolation;
accessibility inspection; standing visual goldens; and running-app owner review.

<!-- REQ:PRODUCT-REVISION-ENGINE:REV-I11 -->
### REV-I11 — Standards witnesses, retention, and migration closure

Implement clause-addressable StandardsProfile, RequirementDisposition,
AuditEvaluation, retention, hold, and disposition evaluation. Ship only profiles
whose exact normative source strength and licensing permit their claimed
obligations. Sequential Alphanumeric remains Datum product policy; an
ISO-oriented profile must keep status/revision separate and must not imply
certification.

Close every REV-GAP migration with machine-readable before/after evidence:

| Gap | Required closure |
|---|---|
| 01–02 | journal tip/fingerprint, parent/shard/integrity/recovery reconciliation |
| 03, 10 | exact release/evidence manifest and reproduction ownership |
| 04 | technical revision creation/resolver migration without identity collapse |
| 05, 11 | remove false journal-ledger/PLM/ECO authority claims and migrate projections |
| 06 | typed GUI technical versus engineering revision wording |
| 07 | readiness check-profile versus atomic Release naming |
| 08 | proposal/review/waiver/deviation equivalence or explicit insufficiency |
| 09 | free-form revision/release-like strings remain non-authoritative labels |
| 12 | generalized dependency/evidence freshness with ZoneFill compatibility |

**Exit evidence:** public/available standards witness; licensed-text-gated
profile refusal; applicable/tailored/equivalent/not-applicable/unresolved
dispositions; independent audit-package reproduction; hold/disposition history;
old-project migration, rollback/read-only fallback, and no silent claim upgrade.

<!-- REQ:PRODUCT-REVISION-ENGINE:REV-I12 -->
### REV-I12 — Production acceptance

Production acceptance is a dedicated bounded step, not a prose declaration. It
requires:

- all REV-I01 through I11 acceptance evidence, including REV-I10A, committed
  and addressable;
- locked/offline guarded workspace tests and strict all-target Clippy;
- all mutation/resolver/dependency/source-health/spec/evidence/parity gates;
- crash, corruption, replay, authorization, impact-unknown, nondeterminism,
  adapter-failure, migration, and accessibility negative tests;
- an end-to-end real-project release performed without Git/network and
  independently restored, verified, reproduced, packaged, and audited;
- a second profile proving stronger roles/status/evidence without a rival model;
- performance/resource budgets measured and owner-dispositioned before becoming
  gates rather than invented in this plan;
- independent findings-first audit against Product Mechanics 034 and the full
  normative specification; and
- explicit owner production acceptance after running-app GUI review.

Closure lands the engine foundation only. It does not automatically authorize
Publish Space implementation, external certification claims, a dependency, or
distributed multi-writer collaboration. The Frontier must explicitly select any
successor.

## 5. Dependency graph

```text
REV-I00 authorization
  -> I01 integrity
  -> I02 typed core
  -> I03 policy/roles/effectivity
  -> I04 EngineeringChange
  -> I05 impact/baseline
  -> I06 atomic release/documents
  -> I07 regeneration/reproduction
  -> I08 offline exchange/Git adapter
  -> I09 API/CLI/MCP parity
  -> I10 approved GUI
  -> I10A enterprise role-workflow UX
  -> I11 standards/migration closure
  -> I12 production acceptance
```

The sequence is intentionally serial at the acceptance boundary. A later
conductor may delegate read-only audits or independent test authoring inside one
selected slice, but dependency independence never authorizes multiple Frontier
steps or parallel Cargo proof builds.

## 6. Standing proof rail

Every execution commit shall run the smallest applicable focused tests and all
affected non-compiling gates. Milestone/closure commits additionally run:

```text
python3 scripts/check_dependency_authority.py
python3 scripts/check_cargo_resource_policy.py
python3 scripts/check_source_health.py
python3 scripts/check_spec_governance.py
python3 scripts/check_spec_parity.py
python3 scripts/check_evidence_traceability.py
python3 scripts/check_progress_coverage.py
python3 scripts/project_status.py check
python3 scripts/project_status.py check-render
python3 scripts/check_schematic_private_writers.py
python3 scripts/check_daemon_write_parity.py
python3 scripts/check_resolver_raw_loads.py
```

Rust proof, Clippy, release, compatibility, and GUI-smoke commands shall run
serially through `scripts/run_cargo_guarded.py --workload proof`. Proof targets
remain disk-backed and must not use `/tmp`. No shared target cleanup occurs while
Cargo/rustc may be active.

## 7. Real-project evidence policy

Proofs shall operate on checked-in real design projects or deterministic working
copies derived from them. Authority scenarios may add revision records and
controlled failure injections around those real projects; they may not replace
the product path with a toy design that avoids actual library, schematic, board,
rules, checks, manufacturing, and Publish dependencies.

Every durable witness records Project identity, source revision/tip, active
policy/profile and version, engine/build identity, command/invocation,
environment assumptions, expected result, actual result, exact output/evidence
digests, and retained logs/artifacts where policy permits.

## 8. Deliberate exclusions

- No Publish Sheet/Viewport/composition implementation.
- No Global Preferences storage or UI implementation inside this program.
- No simultaneous distributed semantic merge or live collaboration protocol.
- No verified redaction/package-variant implementation.
- No automatic certification/accreditation/regulator/customer claim.
- No unratified cryptographic, Git, PDF, network, identity-provider, or standards
  dependency.
- No product topology imposed on scalable Projects.

<!-- EVIDENCE:PRODUCT-REVISION-SPEC:REV-C08-IMPLEMENTATION-PLAN -->
This plan is the REV-C08 decomposition required by Product Mechanics 034. Its
presence on the Frontier is scheduling authority only; `REV-I00` remains the
explicit execution boundary.
