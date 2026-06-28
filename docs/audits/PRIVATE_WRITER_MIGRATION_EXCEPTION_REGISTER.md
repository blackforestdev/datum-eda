# Private Writer Migration Exception Register

Date: 2026-06-25

## Purpose

Datum authored source writes must flow through typed `OperationBatch` commits,
journal staging, and resolver-owned source shards. This register documents the
remaining direct-write classes that are allowed by
`scripts/check_schematic_private_writers.py`.

An exception below is not permission to add another writer. It is a bounded
owner classification for an existing non-authoring path. New authoring paths
must use the substrate.

## Exception Classes

| Class | Owner | Allowed Surface | Retirement / Tightening Condition |
| --- | --- | --- | --- |
| Project bootstrap | Native project creation | Initial `project.json`, schematic root, board root, and rules root creation in `command_project_roots.rs` | Keep only as genesis creation. Any post-create mutation must be journaled. |
| Route-strategy fixture generation | Deterministic regression fixtures | Generated route-strategy fixture and artifact files in `command_project_route_proposal.rs` | Keep only for deterministic test fixture generation. Production project mutation must not use this path. |
| Legacy KiCad modify persistence | Retired compatibility island | Save/save-original calls in `command_modify/modify_ops.rs`, guarded to tests | Remove when legacy KiCad modify compatibility tests are replaced by native Datum project authoring tests. |
| Proposal apply bridge | Proposal substrate | Proposal apply files may call journaled proposal helpers, not mutate proposal sidecars directly | Retire bridge-only helpers when proposal apply is entirely modeled as typed domain operations. |
| Legacy proposal sidecar | Retired sidecar boundary | No direct writes; `proposal.rs` exposes journaled proposal metadata commit only | Remove legacy naming once all callers use `ProposalMetadata` terminology. |
| Generated evidence | Resolver-owned generated artifacts | CLI generated-evidence producers must use explicit generated-evidence operation batches; engine owns actual generated-evidence writes | Keep retiring helper-call allowances as each generated-evidence family receives journaled lifecycle/undo semantics. |
| Engine generated-evidence helper | Generated evidence persistence | `persist_generated_evidence` owns atomic temp-write/rename/sync for generated-evidence shards | Keep as the single generated-evidence file writer; do not expose as authored-design mutation. |
| Engine source-stage helper | Journal staging | Shared `journal::stage_new_shard_write` writes new shard content under `.datum/stage` before journal promotion; ComponentInstance, relationship/variant, production, pool, proposal, schematic-definition, generated-evidence, ZoneFill, import-map, and forward-annotation staging route through that shared helper | Keep as journal implementation detail; continue retiring any new per-family stage writer back to the shared staging helper. |
| Engine journal persistence | Journal owner | Transaction journal, cursor, and staged shard promotion | Permanent substrate owner for durable transaction state. |
| Generated export | Output files | Gerber, drill, BOM, PnP, and panelized generated outputs | Keep as generated output only; generated exports must not be source authority. |
| GUI board text | GUI command handoff | GUI may prefill terminal commands only; direct board-text mutation files are retired | Keep GUI edits routed through terminal/CLI journaled commands until GUI commands dispatch typed operations directly. |

## Enforcement

`scripts/check_schematic_private_writers.py` enforces the current file list,
required owner patterns, and exact direct-write counts for the classes above.
`scripts/run_drift_gates.sh` runs that guard as part of the standard drift gate.

The register is complete only when every allowed direct writer is one of:

- genesis/bootstrap creation,
- journal or staging persistence owned by the substrate,
- generated evidence or generated export output,
- explicit compatibility sidecar with a retirement condition,
- GUI command handoff with no private authored write.

## Retired Exceptions

| Class | Retirement Evidence |
| --- | --- |
| Legacy import-map sidecar | The former test-only `write_legacy_import_map_sidecar` helper was removed. Import-map shard creation coverage now uses journaled `Operation::CreateImportMapShard`, path validation rejects unsafe relative paths through source-shard ownership checks, and undo removes the promoted sidecar through the journal. |
| Direct forward-annotation review writer | Forward-annotation review persistence now uses journaled `Operation::SetForwardAnnotationReview` / `Operation::DeleteForwardAnnotationReview`; resolver/replay can recover `.datum/forward_annotation_review/review.json` from the journal, while embedded manifest review data remains a read-only legacy fallback. |
| Forward-annotation review state direct writer | The review-state command remains guarded, but only as a retired path: the guard requires `Operation::SetForwardAnnotationReview` / `Operation::DeleteForwardAnnotationReview` plus `commit_journaled()` and forbids `std::fs::write`, `write_canonical_json`, and `project.json` writes. |
| Per-family source-stage writers | ComponentInstance, relationship/variant, and production staging now route new-shard writes through shared `journal::stage_new_shard_write`; the private-writer guard expects zero direct `std::fs::write` calls in those family files. |
