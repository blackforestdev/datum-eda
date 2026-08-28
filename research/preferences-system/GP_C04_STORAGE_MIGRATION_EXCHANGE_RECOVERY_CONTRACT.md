# GP-C04 Storage, Migration, Exchange, and Recovery Contract

> **Status:** complete planning contract. Claude commit `586eb0d` renders the
> recovery, portable-import collision, migration-result, and backup/restore
> states of the ratified **Manage preferences** surface; this contract extracts
> those states without editing their visual truth. No implementation,
> dependency, synchronization service, or prototype edit is authorized.
>
> **Tracker:** `dat-global-preferences-engine-qcv`
> **Frontier:** `GLOBAL-PREFERENCES-SPEC / GP-C04`

## 1. Decision and boundary

GP-C04 specifies the canonical local repository, atomic commit/recovery law,
schema and value evolution, downgrade and unavailable-provider behavior,
portable exchange, synchronization conflicts, audit evidence, and the exact
Project-policy seam consumed by approved GP-C03.

It does not reopen Q1–Q10 or Q11, choose the final native GUI composition,
publish the GP-C08 descriptor catalog (historical alias GP-C05A), add a network/account authority, select
a cryptographic or storage dependency, implement the system, or mutate any
Claude-owned prototype.

## 2. Complete owning-route review

The complete `workspace-documentation-and-revision` route in
`specs/evidence_traceability_manifest.json` was re-reviewed before this draft:
all registered Revision, Workspace, Preferences, drafting-standard, visual,
decision, GUI, Publish, and traceability sources and consumers. The following
evidence controls GP-C04 most directly:

- GP-C01 proves that the sole existing Console file has rename-only replacement,
  silently ignores corrupt input, discards unknown fields on rewrite, and has no
  lock, generation, migration, backup selection, or audit
  (`GP_C01_INTERNAL_AUTHORITY_AUDIT.md:84-112,282-296`).
- GP-C02 requires same-directory complete writes, explicit durability ordering,
  last-known-good recovery, quarantine, single-writer locking, expected-generation
  mutation, opaque unknown preservation, deterministic migrations, typed exchange,
  and three-way conflicts rather than timestamp last-writer-wins
  (`GP_C02_EXTERNAL_AND_STANDARDS_RESEARCH.md:234-364`).
- Approved Q5 requires one immutable resolved seed snapshot, an atomic Project
  copy, and a durable itemized receipt with no live following
  (`GP_C03_PROJECT_SEED_AND_CONTEXT_DECISION_PACKET.md:139-174,196-201`).
- Approved Q7 keeps a locally validated managed generation effective only under
  its declared offline-validity law and distinguishes unavailable, stale,
  expired, and revoked without fabricating policy removal (ibid. `:992-1068`).
- Approved Q8 requires byte-faithful inactive unknown envelopes, explicit
  descriptor-owned alias migration, non-deletion on ordinary writes/reset/import/
  export, and the dedicated **Manage preferences** home (ibid. `:1215-1405`).
- Approved Q9 makes inspection read-only and side-effect free and requires the
  same resolver truth across GUI, CLI, and MCP (ibid. `:1547-1688`).
- Product Mechanics 034 and the Revision specification keep complete local
  authority, atomic committed-state recovery, Project policy, and immutable
  engineering/audit facts outside machine Preferences
  (`docs/decisions/PRODUCT_MECHANICS_034_PRODUCT_REVISION_ENGINE.md:40-58,121-154`;
  `specs/PRODUCT_REVISION_ENGINE_SPEC.md:111-124,313-350`).
- Product Mechanics 035 permits only explicit receipted seeding into the
  Project-owned `AdoptedDraftingStandard`; Product Mechanics 036 classifies the
  drawing theme as persisted machine Presentation state and leaves migration to
  this specification (`PRODUCT_MECHANICS_035_ADOPTED_DRAFTING_STANDARD_AUTHORITY.md:18-51`;
  `PRODUCT_MECHANICS_036_SCHEMATIC_DRAWING_THEMES.md:18-57`).
- The reviewed `preferences-window.html#manage-preferences` draws the
  machine-only store location, protected reset/export/import, unknown envelopes,
  explicit alias migration, and export-first deletion.
- Claude commit `586eb0d` adds the reviewed
  `preference-store-states-study.html`: corrupt-store refusal and recovery
  (`:78-83`), portable-import choices and protected-source refusal (`:84-91`),
  migration results and retained human choice (`:92-97`), backup/restore
  generations (`:98-102`), and their common non-blocking preservation law
  (`:103`). These are states of that ratified surface, not a second settings
  authority.

External products and standards remain evidence, not dependencies: XDG supplies
config/state/runtime separation; QSaveFile and SQLite supply durability and
locking lessons; Protocol Buffers supplies unknown-preservation hazards; RFC
6902/7396/9110 supply typed patch and compare-and-swap lessons; RFC 8785 supplies
canonicalization evidence; professional peers supply selective previewed
portability and explicit conflict choices. None becomes Datum authority or an
adopted library by citation.

## 3. Canonical repository model

Datum shall expose one logical `PreferenceRepository` per local installation,
resolved through a platform-location adapter rather than direct scattered
environment reads. The repository is a directory-backed generation store, not
one mutable flat preference object and not a collaborative database.

```text
PreferenceRepository {
  repository_id,
  format_version,
  head: GenerationRef,
  user_and_installation_contributions,
  authority_releases,
  managed_generation_refs,
  registered_state_records,
  unknown_envelopes,
  retained_conflicts,
  audit_receipts,
  recovery_metadata
}

GenerationRef {
  repository_id,
  generation,
  parent_generation?,
  canonical_manifest_digest,
  committed_order,
  writer_instance,
  format_and_canonicalization_versions
}
```

The logical repository keeps unlike authorities in explicit partitions:

1. `DescriptorDefault` remains compiled descriptor truth and is never serialized
   as an explicit contribution merely because Datum read it.
2. `Installation` and `User` contributions are separate, typed partitions.
   A user write cannot rewrite Installation data.
3. `AuthorityRelease` records are machine/user-granted authority facts with
   their own auditable lifecycle; they are not organization package contents.
4. Validated Organization packages are immutable local managed generations.
   Provider download/cache presence alone is not active authority.
5. Q5A guided-setup and Q10 S2 onboarding records are registered machine-local
   state, not ordinary preference contributions or Project facts.
6. Restartable workspace state may use the same durability service later, but
   remains a separately typed state partition with its own reset/export/privacy
   law; GP-C04 does not promote it into Preferences.
7. Project policy and `ProjectSeedReceipt` live only in the governed Project.
   The machine repository may retain a non-authoritative genesis audit receipt,
   but never the mutable Project policy.
8. Secret material is stored by an approved credential facility when one exists;
   repository values carry typed references/redactions rather than inventing a
   plaintext secret store.

V1 persistence uses versioned Datum canonical JSON manifests and typed canonical
JSON values, plus separately addressed opaque payloads for material that cannot
be safely reconstructed. Digests are algorithm-qualified and canonicalization
versions are explicit. This is a byte contract, not adoption of RFC 8785 or a
third-party JSON/database dependency.

Unknown Q8 material is never parsed and re-emitted through a known-only object.
Its exact identity, payload bytes, provider/scope envelope, source version, and
required extension/order material are retained as opaque addressed content and
referenced by every successor generation until explicit Q8 removal.

## 4. Transaction, concurrency, and durability law

Every persistent mutation is one repository transaction against an expected
`GenerationRef`:

1. Acquire the repository's exclusive local writer lease. Readers continue on
   one already validated immutable generation.
2. Verify repository identity, expected head generation/digest, descriptor and
   source eligibility, Q3/Q4 controls, Q7 validation, and all referenced opaque
   content before staging.
3. Build a complete immutable successor generation in the repository's target
   filesystem. Copy/reference all unaffected known and unknown material; no
   partial in-place edit is authoritative.
4. Re-read and validate the staged manifest, canonical values, opaque payloads,
   audit receipt, parent link, and integrity references before promotion.
5. Durably flush staged content and its containing directory in platform-defined
   order. A platform that cannot provide the required atomic replace and locking
   semantics refuses persistent mutation; it never opts into unsafe direct write.
6. Atomically replace the small head reference, then durably flush its parent
   directory. Head promotion is the sole repository commit point.
7. Release the writer lease and publish the new immutable generation to readers.

An expected-generation mismatch creates a typed stale-write conflict; it never
retries by blindly applying the old mutation to the new head. A process-local
writer lease prevents cooperating local writers, while the expected generation
protects against stale clients and broken/foreign writers. A synchronized or
network-mounted directory is not a supported multi-writer protocol.

The repository retains the current generation plus at least the two immediately
preceding complete validated generations. A policy may retain more. Pending
staging content is never counted as a recovery generation, and cleanup cannot
remove the last two predecessors until a newer head is fully committed.

## 5. Recovery and corruption authority

Startup validates the head reference and referenced generation before any value
becomes effective. Selection is deterministic:

1. use the declared head only if its complete manifest, parent/reference graph,
   typed values, opaque payloads, and integrity facts validate;
2. otherwise preserve the suspect bytes exactly under a named unreadable
   identity, start the session from descriptor defaults plus other independently
   valid sources, and offer the newest complete valid retained predecessor as a
   previewable recovery candidate rather than silently promoting it;
3. never merge a partial current generation with a predecessor;
4. never treat corrupt, truncated, unsupported, or missing bytes as a user reset;
5. if no valid generation exists, resolve descriptor defaults and other
   independently valid sources in a persistence-disabled diagnostic state; do
   not overwrite the damaged repository or claim that defaults were stored.

Detection writes nothing over the unreadable store. Recovery does not repair,
truncate, parse-and-rewrite, or delete it. A restore is a separate previewed and
confirmed expected-generation operation with its own receipt; before promotion,
Datum makes the session's current values visible and preserves the current store
as a new recovery generation so the restore is itself reversible.
The repository exposes exact chosen/rejected generation identities and reasons
to the Q9 query and audit surface, subject to redaction.

Dismissal changes nothing on disk and the diagnostic returns on later launch
until the user restores or independently removes the damaged file. The user may
continue authoring throughout.

## 6. Schema, value, alias, and downgrade migration

Datum distinguishes four independent evolution axes:

- repository format/canonicalization version;
- descriptor schema/value version;
- explicit Q8 retired-key alias migration;
- provider/package schema version.

Every supported migration is a registered deterministic transformation naming
its exact source/target identity and version, compatible canonical input domain,
output validation law, and reversibility/downgrade posture. Migrations are pure
plans before commit: repeated planning over the same bytes produces the same
result, and merely reading/querying a store never writes or acknowledges them.

A migration transaction atomically commits its complete selected plan. It preserves
the complete pre-migration generation, stages the successor under §4, validates
every transformed value and untouched unknown envelope, and records a receipt
containing source/target generations, migration identities/versions, affected
keys, before/after digests or explicit redactions, omissions/refusals, actor/
trigger, and result. A plan may deliberately carry an old, now-out-of-domain
value forward as retained-but-unresolved while compatible schema and alias work
commits; that retained state is part of the atomic result, not a partial write.
Datum may show a nearest eligible value but does not make it effective until the
user chooses. A failed structural transform leaves the source generation
effective and preserves/quarantines the failed candidate; it never substitutes
a factory default or partially advances the format version.

Q8 aliases migrate on the next owning write or explicit **Migrate now** exactly
as drawn. The owning transaction carries the valid user value to the one live
key, preserves the retired source on refusal, and never creates two effective
identities.

An older Datum version may write only when the repository format declares a
compatible round-trip projection and every unrecognized envelope can remain
byte-faithful. Newer keys then remain inactive Unknown under Q8. If the format or
canonicalization version is not safely writable, the older version is read-only
for that repository; it cannot perform a known-only downgrade rewrite. Upgrade
does not turn defaults into explicit values.

Migration first writes a complete pre-migration copy. Its result names unchanged
values, aliases moved to one live identity, values awaiting choice, unknown keys
preserved untouched, and the way back. An older compatible Datum reads that
pre-migration copy; newer identities remain ordinary inactive unknown data.

## 7. Managed generation and unavailable-provider persistence

Received organization material enters an immutable quarantine candidate first.
Datum records received bytes/digest, provider/package/generation identity,
declared effective and offline-validity intervals, trust/validation results,
descriptor compatibility, directives, and any refusal. Only a fully validated
generation becomes eligible for Q3/Q4 resolution, and only within an active
`AuthorityRelease`.

Provider reconnection, arrival time, or a larger generation number cannot replace
the active local generation by itself. Acceptance is an expected-generation
repository transaction. Stale, expired, revoked, unavailable, and restored
states follow approved Q7; cached bytes survive for diagnosis/audit according to
retention policy, while expired/revoked contributions cease participation.
Provider absence never deletes local user values, rewrites Project policy, or
creates a lower-source write.

## 8. Typed exchange, backup, and restore

One generic “preferences file” cannot truthfully serve every exchange purpose.
Datum defines distinct envelopes:

```text
PreferenceExchangeEnvelope {
  kind: PortableSelection | ExactBackup | ManagedPackage |
        ProjectSeedPackage | DiagnosticEvidence,
  format_and_schema_versions,
  package_id,
  source_repository_or_provider,
  source_generation,
  declared_base?,
  included_manifest,
  explicit_omissions_and_redactions,
  opaque_unknown_envelopes,
  integrity_metadata,
  provenance,
  optional_trust_evidence
}
```

- **PortableSelection** contains explicitly selected export-eligible User/
  WorkflowDefault values and byte-faithful Unknown envelopes. It excludes
  defaults, live Session/Context, secrets, ineligible Capability values,
  AuthorityReleases, and managed controls unless their owning law explicitly
  permits a non-authoritative diagnostic copy.
- **ExactBackup** contains one complete validated repository generation plus
  every referenced payload, version, receipt, and completeness manifest needed
  to restore and independently verify it. Restore is not semantic merge.
- **ManagedPackage** remains subordinate Organization input under Q3/Q7 and
  cannot carry or manufacture the user's `AuthorityRelease`.
- **ProjectSeedPackage** supplies immutable Q5 seed material. Selecting it for
  genesis does not import its values into the machine repository or create a
  continuing Project link.
- **DiagnosticEvidence** is explicitly non-applying and redacted by descriptor
  policy.

Every incoming envelope is retained and validated in quarantine before it can
affect resolution. Product/schema compatibility, exact identity, descriptor
eligibility, protected sources, redaction, unknown preservation, integrity, and
base generation are checked before a proposed apply/restore plan exists. Apply
is one expected-generation transaction over the user's accepted plan; closing,
previewing, or inspecting an import is side-effect free.

A portable import displays identical values as skipped, new values as reviewable
and individually rejectable even under **Take all**, and differing values as
per-setting **Keep mine**/**Take theirs** choices. Nothing applies until the
user has chosen and confirms one accepted plan. Capability- and security-class
keys are refused rather than offered; organization directives arrive only as a
validated `ManagedPackage`, never through portable import. Unknown envelopes
remain inactive and byte-faithful through import and later export.

Automatic exact-backup generations are machine-local until explicitly exported.
Restore preview lists the settings that change and those that remain identical,
requires confirmation, and first turns the current store into another complete
backup. A restore therefore remains reversible and is never presented as a
per-key portable merge.

## 9. Synchronization and conflicts

GP-C04 does not authorize a synchronization service, account system, network
dependency, shared-folder multi-writer mode, or network requirement. A future
subordinate adapter may exchange `PortableSelection` deltas between repository
replicas, but local resolution and mutation remain complete offline.

Each sync candidate names source repository/replica, source generation, common
base generation or digest, and incoming generation. Datum compares base, current
local, and incoming per stable `PreferenceKey` and descriptor merge law:

- one-side-only change applies in the candidate plan when eligible;
- byte/type-equal concurrent changes coalesce;
- descriptor-declared deterministic joins may produce a typed joined candidate;
- different scalar/indivisible changes become a retained conflict;
- Unknown opaque values coalesce only when byte-identical, otherwise conflict;
- protected, managed, secret, or ineligible values never gain authority through
  synchronization.

Conflicts retain base/local/incoming identities, values or redactions,
provenance, descriptor law, and resolution status. No timestamp, file order,
arrival order, provider order, or “latest generation wins” rule resolves them.
Unresolved candidates do not change the effective repository. A later explicit
resolution commits against the then-current expected generation; if that head
changed, Datum recomputes/refuses rather than applying a stale choice.

Portable-import collision choreography is the rendered local analogue: retain
both values, require a per-setting choice, and apply nothing while unresolved.
A future synchronization adapter shall use the same vocabulary and invariants;
any materially different sync-specific surface remains render-first under GP-C05.

## 10. Audit and Project-policy seam

Every committed mutation records a typed receipt for actor/source, operation,
expected and resulting generation, affected keys/partitions, before/after value
digests or redactions, descriptor and migration versions, provider/package facts,
reason, result, and asserted/observed time classes. Refused stale writes,
migrations, imports, restores, conflict resolutions, AuthorityRelease changes,
recovery/quarantine, explicit unknown removal, and export/backup creation retain
typed evidence proportional to descriptor/profile sensitivity.

Ordinary reads and the approved Q9 explanation are side-effect free and do not
create audit events. Preference audit is machine configuration evidence, not the
Project mutation journal, an EngineeringRevision, a Release, or certification.
Retention/pruning is explicit, never silent; secrets remain redacted while the
authority/disposition reason remains explainable.

For Q5 genesis, the resolver captures one immutable evaluation snapshot naming
the repository generation, descriptor versions, applicable managed package
generations, selected seed package/profile, Context, effective eligible values,
and every omission/refusal. The Project mutation authority consumes that frozen
snapshot and atomically creates Project policy plus the durable
`ProjectSeedReceipt`. A concurrent later preference write cannot change the
captured snapshot. Failure of the preference repository after Project commit
cannot invalidate or rewrite the Project; later comparison/uptake is a deliberate
governed Project proposal under Q5.

No machine import, reset, restore, synchronization event, provider change, or
migration mutates an existing Project. Conversely, opening or mutating a Project
does not write machine Preferences, onboarding state, or AuthorityRelease.

## 11. Rendered Manage-preferences clauses

The following clauses are extracted from Claude commit `586eb0d`, lines 78–103,
and govern the four drawn states:

1. An unreadable store is preserved exactly under a named damaged-file identity;
   Datum never repairs, truncates, rewrites, or deletes it in place.
2. The affected session uses factory defaults while independent valid authorities
   continue normally; the state says what happened and never blocks authoring.
3. Last-good recovery is offered through Preview and confirmed Restore. Preview
   shows affected session values; Restore first backs up the current state and is
   itself reversible. Dismiss writes nothing and the notice returns.
4. Portable import applies nothing during inspection or choice. Identical values
   skip, new values remain individually rejectable, and collisions require an
   explicit per-setting local/incoming choice.
5. Capability and security keys refuse every portable source. Organization
   directives enter only through the managed-provider path and an independently
   sufficient `AuthorityRelease`.
6. Unknown data remains inactive and byte-faithful through recovery, import,
   export, migration, backup, and restore.
7. Migration writes a complete pre-migration copy first, reports unchanged and
   alias-moved values, and keeps one live identity per alias result.
8. When an old value has no legal successor, Datum retains and shows it until the
   user chooses. A suggested nearest value is not applied or substituted.
9. Backup generations stay machine-local unless explicitly exported. Restore is
   visibly distinct from portable import, previewed, listed, confirmed, and
   reversible.
10. No store operation reads, writes, resets, restores, migrates, synchronizes,
    or otherwise changes Project policy or design data.
11. Every destructive store action is previewed, named, and reversible; no path
    silently repairs, substitutes, discards, or grants authority.
12. These are states of Q8-A/P2's sole **Manage preferences** surface. They create
    no second settings surface, network requirement, authoring gate, or reopening
    of Q1–Q10 or Q11.

## 12. Typed operation, query, refusal, and proof contract

The engine-facing boundary is one typed API family; names below are normative
operation identities, not a programming-language ABI:

- Mutations: `CommitPreferenceMutation`, `CommitPortableImport`,
  `CommitManagedGeneration`, `CommitMigration`, `ResolveMigrationValue`,
  `CreateExactBackup`, `CommitExactRestore`, `ResolveExchangeConflict`, and
  `RemoveUnknownEnvelope`. Every mutation names an expected generation.
- Pure plans/queries: `PlanPortableImport`, `PlanMigration`, `PlanExactRestore`,
  `InspectRepositoryStatus`, `ListRetainedGenerations`,
  `InspectRecoveryCandidate`, `QueryPreferenceExplanation`,
  `QueryMutationReceipt`, and `CaptureProjectSeedSnapshot`. Planning, preview,
  dismissal, and explanation never mutate.
- Typed refusals include `ExpectedGenerationMismatch`, `WriterLeaseUnavailable`,
  `AtomicPromotionUnavailable`, `UnreadableStorePreserved`,
  `UnsupportedRepositoryVersion`, `MigrationTransformUnavailable`,
  `MigrationValueChoiceRequired`, `PortableCapabilityRefused`,
  `PortableManagedDirectiveRefused`, `ExchangeConflictUnresolved`,
  `UnknownOpaqueConflict`, `RestorePreviewStale`, `BackupIncomplete`, and
  `ProjectAuthorityBoundary`. Each returns stable identity, reason, retained
  state, and remaining user actions; none is encoded as a substituted value.

Conformance must prove, at minimum: crash interruption at every §4 durability
boundary; truncated/corrupt head preservation without repair-in-place; exact
unknown-envelope survival across every mutation above; refusal of Capability and
managed authority through portable sources; migration alias uniqueness and no
unpicked substitution; downgrade round-trip preservation; per-setting import and
three-way conflict stability; stale-preview and concurrent-writer refusal;
previewed restore followed by reverse restore; audit receipt completeness and
redaction; byte-independent Project-seed snapshots; zero Project/design mutations
from all store operations; complete offline operation; and uninterrupted Design
authoring throughout every displayed recovery/refusal state.

<!-- EVIDENCE:GLOBAL-PREFERENCES-SPEC:GP-C04-RENDER-GAP -->
<!-- EVIDENCE:GLOBAL-PREFERENCES-SPEC:GP-C04-CONTRACT -->
