<!-- REQ:PRODUCT-REVISION-ENGINE:REV-I14 -->

# REV-I03 Project Policy and Authority Evaluation Execution-Authorization Packet

> **Status:** owner-approved; REV-I03 execution authorized.
>
> **Boundary:** approval authorizes REV-I03 only. It does not authorize an
> EngineeringChange lifecycle, a Design-authoring gate, revision reservation or
> issuance, baseline establishment, Release, public CLI/MCP/daemon or GUI
> surfaces, Global Preferences implementation, adapters, semantic migration, a
> cryptographic provider, or any new dependency.

## 1. Findings first

### Finding 1 — REV-I02 supplies inert typed authority, not policy

REV-I02 landed the complete exclusive 43-family authority inventory, canonical
records/events, structural validation, REV-I01-integrated persistence, and exact
local read-only resolution in commit `d274485`. Its resolver has an explicit
`Unconfigured` result when no authority snapshot exists, and ordinary Design
commits only carry an existing authority snapshot forward. REV-I02 deliberately
defines no Project revision policy, scheme semantics, actor/role authority,
attestation evaluation, effectivity resolution, or product workflow.

Evidence: `crates/engine/src/revision/authority.rs:20-114,334-517`;
`crates/engine/src/revision/resolver.rs:11-22,24-121`; REV-I02 closure commit
`661a7c3`.

### Finding 2 — REV-I03 is policy and authorization evaluation, not lifecycle

The governed plan assigns REV-I03 the explicit resolved Project-policy input,
Project-owned policy versions, revision-scheme registry, scoped actors and
roles, approval attestations, effectivity, and the copy-once Global Preferences
seam. EngineeringChange behavior and any earlier-control Design refusal begin in
REV-I04; dependency/impact/baseline authority begins in REV-I05; allocation,
issuance, Release, document authority, and standing begin in REV-I06; public
surface parity begins in REV-I09; GUI and enterprise workflow presentation begin
in REV-I10 and REV-I10A.

Evidence: `specs/PRODUCT_REVISION_ENGINE_IMPLEMENTATION_PLAN.md:156-205`;
`specs/PRODUCT_REVISION_ENGINE_SPEC.md:111-133,180-205`.

### Finding 3 — the policy input is Project-owned and absence-aware

The engine accepts one explicit resolved Project policy. A copied policy is
versioned Project authority; later factory, user, organization, package, or
Preference changes cannot rewrite it. The only authorized preference crossing
is a frozen `ProjectPolicySeed` snapshot consumed once by Project mutation
authority with a durable itemized receipt. Presentation, Capability, and
WorkflowDefault descriptors never cross that seam.

The registered revision seeds are exactly `datum.revision.profile_seed`,
`datum.revision.build_presentation_seed`, and
`datum.revision.prototype_transition_seed`. REV-I03 may consume a frozen test or
engine input with those three keys; it may not implement the PreferenceRepository,
preference resolution, providers, synchronization, the Preferences window, or a
live link. A Project with no adopted policy resolves as `Unmanaged`, not as
corrupt and not as though Datum had silently persisted the factory profile.

Evidence: `docs/decisions/PRODUCT_MECHANICS_037_GLOBAL_PREFERENCES_ENGINE.md:17-35,54-70`;
`specs/GLOBAL_PREFERENCES_V1_DESCRIPTOR_CATALOG.md:166-185`;
`research/preferences-system/GP_C03_PROJECT_SEED_AND_CONTEXT_DECISION_PACKET.md:141-173`;
`research/preferences-system/GP_C04_STORAGE_MIGRATION_EXCHANGE_RECOVERY_CONTRACT.md:378-390`.

### Finding 4 — scheme and profile distinctions are already ratified

V1 has one registry and exactly five scheme kinds: `LinearAlphabetic`,
`LinearNumeric`, `IssueRevision`, `Iso19650InformationContainer`, and
`OrganizationCustom`. `SequentialAlphanumericLegacy` is the factory new-Project
profile, while ISO-oriented policy keeps revision and suitability/status as
separate axes. A custom scheme is editioned and deterministic; it is not free
text, executable code, a plugin, or permission to reinterpret an issued label.
Every revision namespace belongs to one ConfigurationItem, and historical
records retain the policy/scheme version that interpreted them.

REV-I03 may validate registry and namespace rules, but it cannot reserve or
allocate a token. Those behaviors remain with REV-I04 and REV-I06.

Evidence: `docs/decisions/PRODUCT_MECHANICS_034_PRODUCT_REVISION_ENGINE.md:40-58`;
`research/documentation-system/REV_C03_REVISION_IDENTITY_DECISION_PACKET.md:118-158,290-387`;
`specs/PRODUCT_REVISION_ENGINE_SPEC.md:111-133,139-153`.

### Finding 5 — authority is capability, scope, interval, and policy

The complete core capability set is Author, ChangeCoordinator, Reviewer,
Verifier, ConfigurationAuthority, ReleaseAuthority, Auditor, and
RecordsAuthority. An ActorIdentity names a person, service, organization, or
controlled-automation principal. RoleAssignment and bounded RoleDelegation bind
capabilities to an exact Project or typed authority-record scope, effective
interval, source, and lineage. Display role names and a valid credential confer
no authority. Delegation cannot expand the delegator's active capability or
scope.

The complete attestation-intent set is Approve, Reject, Acknowledge, Verify,
Authorize, and Release. Authorization evaluates capability, exact scope,
effective interval, target and digest, method requirement, policy version,
order, quorum, independence, and separation of duty. Supersession and revocation
append facts; they never edit the attestation being corrected.

Evidence: `specs/PRODUCT_REVISION_ENGINE_SPEC.md:180-200`;
`research/documentation-system/PRODUCT_REVISION_ENGINE_AUTHORITY_MODEL.md:262-284,491-513`.

### Finding 6 — effectivity must refuse uncertainty

REV-I03 may define a typed predicate tree with exactly four node kinds—Selector,
AllOf, AnyOf, and Not—and exactly eleven selector families: ProductOrAssembly,
ConfigurationItem, Variant, BoardOrSubassembly, SerialRange, LotOrBatch, Build,
Customer, Site, Contract, and Date. Resolution uses one explicit context and may
retain an exact resolved-population snapshot when enumerable. Unsupported,
ambiguous, unknown, or unresolvable selectors refuse; none means universal.

This slice evaluates effectivity only. It does not attach effectivity to a new
Change, Release, revision, departure, or standing operation assigned later.

Evidence: `specs/PRODUCT_REVISION_ENGINE_SPEC.md:202-205`;
`research/documentation-system/PRODUCT_REVISION_ENGINE_AUTHORITY_MODEL.md:286-305`.

### Finding 7 — PM-034's never-blocks law now requires an explicit code fence

REV-I03 introduces actual policy, so absence alone no longer protects ordinary
authoring. The exact controlling law remains: under a profile that has not
explicitly adopted earlier control, the Revision Engine “never blocks, prompts,
or delays Design authoring,” and “ignoring the revision engine entirely is a
supported, first-class workflow.” REV-I03 enforces it as follows:

1. `resolve_project_revision_policy` returns `Unmanaged` for no adopted policy;
   it does not synthesize, persist, or auto-adopt the factory profile;
2. the policy field controlling pre-authoring behavior has exactly two values,
   `NoEarlierControl` and `AuthorizedChangeRequired`, with
   `NoEarlierControl` in the factory copied profile;
3. REV-I03 policy, role, approval, and effectivity evaluation is called only by
   revision-authority commands and pure queries in this slice;
4. ordinary Design operation validation/application, Project open, and native
   commit do not import or invoke the Project-policy evaluator, perform a role
   check, create an attestation, emit an authority event, prompt, retry, sleep,
   or access a provider; and
5. `AuthorizedChangeRequired` is representable but has no Design-mutation effect
   until REV-I04 separately authorizes and proves the one canonical integration.

The decisive proof is the **no-adopted-policy authoring witness**. On working
copies of `native_authored_baseline_v1`,
`profile-divergence-authored-copper`, and `via-available`, a representative
ordinary native Design mutation must succeed through the unchanged commit path
with no Project revision policy, no prompt/refusal/input, no policy lookup, no
authority event, and the expected DesignModel/journal result. Paired runs with
an adopted `NoEarlierControl` policy must remain behaviorally identical and
leave policy/authority records unchanged. A source-boundary test must prove that
ordinary Design validation/application cannot call the REV-I03 policy or
authorization evaluator.

Evidence: `docs/decisions/PRODUCT_MECHANICS_034_PRODUCT_REVISION_ENGINE.md:173-177`;
`specs/PRODUCT_REVISION_ENGINE_SPEC.md:48-53`;
`specs/PRODUCT_REVISION_ENGINE_IMPLEMENTATION_PLAN.md:198-201`.

### Finding 8 — no dependency or visual authority is needed

The existing serde, UUID, canonical JSON, SHA-256, REV-I01 store, and REV-I02
authority modules are sufficient. REV-I03 needs no clock provider, signature
algorithm, key store, certificate library, database, network client, Git
library, service, or executable. Signature validity versus authorization can be
proved with a private test-only provider; production cryptographic verification
remains unavailable until an exact PM-029 decision and later authorization.

REV-I03 adds no product surface. If implementation implies a visible policy,
role, approval, effectivity, seed-receipt, migration, or refusal behavior, work
pauses for a bounded Claude-owned render before clauses or UI are added.

Evidence: `docs/decisions/PRODUCT_MECHANICS_029_DEPENDENCY_AUTHORITY.md:5-13,29-60`;
`specs/PRODUCT_REVISION_ENGINE_IMPLEMENTATION_PLAN.md:171-178`.

### Finding 9 — approval remains one slice only

Approval creates execution authority for REV-I03 only. REV-I04 through REV-I12,
Global Preferences implementation, public operations, Publish, GUI, enterprise
workflow surfaces, adapters, semantic migration, cryptographic production
methods, and every new dependency remain unauthorized. REV-I03 completion
returns to a fresh Frontier authorization state and cannot roll directly into
REV-I04.

## 2. Exact REV-I03 slice proposed for authorization

REV-I03 may:

1. add exactly six authority-record families to the REV-I02 inventory:
   `ProjectRevisionPolicy`, `RevisionScheme`, `ActorIdentity`, `RoleAssignment`,
   `RoleDelegation`, and `ProjectSeedReceipt`;
2. define exactly five `RevisionSchemeKind` values: `LinearAlphabetic`,
   `LinearNumeric`, `IssueRevision`, `Iso19650InformationContainer`, and
   `OrganizationCustom`; define deterministic versioned scheme entries,
   per-ConfigurationItem namespaces, immutable in-use scheme versions, separate
   ISO revision/status axes, and successor-version rather than reinterpretation;
3. define exactly four `ActorKind` values—`Person`, `Service`, `Organization`,
   and `ControlledAutomation`—and exactly eight capabilities—`Author`,
   `ChangeCoordinator`, `Reviewer`, `Verifier`, `ConfigurationAuthority`,
   `ReleaseAuthority`, `Auditor`, and `RecordsAuthority`;
4. define RoleAssignment and RoleDelegation over an exact Project scope or one
   typed `AuthorityRef` scope, effective intervals, provenance, rationale,
   supersession/revocation lineage, and non-expansion validation;
5. define exactly six attestation intents—`Approve`, `Reject`, `Acknowledge`,
   `Verify`, `Authorize`, and `Release`—plus exactly three later dispositions:
   `Active`, `Superseded`, and `Revoked`; evaluate exact target digest, actor,
   role assignment, policy version, method, time input, quorum, order,
   independence, and separation without equating credential validity with
   authorization;
6. define exactly four effectivity-expression nodes—`Selector`, `AllOf`,
   `AnyOf`, and `Not`—and exactly eleven selector families—`ProductOrAssembly`,
   `ConfigurationItem`, `Variant`, `BoardOrSubassembly`, `SerialRange`,
   `LotOrBatch`, `Build`, `Customer`, `Site`, `Contract`, and `Date`—with an
   explicit resolution context and optional exact resolved-population snapshot;
7. define versioned `ProjectRevisionPolicy` with exactly these policy sections:
   scheme/profile selection; per-CI namespace rules; earlier-control mode
   (`NoEarlierControl` or `AuthorizedChangeRequired`); build presentation
   (`Quiet` or `PhaseBuild`); namespace transition (`Continuous` or
   `GovernedProductionIdentity`); approval intents/roles/quorum/order/
   independence/separation; effectivity obligations; controlled terminology;
   and required method classes. These sections configure one engine and cannot
   redefine core record or fact meanings;
8. implement exactly these internal mutations through the canonical revision
   transaction/store path: `AdoptProjectRevisionPolicy`,
   `SupersedeProjectRevisionPolicy`, `RegisterRevisionScheme`,
   `SupersedeRevisionScheme`, `RegisterActorIdentity`, `AssignScopedRole`,
   `RevokeScopedRole`, `DelegateScopedRole`, `RevokeRoleDelegation`,
   `RecordApprovalAttestation`, `SupersedeApprovalAttestation`,
   `RevokeApprovalAttestation`, `DefineEffectivity`, `SupersedeEffectivity`, and
   `ConsumeProjectRevisionSeedSnapshot`;
9. implement exactly these pure internal query/evaluation operations:
   `ResolveProjectRevisionPolicy`, `ResolveRevisionScheme`,
   `QueryEffectiveRoleAssignments`, `EvaluateCapability`,
   `EvaluateApprovalPolicy`, `QueryAttestationStanding`,
   `ResolveEffectivity`, and `QueryProjectSeedReceipt`;
10. consume only the frozen revision-seed keys
    `datum.revision.profile_seed`, `datum.revision.build_presentation_seed`, and
    `datum.revision.prototype_transition_seed`; atomically create the copied
    Project policy and durable itemized receipt without reading or writing a
    PreferenceRepository or following later source changes;
11. persist the added records and events inside the REV-I01/REV-I02 canonical
    authority generation, retaining deterministic encoding, integrity,
    single-writer exclusion, atomic promotion, backup/restore, unknown-data
    preservation, and read-only recovery; and
12. add cohesive `engine::revision` policy/scheme/role/approval/effectivity/seed
    modules, internal transaction integration, and test-only fixtures/providers
    needed to prove this closed slice without exposing a public generic writer.

**Closure rule.** Every family, kind, capability, intent, disposition, selector,
policy section, mutation, query, and seed key enumerated in items 1–10 is the
complete and exclusive set REV-I03 may define or make operational. An item not
named there is deferred to its assigned slice. Adding or substituting an item
after approval requires a new owner authorization and is not a within-slice
implementation detail. REV-I03 completion is verified bidirectionally against
every enumeration.

REV-I03 explicitly may not:

- auto-open or advance EngineeringChange, collect divergence, map departures,
  reserve/allocate an identity, or enforce earlier control on Design mutations
  (REV-I04+);
- compute dependency/impact/evidence, establish a baseline, issue an
  EngineeringRevision, create a Release/DocumentIssue/package/transmittal, or
  change standing (REV-I05+ and REV-I06+);
- add public native-write, proposal, daemon, CLI, MCP, script, or GUI surfaces
  (REV-I09+ and REV-I10+);
- implement the PreferenceRepository, preference resolver, Preferences or Start
  page, providers, organization management, synchronization, or live Project
  following; GP-I01 through GP-I05 retain those responsibilities;
- implement enterprise assignment/delegation UI, role-derived queues, external
  attestation exchange, Git, PLM/PDM, or any adapter (REV-I08/REV-I10A);
- select or implement a production signature algorithm, key/certificate store,
  trusted-time source, identity provider, or credential verifier;
- reinterpret or migrate free-form revision labels, review booleans, proposals,
  waivers/deviations, Part lifecycle, library provenance, ZoneFill staleness,
  CLI `release` naming, title blocks, or GUI technical `rev` wording;
- create a prompt, delay, refusal, policy lookup, role lookup, or authority event
  on ordinary Design authoring; or
- add a third-party dependency.

## 3. REV-I03 proof gates

Acceptance requires committed, addressable evidence for:

- one bidirectional inventory proof showing every item in every section 2
  enumeration exists exactly as authorized and no additional REV-I03 record
  family, kind, capability, intent, disposition, selector, policy section,
  mutation, query, or seed key exists;
- unique stable tags, versioned canonical round trips, insertion-independent
  ordering, digest goldens, type-confusion refusals, and backup/restore parity for
  all six new record families and their events;
- all five scheme kinds, per-CI namespace isolation, duplicate/exhausted/
  ambiguous-sequence refusal, separate ISO revision/status axes, immutable
  referenced scheme versions, and no token reservation/allocation;
- factory `SequentialAlphanumericLegacy` policy resolution and explicit
  `Unmanaged` resolution without synthesizing or storing a default policy;
- lightweight authority where one actor holds multiple capabilities, and a
  regulated flow where distinct actors satisfy ordered quorum, independence,
  and separation requirements;
- expired, not-yet-effective, revoked, superseded, wrong-scope, and delegated-
  beyond-authority refusals with stable diagnostic codes;
- a valid synthetic signature from a test-only provider remaining unauthorized
  when role/scope/policy fails, and an authorized actor remaining distinct from
  cryptographic validity;
- exact-target changed-digest invalidation, attestation supersession and
  revocation without issued-body mutation, and deterministic standing queries;
- all four effectivity nodes and eleven selector families, exact enumerable
  population snapshots, and unsupported/ambiguous/unknown/unresolvable refusal
  without universal fallback;
- seed capture/consumption for exactly the three registered revision keys,
  atomic copied policy plus durable itemized receipt, concurrent-source
  independence, and an existing Project remaining byte-for-byte stable after a
  later global/frozen-seed change;
- the Finding 7 no-adopted-policy authoring witness on all three named real
  Projects, its paired adopted-`NoEarlierControl` runs, and a source-boundary
  proof that ordinary Design validation/application cannot invoke REV-I03 policy
  or authorization evaluation;
- exact offline operation with `.git`, Git executable, network, provider,
  service, and production cryptography absent;
- REV-I01/REV-I02 integrity, atomicity, corruption recovery, unknown
  preservation, backup/restore, single-writer, structural-validation, exact
  resolution, and inert-authoring proofs remaining green;
- no private writer, no public generic JSON patch, and no public product surface;
  and
- all standing dependency, Cargo-resource, source-health, specification,
  evidence, project-state, private-writer, daemon-parity, resolver-fence,
  native-Project, drift, guarded locked/offline workspace, and strict all-target
  Clippy gates.

The full guarded locked/offline suite must pass once without retry. REV-I02
closure results are baseline evidence, not an execution lease or waiver.

## 4. Dependency, migration, and visual posture

- **Dependency:** none requested. Any new crate, provider, executable, service,
  signature/trust/time algorithm, or other dependency stops at PM-029 before
  code changes.
- **Migration:** no existing Project is backfilled or guessed into policy.
  Absence remains `Unmanaged`. Only an explicit new-Project frozen seed or a
  deliberate Project policy mutation creates policy. Existing labels, reviews,
  proposals, waivers/deviations, and other narrow facts retain their current
  meaning until REV-I04/REV-I11 migration work.
- **Visual:** no product surface changes. If policy, role, attestation,
  effectivity, receipt, refusal, or migration implies visible behavior, REV-I03
  pauses for a bounded Claude-owned render before any UI behavior or clauses are
  added. Codex does not edit the prototype lane.
- **Authority:** an adopted policy configures only the enumerated semantics.
  Record presence, display terminology, signature validity, external state, or a
  preference source does not confer capability or permit Design gating.

## 5. Owner boundary

REV-I14 is the next free integer step and owns this authorization boundary;
REV-I03 remains the existing execution-slice identity. No sub-lettered step is
created.

<!-- OWNER:PRODUCT-REVISION-ENGINE:REV-I14:REV-I03-PACKET -->
<!-- EVIDENCE:PRODUCT-REVISION-ENGINE:REV-I03-PACKET -->

The exact question is:

> Does this packet preserve Product Mechanics 034's Project-owned policy,
> capability/scope authorization, exact approval/effectivity distinctions, and
> never-blocks/first-class unmanaged workflow while bounding REV-I03 to the
> closed policy, scheme, role, attestation, effectivity, and copy-once seed
> inventories only?

**Recommended response:** approve only if REV-I03 remains a dependency-free,
offline policy and authority-evaluation slice; a Project with no adopted policy
authors without prompt, gate, or delay; earlier-control enforcement, lifecycle,
issuance, public surfaces, Preferences implementation, adapters, production
cryptography, and migrations remain deferred.

**Exact response format:**

```text
REV-I03-EXECUTION: approve
```

or

```text
REV-I03-EXECUTION: revise — <specific scope, enumeration, policy, authority, never-blocks, proof, dependency, migration, or visual correction>
```

## 6. Owner disposition

<!-- EVIDENCE:PRODUCT-REVISION-ENGINE:REV-I14-APPROVED -->

The owner approved REV-I14 on 2026-08-29 with the exact response
`REV-I03-EXECUTION: approve`. Authorization is limited to REV-I03 as closed by
sections 2 through 4, including every complete-and-exclusive enumeration and
its bidirectional completion proof. Finding 7's no-adopted-policy and
`NoEarlierControl` code fence and three-real-Project authoring witness remain
controlling.

REV-I04 through REV-I12, Design-gate integration, public surfaces, Global
Preferences implementation, adapters, production cryptography, semantic
migration, visible behavior, and every new dependency remain unauthorized.
REV-I03 begins only after fresh drift and guarded locked/offline Cargo
baselines; prior packet results are evidence, not an execution lease.
