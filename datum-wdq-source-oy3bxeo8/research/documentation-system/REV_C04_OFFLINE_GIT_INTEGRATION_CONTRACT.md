# REV-C04 Standalone Authority and Git Integration Contract

> **Status:** REV-C04 candidate specification; not ratified and not an
> implementation authorization. This contract defines the Product Revision
> Engine's local/offline authority and the boundary presented to optional Git
> and exchange adapters. The owner-approved REV-C03 identities and lifecycles
> remain controlling. Full multi-writer collaboration, semantic merge
> algorithms, live presence, permissions, and server topology remain owned by
> `dat-distributed-collaboration-architecture-lt1`.

## 1. Decision already made

The project owner has already decided the top-level authority question:

1. Datum must provide complete configuration, change, baseline, approval,
   revision, release, effectivity, status-accounting, reproduction, and audit
   authority without Git or a network service.
2. Git is an optional Datum-owned adapter for inspectable storage history,
   exchange, and collaboration. A commit, branch, merge, or tag never becomes
   engineering authority by existing.
3. Air-gapped operation is a normal supported posture, not a degraded or
   emergency mode.
4. External state returns through Datum's resolver, semantic validation,
   typed-operation, and journal boundary before it can become authoritative.

This packet does not reopen those decisions. It makes them executable as a
specification.

## 2. Scope boundary

REV-C04 owns:

- durable local revision-engine records and recovery semantics;
- the mapping between Datum identities and Git object identities;
- adapter state, divergence, receipts, and failure isolation;
- air-gapped exchange-envelope requirements;
- remote/external attestation verification at the revision-engine boundary;
- the distinction between textual integration and semantic acceptance; and
- release behavior when optional adapter evidence is required by policy.

REV-C04 does **not** choose:

- a CRDT, operational transformation, lock, claim, or multi-writer merge
  algorithm;
- a central service, peer-to-peer protocol, presence system, or access-control
  service;
- user-visible collaboration choreography;
- the final cryptographic implementation or a third-party dependency;
- impact propagation and release reproduction, which belong to REV-C05; or
- visual approval/signing workflows, which belong to REV-C06.

The separate distributed-collaboration specification must consume this
contract. It may strengthen exchange and merge policy but cannot move release
authority into Git or a server.

## 3. Evidence

### 3.1 Internal Datum authority

- `PRODUCT_REVISION_ENGINE_AUTHORITY_MODEL.md:144-157` defines
  `ConfigurationRef` and explicitly excludes Git identity from that union.
- `PRODUCT_REVISION_ENGINE_AUTHORITY_MODEL.md:159-190` makes an established
  `ConfigurationBaseline` an immutable exact manifest.
- `PRODUCT_REVISION_ENGINE_AUTHORITY_MODEL.md:262-284` makes approvals
  append-only attestations over a target digest and scoped role assignment.
- `PRODUCT_REVISION_ENGINE_AUTHORITY_MODEL.md:354-387` makes `Release` the
  atomic Datum issuance boundary over an immutable baseline and frozen
  evidence.
- `PRODUCT_REVISION_ENGINE_AUTHORITY_MODEL.md:580-615` requires one mutation
  path, exact immutable evidence, and a non-authoritative Git boundary.
- `REV_C01_INTERNAL_AUTHORITY_AUDIT.md:103-131` verifies that guarded
  `commit()` plus the journal is Datum's current technical concurrency and
  provenance substrate, while explicitly finding that it is not formal
  release authority.
- `REV_C01_INTERNAL_AUTHORITY_AUDIT.md:248-267` records standalone local
  authority and an optional Git mapping as missing product capabilities.
- Product Mechanics 000D requires deterministic sharding and semantic
  validation above textual merge, but reserves the complete multi-writer merge
  algorithm.
- Product Mechanics 007 requires a later distributed-collaboration gate for
  remote, intermittent, and air-gapped teams.

### 3.2 External primary and field evidence

| Source | Relevant fact | Datum consequence |
|---|---|---|
| [Git `git-tag`](https://git-scm.com/docs/git-tag.html) | Annotated tags carry tagger, time, message, and optional signature; lightweight tags are only refs. Tags can also be force-replaced or deleted. | An annotated/signed tag can mirror a Datum Release, but mutable Git refs cannot be its authority. |
| [Git bundle](https://git-scm.com/docs/git-bundle.html) | Bundles can be self-contained or incremental with prerequisite commits, can be verified, and are explicitly suitable when direct connection is unavailable. | Git bundles are a valid optional air-gap transport, never the only Datum exchange format. |
| [Git pack protocol](https://git-scm.com/docs/pack-protocol.html#_push_certificate) | A signed push certificate binds pusher, destination, commands, and a receiver nonce intended to prevent replay. | Transport authentication is distinct from approval; Datum exchange receipts need destination/context and replay protection when a peer challenges an exchange. |
| [Git merge](https://git-scm.com/docs/git-merge) and [`git-merge-tree`](https://git-scm.com/docs/git-merge-tree) | Git records textual/index conflicts, but some conflicts are not representable as file conflict markers. | A clean Git merge is insufficient proof of a valid DesignModel; every merged tree must re-enter semantic resolution and validation. |
| [Git hash transition](https://git-scm.com/docs/hash-function-transition.html) | Git object formats can use SHA-1 or SHA-256 and maintain mappings during transition. | Every stored Git identity must carry its object-format algorithm; bare fixed-width hashes are forbidden. |
| [Local-first software](https://www.inkandswitch.com/essay/local-first/) | The local-first model treats the network as optional and keeps primary work available locally. | Datum local authority must remain fully functional during indefinite disconnection; sync is reconciliation, not permission to work. |
| [RFC 8785](https://www.rfc-editor.org/rfc/rfc8785.html) | Signing/hashing structured JSON requires an invariant canonical representation. | Signed Datum statements must use a versioned canonical byte encoding; ordinary pretty JSON bytes are not an implicit signature payload. |
| [NIST FIPS 186-5](https://csrc.nist.gov/pubs/fips/186-5/final) | Digital signatures detect unauthorized modification and authenticate a claimed signatory under the applicable trust context. | Cryptographic validity is evidence, not sufficient Datum authorization; role, scope, policy, and target digest must also validate. |
| [NIST SP 800-57 Part 1 Rev. 5](https://csrc.nist.gov/pubs/sp/800/57/pt1/r5/final) | Key lifecycle, protection, inventory, trust anchors, compromise, archival, and recovery require explicit policy. | Datum must not equate an email/key string with actor authority; profile-owned trust and key-state evidence are required. |
| [RFC 3161](https://www.rfc-editor.org/rfc/rfc3161.html) | A trusted timestamp proves a datum existed before a time; it is distinct from a local asserted time. | Offline operation records journal order and asserted time honestly. A profile requiring trusted time must receive acceptable timestamp evidence or refuse; Datum must not fabricate trusted chronology. |
| [TUF role metadata](https://theupdateframework.io/docs/metadata/) | Mature signed-metadata systems separate roles, keys, thresholds, and offline versus online key exposure. | Datum approval policy may require scoped roles and quorum while keeping high-authority keys offline; a transport account is not automatically a release key. |

These sources are architectural precedents, not automatic Datum conformance
claims. Algorithm selection, cryptographic libraries, certificate profiles,
and regulated electronic-signature claims remain subject to explicit standards
and dependency authority.

## 4. Authority domains

The engine must keep five domains distinct:

| Domain | Authority | What it proves |
|---|---|---|
| Datum technical history | Typed operations, `commit()`, journal, object/model revision | What Datum accepted as authored state and in what transaction order. |
| Datum engineering authority | Changes, attestations, baselines, EngineeringRevisions, Releases, standing events | What controlled configuration was authorized, issued, and remains applicable. |
| Local persistence | Deterministic project shards, authority records, immutable evidence/blob bytes, recovery metadata | That the authoritative Datum state survives process, machine, and network failure. |
| Git adapter | Git commits/trees/tags/refs/remotes plus immutable Datum mapping receipts | Where a Datum state was mirrored or observed in a Git object graph. |
| Exchange/collaboration | Verified exchange envelopes and later semantic-integration decisions | What arrived from another replica/site and whether Datum accepted it. |

No identity in a lower row manufactures an identity in a higher row.

## 5. Standalone local authority

### 5.1 Required capabilities without Git

A project with no `.git` directory, no Git executable, and no network must be
able to:

- create and resolve every REV-C03 authority object;
- author and disposition EngineeringChanges;
- record role-scoped approvals using methods permitted by Project policy;
- establish baselines and allocate/issue EngineeringRevisions;
- perform atomic Release and later standing events;
- prepare DocumentIssues and ReleasePackages and record offline Transmittals;
- query current and historical status as of a journal event;
- export and independently verify an audit/evidence package; and
- back up, restore, and reproduce retained records without a remote service.

“Standalone” does not mean “single person” or “no security.” Several people may
act on one air-gapped workstation or exchange signed evidence on removable
media. Profiles may require separation of duties, threshold approvals,
cryptographic credentials, or trusted time. If required evidence is unavailable,
the engine refuses the governed transition rather than silently weakening the
profile.

### 5.2 Local record set

The local Project authority must durably retain:

```text
RevisionAuthorityStore {
  project_identity,
  schema_and_canonicalization_versions,
  project_policy_versions[],
  authority_records[],
  append_only_events[],
  transaction_tip,
  immutable_blob_manifest[],
  actor_role_and_trust_records[],
  adapter_mappings[],
  exchange_receipts[],
  retention_and_hold_records[],
  recovery_checkpoints[],
  integrity_root
}
```

This is a logical authority boundary, not a database or directory-layout
decision. It may be physically sharded, but all shards resolve as one Project
authority. Human-readable source records and canonical hash/signature payloads
may have different byte encodings; the canonicalization version is always
explicit.

### 5.3 Durability and recovery

The existing one-mutation-path rules remain controlling:

1. An authored revision-engine operation is staged with its affected shards and
   immutable evidence references.
2. `commit()` validates expected model revision, accepted transaction tip,
   authority, policy, and record invariants.
3. One journal record is the local commit point for the complete operation
   batch; partial authority visibility is forbidden.
4. Recovery either resolves the last complete committed state or opens a
   read-only diagnostic state. It never promotes a partial Release.
5. Integrity verification walks the journal/checkpoint chain, immutable record
   digests, referenced blob hashes, and policy/schema identities.
6. Backup/export records an exact integrity root and completeness manifest.

REV-C04 does not claim the existing transaction journal already satisfies these
product-record requirements. REV-GAP-01 and REV-GAP-02 remain implementation
gates rather than being hidden by a new Release label.

### 5.4 Time and ordering

Datum records separate facts:

- append/journal order, authoritative within one Project history;
- actor-asserted wall time and its source;
- operating-system-observed receipt time;
- trusted timestamp evidence, if supplied; and
- peer/server receipt evidence, if supplied.

The UI and audits must state which fact is shown. Offline work does not acquire
trusted time merely because a local clock produced an ISO timestamp. Conflicting
wall clocks do not reorder already committed Project events; distributed event
ordering and merge remain a collaboration-spec concern.

## 6. Attestations and signatures

### 6.1 Canonical attestation statement

Every cryptographic approval or release signature covers a domain-separated,
versioned canonical statement, not a screenshot, title block, Git tag name, or
mutable filename:

```text
AttestationStatement {
  statement_type,
  statement_schema_version,
  canonicalization_id,
  project_id,
  governed_target_kind,
  governed_target_id,
  governed_target_digest,
  intent,
  role_assignment_id,
  approval_policy_id_and_version,
  actor_identity_id,
  asserted_time_and_source,
  challenge_or_exchange_nonce?,
  contextual_evidence_digests[]
}
```

The signature envelope additionally names signature algorithm, key/credential
identity, certificate or trust evidence, and encoded signature bytes. Hash and
signature identifiers are algorithm-qualified and versioned.

### 6.2 Verification is not authorization

Importing or recording an attestation succeeds only when:

1. canonical bytes reproduce the claimed target digest;
2. the cryptographic signature verifies when the method is cryptographic;
3. the credential/key was trusted for the actor and intent under the referenced
   policy and effective interval;
4. the role assignment covers the target scope;
5. quorum, order, independence, and separation-of-duty rules pass;
6. required time evidence is present and acceptable; and
7. the attestation is not superseded, revoked, expired, or known compromised.

A valid Git tag signature proves only that a key signed that Git tag object. It
does not prove the signer held Datum Release authority, that the tag message
identified the correct Datum record, or that the release policy passed.

### 6.3 Offline trust

Project policy may trust organization-managed identities, certificates, local
credentials, hardware-backed keys, or another ratified method. Trust roots,
role assignments, key lifecycle, revocation/compromise facts, and policy
versions must be exportable with the evidence needed for later verification.
An online identity provider may assist but cannot be the only way to inspect
already recorded authority.

No cryptographic algorithm, certificate profile, key store, or third-party
library is selected by this planning contract. Those choices require standards
and dependency ratification before implementation.

## 7. Git adapter contract

### 7.1 Logical records

```text
GitObjectId {
  object_format,        // e.g. sha1 or sha256; never inferred by length alone
  object_kind,          // commit, tree, tag, blob
  hex
}

GitMappingReceipt {
  id,
  direction: Exported | Observed | Imported,
  datum_ref,
  datum_digest,
  git_commit,
  git_tree,
  git_tag?,
  repository_identity,
  adapter_version,
  observed_at,
  verification_result,
  prior_receipt?
}

GitDivergence =
  AdapterAbsent
  | Unmapped
  | InSync
  | DatumAhead
  | GitAhead
  | Diverged
  | GitWorktreeDirty
  | MissingPrerequisite
  | RefMovedOrDeleted
  | ObjectInvalid
  | AdapterUnavailable
```

Mappings and observations are append-only evidence. A later force-push, tag
replacement, deletion, hash transition, or remote disappearance creates a new
observation; it never rewrites the prior receipt or the Datum Release.

### 7.2 Permitted adapter operations

The adapter may:

- materialize deterministic Datum shards and immutable evidence pointers;
- create a Git commit representing a resolved Datum technical state;
- record algorithm-qualified commit/tree identities against a Datum digest;
- mirror an existing Datum Release with an annotated or signed tag whose
  annotation includes the immutable Datum Release ID and record digest;
- fetch, push, bundle, or otherwise exchange Git objects;
- inspect refs/remotes and report divergence;
- verify commit, tag, or push-certificate signatures as transport evidence;
- stage an external Git tree for Datum ingestion; and
- record immutable success/failure receipts.

The adapter may not:

- infer an EngineeringRevision from commit count, topology, or message;
- infer approval, Release, standing, effectivity, or transmittal from a branch,
  merge, tag, signature, push, or remote;
- write revision-engine records outside typed operations and `commit()`;
- expose a textual merge directly as authoritative DesignModel state;
- rewrite a Datum record because Git history was rebased or force-pushed;
- treat Git author/committer identity as a Datum role assignment; or
- make ordinary Project open/edit/query depend on Git availability.

### 7.3 Release mirroring and failure atomicity

Datum cannot atomically commit a local Release and mutate an external Git
repository/remote in one transaction. The contract therefore uses ordered local
authority plus idempotent adapter work:

1. Optionally obtain a verified **pre-release mapping receipt** proving that the
   candidate's exact source/evidence state is represented by a specified Git
   commit. A Project policy may require this receipt before local Release.
2. Commit the Datum `Release` atomically. This is the sole issuance event.
3. Append a `ReleaseMirrorRequested` outbox event.
4. The adapter creates/verifies the annotated or signed tag and optional remote
   push, then records `ReleaseMirrorSucceeded` with exact Git identities or
   `ReleaseMirrorFailed` with a typed reason.
5. Retry is idempotent against the Datum Release ID and digest.

A failed post-release mirror does not unrelease or corrupt the local Release.
Policy may block distribution/transmittal until required mirror evidence exists,
or raise a release-administration finding. It must not pretend the external tag
was part of the already-completed local atomic transaction.

## 8. External changes and semantic re-entry

An external Git checkout, received shard set, or exchange bundle is untrusted
candidate input until Datum accepts it.

```text
ExternalChangeCandidate {
  id,
  source_kind,
  source_identity_and_receipt,
  claimed_project_id,
  claimed_base_configuration_ref?,
  received_payload_digest,
  resolver_result,
  schema_migration_result,
  semantic_delta,
  textual_conflicts[],
  semantic_conflicts[],
  signature_and_trust_results[],
  proposed_operations[],
  affected_authority_refs[],
  disposition
}
```

Required ingestion stages are:

1. **Quarantine:** retain received bytes and provenance without loading them as
   Project authority.
2. **Integrity:** verify envelope, object, blob, and prerequisite digests.
3. **Compatibility:** validate Project identity, schemas, canonicalization,
   algorithms, and required migrations.
4. **Resolve:** assemble the candidate model in isolation; fatal resolution does
   not partially alter the active Project.
5. **Compare:** compute a typed semantic delta against the claimed or selected
   base and current local configuration.
6. **Validate:** run invariants, rules, impact discovery, and policy checks
   appropriate to candidate acceptance.
7. **Translate:** produce canonical typed operations or an explicitly bounded
   import operation carrying the semantic delta. Opaque file replacement is not
   a normal acceptance path.
8. **Review/refuse:** expose conflicts and provenance. Unknown identity,
   ambiguous intent, invalid signatures, or unrepresentable semantic change
   refuses or remains quarantined.
9. **Commit:** accepted operations pass through ordinary expected-revision and
   journal guards. Acceptance creates Datum technical history, not an issued
   EngineeringRevision or Release.

A Git merge with no textual conflict may still fail stages 4-8. Conversely,
compatible edits in separate shards may integrate cleanly after semantic
validation. The exact multi-writer merge and conflict-resolution algorithm is
deferred to `dat-distributed-collaboration-architecture-lt1`.

## 9. Transport-neutral air-gapped exchange

Datum requires a transport-neutral envelope so Git is optional:

```text
AuthorityExchangeEnvelope {
  format_and_schema_version,
  exchange_id,
  source_project_and_replica,
  intended_destination_or_scope?,
  created_order_and_asserted_time,
  base_configuration_refs[],
  included_record_and_blob_manifest[],
  prerequisite_digests[],
  policy_and_trust_context_refs[],
  canonical_payload_digest,
  signatures[],
  challenge_nonce?,
  confidentiality_and_egress_metadata,
  optional_transport_payloads[]
}
```

The envelope may carry a complete bootstrap set or an incremental set with
explicit prerequisites. Optional payloads may include a Git bundle, but the
Datum manifest remains sufficient to identify, verify, quarantine, and explain
the exchange without interpreting Git refs as product authority.

Import records:

- receipt identity, source, destination context, media/channel, and observed
  time;
- complete integrity/signature/trust results;
- missing prerequisites and replay/duplicate status;
- semantic integration disposition and resulting Datum transactions; and
- retained original bytes or a policy-governed immutable digest/location.

Encryption, export classification, removable-media procedure, malware scanning,
and cross-domain transfer controls are profile/site concerns. REV-C04 reserves
their typed evidence and refusal seams but does not claim to implement them.

## 10. Typed events, queries, and refusals

### 10.1 Additional typed events

- `RecordExternalMappingReceipt`
- `RecordAdapterDivergenceObservation`
- `RequestReleaseMirror`
- `RecordReleaseMirrorResult`
- `ReceiveAuthorityExchange`
- `VerifyAuthorityExchange`
- `CreateExternalChangeCandidate`
- `AcceptExternalChangeCandidate`
- `RejectOrQuarantineExternalChangeCandidate`
- `RecordCredentialOrTrustEvent`
- `RecordTrustedTimestampEvidence`

These extend the REV-C03 operation catalog; they do not create a second writer.

### 10.2 Required queries

- `authority verify-local [--as-of <event>]`
- `authority export-audit-package <scope>`
- `adapter git status|mappings|divergence|verify <ref>`
- `adapter git explain-mapping <datum-or-git-ref>`
- `release mirror-status <release-id>`
- `exchange show|verify|prerequisites|disposition <exchange-id>`
- `external-change show|compare|conflicts|proposed-operations <candidate-id>`
- `attestation verify|trust-path|effective-authority <attestation-id>`

Every query works with the adapter absent. Adapter queries return
`AdapterAbsent` rather than treating absence as Project corruption.

### 10.3 Additional typed refusals/findings

| Code | Meaning |
|---|---|
| `LocalAuthorityIntegrityFailure` | Journal, immutable record, blob, or integrity-root verification failed. |
| `UnsupportedIdentityAlgorithm` | Hash/signature/object-format algorithm is unknown or prohibited by policy. |
| `ExternalProjectIdentityMismatch` | Received content claims an incompatible Project identity. |
| `ExchangePrerequisiteMissing` | Incremental exchange cannot resolve its declared base/dependencies. |
| `ExchangeReplayOrDestinationMismatch` | Nonce, destination, or replay policy fails. |
| `ExternalSignatureInvalid` | Cryptographic verification failed. |
| `ExternalSignerUnauthorized` | Signature is valid but signer/role/scope/policy is not authorized. |
| `ExternalSemanticConflict` | Candidate resolves textually but violates identity, relationship, or domain semantics. |
| `ExternalChangeUnrepresentable` | Received state cannot be expressed through a permitted semantic import/operation path. |
| `ReleaseMirrorStale` | Git object no longer matches the Datum Release mapping. |
| `RequiredPreReleaseMappingMissing` | Project policy requires an exact verified mapping receipt before Release. |
| `RequiredReleaseMirrorPending` | Release exists, but policy blocks distribution pending mirror evidence. |
| `TrustedTimeEvidenceMissing` | Active policy requires trusted time evidence not available offline. |

`ExternalAdapterUnavailable` from REV-C03 is retained for a policy-required
adapter precondition. Ordinary absence remains a non-error capability state.

## 11. Core invariants added by REV-C04

1. **Local completeness:** every engineering authority workflow and historical
   query has a non-Git, non-network execution path, subject only to the selected
   policy's honestly reported evidence requirements.
2. **Adapter subordination:** Git identities are mappings/receipts, never
   `ConfigurationRef`, `EngineeringRevision`, `ConfigurationBaseline`, or
   `Release` identity.
3. **Algorithm qualification:** every external hash and signature names its
   algorithm and encoding; no bare hexadecimal string is authoritative.
4. **No external partial commit:** adapter or exchange failure cannot partially
   mutate Datum authority or roll back a completed local Release.
5. **Semantic re-entry:** no external tree becomes authoritative without
   isolated resolution, semantic comparison/validation, and typed commit.
6. **Signature separation:** cryptographic validity, actor identity, scoped
   authorization, policy satisfaction, and trusted time are separate results.
7. **Immutable observation:** moved/deleted Git refs and changed remote state add
   observations; they never rewrite historical mapping evidence.
8. **Honest clocks:** journal order, asserted time, receipt time, and trusted
   timestamp are never conflated.
9. **Transport neutrality:** an exchange can be verified and understood without
   Git; a Git bundle is one payload option.
10. **Collaboration reservation:** this contract does not imply that concurrent
    edits commute or select a multi-writer merge mechanism.

## 12. Profile policy seams

Profiles may configure:

- whether cryptographic signatures are required for particular intents;
- accepted signature/hash/canonicalization methods and trust roots;
- approval roles, quorum, order, and separation of duty;
- trusted-time requirements;
- whether a verified pre-release Git mapping is required;
- whether distribution is blocked until a post-release mirror succeeds;
- accepted exchange origins, destinations, media, signatures, and replay rules;
- retention of received payloads and failed/quarantined candidates; and
- whether specified external semantic findings block acceptance.

Profiles may not:

- declare a Git commit/tag/branch to be a Datum Release;
- eliminate the local authority record because a remote system stores a copy;
- bypass semantic re-entry or the canonical commit path;
- make a cryptographically valid signer authorized without role/scope policy;
- rewrite released history after adapter reconciliation; or
- hide unavailable required evidence by silently downgrading policy.

Global Preferences may later seed these policies into new Projects. Existing
Projects retain their copied Project policy, consistent with the owner-approved
REV-C03 preference boundary.

## 13. Conformance proofs required before implementation acceptance

REV-C08 must schedule bounded proofs for at least:

1. full change/baseline/approval/release/audit flow in a Project that has never
   been a Git repository and has no network;
2. crash injection around local release commit proving no partial Release;
3. backup/export, restore, and independent integrity verification;
4. algorithm-qualified Git mappings across supported object formats;
5. force-moved/deleted tag detection without Datum history mutation;
6. Git-clean but semantically invalid merge quarantine;
7. air-gap full and incremental exchange, including missing prerequisites,
   duplicate/replay, wrong destination, corrupted payload, and unknown schema;
8. valid signature by unauthorized actor versus invalid signature versus
   authorized signature, as distinct results;
9. absent trusted-time evidence under permissive and requiring profiles;
10. required pre-release mapping failure versus post-release mirror failure;
11. adapter crash/retry idempotence; and
12. no private writer, dependency-authority, evidence-traceability, or
    source-health regression.

## 14. REV-C04 completion disposition

REV-C04 is complete when this contract is integrated into the revision-engine
evidence route and the authority model records its seams, with no implementation
authorization implied. REV-C05 may then define dependency impact, staleness,
baseline comparison, regeneration, and reproducibility on top of the local
authority and exact exchange/mapping identities defined here.

Items deliberately carried forward:

- exact multi-writer merge, replica, live-session, permission, and server
  mechanisms → `dat-distributed-collaboration-architecture-lt1`;
- final signature algorithms, libraries, key stores, and dependency/license
  decisions → REV-C07/REV-C08 plus Product Mechanics 029;
- detailed impact and reproduction behavior → REV-C05;
- human approval, divergence, and exchange presentation → REV-C06;
- Global Preferences storage and precedence → `dat-global-preferences-engine-qcv`.
