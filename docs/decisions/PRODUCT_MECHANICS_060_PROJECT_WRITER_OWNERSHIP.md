# Decision 060 — Engine-owned Project writer protocol

Status: **ratified for the bounded UED-FIRST-MOVE-01 writer prerequisite**.
Scope: UED-FIRST-MOVE-01's existing-Project writer prerequisite. This is not a
new authoring contract or full foundation acceptance.

## Authority and evidence

The owner approved UED-FIRST-MOVE-01 at e7fd1b28dd2c8ecb79eab6a7f30a5ed594f6c8b4,
packet blob 874ae14396bd25ed282df3cd95cb5f2aed93461d, and appointed session
`codex-user-edit-delivery-20261008` delivery manager under PM059. Bounded PWO-C03
ratification is delegated only after actual reserved independent specification
PASS and disposition of mandatory findings. The manager cannot self-review.
The reserved reviewer is audit-agent thread
`01a0e343-3d9c-70f1-899e-5bd239513383`.

Reuse FOUNDATION_MANUAL_WORKFLOW_CONTRACT F6, its existing writer research,
PWO-C01/C02/PWO-REVIEW/C03 and the source pins in
`docs/reviews/workflow-delivery-rollout/user-edit/first-footprint-edit-readiness.json`.
The owning foundation-manual-workflow route's sources and consumers were reviewed.
The current production source is e7fd1b28. The observations below are source
findings, not a newly reproduced race or a claim of conformance.

<!-- EVIDENCE:USER-EDIT-DELIVERY:UED-C03-WRITER-PROPOSAL -->

## Decision proposed for bounded ratification

Keep one engine-owned ProjectWriteLease, strengthen it to a kernel-backed,
nonblocking exclusive guard, and validate the authoritative persisted preimage
under that guard before composing any design transaction. Do not serialize only
inside the daemon: CLI, in-process Project Preferences, revision operations and
recovery must share the boundary. No broker, dependency or project schema change.

### W1 — Identity, acquisition and release

Use a stable coordination file `.datum/revision/writer.lock` in the canonical
existing Project root. Open read/write, create without truncation, private mode
0600 on first creation; acquire an exclusive `std::fs::File::try_lock`.
Rust's standard-library file-lock API is stable since 1.89; the installed toolchain
is 1.98. No new dependency is needed. Retain the descriptor in the opaque,
root-qualified guard; never clone it or send it to a child. Opening uses normal
close-on-exec behavior. Explicit unlock/drop releases ownership; kernel descriptor
closure releases ownership on process death. Never unlink the coordination file
on release, crash or stale-PID inference. Never truncate or replace a locked inode.
First creation must sync the file and parent before using it as the boundary.

Canonical root aliases share one guard. Refuse symlinked coordination file or
revision directory, nonregular lock files and inconsistent opened/path inode
identity. A held guard is valid only for its canonical root and descriptor.
The guard is not transferable through serialized request parameters. Lock errors
and contention refuse without waiting on the GUI thread or creating a transaction.
Cancellation before lock acquisition has no persisted effects.

The supported assumption is cooperating candidate engine writers on a local
Linux filesystem with functioning advisory file locks and file/directory fsync.
This does not enforce access against an external byte editor, an old binary that
ignores the protocol, or a privileged actor replacing directories. External
changes must still be detected by fresh integrity/revision validation. Mixed old
and new writable binaries are unsupported: pin GUI, companion CLI and daemon to
the same delivery candidate, never share their editable Project with an old
writer. A preexisting legacy `write.lock` refuses adoption/acquisition; do not
reap it using `/proc` or parse failure. Existing roots without that file need no
source migration. The new coordination file is operational metadata, not design
state; exclude it from backup design manifests. Network filesystems and other
platforms are unqualified, not silently accepted or waived.

### W2 — Fresh validation and one canonical mutation path

For an existing Project write, acquire before reading the authoritative state
used to decide the write. Under the guard: inspect/recover eligible committed
staging, resolve the persisted model, verify integrity, journal cursor and tip,
then compare the caller's expected model revision, expected transaction tip where
applicable, object revisions and root/project identity against that fresh model.
Validate object existence, lock state and operation-specific source/pad guards
against the same model. Reject stale intent; never silently rebase it.

Only then compose electrical changes, capture exact inverse operations, calculate
new shards/authority and call the canonical journal commit. All checks that can
change the write decision use the fresh preimage, even if a prepared builder or
GUI preview was created earlier. Reject missing revision qualification. In-memory
`self` is not proof that persisted state is current. A failed stale commit must
not overwrite either disk state or the caller's snapshot. On success publish the
fresh committed model/report atomically to the caller and GUI; complete scene and
selection refresh are revision-qualified. No partial membership publication.

Expose an internal resolver/recovery/commit path taking a borrowed opaque held
lease to avoid recursive acquisition. The existing public commit entry acquires
and delegates to it; public callers cannot supply a fabricated token. Normal
Project resolution that may promote recovery takes this same guard and re-reads
its inputs under it. A read-only snapshot path must never promote stages. Cached
immutable query results may remain revision-qualified, but invalidation must
probe actual lock contention/state validity, not permanent lock-file existence.

### W3 — Durability and recovery ownership

Retain the existing append-and-promote protocol and formats. Stage new shards,
exact inverse/transaction data and revision authority with their existing durable
file writes; sync containing directories before treating stage names as durable.
The durable journal append is the commit point. Hold the guard through shard
promotion, authority promotion and durable cursor publication. Preserve existing
fault boundaries (authority stage, journal append, shard promotion, authority
promotion, cursor write) and prove their ordering; correct an ordering defect if
proof exposes one, without inventing a second writer.

Before durable journal append: failure/cancellation leaves the old authoritative
state; uncommitted stages cannot be promoted. After append: cancellation cannot
undo the transaction by abandoning promotion; finish or recover it. A failed
response after append reports an uncertain outcome, retains transaction evidence,
and requires authoritative reconciliation before retry. No blind duplicate move.

Only a guard holder may select and promote recovery stages. Re-read head, journal
and stages after acquiring; never promote a list selected before acquisition.
Recover only stages with valid hashes and committed journal/authority linkage;
roll forward deterministically and idempotently. Corrupt, ambiguous or incomplete
committed recovery evidence refuses further writes with a recoverable diagnostic
and preserves bytes for investigation. No stale-lock deletion, silent rollback or
invented successful receipt. Revision-only operations follow their established
head/integrity guards and same serialization boundary. Restore operations retain
backup/recovery semantics; they cannot remove the live guard inode.

### W4 — Owning consumers and concrete failure timeline

| Entry/consumer | Required integration |
| --- | --- |
| substrate/commit.rs; undo_redo.rs; native_write/context.rs | Acquire/fresh resolve/validate before compose; canonical guarded commit; history uses fresh tip, not imported-session undo |
| substrate/project_resolver.rs; revision/store.rs | Guard-qualified recovery and coherent resolve; replace PID-based reclaim; re-read recovery candidates under guard |
| revision/authority_store.rs; revision/backup.rs | Same root guard and fresh head; restore preserves coordination inode; exclude coordination metadata from design backup |
| substrate/project_read_cache.rs | Permanent lock-file presence is not busy; retain revision/integrity qualification and actual contention invalidation |
| native CLI mutation/history and Project Preferences callers | Existing APIs converge on the strengthened engine boundary; no private file writer or bypass |
| engine-daemon native_write dispatch; verb-registry; GUI protocol/app | Typed native requests, expected revision/tip and stable Footprint ID; async busy/stale outcome; no shell-command authoring fallback |
| KiCad materialization/project genesis | Preserve converter and canonical genesis; new-root staging/no-replace stays distinct; edit persistent derived Project with --project-root, never reimport over edits |

Two clients can prepare revision R. A acquires, freshly validates R, commits R+1
and releases. B acquires afterwards, freshly resolves R+1, rejects its R intent
without new journal/shard bytes. Simultaneous acquisition refuses one writer.
If A dies before append, a guard-qualified resolver leaves R authoritative; if it
dies after append, the next guard holder validates and completes R+1 before any
new intent is accepted. PID reuse and malformed legacy records never establish
ownership or justify deleting a live boundary. This is a protocol obligation,
not yet a measured implementation result.

## Alternatives and limits

Daemon-only serialization misses CLI and in-process writes. The existing
create-new/PID-reap lease leaves a stale-record unlink race and permits stale
resolved preimages; retaining it without a safer reclaim rule is insufficient.
A new dependency, distributed lease, persistent design-profile migration and
socket-only writer authority are outside this bounded prerequisite.

Stable kernel ownership avoids PID-reap races for cooperating candidate writers.
It deliberately makes no guarantee about uncooperative old binaries. First-edit
ordinary launch must pin all writable binaries and name the local filesystem.
If the reserved review finds those limits incompatible with existing requirements,
return that mandatory finding before ratification rather than silently waive it.

## Finite implementation and delivery obligations

1. Strengthen the existing engine guard and fresh-preimage/recovery path. Reuse
   existing journal guard, fault-point, backup and stale-intent fixtures. Add only
   missing two-process contention/stale-prepared-intent coverage; prove release
   after process death and before/after-append recovery without deleting locks.
   Check alias identity, legacy lock refusal, corrupt recovery refusal, restore
   inode preservation and unchanged state on rejected requests.
2. Reuse canonical Footprint translation and exact pad-source guards; wire typed
   native Move, one undo/redo and Save acknowledgement. Preserve original target,
   cursor offset and revision across preview. Escape cancels without writes;
   locked/deleted/foreign/stale targets refuse. Pending commit cannot be silently
   canceled after its commit point. No rotation, routing or batch-edit scope.
3. Derive a persistent Datum-owned DOA2526 Project, preserve original KiCad hashes,
   demonstrate preview/cancel/commit/undo/redo/Save/reopen in ordinary native
   Wayland+WGPU. Save acknowledges the canonical durable state; it is not another
   serializer. Reuse imported scenes and existing adequate selection evidence.
4. Return exact implementation/source, executable and packet pins for reserved
   bounded independent candidate review. Development-build acceptance belongs
   to the appointed manager only after real receipts and required evidence.
   Missing native/HUMAN evidence remains missing; no X11 substitution.

## Existing PWO clause disposition map

WRITER-ENTRY-POINTS and FAILURE-TIMELINE: W4 and source-backed readiness owners.
SERIALIZATION-MECHANISM: proposed W1/W2 and alternatives.
LOCK-AND-RECOVERY: proposed W1/W2/W3.
DURABILITY-PLATFORM: proposed W1/W3 and explicit limits.
TIP-AND-CONFORMANCE: W2/W3 and finite proof obligations above.
CONSUMER-HANDOFF: W4 and ordered implementation obligations.
INDEPENDENT-REVIEW: **pending actual reserved receipt**.
OWNER-RATIFICATION: **pending delegated manager disposition after PASS**.
IMPLEMENTATION-OBLIGATION: UED-C06/C07 conditional grant; **not implemented**.
No whole PWO, GWP W1–W3, S5A C04–C06, NA or CAM/T04 closure is claimed.

## Actual review and delegated ratification

<!-- EVIDENCE:USER-EDIT-DELIVERY:UED-C04-WRITER-PASS -->
Reserved audit-agent thread 01a0e343-3d9c-70f1-899e-5bd239513383 returned actual
independent specification PASS at 2026-10-08T20:27:05Z for commit
49ef303e2f1ee7290fc00dfef47ae72d447f3442, proposal blob
e9a0991f81aa8777e6acfef38e6d06a4c1ccc068. No mandatory findings; ten bounded
PWO clauses reviewed, 26 readiness source pins verified. Preserve exact receipt
in docs/reviews/workflow-delivery-rollout/user-edit/writer-spec-receipt.json.
This replaces the transport blocker; no new reviewer or self-review was used.

<!-- EVIDENCE:USER-EDIT-DELIVERY:UED-C05-WRITER-RATIFICATION -->
Appointed manager codex-user-edit-delivery-20261008 checked the unchanged reviewed
blob and exercises the owner's explicit conditional PWO-C03 delegation: ratify
W1–W4 and finite implementation obligations for this first-edit scope. UED-C06
execution proceeds under the retained owner grant. Preserve reviewer watchpoints
on fresh validation, nonrecursive guards, directory durability, uncertain
outcomes, cache contention and restore inode preservation. Specification PASS
is not implementation acceptance; all broader qualification remains unpassed.
