<!-- REQ:PRODUCT-REVISION-ENGINE:REV-I15 -->

# REV-I04 EngineeringChange and Bounded-Departure Execution-Authorization Packet

> **Status:** owner review required.
>
> **Boundary:** approval authorizes REV-I04 only. It does not authorize impact
> analysis, baseline establishment, issuance, public revision operations,
> Preferences, Publish, GUI, adapters, enterprise workflow, production
> cryptography, or any new dependency.

## 1. Findings first

### Finding 1 — REV-I03 supplies policy and evaluation, but deliberately no Design gate

REV-I03 closed with Project-owned revision policy, scheme resolution, scoped
roles and delegation, approval attestations, effectivity, and copy-once seeding.
It deliberately left `AuthorizedChangeRequired` representable but inert and
returned the Frontier to authorization `none`. REV-I04 is therefore the first
slice that may connect explicitly adopted earlier control to a Design commit;
that authority must not leak into ordinary operation validation.

Evidence: `research/documentation-system/REV_I03_EXECUTION_AUTHORIZATION_PACKET.md:118-149,381-396`;
implementation commit `3ae3dea`.

### Finding 2 — one lifecycle means one closed state and transition vocabulary

Datum owns one `EngineeringChange`, not separate ECR/ECO/DCO objects. Its
append-only projected states are exactly `Draft`, `ImpactReview`, `Authorized`,
`Implementing`, `Verification`, `Closed`, `Rejected`, `Deferred`, and
`Cancelled`. Profiles may rename those stages; REV-I04 does not add arbitrary
states or a second lifecycle. Authorization does not prove implementation,
verification does not issue, and closure requires either exact released-
successor evidence or an explicit no-release disposition. Because issuance is
REV-I06, REV-I04 can prove the no-release closure path and represent—but cannot
create—the later released-successor reference.

Evidence: `docs/decisions/PRODUCT_MECHANICS_034_PRODUCT_REVISION_ENGINE.md:62-71`;
`specs/PRODUCT_REVISION_ENGINE_SPEC.md:155-172`;
`research/documentation-system/PRODUCT_REVISION_ENGINE_AUTHORITY_MODEL.md:428-449`.

### Finding 3 — identity-free collection is an atomic consequence, never allocation

For a Project under `NoEarlierControl`, the first committed divergence from an
exact released/baselined predecessor may quietly create one Draft Change and
link the accepted transaction. Later edits collect into that same open
successor work unless an explicit split, merge, or reassignment says otherwise.
The collection step creates no `RevisionReservation`, `EngineeringRevision`,
baseline, approval, `Release`, or `DocumentIssue`.

An explicit `ReserveRevisionLabel` mutation is the only REV-I04 action that may
create `RevisionReservation`. It must commit through the revision authority
store with an exact CI namespace, scheme/version, proposed label, governing
Change, and expiry input. Until that commit succeeds, no label is reserved and
no reservation or revision-identity authority event exists. A reservation
remains provisional and cannot become issued or title-block truth in this slice.

A literal prohibition on every `AuthorityEvent` during collection would
contradict the ratified requirement that the first divergence atomically opens
an append-only Draft `EngineeringChange` and links its transaction. The bounded
invariant is therefore exact: collection may append only the Change `Created`
and `ImplementationTransactionLinked` events; it emits no reservation,
allocation, issuance, approval, baseline, Release, or document authority event.
The explicit `RevisionReservation` commit is the first event that may bind a
revision label.

Evidence: `docs/decisions/PRODUCT_MECHANICS_034_PRODUCT_REVISION_ENGINE.md:73-77`;
`specs/PRODUCT_REVISION_ENGINE_SPEC.md:149-153,174-178`;
`research/documentation-system/PRODUCT_REVISION_ENGINE_AUTHORITY_MODEL.md:209-214,922-942`.

### Finding 4 — the canonical Design integration is singular and source-bounded

REV-I04 authorizes exactly one integration point:
`DesignModel::commit_journaled_with_links_and_inverse` invokes one
`prepare_revision_design_commit` preflight after the existing technical guards
and before inverse calculation or any shard, journal, or authority staging. The
preflight returns one closed `RevisionDesignCommitPlan`:
`UnmanagedPassThrough`, `NoEarlierControlCollect`, or
`AuthorizedChangeValidated`.

The coordinator applies that plan in the same staged integrity generation as
the Design transaction. It is the only code permitted to turn a Design commit
into an EngineeringChange transaction link, quietly create successor work, or
return `ChangeNotAuthorized`. `Unmanaged` and `NoEarlierControl` cannot refuse,
prompt, retry, sleep, consult a provider, or delay on missing revision data.
`AuthorizedChangeRequired` may refuse only when the commit supplies no exact
governing Change or that Change is not currently authorized for the affected
scope under the already-ratified REV-I03 policy/capability evaluation.

The ordinary operation validators and applicators remain revision-unaware;
`DesignModel::commit`, Project open, resolver assembly, and native-write batch
construction do not gain policy or capability lookups. Normal journaled commit,
accepted-proposal apply, undo, and redo all traverse the one coordinator, so
there is no second gate or bypass.

Evidence: `docs/decisions/PRODUCT_MECHANICS_034_PRODUCT_REVISION_ENGINE.md:173-177`;
`specs/PRODUCT_REVISION_ENGINE_IMPLEMENTATION_PLAN.md:198-206`;
current coordinator boundary at `crates/engine/src/substrate/commit.rs:90-159,179-248`.

### Finding 5 — PM-034 remains first-class unmanaged, not merely permissive

The controlling invariant remains verbatim: the Revision Engine “never blocks,
prompts, or delays Design authoring” under any profile without explicitly
adopted earlier control, and “ignoring the revision engine entirely is a
supported, first-class workflow.” REV-I04 enforces that law in code:

1. no adopted policy resolves to `UnmanagedPassThrough` without synthesis,
   persistence, auto-adoption, authority event, or Change creation;
2. adopted `NoEarlierControl` always permits the Design commit; missing or
   inapplicable predecessor authority means no collection, never refusal;
3. quiet collection is a post-authorization atomic accompaniment to the same
   commit, not a pre-authoring question or separate user action;
4. only an explicitly adopted `AuthorizedChangeRequired` policy can select the
   validating branch; and
5. that branch returns one typed refusal before staging and never partially
   writes Design or authority state.

The decisive proof repeats the no-adopted-policy authoring witness on
`native_authored_baseline_v1`, `profile-divergence-authored-copper`, and
`via-available`, paired with `NoEarlierControl` runs. Results must remain
behaviorally identical: no prompt, gate, delay, policy synthesis, or authority
event where no successor collection is applicable. A source-boundary test must
prove that ordinary Design validation/application cannot import or invoke any
revision-policy or capability evaluator and that the sole permitted call edge
is the named journaled coordinator preflight.

Evidence: `docs/decisions/PRODUCT_MECHANICS_034_PRODUCT_REVISION_ENGINE.md:173-177`;
`research/documentation-system/REV_I03_EXECUTION_AUTHORIZATION_PACKET.md:118-153`.

### Finding 6 — affected-item accounting is bounded before impact computation

REV-I04 may record exact affected authority references and the four ratified
actions `Add`, `Modify`, `Retire`, and `NoChange`, plus before/proposed-after
references and an optional already-resolved effectivity reference. It may not
compute dependency propagation, semantic impact, freshness, library uptake, or
baseline comparison. Those remain REV-I05. An affected item therefore records
declared scope and accounting in this slice; it cannot assert a machine-derived
`Affected` or `Unaffected` conclusion without the later witness engine.

Evidence: `research/documentation-system/PRODUCT_REVISION_ENGINE_AUTHORITY_MODEL.md:224-260`;
`specs/PRODUCT_REVISION_ENGINE_IMPLEMENTATION_PLAN.md:183-206,208-228`.

### Finding 7 — legacy facts are assessed explicitly and never promoted by resemblance

Proposal acceptance remains proposal authority. A review boolean or free-form
actor cannot satisfy controlled approval. Existing `CheckWaiver` and accepted
`CheckDeviation` facts remain distinct and can map to bounded departure
authority only when exact requirement/finding, scope, authorizing authority,
validity, effectivity, and disposition are available. The original fact is never
rewritten or deleted.

Each assessed source receives exactly one `Mapped`, `RetainedAsLegacyEvidence`,
or `RejectedAsInsufficient` disposition in an append-only
`LegacyRevisionFactMapping`. Mapping never creates an attestation or advances a
Change lifecycle. A mapped waiver and a mapped deviation remain different
record families, and an insufficient source remains honest legacy evidence
rather than becoming an implicit approval.

Evidence: `specs/PRODUCT_REVISION_ENGINE_IMPLEMENTATION_PLAN.md:188-196`;
`specs/PRODUCT_REVISION_ENGINE_SPEC.md:169-172,515-517`;
`research/documentation-system/PRODUCT_REVISION_ENGINE_AUTHORITY_MODEL.md:601-608,684-687`.

### Finding 8 — approval remains one dependency-free engine slice

Approval creates execution authority for REV-I04 only. REV-I05 through REV-I12,
public revision verbs and queries, Preferences, Publish, GUI, adapters,
enterprise workflow surfaces, production cryptography, standards migration,
and every new dependency remain unauthorized. REV-I04 completion returns to a
fresh Frontier authorization state and cannot roll directly into REV-I05.

Existing workspace crates are sufficient. No crate, provider, executable,
service, signature/trust/time algorithm, database, Git library, or network path
is requested. Caller-supplied deterministic time values used for reservation or
validity evaluation do not select a trusted-time mechanism.

Evidence: `docs/decisions/PRODUCT_MECHANICS_029_DEPENDENCY_AUTHORITY.md:5-13,29-60`;
`specs/PRODUCT_REVISION_ENGINE_IMPLEMENTATION_PLAN.md:26-32,180-206`.

## 2. Exact REV-I04 slice proposed for authorization

REV-I04 may:

1. make semantic payloads operational for exactly the existing REV-I02
   `EngineeringChange` and `RevisionReservation` authority-record families, and
   add exactly three authority-record families:
   `WaiverDeparture`, `DeviationDeparture`, and
   `LegacyRevisionFactMapping`;
2. define exactly nine `EngineeringChangeState` values: `Draft`,
   `ImpactReview`, `Authorized`, `Implementing`, `Verification`, `Closed`,
   `Rejected`, `Deferred`, and `Cancelled`;
3. define exactly fourteen `EngineeringChangeEventKind` values: `Created`,
   `RationaleAndClassificationSet`, `AffectedItemUpserted`,
   `EffectivitySet`, `ImplementationTransactionLinked`,
   `SubmittedForImpactReview`, `Authorized`, `ImplementationBegan`,
   `ImplementationVerificationRecorded`, `Closed`, `Rejected`, `Deferred`,
   `Cancelled`, and `ReworkRequested`; derive state only from the append-only
   event chain and retain every prior event;
4. permit exactly these base lifecycle transitions:
   `Draft -> ImpactReview`, `ImpactReview -> Authorized`,
   `Authorized -> Implementing`, `Implementing -> Verification`, and
   `Verification -> Closed`; permit `Rejected`, `Deferred`, or `Cancelled` only
   from a nonterminal state; permit `ReworkRequested` only from `ImpactReview`
   to `Draft`, `Authorized` to `ImpactReview`, `Implementing` to `Authorized`, or
   `Verification` to `Implementing`; define no profile shortcut in this slice;
5. define exactly four `AffectedItemAction` values: `Add`, `Modify`, `Retire`,
   and `NoChange`, with exact typed authority reference, optional exact before
   and proposed-after references, optional already-resolved effectivity, and no
   computed impact claim;
6. define exactly two `ChangeClosureKind` values:
   `ReleasedSuccessorReference` and `NoReleaseDisposition`; REV-I04 may validate
   and retain an already-existing released-successor reference but cannot create
   a baseline, EngineeringRevision, Release, or DocumentIssue;
7. define exactly three `RevisionReservationStanding` values: `Active`,
   `Released`, and `Expired`; create an active reservation only through explicit
   `ReserveRevisionLabel`, and append release or expiry disposition without
   issuing, reusing, or silently reallocating its label;
8. implement identity-free successor collection only for an exact predecessor
   baseline/CI context under `NoEarlierControl`: first applicable divergence
   atomically creates one quiet Draft Change and transaction link; repeated
   commits reuse it; explicit split, merge, or reassignment preserves the
   predecessor relationship and append-only transaction history; collection
   creates no reservation or other authority family;
9. define exactly four `LegacyRevisionFactKind` values:
   `ProposalAcceptance`, `ReviewBooleanOrFreeFormActor`, `CheckWaiver`, and
   `AcceptedCheckDeviation`, and exactly three `LegacyMappingDisposition`
   values: `Mapped`, `RetainedAsLegacyEvidence`, and
   `RejectedAsInsufficient`;
10. define `WaiverDeparture` and `DeviationDeparture` with exactly these bounded
    authority fields: source fact identity and digest, governing requirement or
    finding, exact scope, authorizing attestation or scoped authority reference,
    validity interval, optional resolved effectivity reference, rationale, and
    disposition; require every field except optional effectivity before mapping;
11. implement exactly these internal revision mutations through the canonical
    revision transaction/store path: `CreateEngineeringChange`,
    `SetChangeRationaleAndClassification`, `AddOrUpdateAffectedItem`,
    `SetChangeEffectivity`, `LinkImplementationTransaction`,
    `SubmitChangeForImpactReview`, `AuthorizeChange`, `BeginChangeImplementation`,
    `RecordImplementationVerification`, `RequestChangeRework`, `CloseChange`,
    `RejectChange`, `DeferChange`, `CancelChange`, `SplitSuccessorWork`,
    `MergeSuccessorWork`, `ReassignSuccessorTransactions`,
    `ReserveRevisionLabel`, `ReleaseRevisionReservation`,
    `ExpireRevisionReservation`, `RecordWaiverDeparture`,
    `RecordDeviationDeparture`, and `RecordLegacyRevisionFactMapping`;
12. implement exactly these pure internal queries/evaluations:
    `ResolveEngineeringChange`, `QueryOpenSuccessorWork`,
    `QueryChangeTransactionLinks`, `EvaluateChangeTransition`,
    `EvaluateDesignMutationAuthority`, `ResolveRevisionReservation`,
    `EvaluateReservationAvailability`, `AssessLegacyRevisionFact`, and
    `QueryLegacyRevisionFactMapping`;
13. add exactly one Design-mutation integration:
    `DesignModel::commit_journaled_with_links_and_inverse` calls
    `prepare_revision_design_commit` once after existing technical guards and
    before staging, obtains exactly one of `UnmanagedPassThrough`,
    `NoEarlierControlCollect`, or `AuthorizedChangeValidated`, and atomically
    commits any planned Change/event updates with the Design transaction;
14. apply that integration to exactly four journaled Design transaction contexts:
    `Normal`, `AcceptedProposalApply`, `Undo`, and `Redo`; no in-memory-only
    mutation, operation validator, resolver, Project-open path, batch builder,
    CLI/MCP/GUI surface, or adapter may independently gate or collect;
15. define exactly four REV-I04 typed refusal codes:
    `ChangeTransitionInvalid`, `ChangeNotAuthorized`,
    `RevisionReservationConflict`, and `DepartureInvalidOrExpired`, each with
    affected identities, controlling policy or requirement, expected/current
    facts, and actionable remediation; and
16. persist the new payloads, records, and events inside the REV-I01/REV-I02
    canonical authority generation, retaining deterministic encoding,
    integrity, single-writer exclusion, atomic promotion, backup/restore,
    unknown-data preservation, and read-only recovery, with cohesive
    `engine::revision` modules and private test-only setup helpers only.

**Closure rule.** Every family, state, event kind, transition, affected-item
action, closure kind, reservation standing, legacy fact kind, mapping
disposition, departure field set, mutation, query/evaluation, integration
context, commit-plan outcome, and refusal enumerated in items 1–15 is the
complete and exclusive set REV-I04 may define or make operational. An item not
named there is deferred to its assigned slice. Adding or substituting an item
after approval requires a new owner authorization and is not a within-slice
implementation detail. REV-I04 completion is verified bidirectionally against
every enumeration.

REV-I04 explicitly may not:

- define another EngineeringChange lifecycle, arbitrary state, profile shortcut,
  ECR/ECO/DCO record family, or mutable current-state authority;
- compute dependency, semantic impact, evidence freshness, library uptake,
  baseline comparison, or establish a ConfigurationBaseline (REV-I05);
- issue an EngineeringRevision or DocumentIssue, create ReleaseCandidate,
  Release, package, Transmittal, or standing authority (REV-I06+);
- treat quiet collection, proposal acceptance, a review boolean/free-form actor,
  waiver, deviation, reservation, transaction, technical revision, Git fact, or
  display label as approval, issuance, or release authority;
- reserve a label or emit a reservation/authority event during identity-free
  collection, preview, query, failed preflight, or failed commit;
- gate, prompt, delay, synthesize policy for, or emit authority from an
  `Unmanaged` or `NoEarlierControl` authoring path;
- add a second Design gate, place revision evaluation inside ordinary operation
  validation/application, or bypass the single journaled coordinator;
- rewrite or delete a legacy proposal, review, waiver, or deviation source;
- expose public native-write, proposal, daemon, CLI, MCP, script, or GUI
  revision surfaces (REV-I09/REV-I10);
- implement Preferences, Publish, enterprise role UI/exchange, Git, PLM/PDM,
  distributed merge, or any adapter; or
- add a third-party dependency.

## 3. REV-I04 proof gates

Acceptance requires committed, addressable evidence for:

- one bidirectional inventory proof showing every item in every section 2
  enumeration exists exactly as authorized and no additional REV-I04 family,
  state, event kind, transition, action, closure, standing, legacy kind,
  disposition, field, mutation, query, integration context/outcome, or refusal
  exists;
- unique stable tags, canonical round trips, insertion-independent ordering,
  digest goldens, type-confusion refusals, unknown-data preservation, and
  backup/restore parity for the three added record families and the two
  operationalized REV-I02 families;
- the complete append-only lifecycle, every permitted transition and side exit,
  every forbidden transition, every rework edge, immutable prior events, and
  both closure kinds without creating issued authority;
- first applicable divergence creating exactly one quiet Draft Change and one
  transaction link; repeated edits collecting without duplication; explicit
  split, merge, and reassignment preserving predecessor and transaction history;
- an identity-free collection proof asserting byte-for-byte that no
  `RevisionReservation`, `EngineeringRevision`, baseline, approval, Release,
  DocumentIssue, or corresponding authority event exists before and after
  collection;
- explicit reservation success, conflict, release, and expiry proofs showing no
  label or authority event is reserved before `ReserveRevisionLabel` commits,
  and failed/staged/crash-injected reservations leave no reservation visible;
- affected-item Add/Modify/Retire/NoChange accounting without dependency or
  impact evaluation and verification evidence that cannot issue or release;
- every legacy fact kind receiving exactly one explicit mapping disposition;
  proposal/review non-equivalence; complete waiver/deviation mapping; missing
  field, wrong scope, unauthorized, expired, and unresolved-effectivity refusal;
  and original source bytes unchanged;
- exactly one source call edge from the journaled commit coordinator to
  `prepare_revision_design_commit`, with ordinary operation validation/
  application, Project open, resolver assembly, native-write builders,
  in-memory commit, GUI, CLI, MCP, and adapters unable to invoke policy or
  authorization evaluation or create collection events;
- `AuthorizedChangeRequired` refusing before staging when exact authorized
  Change scope is absent or invalid, accepting only a valid Authorized or
  Implementing Change, and leaving both Design and authority state unchanged on
  refusal or injected failure;
- the Finding 5 unmanaged and `NoEarlierControl` authoring witness on all three
  named real Projects, behaviorally identical paired runs, and Normal/
  AcceptedProposalApply/Undo/Redo parity with no prompt, gate, or delay;
- REV-I01 through REV-I03 integrity, atomicity, corruption recovery,
  unknown-preservation, backup/restore, single-writer, structural-validation,
  policy, role, attestation, effectivity, seed, and inert-authoring proofs
  remaining green;
- exact offline operation with `.git`, Git executable, network, provider,
  service, production cryptography, and trusted-time mechanism absent;
- no private writer, no public generic JSON patch, and no new product surface;
  and
- all standing dependency, Cargo-resource, source-health, specification,
  evidence, project-state, private-writer, daemon-parity, resolver-fence,
  native-Project, drift, guarded locked/offline workspace, and strict all-target
  Clippy gates.

The full guarded locked/offline suite must pass once without retry. REV-I03
closure results are baseline evidence, not an execution lease or waiver.

## 4. Dependency, migration, and visual posture

- **Dependency:** none requested. Any new crate, provider, executable, service,
  signature/trust/time algorithm, database, adapter, or other dependency stops
  at PM-029 before code changes.
- **Migration:** assessment is explicit and append-only. Existing proposal,
  review, waiver, and deviation bytes are never rewritten or deleted. Each
  selected source receives one of the three enumerated dispositions; only a
  complete waiver/deviation can create the corresponding bounded-departure
  record. Free-form revision labels, Part lifecycle, library review/provenance,
  ZoneFill staleness, title blocks, CLI `release` naming, and other REV-GAP
  migrations remain REV-I11.
- **Visual:** no product surface changes. Stable typed refusals may use existing
  diagnostic channels only. If Change collection, earlier-control refusal,
  reservation, migration, or recovery implies user-visible behavior beyond
  existing typed diagnostics, REV-I04 pauses for a bounded Claude-owned render
  before UI behavior or clauses are added. Codex does not edit the prototype
  lane.
- **Authority:** the sole gate is Project-owned `AuthorizedChangeRequired`
  evaluated at the named journaled commit coordinator. `Unmanaged` and
  `NoEarlierControl` remain first-class authoring. Collection is not approval;
  reservation is not issuance; migration resemblance is not equivalence.

## 5. Owner boundary

REV-I15 is the next free integer step and owns this authorization boundary;
REV-I04 remains the existing execution-slice identity. No sub-lettered step is
created.

<!-- OWNER:PRODUCT-REVISION-ENGINE:REV-I15:REV-I04-PACKET -->
<!-- EVIDENCE:PRODUCT-REVISION-ENGINE:REV-I04-PACKET -->

The exact question is:

> Does this packet preserve Product Mechanics 034's single append-only Change,
> identity-free/no-reservation collection, bounded-departure honesty, and
> first-class unmanaged workflow while authorizing exactly one source-bounded
> `AuthorizedChangeRequired` integration and the closed REV-I04 inventories only?

**Recommended response:** approve only if REV-I04 remains a dependency-free,
offline engine slice; no-policy and `NoEarlierControl` authoring remain
prompt-free and ungated; identity-free collection reserves nothing; legacy
facts are never promoted by resemblance; and impact, baseline, issuance, public
surfaces, adapters, remaining migrations, and later slices stay deferred.

**Exact response format:**

```text
REV-I04-EXECUTION: approve
```

or

```text
REV-I04-EXECUTION: revise — <specific scope, enumeration, lifecycle, integration, never-blocks, reservation, migration, proof, dependency, or visual correction>
```

## 6. Owner disposition

<!-- EVIDENCE:PRODUCT-REVISION-ENGINE:REV-I15-APPROVED -->

The owner approved REV-I15 on 2026-08-29 with the exact response
`REV-I04-EXECUTION: approve`. Authorization is limited to REV-I04 as closed by
sections 2 through 4, including every complete-and-exclusive enumeration and
its bidirectional completion proof.

The sole Design integration remains the named pre-staging preflight at the
existing journaled commit coordinator. `Unmanaged` and `NoEarlierControl`
remain prompt-free and ungated. Identity-free collection may append only the
Change creation and transaction-link events; it reserves no label and emits no
reservation or revision-identity authority event until an explicit
`RevisionReservation` commits. Legacy facts remain append-only assessed
evidence and are never promoted by resemblance or rewritten.

REV-I05 through REV-I12, impact and baseline computation, issuance, public
surfaces, Preferences, Publish, GUI, adapters, production cryptography,
remaining migrations, visible behavior, and every new dependency remain
unauthorized. REV-I04 begins only after fresh drift and guarded locked/offline
Cargo baselines; prior packet results are evidence, not an execution lease.
