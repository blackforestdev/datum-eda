# Review records and temporary proof output

This index owns temporary-evidence storage guidance. Product authority lives in
numbered decisions and domain contracts; the structured Frontier selects work.

- [Specification/governance reconciliation](specification-history/governance-reconciliation.md): five-item maintenance scope, dispositions and review.
- [Completed repository maintenance](specification-history/maintenance-records.md): cleanup requirements, completion receipts, exact history-rewrite approval and recovery.
- [GUI performance](gui-performance/): bounded measurements, source pins, owner dispositions and qualification limits.
- [Workflow delivery](workflow-delivery-rollout/): retained planning, implementation and independent-review receipts; retired enforcement stays retired.

## Cargo output and native workspace separation

`target/` holds disposable Cargo output. Native development projects and temporary
proof use task-owned external workspaces. [Completed relocation and recovery](specification-history/maintenance-records.md#cargo-output-and-native-workspace-separation)
records preserve source/history identities; untracked does not mean disposable.
Inspect ownership and running processes before any cleanup. Current helpers and
check commands are indexed in [documentation navigation](../README.md#development-tools).

The owner's 2026-10-13 direction is controlling: screenshots used to demonstrate
work are temporary. Do not commit or push them, recordings, raw capture archives,
or compressed diagnostic bundles. Raw per-process DRM samples also belong in
temporary output; retained JSON findings use explicit summaries and availability.
The pre-commit temporary-artifact gate enforces
this rule, including forced additions. It also rejects compressed archives
anywhere in the repository so moving a bundle into testdata cannot evade the rule.

Write temporary proof to an external, task-owned development workspace. Reserve
`target/` for disposable Cargo output. Keep proof only while needed for debugging or
review, then remove it. Before removing local output, check ownership and active
processes; never delete another session's live work or clean active Cargo targets.
Do not preserve duplicate build trees indefinitely as proof archives.

## Cargo cache maintenance

Run `python3 scripts/manage_cargo_cache.py` for a read-only usage report.
It separately lists large direct proof logs; these are temporary verification
output rather than Cargo caches and need task-owner cleanup after review. Do not
run unbounded diagnostic tracing or preserve multi-gigabyte stdout indefinitely.
Use the shared `target/` for ordinary builds. A separate task-owned build tree
must be released and removed when its build/review use ends; do not move it into
`.datum/` to retain it indefinitely. Preserve needed findings as text instead.

For a reported disposable Cargo target, first preview the exact path:

```sh
python3 scripts/manage_cargo_cache.py --remove target/completed-proof
```

After confirming the task has released it, append `--apply` to remove it. Multiple
explicit `--remove` paths are supported. This is a local command, not a scheduled
automatic eviction. It holds the existing Cargo resource lock and refuses while
Cargo, rustc, the GUI or engine daemon is active. It protects shared `target/`,
`debug/`, `release/`, tracked files, unmarked fixture/evidence directories, unknown
directories and symlinks. A separate target with extra non-Cargo root entries
also refuses cleanup. Selecting an eligible cache removes its entire tree,
including any output or fixture placed inside its build subdirectories; inspect
the contents and release ownership before applying cleanup. A running executable
inside any selected cache refuses cleanup even if its process name is unfamiliar.
Only separate Cargo trees with Cargo markers, named historical
`.datum/agent-cargo-archive-*` directories, and explicitly selected shared
`target/debug/incremental` or `target/release/incremental` caches are eligible.
Clearing an incremental cache preserves binaries but makes the next incremental
build redo that cached work.

The Cargo guard now applies its existing soft/hard storage limits to the combined
`target/`, named historical Cargo archives, and any external selected target,
without counting a nested target twice. Choosing another small target no longer
bypasses the retained-cache limit. Limits are unchanged; cleanup is explicit and
never initiated by a build or by an elapsed-time guess about another task.

Commit concise text findings, relevant source revisions, reproducible commands,
review disposition and limitations. Hashes can identify artifacts that existed
during review; they do not promise permanent artifact availability. Screenshots
alone do not establish acceptance, and this storage policy does not waive required
runtime checks or independent review.

Automated image comparison inputs under `crates/gui-render/testdata/golden/` and
the two owner-approved design references under `docs/gui/reference/` remain test
and design inputs. Incidental `revision/running-app/` captures are excluded.
Authored product images belong under `crates/**/assets/`; authored diagrams are
also distinct from temporary work proof. Do not relocate screenshots into these
input directories to bypass the policy. Other raster images and recordings are
ignored and rejected by the gate even in newly named proof directories.

## Bounded GUI diagnostics

`DATUM_ACTION_EVIDENCE=1` now emits `datum.action-evidence/v2`. Selection records
contain kind, revision, scope, focus, counts, exact sorted membership SHA-256
digests and at most eight identity samples. They omit complete property and
projection maps and explicitly set `complete_inspection=false`. Normal launches
leave this diagnostic stream disabled. There is no full-Debug-dump override.

The stream limits a record to 64 KiB and a session to 8 MiB, reserving space for
an explicit `diagnostics_incomplete` event before stopping output. These are
diagnostic storage limits, not selection limits or product performance budgets.
They provide ample room for the ordinary registry and compact state, while
preventing a few interactions from generating gigabytes. Stream exhaustion is
never a successful or complete proof receipt. Other timing/verbose output is
separate and should be enabled only for the specific diagnostic question.
The byte bounds do not qualify diagnostic memory use or input latency: exact
identity hashing still scales with membership and uses temporary serialized
buffers, and the existing asynchronous queue is not a bounded-memory queue.

Legacy pilot observations reject v2 and incomplete records because the summaries
do not satisfy their old complete-selection assertions. Historical, hash-pinned
scratch readers need deliberate migration before use with current binaries; do
not silently substitute summaries for those assertions. Exact field/membership
checks should query ordinary typed context/accessibility/engine interfaces at the
required checkpoints rather than repeatedly serialize internal state to stderr.

The owner-directed 2026-10-14 cleanup purged 31 obsolete S5A stdout logs and
duplicate full-state excerpts, reclaiming 8.09 GiB after confirming no application,
build or open log reader/writer was active. Their old paths and hashes describe
historical observations, not currently available replay input. Existing derived
findings remain at their recorded scope; this cleanup does not claim S5A passes.

## Historical records

This cleanup removes accumulated proof images and compressed bundles from the
current tree. Older review records retain their original observations, paths and
checksums as historical records. Those paths no longer guarantee files in a fresh
checkout. Do not claim a historical bundle was replayed when it is unavailable;
regenerate only the evidence actually needed for a current decision.

The retired manual `scripts/workflow_delivery_pilot_capture.py` and
`scripts/workflow_delivery_pilot_package.py` require the removed historical
`research/process-quality/evidence/wdq-c01-current/native-fixture.tar.gz`.
They cannot replay that pilot from the cleaned checkout. Historical diagnosis
requires the pre-cleanup tree; this does not reinstall retired WDQ enforcement
or authorize a historical pilot replay as a prerequisite to current work.

The owner-approved history rewrite is complete; its exact approval and rollback
record are retained in the [maintenance history](specification-history/maintenance-records.md#owner-directed-maintenance-completion). Historical object IDs require the retained
source-history map/tags where applicable. No maintenance receipt completes S5A
or changes its acceptance requirements.
