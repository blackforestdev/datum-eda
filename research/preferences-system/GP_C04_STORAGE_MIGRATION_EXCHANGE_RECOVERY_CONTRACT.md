# GP-C04 Storage, Migration, Exchange, and Recovery Contract

> **Status:** architecture draft through the render-first boundary. The
> non-visual authority and persistence model is reconciled below. User-visible
> recovery, import/synchronization collision, and migration-result clauses are
> deliberately withheld until Claude renders them in the real Preferences
> window. No owner boundary, implementation, dependency, or prototype edit is
> authorized.
>
> **Tracker:** `dat-global-preferences-engine-qcv`
> **Frontier:** `GLOBAL-PREFERENCES-SPEC / GP-C04`

## 1. Decision and boundary

GP-C04 specifies the canonical local repository, atomic commit/recovery law,
schema and value evolution, downgrade and unavailable-provider behavior,
portable exchange, synchronization conflicts, audit evidence, and the exact
Project-policy seam consumed by approved GP-C03.

It does not reopen Q1–Q10 or Q11, choose the final native GUI composition,
publish the GP-C05A descriptor catalog, add a network/account authority, select
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
- The reviewed `preferences-window.html#manage-preferences` already draws the
  machine-only store location, protected reset/export/import, unknown envelopes,
  explicit alias migration, and export-first deletion. It does not draw recovery,
  import/sync conflicts, or migration refusal/results.

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
2. otherwise quarantine the suspect bytes and select the newest complete valid
   retained predecessor;
3. never merge a partial current generation with a predecessor;
4. never treat corrupt, truncated, unsupported, or missing bytes as a user reset;
5. if no valid generation exists, resolve descriptor defaults and other
   independently valid sources in a persistence-disabled diagnostic state; do
   not overwrite the damaged repository or claim that defaults were stored.

Recovery changes which already committed generation is readable; it is not a
new user preference transaction and cannot promote a pending write. A later
repair/restore is a separate expected-generation operation with its own receipt.
The repository exposes exact chosen/rejected generation identities and reasons
to the Q9 query and audit surface, subject to redaction.

The user-visible presentation and actions for successful fallback and the
no-valid-generation diagnostic state are intentionally not clause-complete here;
§11 requires a Claude render first.

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

A migration transaction is all-or-nothing across its selected plan. It preserves
the complete pre-migration generation, stages the successor under §4, validates
every transformed value and untouched unknown envelope, and records a receipt
containing source/target generations, migration identities/versions, affected
keys, before/after digests or explicit redactions, omissions/refusals, actor/
trigger, and result. A failed transform leaves the source generation effective
and preserves/quarantines the failed candidate; it never substitutes a factory
default or partially advances the format version.

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

The visible pending/success/refusal and migration-receipt presentation is held
at §11's render gate.

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

The visible preview, collision choices, exact-backup warning, and completion/
refusal result remain withheld until §11's render is reconciled.

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

The conflict-review choreography is a visible behavior and remains held at the
render gate.

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

## 11. Render-first reconciliation required before visible clauses

The architecture above necessarily creates visible states that the current
Claude-owned prototypes do not draw. `preferences-window.html#manage-preferences`
currently shows location, reset/export/import, unknown preservation, alias
migration, and deliberate deletion, but not the following consequences. Codex
must not author their interaction clauses ahead of visual evidence.

### Bounded Claude reconciliation list

**Owning file and primary anchor:**
`docs/gui/prototypes/preferences-window.html`, `#manage-preferences`. Claude may
add a dedicated comparison study if genuine alternatives need simultaneous
rendering, but the selected behavior must ultimately be shown as a real-window
composition rather than an abstract card alone.

1. **Corrupt-current recovery:** render both a successful automatic selection of
   the newest complete last-known-good generation and the no-valid-generation
   diagnostic state. Show the chosen and quarantined generation identities,
   what remains effective, whether persistence is disabled/read-only, and the
   available review/export/restore actions. Factory reset or silent overwrite
   must not be implied.
2. **Portable import and synchronization collision:** render the pre-apply
   candidate beside retained local truth, including base/local/incoming values
   or redactions, compatible changes, descriptor conflicts, unknown opaque
   conflicts, ineligible/protected refusals, explicit omissions, and the atomic
   apply boundary. No preview action may already apply, grant authority, or use
   timestamp/arrival-order last-writer-wins.
3. **Migration result:** render pending alias/value/store migrations, successful
   receipt access, and a refused migration that leaves the source generation
   effective and preserved. Distinguish deterministic migration from reset,
   import, and recovery; preserve the existing **Migrate now** and one-live-key
   Q8 law.
4. **Exact backup/restore versus portable import:** make the two operations
   visibly distinct so a full-generation restore is not mistaken for per-key
   merge and a portable import is not represented as opaque store replacement.
5. **Accessibility and stable context:** every state uses text plus non-color
   cues, programmatic status, keyboard operation, focus-preserving/polite
   announcements, and open-beside/stable-list behavior consistent with Q9.

**Preserve unchanged:** Q1–Q10 and Q11; Q8-A/P2's sole **Manage preferences**
home, byte-faithful inactive unknowns, descriptor-gated activation, one-live-key
alias migration, non-deletion, and export-first removal; Q9 one-query semantic
parity; Q7 stale/unavailable/expired/revoked distinctions; Q5 copy-once Project
receipt and no live following; PM-034/035/036 authority boundaries; manual-first,
no network startup/load requirement, no new modal gate, no prototype mutation by
Codex.

**Proof expected:** Claude-owned prototype commit(s), stable file/section anchors,
source-line review evidence, rendered screenshots at normal and narrow/keyboard
states, and evidence-route registration/digest reconciliation. If multiple
genuine candidates survive, label each honestly and leave selection open; do not
hide a product decision inside polish.

<!-- EVIDENCE:GLOBAL-PREFERENCES-SPEC:GP-C04-RENDER-GAP -->

## 12. Completion conditions after render reconciliation

After the render lands, GP-C04 must extract rather than invent the missing
visible clauses, adversarially reconcile them with §§3–10, publish exact typed
operations/queries/refusals and conformance proofs, refresh the route digest,
and complete the Frontier step without opening an owner boundary unless the
render leaves a genuine mechanism choice that planning authority cannot resolve.
