<!-- REQ:PRODUCT-REVISION-ENGINE:REV-I13 -->

# REV-I02 Typed Authority Core Execution-Authorization Packet

> **Status:** owner review required.
>
> **Boundary:** approval authorizes REV-I02 only. It does not authorize Project
> revision policy, business workflows, public mutation surfaces, Preferences,
> Publish, GUI, adapters, enterprise workflow, migration of legacy meanings, or
> any new dependency.

## 1. Findings first

### Finding 1 — REV-I01 supplies the integrity substrate and nothing more

REV-I01 closed with a versioned canonical local store, algorithm-qualified
digests, immutable blobs and generations, atomic head promotion, single-writer
exclusion, last-complete/read-only recovery, and independently verified
backup/restore. Its closure deliberately defines no product authority record
kind and returns the Frontier to authorization `none`.

Evidence: `research/documentation-system/REV_I00_EXECUTION_AUTHORIZATION_PACKET.md:235-275`;
implementation commits `b329b9e` and `cf16a2f`.

### Finding 2 — REV-I02 is a type, persistence, and read-only-resolution slice

The governed plan requires non-interchangeable IDs, canonical records, typed
events, persistence, and exact local resolution for the complete authority
vocabulary before later slices add policy or workflow. REV-I02 therefore makes
the record universe representable and independently resolvable, but it does not
make any lifecycle operational. Project policy, schemes, roles, attestation
evaluation, and effectivity semantics begin in REV-I03; Change behavior begins
in REV-I04; impact and baseline establishment begin in REV-I05; issuance and
document authority begin in REV-I06.

Evidence: `specs/PRODUCT_REVISION_ENGINE_IMPLEMENTATION_PLAN.md:131-175`;
`specs/PRODUCT_REVISION_ENGINE_SPEC.md:76-92,135-238`.

### Finding 3 — the schema must preserve distinctions instead of predicting
later policy

REV-I02 must provide distinct Rust newtypes and tagged references for every
required identity class. It must provide closed, versioned record and event
envelopes whose payloads preserve the ratified distinctions among technical
history, configuration items, baselines, engineering revisions, reservations,
changes, approvals, effectivity, releases, controlled documents, issues,
packages, transmittals, standing facts, standards/audit/records authority, and
the later impact/evidence/reproduction/exchange families. A UUID, label, path,
digest, or untyped string cannot substitute for a typed ID.

This slice may validate structural invariants that are true without policy:
schema/version support, ID kind, record/event identity, Project identity,
canonical form, digest, exact reference existence and kind, immutable-record
body identity, deterministic ordering, and append-only event ancestry. It may
not invent a revision scheme, decide authority, advance a lifecycle, resolve an
effectivity expression, establish a baseline, issue a revision, or declare a
Release.

Evidence: `docs/decisions/PRODUCT_MECHANICS_034_PRODUCT_REVISION_ENGINE.md:30-52,79-87`;
`specs/PRODUCT_REVISION_ENGINE_SPEC.md:27-55,76-92,157-167,207-238`.

### Finding 4 — exact resolution is local, deterministic, and absence-aware

The resolver must read one verified REV-I01 generation, resolve stable typed
IDs independently of labels and paths, follow only exact typed references,
derive projections from the retained event chain, and return typed diagnostics
for unsupported schema, kind substitution, missing or mismatched references,
mutable issued-body attempts, invalid canonical payloads, and integrity
failure. Unknown schema remains preserved and places product-authority
resolution in read-only diagnostic state; it is never rewritten, discarded, or
silently interpreted as a known record.

Resolution must work with no `.git`, Git executable, network, provider, or
remote service. An absent product-authority collection resolves as an explicit
empty/unconfigured result, not as corruption or a reason to modify the Project.

Evidence: `specs/PRODUCT_REVISION_ENGINE_IMPLEMENTATION_PLAN.md:147-154`;
`specs/PRODUCT_REVISION_ENGINE_SPEC.md:313-348,428-446`.

### Finding 5 — PM-034's never-blocks invariant has a concrete code boundary

REV-I02 enforces the exact law that the Revision Engine “never blocks, prompts,
or delays Design authoring” and that “ignoring the revision engine entirely is
a supported, first-class workflow” through all of these code constraints:

1. product-authority types, validation, persistence, and resolution live in
   cohesive `engine::revision` modules and are not imported by ordinary Design
   operation validation or application;
2. Project open, Project creation, and ordinary Design commits do not auto-
   designate a `ConfigurationItem`, auto-create an `EngineeringChange`, allocate
   or reserve a revision identity, or consult a product-authority policy;
3. the absence of product-authority records resolves to `Unconfigured`/empty and
   cannot emit a prompt, warning gate, retry, sleep, network access, or refusal;
4. an ordinary Design commit carries no implicit authority event batch and
   leaves the product-authority record/event head unchanged; and
5. REV-I02 exposes no user-facing product workflow and no public mutation verb.
   Later typed authoring operations must still enter through `commit()` and are
   separately authorized in their assigned slices.

The decisive proof is the **unconfigured-authoring witness**: on deterministic
working copies of `native_authored_baseline_v1`,
`profile-divergence-authored-copper`, and `via-available`, open and perform a
representative ordinary native Design mutation with no product-authority
records. Each mutation must succeed through the unchanged canonical path, emit
no product-authority record or event, require no input, produce no Revision
prompt/refusal, and preserve the expected DesignModel/journal result. A paired
case with inert REV-I02 records but no explicitly adopted earlier-control policy
must produce the same authoring result while leaving those records unchanged.

Evidence: `docs/decisions/PRODUCT_MECHANICS_034_PRODUCT_REVISION_ENGINE.md:173-177`;
`specs/PRODUCT_REVISION_ENGINE_SPEC.md:48-53`; real-Project precedent at
`research/documentation-system/REV_I00_EXECUTION_AUTHORIZATION_PACKET.md:256-265`.

### Finding 6 — source ownership remains bounded and dependency-free

REV-I02 extends the existing `crates/engine/src/revision/` domain with cohesive
modules for IDs/references, record payloads, event envelopes, validation,
canonical authority persistence/indexing, resolver projections, diagnostics,
and focused tests. It may make only narrow REV-I01 store integration changes.
It may not grow the grandfathered substrate monoliths, create a second writer,
or add authority reconstruction to GUI, CLI, daemon, MCP, or adapters.

Serde, UUID, canonical JSON, and SHA-256 support already used by the workspace
and REV-I01 are sufficient. No new crate, system package, service, provider,
signature algorithm, clock source, Git library, database, or network client is
requested. PM-029 remains a hard stop.

Evidence: `specs/PRODUCT_REVISION_ENGINE_IMPLEMENTATION_PLAN.md:26-47,131-154`;
`docs/decisions/PRODUCT_MECHANICS_029_DEPENDENCY_AUTHORITY.md`.

### Finding 7 — the policy gates are green, but one unrelated drift gate is red

At packet head `8808ee9`, evidence traceability, spec governance, project-state
and generated-Frontier parity, dependency authority, source health, Cargo
resource policy, rustfmt, strict workspace Clippy, progress coverage, spec
parity, alignment, and resolver raw-load checks pass. The escalated full drift
suite then fails the GUI agent/terminal convergence guard because typed
authoring handoffs do not explicitly schedule workspace refresh. The failure
reproduces independently and is tracked as `dat-wg1`; this documentation-only
transaction changes none of the implicated GUI or terminal files.

Approval does not waive that failure. A REV-I02 execution claim must begin from
a fresh green drift suite and guarded locked/offline Cargo baseline. If
`dat-wg1` remains red, REV-I02 does not start.

### Finding 8 — approval is deliberately narrow

Approval creates execution authority for REV-I02 only. REV-I03 through REV-I12,
Global Preferences implementation, Publish, GUI, public CLI/MCP/daemon verbs,
enterprise workflow, optional Git or external adapters, and every new dependency
remain unauthorized. REV-I02 completion returns to a fresh Frontier
authorization state and cannot roll directly into REV-I03.

## 2. Exact REV-I02 slice proposed for authorization

REV-I02 may:

1. define versioned, non-interchangeable stable ID newtypes and closed typed
   references for every identity family named by REV-I02;
2. define versioned canonical payloads for ConfigurationItem,
   ConfigurationRef, ConfigurationBaseline, EngineeringRevision,
   RevisionReservation, BuildIdentity, EngineeringChange,
   ApprovalAttestation, Effectivity, ReleaseCandidate, Release,
   SupersessionEstablished, AuthorizationWithdrawn, ObsolescenceDeclared,
   ControlledDocument, DocumentIssue, ReleasePackage, Transmittal,
   StandardsProfile, RequirementDisposition, AuditEvaluation, RetentionPolicy,
   LegalOrPolicyHold, RecordDisposition, DependencySnapshot, SemanticDelta,
   ImpactEvaluation, EvidenceInputContext, EvidenceFreshness,
   LibraryUptakeCandidate, BaselineComparison, RegenerationPlan,
   ReproductionManifest, ReproductionAttempt, ExternalMappingReceipt,
   AdapterDivergenceObservation, ReleaseMirrorRequest, ReleaseMirrorResult,
   AuthorityExchangeEnvelope, AuthorityExchangeReceipt,
   ExternalChangeCandidate, CredentialOrTrustEvent, and
   TrustedTimestampEvidence;
3. define stable record-kind and event-kind tags, immutable record/event
   envelopes, append-only ancestry, and algorithm-qualified canonical digests;
4. persist those envelopes and deterministic indexes inside the REV-I01 local
   authority generation, preserving atomicity, verification, backup, restore,
   and last-complete/read-only recovery;
5. implement structural validation and typed diagnostics for unsupported
   versions, ID-kind substitution, duplicate identities, invalid canonical
   payloads, broken exact references, issued-body mutation, and event-chain
   inconsistency;
6. implement exact local read-only resolution by stable ID, typed reference,
   record kind, and as-of event position, including explicit empty/unconfigured
   and unknown-version read-only results; and
7. add focused constructors and test-only fixtures needed to prove the schema
   and store without exposing a public generic writer or business operation.

**Closure rule.** The families enumerated in item 2 are the complete and
exclusive set REV-I02 may define. A family not named there is deferred to its
assigned slice. Adding a family to REV-I02 after approval requires a new owner
authorization and is not a within-slice implementation detail. REV-I02
completion is verified against this enumeration.

REV-I02 explicitly may not:

- implement revision schemes, Project policy, Global Preferences seeding,
  roles/capability evaluation, attestation authorization, signature/trust/time
  providers, or effectivity evaluation (REV-I03);
- auto-open or advance EngineeringChange, map legacy departures, reserve a
  label, or gate Design authoring (REV-I04);
- compute dependency/impact/evidence freshness, adopt libraries, establish a
  baseline, or regenerate evidence (REV-I05+);
- create a Release, issue an EngineeringRevision or DocumentIssue, prepare a
  package, record a Transmittal, or change standing (REV-I06+);
- add public native-write, daemon, CLI, MCP, proposal, or GUI surfaces (REV-I09+
  and REV-I10+);
- reinterpret or migrate free-form revision labels, existing proposal/review/
  waiver/deviation facts, Part lifecycle, library provenance, ZoneFill
  staleness, the CLI `release` profile, or GUI technical `rev` wording;
- implement Git, exchange admission, PLM/PDM, distributed merge, Publish, or any
  user-visible workflow; or
- add a third-party dependency.

## 3. REV-I02 proof gates

Acceptance requires committed, addressable evidence for:

- an exact inventory proof that the implementation defines every family named
  in section 2 item 2 and defines no additional authority-record family, with
  REV-I02 completion checked against that closed enumeration;
- exhaustive unique wire tags and non-interchangeable Rust ID newtypes for the
  complete REV-I02 identity/record/event inventory;
- compile-fail and runtime type-confusion cases covering representative UUID-
  compatible IDs, wrong tagged references, and wrong record-kind payloads;
- deterministic canonical round trip, stable ordering, and digest goldens for
  every record/event family, including independence from insertion order;
- exact stable-ID resolution after human rename and physical path movement,
  with labels and paths never becoming authority;
- missing, duplicate, dangling, cyclic-where-forbidden, wrong-kind, malformed,
  non-canonical, and mutable-issued-body refusals with stable diagnostic codes;
- unknown record/event schema and unknown kind preservation with read-only
  diagnostic resolution, proving bytes survive backup/restore unchanged;
- append-only as-of resolution and derived standing projection without rewriting
  issued records or treating `Current`/`Historical` as stored lifecycle states;
- REV-I01 atomicity, corruption recovery, backup/restore, single-writer, and
  integrity verification remaining green with authority records present;
- exact local resolution with `.git`, Git executables, network, providers, and
  services absent;
- the unconfigured-authoring witness from Finding 5 on all three named real
  Projects, plus the paired inert-record/no-policy case;
- a source-boundary proof that ordinary Design validation/application neither
  imports the product-authority resolver nor emits product-authority events;
- no private writer and no public generic JSON patch; and
- all standing dependency, Cargo-resource, source-health, specification,
  evidence, project-state, private-writer, daemon-parity, resolver-fence, native-
  Project, guarded locked/offline workspace, and strict all-target Clippy gates.

The full guarded locked/offline suite must pass once without retry. Results in
the REV-I01 closure are baseline evidence, not an execution lease or waiver.

## 4. Dependency, migration, and visual posture

- **Dependency:** none requested. Any new crate, provider, executable, service,
  or algorithm choice stops at PM-029 before code changes.
- **Migration:** REV-I02 introduces a versioned product-authority namespace over
  the REV-I01 store. Absence means unconfigured. Unknown future versions and
  kinds are preserved and resolved read-only. No existing free-form or narrow
  lifecycle field is promoted, guessed, rewritten, or deleted; all semantic
  migrations remain assigned to REV-I04/REV-I11.
- **Visual:** no product surface changes. Typed diagnostics may flow through
  existing diagnostic channels only. If implementation implies any new visible
  recovery, migration, unknown-record, or authority state, work pauses for a
  bounded Claude-owned render before UI behavior or clauses are added.
- **Authority:** record presence alone grants no policy, role, approval,
  effectivity, lifecycle, release, or Design-gating power. No external adapter
  state becomes Datum authority.

## 5. Owner boundary

REV-I13 is the next free integer step and owns this authorization boundary;
REV-I02 remains the existing execution-slice identity. No sub-lettered step is
created.

<!-- OWNER:PRODUCT-REVISION-ENGINE:REV-I13:REV-I02-PACKET -->
<!-- EVIDENCE:PRODUCT-REVISION-ENGINE:REV-I02-PACKET -->

The exact question is:

> Does this packet preserve Product Mechanics 034's identity distinctions,
> one-mutation-path law, standalone authority, and exact never-blocks/first-class
> unmanaged workflow while bounding REV-I02 to typed schema, canonical local
> persistence, structural validation, and read-only resolution only?

**Recommended response:** approve only if REV-I02 remains an inert typed
authority substrate: no policy, workflow, issuance, public mutation surface,
Design gate, visible behavior, adapter, migration guess, or dependency.

**Exact response format:**

```text
REV-I02-EXECUTION: approve
```

or

```text
REV-I02-EXECUTION: revise — <specific scope, type, invariant, proof, dependency, migration, or visual correction>
```

## 6. Owner disposition

<!-- EVIDENCE:PRODUCT-REVISION-ENGINE:REV-I13-APPROVED -->

The owner approved REV-I13 on 2026-08-29 with the exact response
`REV-I02-EXECUTION: approve`. The authorization is limited to REV-I02 as closed
by sections 2–4, including the exclusive family enumeration and its completion
proof. Finding 5's never-blocks enforcement and unconfigured-authoring witness
remain controlling. REV-I03 through REV-I12, public mutation surfaces,
Preferences, Publish, GUI, adapters, semantic migration, visible behavior, and
every new dependency remain unauthorized. Approval does not waive `dat-wg1`:
REV-I02 may not be claimed or started until a fresh drift suite and guarded
locked/offline Cargo baseline are green.
