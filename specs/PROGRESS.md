# Specification Progress Tracker

> **Purpose**: Maps every requirement from the controlling specs to its
> implementation status. Updated when code changes.
>
> Legend: `[x]` done, `[~]` partial, `[ ]` not started, `[—]` deferred/N/A

Last updated: 2026-06-21

**Current-vs-target framing**:
- **Current implementation evidence**: the historical milestone tables below
  remain truthful records of the implementation slices that have landed.
- **Active target**: product-mechanics substrate readiness before further
  imported-board fidelity expansion.
- **Not the North Star**: legacy M0-M7 milestone completion rows are retained as
  evidence, but they no longer define the next implementation priority.

**Active milestone**: substrate readiness for product mechanics. Imported-board
fidelity work is paused unless it directly proves or unblocks the substrate
contracts below.
**Frozen**: M6 (strategy reporting layer landed; pending repeated evidence
runs from the checked-in baseline gate).
**Closed for scope**: M0–M5.
**Spec stubs awaiting implementation**: Standards Audit Batch 1 — see
section "Standards Audit Batch 1 — Spec Stubs Awaiting Implementation"
below.

Machine-checked inventory shapes live in `specs/SPEC_PARITY.md` (gated by
`scripts/check_spec_parity.py`, wired into `scripts/run_drift_gates.sh`).
Surfaces currently locked: `mcp_runtime_methods`, `cli_project_commands`,
`engine_text_modules`, `m7_text_visual_fixtures`, `workspace_crates`.

---

## Scope Integration / Substrate Readiness

This is the active tracking surface for the new scope. Items here should move
from target to current evidence only when there is an implementation anchor,
test/gate, and CLI/MCP/API surface where applicable. Imported-board fidelity can
continue only after the relevant substrate contracts are implemented or after an
explicit governance decision records why a fidelity slice is exempt.

| Substrate Area | Status | Current Evidence | Target / Readiness Definition |
|----------------|--------|------------------|-------------------------------|
| ProjectResolver | [~] | First engine-owned scaffold landed in `crates/engine/src/substrate/mod.rs`: `ProjectResolver::resolve()` reads native project roots, assembles deterministic source-shard metadata, emits diagnostics, replays the accepted journal prefix, exposes materialized source-shard values, and is exposed through `project query <root> resolve-debug`. Core board query helpers for components, pads, routing objects/nets, net classes, dimensions, text, keepouts, outline, stackup, diagnostics, and routing substrate now read the resolver-materialized board state rather than only promoted `board.json`, with stale-promoted-file regressions for stackup, pads, board name summary, project name summary, rules query, and forward-annotation proposal component comparisons. Board materialization now preserves native `pad_expansion_setup` so standards/process-aperture DRC sees the authored pad mask/paste policy. Project manifest materialization now replays journaled project-name edits into summary readback, and rules-root materialization now replays journaled rules replacement into `project query design-rules`. Board-component mutation readbacks and package-materialization pre-reads now use resolver-materialized board state after journaled writes. Initial fabrication read paths also moved: Gerber outline and Gerber copper export, validation, and semantic comparison now use resolver-materialized board state with stale-promoted-file regressions; copper CAM derives pads/tracks/zones/vias from one resolved board snapshot per command. Gerber soldermask, paste, silkscreen, and mechanical export, validation, and semantic comparison now load resolver-materialized board state; export has stale-promoted-file regressions for mask, paste, silkscreen, and mechanical layers. Gerber plan/set discovery now derives the planned artifact set from resolver-materialized stackup state with a stale-promoted-file plan regression. Native CSV drill, Excellon drill, and drill hole-class reporting now read one resolver-materialized board snapshot, with stale-promoted-file export regressions for CSV and Excellon drill. Manufacturing report/manifest/inspect/export/validate/compare wrappers now load resolver-materialized board state for wrapper metadata and default artifact naming; manifest has a stale-promoted-file regression. Native project summary now reads resolver-materialized project and board state for product identity. It does not yet own imported KiCad/Eagle resolution, dependency policy, mutation routing, or the remaining fabrication/export read surfaces. | One resolver owns project roots, source discovery, dependency resolution, identity lookup, and deterministic diagnostics across native, imported, CLI, MCP, and GUI paths. |
| Source shards | [~] | First source-shard metadata exists for native project manifests, schematic roots/sheets, board roots, rules roots, pools, proposal sidecars, output jobs, manufacturing plans, panel projections, output-job runs, artifact runs, check runs, ZoneFill records, and artifact metadata, including stable shard IDs, relative paths, schema versions, content hashes, authority class, and dirty-state field. Resolver-visible native pool shards now include `units`, `symbols`, `entities`, `parts`, `packages`, `footprints`, `padstacks`, and `pin_pad_maps`, with discovered root object kinds reflecting the pool subdirectory rather than the generic object fallback. `project validate` now checks declared pool directories for root `schema_version`, filename/payload UUID parity, and the first logical library reference graph across units, symbols, entity gates, packages, padstacks, parts, and pin-pad maps while leaving footprint geometry validation out of scope. Journaled raw pool-library create/set now also rejects authored pool shards without `schema_version: 1`, deserializes `units`, `symbols`, `entities`, `parts`, `packages`, and `padstacks` through the engine's canonical pool structs before staging, validates the first `pin_pad_maps` envelope shape, and intentionally leaves `footprints` at identity/path validation until a canonical footprint schema exists. New native scaffolds now emit explicit `RulesRoot` `uuid` / `object_revision` identity while older rules roots without identity remain readable. Resolver and `resolve-debug` now distinguish authored design shards from generated evidence and sidecar metadata, with native authored shards locked as clean authored-design evidence. Ownership enforcement, dirty-state mutation tracking beyond clean resolved files, recovery semantics, richer semantic library editors, and full shard taxonomy are not complete. | Project state is decomposed into source shards with explicit ownership, load order, dirty-state tracking, and recovery semantics. |
| ObjectId / ObjectRevision / ModelRevision | [~] | Resolver now discovers UUID-bearing JSON objects as `DomainObject`s, assigns placeholder `ObjectRevision(0)` unless persisted `object_revision` exists, and computes deterministic `ModelRevision` from sorted shard hashes and object revisions. Full mutable object identity, revision bumps, diffs, and collaboration semantics remain pending. | Every mutable product object has stable identity, per-object revision, and model revision semantics suitable for diff, undo/redo, proposal/apply, collaboration, and artifact provenance. |
| ComponentInstance | [~] | Component mutation and replacement evidence exists for the imported-board write slice. `ProjectResolver` now derives a conservative `ComponentInstance` bridge only when a native schematic symbol and board package share exact `reference` and `part`, exposes the count through `resolve-debug`, and regression tests forbid reference-only or part-only joins. Resolver diagnostics now report unmatched schematic symbols, unmatched board packages, and ambiguous compatibility joins, and ambiguous joins are refused instead of silently grouping multiple symbols/packages. First persisted authored identity now exists through `.datum/component_instances/<uuid>.json` shards: resolver classifies them as `AuthoredDesign`, includes them in `ModelRevision`, validates filename/embedded-id parity, validates revisioned symbol/package refs against resolved objects, inserts `component_instance` domain objects, and lets persisted refs override ambiguous compatibility joins without emitting the legacy ambiguity diagnostic. OperationBatch create/set/delete now commits ComponentInstance shards through the journal and MCP exposes the matching read/write aliases. BOM/PnP export now emits `component_instance_uuid`, compare/drift keys by ComponentInstance when present with package UUID fallback for board-only legacy rows, direct BOM/PnP export/validate/compare can filter expected rows by a selected `VariantOverlay`'s derived fitted population, manufacturing-set export/validate/compare pass `--variant` through to those BOM/PnP rows, and OutputJob create/update/run stores and honors variant context for manufacturing-set BOM/PnP output plus generated ArtifactMetadata. Multi-unit semantics, richer symbol/package role metadata, richer assembly population, and replacement of remaining legacy reference joins outside BOM/PnP remain pending. | Component instances are first-class product objects spanning schematic, board, parts, packages, fields, placement, connectivity attachments, and import provenance. |
| Relationship / variant substrate | [~] | Engine substrate now defines `Relationship`, `RelationshipKind`, `RevisionedRef`, `AuthoredIntentRecord`, `DerivedRelationshipStatus`, `FittedState`, `VariantOverlay`, `RelationshipOverride`, and `DerivedVariantPopulation`. `ProjectResolver` discovers authored `.datum/relationships/*.json` and `.datum/variants/*.json` shards as `AuthoredDesign`, includes relationship/variant shards and objects in `ModelRevision`, derives resolver-only relationship statuses and variant population/applicability maps, exposes relationship/status and variant/population counts through `resolve-debug`, rejects relationship shards that try to persist derived status as source authority, and rejects variant overlays that try to persist derived population. `CreateRelationship`, `SetRelationship`, `DeleteRelationship`, `CreateVariantOverlay`, `SetVariantOverlay`, and `DeleteVariantOverlay` now commit these records through `OperationBatch` + journal staging as one-record authored shards, with undo/redo and missing-shard replay coverage. `project query <root> relationships` / `variants` and MCP `get_relationships` / `get_variants` expose authored records plus derived status/population through read-only resolver-backed query envelopes. Richer variant composition across option links/scopes and first-class UI review remain pending. | Authored relationship bindings, derived relationship status, sparse variants, and derived `NotApplicableForVariant` are separate model surfaces with check/proposal/GUI visibility. |
| Import Map `import_key` | [~] | Engine substrate now defines `ImportMapEntry`, passively collects embedded `import_key` metadata from UUID-bearing source objects, discovers `.datum/import_map/*.json` sidecar shards as `SidecarMetadata`, excludes ImportMap sidecars from `ModelRevision`, validates sidecar entries against resolved objects/source shards, reports missing-object and shard-mismatch diagnostics, and exposes `import_map_count` through `project query <root> resolve-debug`. CLI `project query <root> import-map` and MCP `get_import_map` now expose resolver-validated import-key mappings through read-only provenance envelopes. The first importer-facing identity primitive now exists as `allocate_import_identity`: it reuses the resolved Datum `ObjectId` for an existing `import_key` and otherwise allocates a deterministic new Datum object ID, giving importers a substrate-owned path away from source-format UUIDs. KiCad standalone footprint package import now has an Import Map-aware path that reuses a mapped package identity, otherwise allocates deterministic identity from the same `import_key`, and reports `import_key`, `source_hash`, and `reused_existing_identity` metadata. The substrate now also owns durable first-allocation sidecar persistence through `persist_import_map_shard`, writing canonical `.datum/import_map/*.json` shards with traversal-safe names and resolver round-trip coverage. First native project-root importer wiring now exists through `datum-eda project import-kicad-footprint <root> --source <file> [--pool pool]`: it imports the package/padstacks into a project-local pool through journaled `AddProjectPoolRef`, `CreatePoolPackage`, and `CreatePoolPadstack` operations, makes pool directory package/padstack JSON files resolver-visible, creates the package Import Map sidecar through journaled `CreateImportMapShard`, re-imports the same source by reusing the mapped Datum package ID, and proves undo removes both pool objects and the Import Map shard. Extending board/schematic/Eagle importers to re-import by `import_key`, richer provenance fields, and broader library/pool authoring operations remain pending. | Imported objects carry durable `import_key` mappings that preserve source fidelity without making source-format identity the internal product identity. |
| OperationBatch | [~] | Engine substrate now defines typed `OperationBatch`, `Operation`, provenance, revision guards, deterministic diff metadata, and inverse operation capture for native project-name, native project-rules replacement, granular project-rule create/set/delete, first pool/library package and padstack creation, generic pool-library object creation/replacement/deletion, Import Map shard creation/deletion, proposal metadata creation/update/deletion, native board-package, authored board-pad, authored board-track, authored board-via, authored board-zone, authored board-net, authored board-netclass, authored board-dimension, authored board-text, authored board-keepout, authored board-outline, authored board-stackup, authored board-name, authored production `ManufacturingPlan`, `PanelProjection`, and `OutputJob` creation/set/deletion, authored relationship/variant record creation/set/deletion, and authored schematic-wire, schematic-junction, schematic-noconnect, schematic-label, schematic-port, schematic-bus, schematic-bus-entry, schematic-text, schematic-drawing, and schematic-symbol edits. First pool/library/import/proposal operation vocabulary now covers `AddProjectPoolRef`, `DeleteProjectPoolRef`, `CreatePoolPackage`, `DeletePoolPackage`, `CreatePoolPadstack`, `DeletePoolPadstack`, `CreatePoolLibraryObject`, `SetPoolLibraryObject`, `DeletePoolLibraryObject`, `CreateImportMapShard`, `DeleteImportMapShard`, `CreateProposalMetadata`, `SetProposalMetadata`, and `DeleteProposalMetadata`; the KiCad footprint project import path uses those operations so package/padstack pool files and the Import Map sidecar are journaled source shards rather than bootstrap-only direct writes, generic library object operations make native pool directories such as `symbols`, `units`, `entities`, `parts`, `footprints`, and `pin_pad_maps` replayable, replaceable, undoable, schema-version guarded, and typed for canonical non-footprint pool structs through the journal, and proposal create/review/apply status metadata now stages sidecar writes through the journal rather than direct overwrite. Native board-package operations now cover create, delete, part reference, package reference, value, reference, position, raw layer field, component side, rotation, and locked state through the substrate path; package position moves translate owned board pads by the authored move delta, and component side changes mirror owned authored pads plus persisted component_pads across the package origin, swap side-sensitive pad layer arrays, mirror pad rotation, and create/delete/package reassignment carry deterministic per-component materialization payloads for derived pads/models/graphics and object-revision accounting. Native board-pad operations now cover create, replace/edit, net assignment/clear, and delete using exact pad payloads. Native board-track operations now cover create, replace/edit, and delete using exact track payloads, and route-apply now batches all draw-track proposal actions into one atomic operation batch. Native board-via operations now cover create, replace/edit, and delete using exact via payloads. Native board-zone, board-net, board-netclass, board-dimension, board-text, and board-keepout operations now cover create/delete using exact payloads; board-net, board-netclass, board-dimension, board-text, and board-keepout also cover replace/edit. Project-name edits cover project-manifest field replacement and inverse capture against the project root object; project-rules replacement covers the shard-level `rules` array shape, while granular project-rule create/set/delete now uses `RulesRoot` identity, rule UUID identity, rules-root revision bumping, inverse capture, and stale-promoted-rules replay coverage. Rule payloads do not yet carry their own explicit `object_revision`, so set operations preserve rule identity while bumping the containing rules root. Board-outline, board-stackup, and board-name operations cover board-root field replacement and inverse capture against the board root object. Native schematic-wire, schematic-junction, schematic-noconnect, and schematic-bus-entry operations now cover create/delete; schematic-label, schematic-port, schematic-bus, schematic-text, schematic-drawing, and schematic-symbol operations now cover create/replace/delete against schematic sheet shards through the same operation vocabulary. Schematic bus-entry writes reject missing/cross-sheet bus references and optional missing/cross-sheet wire references at the engine operation boundary. Schematic symbol set operations also synchronize nested UUID-bearing field/pin object membership so promoted-tip replay keeps model revisions aligned. `BumpObjectRevision` remains as a narrow proof operation. Remaining richer semantic library editors, canonical footprint schema validation, richer rule schemas, check, artifact/generated-evidence, and broader proposal operation vocabulary remain pending. | Product edits are grouped as atomic, typed batches with validation, deterministic ordering, journal entries, revision updates, and surfaced results. |
| `commit()` / journal / recovery | [~] | `DesignModel::commit()` applies operation batches in memory, rejects stale model revisions, computes before/after `ModelRevision`, and records `TransactionRecord` metadata. `commit_journaled()` stages touched shard bytes from journal-materialized source state, captures inverse operations for project-rules, board-package, board-pad, board-track, board-via, board-zone, board-net, board-netclass, board-dimension, board-text, board-keepout, board-outline, board-stackup, board-name, schematic-wire, schematic-junction, schematic-noconnect, schematic-label, schematic-port, schematic-bus, schematic-bus-entry, schematic-text, schematic-drawing, and schematic-symbol writes, fsyncs staged data, appends canonical transaction JSONL to `.datum/journal/transactions.jsonl`, fsyncs the journal, atomically promotes staged shard files, and `ProjectResolver` replays persisted transactions on resolve. Replay now accepts production-only promoted histories by comparing final production shard state against the journal when transient replay would require already-deleted intermediate files. Replay skips duplicate transaction IDs, reports duplicate/parse/conflict/chain/after-revision/link diagnostics, preserves the valid prefix, handles already-promoted board, rules-root, and schematic-sheet shard tips, reconstructs missing/stale authored production shards for `ManufacturingPlan`, `PanelProjection`, and `OutputJob` create/delete records, accounts for create/delete/set object membership while validating replayed source shards, replays source-shard hashes during promoted-tip validation for source-changing operations, and append idempotency is tip-scoped so historical duplicate transaction IDs are refused. Materialized shard readback now skips replay when the promoted source hash already matches the accepted shard hash, preventing same-process double-application of source-changing rules operations while still replaying stale promoted files. Journal append now treats newline-terminated records as the durable prefix, truncates torn trailing JSONL fragments before append, fsyncs the repaired journal length, and still refuses complete malformed lines. `.datum/journal/cursor.json` now stores a durable applied-transaction cursor, resolver validates malformed/out-of-range/behind cursors, normalizes stale behind cursors to the journal tip, and `journal-list` plus canonical `datum-eda journal list` expose append-only undo/redo availability from the journal tip. First compensating undo/redo exists for invertible project-rules, board-package, board-pad, board-track, board-via, schematic-wire, schematic-junction, schematic-noconnect, schematic-label, schematic-port, schematic-bus, schematic-bus-entry, schematic-text, schematic-drawing, and schematic-symbol transactions through `DesignModel::commit_journal_undo`, `DesignModel::commit_journal_redo`, `project undo`, `project redo`, canonical `datum-eda journal undo`, and canonical `datum-eda journal redo`; undo records `undo_of`, redo records `redo_of`, and links survive reopen. Canonical `datum-eda journal show` reads full transaction records by UUID. MCP exposes both flat journal compatibility methods and canonical `datum.journal.list`, `datum.journal.show`, `datum.journal.undo`, and `datum.journal.redo` aliases. A normal transaction after undo now clears redo availability, `project redo` reports that the redo stack was cleared, and resolver replay rejects externally injected stale redo records after branch commits. `set-project-rules`, `create-project-rule`, `delete-project-rule`, `place-board-component`, `move-board-component`, `set-board-component-part`, `set-board-component-package`, `set-board-component-layer`, `delete-board-component`, `place-board-pad`, `edit-board-pad`, `set-board-pad-net`, `clear-board-pad-net`, `delete-board-pad`, `draw-board-track`, `edit-board-track`, `delete-board-track`, `place-board-via`, `edit-board-via`, `delete-board-via`, `place-board-zone`, `delete-board-zone`, `place-board-net`, `edit-board-net`, `delete-board-net`, `place-board-net-class`, `edit-board-net-class`, `delete-board-net-class`, `place-board-dimension`, `edit-board-dimension`, `delete-board-dimension`, `place-board-text`, `edit-board-text`, `delete-board-text`, `place-board-keepout`, `edit-board-keepout`, `delete-board-keepout`, `set-board-outline`, `set-board-stackup`, `set-board-name`, `add-default-top-stackup`, `draw-wire`, `delete-wire`, `place-junction`, `delete-junction`, `place-noconnect`, `delete-noconnect`, `place-label`, `rename-label`, `delete-label`, `place-port`, `edit-port`, `delete-port`, `create-bus`, `edit-bus-members`, `place-bus-entry`, `delete-bus-entry`, `place-schematic-text`, `edit-schematic-text`, `delete-schematic-text`, `place-drawing-line`, `place-drawing-rect`, `place-drawing-circle`, `place-drawing-arc`, `edit-drawing-line`, `edit-drawing-rect`, `edit-drawing-circle`, `edit-drawing-arc`, `delete-drawing`, all native schematic symbol mutation commands, direct `route-apply`, and route-proposal artifact apply now use the journaled substrate path; board-component mutation readbacks and package-materialization pre-reads resolve journal state before reporting or deriving replacement payloads; multi-track route apply and place/edit/delete track plus place/edit/delete via/zone/net/netclass/dimension/text/keepout plus edit net/netclass/dimension/text/keepout and set outline/stackup/name are locked by journal-list or undo/redo regressions, rules replacement/create/delete are locked by stale-promoted-rules query and undo regressions, schematic-wire, schematic-junction, schematic-noconnect, schematic-label, schematic-port, schematic-bus, schematic-bus-entry, schematic-text, schematic-drawing, and schematic-symbol create/delete/set are locked by journal-list and stale-promoted-sheet replay regressions, authored production record creation is locked by a missing-shard replay regression, delete target lookup for those schematic primitives resolves journal-materialized sheet state before committing deletion, label/port/bus/bus-entry/text/drawing/symbol queries plus schematic summary counts resolve journal-materialized sheet state, core board query helpers now resolve stale promoted board files through journal replay, Gerber outline/copper/mask/paste/silkscreen/mechanical export/validate/compare now read resolver-materialized board state, Gerber plan/set discovery resolves stale promoted stackup files through journal replay, native CSV/Excellon drill export/validate/compare plus drill hole-class reporting now read resolver-materialized board state, manufacturing report/manifest/inspect/export/validate/compare wrappers now resolve journal-materialized board state, and forward-annotation review decisions moved out of direct `project.json` rewrites into `.datum/forward_annotation_review/review.json` with legacy embedded review read fallback. The forward-annotation review-state sidecar remains an explicit tracked exception confined to `command_project_forward_annotation_review_state.rs`; the guard verifies its path and forbids it from writing `project.json` or using direct `std::fs::write`. The former GUI board-text private writer module has been deleted, selected GUI text edits route through terminal-prefilled journaled CLI commands, and the private-writer guard now covers both the forward-annotation review mutation modules and GUI board-text regression names. Full crash recovery mode UX, universal operation inverses, migration of remaining private write paths, and resolver-backed fabrication/export readers beyond those CAM layers remain pending. | Commits persist operation batches through a journal with crash recovery, idempotency, replay, and clear failure boundaries. |
| proposal / apply | [~] | Engine substrate now defines `Proposal`, `ProposalStatus`, `ProposalSource`, and `ProposalRef`; `ProjectResolver` discovers `.datum/proposals/*.json` as sidecar proposal metadata excluded from `ModelRevision`, rejects proposal sidecars whose filename does not match the embedded `proposal_id`, `DesignModel` exposes proposals, and `apply_accepted_proposal()` requires `accepted` status plus current `expected_model_revision` before applying the embedded `OperationBatch` through `commit_journaled()`, then includes a journaled `SetProposalMetadata` status update in the same accepted model-change transaction, carrying that transaction id as the applied proposal id. `ProposalStatus` now includes `deferred`; `review_proposal_status()` centralizes draft-to-accepted/deferred/rejected transitions, refuses applied/non-draft proposal edits, and refuses to mark stale or missing-revision-guard drafts as accepted while still allowing stale drafts to be deferred or rejected. `ProposalApplyValidation` now provides shared typed blocker codes (`missing_acceptance`, `stale_model_revision`, `missing_revision_guard`) used by CLI validation and apply rejection. `create_draft_proposal_from_batch()` now provides generic arbitrary proposal authoring for typed `OperationBatch` payloads: it stamps or validates the prepared model revision guard, dry-runs the batch for operation validation/affected-object preview, commits draft proposal metadata through journaled `CreateProposalMetadata`, and never mutates authored design state. Producer-specific production proposal builders now cover OutputJob, ManufacturingPlan, and PanelProjection create/update/delete: canonical `datum-eda proposal create|update|delete-output-job`, `create|update|delete-manufacturing-plan`, and `create|update|delete-panel-projection` commands, flat MCP `*_proposal` tools, and canonical `datum.proposal.create|update|delete_*` aliases build the same typed operation batches as direct CRUD, persist draft proposals through journaled proposal metadata operations, leave authored production shards unchanged until review/apply, and then rely on the generic proposal gateway. Legacy `project ... --as-proposal` compatibility commands remain available. `project query <root> resolve-debug` exposes `proposal_count`; CLI `project query <root> proposals`, canonical `datum-eda proposal create <root> --batch <operation-batch.json> --rationale <text>`, `datum-eda proposal list <root>`, MCP `create_proposal`, `get_proposals`, `datum.proposal.create`, and `datum.proposal.list` expose generic proposal creation/query. CLI `project show-proposal`, `validate-proposal`, `defer-proposal`, `review-proposal`, and `apply-proposal`; canonical `datum-eda proposal show`, `preview`, `validate`, `review`, `defer`, `reject`, `accept-apply`, and `apply`; flat MCP `show_proposal`, `preview_proposal`, `validate_proposal`, `defer_proposal`, `reject_proposal`, `review_proposal`, `accept_apply_proposal`, and `apply_proposal`; and canonical MCP `datum.proposal.show`, `preview`, `validate`, `review`, `defer`, `reject`, `accept_apply`, and `apply` now provide single-proposal inspection, explicit stale/prepared-against validation, draft deferral/rejection, review, and apply surfaces. `accept-apply` composes accepted review plus guarded apply without bypassing proposal policy. GUI production status now loads persisted proposal summaries plus best-effort validation state, and the Outputs dock renders proposal rows with terminal-command actions for proposal list/show, read-only preview, validate, defer, reject, and accept-apply so GUI review still routes through the same CLI proposal gateway while exposing the stored-proposal diff path needed by ghost-review consumers. `project route-apply` now wraps draw-track operations in a persisted accepted proposal and applies it through the proposal gateway. Exported route proposal artifacts now embed the engine `Proposal`, and artifact apply prefers that embedded proposal with legacy action replay retained for older artifacts. Forward-annotation exports/filtering now embed an engine `Proposal` for review-accepted self-sufficient value updates and board-component removals; artifact apply refuses executable self-sufficient artifacts that lack an embedded substrate proposal, and direct/reviewed self-sufficient forward-annotation applies now route through the proposal gateway while explicit-input add/package-resolution actions remain on the legacy path until their substrate operation mappings are complete. Richer direct GUI proposal authoring/apply surfaces and proposal builders for remaining production-adjacent operations remain pending. | All AI/tool-suggested changes can be proposed, inspected, checked, and applied through one substrate path with stable IDs and revision guards. |
| CheckRun / CheckFinding | [~] | ERC/DRC/check reports exist across older engine, CLI, and MCP slices. `project query <root> check-run` and canonical `datum-eda check run <root>` now expose and persist a resolver-keyed `check_run_v1` generated-evidence record under `.datum/check_runs/<check_run_id>.json` with deterministic run/finding IDs, model revision, summary, ordered findings, raw legacy report, and artifact-linked findings for resolver-discovered invalid generated artifacts. Native schematic check inputs now build sheets from `ProjectResolver` materialized source-shard values rather than promoted sheet files; a stale-promoted-sheet regression proves a journaled no-connect edit changes CheckRun ERC findings even when the promoted sheet omits the marker. Invalid artifact metadata raises check status to `error`, increments summary error counts, and emits deterministic `artifact_validation_invalid` findings with the artifact id, file payload, and persisted production projection proofs for generated Gerber copper / Excellon drill evidence. Resolver-discovered proposals can now link back to findings through semantic `Proposal.finding_fingerprints`; transitional matching still accepts older proposal sidecars that stored finding IDs under that field. DRC violation IDs are now deterministic across the native DRC check module, and check-run regression coverage locks stable process-aperture standards findings for missing pad mask expansion and paste reduction. Resolver-derived unfilled zone fills now emit deterministic `zone_fill_unfilled` findings and raise check-run status to `error`. Persisted `CheckFinding` records now carry target-compatible fields for `fingerprint`, `domain`, `rule_id`, `status`, `primary_target`, `related_targets`, `message`, `evidence`, `waiver_refs`, and `deviation_refs`; `fingerprint` is now a deterministic `sha256:` semantic hash over domain, rule id, primary target, and normalized evidence, while `finding_id` remains revision-keyed run identity. `WaiverTarget::Fingerprint` now exists in the shared waiver model, and check runs apply fingerprint-scoped native waivers by keeping waived findings visible, marking `status=waived`, filling `waiver_refs`, and recomputing summary severity counts from normalized findings. `project waive-finding <root> --fingerprint <sha256:...> --rationale <text>` and canonical `datum-eda check waive <root> --fingerprint <sha256:...> --rationale <text>` now commit fingerprint-scoped schematic waivers through `OperationBatch`, staged schematic-root source-shard writes, the append-only transaction journal, and journal undo/redo; `project accept-deviation` and canonical `datum-eda check accept-deviation` do the same for accepted deviations. MCP exposes both flat compatibility names (`waive_finding`, `accept_deviation`, `generate_standards_repair_proposals`) and canonical aliases (`datum.check.waive`, `datum.check.accept_deviation`, `datum.check.repair_standards`). `project generate-standards-repair-proposals` and `datum-eda check repair-standards` now create opt-in draft `ProposalSource::Check` repair proposals for process-aperture DRC findings by grouping mask/paste findings per pad into `SetBoardPad`; track-width DRC findings produce direct `SetBoardTrack` geometry proposals, and via hole/annular DRC findings produce direct per-via `SetBoardVia` geometry proposals while preserving netclass rules as the standard authority. Standards profiles, persisted check history UX beyond the latest deterministic record, additional repair families for clearance/silk/connectivity/topology, and proposal review/apply UX remain pending. | Checks produce first-class runs and findings with provenance, affected objects, waivers, revisions, severity policy, and artifact linkage. |
| ZoneFill | [~] | Engine substrate now defines `ZoneFillState` and `ZoneFill`; `ProjectResolver` derives native board zones as explicit `Unfilled` zone fills with source zone revision, model revision, empty islands, and no provenance. ZoneFill generated evidence can now persist under `.datum/zone_fills/<zone_id>.json`, resolves as `GeneratedEvidence`, and is marked `Stale` if its stored model/source-zone revision no longer matches the current resolved model. Resolver validation rejects invalid generated ZoneFill records before they can become renderable copper: `Filled` records must carry provenance plus at least one closed, non-degenerate, non-self-intersecting island, and `Unfilled`/`Unsupported` records must not carry renderable islands. `project query <root> resolve-debug` exposes `zone_fill_count`, `project query <root> zone-fills` now returns a resolver-backed `zone_fills_query_v1` envelope, MCP exposes the same read surface as `get_zone_fills`, and `project query <root> check-run` emits deterministic `zone_fill_unfilled`, `zone_fill_stale`, or `zone_fill_unsupported` findings so authored zones are never silently treated as filled copper. Native copper Gerber export/validation/comparison now uses a ZoneFill-backed projection adapter: only `ZoneFill{Filled}.islands` become renderable copper regions, while `Unfilled`, `Stale`, and `Unsupported` states remain non-renderable hard-check states. Canonical `datum-eda check fill-zones <root> [--zone <uuid>] [--net <uuid>]`, compatibility `project fill-zones`, MCP `fill_zones`, and canonical MCP `datum.check.fill_zones` now commit journaled `SetZoneFill` generated-evidence operations with inverse delete/restore behavior for undo, redo, and replay; `SetZoneFill`/`DeleteZoneFill` payloads are validated against operation ids before model application and staging, and `fill-zones` captures `previous_zone_fill` from resolver/journal-materialized evidence so undo restores stale or missing-promoted prior fills rather than derived `Unfilled` placeholders. The engine-owned bounded solver emits `ZoneFill{Filled}` with one island equal to the authored polygon for closed, non-degenerate, no-thermal zones when intersecting same-layer authored board pads, tracks, and vias are on the same net; packages alone no longer block fill generation. It also supports rectangular foreign pad/via/orthogonal-track cutouts for rectangular zones by inflating each obstacle by the zone netclass clearance and emitting rectangular islands around strictly-contained, non-overlapping inflated obstacles, with Gerber copper export rendering those islands as production regions. Non-orthogonal different-net tracks, overlapping/touching obstacles, unresolved component pads, keepouts, thermals, missing/nonpositive clearance basis, general clearance subtraction, antipads, and arbitrary obstacle-avoidance cases remain explicit `Unsupported` evidence with provenance rather than fake copper. Current resolver output without persisted generated evidence is still `Unfilled`, so authored zone boundaries are withheld from emitted copper, `unfilled_zone_count` / `unfilled_zone_ids` report the blocked zones, and supplied Gerber regions compare as extra until a supported fill producer supplies `Filled` islands. Remaining gaps are the general pour solver, GUI rendering, and richer persisted fill provenance. | Zone fill is represented as a product operation/result with deterministic generated geometry, invalidation, check integration, and artifact provenance. |
| OutputJob / artifacts | [~] | Engine substrate now defines shared `PanelProjection`, `PanelBoardInstance`, `ManufacturingPlan`, `OutputJob`, `OutputJobRun`, `ArtifactRun`, `OutputJobRunProvenance`, `OutputJobLogEntry`, `ArtifactMetadata`, `ArtifactFile`, `ArtifactProductionProjection`, `ArtifactKind`, and validation/status vocabulary; `ArtifactKind` now distinguishes `gerber_set` from full `manufacturing_set` output bundles. `ProjectResolver` discovers authored `.datum/panel_projections/*.json`, `.datum/manufacturing_plans/*.json`, and `.datum/output_jobs/*.json` shards as design authority and derived `.datum/output_job_runs/*.json`, `.datum/artifact_runs/*.json`, plus `.datum/artifacts/*.json` evidence shards without letting generated evidence change `ModelRevision` or create `DomainObject`s. Authored production records now have typed create/delete operations and journaled per-object shard staging, and `OutputJob` now also has a replacement `SetOutputJob` operation with inverse capture, object-revision bumping, stale-promoted replay coverage, and `project update-output-job` CLI editing for name/manufacturing-plan/variant links. `project delete-output-job` now commits `DeleteOutputJob` through the same journal path, removes the authored shard, and is locked by query/undo/redo regression coverage. `project create-panel-projection`, `project update-panel-projection`, `project delete-panel-projection`, `project create-manufacturing-plan`, `project update-manufacturing-plan`, `project delete-manufacturing-plan`, and `project create-gerber-output-job` create/edit/delete production records through the substrate path rather than direct JSON writes; delete guards refuse dangling ManufacturingPlan-to-PanelProjection and OutputJob-to-ManufacturingPlan references. Promoted production-journal replay now preserves the accepted journal tip revision when materialized production shards already match the journal, so later production deletes can append and undo/redo remains available after reopen. Generated evidence manifests now persist through engine substrate helpers (`persist_artifact_metadata`, `persist_output_job_run`, `persist_artifact_run`) rather than CLI-local JSON writers; artifact metadata rejects absolute/parent-traversal artifact file paths, resolver rejects artifact/run manifests whose filename does not match the embedded id, and generated evidence shards are asserted as `GeneratedEvidence`. `project create-panel-projection` creates a production-only panel target from board instances without mutating board geometry; `project create-manufacturing-plan --panel-projection <uuid>` targets that panel; `project create-gerber-output-job --manufacturing-plan <uuid>` links Gerber output jobs to manufacturing intent; `project create-output-job --variant <uuid>` stores variant context; `project update-output-job --variant/--clear-variant` journals variant edits with undo coverage; and `project query <root> panel-projections`, `manufacturing-plans`, and `output-jobs` round-trip the links. `project export-gerber-set` writes linked artifact metadata plus a deterministic succeeded `OutputJobRun` log; when launched from the GUI PTY it records structured `gui_terminal` run provenance plus human-readable terminal session/context log entries. `project validate-gerber-set` persists artifact validation state. `project export-manufacturing-set` now writes a full manufacturing-set `ArtifactMetadata` record and succeeded `OutputJobRun` covering BOM, PnP, drill CSV, Excellon, and Gerber files, with the same structured GUI-terminal provenance when inherited. `project run-output-job` now honors stored OutputJob variant context for manufacturing-set BOM/PnP rows and generated ArtifactMetadata. Gerber-set and manufacturing-set `ArtifactMetadata.production_projections` now persist live-production projection proofs for generated Gerber copper and Excellon drill bytes, output-job logs record the same proof trail, artifact queries expose it, and validation updates preserve it. `project validate-manufacturing-set` updates that artifact validation state, and invalid manufacturing-set metadata feeds `project query <root> check-run` as an `artifact_validation_invalid` finding. Manufacturing-set report, manifest, inspect, validate, compare, and export now share the same T0 projection helper for expected artifact entries and Gerber plan context; export parity is locked by a regression that compares manifest filenames to exported artifact metadata and artifact view paths. MCP exposes panel projection, manufacturing plan, output-job query/authoring/update/delete, and manufacturing-set export/validation evidence methods over the CLI bridge, including journaled panel/manufacturing update/delete methods. GUI workspace load now queries `output-jobs`, `manufacturing-plans`, `panel-projections`, and persisted proposal sidecars opportunistically for native projects, stores aggregate/per-job/per-plan/per-panel/per-proposal `ProductionStatus`, renders output-job count/artifact count/latest status in the side-panel chrome, exposes a read-only `OUTPUTS` dock tab listing panel projection summaries, manufacturing plan summaries, proposal summaries and terminal actions, job names/status/run counts/artifact counts/latest run IDs, capped artifact kind/file path/hash summaries, and production projection proof rows, and marks production refresh pending after terminal newline submission so subsequent PTY output can reload only `ProductionStatus` without parsing shell text, keeps pending refresh alive across unchanged early command output, and performs a final refresh attempt when the terminal command exits. First live-production equivalence metadata now exists for Gerber copper and Excellon drill export/validation: both surfaces emit a `production_projection` envelope with projection kind, projection contract, source model revision, byte count, and SHA-256 derived from the same resolver-materialized generation path used for the artifact bytes. Concrete panel production geometry editing, artifact selection/opening, richer artifact drill-down beyond capped summaries, and broader live-production projection families remain pending. | Outputs are modeled as jobs with inputs, revisions, produced artifacts, logs, status, reproducibility metadata, and CLI/MCP/GUI visibility. |
| PTY terminal | [~] | GUI terminal lane now opens a Unix PTY, starts `$SHELL` in the active project root with the slave as controlling terminal, routes terminal-focused printable keys, Enter, Backspace, Tab, arrows, Home/End, Escape, and paste text to the PTY master, sends Ctrl-C as SIGINT to the terminal process group, supports Ctrl+Shift+K process-group termination and Ctrl+Shift+R restart from the original project root, suppresses canvas hotkeys while the terminal is focused, propagates dock/window size to the PTY with `TIOCSWINSZ`, streams raw PTY bytes into a persistent first screen-row model, preserves partial prompts without requiring newline output, handles newline/carriage-return/backspace/delete/tab basics, decodes split UTF-8 safely, strips split CSI sequences, tracks row/column cursor state, supports `CSI K` erase-in-line modes 0/1/2, `CSI J` erase-display modes 0/1/2/3, `CSI nA` / `CSI nB` vertical cursor moves, `CSI nC` / `CSI nD` horizontal cursor moves, `CSI G` absolute columns, `CSI row;col H` / `f` absolute cursor positioning, display-neutral multi-parameter SGR swallowing without byte leakage, maps plain Ctrl-C to PTY interrupt while Ctrl+Shift+C copies terminal scrollback text to the clipboard, surfaces the scrollback-copy and paste shortcuts directly in the terminal dock, reports terminal lifecycle status as running/exited/terminated, and no longer presents the lane as read-only or manually echoes submitted commands. Terminal-launched shells now receive canonical first-slice context variables `DATUM_PROJECT_ROOT`, `DATUM_PROJECT_ID`, `DATUM_MODEL_REVISION`, `DATUM_CONTEXT_ID`, `DATUM_SESSION_ID`, `DATUM_DISCOVERY`, and `DATUM_CLI=datum-eda`, while retaining legacy compatibility aliases `DATUM_SOURCE_REVISION`, `DATUM_TERMINAL_SESSION_ID`, `DATUM_TERMINAL_CONTEXT`, and `DATUM_LEGACY_CLI=eda`. `.datum/gui-terminal-context.json` carries a `datum_terminal_context_v1` discovery envelope derived from the active project/scene with canonical context/session/model fields, the current `ProductionStatus` snapshot, explicit terminal-launched `agent_commands` templates (`codex`, `claude`, `aider`, inspect context, refresh context), and explicit context, canonical check (`run`, `list`, `show`, `profiles`, `repair-standards`, `waive`, `accept-deviation`), canonical proposal (`list`, `show`, `preview`, `validate`, `review`, `defer`, `reject`, `accept-apply`, `apply`), production, journal, and resolver-query command templates; current Gerber/manufacturing export commands inherit those env vars and persist structured `OutputJobRun.provenance` plus ordered terminal-origin log entries. Terminal discovery now advertises canonical `datum-eda journal list|show|undo|redo` templates instead of legacy `project query` journal routes, and resolver query templates for schematic sheets/hierarchy plus import map, relationships, variants, and zone fills now use canonical `datum-eda query <family>` commands in both GUI-written and CLI-refreshed discovery envelopes. Focused coverage proves a shell command executes through the PTY, the PTY accepts resize calls, exit status is emitted, SIGTERM termination emits a signal exit, partial prompt bytes render, carriage-return rewrites a row by cursor column, erase-in-line clears progress tails, row-addressed cursor movement rewrites prior terminal rows, erase-display clears stale terminal rows, split ANSI color escapes do not leak bytes, split UTF-8 decodes once complete, Ctrl-C remains interrupt while Ctrl+Shift+C is a copy handoff, terminal scrollback copy text is stable and trims trailing blank rows, render coverage proves the terminal dock advertises copy/paste shortcuts and canonical journal list/undo/redo handoffs, a real shell can read the injected Datum context including terminal agent launch templates, and the CLI helper converts GUI terminal context into structured output-job provenance plus ordered log entries. Full VT control-sequence screen rendering, multi-tab lifecycle, pointer selection ranges, and richer restart UX/state history remain pending. | A PTY-backed terminal can run project-scoped jobs/commands with streamed output, cancellation, exit status, and artifact/job linkage. |
| CLI/MCP taxonomy `datum-eda` | [~] | Current CLI/MCP surfaces are inventoried in `specs/SPEC_PARITY.md`; runtime MCP catalog and CLI command counts are locked by parity gates. First canonical CLI taxonomy slice now exists as read-only `datum-eda context get|refresh|session-events`, returning the GUI terminal discovery envelope from `DATUM_DISCOVERY`, `DATUM_TERMINAL_CONTEXT`, `--path`, or `--project-root`, with optional `--session` mismatch rejection; `context refresh` persists the enriched envelope back to the resolved context file, while `context session-events` resolves `.datum/tool-sessions/<session>.events.jsonl` and returns parsed `datum_tool_session_events_v1` tool-session provenance without touching the project journal. Canonical query aliases now exist for resolver-backed native summary, component instances, schematic sheets/symbols/labels/ports/buses/bus entries/no-connects/hierarchy/nets/diagnostics, relationships, variants, import map, zone fills, panel projections, manufacturing plans, and output jobs; legacy imported-design reads remain under `query imported <file> <what>` plus the historical `query <file> <what>` compatibility parser. MCP already exposes the matching first query family through `datum.query.board_summary`, `components`, `netlist`, `schematic_summary`, `zone_fills`, `component_instances`, `relationships`, `variants`, `import_map`, `panel_projections`, `manufacturing_plans`, and `output_jobs` over compatibility dispatch methods. Canonical ComponentInstance write aliases now exist as `datum.component_instance.bind`, `set`, and `delete` over the journal-backed compatibility dispatch methods. Native library-object authoring now has CLI/MCP journal and query paths through `datum-eda project create-pool-library-object <root> --pool <pool> --kind <kind> --object <uuid> --from-json <file>`, typed `create-pool-unit`, typed `set-pool-unit-pin`, typed `create-pool-symbol`, typed `add-pool-symbol-line`, typed `add-pool-symbol-rect`, typed `add-pool-symbol-circle`, typed `add-pool-symbol-polygon`, typed `add-pool-symbol-arc`, typed `add-pool-symbol-text`, typed `create-pool-entity`, typed `create-pool-padstack`, typed `create-pool-package`, typed `set-pool-package-pad`, typed `set-pool-package-courtyard-rect`, typed `set-pool-package-courtyard-polygon`, typed `add-pool-package-silkscreen-line`, typed `add-pool-package-silkscreen-rect`, typed `add-pool-package-silkscreen-circle`, typed `add-pool-package-silkscreen-polygon`, typed `create-pool-part`, typed `set-pool-part-metadata`, typed `set-pool-part-parametric`, typed `set-pool-part-orderable-mpns`, typed `set-pool-part-pad-map-entry`, typed `set-pool-part-pad-map`, `set-pool-library-object`, `delete-pool-library-object`, `project query <root> pool-library-objects`, flat MCP `get_pool_library_objects` / `show_pool_library_object`, and canonical `datum.library.*` aliases, with public paths computed as `<pool>/<kind>/<uuid>.json` rather than user-supplied relative paths, typed unit pins rejecting blank names, duplicate pin IDs, and unsupported direction enums, typed symbols rejected unless their referenced unit is present in the resolved model, typed symbol lines rejecting zero-length endpoints and nonpositive stroke widths, typed symbol rectangles rejecting zero-area bounds and nonpositive stroke widths, typed symbol circles rejecting nonpositive radius or stroke width, typed symbol polygons preserving vertices, closed state, and stroke width while rejecting malformed vertices, nonpositive width, too few vertices, or closed polygons with fewer than three vertices, typed symbol arcs rejecting nonpositive radius or stroke width while preserving center, radius, start angle, end angle, and stroke width, typed symbol text rejecting blank text while preserving authored position and rotation, typed entities rejected unless their gate unit/symbol pair is consistent, typed padstacks validating circle/rect aperture dimensions before commit, typed packages rejected unless their initial pad references an existing padstack on a positive layer, typed package pads rejecting blank names, duplicate pad IDs, missing padstacks, and nonpositive layer ids, typed package courtyards rejecting zero-area rectangles, malformed polygon vertices, or polygons with fewer than three vertices while preserving the accepted closed polygon, typed package silkscreen lines rejecting zero-length endpoints and nonpositive stroke widths, typed package silkscreen rectangles rejecting zero-area bounds and nonpositive stroke widths, typed package silkscreen circles rejecting nonpositive radius or stroke width, typed package silkscreen polygons preserving vertices, closed state, and stroke width while rejecting malformed vertices, nonpositive width, too few vertices, or closed polygons with fewer than three vertices, typed parts rejected unless their referenced entity and package exist with a supported lifecycle, typed part metadata edits preserving omitted fields while rejecting empty no-op requests, unsupported lifecycle values, and out-of-range JEP106 manufacturer codes, typed part parametric edits rejecting malformed entries, blank keys, duplicate request keys, or unsupported merge modes, typed part orderable-MPN edits rejecting blank values, duplicate request MPNs, or unsupported merge modes, typed part tag edits rejecting blank values, duplicate request tags, or unsupported merge modes, and typed part pad-map authoring rejected unless the package pad, entity gate, and gate unit pin all resolve, with bulk `merge` / `replace` mode rejecting duplicate request pads before commit. Canonical check aliases now exist as `datum-eda check run`, `check repair-standards`, `check waive`, and `check accept-deviation`, with legacy imported-design checks retained as `check imported`; MCP exposes `datum.check.run`, `datum.check.repair_standards`, `datum.check.waive`, and `datum.check.accept_deviation` over the existing compatibility dispatch methods. Canonical proposal aliases now exist as `datum-eda proposal list`, `show`, `preview`, `validate`, `review`, `defer`, `reject`, `accept-apply`, and `apply`; MCP exposes `datum.proposal.list`, `show`, `preview`, `validate`, `review`, `defer`, `reject`, `accept_apply`, and `apply` over canonical CLI-backed compatibility dispatch methods; `proposal preview` / `datum.proposal.preview` dry-runs a stored proposal batch on a cloned resolved model and returns `proposal_preview_v1` with classified `CommitDiff`, affected objects, validation state, and no shard writes for GUI ghost-review consumers. Canonical journal aliases now exist as `datum-eda journal list`, `show`, `undo`, and `redo`; MCP exposes `datum.journal.list`, `show`, `undo`, and `redo` over the existing compatibility dispatch methods. Canonical artifact aliases now exist as `datum-eda artifact generate`, `start-output-job-run`, `cancel-output-job-run`, `list`, `show`, `files`, `preview`, `compare`, `validate`, `export-manufacturing-set`, and `validate-manufacturing-set`; MCP exposes `datum.artifact.generate`, `list`, `show`, `files`, `preview`, `compare`, `validate`, `export_manufacturing_set`, and `validate_manufacturing_set` over compatibility dispatch methods, while flat lifecycle compatibility methods now bridge to the canonical artifact CLI aliases. MCP `tools/call` now wraps canonical `datum.*` success responses in the target `ok/schema/context/result` envelope, derives context fields from raw CLI payloads when present, preserves top-level raw fields for migration compatibility, and returns `ok:false` target envelopes for canonical tool-level failures. Broader `datum-eda query` coverage remains pending; first MCP boundary result normalization now exists for canonical `datum.context.*`, `datum.check.*`, `datum.artifact.*`, `datum.proposal.*`, `datum.journal.*`, `datum.component_instance.*`, and the first resolver-backed `datum.query.*` set, while deeper family-specific semantics beyond the current compatibility bridge remain pending. | CLI and MCP expose one coherent `datum-eda` product taxonomy aligned to substrate nouns, not milestone-era command accumulation. |
| Spec parity / governance | [~] | `specs/SPEC_PARITY.md` and drift gates lock selected current inventories. `scripts/check_schematic_private_writers.py` is now wired into `scripts/run_drift_gates.sh` and fails if migrated schematic or production authoring modules reintroduce direct JSON writers instead of typed journaled `OperationBatch` commits, if generated-evidence manifest paths reintroduce CLI-local `.datum/artifacts` / `.datum/artifact_runs` / `.datum/output_job_runs` / `.datum/zone_fills` writers instead of substrate persistence helpers or journaled generated-evidence staging, if artifact/run CLI evidence users stop calling the expected `persist_artifact_metadata`, `persist_artifact_run`, or `persist_output_job_run` helpers, if the engine-owned generated-evidence helper grows additional raw `std::fs::write` call sites beyond its single temp-file atomic write, if `gui-app` re-imports board-text private writer helpers instead of terminal-prefilled journaled CLI commands, if retired GUI board-text private writer files reappear, or if forward-annotation review state escapes its explicit `.datum/forward_annotation_review/review.json` sidecar helper. The same guard now classifies non-journaled `project new` bootstrap writes, route-strategy fixture/artifact writes, ZoneFill generated-evidence staging through `zone_fill_journal_ops`, the single engine generated-evidence temp-file writer, and generated Gerber/drill/BOM/PnP export file writes as exact-count exceptions, so new source-writing `write_canonical_json` or `std::fs::write` call sites cannot hide among bootstrap/generated output paths without being deliberately classified. | Governance defines which specs are source of truth, how target/current claims are separated, and which gates prevent stale milestone evidence from driving active scope. |

MCP bridge note: flat compatibility methods and canonical aliases now invoke canonical CLI argv for the first resolver query family, proposals, journals, check waivers/deviations, artifact read/export/validation surfaces, OutputJob execution/lifecycle evidence, and producer-specific OutputJob/ManufacturingPlan/PanelProjection proposal builders. Remaining `project ...` bridge calls are direct production authoring compatibility commands rather than proposal authoring paths.

MCP library taxonomy note: raw native library-object authoring now has canonical aliases `datum.library.create_object`, `datum.library.set_object`, and `datum.library.delete_object` over `create_pool_library_object` / `set_pool_library_object` / `delete_pool_library_object`; typed semantic editor aliases now include `datum.library.create_unit` over `create_pool_unit`, `datum.library.set_unit_pin` over `set_pool_unit_pin`, `datum.library.create_symbol` over `create_pool_symbol`, `datum.library.add_symbol_line` over `add_pool_symbol_line`, `datum.library.add_symbol_rect` over `add_pool_symbol_rect`, `datum.library.add_symbol_circle` over `add_pool_symbol_circle`, `datum.library.add_symbol_polygon` over `add_pool_symbol_polygon`, `datum.library.add_symbol_arc` over `add_pool_symbol_arc`, `datum.library.add_symbol_text` over `add_pool_symbol_text`, `datum.library.set_symbol_pin_anchor` over `set_pool_symbol_pin_anchor`, `datum.library.create_entity` over `create_pool_entity`, `datum.library.create_padstack` over `create_pool_padstack`, `datum.library.create_package` over `create_pool_package`, `datum.library.set_package_pad` over `set_pool_package_pad`, `datum.library.set_package_courtyard_rect` over `set_pool_package_courtyard_rect`, `datum.library.set_package_courtyard_polygon` over `set_pool_package_courtyard_polygon`, `datum.library.add_package_silkscreen_line` over `add_pool_package_silkscreen_line`, `datum.library.add_package_silkscreen_rect` over `add_pool_package_silkscreen_rect`, `datum.library.add_package_silkscreen_circle` over `add_pool_package_silkscreen_circle`, `datum.library.add_package_silkscreen_polygon` over `add_pool_package_silkscreen_polygon`, `datum.library.add_package_silkscreen_arc` over `add_pool_package_silkscreen_arc`, `datum.library.add_package_silkscreen_text` over `add_pool_package_silkscreen_text`, `datum.library.add_package_model_3d` over `add_pool_package_model_3d`, `datum.library.create_part` over `create_pool_part`, `datum.library.set_part_metadata` over `set_pool_part_metadata`, `datum.library.set_part_pad_map_entry` over `set_pool_part_pad_map_entry`, and `datum.library.set_part_pad_map` over `set_pool_part_pad_map`. Resolver-backed native pool-library inspection now exists through `datum-eda project query <root> pool-library-objects`, flat MCP `get_pool_library_objects` / `show_pool_library_object`, and canonical `datum.library.list_objects` / `datum.library.show_object`; pool-model blob verification now exists through `datum-eda project query <root> pool-models`, flat MCP `get_pool_model_blobs`, and canonical `datum.library.pool_models`; flat MCP `gc_pool_model_blobs` exposes `datum-eda project gc-pool-models` dry-run/apply cleanup for orphaned regular hash-matching blobs. The aliases dispatch through the CLI path, inherit the CLI `pool` default for writes, keep public object paths computed as `<pool>/<kind>/<uuid>.json` rather than user-supplied relative paths, reject typed unit pins with blank names, duplicate pin IDs, or unsupported direction enums, reject typed symbols whose referenced unit is absent from the resolved model, reject typed symbol lines with zero-length endpoints or nonpositive stroke widths while preserving authored endpoints and stroke width, reject typed symbol rectangles with zero-area bounds or nonpositive stroke widths while preserving authored bounds and stroke width, reject typed symbol circles with nonpositive radius or stroke width while preserving authored center, radius, and stroke width, reject typed symbol polygons with malformed vertices, nonpositive width, too few vertices, or closed polygons with fewer than three vertices while preserving vertices, closed state, and stroke width, reject typed symbol arcs with nonpositive radius or stroke width while preserving center, radius, start angle, end angle, and stroke width, reject typed symbol text with blank text while preserving authored position and rotation, reject typed symbol pin anchors whose symbol unit or referenced unit pin is missing while preserving the authored unit-pin UUID and symbol-local position, place schematic symbols with pins when `--lib-id` is a pool symbol UUID with authored pin anchors while preserving arbitrary non-UUID lib IDs as unresolved compatibility identifiers, reject typed entities whose initial gate references a missing unit, missing symbol, or symbol/unit mismatch, reject invalid typed padstack aperture/drill dimensions before journal commit, reject typed packages whose initial pad references a missing padstack or nonpositive layer, reject typed package pads with blank names, duplicate pad IDs, missing padstacks, or nonpositive layer ids, reject typed package courtyards with zero-area rectangles, malformed polygon vertices, or polygons with fewer than three vertices while preserving accepted closed polygons, reject typed package silkscreen lines with zero-length endpoints or nonpositive stroke widths, reject typed package silkscreen rectangles with zero-area bounds or nonpositive stroke widths, reject typed package silkscreen circles with nonpositive radius or stroke width, reject typed package silkscreen polygons with malformed vertices, nonpositive width, too few vertices, or closed polygons with fewer than three vertices while preserving vertices, closed state, and stroke width, reject typed package silkscreen arcs with nonpositive radius or stroke width while preserving center, radius, start angle, end angle, and stroke width, reject typed package silkscreen text with blank text while preserving authored position and rotation, reject typed package 3D model paths that are blank, absolute, or traversal paths and malformed transform JSON while preserving the accepted model path and transform, reject typed parts whose referenced entity/package are missing or whose lifecycle is unsupported, reject typed part metadata edits with empty no-op requests, unsupported lifecycle values, or out-of-range JEP106 manufacturer codes, reject typed part parametric edits with malformed entries, blank keys, duplicate request keys, or unsupported merge modes, reject typed part orderable-MPN edits with blank values, duplicate request MPNs, or unsupported merge modes, reject typed part tag edits with blank values, duplicate request tags, or unsupported merge modes, and reject typed part pad-map authoring whose package pad, entity gate, or gate unit pin is missing, including bulk merge/replace requests with duplicate pads. Richer symbol graphics beyond lines/rectangles/circles/text/arcs/polygons, package silkscreen primitives beyond lines/rectangles/circles/text/arcs/polygons.

MCP schematic query taxonomy note: canonical `datum.query.sheets`, `symbols`, `symbol_fields`, `labels`, `ports`, `buses`, `bus_entries`, `noconnects`, `schematic_nets`, `connectivity_diagnostics`, and `design_rules` now expose the existing compatibility schematic query methods through the `datum.*` envelope. `datum.query.hierarchy` is now path-aware for native projects and dispatches to `datum-eda project query <path> hierarchy` when `path` is supplied, while preserving the legacy open-session fallback. This gives GUI panels and agents stable read-side schematic context, including connectivity-derived hierarchy links from persisted sheet-instance port bindings.

MCP proposal taxonomy note: producer-specific production proposal builders now have canonical aliases under `datum.proposal.*`: `create_panel_projection`, `update_panel_projection`, `delete_panel_projection`, `create_manufacturing_plan`, `update_manufacturing_plan`, `delete_manufacturing_plan`, `create_output_job`, `update_output_job`, and `delete_output_job`. The matching canonical CLI proposal aliases dispatch to the same proposal-gateway code path as the compatibility `project ... --as-proposal` commands. The MCP aliases dispatch to the same flat compatibility proposal builders and return the canonical target envelope.

MCP PCB taxonomy note: canonical `datum.pcb.place_component`, `move_component`, `rotate_component`, `flip_component`, `delete_component`, `set_component_reference`, `set_component_value`, `set_component_part`, `set_component_package`, `lock_component`, `unlock_component`, `draw_track`, `edit_track`, `delete_track`, `place_via`, `edit_via`, `delete_via`, `place_zone`, `delete_zone`, `place_pad`, `edit_pad`, `delete_pad`, `set_pad_net`, `clear_pad_net`, `place_net`, `edit_net`, `delete_net`, `place_net_class`, `edit_net_class`, `delete_net_class`, `set_board_name`, `set_outline`, `set_stackup`, `add_default_top_stackup`, `place_keepout`, `edit_keepout`, `delete_keepout`, `place_dimension`, `edit_dimension`, `delete_dimension`, `place_text`, `edit_text`, and `delete_text` are now native-project scoped, require explicit `path` plus operation-specific arguments, and bridge to the matching `datum-eda project ...board-component...`, `...board-track`, `...board-via`, `...board-zone`, `...board-pad`, `...board-net...` / `...board-net-class`, board setup, `...board-keepout`, `...board-dimension`, and `...board-text` commands over journaled native project substrates. Flat `move_component`, `rotate_component`, and `flip_component` remain legacy daemon-session compatibility.

MCP schematic taxonomy note: canonical `datum.schematic.create_sheet_definition`, `create_sheet_instance`, `delete_sheet_instance`, `move_sheet_instance`, `bind_sheet_instance_port`, `unbind_sheet_instance_port`, `draw_wire`, `delete_wire`, `place_junction`, `delete_junction`, `place_noconnect`, `delete_noconnect`, `place_label`, `rename_label`, `delete_label`, `place_port`, `edit_port`, `delete_port`, `create_bus`, `edit_bus_members`, `place_bus_entry`, `delete_bus_entry`, `place_text`, `edit_text`, `delete_text`, `place_drawing_line`, `place_drawing_rect`, `place_drawing_circle`, `place_drawing_arc`, `edit_drawing_line`, `edit_drawing_rect`, `edit_drawing_circle`, `edit_drawing_arc`, `delete_drawing`, `place_symbol`, `move_symbol`, `rotate_symbol`, `mirror_symbol`, `delete_symbol`, `set_symbol_reference`, `set_symbol_value`, symbol metadata, pin override, and symbol field aliases are now native-project scoped, require explicit `path` plus sheet/object arguments, and bridge to the matching journaled `datum-eda project create-sheet-definition`, `create-sheet-instance`, `delete-sheet-instance`, `move-sheet-instance`, `bind-sheet-instance-port`, `unbind-sheet-instance-port`, `draw-wire`, `delete-wire`, `place-junction`, `delete-junction`, `place-noconnect`, `delete-noconnect`, `place-label`, `rename-label`, `delete-label`, `place-port`, `edit-port`, `delete-port`, `create-bus`, `edit-bus-members`, `place-bus-entry`, `delete-bus-entry`, `place-schematic-text`, `edit-schematic-text`, `delete-schematic-text`, `place-drawing-*`, `edit-drawing-*`, `delete-drawing`, `place-symbol`, `move-symbol`, `rotate-symbol`, `mirror-symbol`, `delete-symbol`, `set-symbol-reference`, `set-symbol-value`, `set-symbol-*`, `clear-symbol-*`, `set-pin-override`, `clear-pin-override`, `add-symbol-field`, `edit-symbol-field`, and `delete-symbol-field` commands. `create_sheet_definition` creates the definition shard and updates the schematic root definitions map in one undoable transaction; `create_sheet_instance`, `delete_sheet_instance`, and `move_sheet_instance` update the schematic root instances array through the same journaled substrate; `bind_sheet_instance_port` and `unbind_sheet_instance_port` persist parent-port bindings that make native hierarchy links real.

Terminal discovery note: production object create/update/delete templates now default to canonical proposal-first commands: `datum-eda proposal create|update|delete-output-job`, `create|update|delete-manufacturing-plan`, and `create|update|delete-panel-projection`. Agents launched from Datum discover draft-proposal authoring rather than direct production CRUD or compatibility `project ... --as-proposal` syntax.

Artifact execution note: `datum-eda artifact generate <root> --output-job <uuid>`, MCP `datum.artifact.generate` with `output_job`, and the flat MCP `run_output_job` compatibility method now execute an authored OutputJob through the canonical artifact family; production object CRUD remains on compatibility `project ...` verbs for now, but terminal discovery points those verbs at proposal creation rather than direct mutation.

Terminal discovery note: authored OutputJob execution is now advertised as `datum-eda artifact generate <root> --output-job <uuid>` and lifecycle evidence as `datum-eda artifact start-output-job-run|cancel-output-job-run` instead of the legacy `project run-output-job` / lifecycle compatibility verbs. The GUI terminal command catalog now carries structured `datum.artifact.start_output_job_run` and `datum.artifact.cancel_output_job_run` entries, and the Outputs dock exposes status-aware `START` / `CANCEL` lifecycle handoffs on OutputJob rows without adding a GUI private writer.

Terminal discovery manufacturing-set note: the GUI terminal command catalog and
`datum_terminal_context_v1.production_commands` now advertise canonical
`datum-eda artifact export-manufacturing-set "$DATUM_PROJECT_ROOT" --output-dir <dir> ...`
and
`datum-eda artifact validate-manufacturing-set "$DATUM_PROJECT_ROOT" --output-dir <dir> ...`
templates, matching the artifact CLI/MCP taxonomy instead of leaving those
production commands discoverable only through the written spec.

Terminal agent entry note: the visible GUI `AGENTS` dock entry now focuses the PTY terminal and prints shell launch guidance for installed agents (`codex`, `claude`, `aider`) using the injected `DATUM_DISCOVERY` context, rather than treating the legacy assistant lane as the canonical agent authority.

GUI text edit note: selected board-text Inspector entries now prefill the PTY terminal with canonical `datum-eda project edit-board-text "$DATUM_PROJECT_ROOT" --text <uuid> ...` commands instead of opening assistant-only `/text` commands or mutating board JSON directly. Explicit center edits prefill the current value; edge/cycle controls prefill the next boolean/cycle/step value, with height steppers carrying paired `--height-nm` and `--stroke-width-nm` arguments so proportional scaling still routes through the journaled `SetBoardText` CLI path. The legacy assistant `/text` command parser and completions have been removed, and the old `gui-protocol` board-text private writer modules were deleted rather than left as dormant public helpers.

Open tracking rule:
- Add implementation evidence here before marking a substrate row complete.
- If a row is intentionally deferred, record the governance reason and the
  downstream fidelity work allowed despite the deferral.
- Do not promote legacy milestone completion to substrate readiness without a
  direct product-mechanics contract.

Context envelope correction:
- `datum-eda context get|refresh|session-events|session-activity` no longer returns only the raw GUI discovery
  file. The CLI now enriches compatible `datum_terminal_context_v1` discovery
  payloads with the first `DatumContextEnvelope` fields: `actor_type`,
  `capabilities`, resolver-derived `project_id` / `project_name` /
  `model_revision` when a project root resolves, `accepted_transaction_tip`,
  `visible_artifact_ids`, `visible_check_run_ids`, `output_context`,
  `provenance_seed`, `expires_at`, `refresh_command`, and storage metadata.
  The GUI terminal discovery writer emits the same first-slice fields for
  terminal-launched shells, writes authoritative per-session context files under
  `.datum/terminal-contexts/<session>.json`, keeps
  `.datum/gui-terminal-context.json` as a latest-session compatibility alias,
  and exports `DATUM_DISCOVERY` / `DATUM_TERMINAL_CONTEXT` to the per-session
  file so concurrent terminals do not race through the singleton path. CLI
  project-root lookup now prefers the requested session file before falling
  back to the legacy latest alias. `datum-eda context get` returns the normalized
  envelope without writing it, while `datum-eda context refresh` persists the
  enriched envelope back to the resolved session context file without losing
  GUI-owned typed context fields; `context session-events` returns parsed
  `.datum/tool-sessions/<session>.events.jsonl` records as
  `datum_tool_session_events_v1`, including terminal command `origin`,
  `command_id`, optional `mcp_alias`, and `handoff_mode` fields that distinguish
  board-text prefill commands from executed terminal handoffs. `context session-events`
  and MCP `datum.context.session_events` also expose exact-match `event_kind`,
  `origin`, and `command_id` filters plus `limit` for the newest matching
  events after filters, with returned, matched, and raw event counts.
  `context session-activity` returns a compact
  `datum_tool_session_activity_summary_v1` aggregate over the same filtered
  window for agent orientation. MCP `datum.context.get|refresh|session_events|session_activity`
  continues to bridge through the canonical CLI while its catalog/test doubles
  document the richer envelope and event stream. GUI protocol now defines shared typed session/context
  structs, GUI-authored context files carry `selection_context`,
  `cursor_context`, and `projection_context` derived from `ReviewWorkspaceState`,
  GUI terminals persist `DatumToolSessionMetadata` under
  `.datum/tool-sessions/<session>.json`, and GUI selection, cursor/hover, dock,
  and frame-affecting session events rewrite the same per-session context file
  without minting a new session. Remaining context gaps are richer engine-owned
  session policy/expiry semantics and broader projection/cursor vocabulary
  beyond the current GUI snapshot.

CheckRun row correction:
- ComponentInstance operation correction: `OperationBatch` now covers
  `CreateComponentInstance`, `SetComponentInstance`, and
  `DeleteComponentInstance`; journaled commits stage/promote the authored shard,
  capture inverse operations, support undo/redo, and reconstruct missing
  promoted component-instance shards from accepted journal replay. CLI now
  exposes `project query <root> component-instances`,
  `project bind-component-instance`, `project set-component-instance`, and
  `project delete-component-instance`, with regression coverage proving the
  commands use the journal path and remain undoable. MCP now exposes
  `get_component_instances`, `bind_component_instance`,
  `set_component_instance`, `delete_component_instance`, and canonical
  `datum.query.component_instances` / `datum.component_instance.*` aliases over
  the same CLI bridge. BOM/PnP export now emits `component_instance_uuid`, and
  BOM/PnP compare keys matched/missing/extra/drift evidence by ComponentInstance
  when present, with package UUID fallback for board-only legacy rows.
  Multi-unit semantics, richer symbol/package role metadata, variant-aware
  manufacturing population, and replacement of remaining legacy reference joins
  outside BOM/PnP remain pending.
- The deviation lifecycle is no longer pending for the current compatibility
  slice. `project accept-deviation` and MCP `accept_deviation` now author
  fingerprint-scoped accepted deviations through the native journal; CheckRun
  readback reports `status=accepted_deviation` and `deviation_refs`.
- Canonical CLI now exposes `datum-eda check list` and
  `datum-eda check show --check-run <uuid>` over resolver-discovered persisted
  `.datum/check_runs` generated evidence. MCP now exposes matching
  compatibility tools `get_check_runs` / `show_check_run` plus canonical
  `datum.check.list` / `datum.check.show` aliases over the same CLI bridge.
  Canonical CLI also exposes `datum-eda check profiles <root>` and MCP exposes
  matching `get_check_profiles` / `datum.check.profiles`, now reporting the
  bounded supported profile set `native-combined`, `erc`, `drc`, `standards`,
  `manufacturing`, and `release`. `datum-eda check run <root> --profile <id>`
  and MCP `datum.check.run` with `profile=<id>` persist profile-keyed CheckRun
  evidence, with non-default profiles filtering the current deterministic
  findings by domain or standards-repair rule family; unsupported profile ids
  are rejected. The remaining CheckRun
  proposal-link gap has been narrowed: live and persisted `check_run_v1`
  payloads now preserve compatibility `proposal_refs` while adding structured
  `proposal_links` at run and finding level with proposal status/source,
  rationale, validation blockers, and canonical `datum-eda proposal ...`
  command templates including preview. Live and persisted CheckRun payloads now also carry
  deterministic `profile_basis` and `coverage` entries so evaluated,
  profile-filtered, and not-yet-implemented rule families are explicit instead
  of inferred from missing findings; canonical MCP normalization and GUI
  protocol preserve those fields. Live and persisted `CheckFinding` records now also carry
  inline `explanation` and nullable `suggested_next_action` fields so GUI and
  agents do not need a separate index-addressed explain tool. GUI workspace
  load now best-effort runs the current CheckRun into `ReviewWorkspaceState`,
  the Outputs dock renders a compact latest-run/finding action lane, and those
  actions prefill canonical terminal commands for check show/run, zone-fill
  refresh, standards repair generation, proposal show/accept-apply, waiver,
  and accepted deviation without adding any GUI private writer. Terminal
  discovery context now includes visible check-run ids, visible finding
  fingerprints, the compact check status snapshot, and a profile-specific
  `datum-eda check run "$DATUM_PROJECT_ROOT" --profile <profile>` template for
  in-Datum agents. The Outputs dock now renders standards-basis evidence for
  process-aperture findings, exposes direct profile discovery via
  `datum-eda check profiles "$DATUM_PROJECT_ROOT"`, exposes persisted check-run
  history via `datum-eda check list "$DATUM_PROJECT_ROOT"`, and exposes direct
  `standards` / `release` profile rerun actions through the terminal command
  catalog and matching MCP aliases. Remaining CheckRun UX gaps are deeper
  profile configuration and richer first-class GUI review/apply widgets; the canonical
  waive/deviation taxonomy now exists for the current native-project slice.

Artifact / taxonomy row correction:
- Manufacturing-set T0 projection correction: report, manifest, inspect,
  validate, compare, and export now share
  `native_project_manufacturing_projection` for expected artifact entries and
  Gerber plan context. The older row wording that only listed
  report/manifest/inspect/validate/compare is superseded by this correction.
- Artifact aliases are no longer pending for the current generated-evidence
  evidence slice. Canonical CLI now exposes `datum-eda artifact list`,
  `artifact generate`, `artifact show`, `artifact files`, `artifact preview`,
  `artifact compare`, `artifact validate`, `artifact export-manufacturing-set`,
  and `artifact validate-manufacturing-set`.
- MCP now exposes `datum.artifact.list`,
  `datum.artifact.generate`, `datum.artifact.show`, `datum.artifact.files`,
  `datum.artifact.preview`, `datum.artifact.compare`, `datum.artifact.validate`,
  `datum.artifact.export_manufacturing_set`, and
  `datum.artifact.validate_manufacturing_set`; the daemon bridge routes these
  through the canonical artifact CLI family.
- `artifact preview` / `datum.artifact.preview` verifies a requested safe
  relative file against resolver-owned artifact metadata, reads it from either
  explicit `--artifact-dir` or persisted `ArtifactMetadata.output_dir`, checks
  the on-disk hash against the metadata hash, and returns
  `artifact_file_preview_v1` using real semantic readers for supported
  RS-274X Gerber, Excellon drill inspection, bounded Gerber/Excellon preview
  primitives in nanometer coordinates, and CSV BOM/PnP/drill summaries.
- `artifact generate` / `datum.artifact.generate` now also accepts `bom`,
  `pnp`, and `drill` scopes. `bom` and `pnp` persist independent one-file CSV
  artifact metadata records; `drill` persists an independent drill-family
  artifact covering drill CSV plus Excellon drill, including the Excellon
  production projection proof. Generic `artifact validate` now performs
  family-specific semantic validation for those finer scopes, updates their
  persisted `ArtifactMetadata.validation_state`, and therefore feeds invalid
  BOM/PnP/drill artifacts into the existing artifact check-finding path.
  `project create-output-job --include <gerber-set|manufacturing-set|bom|pnp|drill|all>[,<scope>...]`
  now authors deterministic OutputJob templates for one or more implemented
  artifact scopes through the journaled `CreateOutputJob` path, and generated
  BOM/PnP/drill artifacts attach to a matching authored job when the
  prefix/scope match or when the stored include list contains the generated
  scope. Stored OutputJob variant context is now passed into direct BOM/PnP
  artifact scopes rather than only manufacturing-set generation.
  Direct manufacturing-set export, validate, compare, manifest, and inspect
  now accept `--include <scope>[,<scope>...]`, `--output-job <uuid>`, and
  exact-name `--job <name>`; selected jobs provide default prefix, variant, and
  include scope, duplicate names are rejected as ambiguous, and direct
  manufacturing export writes only the selected artifact families.
  Generic BOM/PnP/drill `artifact generate` executions now persist succeeded
  `OutputJobRun` evidence for linked authored jobs. Unlinked ad hoc finer-scope
  artifact generation persists separate `ArtifactRun` generated evidence so
  artifact history remains visible without inventing an authored `OutputJob`.
  `artifact_generate_v1.generated[]` entries now expose normalized top-level
  `output_job_run`, `output_job_run_path`, `artifact_run`, and
  `artifact_run_path` fields across Gerber-set, manufacturing-set, BOM, PnP,
  and drill scopes, while preserving the family-specific nested report fields.
  Generic multi-scope `run-output-job` now suppresses per-artifact run
  persistence and records one aggregate `OutputJobRun` for the logical command;
  the aggregate run keeps `artifact_id` null for multi-artifact executions and
  records the generated scopes in its log. `OutputJobRun.run_sequence` is
  assigned monotonically per authored job, repeated identical runs create
  distinct generated-evidence records, and `project query <root> output-jobs`
  reports status, execution count, sequence-ordered latest run, and artifact
  linkage for those finer scopes.
  `datum-eda artifact list/show` expose resolver-discovered `ArtifactRun`
  history for ad hoc generated artifacts and linked `OutputJobRun` history for
  artifacts generated by authored jobs; direct Gerber/manufacturing export
  reports now also include the persisted `output_job_run_path` when they create
  an `OutputJobRun`. GUI production status now consumes
  artifact-list evidence, stores ad hoc `ArtifactRun` summaries, can focus the
  latest ad hoc generated artifact, and renders ad hoc artifact-run rows in the
  Outputs dock as clickable artifact evidence. GUI production status also maps
  OutputJob include scopes into explicit normalized job-family labels (`GERBER
  SET`, `MANUFACTURING SET`, `BOM`, `PNP`, `DRILL`), retains the latest run's
  artifact id, and renders those family/run/artifact details in the Outputs
  dock instead of presenting finer-scope jobs as indistinguishable generic
  artifact rows.
- The remaining artifact taxonomy gap is richer family-specific GUI actions.

GUI production artifact drill-down correction:
- GUI terminal-launched production commands now keep production refresh pending
  across unchanged early PTY output, perform a final refresh attempt when the
  terminal exits, and use a bounded event-loop retry so proposal/artifact/output
  rows can refresh even when the command emits no useful trailing output.
- GUI production status now consumes the canonical `datum-eda artifact files`
  contract for a focused artifact discovered from output-job artifacts, stores
  it as `ProductionStatus.focused_artifact`, and renders focused file/hash plus
  production-projection proof rows in the `OUTPUTS` dock.
- Artifact rows in the `OUTPUTS` dock are now clickable hit targets that focus
  the selected artifact from production summaries through a session command.
- Focused artifact file rows are now clickable hit targets that select a file
  proof and render a generated-file viewer block with path/hash. The first
  dedicated viewer classifies Gerber, Excellon/NC drill, drill CSV, BOM, and
  PnP artifact files and attaches matching production-projection proof rows
  when available. Generated artifact metadata now carries optional
  `output_dir`, GUI production summaries/details retain it, production refresh
  can load `artifact_file_preview_v1` for the focused file without guessing
  filesystem layout, and the Outputs dock renders verified preview kind,
  hash status, primitive count, semantic counts, and a lightweight geometric
  CAM viewport from bounded Gerber/Excellon primitives. The CAM viewport now
  has first stateful drill-down controls in `WorkspaceUiState`: click targets
  and Outputs-dock mouse-wheel handling zoom the artifact preview, reset returns
  it to fit view, and geometry/drill toggles gate rendered Gerber and Excellon
  primitive families. The preview body is now a hit-test target, and middle/
  right drag over it pans the generated-artifact viewport through normalized
  `pan_x_ppm` / `pan_y_ppm` state instead of moving the board camera. The
  artifact preview contract now carries bounded CSV columns and sample rows for
  BOM, PnP, and drill-table artifacts, and the Outputs dock renders those rows
  as a small table instead of only showing `row_count`. Focused generated
  artifacts now also expose clickable top-level `datum-eda artifact list`,
  focused-artifact `datum-eda artifact show`, `datum-eda artifact validate`,
  `datum-eda artifact files`, and focused-file `datum-eda artifact preview`
  terminal actions, and the artifact-run section exposes
  `datum-eda artifact compare "$DATUM_PROJECT_ROOT" --before <older> --after <newer>`
  for the latest two distinct generated artifacts, so output-job artifacts can
  be discovered, inspected, compared, and validated from the same drill-down surface as ad hoc artifact-run evidence. The
  remaining GUI artifact UX gap is full generated-artifact drill-down with
  richer layer/family controls, richer artifact-family-specific viewers, and
  independently modeled drill, BOM, and PnP artifact scopes; drill-down data is
  no longer limited to capped output-job aggregate summaries, row counts, or
  the first discovered artifact.

---

## Current Repo Health

Current repo health status (2026-03-25 audit):
- `cargo test -q` currently passes.
- The current `M2` implementation slice is backed by `m2_quality`,
  `m2_perf`, CLI tests, daemon tests, and MCP self-test.
- Milestone completion status and workspace health are currently reconciled.
- API/daemon/MCP test-support monolith risk has been reduced via module splits,
  and file-size budgets are now enforced in CI.

Current M7 delivery rule (2026-04-16):
- opening `M7` work may not advance on a "low resolution but technically
  implemented" basis
- user-facing slices must be intentionally triggerable, externally observable,
  and supported by the minimum interaction/render substrate they depend on
- missing prerequisite work in selection, hit-testing, focus/relatedness,
  visibility, or render-state consistency is not "scope creep"; it is
  prerequisite completion for the slice already being claimed
- working note:
  `docs/gui/M7_DELIVERY_GATES.md`

### Drift RCA + Prevention (2026-03-25)

Drift cause (root):
- Prior alignment checks were primarily static text-presence checks.
- They did not verify factual claims against live repo state (for example
  daemon method count, git/CI status, and current MCP method catalog parity).
- `specs/MCP_API_SPEC.md` mixed historical deferral wording with current write
  support, creating internal contradiction.

Corrections landed:
- Added `scripts/check_progress_coverage.py` to enforce:
  - `PLAN.md` progress-block coverage for active milestones (`M0`-`M4`, `R1`)
  - `specs/PROGRESS.md` section presence and infrastructure-fact parity
  - MCP current-method parity (`dispatch.rs` ↔ `tools_catalog.py` ↔
    `specs/MCP_API_SPEC.md` list/headings)
  - stale deferral text rejection for write/save support
- Wired `check_progress_coverage.py` into CI (`.github/workflows/alignment.yml`).

---

## Legacy Implementation Evidence

The following milestone tables preserve historical/current implementation
evidence. They are not the active North Star for the new scope. Use them to
ground factual current-state claims, then promote only substrate-relevant facts
into "Scope Integration / Substrate Readiness" above.

---

## PROGRAM_SPEC.md — M0 Exit Criteria

| Criterion | Status | Notes |
|-----------|--------|-------|
| Pool types: Unit, Pin, Entity, Gate, Package, Pad, Padstack, Part, Symbol | [x] | `pool/mod.rs` |
| Pool serialization round-trip (100% of types) | [x] | serde derive + golden tests |
| Pool SQLite index: create, insert, keyword search, parametric search | [x] | `pool/mod.rs` PoolIndex |
| Deterministic serialization (byte-identical 3 runs) | [x] | `ir/serialization.rs` |
| Eagle .lbr import (20 libraries, 0 errors) | [x] | `import/eagle/mod.rs` |
| Eagle canonicalization round-trip | [x] | import → pool JSON → deserialize |
| Eagle deterministic re-import (identical UUIDs) | [x] | UUID v5 with eagle namespace |
| RuleScope IR compiles, serializes, leaf evaluator works | [x] | `rules/ast.rs`, `rules/eval.rs` |
| UUID v5 import identity | [x] | `ir/ids.rs` |
| .ids.json sidecar (write on import, restore on re-import) | [x] | `import/ids_sidecar.rs` |
| **No .lbr write-back** | [x] | import-only, confirmed |

**M0 overall**: [x] Complete (closed by architect)

---

## PROGRAM_SPEC.md — M1 Exit Criteria

| Criterion | Status | Notes |
|-----------|--------|-------|
| KiCad .kicad_pcb import: DOA2526 + 4 designs, 0 errors | [~] | Skeleton parser exists; warns "full geometry not implemented" |
| KiCad .kicad_sch import: DOA2526 + 2 schematics, 0 errors | [~] | Skeleton parser exists; warns "full symbol connectivity not implemented" |
| Eagle .brd/.sch import: 3 designs, 0 errors | [ ] | Returns "not implemented" error |
| Schematic connectivity: hierarchy, labels, power symbols | [~] | Union-find solver in `connectivity/mod.rs`; imported KiCad child-sheet hierarchical linking and basic bus/member expansion now feed existing net/query/check surfaces for the current supported subset, while wider import fidelity and advanced bus syntax remain open |
| Board connectivity: net-to-pin resolution | [x] | `board/mod.rs` net_pins(), net_pad_points() |
| Airwire computation: matches KiCad on DOA2526 | [~] | Algorithm implemented; DOA2526 validation not confirmed |
| Board diagnostics: net_without_copper, via_only, partially_routed | [x] | `board/mod.rs` diagnostics() |
| Golden tests: 8+ designs with checked-in golden files | [ ] | `tests/corpus/` is empty |
| Deterministic import: byte-identical on 3 runs | [~] | Current KiCad import/query corpus is now gated by `scripts/check_import_query_determinism.py` over checked-in query/check fixtures and passes 3-run stability for the current supported subset; broader import fidelity and non-query/save-backed import determinism remain open |
| Import fidelity KiCad ≥ 90% | [ ] | No fidelity matrix exists |
| Import fidelity Eagle ≥ 85% | [ ] | Eagle design import not implemented |

### PROGRAM_SPEC.md — M1 Query API

| Method | Engine | Daemon | MCP | CLI |
|--------|--------|--------|-----|-----|
| get_netlist | [x] | [x] | [x] | [x] |
| get_components | [x] | [x] | [x] | [x] |
| get_nets / get_net_info | [x] | [x] | [x] | [x] |
| get_schematic_net_info | [x] | [x] | [x] | [x] |
| get_board_summary | [x] | [x] | [x] | [x] |
| get_schematic_summary | [x] | [x] | [x] | [x] |
| get_sheets | [x] | [x] | [x] | [x] |
| get_symbols | [x] | [x] | [x] | [x] |
| get_ports | [x] | [x] | [x] | [x] |
| get_labels | [x] | [x] | [x] | [x] |
| get_buses | [x] | [x] | [x] | [x] |
| get_bus_entries | [x] | [x] | [x] | [x] |
| get_noconnects | [x] | [x] | [x] | [x] |
| get_hierarchy | [x] | [x] | [x] | [x] |
| get_connectivity_diagnostics | [x] | [x] | [x] | [x] |
| get_unrouted | [x] | [x] | [x] | [x] |
| search_pool | [x] | [x] | [x] | [x] |

**M1 overall**: [~] Query-surface parity is now complete across engine/daemon/MCP/CLI for the current imported-design read slice; import fidelity, corpus, and broader golden coverage are still open

M1 imported-query reliability note (2026-03-30):
- Checked-in CLI query goldens now cover the current imported-design read
  surfaces for `simple-demo.kicad_sch`, `bus-demo.kicad_sch`,
  `hierarchy-mismatch-demo.kicad_sch`, `erc-coverage-demo.kicad_sch`,
  `simple-demo.kicad_pcb`, and `airwire-demo.kicad_pcb`:
  - `summary`
  - `netlist`
  - `nets`
  - `schematic-nets`
  - `sheets`
  - `symbols`
  - `buses`
  - `bus-entries`
  - `noconnects`
  - `labels`
  - `ports`
  - `hierarchy`
  - `diagnostics`
  - `unrouted`
- Those fixtures live under `crates/cli/testdata/golden/query` and are enforced
  by `main_tests_query_goldens`, with `UPDATE_GOLDENS=1` as the explicit
  regeneration path.
- Checked-in CLI check goldens now cover the imported schematic fixtures already
  used by the current M1/M2 corpus:
  - `simple-demo.kicad_sch`
  - `bus-demo.kicad_sch`
  - `analog-input-demo.kicad_sch`
  - `analog-input-bias-demo.kicad_sch`
  - `erc-coverage-demo.kicad_sch`
  - `hierarchy-mismatch-demo.kicad_sch`
- Those fixtures live under `crates/cli/testdata/golden/check` and are enforced
  by `main_tests_check_goldens`, again with `UPDATE_GOLDENS=1` for intentional
  regeneration.
- Imported KiCad schematic hierarchy now follows the current supported child-sheet
  subset:
  - `simple-demo.kicad_sch` now resolves its checked-in `sub.kicad_sch`
    child file during import
  - hierarchical net propagation now merges the parent sheet pin and
    hierarchical label with the child-sheet hierarchical label target
  - existing query/check surfaces now expose the resulting extra child sheet,
    hierarchy link, and propagated `SUB_IN` pin attachment without adding new
    endpoints
  - resolver diagnostics now report missing or multiply-mapped hierarchical
    child targets deterministically when the imported subset cannot link them
- Imported KiCad bus/member expansion now covers the current supported subset:
  - scalar member labels like `DATA[0]` resolve to deterministic scalar net
    names like `DATA0` on existing net/query/check surfaces
  - simple bus-range labels like `DATA[0..1]` populate imported bus members
  - imported `bus_entry` forms now associate to one bus and one wire when the
    current geometric subset is unambiguous
  - the checked-in `bus-demo.kicad_sch` fixture plus CLI goldens now exercise
    `schematic-nets`, `buses`, `bus-entries`, and `diagnostics` for this path
- The current KiCad import/query corpus now has a repo-native determinism gate:
  - manifest: `crates/test-harness/testdata/quality/import_query_determinism_manifest_v1.json`
  - gate: `python3 scripts/check_import_query_determinism.py`
  - CI: `.github/workflows/alignment.yml`
  - scope: repeated `query` and `check` runs across the checked-in KiCad board
    and schematic fixture corpus used by the current M1 goldens
  - current result: 36/36 cases stable across 3 repeated runs for the current
    supported subset

---

## PROGRAM_SPEC.md — M2 Exit Criteria

### ERC Rules (specs/ERC_SPEC.md §4)

| Check | Status | Notes |
|-------|--------|-------|
| output_to_output_conflict | [x] | `erc/mod.rs` |
| undriven_input | [x] | `erc/mod.rs` (undriven_input_pin + input_without_explicit_driver) |
| power_without_source | [x] | `erc/mod.rs` (power_in_without_source) |
| noconnect_connected | [x] | `erc/mod.rs` |
| unconnected_required_pin | [x] | `erc/mod.rs` (unconnected_component_pin) |
| passive_only_net | [~] | No distinct `passive_only_net` rule/code ships in `erc/mod.rs` (0 grep hits 2026-06-22). Passive pins are only consumed as a modifier that softens `input_without_explicit_driver` messaging; a standalone passive-only-net check is target work. |
| hierarchical_connectivity_mismatch | [x] | Implemented as sheet-level hierarchical-label vs port mismatch check in `erc/mod.rs` |

ERC correctness: [x] `m2_quality` harness reports 0.0% FP / 0.0% FN on current ERC fixtures (2026-03-25)

### DRC Rules

| Check | Status | Notes |
|-------|--------|-------|
| clearance_copper | [x] | Implemented in `drc/mod.rs` (track-track same-layer clearance) |
| track_width | [x] | Implemented in `drc/mod.rs` (`track_width_below_min`) |
| via_hole | [x] | Implemented in `drc/mod.rs` (`via_hole_out_of_range`) |
| via_annular | [x] | Implemented in `drc/mod.rs` (`via_annular_below_min`) |
| connectivity | [x] | Implemented in `drc/mod.rs` (no-copper + unrouted-net checks) |
| unconnected_pins | [x] | Implemented in `drc/mod.rs` (`connectivity_unconnected_pin`) |
| silk_clearance | [x] | Implemented in `drc/mod.rs` (`silk_clearance_copper`) |

DRC correctness: [x] `m2_quality` harness reports 0.0% FP / 0.0% FN on current DRC fixtures (2026-03-25)

### MCP Tools (specs/MCP_API_SPEC.md — M2)

| Tool | Status | Notes |
|------|--------|-------|
| open_project | [x] | Current implementation contract |
| close_project | [x] | Current implementation contract |
| search_pool | [x] | Current implementation contract |
| get_part | [x] | Current implementation contract |
| get_package | [x] | Current implementation contract |
| get_package_change_candidates | [x] | Current implementation contract |
| get_board_summary | [x] | Current implementation contract |
| get_schematic_summary | [x] | Current implementation contract |
| get_sheets | [x] | Current implementation contract |
| get_symbols | [x] | Current (all-sheets only) |
| get_ports | [x] | Current (all-sheets only) |
| get_labels | [x] | Current (all-sheets only) |
| get_buses | [x] | Current (all-sheets only) |
| get_bus_entries | [x] | Current implementation contract |
| get_noconnects | [x] | Current (all-sheets only) |
| get_symbol_fields | [x] | Current implementation contract |
| get_hierarchy | [x] | Current implementation contract |
| get_netlist | [x] | Current implementation contract |
| get_components | [x] | Current implementation contract |
| get_net_info | [x] | Current (full inventory, not single-net selector) |
| get_schematic_net_info | [x] | Current (full inventory, not single-net selector) |
| get_connectivity_diagnostics | [x] | Current implementation contract |
| get_design_rules | [x] | Current implementation contract |
| get_unrouted | [x] | Current implementation contract |
| get_check_report | [x] | Current implementation contract (not in M2 exit list but implemented) |
| run_erc | [x] | Current implementation contract |
| run_drc | [x] | Current implementation contract |
| explain_violation | [x] | Current implementation contract |

MCP tools: M2 slice 26/26 implemented. Current MCP runtime catalog: 181
methods (daemon-dispatched + CLI-bridged via `mcp-server/server_runtime.py`),
locked via `specs/SPEC_PARITY.md` → `mcp_runtime_methods`.

### CLI Commands (specs/PROGRAM_SPEC.md — M2)

> **Scope note.** The table below is the *M2 historical slice* — the eight
> commands M2 froze. It is **not** the current CLI surface. The present
> `tool project` surface is **256 commands**, locked via
> `specs/SPEC_PARITY.md` → `cli_project_commands`. For the authoritative,
> code-derived enumeration run
> `python3 scripts/check_spec_parity.py --print` and read the
> `[cli_project_commands]` block. Do not read this M2 table as today's reach.

| Command | Status | Notes |
|---------|--------|-------|
| tool import \<file\> | [x] | Eagle .lbr only in current slice |
| tool query \<design\> --nets | [x] | |
| tool query \<design\> --components | [x] | |
| tool query \<design\> --summary | [x] | |
| tool erc \<design\> | [x] | .kicad_sch only |
| tool drc \<design\> | [x] | Runs DRC, reports JSON/text, exits nonzero on violations |
| tool pool search \<query\> | [x] | |
| tool check \<design\> | [x] | Unified check report |
| CLI exit codes (0/1/2) | [x] | Checking commands now return 1 on violations and 2 only on execution errors |

### Other M2 Requirements

| Criterion | Status | Notes |
|-----------|--------|-------|
| MCP registration in active MCP host config | [x] | Configured `datum-eda` MCP entry with `EDA_ENGINE_SOCKET=/tmp/datum-eda-engine.sock` (2026-03-25) |
| Test corpus: 10+ designs with ERC/DRC golden files | [x] | `m2_quality` reports 11 unique designs (erc=5, drc=6) |
| Quality-rate harness (ERC/DRC FP/FN + corpus gate) | [x] | `m2_quality` implemented with checked-in manifest; current gate state is `pass=true` |
| ERC on DOA2526 < 3 seconds | [x] | `m2_perf` median ERC = 2ms (3-iteration run, 2026-03-25) |
| DRC on DOA2526 < 5 seconds | [x] | `m2_perf` median DRC = 18ms baseline, 20ms compare run (2026-03-25) |
| MCP transport (daemon ↔ MCP server) | [x] | Unix socket JSON-RPC implemented in daemon + Python client; live transport smoke is tracked separately in unrestricted CI |

**M2 overall**: [x] Complete for the current implementation slice — ERC/DRC checks, MCP/CLI parity, quality/performance harnesses, and local MCP host registration are all in place.

### M2 Pre-Freeze Closeout Checklist (Ordered)

This is the execution order to close meaningful M2 gaps before full integrated
program specification freeze.

| Order | Work Item | Exit Condition | Status |
|------:|-----------|----------------|--------|
| 1 | DRC foundation + first runnable checks | `run_drc` returns structured violations for at least connectivity + clearance on fixture boards | [x] |
| 2 | CLI DRC integration | `tool drc <board.kicad_pcb>` works with pass/fail exit behavior and JSON output | [x] |
| 3 | MCP/daemon DRC path | MCP `run_drc` round-trips through daemon with stable payload shape | [x] |
| 4 | Remaining high-value MCP query parity | `get_sheets`, `get_netlist`, `get_design_rules` implemented or explicitly deferred with gate note | [x] |
| 5 | ERC corpus hardening | Corpus-backed ERC goldens exist and cover required M2 codes in current implementation slice | [x] |
| 6 | DRC corpus hardening | Corpus-backed DRC goldens exist for implemented DRC checks | [x] |
| 7 | Performance gate harness | Reproducible timing harness for ERC/DRC on DOA2526 with recorded baseline | [x] |
| 8 | M2 gate reconciliation pass | `PROGRAM_SPEC.md`, `MCP_API_SPEC.md`, `ENGINE_SPEC.md`, `PLAN.md`, `PROGRESS.md` mutually consistent for remaining open items | [x] |

Item 4 defer notes (2026-03-25):
- `get_sheets`: implemented in engine/daemon/MCP.
- `get_design_rules`: implemented in engine/daemon/MCP using the current rule
  evaluator subset payload.
- `get_netlist`: implemented in engine/daemon/MCP using a canonical net
  inventory payload with board-vs-schematic field parity.

Item 5 current artifacts (2026-03-25):
- ERC golden fixtures added:
  - `crates/engine/testdata/golden/erc/simple-demo.kicad_sch.json`
  - `crates/engine/testdata/golden/erc/analog-input-demo.kicad_sch.json`
  - `crates/engine/testdata/golden/erc/analog-input-bias-demo.kicad_sch.json`
  - `crates/engine/testdata/golden/erc/erc-coverage-demo.kicad_sch.json`
  - `crates/engine/testdata/golden/erc/hierarchy-mismatch-demo.kicad_sch.json`
- Golden validation tests:
  - `api::tests::erc_golden_simple_demo_matches_checked_in_fixture`
  - `api::tests::erc_golden_analog_input_demo_matches_checked_in_fixture`
  - `api::tests::erc_golden_analog_input_bias_demo_matches_checked_in_fixture`
  - `api::tests::erc_golden_coverage_demo_matches_checked_in_fixture`
  - `api::tests::erc_golden_hierarchy_mismatch_demo_matches_checked_in_fixture`
  - `api::tests::erc_golden_corpus_covers_required_m2_codes_for_current_implementation_slice`
- Golden contract is normalized to stable semantic fields (code/severity/message/
  net/component/pin/objects/waived) to avoid false churn from volatile UUIDs.
- Added implementation slice coverage for required ERC codes:
  - `output_to_output_conflict`
  - `undriven_input_pin`
  - `input_without_explicit_driver`
  - `power_in_without_source`
  - `unconnected_component_pin`
  - `undriven_power_net`
  - `noconnect_connected`
  - `hierarchical_connectivity_mismatch`
- Note: current implementation is sheet-level interface name consistency. Full
  instance-aware hierarchical net resolution remains tracked under schematic
  connectivity maturity (M1/M3+), not as an open M2 rule omission.

Item 6 current artifacts (2026-03-25):
- DRC golden fixtures added:
  - `crates/engine/testdata/golden/drc/simple-demo.kicad_pcb.json`
  - `crates/engine/testdata/golden/drc/partial-route-demo.kicad_pcb.json`
  - `crates/engine/testdata/golden/drc/airwire-demo.kicad_pcb.json`
  - `crates/engine/testdata/golden/drc/clearance-violation-demo.kicad_pcb.json`
  - `crates/engine/testdata/golden/drc/drc-coverage-demo.kicad_pcb.json`
  - `crates/engine/testdata/golden/drc/silk-clearance-demo.kicad_pcb.json`
- Golden validation tests:
  - `api::tests::drc_golden_simple_demo_matches_checked_in_fixture`
  - `api::tests::drc_golden_partial_route_demo_matches_checked_in_fixture`
  - `api::tests::drc_golden_airwire_demo_matches_checked_in_fixture`
  - `api::tests::drc_golden_clearance_violation_demo_matches_checked_in_fixture`
  - `api::tests::drc_golden_coverage_demo_matches_checked_in_fixture`
  - `api::tests::drc_golden_silk_clearance_demo_matches_checked_in_fixture`
- Golden contract is normalized to stable semantic fields (pass/fail, summary,
  code/rule/severity/message/location/objects) and excludes volatile violation IDs.
- Coverage for implemented DRC checks now includes:
  - connectivity (`connectivity_unrouted_net`)
  - clearance (`clearance_copper`)
  - connectivity unconnected pin (`connectivity_unconnected_pin`)
  - track width (`track_width_below_min`)
  - via hole (`via_hole_out_of_range`)
  - via annular ring (`via_annular_below_min`)
  - silk clearance (`silk_clearance_copper`)
- Remaining gap: broaden corpus and validation metrics against external reference
  behavior for quality-rate gates.
- Corpus-size gate is now met by checked-in golden fixtures:
  - total unique designs: 10
  - DRC additions in this pass include `airwire-demo.kicad_pcb`.
  - ERC additions in this pass include `analog-input-bias-demo.kicad_sch`.

Item 7 current artifacts (2026-03-25):
- Harness binary:
  - `crates/test-harness/src/bin/m2_perf.rs`
- Baseline artifact:
  - `crates/test-harness/testdata/perf/m2_doa2526_baseline.json`
- Commands:
  - Baseline write:
    - `cargo run -p eda-test-harness --bin m2_perf -- --iterations 3 --write-baseline crates/test-harness/testdata/perf/m2_doa2526_baseline.json`
  - Baseline compare:
    - `cargo run -p eda-test-harness --bin m2_perf -- --iterations 3 --compare-baseline crates/test-harness/testdata/perf/m2_doa2526_baseline.json`
- Current measured medians:
  - Baseline run: import_board=1200ms, import_schematic=1759ms, ERC=2ms, DRC=18ms
  - Compare run: import_board=1285ms, import_schematic=1838ms, ERC=2ms, DRC=20ms

Item 9 quality-rate gate artifacts (2026-03-25):
- Harness binary:
  - `crates/test-harness/src/bin/m2_quality.rs`
- Manifest artifact:
  - `crates/test-harness/testdata/quality/m2_quality_manifest.json`
- Command:
  - `cargo run -p eda-test-harness --bin m2_quality -- --json`
- Current measured output:
  - `erc false_positive_rate_pct=0.0`
  - `erc false_negative_rate_pct=0.0`
  - `drc false_positive_rate_pct=0.0`
  - `drc false_negative_rate_pct=0.0`
  - `unique_designs=11`, `required_min_designs=10`
  - `pass=true`

Item 8 reconciliation notes (2026-03-25):
- Performance harness status synced across:
  - `specs/PROGRESS.md` M2 requirement table
  - `specs/PROGRESS.md` closeout checklist
  - `PLAN.md` M2 progress section
- Deferred MCP query language remains aligned across:
  - `specs/PROGRAM_SPEC.md`
  - `specs/MCP_API_SPEC.md`
- Stale wording corrected:
  - `Waiver matching in DRC` now states DRC exists and waiver matching is the remaining gap.

Checking follow-up note (2026-03-30):
- Native-project authored waivers are now honored by the existing `project query erc`
  and `project query check` load path.
- DRC-domain waiver matching is now honored inside `run_drc`; waived DRC
  violations remain visible and waived-only DRC runs do not fail.
- Native authored board state now exposes the same waiver-aware DRC report through
  `project query <dir> drc`.
- Native `project query <dir> check` now returns a combined report with distinct
  `erc` and `drc` sections while leaving `project query <dir> board-check`
  unchanged.
- Native-project structural validation is now exposed through
  `project validate <dir>`, covering required native files, schema-version
  compatibility, duplicate UUID consistency within authored object types, and
  non-dangling persisted schematic/board references.
- The same native-project validation contract is now available through MCP as
  `validate_project`, preserving the CLI report shape and valid/invalid result
  semantics.
- MCP tool registration now also uses one shared spec table for `tools/list`
  export and `tools/call` dispatch, with parity tests checking runtime and
  fake-daemon coverage for each registered tool.
- Checked-in native fixture projects are now exercised continuously through
  `scripts/check_native_project_fixtures.py`, driven by
  `crates/test-harness/testdata/quality/native_project_validation_manifest_v1.json`
  and the existing `project validate` contract in CI.
- That manifest now covers both real route-strategy native fixtures and a
  dedicated checked-in invalid-case suite for duplicate UUID, missing sheet,
  and unsupported schema-version failures, and it currently exhausts all
  checked-in native project roots under `crates/test-harness/testdata/quality`.

### 2026-03-25 Contract Alignment Pass

| Item | Status | Notes |
|------|--------|-------|
| Integrated spec scaffold added | [x] | `specs/INTEGRATED_PROGRAM_SPEC.md` |
| Integrated contract verification matrix added | [x] | `specs/INTEGRATED_PROGRAM_SPEC.md` §8 |
| Integrated M3/M4 boundary contracts drafted | [x] | `specs/INTEGRATED_PROGRAM_SPEC.md` §§9-11 |
| Integrated M3/M4 acceptance tables drafted | [x] | `specs/INTEGRATED_PROGRAM_SPEC.md` §§12-13 |
| M3 determinism evidence hook behavioral | [x] | `crates/test-harness/src/bin/m3_op_determinism.rs` and `crates/test-harness/src/bin/m3_replacement_op_determinism.rs` together pass for the full current save-backed M3 mutation slice: `move_component`, `delete_track`, `delete_via`, `delete_component`, `rotate_component`, `set_value`, `set_reference`, `set_design_rule`, `assign_part`, `set_package`, `set_package_with_part`, `replace_component`, `replace_components`, `apply_component_replacement_plan`, `apply_component_replacement_policy`, `apply_scoped_component_replacement_policy`, `apply_scoped_component_replacement_plan`, and `set_net_class` |
| M3 undo/redo evidence hook behavioral | [x] | `crates/test-harness/src/bin/m3_undo_redo_roundtrip.rs` passes for current `delete_track`, `delete_via`, `delete_component`, `move_component`, `rotate_component`, `set_value`, `set_reference`, `set_design_rule`, `assign_part`, `set_package`, and `set_net_class` undo/redo slice |
| M3 replacement undo/redo evidence hook behavioral | [x] | `crates/test-harness/src/bin/m3_replacement_undo_redo_roundtrip.rs` passes for current `set_package_with_part`, `replace_component`, `replace_components`, `apply_component_replacement_plan`, `apply_component_replacement_policy`, `apply_scoped_component_replacement_policy`, and `apply_scoped_component_replacement_plan` undo/redo slice |
| M3 board round-trip fidelity hook behavioral | [x] | `crates/test-harness/src/bin/m3_board_roundtrip_fidelity.rs` passes for unmodified KiCad-board identity plus current `delete_track`, `delete_via`, `delete_component`, `move_component`, `rotate_component`, `set_value`, and `set_reference` save→reimport→save artifact stability |
| M3 sidecar round-trip fidelity hook behavioral | [x] | `crates/test-harness/src/bin/m3_sidecar_roundtrip_fidelity.rs` passes for current `set_design_rule`, `assign_part`, `set_package`, `set_package_with_part`, `replace_component`, `replace_components`, `apply_component_replacement_plan`, `apply_component_replacement_policy`, `apply_scoped_component_replacement_policy`, `apply_scoped_component_replacement_plan`, and `set_net_class` save→reimport→save artifact stability slice |
| M3 write-surface parity hook behavioral | [x] | `crates/test-harness/src/bin/m3_write_surface_parity.rs` passes for current engine/daemon/MCP/CLI `move_component`/`rotate_component`/`set_value`/`set_reference`/`assign_part`/`set_package`/`set_package_with_part`/`set_net_class`/`delete_component`/`delete_track`/`delete_via`/`set_design_rule`/`undo`/`redo`/`save` slice, including follow-up derived-state checks |
| M3 aggregate acceptance gate behavioral | [x] | `crates/test-harness/src/bin/m3_acceptance_gate.rs` passes and composes base determinism, replacement determinism, base undo/redo, replacement undo/redo, board round-trip fidelity, sidecar round-trip fidelity, and write-surface parity into one milestone checkpoint |
| PLAN progress ticks added for M0/M1/M2/R1 | [x] | `PLAN.md` now has dated progress blocks |
| MCP contract split (current vs target) | [x] | `specs/MCP_API_SPEC.md` |
| PROGRAM spec references MCP contract split | [x] | `specs/PROGRAM_SPEC.md` M2 status section |
| ENGINE spec API split (current vs target) | [x] | `specs/ENGINE_SPEC.md` §5 |
| MCP design doc hardened to target/current labels | [x] | `docs/MCP_DESIGN.md` |
| User workflows aligned to current MCP/CLI surfaces | [x] | `docs/USER_WORKFLOWS.md` |
| Engine design doc marked target-state vs live API | [x] | `docs/ENGINE_DESIGN.md` |

---

## PROGRAM_SPEC.md — R1 Research Gates

| Criterion | Status | Notes |
|-----------|--------|-------|
| R1-G0 Foundation Gate | [x] | Minimum history/context gate is now evidenced in `docs/R1_G0_FOUNDATION.md` and blocks downstream milestone completion claims unless maintained |
| R1-G0 tool lineage + format evolution map | [x] | Evidence: `docs/R1_G0_FOUNDATION.md` §1 covers KiCad/Eagle/Altium/PADS/OrCAD lineage, likely ingestion surfaces, and roadmap implications |
| R1-G0 migration pain taxonomy | [x] | Evidence: `docs/R1_G0_FOUNDATION.md` §2 classifies migration failure modes using the current KiCad/Eagle design/library corpus and current interop workflow evidence |
| R1-G0 fidelity boundary policy | [x] | Evidence: `docs/R1_G0_FOUNDATION.md` §3 defines exactness, approximation, preservation-as-metadata, and unsupported-loss boundaries for future interop claims |

**R1 overall**: [~] Minimal context gate is complete, but broader commercial-interop research exit criteria remain open (corpus, legal posture, prototypes, recommendation)

---

## PROGRAM_SPEC.md — M3 Exit Criteria

| Criterion | Status | Notes |
|-----------|--------|-------|
| MoveComponent | [x] | Detailed status and evidence anchors are maintained in `specs/progress/m3_details.md#movecomponent`. |
| RotateComponent | [x] | Detailed status and evidence anchors are maintained in `specs/progress/m3_details.md#rotatecomponent`. |
| DeleteComponent | [x] | Detailed status and evidence anchors are maintained in `specs/progress/m3_details.md#deletecomponent`. |
| SetValue | [x] | Detailed status and evidence anchors are maintained in `specs/progress/m3_details.md#setvalue`. |
| SetReference | [x] | Detailed status and evidence anchors are maintained in `specs/progress/m3_details.md#setreference`. |
| AssignPart | [x] | Detailed status and evidence anchors are maintained in `specs/progress/m3_details.md#assignpart`. |
| SetPackage | [x] | Detailed status and evidence anchors are maintained in `specs/progress/m3_details.md#setpackage`. |
| SetPackageWithPart | [x] | Detailed status and evidence anchors are maintained in `specs/progress/m3_details.md#setpackagewithpart`. |
| ReplaceComponents | [x] | Detailed status and evidence anchors are maintained in `specs/progress/m3_details.md#replacecomponents`. |
| ApplyComponentReplacementPlan | [x] | Detailed status and evidence anchors are maintained in `specs/progress/m3_details.md#applycomponentreplacementplan`. |
| ApplyComponentReplacementPolicy | [x] | Detailed status and evidence anchors are maintained in `specs/progress/m3_details.md#applycomponentreplacementpolicy`. |
| ApplyScopedComponentReplacementPolicy | [x] | Detailed status and evidence anchors are maintained in `specs/progress/m3_details.md#applyscopedcomponentreplacementpolicy`. |
| ApplyScopedComponentReplacementPlan | [x] | Detailed status and evidence anchors are maintained in `specs/progress/m3_details.md#applyscopedcomponentreplacementplan`. |
| ScopedReplacementPlanManifest | [x] | Detailed status and evidence anchors are maintained in `specs/progress/m3_details.md#scopedreplacementplanmanifest`. |
| Package-change introspection | [x] | Detailed status and evidence anchors are maintained in `specs/progress/m3_details.md#package-change-introspection`. |
| Part-change introspection | [x] | Detailed status and evidence anchors are maintained in `specs/progress/m3_details.md#part-change-introspection`. |
| Replacement-plan introspection | [x] | Detailed status and evidence anchors are maintained in `specs/progress/m3_details.md#replacement-plan-introspection`. |
| SetNetClass | [x] | Detailed status and evidence anchors are maintained in `specs/progress/m3_details.md#setnetclass`. |
| SetDesignRule | [x] | Detailed status and evidence anchors are maintained in `specs/progress/m3_details.md#setdesignrule`. |
| DeleteTrack | [x] | Detailed status and evidence anchors are maintained in `specs/progress/m3_details.md#deletetrack`. |
| DeleteVia | [x] | Detailed status and evidence anchors are maintained in `specs/progress/m3_details.md#deletevia`. |
| Undo/redo (100% undoable) | [x] | Detailed status and evidence anchors are maintained in `specs/progress/m3_details.md#undoredo-100-undoable`. |
| Operation determinism | [x] | Detailed status and evidence anchors are maintained in `specs/progress/m3_details.md#operation-determinism`. |
| KiCad write-back | [x] | Detailed status and evidence anchors are maintained in `specs/progress/m3_details.md#kicad-write-back`. |
| Round-trip fidelity | [x] | Detailed status and evidence anchors are maintained in `specs/progress/m3_details.md#round-trip-fidelity`. |
| MCP write tools | [x] | Detailed status and evidence anchors are maintained in `specs/progress/m3_details.md#mcp-write-tools`. |
| Derived data recomputation | [x] | Detailed status and evidence anchors are maintained in `specs/progress/m3_details.md#derived-data-recomputation`. |
| CLI modify command | [x] | Detailed status and evidence anchors are maintained in `specs/progress/m3_details.md#cli-modify-command`. |

**M3 overall**: [x] M3 is complete for the implemented imported-board write slice; criterion-level details and evidence anchors are consolidated in `specs/progress/m3_details.md`.

---

## PROGRAM_SPEC.md — M4 Exit Criteria

| Criterion | Status | Notes |
|-----------|--------|-------|
| Native format | [x] | Detailed status and evidence anchors are maintained in `specs/progress/m4_details.md#native-format`. |
| Schematic operations | [x] | Detailed status and evidence anchors are maintained in `specs/progress/m4_details.md#schematic-operations`. |
| Board operations | [x] | Detailed status and evidence anchors are maintained in `specs/progress/m4_details.md#board-operations`. |
| Schematic query parity | [x] | Detailed status and evidence anchors are maintained in `specs/progress/m4_details.md#schematic-query-parity`. |
| Forward annotation | [x] | Detailed status and evidence anchors are maintained in `specs/progress/m4_details.md#forward-annotation`. |
| Gerber export | [x] | Detailed status and evidence anchors are maintained in `specs/progress/m4_details.md#gerber-export`. |
| Drill export | [x] | Detailed status and evidence anchors are maintained in `specs/progress/m4_details.md#drill-export`. |
| BOM export | [x] | Detailed status and evidence anchors are maintained in `specs/progress/m4_details.md#bom-export`. |
| PnP export | [x] | Detailed status and evidence anchors are maintained in `specs/progress/m4_details.md#pnp-export`. |
| Gerber comparison | [x] | Detailed status and evidence anchors are maintained in `specs/progress/m4_details.md#gerber-comparison`. |

**M4 overall**: [x] Closed for scope; details remain in `specs/progress/m4_details.md`.

## M5 Opening Charter

Status: [~] Routing-kernel scope complete; closure review pending
- Authority for the proposed opening charter and entry criteria:
  `specs/progress/m5_opening.md`
- Recommended M5 focus: deterministic layout-kernel groundwork from persisted
  native board state, opening with one narrow routing/constraint slice rather
  than broad placement/routing ambition.
- Governance narrowing (2026-03-30):
  - for milestone closure, `M5` is now interpreted as the deterministic
    persisted-state routing-kernel milestone recorded in the opening charter
    and current frontier below
  - placement-kernel and placement/routing co-optimization work are deferred
    to a later reopened milestone/slice and are not required for `M5` closure
  - `M6` may open from the completed routing-kernel substrate once closure
    review accepts this narrowed milestone scope
- M5 must not inherit M4 parity work by default; new slices require an
  explicit layout-kernel contract and acceptance criteria.
- MCP implementation parity remains deferred unless explicitly reopened, but
  deferred parity tracking for newer native M4/M5 slices must stay current in
  `specs/MCP_API_SPEC.md`.
- A narrow MCP parity exception is now live for the policy-selected
  authored-copper-graph proposal export surface; broader M5 MCP query/apply
  parity remains deferred.
- That narrow exception now covers the full route-proposal artifact lifecycle
  for the policy-selected authored-copper-graph family: export, inspect, and
  apply.
- Current M5 checkpoint chain (routing-kernel read/query lane):
  - `project query <dir> routing-substrate`
  - `project query <dir> route-preflight --net <uuid>`
  - `project query <dir> route-corridor --net <uuid>`
  - canonical route query surface: `project query <dir> route-path-candidate --net <uuid> --from-anchor <pad_uuid> --to-anchor <pad_uuid> --candidate <accepted_candidate> [--policy <policy>]`
  - canonical route explanation surface: `project query <dir> route-path-candidate-explain --net <uuid> --from-anchor <pad_uuid> --to-anchor <pad_uuid> --candidate <accepted_candidate> [--policy <policy>]`
  - contract-specific `route-path-candidate-*` and
    `route-path-candidate-*-explain` commands are now compatibility wrappers
    around those bounded generic surfaces
  - the remaining legacy wrapper implementations now also dispatch through the
    same shared generic candidate/policy executor path as the canonical
    surfaces, reducing wrapper-only branching ahead of removal
- Current M5 routing expansion (persisted-via reuse lane):
  - fixed-arity via reuse was proven through bounded ordinal slices and is no
    longer the preferred growth model
  - current generalized contract is `project query <dir>
    route-path-candidate --net <uuid> --from-anchor <pad_uuid> --to-anchor
    <pad_uuid> --candidate route-path-candidate-authored-via-chain`
  - current adjacent observability surface is `project query <dir>
    route-path-candidate-explain --net <uuid> --from-anchor <pad_uuid>
    --to-anchor <pad_uuid> --candidate route-path-candidate-authored-via-chain`
  - a same-layer synthesis slice now also exists via `project query <dir>
    route-path-candidate --net <uuid> --from-anchor <pad_uuid> --to-anchor
    <pad_uuid> --candidate route-path-candidate-orthogonal-dogleg`, with the
    paired explanation surface under `route-path-candidate-explain`
  - that same same-layer synthesis lane now also covers `project query <dir>
    route-path-candidate --net <uuid> --from-anchor <pad_uuid> --to-anchor
    <pad_uuid> --candidate route-path-candidate-orthogonal-two-bend`, with the
    paired explanation surface under `route-path-candidate-explain`
  - that same same-layer synthesis lane now also covers `project query <dir>
    route-path-candidate --net <uuid> --from-anchor <pad_uuid> --to-anchor
    <pad_uuid> --candidate route-path-candidate-orthogonal-graph`, with the
    paired explanation surface under `route-path-candidate-explain`
  - a first bounded cross-layer graph-search lane now also exists via
    `project query <dir> route-path-candidate --net <uuid> --from-anchor
    <pad_uuid> --to-anchor <pad_uuid> --candidate
    route-path-candidate-orthogonal-graph-via`, with the paired explanation
    surface under `route-path-candidate-explain`
  - that same bounded cross-layer graph-search lane now also covers `project
    query <dir> route-path-candidate --net <uuid> --from-anchor <pad_uuid>
    --to-anchor <pad_uuid> --candidate
    route-path-candidate-orthogonal-graph-two-via`, with the paired
    explanation surface under `route-path-candidate-explain`
  - that same bounded cross-layer graph-search lane now also covers the
    remaining authored-via graph sequence via `project query <dir>
    route-path-candidate --net <uuid> --from-anchor <pad_uuid> --to-anchor
    <pad_uuid> --candidate
    route-path-candidate-orthogonal-graph-three-via|route-path-candidate-orthogonal-graph-four-via|route-path-candidate-orthogonal-graph-five-via|route-path-candidate-orthogonal-graph-six-via`,
    with the paired explanation surface under `route-path-candidate-explain`
  - the shared orthogonal graph selector now uses an explicit deterministic
    cost rule across that whole family: bend count ascending, then segment
    count ascending, then point-sequence coordinate ascending
  - the same orthogonal graph query and explanation reports now expose that
    selected path cost directly as bend/segment/point counts on the returned
    path or per-segment path data
  - the same orthogonal graph route proposal artifact lane now preserves the
    selected bend count in exported actions and exposes it again through
    artifact inspection/apply reporting
  - `export-route-path-proposal` now also returns the recorded
    orthogonal-graph layer-segment bend/point/track-action breakdown
  - the same-layer orthogonal-graph `route-path-candidate` and
    `route-path-candidate-explain` reports now also return `segment_evidence`
    so direct query output matches the artifact lane vocabulary
  - the same-layer orthogonal-graph candidate and explain surfaces now also
    share one internal graph-search spine, keeping the public reports
    unchanged while consolidating the preflight/search/segment-evidence path
  - the one-authored-via orthogonal-graph candidate and explain surfaces now
    also share one internal via-selection spine that reuses the same
    orthogonal-graph layer-search structure without changing the public
    reports
  - the two-authored-via orthogonal-graph candidate and explain surfaces now
    also share one internal pair-selection spine that reuses the same
    orthogonal-graph layer-search structure without changing the public
    reports
  - orthogonal-graph artifact apply now reports whether stale proposals
    drifted because candidate availability changed, the deterministic ranked
    winner changed, or same-rank geometry changed
  - `inspect-route-proposal-artifact` now also returns the recorded
    orthogonal-graph layer-segment bend/point/track-action breakdown
  - the same lane now also has `project revalidate-route-proposal-artifact
    <dir> --artifact <path>` so callers can read that drift classification
    and live/recorded path summaries without applying
  - that revalidation report now also carries segment-level orthogonal-graph
    evidence so stale proposals can show which layer-side segment changed and
    how its bend/point/track-action facts differ live
- Current M5 existing-copper readback lane:
  - deterministic authored-copper graph path queries now exist in
    increasingly filtered/readback-focused forms recorded in
    `specs/progress/m5_opening.md`
  - that same generalized query surface also covers the accepted
    authored-copper-graph family when `--candidate authored-copper-graph
    --policy <policy>` is selected
  - that same generalized explanation surface also covers the accepted
    authored-copper-graph family when `--candidate authored-copper-graph
    --policy <policy>` is selected
  - accepted bounded policies are `plain`, `zone_aware`, `obstacle_aware`,
    `zone_obstacle_aware`, `zone_obstacle_topology_aware`, and
    `zone_obstacle_topology_layer_balance_aware`
  - the plus-one-gap bridge is now reached canonically through `project query
    <dir> route-path-candidate --net <uuid> --from-anchor <pad_uuid>
    --to-anchor <pad_uuid> --candidate authored-copper-plus-one-gap`
  - a first write-capable bridge now exists for that accepted query contract:
    `project export-route-path-proposal <dir> --net <uuid> --from-anchor
    <pad_uuid> --to-anchor <pad_uuid> --candidate
    authored-copper-plus-one-gap --out <path>`,
    `project inspect-route-proposal-artifact <path>`, and
    `project apply-route-proposal-artifact <dir> --artifact <path>`
  - the same route-proposal artifact lane now also covers the accepted
    completed write-capable family, now canonically via `project
    export-route-path-proposal <dir> --net <uuid> --from-anchor <pad_uuid>
    --to-anchor <pad_uuid> --candidate <accepted_candidate> [--policy
    <policy>] --out <path>`, spanning the accepted single-layer, via-path, and
    authored-copper-graph policy-selected contracts
  - a native direct convenience apply surface now also exists for that same
    accepted single-layer path contract via `project route-apply <dir> --net
    <uuid> --from-anchor <pad_uuid> --to-anchor <pad_uuid> --candidate
    route-path-candidate`
  - a native direct convenience apply surface now also exists for that same
    accepted bounded single-via contract via `project route-apply <dir> --net
    <uuid> --from-anchor <pad_uuid> --to-anchor <pad_uuid> --candidate
    route-path-candidate-via`
  - a native direct convenience apply surface now also exists for that same
    accepted bounded two-via contract via `project route-apply <dir> --net
    <uuid> --from-anchor <pad_uuid> --to-anchor <pad_uuid> --candidate
    route-path-candidate-two-via`
  - a native direct convenience apply surface now also exists for that same
    accepted bounded three-via contract via `project route-apply <dir> --net
    <uuid> --from-anchor <pad_uuid> --to-anchor <pad_uuid> --candidate
    route-path-candidate-three-via`
  - the same direct convenience apply lane now also covers the remaining
    bounded ordinal via contracts via `project route-apply <dir> --net <uuid>
    --from-anchor <pad_uuid> --to-anchor <pad_uuid> --candidate
    route-path-candidate-four-via|route-path-candidate-five-via|route-path-candidate-six-via`
  - that same direct convenience apply lane now also covers the accepted
    deterministic authored-via-chain contract via `project route-apply <dir>
    --net <uuid> --from-anchor <pad_uuid> --to-anchor <pad_uuid> --candidate
    route-path-candidate-authored-via-chain`
  - that same direct convenience apply lane now also covers the accepted
    same-layer orthogonal dogleg contract via `project route-apply <dir> --net
    <uuid> --from-anchor <pad_uuid> --to-anchor <pad_uuid> --candidate
    route-path-candidate-orthogonal-dogleg`
  - that same direct convenience apply lane now also covers the accepted
    same-layer orthogonal two-bend contract via `project route-apply <dir> --net
    <uuid> --from-anchor <pad_uuid> --to-anchor <pad_uuid> --candidate
    route-path-candidate-orthogonal-two-bend`
  - that same direct convenience apply lane now also covers the accepted
    same-layer orthogonal graph contract via `project route-apply <dir> --net
    <uuid> --from-anchor <pad_uuid> --to-anchor <pad_uuid> --candidate
    route-path-candidate-orthogonal-graph`
  - that same direct convenience apply lane now also covers the accepted
    one-authored-via orthogonal graph contract via `project route-apply <dir>
    --net <uuid> --from-anchor <pad_uuid> --to-anchor <pad_uuid> --candidate
    route-path-candidate-orthogonal-graph-via`
  - that same direct convenience apply lane now also covers the accepted
    two-authored-via orthogonal graph contract via `project route-apply <dir>
    --net <uuid> --from-anchor <pad_uuid> --to-anchor <pad_uuid> --candidate
    route-path-candidate-orthogonal-graph-two-via`
  - that same direct convenience apply lane now also covers the remaining
    authored-via orthogonal graph sequence via `project route-apply <dir>
    --net <uuid> --from-anchor <pad_uuid> --to-anchor <pad_uuid> --candidate
    route-path-candidate-orthogonal-graph-three-via|route-path-candidate-orthogonal-graph-four-via|route-path-candidate-orthogonal-graph-five-via|route-path-candidate-orthogonal-graph-six-via`
  - a bounded convenience export surface now also exists for the completed
    write-capable route family via `project export-route-path-proposal <dir>
    --net <uuid> --from-anchor <pad_uuid> --to-anchor <pad_uuid> --candidate
    <accepted_candidate> [--policy <policy>] --out <path>`
  - `project route-apply` now parses `--candidate` from a bounded accepted
    value set instead of a free-form string, and enforces `--policy` only for
    `--candidate authored-copper-graph`
  - a native direct convenience apply surface now also exists for that same
    policy-selected family via `project route-apply <dir> --net <uuid>
    --from-anchor <pad_uuid> --to-anchor <pad_uuid> --candidate
    authored-copper-graph --policy <policy>`
  - the bounded convenience export surface is now exposed through MCP as
    `export_route_path_proposal`
  - the bounded direct route-apply surface is now exposed through MCP as
    `route_apply`
  - the matching generic artifact follow-up surfaces are now exposed through
    MCP as `inspect_route_proposal_artifact`,
    `revalidate_route_proposal_artifact`, and
    `apply_route_proposal_artifact`
  - `specs/PROGRESS.md` tracks only the checkpoint/frontier; detailed per-slice
    history stays in `specs/progress/m5_opening.md`
- Current M5 frontier:
  - deterministic persisted-state layout-kernel/routing queries continue under
    explicit contract selection from `specs/progress/m5_opening.md`
  - a bounded native route selector now exists via `project route-proposal
    <dir> --net <uuid> --from-anchor <pad_uuid> --to-anchor <pad_uuid>`,
    selecting the first successful family from the explicit accepted candidate
    order recorded in `specs/progress/m5_opening.md`
  - that same selector now also feeds a selected-proposal write lane via
    `project export-route-proposal <dir> --net <uuid> --from-anchor
    <pad_uuid> --to-anchor <pad_uuid> --out <path>` and
    `project route-apply-selected <dir> --net <uuid> --from-anchor
    <pad_uuid> --to-anchor <pad_uuid>`
  - that same selector now also has a bounded explanation surface via
    `project route-proposal-explain <dir> --net <uuid> --from-anchor
    <pad_uuid> --to-anchor <pad_uuid>`
  - that same selector now also accepts a bounded deterministic profile set
    via `--profile default|authored-copper-priority`; the default order is
    unchanged, while `authored-copper-priority` prepends the accepted
    authored-copper-graph policy family ahead of the existing default family
    order without introducing scoring
  - a narrow MCP parity reopening now covers the selector lane via
    `route_proposal`, `route_proposal_explain`, `export_route_proposal`, and
    `route_apply_selected`
  - the first artifact/export/apply write lane now covers:
    the accepted plus-one-gap bridge, the accepted single-layer
    `route-path-candidate` contract, and the accepted bounded single-via
    `route-path-candidate-via` contract, and the accepted bounded two-via
    `route-path-candidate-two-via` contract, and the remaining bounded
    `three`/`four`/`five`/`six`-via plus `authored-via-chain` contracts, and
    the accepted same-layer orthogonal dogleg contract, and the accepted
    same-layer orthogonal two-bend contract, and
    the accepted zone-aware and zone-obstacle-aware existing-copper graph
    reuse contracts, and the accepted topology-aware plus layer-balance-aware
    topology-aware zone-obstacle-aware existing-copper graph reuse contracts,
    and the accepted obstacle-aware existing-copper graph reuse contract, and
    the policy-selected authored-copper graph family over the accepted bounded
    policy set
  - new slices must still avoid broad autorouting semantics, invented
    constraints, and untracked MCP drift
  - closure interpretation:
    - this frontier is the intended `M5` closure target under the narrowed
      routing-kernel milestone definition above
    - remaining placement-kernel work is explicitly out of scope for `M5`
      closure

## M6 Opening Charter

Status: [~] Frozen pending evidence
- Authority for the proposed opening charter and entry criteria:
  `specs/progress/m6_opening.md`
- Recommended M6 focus: read-only deterministic strategy reporting layered on
  top of the completed M5 routing-kernel substrate.
- Current M6 checkpoint chain:
  - `project route-strategy-report <dir> --net <uuid> --from-anchor <pad_uuid> --to-anchor <pad_uuid> [--objective <objective>]`
  - `project route-strategy-compare <dir> --net <uuid> --from-anchor <pad_uuid> --to-anchor <pad_uuid>`
  - `project route-strategy-delta <dir> --net <uuid> --from-anchor <pad_uuid> --to-anchor <pad_uuid>`
  - `project write-route-strategy-curated-fixture-suite --out-dir <path> [--manifest <path>]`
  - `project capture-route-strategy-curated-baseline --out-dir <path> [--manifest <path>] [--result <path>]`
  - `project route-strategy-batch-evaluate --requests <path>`
  - `project inspect-route-strategy-batch-result <path>`
  - `project validate-route-strategy-batch-result <path>`
  - `project compare-route-strategy-batch-result <before> <after>`
  - `project gate-route-strategy-batch-result <before> <after> [--policy <policy>]`
  - `project summarize-route-strategy-batch-results [--dir <path> | --artifact <path> ...] [--baseline <path> --policy <policy>]`
  - accepted objective set currently reuses the selector profile vocabulary:
    - `default`
    - `authored-copper-priority`
  - the report maps the requested objective to one existing selector profile,
    explains that mapping, and includes the current live selector outcome under
    that profile without reopening M5 routing semantics
  - the comparison report evaluates that same accepted objective/profile set,
    reports the current live selector outcome for each entry, and recommends
    one profile under a deterministic baseline-preserving comparison rule
  - the decision-delta report reduces that same accepted objective/profile set
    to one bounded explicit delta classification plus one short material
    difference summary for the user
  - the batch evaluation surface runs the existing report/compare/delta
    surfaces over a versioned explicit request manifest and returns both
    per-request evidence and aggregate summary counts
  - one curated fixture-suite writer now materializes a deterministic native
    fixture set plus a compatible batch-request manifest for repeated evidence
    runs over real persisted projects
  - one curated baseline-capture surface now materializes that fixture suite,
    runs the existing batch evaluator, and saves one reusable versioned
    batch-result artifact with deterministic default paths
  - one checked-in repo baseline asset set now lives under
    `crates/test-harness/testdata/quality/route_strategy_curated_baseline_v1`
    and the alignment CI lane regenerates and strict-gates a fresh run against
    that baseline through `scripts/check_route_strategy_evidence.py`
  - the current curated suite covers:
    - same-outcome baseline route selection
    - profile divergence between `default` and
      `authored-copper-priority`
    - no-proposal-under-any-profile
    - one cross-layer routable same-outcome case
  - the batch-evaluate JSON output is now also the explicit saved result
    artifact format with `kind = native_route_strategy_batch_result_artifact`
    and `version = 1`
  - saved batch result artifacts can now be inspected and structurally
    validated through read-only surfaces that report summary/distribution
    details, per-request outcomes, malformed entries, version compatibility,
    required-field coverage, and deterministic summary/result integrity checks
  - saved batch result artifacts can now also be compared without live
    re-evaluation by `request_id`, reporting aggregate count deltas,
    added/removed/common request ids, common-request recommendation/delta/live
    outcome changes, and one bounded summary classification
  - saved batch result artifact comparisons can now also be evaluated as a
    read-only CI/review gate under the explicit accepted policy set
    `strict_identical|allow_aggregate_only|fail_on_recommendation_change`,
    reporting pass/fail reasons and count facts while returning CLI exit code
  - the current M6 implementation frontier is intentionally frozen pending
    repeated evidence from the checked-in baseline gate and curated fixture
    suite; new semantics are not the default next step

## M7 Opening Charter

Status: [~] Opened narrowly as a read-only route-proposal review layer
- Authority for the opening charter and entry criteria:
  `specs/progress/m7_opening.md`
- Concrete workspace/contract/spike definition:
  `specs/M7_FRONTEND_SPEC.md`
- Active imported-board fidelity execution plan:
  `docs/gui/M7_IMPORTED_BOARD_FIDELITY_PLAN.md`
- Active board-review fidelity diagnosis:
  `docs/gui/M7_BOARD_REVIEW_FIDELITY_GAP.md`
- Active M7 focus: one narrow visual review layer on top of the closed M5
  routing-kernel substrate and the frozen M6 strategy-reporting/evidence
  stack.
- Entry conditions satisfied before opening:
  - daemon result serialization no longer unwrap-panic on response encoding in
    `crates/engine-daemon/src/dispatch.rs`; serialization failures now return
    structured internal JSON-RPC errors instead of crashing the daemon
  - one remaining imported-connectivity runtime unwrap in
    `crates/engine/src/connectivity/mod.rs` was removed from the hierarchical
    link resolver path
  - `mcp-server/server_runtime.py` now generates the plain daemon-backed MCP
    request/call wrappers from one shared `DAEMON_CLIENT_METHOD_SPECS` table
    instead of duplicating those wrappers by hand across request builders and
    call helpers
  - `mcp-server/daemon_client_request_tests.py` now sanity-checks that the
    generated daemon client wrappers are installed and preserve required fixed
    parameter shapes such as `rotate_component`
  - governance/docs no longer treat `M5` as the active execution window; M5 is
    closed and M6 is frozen pending evidence
- Implemented opening slice:
  - new native CLI review surface:
    `project review-route-proposal <dir> --net <uuid> --from-anchor <pad_uuid> --to-anchor <pad_uuid> [--profile <profile>]`
  - the same command also supports saved-artifact review with:
    `project review-route-proposal --artifact <path>`
  - the review payload reuses the existing selected route-proposal and route
    proposal artifact data structures, exposing deterministic proposal actions,
    segment evidence, selected contract metadata, and source identity for one
    frontend-consumable review object
  - the same review surface is exposed through MCP as
    `review_route_proposal`
  - apply/export remain in the existing machine-native CLI/MCP surfaces; M7 is
    review-only in this opening slice

- Next accepted frontend slice for `M7`:
  - one fixed route-proposal review workspace defined in
    `specs/M7_FRONTEND_SPEC.md`
  - one deterministic `board_review_scene_v1` contract for the opening board
    review context
  - one locked viewport-centered three-column shell with bottom dock strip
  - one fixed panel taxonomy: `Project`, `Filters`, `Inspector`, `Review`
  - one single-selection read-only interaction model with a separate active
    review target
  - one fixed panel set plus integrated terminal and AI supporting lanes, all
    remaining subordinate to engine authority
  - one explicit authored/proposed/diagnostic visual-state model

- Accepted post-spike correction track inside opening `M7`:
  - one bounded imported-board fidelity program defined in
    `docs/gui/M7_IMPORTED_BOARD_FIDELITY_PLAN.md`
  - scope split:
    - import fidelity for KiCad PCB truth preservation
    - scene-contract fidelity for explicit authored / unrouted / proposed /
      diagnostic lanes
    - renderer fidelity for PCB-native semantic readability
  - sequencing rule:
    - execute after the architecture spike proves the
      `gui-app` / `gui-protocol` / `gui-render` boundary
    - execute before broadening imported-board review claims or generalizing
      the opening `M7` workflow beyond the current bounded review surface
  - current correction-track read:
    - Stage 3 unrouted-lane work is functionally landed on the canonical
      half-routed fixture
    - previously recorded Stage 1 / Stage 2 blockers around outline ownership,
      pad rotation, roundrect semantics, outline layer-id carriage, and outline
      visibility gating have materially landed in code; the fidelity docs are
      now being reconciled to that newer repo state
    - `M7-INT-001` closed its first slice (2026-06-09): authored-object
      selection ownership, switch-clears-prior, and hover-preview-only
      behavior are regression-locked in
      `crates/gui-render/tests/selection_ownership.rs`
    - `M7-REN-006` closed (2026-06-09): the render-stack policy now has a
      single code encoding (`RenderStage` declaration order, shared
      `POST_COPPER_STAGES` walk), copper-layer appearance is constructed
      material-first, the bounded exception set is documented at the
      retained-geometry pass header, and contract regression tests lock the
      declared stage ladder
    - `M7-REN-003` closed (2026-06-12): proposed overlays verified
      copper-like at world-true width in selected and non-selected states;
      the diagnostic per-vertex marker violation ("generic path nodes" over
      proposed copper) was removed and regression-locked
    - `M7-REN-004` closed (2026-06-12): filled-zone copper is a declared
      derived shade of the layer material so pad/teardrop/pour boundaries
      read correctly (teardrop tangency criterion verified on DOA2526), and
      dim-unrelated readability is verified on the canonical fixture with
      regression locks in `render_contract_tests.rs`
    - active next implementation frontier is `M7-REG-001..003` fixture-backed
      import/scene/visual regression coverage (owner-ordered 2026-06-11)
    - the renderer semantic contract note now lives in
      `docs/gui/M7_RENDER_SEMANTIC_CONTRACT.md`
  - standards amendment for the opening slice:
    - opening `M7` remains review-focused and does not expand into a full IPC
      authoring/validation milestone
    - standards-relevant imported observables already exposed in the review
      surface must preserve source truth
    - bounded import-audit diagnostics are in-scope where they report delta
      without mutating imported geometry

- Acceptance checks met for the opening slice:
  - deterministic repeated output on unchanged persisted state
  - no pool re-resolution or live import-session dependence
  - no invented routing geometry or inferred constraints beyond persisted facts
  - engine + CLI test coverage for sorted/deterministic extraction

- Acceptance checks required for the next frontend slice:
  - deterministic repeated `board_review_scene_v1` output on unchanged
    persisted state
  - stable authored and review companion identities on unchanged state
  - no frontend-owned semantic geometry or review inference beyond explicit
    machine contracts
  - explicit authored vs proposed vs diagnostic visual separation in the
    opening workspace
  - terminal and AI supporting lanes consume explicit selection/review context
    without becoming parallel design truth
  - the opening route review starts from the first proposal action in
    deterministic review order

- Acceptance checks required for the imported-board fidelity track:
  - supported KiCad PCB layer identities do not silently collapse to fallback
    copper layers
  - supported imported pads preserve the physical dimensions and shape
    semantics required for board review
  - unsupported imported-board cases fail explicitly or remain clearly bounded
    instead of silently producing materially wrong board meaning
  - the accepted fixture set can visually distinguish authored copper,
    unrouted connectivity, proposed overlay geometry, and board-context
    primitives reliably enough for a PCB user to trust the review
  - fixture-backed tests plus screenshot or image-based review cover the
    canonical half-routed board and the supporting imported-board edge cases

---

## Standards Audit Batch 1 — Spec Stubs Awaiting Implementation

Spec edits landed in commit `db98eff` (2026-04-17) per the apply order in
`docs/STANDARDS_AUDIT_BATCH_1_GUIDANCE.md`. This section tracks each stub
against its implementation status. Status semantics:
- `[x]` spec/doc text landed in the named anchor
- `[ ]` implementation work not started (no Rust type, no engine op, no
  pool storage, no MCP runtime entry, no importer/exporter, etc.)
- A row may carry both: `[x]` spec landed + `[ ]` implementation pending —
  shown as two columns

### Pass 0 — Standards Compliance Disposition Refresh

| Stub | Spec anchor | Spec | Impl |
|------|-------------|:----:|:----:|
| Domain 1 disposition refresh (STEP/IDF/IDX/EDMD/DXF prerequisites; Gerber X3 / IPC-2581C / IPC-D-356 / ODB++ contracts) | `specs/STANDARDS_COMPLIANCE_SPEC.md` §4.1 | [x] | [—] N/A (disposition text only) |
| Domain 2 disposition refresh (IBIS/Touchstone/SPICE attachment; encrypted-content policy) | `specs/STANDARDS_COMPLIANCE_SPEC.md` §4.2 | [x] | [—] N/A (disposition text only) |

### Pass 1 — Engine Schema Bedrock

| Stub | Spec anchor | Spec | Impl |
|------|-------------|:----:|:----:|
| `ModelRole`, `SpiceDialect`, `EncryptionScheme`, `ModelAttachment`, `ModelProvenance`, `ModelFormatMetadata` (D2-1) | `specs/ENGINE_SPEC.md` §1.1a | [x] | [~] (engine schema is model-backed and validated through typed part behavioural-model authoring; attach command extracts file hash/provenance and promotes source files into `pool/models`; detach removes attachment edges; richer file parsing remains pending) |
| `ModelFormat` enum, typed `Transform3D`, expanded `ModelRef`, `Package.body_height_nm` / `body_height_mounted_nm` (D1-2) | `specs/ENGINE_SPEC.md` §1.1a + §1.2 | [x] | [~] (`ModelFormat`, deterministic typed `Transform3D`, expanded `ModelRef`, and package body-height fields are model-backed; `add-pool-package-model-3d` and `set-pool-package-body-heights` journal typed authoring; model provenance remains an empty slot and richer 3D import/export use remains pending) |
| `StackupLayer` material fields (`dielectric_constant`, `loss_tangent`, `copper_weight_oz`, `roughness_um`, `material_name`) (D1-3) | `specs/ENGINE_SPEC.md` §1.3 | [x] | [~] (engine schema is model-backed with deterministic JSON-number material fields, legacy board JSON deserializes with unset material metadata, and `set-board-stackup` plus `datum.pcb.set_stackup` accept backward-compatible material-aware layer tuples; richer KiCad stackup-material extraction and impedance-solving use remain pending) |
| `Net.controlled_impedance: Option<ImpedanceSpec>` and `ImpedanceSpec` (D1-4) | `specs/ENGINE_SPEC.md` §1.3 | [x] | [~] (engine schema is model-backed with deterministic JSON-number impedance target/tolerance fields, legacy board-net JSON deserializes with no impedance target, and `place-board-net` / `edit-board-net` plus canonical MCP `datum.pcb.place_net` / `edit_net` can author, update, and clear per-net controlled-impedance metadata; impedance solving/export use remains pending) |
| `Part` extensions (`manufacturer_jep106`, `packaging_options`, `behavioural_models`, `thermal`, `supply_chain_offers`, `last_supply_chain_check`) plus `ThermalSpec`, `PackagingKind`, `PackagingOption`, `SupplyOffer` (D2-2) | `specs/ENGINE_SPEC.md` §1.2 | [x] | [~] (`manufacturer_jep106`, `packaging_options`, `behavioural_models`, `thermal`, `supply_chain_offers`, and `last_supply_chain_check` are model-backed and journal-authorable through typed part commands; timestamp is currently serialized as a string rather than a chrono `DateTime<Utc>`) |
| `AttachModel` / `DetachModel` operations with `inverse()` reversibility (D2-4) | `specs/ENGINE_SPEC.md` §3 | [x] | [~] (`attach-pool-part-model` promotes a content-addressed model file and journals typed `AttachPoolPartModel` operations over the part attachment list; `detach-pool-part-model` journals typed `DetachPoolPartModel` operations by attachment/model UUID; undo/redo restores exact attachment arrays while retaining shared content-addressed blobs; richer blob lifecycle policy remains pending) |

### Pass 2 — Pool and Native Persistence

| Stub | Spec anchor | Spec | Impl |
|------|-------------|:----:|:----:|
| `pool/models/{ibis,spice,touchstone,ami,thermal}/` directory; `models` and `part_model_attachments` SQL index tables (D2-3) | `docs/POOL_ARCHITECTURE.md` §2 | [x] | [~] (`attach-pool-part-model` writes content-addressed model files under `pool/models/<role>/`; `project query pool-models`, flat MCP `get_pool_model_blobs`, and canonical `datum.library.pool_models` enumerate blobs, recompute SHA-256, derive deterministic model UUIDs, report part attachment references, and expose `referenced` / `orphaned` lifecycle flags; `project gc-pool-models` provides dry-run/apply cleanup for orphaned regular hash-matching blobs; `project validate` now fails missing provenance-backed blobs, filename/hash mismatches, and bad deterministic model UUIDs; SQL/index tables, richer GC policy, and bundle handling remain pending) |
| `pool/models/` in native project layout; new "Pool Model Files" schema (D2-9) | `specs/NATIVE_FORMAT_SPEC.md` §4 + §6.x | [x] | [~] (native projects can materialize, query, validate, and conservatively garbage-collect orphaned regular `pool/models/<role>/<sha256>.<ext>` blobs via attach/query/validate/gc commands with hash and attachment-integrity verification; migration, AMI bundle hashing, richer orphan lifecycle policy, and full bundle schema remain pending) |

### Pass 3 — MCP API Stubs

| Stub | Spec anchor | Spec | Impl |
|------|-------------|:----:|:----:|
| M7+ Export Tools: `export_step`, `export_idf`, `export_odbpp`, `export_ipc2581`, `import_dxf_outline` (D1-5) | `specs/MCP_API_SPEC.md` "M7+ Export Tools" | [x] | [ ] (not in `tools_catalog_data.py`; would need exporter implementations) |
| Component Modelling Tools: `attach_ibis`, `attach_touchstone`, `attach_spice`, `validate_*`, `extract_*`, `export_spice_netlist`, `lookup_part_*`, `refresh_supply_chain`, `find_alternate_parts`, `query_packaging_options`, `normalize_manufacturer`, `infer_diffpair_from_pinnames` (D2-5) | `specs/MCP_API_SPEC.md` "Component Modelling Tools (M7+)" | [x] | [ ] (not in `tools_catalog_data.py`) |
| Encrypted Content Handling Policy (D2-6) | `specs/MCP_API_SPEC.md` top-level | [x] | [—] N/A (policy framing) |

### Pass 4 — Import Spec

| Stub | Spec anchor | Spec | Impl |
|------|-------------|:----:|:----:|
| IPC-2581 Import (Future — Post-M7) rationale + feature matrix (D1-7) | `specs/IMPORT_SPEC.md` §5 | [x] | [ ] (importer deferred) |
| KiCad and Eagle import matrices: SPICE/IBIS/Touchstone rows promoted Deferred → Best-effort (M7+) with `Part.behavioural_models` mapping note (D2-8) | `specs/IMPORT_SPEC.md` §3 + §4 | [x] | [ ] (importer deferred) |

### Pass 5 — Architecture and Scope Docs

| Stub | Spec anchor | Spec | Impl |
|------|-------------|:----:|:----:|
| Behavioural model attachment subsection (D2-7) | `docs/LIBRARY_ARCHITECTURE.md` | [x] | [—] N/A (architecture text) |
| Interop scope re-organisation — Hard/Should/On-demand/Out-of-scope per Domain 1 (D1-1) | `docs/INTEROP_SCOPE.md` §Future (M5+) | [x] | [—] N/A (scope text) |
| Behavioural model attachment & export scope buckets (D2-10) | `docs/INTEROP_SCOPE.md` new section | [x] | [—] N/A (scope text) |

### Deferred — Held For A Later Batch

| # | Spec target | Why deferred |
|---|-------------|--------------|
| D1-6 | `specs/NATIVE_FORMAT_SPEC.md` §12 or `docs/POOL_ARCHITECTURE.md` | `.gitignore` / `.gitattributes` conventions — descriptive, no contract impact |
| D1-8 | `docs/COMMERCIAL_INTEROP_STRATEGY.md` §10 | "Datum's Open-Stack Position" appendix — marketing position |
| D2-11 | `docs/COMMERCIAL_INTEROP_STRATEGY.md` | "Behavioural Model Stack — Open-Stack Position" appendix — marketing position |

**Standards Audit Batch 1 overall**: [x] spec/doc/policy stubs landed; [~]
implementation is mixed. Model schema, part metadata, stackup materials,
controlled-impedance metadata, package/body/model geometry, part model
attach/detach, and first pool-model blob query/verification are partial.
SQL indexes, AMI bundle handling, blob lifecycle GC, migration, richer parsing,
MCP query aliases, and importer/exporter handlers remain pending.

---

## ENGINE_SPEC.md — Core Types

### §1.1 Geometry Primitives

| Type | Status |
|------|--------|
| Point | [x] |
| Rect | [x] |
| Polygon | [x] |
| Arc | [x] |
| LayerId | [x] |

### §1.1a Shared Enums

| Type | Status | Notes |
|------|--------|-------|
| PinDirection (10 variants) | [x] | In pool/mod.rs as PinElectricalType on SymbolPin |
| Lifecycle | [x] | In pool/mod.rs |
| StackupLayerType | [x] | In board/mod.rs |
| PortDirection | [x] | In schematic/mod.rs |
| Primitive | [x] | In pool/mod.rs (Symbol graphics) |

### §1.2 Pool Types

| Type | Status |
|------|--------|
| Pin | [x] |
| Unit | [x] |
| Gate | [x] |
| Entity | [x] |
| Pad | [x] |
| Package | [x] |
| PadMapEntry | [x] |
| Part | [x] |

### §1.3 Board Types

| Type | Status |
|------|--------|
| Board | [x] |
| PlacedPackage | [x] |
| Track | [x] |
| Via | [x] |
| Zone | [x] |
| Net | [x] |
| NetClass | [x] |
| Stackup / StackupLayer | [x] |
| Keepout | [x] |
| Dimension | [x] |
| BoardText | [x] |

### §1.4 Schematic Types

| Type | Status |
|------|--------|
| Schematic | [x] |
| Sheet | [x] |
| PlacedSymbol | [x] |
| SheetDefinition | [x] |
| SheetInstance | [x] |
| SchematicWire | [x] |
| Junction | [x] |
| NetLabel / LabelKind | [x] |
| Bus | [x] |
| BusEntry | [x] |
| HierarchicalPort | [x] |
| NoConnectMarker | [x] |
| Variant | [x] |

### §1.5 Rule Types

| Type | Status |
|------|--------|
| Rule | [x] |
| RuleType (7 variants) | [x] |
| RuleParams (7 variants) | [x] |
| RuleScope (leaf nodes) | [x] |
| RuleScope (combinators) | [x] parse, [—] eval deferred to M6 |

### §2 Object Invariants

| Invariant | Status | Notes |
|-----------|--------|-------|
| All authored objects have non-nil UUID | [x] | Enforced by constructors |
| No duplicate UUIDs within type | [x] | `project validate` checks persisted UUID-key consistency and duplicate authored UUIDs across native project object types |
| Non-dangling references | [x] | `project validate` checks required native files plus persisted schematic/board cross-file references and board-side object references |
| Integer coordinates (no float) | [x] | i64 throughout |
| Connectivity recomputed from authored data | [x] | |

### §3 Operations

| Item | Status |
|------|--------|
| Operation trait | [ ] |
| OpDiff | [ ] |
| Transaction | [ ] |
| Undo/redo semantics | [ ] |
| Derived data invalidation | [ ] |

### §5 Engine API

| Method | Status | Notes |
|--------|--------|-------|
| new() | [x] | |
| has_open_project() | [x] | |
| import() | [x] | KiCad skeleton + Eagle .lbr |
| save() | [~] | Writes unmodified imported KiCad boards byte-identically, persists current `delete_track`/`delete_via`/`delete_component` removals, rewrites current `move_component`/`rotate_component` footprint placement and component `Value`/`Reference` properties, rewrites package-backed footprint bodies for current `set_package`, `set_package_with_part`, `replace_component`, `replace_components`, `apply_component_replacement_plan`, `apply_component_replacement_policy`, `apply_scoped_component_replacement_policy`, and `apply_scoped_component_replacement_plan` slices, and writes rule/part-assignment/package-assignment/net-class sidecars for current `set_design_rule`/`assign_part`/`set_package`/`set_package_with_part`/`replace_component`/`replace_components`/`apply_component_replacement_plan`/`apply_component_replacement_policy`/`apply_scoped_component_replacement_policy`/`apply_scoped_component_replacement_plan`/`set_net_class` slice |
| save_to_original() | [~] | Current M3 helper for imported-design write-back to original file |
| search_pool() | [x] | |
| import_eagle_library() | [x] | |
| get_board_summary() | [x] | |
| get_components() | [x] | |
| get_package_change_candidates() | [x] | Current engine API supports component-scoped package compatibility introspection |
| get_part_change_candidates() | [x] | Current engine API supports component-scoped part compatibility introspection |
| get_component_replacement_plan() | [x] | Current engine API supports unified component replacement planning introspection |
| get_scoped_component_replacement_plan() | [x] | Current engine API supports scoped policy-driven replacement preview introspection |
| edit_scoped_component_replacement_plan() | [x] | Current engine API supports scoped replacement preview post-processing via exclusions and explicit compatible target overrides |
| replace_components() | [x] | Current M3 board write slice supports batched explicit component replacement as one transaction / one undo step |
| apply_component_replacement_plan() | [x] | Current M3 board write slice resolves package/part selections from the unified replacement plan and applies them as one transaction |
| apply_component_replacement_policy() | [x] | Current M3 board write slice resolves deterministic best-candidate replacement policies from the unified replacement plan and applies them as one transaction |
| apply_scoped_component_replacement_policy() | [x] | Current M3 board write slice resolves deterministic best-candidate replacement policies over a scoped component filter and applies them as one transaction |
| apply_scoped_component_replacement_plan() | [x] | Current M3 board write slice validates and applies a previously previewed scoped replacement plan without re-resolving policy |
| get_net_info() | [x] | Returns all nets, not single-net |
| get_stackup() | [x] | |
| get_unrouted() | [x] | |
| get_schematic_summary() | [x] | |
| get_sheets() | [x] | |
| get_symbols() | [x] | |
| get_ports() | [x] | |
| get_labels() | [x] | |
| get_buses() | [x] | |
| get_noconnects() | [x] | |
| get_hierarchy() | [x] | |
| get_schematic_net_info() | [x] | Returns all nets, not single-net |
| get_connectivity_diagnostics() | [x] | |
| get_check_report() | [x] | |
| delete_component() | [x] | Current M3 board write slice |
| delete_track() | [x] | Current M3 board write slice |
| delete_via() | [x] | Current M3 board write slice |
| move_component() | [x] | Current M3 board write slice |
| rotate_component() | [x] | Current M3 board write slice |
| set_value() | [x] | Current M3 board write slice |
| set_reference() | [x] | Current M3 board write slice |
| assign_part() | [x] | Current M3 board write slice |
| set_package() | [x] | Current M3 board write slice |
| set_package_with_part() | [x] | Current M3 board write slice |
| set_net_class() | [x] | Current M3 board write slice |
| set_design_rule() | [x] | Current M3 board write slice |
| run_erc_prechecks() | [x] | |
| run_drc() | [x] | Engine API method implemented (connectivity + clearance checks currently) |
| execute() / execute_batch() | [ ] | |
| undo() / redo() | [~] | Current engine API supports transaction reversal for `delete_component`, `delete_track`, `delete_via`, `move_component`, `rotate_component`, `set_value`, `set_reference`, `assign_part`, `set_package`, `set_package_with_part`, `set_net_class`, and `set_design_rule` |

---

## CHECKING_ARCHITECTURE_SPEC.md

| Item | Status | Notes |
|------|--------|-------|
| CheckDomain enum (ERC, DRC) | [x] | Implemented as check report domain field |
| CheckSeverity (Error, Warning, Info) | [x] | ErcSeverity in erc/mod.rs |
| CheckSummary | [x] | In api/mod.rs |
| CheckWaiver (domain, target, rationale) | [x] | In schematic/mod.rs |
| WaiverTarget variants | [x] | Object, RuleObject, and RuleObjects matching exercised across ERC/DRC tests |
| Waiver matching in ERC | [x] | Includes authored native-project waivers in existing project ERC/check flows |
| Waiver matching in DRC | [x] | `run_drc` now applies authored DRC waivers and keeps waived findings visible |
| Cross-domain checks excluded from M2 | [x] | Not implemented (correct) |

---

## ERC_SPEC.md

| Item | Status | Notes |
|------|--------|-------|
| PinElectricalType enum (10 variants) | [x] | In schematic/mod.rs SymbolPin |
| NetSemanticClass enum | [~] | Partial; power/signal inference exists |
| M2 rule: output_to_output_conflict | [x] | |
| M2 rule: undriven_input | [x] | |
| M2 rule: power_without_source | [x] | |
| M2 rule: noconnect_connected | [x] | |
| M2 rule: unconnected_required_pin | [x] | |
| M2 rule: passive_only_net | [~] | No distinct rule/code ships; passive pins only modulate `input_without_explicit_driver` severity (verified 0 grep hits in `erc/` 2026-06-22). Standalone check is target work. |
| M2 rule: hierarchical_connectivity_mismatch | [x] | Sheet-level hierarchical label/port mismatch check implemented |
| Compatibility matrix (pin-pair) | [~] | Driving analysis exists; full matrix not formalized |
| ErcReport struct | [x] | Via CheckReport |
| ErcViolation struct | [x] | ErcFinding |
| Waiver integration | [x] | |
| Configurable severity | [x] | ErcConfig with BTreeMap overrides |

---

## IMPORT_SPEC.md

| Item | Status | Notes |
|------|--------|-------|
| NAMESPACE_KICAD UUID | [x] | ir/ids.rs |
| NAMESPACE_EAGLE UUID | [x] | ir/ids.rs |
| import_uuid() function | [x] | |
| KiCad object path construction | [~] | Skeleton parser; not all object types mapped |
| Eagle object path construction | [x] | Full library mapping |
| .ids.json schema | [x] | import/ids_sidecar.rs |
| .ids.json precedence rules (3 cases) | [x] | restore_or_merge_mappings() |
| KiCad board feature matrix (Required items) | [~] | Skeleton only; geometry not fully parsed |
| KiCad schematic feature matrix (Required items) | [~] | Skeleton only |
| Eagle board feature matrix | [ ] | Design import not implemented |
| Eagle schematic feature matrix | [ ] | Design import not implemented |
| Eagle library feature matrix | [x] | Symbols, packages, devicesets |

---

## SCHEMATIC_CONNECTIVITY_SPEC.md

| Item | Status | Notes |
|------|--------|-------|
| Sheet-local wire connectivity | [x] | Union-find in connectivity/mod.rs |
| Junction semantics | [x] | |
| Local labels | [x] | |
| Global labels | [x] | |
| Hierarchical labels/ports | [~] | Basic support; full hierarchy links partial |
| Power symbols | [x] | |
| Bus/member expansion | [x] | Basic imported KiCad subset now supports deterministic `NAME[n]` scalar member normalization, `NAME[a..b]` bus-range expansion, and geometric bus-entry association; advanced syntax edge cases remain deferred |
| Net naming and identity | [x] | |
| Connectivity diagnostics | [x] | |
| Deterministic graph output | [x] | Sorted by UUID |

---

## SCHEMATIC_EDITOR_SPEC.md — M4 Operations

Status: [x] Closed for scoped M4 slice
- Verified by code audit in `crates/cli/src/cli_args.rs`,
  `crates/cli/src/command_exec.rs`, and dedicated CLI test shards:
  `main_tests_project_symbol*.rs`, `main_tests_project_label.rs`,
  `main_tests_project_wire.rs`, `main_tests_project_junction.rs`,
  `main_tests_project_port.rs`, `main_tests_project_bus.rs`,
  and `main_tests_project_noconnect.rs`.
- Implemented schematic operation families include symbol
  place/move/rotate/mirror/delete, symbol field add/edit/delete,
  label place/rename/delete, wire draw/delete, junction place/delete,
  hierarchical port place/edit/delete, bus create/edit/place-entry/delete-entry,
  and no-connect place/delete.
- Remaining editor expansion items such as power-symbol placement,
  sheet-instance lifecycle operations, and deterministic annotate flow
  completion are deferred beyond scoped M4 closure.

---

## NATIVE_FORMAT_SPEC.md

Status: [x] Closed for scoped M4 slice
- Native project scaffold, deterministic file layout, and first native
  read/query/check surfaces are implemented in the current M4 slice.
- Remaining native-format contract areas (full schema coverage, migration
  completeness, and richer manufacturing/output semantics remain open beyond
  scoped M4 closure.

---

## Infrastructure

| Item | Status | Notes |
|------|--------|-------|
| Rust workspace (7 crates) | [x] | engine, cli, engine-daemon, test-harness, gui-protocol, gui-render, gui-app (locked via `specs/SPEC_PARITY.md` → `workspace_crates`) |
| Engine compiles without GUI deps | [x] | |
| Test harness (golden file utilities) | [x] | test-harness crate |
| Test corpus (real designs) | [ ] | tests/corpus/ empty |
| Daemon JSON-RPC dispatch | [x] | 54 methods in `dispatch.rs`, with coverage in daemon tests |
| Daemon socket transport | [x] | `main()` parses `--socket` and serves Unix socket; live smoke is environment-gated because sandboxed local runs deny socket IPC |
| MCP Python server (tool host) | [x] | Tool definitions + stdio dispatch |
| MCP→daemon transport | [x] | `EngineDaemonClient.call()` uses Unix socket JSON-RPC; behavioral parity remains covered separately from live socket smoke |
| Git repository initialized | [x] | `main` branch with GitHub remote configured |
| CI pipeline | [x] | `.github/workflows/alignment.yml` runs alignment and file-size budget checks |
