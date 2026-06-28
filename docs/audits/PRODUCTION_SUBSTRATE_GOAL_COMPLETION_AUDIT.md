# Production Substrate Goal Completion Audit

Date: 2026-06-25
Last evidence refresh: 2026-06-27

## Verdict

The production substrate goal is not yet proven complete. The current worktree
shows a substantial first substrate pass with a green full drift gate, but the
authoritative state is still mixed: several core rows in `specs/PROGRESS.md`
remain `[~]`, the embedded terminal is a real PTY but is still explicitly
tracked as partial, and some contracts intentionally describe broader target
behavior than the first production substrate pass.

At the time of this audit's first pass, the repo had no unrelated dirty files.
Subsequent implementation slices may leave intentional working-tree changes
while the active goal remains open. The best current estimate is that the first
production-substrate pass is roughly 90-95% complete. This is not a claim that
Datum is a complete EDA product.

## Requirement Completion Matrix

Evidence refresh on 2026-06-27 included:

- `bash scripts/run_drift_gates.sh` passed, ending with
  `Ran 347 tests in 7.035s OK (skipped=1)`.
- `python3 scripts/check_mcp_public_taxonomy.py` passed with 308 public tools,
  517 registered tools, and 209 hidden compatibility aliases.
- Hidden compatibility alias status is now 209 `deprecated` and 0 retained
  aliases. The flat session aliases `open_project`, `close_project`, `save`,
  and `validate_project` are deprecated in favor of public `datum.session.*`
  replacements.
- Three read-only worker audits inspected the core substrate, authored
  operation/generated-evidence, and CLI/MCP/terminal/GUI slices against current
  files and gates.

Completion status below uses the active production-substrate objective as the
scope. `Done for substrate pass` means the first production substrate requirement
is implemented and proof-gated, while broader product expansion remains normal
future work. `Partial` means the repo still contains an explicit current-state
gap that prevents marking the active goal complete.

| # | Requirement | Status | Completion Evidence | Remaining Blocker |
| --- | --- | --- | --- | --- |
| 1 | ProjectResolver authority | Partial | `scripts/check_resolver_raw_loads.py` passes with zero CLI raw-load exceptions; `PG-RESOLVER-RECOVERY` and `PG-RESOLVER-BACKED-VALIDATION` pass. | `specs/PROGRESS.md` and this audit still identify imported KiCad/Eagle resolution, dependency policy, and remaining mutation/export routing as pending. |
| 2 | Source-shard substrate | Partial | `PG-RESOLVER-RECOVERY`, `PG-SOURCE-SHARD-CONTEXT`, and `PG-SOURCE-SHARD-GUI-SUMMARY` pass; resolver/debug, terminal context, GUI summary, and source-shard taxonomy coverage exists across authored design, sidecar metadata, and generated evidence. | Remaining source-shard recovery semantics are still tracked as open as new shard families land. |
| 3 | ObjectId/ObjectRevision/ModelRevision | Done for substrate pass | `PG-REVISION-GUARDS` passes; `OperationBatch.expected_model_revision` and `GuardObjectRevision` are enforced before staging and stripped from durable journal operations. | No current blocker for the substrate pass; future operation families must keep adding guards. |
| 4 | Journal hardening | Partial | `PG-COMMIT-ATOMIC+DURABLE-UNDO` passes; journal append, cursor, staged writes, torn-tail, duplicate transaction, generated-evidence replay cases, and missing required board-root replay diagnostics are covered. | Remaining non-generated replay/dirty-state hardening is still explicitly open. |
| 5 | Undo/redo | Done for substrate pass | Compensating `undo_of`/`redo_of` transactions are covered by `PG-COMMIT-ATOMIC+DURABLE-UNDO` and broader undo/redo harnesses. | No current blocker beyond keeping new operation families covered. |
| 6 | Private writer migration | Partial | `scripts/check_schematic_private_writers.py` passes; migrated schematic, production, board, check, import-map, and forward-annotation paths use journaled operations. Forward-annotation review state is now classified as a retired direct-writer path rather than an active exception. | The exception register still contains direct-write classes for bootstrap, fixtures, retired compatibility, proposal apply bridges, generated evidence/export, journal internals, and GUI handoff. |
| 7 | Operation vocabulary | Partial | Central `Operation` enum covers guards, board, schematic, pool/import/proposal, production, generated evidence, relationships/variants, ComponentInstance, and ZoneFill families. `PG-LIBRARY-AUTHORING-SUBSTRATE` and `PG-LIBRARY-AUTHORING-CLI` now prove the current typed pool/library authoring substrate and CLI editor slice. | Remaining operation-vocabulary work is narrower: richer library primitives beyond the current typed slice and policy-driven guard emission from future public surfaces. |
| 8 | Revision guards | Done for substrate pass | `PG-REVISION-GUARDS` passes; stale model revision, stale object guard, matching guard, object revision bump, guard stripping, and stale undo cursor cases are tested. | No current blocker for the substrate pass; future authored surfaces must emit guards. |
| 9 | ComponentInstance | Partial | `PG-COMPONENT-INSTANCE-SUBSTRATE` and `PG-COMPONENT-INSTANCE-PROPOSAL-POLICY` pass; persisted shards, journal create/set/delete, roles, part refs, MCP/CLI, BOM/PnP, and undo/redo exist. | Broader semantic coverage and remaining legacy/compatibility joins outside BOM/PnP remain open. |
| 10 | Relationship/variant substrate | Partial | `PG-SHARD-DIFF-ISOLATION` and `PG-VARIANT-RESOLUTION` pass; authored relationship/variant shards, journal operations, query surfaces, and schema guards exist. | Richer variant composition and first-class UI review remain open. |
| 11 | ImportMap substrate | Partial | `PG-IDENTITY-SUBSTRATE` and `PG-IDENTITY-SUBSTRATE:SCHEMATIC` pass; ImportMap sidecars, journal create/delete, KiCad board/schematic and Eagle library project-root identity reuse slices exist. ImportMap entries now expose lifecycle `status`, with legacy and newly imported entries defaulting to `active`; KiCad board/schematic and Eagle library reimport mark same-source keys absent from the latest source as `missing_in_source`; KiCad board/schematic and Eagle library entries now carry source-facing `source_object_ref` provenance rather than UUID-only refs while preserving `source_hash` and `source_path`. | Eagle board/schematic importers, remaining lifecycle transitions beyond current KiCad board/schematic and Eagle library missing-source reconciliation, richer provenance for future import families, and broader provenance/library operations remain open. |
| 12 | Schematic/PCB authored operations | Partial | Journaled schematic and PCB operation families exist; MCP taxonomy and daemon write-parity gates pass, and authored board/schematic tests are present. | Semantic schematic editors and migration away from remaining compatibility/project-era board commands remain open. |
| 13 | Proposal policy | Partial | `PG-PROPOSAL-PARITY`, `PG-PROPOSAL-SCHEMA+REPLAY`, `PG-COMPONENT-INSTANCE-PROPOSAL-POLICY`, `PG-LIBRARY-AUTHORING-PROPOSALS`, and `PG-PRODUCTION-SCHEMA+POLICY` pass. | Broader proposal lifecycle/source policy coverage remains beyond the current substrate slice. |
| 14 | CheckRun/CheckFinding | Partial | `PG-CHECKRUN-GENERATED-EVIDENCE` and `PG-CHECKRUN-CLI-LIFECYCLE` pass; persisted generated evidence, fingerprints, typed v1 `standards_basis_detail` beside flat compatibility `standards_basis`, an engine-owned v1 check standards-basis registry seam, richer domain target projection for common artifact/zone/pad identifiers, waivers/deviations, repair links, MCP/CLI views, schema guards, and terminal active-context check-history command discovery exist. | Broader project-authored StandardsRegistry integration, richer target categories beyond the current common identifiers, and deeper first-class check-history UX remain open. |
| 15 | Standards repair proposals | Partial | `PG-STANDARDS-REPAIR-PROPOSALS` passes; bounded process-aperture, peer-aperture, track/via geometry, copper-clearance, silkscreen, and ZoneFill repair proposals exist. | Remaining repair families and per-family apply coverage need expansion. |
| 16 | ZoneFill | Partial | `PG-ZONEFILL-SUBSTRATE` and `PG-ZONEFILL-CLI-SOLVER` pass; journaled generated evidence, stale/unfilled/unsupported checks, bounded solver, and repair proposals exist. | General copper-fill solver/rendering/provenance remains bounded and incomplete. |
| 17 | OutputJob/artifacts | Partial | `PG-ARTIFACT-TRACEABILITY:OUTPUT-GRAPH`, `PG-OUTPUT-JOB-RUN-REPLAY`, `PG-OUTPUT-JOB-CLI-REPLAY`, and `PG-ARTIFACT-TRACEABILITY:MANUFACTURING-PROJECTION` pass. `artifact list` latest navigation now uses explicit generated-evidence ordering rather than resolver map order. | Broader production-output execution coverage and ongoing helper-gate alignment remain open. |
| 18 | CLI/MCP taxonomy | Partial | `scripts/check_mcp_public_taxonomy.py`, MCP protocol catalog tests, daemon write-parity, spec parity, and alignment gates pass. All 209 hidden compatibility aliases are now deprecated with public canonical replacements. | Domain contracts still need ongoing reconciliation as new canonical families land. |
| 19 | PTY terminal substrate | Partial | `scripts/check_gui_agent_terminal_convergence.py` passes; real Unix PTY spawn, context env injection, session tabs, lifecycle controls, geometry, event logs, and broad VT parsing coverage exist. | Remaining VT/screen-grid behavior and richer persistent session-history UX remain open. |
| 20 | GUI/agent context integration | Partial | GUI convergence gate passes; terminal-owned agent entry, context snapshots, production/check/source-shard fields, `datum.context.*` normalization, active check-history commands, and assistant bridge retirement are covered. | Broader GUI action coverage and engine-owned tool-session semantics remain open. |

Conclusion: the active goal is not complete. The next production work should
close the remaining `Partial` blockers or explicitly move them out of this
production-substrate goal by ratified scope decision.

## Conductor Findings

| Goal | Status | Authoritative Evidence | Closure Slice |
| --- | --- | --- | --- |
| ProjectResolver authority | Partial | `specs/PROGRESS.md` records `ProjectResolver` as `[~]`: native roots, shard metadata, journal replay, many board/fab reads are resolver-backed, and the base route query pipeline (`route-preflight`, `route-corridor`, `route-path-candidate`, `route-path-candidate-explain`) now has stale-promoted-file regression coverage proving it reads the resolver-materialized board state after journaled stackup/outline/net/pad edits. All `command_project_route_path_candidate*` solver surfaces now load resolver-materialized board state before solver execution, including bounded via-count, authored-copper, authored-via-chain, orthogonal dogleg/two-bend, and orthogonal-graph multi-via variants. Board-pad mutation validation/readback now also uses resolver-materialized board state, with stale-promoted coverage proving journal-created pads can still be edited after promoted `board.json` is reverted. Schematic wire, junction, and no-connect query surfaces now use resolver-materialized sheet shards, with stale-promoted-sheet regressions proving journaled connectivity objects remain queryable before deletion. Native `project validate` now validates resolver-materialized authored JSON roots/sheets instead of promoted files, with stale-promoted schematic root, sheet, and board regression coverage. Fabrication/export reads have moved substantially: Gerber copper, mask, paste, silkscreen, mechanical, plan/set discovery, CSV/Excellon drill, drill class reports, manufacturing wrappers, and native project summary now read resolver-materialized state with stale-promoted regressions. Raw CLI `load_native_project(root)?` use has been eliminated from resolver-backed project loading; the guard now fails on any reintroduced call site. Imported KiCad/Eagle resolution, dependency policy, and remaining mutation/export routing are still pending. | Continue retiring compatibility/import and remaining mutation/export resolver bypasses; keep the raw-load guard at zero exceptions. |
| Source shards | Partial | `specs/PROGRESS.md` records source shards as `[~]`: native manifests, schematic, board, rules, pools, production, artifacts, checks, ZoneFill, and forward-annotation review sidecars are discovered. Promoted-shard recovery coverage now includes board root, schematic sheets, production records, ComponentInstance, relationships/variants, pool library objects, ZoneFill generated evidence, OutputJobRun generated evidence, ArtifactRun generated evidence, CheckRun generated evidence, ArtifactMetadata generated evidence, and the journal-owned forward-annotation review sidecar. Generated-evidence replay coverage now also proves later journaled deletes suppress stale promoted `.datum` files for OutputJobRun, ArtifactRun, CheckRun, ArtifactMetadata, and ZoneFill; ZoneFill correctly falls back to derived `Unfilled` state rather than retaining stale copper islands. Resolver reads and journal staging now reject source-shard kind/path ownership mismatches across authored design, sidecar metadata, and generated evidence. Value-backed replay/staging now uses canonical source-shard metadata construction for ownership validation, schema-version validation, taxon derivation, authority, dirty-state, and hash metadata instead of duplicate weak `SourceShardRef` builders. Byte-backed promoted-shard metadata now uses the same canonical ownership/taxon path: the shared byte builder rejects cross-authority paths, derives pool taxons when applicable, and promoted ArtifactRun plus CheckRun discovery now route through it instead of hand-built refs. Journal-materialized replay now classifies shard dirty state as `Clean`, `Dirty`, `Missing`, or `Unknown` by comparing materialized content to promoted files; replayed authored production shards now explicitly assert `Missing` dirty-state reporting for recovered ManufacturingPlan, PanelProjection, and OutputJob source shards. Pool source shards now require one of the eight concrete typed taxons (`PoolUnit`, `PoolSymbol`, `PoolEntity`, `PoolPart`, `PoolPackage`, `PoolFootprint`, `PoolPadstack`, `PoolPinPadMap`) while preserving `SourceShardKind::Pool` for journal/replay matching; resolver, CLI `resolve-debug`, and MCP `datum.query.source_shards` regressions prove pool symbol taxons are exposed alongside dirty-state reporting, including missing journal-recovered pool symbols. Authored production source shards now also carry concrete taxons (`ManufacturingPlan`, `PanelProjection`, `OutputJob`), with engine and CLI `resolve-debug` regressions proving the taxon remains visible when those journal-recovered promoted files are missing. Authored identity/relationship sidecars, sidecar metadata, and generated evidence now derive concrete taxons for ComponentInstance, Relationship, VariantOverlay, ImportMap, ProposalMetadata, ForwardAnnotationReview, OutputJobRun, ArtifactRun, CheckRun, ZoneFill, and ArtifactMetadata from the same shared source-shard builder path. Generic unknown source-shard kind/authority have been retired, so accepted source shards must now use a concrete family and authority; only promoted-file dirty-state uncertainty can report `Unknown`. Public `project query <root> resolve-debug` output exposes per-shard `dirty_state`, with CLI regressions for `Clean`, stale promoted authored board `Dirty`, missing journal-recovered `ArtifactMetadata` generated evidence with `GeneratedEvidence` authority and `ArtifactMetadata` taxon, missing journal-recovered `ForwardAnnotationReview` sidecar with `SidecarMetadata` authority and `ForwardAnnotationReview` taxon, and typed pool-symbol/production taxonomy. MCP now exposes the same resolver-backed evidence through canonical read-only `datum.query.source_shards`, with native MCP parity coverage asserting `source_shards[].dirty_state` and pool `taxon` for both clean and missing journal-recovered pool-symbol shards. GUI workspace state now carries a resolver-backed `SourceShardStatusSummary`, terminal-triggered refresh updates it with production/check status, and the project panel renders a compact `SOURCE SHARDS` clean/attention indicator; GUI protocol coverage now proves a missing journal-recovered forward-annotation review sidecar increments `missing` and `attention_count`. | Complete the remaining source-shard recovery semantics and keep the GUI summary aligned as new shard families land. |
| ObjectId/ObjectRevision/ModelRevision | Done for substrate pass | `specs/PROGRESS.md` records object/revision substrate as `[~]`; `OperationBatch.expected_model_revision` exists, object revisions bump, and `GuardObjectRevision{object_id, expected_object_revision}` now provides an explicit preflight per-object compare-and-swap guard before journal staging or mutation. Guard operations are stripped before durable journal operations so undo/redo/replay remain revision-safe. Public CLI guard emission now covers board component value/reference/move/part/package/layer/rotation/locked/delete, board layout/routing/pad/netclass/dimension helper paths, schematic connectivity/text/drawing helper paths, ComponentInstance set/delete, manufacturing plan/panel projection/output job set/delete, project name, whole-root project rules replacement, granular project rule set/delete, generic batch-file proposals, and production proposal builders. | Keep extending object guard emission as new authored operation families are added. |
| Journal hardening | Partial | `commit_journaled()` requires `expected_model_revision` and stages journaled writes; journal append rejects duplicate/conflicting IDs and torn tail cases are tested. `journal_replay_recovers_missing_pool_library_shard` closes a promoted-pool-shard recovery gap; `journal_replay_recovers_missing_production_shards` now locks replay recovery plus `Missing` dirty-state reporting for authored ManufacturingPlan, PanelProjection, and OutputJob shards; `journal_replay_recovers_missing_zone_fill_generated_evidence`, `journal_replay_recovers_missing_output_job_run_generated_evidence`, `journal_replay_recovers_missing_artifact_run_generated_evidence`, `journal_replay_recovers_missing_check_run_generated_evidence`, and `journal_replay_recovers_missing_artifact_metadata_generated_evidence` close generated-evidence recovery gaps; `journal_replay_deleted_*_suppresses_stale_promoted_evidence` coverage now proves generated-evidence deletes dominate stale promoted shards across OutputJobRun, ArtifactRun, CheckRun, ArtifactMetadata, and ZoneFill; staged writes enforce source-shard ownership paths; stale promoted board shards now resolve as `Dirty`, missing journal-recovered evidence resolves as `Missing`. | Continue hardening remaining non-generated replay edge cases and expose dirty-state reporting to callers. |
| Undo/redo | Done for substrate pass | Undo/redo are compensating journaled transactions with `undo_of`/`redo_of`, exposed through CLI/MCP journal aliases, covered across many native parity tests, and now include a mixed schematic+PCB+production reopen golden. | Keep undo/redo coverage current as new operation families are added. |
| Private writer migration | Partial | Private-writer gates pass and migrated schematic/production/board paths use journaled operations. Remaining direct-write classes are no longer informal: `docs/audits/PRIVATE_WRITER_MIGRATION_EXCEPTION_REGISTER.md` classifies bootstrap, route-strategy fixtures, retired KiCad compatibility, proposal metadata, generated evidence, journal staging/persistence, generated export, and GUI command-handoff paths with owners and retirement/tightening criteria. The legacy import-map sidecar writer has been retired; import-map creation coverage now uses journaled `Operation::CreateImportMapShard`, unsafe relative paths are rejected through source-shard ownership checks, and undo removes the promoted sidecar. Native CheckRun generation now records generated evidence through journaled `Operation::SetCheckRun`, and the guard rejects `persist_check_run()` in the production native inspect command path. Forward-annotation review state is now a retired direct-writer path, not an active exception: `.datum/forward_annotation_review/review.json` is journal-owned through `Operation::SetForwardAnnotationReview` / `Operation::DeleteForwardAnnotationReview`, and the guard requires those operations while forbidding direct writes or `project.json` rewrites. ComponentInstance, relationship/variant, and production source-shard staging now route new-shard persistence through shared `journal::stage_new_shard_write`, and the private-writer guard expects zero direct `std::fs::write` calls in those family staging files. | Migrate or retire the registered compatibility/generated exceptions as their replacement operation families land; keep the guard/register synchronized. |
| Operation vocabulary | Partial | `Operation` covers project/rules, board package/pads/tracks/vias/zones/nets/netclasses/dimensions/text/keepouts/outline/stackup, pool/import/proposal, production, relationships/variants, ComponentInstance, schematic authoring, ZoneFill, OutputJobRun generated evidence, ArtifactRun generated evidence, CheckRun generated evidence, ArtifactMetadata generated evidence, and explicit `GuardObjectRevision` checks. Project-rule operations now validate the persisted native rule payload, including parsing `scope` as the engine `RuleScope` AST and rejecting invalid structural scope references before journal staging. Production create/set operations now validate semantic references before mutation: panel instances must target the project board, manufacturing/output targets must be the project board or an existing/current panel, optional variants must exist, and output jobs cannot reference missing manufacturing plans. `PG-LIBRARY-AUTHORING-SUBSTRATE` covers resolver discovery, replay, undo, invalid path/identity rejection, typed payload validation, and pool part model attach/detach. `PG-LIBRARY-AUTHORING-CLI` covers current typed pool unit, padstack, package, raw object create/set/delete, resolver-backed query visibility, and validation failures. | Close remaining semantic library editors beyond the current typed slice and policy-driven guard emission from future public tool surfaces. |
| ComponentInstance | Partial | Persisted `.datum/component_instances` shards, resolver validation, journal create/set/delete, MCP aliases, and BOM/PnP integration exist. Promoted ComponentInstance shard discovery now rejects unsupported future source-shard `schema_version` values before deserializing payloads, matching the source-shard schema guard used by journaled ComponentInstance writes. `ComponentInstanceShard` payload envelopes now carry an explicit schema-version constant, legacy missing-version promoted shards default to version 1 for resolver compatibility, and resolver validation rejects unsupported future envelope versions before authored ComponentInstance records enter the model. CLI and MCP bind/set surfaces now accept multi-symbol ComponentInstance refs through repeated `--symbol` / `symbols`, preserve full existing symbol/package ref vectors when constructing inverse payloads, and have regressions proving multi-symbol refs survive bind, set, delete undo, and query. ComponentInstance shards now carry optional `part_ref` identity to a current native `pool/parts` object; resolver validation rejects stale/missing/non-part refs, commit validation rejects stale part revisions before staging, CLI bind/set expose `--part`, and MCP flat/canonical write tools forward the same field to the CLI bridge. ComponentInstance shards now also carry optional per-symbol and per-package `{role, label}` metadata keyed by referenced object UUID; resolver and commit validation reject invalid role metadata, while CLI/MCP bind/set can author `symbol_roles` and `package_roles` and default new bindings to stable role records. BOM/PnP assembly rows project package role/label into `component_instance_role` and `component_instance_label`, compare reports role/label drift, inspect remains legacy-header compatible, and output-job manufacturing-set artifacts preserve role/label columns through stored variant runs. Forward-annotation audit/proposal matching now uses resolver `ComponentInstance` symbol/package refs before legacy reference fallback for uncovered objects, and uses `part_ref` as the expected part identity when present. Forward-annotation artifact compare now classifies exact `action_id` matches as applicable and same symbol/component UUID plus same action/reason as drifted identity when a reference rename changes the derived `action_id`; legacy reference/action fallback has been removed. Engine commit validation also rejects duplicate ComponentInstance refs and symbol/package refs targeting the wrong schematic/board domain. Resolver-derived ComponentInstance joins are retired; remaining ComponentInstance work is broader semantic coverage, not compatibility-join removal. | Replace remaining legacy/compatibility joins outside BOM/PnP. |
| Relationship and variant substrate | Partial | Authored relationship/variant shards, derived statuses/populations, journal operations, queries, and MCP exposure exist. Relationship and VariantOverlay shard discovery now reject unsupported future source-shard `schema_version` values before deserializing payloads, matching the source-shard schema guard used by journaled relationship/variant writes. `RelationshipShard` and `VariantOverlayShard` payload envelopes now carry explicit schema-version constants, legacy missing-version promoted shards default to version 1 for resolver compatibility, and resolver validation rejects unsupported future envelope versions before authored relationship/variant records enter the model. | Add richer variant composition and first-class UI review. |
| Import Map | Partial | Import-key substrate, sidecars, allocation, KiCad footprint project import, journaled import-map creation/delete, and unsupported future sidecar schema rejection exist. `ImportMapShard` now carries an explicit schema-version constant, legacy missing-version promoted sidecars default to version 1 for resolver compatibility, and focused schema regressions prove those legacy sidecars materialize while unsupported future source-shard versions remain quarantined. ImportMap entries now carry lifecycle `status` provenance (`active`, `missing_in_source`, `replaced`, `split`, `merged`); legacy entries missing the field deserialize as `active`, current KiCad/Eagle import sidecar writers stamp `active`, and CLI import-map query/import regressions assert the field is visible. KiCad board, KiCad schematic, and Eagle library reimport now write full same-source lifecycle sidecars when the latest source drops a previously mapped object, preserving current keys as `active` while marking absent keys `missing_in_source`. KiCad board/schematic entries now carry source-facing `source_object_ref` values such as `board-pad:<source_uuid>` and `schematic-symbol:<source_uuid>` instead of UUID-only refs, and Eagle library entries carry source-facing refs for pool units, symbols, devicesets, devices, packages, and padstacks while preserving `source_hash` and `source_path` provenance. Journal replay now applies ImportMap sidecar create/delete operations to the resolver source-shard set and `model.import_map`; focused missing-promoted and stale-promoted regression coverage proves a missing journal-created `.datum/import_map/*.json` sidecar still recovers import-key entries, while a deleted sidecar cannot resurrect stale import-key entries or source-shard metadata. `PG-IDENTITY-SUBSTRATE` now runs the full project import identity module, covering KiCad board object identity, KiCad board source-facing provenance, KiCad footprint pool identity, journal-recovered ImportMap reuse, resolver-materialized pool refs, Eagle `.lbr` pool-object ImportMap persistence, Eagle `.lbr` missing-source lifecycle reconciliation, and Eagle `.lbr` source-facing provenance. KiCad board import has opt-in engine-level identity reuse for board footprints, pads, routed segments, vias, and zones through `import_board_document_with_import_map`, and project-root `import-kicad-board` persists those supported board-object identities by journaling native board objects plus one ImportMap sidecar. `PG-IDENTITY-SUBSTRATE:SCHEMATIC` covers project-root KiCad schematic ImportMap persistence, source-facing provenance, and journal-recovered reuse for source-backed symbols, wires, junctions, labels, buses, bus entries, no-connect markers, schematic text, bounded drawing primitives, generated sheet definitions, generated sheet instances, and deterministic sheet ports. Eagle board/schematic importers, remaining lifecycle transitions beyond current missing-source reconciliation, richer provenance fields for future import families, and broader provenance/library operations remain pending. | Extend Eagle board/schematic importers, remaining lifecycle transitions, richer provenance fields for future import families, and broader provenance/library operations. |
| Schematic authored operations | Partial | Journaled schematic sheet/object operations exist and current progress records canonical CLI/MCP aliases. | Finish semantic schematic editors and proposal policy for higher-risk cross-domain edits. |
| PCB authored operations | Partial | Journaled board component, pad, track, via, zone, net, netclass, dimension, text, keepout, outline, stackup, and name operations exist. | Keep migrating remaining board-authoring surfaces away from compatibility/project-era commands. |
| Proposal policy | Partial | Proposal source/status/apply validation exists; check-source proposals require CheckRun/finding fingerprints; stale proposal apply is guarded. Promoted proposal metadata discovery now rejects unsupported future source-shard `schema_version` values before deserializing proposal payloads. Proposal payloads now carry explicit `schema_version: 1`, legacy missing-version proposal metadata remains readable through a version-1 serde default, and unsupported future payload versions are rejected before proposal records enter the model. Proposal batches now reject proposal lifecycle metadata operations (`CreateProposalMetadata`, `SetProposalMetadata`, `DeleteProposalMetadata`) so proposal review/apply state remains owned by the proposal subsystem rather than user-authored proposal payloads; CLI coverage rejects lifecycle-metadata bypass attempts during proposal creation. Automated production authoring is now classified at the commit gateway: `Tool`/`Assistant` provenance batches that directly create/set/delete authored `ManufacturingPlan`, `PanelProjection`, or `OutputJob` records are rejected with a machine-readable proposal-policy blocker, while accepted assistant-authored production proposals apply through an explicit internal accepted-proposal commit context. Automated generated-evidence authoring is now also classified at the commit gateway: `Tool`/`Assistant` provenance batches that directly set/delete `OutputJobRun`, `ArtifactRun`, `CheckRun`, `ArtifactMetadata`, or `ZoneFill` evidence are rejected with a machine-readable proposal-policy blocker before evidence staging. Automated pool/library authoring is now classified at the same commit gateway: `Tool`/`Assistant` provenance batches that directly create/delete typed pool packages or padstacks, create/set/delete generic pool-library objects, or attach/detach pool part models are rejected with `proposal_required_for_automated_library_operation` before authored pool shards are staged, and the MCP runtime launches CLI-backed library authoring with `DATUM_COMMIT_SOURCE=tool` so canonical `datum.library.*` aliases cannot downgrade automation into CLI provenance. Positive library proposal producers now exist through raw `datum-eda proposal create-pool-library-object` / `datum.proposal.create_pool_library_object` plus semantic `datum-eda proposal create-pool-unit` / `datum.proposal.create_pool_unit`, `datum-eda proposal create-pool-symbol` / `datum.proposal.create_pool_symbol`, `datum-eda proposal create-pool-entity` / `datum.proposal.create_pool_entity`, `datum-eda proposal create-pool-padstack` / `datum.proposal.create_pool_padstack`, `datum-eda proposal create-pool-package` / `datum.proposal.create_pool_package`, `datum-eda proposal set-pool-package-pad` / `datum.proposal.set_pool_package_pad`, `datum-eda proposal set-pool-package-courtyard-rect` / `datum.proposal.set_pool_package_courtyard_rect`, and `datum-eda proposal set-pool-package-courtyard-polygon` / `datum.proposal.set_pool_package_courtyard_polygon`, creating non-mutating draft proposals that apply through the generic proposal gateway. `PG-LIBRARY-AUTHORING-PROPOSALS` now gates the package/pad/courtyard proposal paths and rejection cases. Producer-specific production proposal builders now preflight the same semantic references as direct production operations for missing variants, missing manufacturing plans, and invalid panel board targets before persisting draft proposals. Board-component replacement proposal builders now cover both single-component and repeated multi-component package/part/value replacements, creating guarded non-mutating draft proposals that apply only through the generic proposal gateway. Proposal preview now has a journal-aware path used by the CLI, and focused engine coverage directly asserts preview after-revision, predicted transaction UUID, and applied transaction UUID all match while preview staging is cleaned up without promoted shard writes. | Continue centralizing proposal-first requirements by operation/source risk so direct CRUD cannot bypass review policy. |
| CheckRun/CheckFinding | Partial | Persisted generated-evidence CheckRun/CheckFinding records, journaled CheckRun generated-evidence recovery, deterministic fingerprints, artifact-linked findings, waivers/deviations, CLI/MCP check aliases, and standards findings exist. Native `project query <root> erc` and `project query <root> drc` now return read-only `check_run_v1` profile views instead of top-level raw `ErcFinding[]` / `DrcReport`; regressions prove they are resolver model-revision keyed, expose normalized findings plus raw compatibility data under `raw_report.erc` / `raw_report.drc`, honor existing waiver state, and do not persist CheckRun sidecars. Native check-run generation now commits generated evidence through `Operation::SetCheckRun` rather than direct helper persistence. CheckRun helper persistence now runs the same semantic validator as resolver discovery before writing `.datum/check_runs`, so invalid generated evidence is rejected at the producer boundary and still quarantined if an invalid promoted shard is discovered later. CheckRun payloads now carry explicit `schema_version: 1`, legacy missing-version generated evidence remains readable through a version-1 serde default, unsupported future payload versions are rejected by producer/model validation, and promoted CheckRun discovery rejects unsupported future source-shard schema versions before deserializing persisted check evidence. Persisted `datum-eda check list` history now identifies `latest_check_run_id`, `latest_profile_id`, per-profile latest run summaries, and row-level `latest_for_profile` flags so GUI/agents can choose current evidence without duplicating sort semantics. The affected-object schema is now reconciled: `specs/CHECKING_ARCHITECTURE_SPEC.md` defines `primary_target` plus `related_targets`, `PRODUCT_MECHANICS_009_RULES_CONSTRAINTS_CHECKS.md` documents that pair as the concrete implementation of affected-object identity, CLI plus engine regression coverage rejects legacy `affected_object_ids` and legacy null targets on live and persisted findings, and ERC `object_uuids[]` now project into normalized `object_uuid` targets instead of falling back to `erc/unknown`. Canonical MCP `datum.check.repair_standards` now normalizes repair output around `check_run_id`, `proposal_count`, affected object ids, finding fingerprints, readiness, and blocker codes, while CLI coverage proves ZoneFill standards-repair proposals link back through CheckRun/finding proposal links. Engine-daemon `run_erc` and `run_drc` compatibility methods now also return live non-persisted `check_run_v1` envelopes with normalized findings and raw compatibility data under `raw_report.erc` / `raw_report.drc`, and daemon/MCP `explain_violation` accepts finding fingerprints while retaining index fallback for legacy callers. | Broaden persisted check-history UX, additional repair families, and richer target categories as new check domains land. |
| Standards repair proposals | Partial | Repair generation creates check-source draft proposals for process-aperture pad fixes, track width, via geometry, simple parallel-track copper clearance, silkscreen clearance, and supported ZoneFill honesty findings. Process-aperture repair proposals now include `pad_process_aperture_inherited_from_copper` findings alongside missing/below-rule mask and paste findings, and `pad_process_aperture_inconsistent_with_peer_footprint` findings when a clear same-package majority aperture policy exists, so imported/inconsistent pads are linked to explicit `SetBoardPad` repairs. Copper-clearance repair proposals now cover conservative same-layer parallel horizontal/vertical track pairs by emitting one `SetBoardTrack` offset proposal and deliberately skip non-parallel/topology-aware reroutes. Silkscreen clearance findings now produce `SetBoardText` draft proposals that move the offending board text away from copper without mutating copper geometry. ZoneFill repair proposals now collect `zone_fill_unfilled` and `zone_fill_stale` findings, recompute the fill through the bounded solver, emit `SetZoneFill` only when the result is honestly `Filled`, preserve stale previous evidence for undo/review, and deliberately skip unsupported fills. Standards repair generation now persists the referenced CheckRun through the explicit journaled check-run path before creating check-authored proposals, satisfying proposal source policy after read-only live check queries were separated from persistent check execution. End-to-end CLI coverage now accepts/applies generated process-aperture, peer-aperture, track-width, via-geometry, copper-clearance, silkscreen-clearance, and ZoneFill repair proposals through the generic proposal gateway, verifies each proposal reaches `Applied`, verifies authored/generated geometry updates to the governing standard, and verifies the repaired object's matching standards findings disappear, including inherited/peer process-aperture findings for repaired pads, copper-clearance findings for moved tracks, silkscreen-clearance findings for repaired board text, and ZoneFill findings for repaired zones. | Expand remaining repair families and add apply-after-acceptance coverage for each new family. |
| ZoneFill | Partial | ZoneFill generated evidence, unfilled/stale/unsupported checks, journaled `SetZoneFill`/`DeleteZoneFill`, missing-file journal replay recovery, and a bounded solver exist. The production CLI fill path is now guarded to emit `Operation::SetZoneFill` through `commit_journaled()`, while standards repair proposals reuse the same bounded fill computation and `SetZoneFill` operation for supported unfilled/stale zones instead of creating direct sidecar writes. ZoneFill payloads now carry explicit `schema_version: 1`, legacy missing-version generated evidence remains readable through a version-1 serde default, unsupported future versions are rejected before evidence becomes renderable copper, and replay/source-shard metadata preserves the schema version for generated evidence. The bounded solver now fills same-net zones, rectangular foreign pad/via/track cutouts, multiple non-overlapping obstacle cutouts, conservatively unioned overlapping/touching inflated obstacle clearances, edge-crossing obstacle clearances clipped to zone bounds, non-orthogonal foreign track bounds removed conservatively, authored keepout bounds removed from copper, and unresolved component pads only block fills when their conservative bounds intersect the requested zone. `scripts/check_schematic_private_writers.py` rejects direct `persist_zone_fill()` use in production command files and keeps the helper confined to engine-owned generated-evidence persistence plus tests. | Expand solver/rendering/provenance beyond bounded cases. |
| OutputJob/artifacts | Partial | OutputJob, journaled OutputJobRun generated evidence, journaled ArtifactRun generated evidence, journaled ArtifactMetadata generated evidence, production projections, variant context, CLI/MCP aliases, and GUI output summaries exist. OutputJobRun, ArtifactRun, and ArtifactMetadata recovery are covered without mutating authored `ModelRevision`; `generated_output_artifact_graph_replays_without_model_revision_mutation` links all three evidence families in one transaction, removes promoted evidence shards, verifies journal replay recovery, and verifies the authored model revision remains stable. Authored ManufacturingPlan, PanelProjection, and OutputJob payloads now carry explicit `schema_version: 1`, legacy missing-version records remain readable through a version-1 serde default, and unsupported future versions are rejected by staging/model validation plus resolver discovery. OutputJobRun, ArtifactRun, and ArtifactMetadata payloads now carry explicit `schema_version: 1`, legacy missing-version generated evidence remains readable through version-1 serde defaults, unsupported future payload schema versions are rejected before insertion, and replay/source-shard metadata preserves the version. Promoted OutputJobRun, ArtifactRun, ArtifactMetadata, ManufacturingPlan, PanelProjection, and OutputJob discovery now rejects unsupported future source-shard schema versions before payload deserialization. Run-evidence provenance is now validated when present: blank terminal session IDs, empty terminal context paths, empty project roots, and blank source revisions are rejected before generated evidence enters the resolver model. ArtifactMetadata generator provenance is also enforced: blank `generator_version` values are rejected at helper persistence and resolver discovery before artifact evidence enters the model. Artifact-only BOM/PnP/drill generation now records ArtifactMetadata and unlinked ArtifactRun evidence through journaled `SetArtifactMetadata`/`SetArtifactRun` operations, linked artifact generation records ArtifactMetadata and OutputJobRun evidence through journaled `SetArtifactMetadata`/`SetOutputJobRun` operations, output-job failed/successful/lifecycle run evidence uses journaled `SetOutputJobRun`, Gerber-set export/validation evidence uses journaled `SetArtifactMetadata`/`SetOutputJobRun`, manufacturing-set export/validation evidence uses journaled `SetArtifactMetadata`/`SetOutputJobRun`, and artifact validation state updates use journaled metadata commits; the private-writer gate rejects direct artifact-only metadata/run/output-run, output-job-run, Gerber, and manufacturing evidence helper persistence in the migrated command files. `datum-eda artifact list` now exposes `latest_artifact_id`, `latest_artifact_run_id`, and `latest_output_job_run_id` so GUI surfaces and agents can navigate current generated evidence without scanning full run arrays. Real CLI `project run-output-job` replay coverage now proves missing promoted `.datum/output_job_runs/<run>.json` shards recover from the journal without authored `ModelRevision` mutation for Gerber, single-scope drill, aggregate BOM/PnP, and manufacturing-set jobs; manufacturing-set execution no longer authors a child Gerber OutputJob while producing generated Gerber files. Authored ManufacturingPlan, PanelProjection, and OutputJob shard readers now reject filename/payload UUID mismatches so bad promoted production files are diagnostic-only and excluded from the model. | Broaden production-output execution coverage beyond the linked generated-evidence oracle and keep generated-evidence helper gates aligned as new artifact families land. |
| CLI/MCP taxonomy | Partial | CLI binary is canonical `datum-eda`; public MCP catalog is canonical `datum.*` with hidden compatibility aliases; current public count is 308, registered count is 517, and hidden compatibility count is 209. Granular public families such as `datum.pcb`, `datum.schematic`, `datum.library`, and `datum.route` are now documented as canonical typed realizations of the seven shared contract classes, and `scripts/check_mcp_public_taxonomy.py` locks the public prefix inventory. The stale schematic/rules domain-contract claims that denied existing journaled schematic MCP writes, journaled waiver/deviation operations, persisted CheckRun records, and first standards repair proposal generation have been reconciled. Hidden compatibility aliases now carry machine-readable retirement metadata, and both `scripts/check_mcp_public_taxonomy.py` plus `mcp-server/test_protocol_catalog.py` fail if a hidden alias lacks retirement status/criteria. Public `datum.proposal.*` aliases are now inventoried as read/dry-run or write-classified surfaces, with proposal metadata, review-state, and apply-gateway aliases carrying machine-readable write-surface class/evidence metadata, including single, batch, replacement-plan board-component replacement producers, raw pool-library object proposals, and typed pool unit/symbol/entity/padstack/package/package-pad/package-courtyard proposal producers. Public production authoring aliases under `datum.manufacturing.*` and `datum.output_job.create_gerber_set|create|update|delete` now dispatch to proposal builders, carry `proposal_metadata_write` evidence metadata, and are locked by MCP dispatch/catalog/taxonomy tests; `datum.output_job.run` remains an execution surface rather than a proposal metadata write. Replacement apply aliases are hidden until they route through journal/proposal-backed mutation semantics; replacement read/planning aliases remain public, while their legacy flat read/planning names now carry explicit deprecation criteria pointing to `datum.replacement.*`. Legacy session, manufacturing, OutputJob, route, proposal, PCB, and library names now also carry explicit deprecation criteria pointing to public `datum.session.*`, proposal-mediated, fixture-scoped, journaled, read-only, review-state, apply-gateway, journaled PCB, or library `datum.manufacturing.*`, `datum.output_job.*`, `datum.route.*`, `datum.proposal.*`, `datum.pcb.*`, or `datum.library.*` replacements. No hidden compatibility alias remains on generic retained criteria. | Keep domain contracts current as new canonical families land, then graduate deprecated aliases to `scheduled_for_removal` when client migration evidence exists. |
| MCP route write-surface governance | Done for current route aliases | Public route fixture generators are classified as generated-fixture-only surfaces with deterministic fixture write-policy metadata. Public route apply aliases (`datum.route.apply`, `datum.route.apply_selected`, `datum.route.apply_proposal_artifact`) now carry internal `x_public_write_surface_class` and evidence metadata proving they are journal/proposal-backed rather than unclassified public writes. Their public MCP descriptions and generated CLI help now advertise the proposal journal gateway rather than direct copper mutation. `scripts/check_mcp_public_taxonomy.py`, `mcp-server/test_protocol_catalog.py`, and CLI route-apply help coverage fail if this classification or wording regresses. | Keep any future public write-like alias explicitly classified at introduction time. |
| MCP proposal write-surface governance | Done for current proposal aliases | Public `datum.proposal.*` aliases are now classified by `scripts/check_mcp_public_taxonomy.py` as either read/dry-run surfaces (`list`, `show`, `preview`, `validate`) or write-capable proposal surfaces. Proposal metadata producers carry `proposal_metadata_write`, review/defer/reject carry `proposal_review_state_write`, and accept/apply aliases carry `proposal_gateway_apply`, each with public catalog evidence. The same guard now covers public production authoring aliases that remain outside `datum.proposal.*`: `datum.manufacturing.*` create/update/delete aliases plus `datum.output_job.create_gerber_set|create|update|delete` must dispatch to proposal builders and carry proposal-metadata write evidence. `mcp-server/test_protocol_catalog.py` mirrors the inventory so future proposal or production-authoring aliases cannot appear without explicit read/write classification. | Keep future proposal aliases classified at introduction time and continue linking higher-risk operation families to proposal-first policy. |
| PTY terminal substrate | Partial | GUI opens a real Unix PTY shell, injects Datum context env, records session events, routes GUI command handoffs to the PTY, and now owns terminal state through a `TerminalSessionRegistry` with active-session context isolation. The GUI protocol now exposes `TerminalTabState` plus `active_session_id`, and the registry can spawn, activate, explicitly detach, rename, close, restart, and sync terminal tab snapshots into `TerminalLaneState`; regressions cover multi-session tab state, active-session switching, explicit detach/reattach without process termination, close behavior, restart lineage, latest-context repointing, and PTY geometry publication. Terminal tabs now carry explicit `attached` state, restart count, previous-session lineage, durable event-log path, activity event count, and compact activity summary; tab activation updates the visible activity lane from the selected tab's persisted summary, activation and explicit detach record `detached`/`attached` terminal lifecycle events without changing process-running status, detached active tabs reject raw PTY input until reattached, and the dock renders detached tabs distinctly plus restart counts or activity badges for active tabs. Active PTY geometry is stored as `TerminalLaneState.columns` / `rows`, updated on dock/window resize, reapplied across restart, and surfaced in the dock header as `SIZE CxR`. The terminal dock now renders protocol-backed session tabs plus `+NEW`, `RENAME`, `RESTART`, `DETACH`, and `CLOSE` controls; clickable session tabs activate or reattach a specific PTY session and refresh its latest context, while controls spawn, enter inline rename mode, restart the active PTY, detach the active PTY without killing it, or close the active session. Inline rename mode is protocol-backed through `rename_session_id`, reuses dock text editing, disables raw PTY input while active, commits on Enter, and cancels on Escape. Terminal screen parsing now handles carriage-return rewrites, LF, VT, FF, mutable/default horizontal tab stops via HT, `ESC H`, `CSI I`, `CSI Z`, and `CSI g`, CSI cursor movement/positioning including next/previous line `CSI E/F`, vertical absolute/relative/backward positioning `CSI d/e/k`, horizontal absolute/relative positioning `CSI n \`` / `CSI n a`, insert-character, delete-character, insert-line/delete-line, scroll-up/scroll-down, erase-character, erase-line/display including `CSI 2K` cursor preservation, normal insert/replace mode via `CSI 4h` / `CSI 4l`, SGR foreground/background/bold/inverse plus 256-color and truecolor metadata retention through protocol `TerminalStyledLine` spans and dock colored-run rendering, OSC control-string swallowing for BEL and ST terminators, DCS/PM/APC/SOS ST-terminated control-string swallowing, `ESC #` intermediate swallowing, split UTF-8/CSI/OSC/private-mode/control-string input, cursor save/restore via `ESC 7`/`ESC 8`, CSI `s`/`u`, normal Device Status Report responses for `CSI 5n`, `CSI 6n`, and DEC private `CSI ?6n`, xterm text-area size reports for `CSI 18t`, plus primary/secondary Device Attributes responses for `CSI c`, DECID `ESC Z`, and `CSI > c` from each active session's own screen state, and private `CSI ?1048h/l`, alternate-screen enter/restore via `CSI ?47h/l`, `CSI ?1047h/l`, and `CSI ?1049h/l`, bracketed paste mode via `CSI ?2004h/l` with paste bytes wrapped in `CSI 200~` / `CSI 201~`, xterm focus-event reporting via `CSI ?1004h/l` plus focus-in/focus-out bytes on GUI window focus changes while attached, classic/default X10 mouse reports for click/release/wheel and button-drag input when sessions request mouse reporting without extended coordinates, UTF-8 coordinate mouse reports for click/release/wheel and button-drag input when sessions request `CSI ?1005h`, URXVT mouse reports for click/release/wheel and button-drag input when sessions request `CSI ?1015h`, xterm SGR mouse reports for click/release/wheel input plus `button_event` and `any_event` motion modes while attached sessions request SGR mouse reporting, bounded scroll-region linefeeds via `CSI <top>;<bottom> r`, `ESC M` Reverse Index with scroll-region top-margin behavior, PTY-width autowrap including pending-wrap carriage-return cancellation plus scroll-region bottom wrapping, DEC origin mode `CSI ?6h/l`, DEC autowrap `CSI ?7h/l`, DEC application cursor-key mode `CSI ?1h/l` with SS3 arrow-key routing while active, DEC application keypad mode `ESC =` / `ESC >` with SS3 physical-numpad routing while active, xterm navigation/edit/function-key routing for `Insert`, `Delete`, `PageUp`, `PageDown`, and `F1`-`F12` while preserving Shift-navigation as GUI scrollback controls, and combined DEC/xterm private-mode application such as `CSI ?6;7h/l`. | Continue toward remaining VT screen-grid behavior such as additional control sequences and richer persistent session-history UX. |
| GUI/agent context integration | Partial | Terminal context files, session activity, agent launch hints, production command handoffs, and context refresh exist. The visible `AGENTS` entry point routes to terminal-agent prefill, terminal activity selection now stays in the PTY terminal lane instead of activating the legacy assistant transcript, active terminal tab close refreshes the surviving session's discovery alias, and runtime context refresh updates the in-memory `TerminalLaunchContext` so new/restarted PTY sessions inherit the latest selection/cursor/dock state. Terminal context snapshots now project GUI production state into top-level agent-readable fields for visible artifact IDs, output job IDs, artifact file paths, latest output-job/run/artifact IDs, focused artifact/file, resolver source-shard health, protocol-backed `terminal_sessions` with active tab/session, geometry, attachment, restart lineage, event-log path, and activity metadata, plus `active_context_commands` that bind concrete catalog-rendered artifact/check/ZoneFill commands for the current focused context while retaining the full `production_status` payload; GUI-written terminal discovery files and lifecycle rewrites now use same-directory atomic temp-file writes with temp-file sync plus rename, and runtime context refresh failures are surfaced in the terminal lane instead of silently leaving stale agent context. CLI/MCP `datum.context.get|refresh` normalization now carries the same top-level field shape with resolver-derived production/source-shard visibility where project evidence exists. The former embedded assistant bridge subprocess/module/script has been removed. Compatibility `DockTab::Assistant` now renders the terminal lane and has no editable assistant transcript surface. The dormant protocol-only `AssistantMessage`, `AssistantLaneState`, and `WorkspaceUiState::assistant` payload were removed, leaving the terminal lane as the only protocol-backed bottom-dock command surface. `scripts/check_gui_agent_terminal_convergence.py` gates the terminal-launcher path, context-refresh behavior, stale launch-context prevention, retired bridge artifacts/runtime markers, assistant-lane rendering/input retirement, and protocol assistant-state retirement. | Broaden GUI action coverage while keeping agent entry points terminal-owned. |
| Spec parity/governance | Partial | Drift gates and private-writer/spec parity gates pass. Schematic and rules/checks domain contracts now align with current journaled schematic authoring, waiver/deviation, persisted CheckRun, standards repair proposal, and canonical MCP taxonomy behavior. `scripts/check_resolver_raw_loads.py` is now wired into the full drift gate and locks zero CLI raw `load_native_project(root)?` exceptions so new resolver bypasses cannot land silently. `scripts/run_migration_proof_gates.sh` now runs 30 named PG proof groups plus PG-HARNESS-WIRING and is invoked from the full drift gate, covering identity substrate import paths, resolver recovery, source-shard terminal context, source-shard GUI summary, journal durability/undo, revision guards, ComponentInstance, library authoring, proposal schema/replay, CheckRun, ZoneFill, production policy, OutputJob replay, standards repair apply, resolver-backed validation, CAM equivalence, panel isolation, variants, and artifact traceability. Hidden compatibility alias retirement is now machine-readable for every hidden alias; legacy session, check, check/explain, journal, artifact/output, ComponentInstance, pool lookup, read-only query, replacement-planning, manufacturing-authoring, OutputJob, route, proposal, PCB, and library families have explicit `deprecated` status and alias-specific criteria enforced by `scripts/check_mcp_public_taxonomy.py` and `mcp-server/test_protocol_catalog.py`. | Continue reconciling remaining domain contracts as implementation slices land, then graduate deprecated aliases to `scheduled_for_removal` when client migration evidence exists. |

## Immediate Closure Queue

Recent progress: missing required authored board-root replay is now
diagnostic instead of opaque. If `board/board.json` is removed after a
journaled board-root edit such as `SetBoardName`, `project query
resolve-debug` reports both `missing_required_shard` and
`journal_replay_failed` diagnostics rather than failing with a raw
`domain_object ... not found` error. Datum does not fake board-root recovery
from partial operations; it quarantines the invalid prefix until a durable
root snapshot/recovery mechanism exists. `PG-RESOLVER-RECOVERY` now runs the
full `main_tests_project_source_shard` regression filter so this split
board-root replay case is part of the named migration proof harness.

Recent progress: authored production source shards now have public `Unknown`
dirty-state coverage. Focused `project query <root> resolve-debug` coverage
replaces journal-recovered ManufacturingPlan, PanelProjection, and OutputJob
promoted files with directories and proves each shard preserves path, kind,
concrete taxon, `AuthoredDesign` authority, and `Unknown` dirty-state metadata.

Recent progress: authored identity/intent source shards now have public
`Unknown` dirty-state coverage. ComponentInstance, Relationship, and
VariantOverlay journal replay skips unreadable promoted files and rebuilds the
materialized shard from journal operations, with `resolve-debug` coverage
proving concrete taxon, `AuthoredDesign` authority, and `Unknown` dirty-state
metadata remain visible.

Recent progress: ImportMap sidecars now have public `Unknown` dirty-state
coverage. ImportMap journal replay skips unreadable promoted sidecars and
rebuilds the materialized sidecar from journal operations, with `resolve-debug`
coverage proving `SidecarMetadata` authority, concrete `ImportMap` taxon, and
`Unknown` dirty-state metadata remain visible.

Recent progress: KiCad schematic ImportMap project import now covers
source-backed root and child-sheet schematic primitives plus generated
hierarchy records.
`project import-kicad-schematic` creates native sheets when needed, journals
supported symbols, wires, junctions, labels, hierarchical ports, buses, bus
entries, no-connect markers, sheet definitions, sheet instances, schematic
text, and bounded drawing primitives, persists one ImportMap sidecar for
source-backed symbols, wires, junctions, labels, buses, bus entries,
no-connect markers, text, drawing, sheet definition, sheet instance, and
deterministic sheet-port identities, and reuses journal-recovered mappings on
reimport without duplicate schematic object creation.

Recent progress: Eagle `.lbr` library import now has a native project-root
surface. `project import-eagle-library` journals imported pool units, symbols,
entities, parts, packages, and padstacks through generic pool-library object
operations, persists one ImportMap sidecar, verifies resolver visibility after
commit, reuses journal-recovered mappings on reimport without duplicate pool
objects, and undo removes both the pool objects and import-map sidecar.

Recent progress: project pool validation now checks standalone
`pin_pad_maps.mappings` entries, not only the `pin_pad_maps[*].part` envelope.
For each mapping, `project validate` verifies the logical-pin key resolves to a
pin on one of the referenced part entity's gate units and the mapped value
resolves to a pad on the referenced part package.

Recent progress: pool/library authored shards now have public `Unknown`
dirty-state coverage. Focused `resolve-debug` coverage replaces a
journal-recovered `pool/symbols/<uuid>.json` file with a directory and proves
the recovered shard preserves `AuthoredDesign` authority, concrete `PoolSymbol`
taxon, and `Unknown` dirty-state metadata. Pool directory discovery and
materialized object refresh now skip unreadable promoted pool entries instead
of aborting before journal replay can rebuild the shard, and MCP
`datum.query.source_shards` parity now proves both `Missing` and `Unknown`
dirty-state reporting for journal-recovered pool symbols.

Recent progress: board-component pool materialization now reads project-local
package and padstack shards through `ProjectResolver` before falling back to
existing promoted files for fixture/external-pool compatibility. Focused CLI
coverage creates a package and padstack through journaled typed pool commands,
removes the promoted package/padstack JSON files, places a board component, and
proves the persisted pad shape, drill, and aperture dimensions came from
journal replay.

Recent progress: schematic authored sheets and definitions now have public
`Unknown` dirty-state coverage. Referenced schematic shard discovery and
ComponentInstance join-key derivation skip unreadable promoted sheet files
instead of aborting, while schematic definition replay now rebuilds missing or
unreadable promoted definition shards from the journal and suppresses stale
promoted files after journaled deletes. Focused `resolve-debug` coverage
replaces journal-created `schematic/sheets/<uuid>.json` and
`schematic/definitions/<uuid>.json` files with directories and proves the
recovered shards preserve `SchematicSheet` / `SchematicDefinition` kind,
`AuthoredDesign` authority, and `Unknown` dirty-state metadata. Native MCP
parity now also proves canonical `datum.query.source_shards` preserves the
same `SchematicDefinition` / `AuthoredDesign` / `Unknown` row when the
promoted definition path is unreadable.

Recent progress: forward-annotation review sidecars now have public `Unknown`
dirty-state and materialization coverage. Replay skips unreadable promoted
`.datum/forward_annotation_review/review.json` files, materialized source-shard
access reconstructs the review from the journal, and focused `resolve-debug`
coverage proves the recovered shard preserves `SidecarMetadata` authority,
concrete `ForwardAnnotationReview` taxon, and `Unknown` dirty-state metadata.

Recent progress: ProposalMetadata sidecars now replay from the journal like
other sidecar families. Resolver replay reconstructs missing or unreadable
`.datum/proposals/<uuid>.json` shards, applies journal create/set/delete
operations to the resolved proposal map, marks stale promoted proposal files
`Dirty`, suppresses stale files after journaled deletes, and lets materialized
source-shard access reconstruct recovered proposal metadata. Engine coverage
proves `Missing`, `Dirty`, `Unknown`, and delete-suppression cases; public
`resolve-debug` coverage proves the recovered shard preserves
`SidecarMetadata` authority, concrete `ProposalMetadata` taxon, and `Unknown`
dirty-state metadata.

Recent progress: generated-evidence replay now tolerates unreadable promoted
files across later journal transactions. OutputJobRun, ArtifactRun, CheckRun,
ArtifactMetadata, and ZoneFill replay skip unreadable promoted values and let
journal operations rebuild materialized source shards; focused public
`resolve-debug` coverage proves a journal-recovered ArtifactMetadata directory
remains `GeneratedEvidence` / `ArtifactMetadata` / `Unknown` after a subsequent
journaled project-name transaction.

Recent progress: canonical MCP `datum.check.explain_violation` is now public.
It dispatches to the existing fingerprint-capable `explain_violation`
implementation, focused MCP tests cover both fingerprint and legacy index
calls, and hidden flat `explain_violation` now points to the public canonical
alias instead of a pending replacement. The taxonomy guard now locks 308 public
tools, 517 registered tools, and 209 hidden compatibility aliases.

Recent progress: PG-HARNESS-WIRING is now executable. The new
`scripts/run_migration_proof_gates.sh` aggregates focused engine and CLI
regressions under the ten named PG gates from product mechanics 000D, and the
full drift gate now invokes it before size/decomposition checks. Direct harness
execution passed on 2026-06-26.

Recent progress: the production proof harness now covers the previously weak
substrate evidence rows directly. `scripts/run_migration_proof_gates.sh` adds
focused PG groups for revision guards, ComponentInstance substrate and proposal
policy, proposal schema/replay, CheckRun generated evidence and CLI lifecycle,
ZoneFill substrate and CLI solver behavior, production schema/policy,
OutputJobRun engine/CLI replay, standards repair proposal apply, and
resolver-backed validation. Direct harness execution and the full drift gate
passed on 2026-06-27.

Recent progress: proposal preview/apply parity is now test-backed. The CLI
preview path uses a journal-aware dry-run that stages shard writes, updates
preview source hashes, cleans the stage directory, and leaves promoted shards
untouched; focused engine coverage asserts preview after-revision, deterministic
predicted transaction UUID, and accepted apply transaction UUID all match.

Recent progress: promoted proposal metadata discovery now enforces the shared
source-shard schema-version guard. `.datum/proposals/*.json` sidecars with
unsupported future `schema_version` values are rejected as
`invalid_proposal_metadata` before proposal payloads enter the model.

Recent progress: proposal payload schema is now explicit. New proposals
serialize `schema_version: 1`; legacy promoted proposal metadata missing the
field still resolve as version 1; and unsupported future proposal payload
versions are rejected at resolver/model materialization boundaries before a
proposal can enter `DesignModel.proposals`.

Recent progress: relationship/variant shard payload envelopes are now explicit.
Journal staging uses `RELATIONSHIP_SHARD_SCHEMA_VERSION` and
`VARIANT_OVERLAY_SHARD_SCHEMA_VERSION`; legacy promoted relationship/variant
shards without `schema_version` still resolve as version 1; and focused
relationship regressions prove those legacy shards materialize while future
source-shard versions remain quarantined.

Recent progress: ComponentInstance shard payload envelopes are now explicit.
Journal staging uses `COMPONENT_INSTANCE_SHARD_SCHEMA_VERSION`; legacy promoted
ComponentInstance shards without `schema_version` still resolve as version 1;
and focused schema regressions prove those legacy shards materialize while
future source-shard versions remain quarantined.

Recent progress: terminal discovery context now exposes missing
identity/relationship source-shard attention rows. `datum-eda context refresh`
projects missing journal-recovered ComponentInstance, Relationship, and
VariantOverlay sidecars into `source_shard_status.attention` with relative path,
snake_case kind/taxon, `authored_design` authority, and `missing` dirty state,
so terminal-launched agents see the same resolver health signal as the GUI
source-shard summary.

Recent progress: automated cross-domain identity/intent writes now require the
proposal path. The commit gateway rejects `Tool`/`Assistant` provenance batches
that directly create, set, or delete ComponentInstance, Relationship, or
VariantOverlay shards with
`proposal_required_for_automated_cross_domain_identity_operation`, so
schematic/board/part bindings, design-intent relationships, and variant overlays
cannot bypass reviewable proposal semantics when authored by automation.

Recent progress: ImportMap shard payload envelopes are now explicit.
`ImportMapShard` exposes `IMPORT_MAP_SHARD_SCHEMA_VERSION`; legacy promoted
ImportMap sidecars without `schema_version` still resolve as version 1; and
focused schema regressions prove those legacy sidecars materialize while future
source-shard versions remain quarantined.

Recent progress: ImportMap replay authority is now test-backed for both create
and delete. Resolver journal replay applies `CreateImportMapShard` /
`DeleteImportMapShard` to the source-shard list and import-key table. Focused
coverage removes a promoted sidecar after journaled create and proves
`model.import_map` still recovers from the journal, while stale-promoted delete
coverage proves a deleted sidecar does not reappear in `model.import_map` or
`source_shards`.

Recent progress: KiCad board ImportMap identity reuse now covers board
footprints, pads, routed segments, vias, and zones.
`board_footprint_import_key`, `board_pad_import_key`,
`board_segment_import_key`, `board_via_import_key`, and
`board_zone_import_key` are stable key contracts. Import-map mode reuses mapped
Datum identities when present and otherwise allocates deterministic non-source
UUIDs while legacy import preserves KiCad source UUIDs. Pad fallback identities
derive from the source footprint UUID and pad name rather than the mapped package
UUID. Project-root `datum-eda project import-kicad-board` now journals imported
native board packages, pads, tracks, vias, zones, required nets, and one
resolver-validated ImportMap sidecar for those supported board object families.
Re-import coverage proves journal-recovered ImportMap entries are reused without
duplicate board-object creation, and undo removes the promoted board ImportMap
sidecar plus created board objects.
derive from the source footprint UUID and pad name rather than the mapped
package UUID, so pad identity stays stable across package remaps. Focused
engine coverage proves mapped reuse and deterministic allocation for each
family.

Recent progress: acceptance-path lineage is now ratified to match the
implemented substrate instead of requiring a duplicate journal field. Direct
commits are classified by `CommitProvenance.source`; proposal-mediated
acceptance is proven by an applied `Proposal` whose `applied_transaction_id`
equals the durable `TransactionRecord.transaction_id`, with `Proposal.source`
preserving proposer origin and CLI/MCP `approval_path` remaining a surface
contract. The stale open-gap language was removed from the conformance audit
and product mechanics docs.

Recent progress: the MCP/daemon hidden write-island drift risk is now
machine-fenced. `scripts/check_daemon_write_parity.py` reconstructs the Rust
daemon non-journaled write arms by parsing `api/write_ops` mutators and
`engine-daemon/src/dispatch.rs`, diffs them against
`NON_JOURNALED_DAEMON_WRITE_METHODS`, and is invoked by the full drift gate.
This does not route the legacy daemon arms through `commit_journaled`, but it
prevents a new hidden/public mismatch from landing silently.

Recent progress: replacement apply MCP aliases no longer appear in public
`tools/list` while they dispatch to non-journaled daemon write arms. The flat
legacy methods and canonical `datum.replacement.apply_*` aliases remain
registered as hidden compatibility tools, all 19 non-journaled daemon write
arms are fenced by `NON_JOURNALED_DAEMON_WRITE_METHODS`, and replacement
read/planning aliases stay public.

Recent progress: hidden MCP compatibility aliases now carry machine-readable
replacement paths. `tools_catalog_data.py` annotates every hidden alias with
non-empty `x_canonical_replacements`, and both `check_mcp_public_taxonomy.py`
and `test_protocol_catalog.py` reject replacements that are neither public
catalog tools nor explicit `pending:<datum.* surface>` migration targets. This
makes alias retirement auditable instead of prose-only.

Recent progress: artifact/output compatibility aliases are now the second
explicitly deprecated hidden alias cohort after check/journal. Legacy
`generate_artifacts`, artifact query/drill-down aliases, output-job run
lifecycle aliases, and manufacturing-set export/validation aliases all carry
alias-specific criteria pointing to their public `datum.artifact.*`
replacements. The taxonomy gate and protocol catalog tests now fail if this
cohort regresses to the blanket retained-until-migration-plan criteria.

Recent progress: ComponentInstance and pool lookup compatibility aliases now
carry explicit deprecation criteria. Legacy `bind_component_instance`,
`set_component_instance`, `delete_component_instance`, `search_pool`,
`get_part`, and `get_package` are mapped to public
`datum.component_instance.*` and `datum.pool.*` replacements, and the taxonomy
gate plus protocol catalog tests fail if they fall back to generic retained
criteria.

Recent progress: the remaining check compatibility aliases now carry explicit
deprecation criteria. Legacy check-run list/show/profile aliases, `fill_zones`,
standards repair generation, fingerprint waiver, and deviation aliases are
mapped to public `datum.check.*` replacements, and the taxonomy/protocol gates
fail if they fall back to generic retained criteria.

Recent progress: read-only query compatibility aliases now carry explicit
deprecation criteria. Legacy board/schematic/provenance/production query names
such as `get_components`, `get_netlist`, `get_symbols`, `get_import_map`,
`get_zone_fills`, and `get_hierarchy` are mapped to public `datum.query.*`
replacements, and the taxonomy/protocol gates fail if they fall back to generic
retained criteria.

Recent progress: replacement-planning compatibility aliases now carry explicit
deprecation criteria. Legacy candidate and plan read/edit names map to public
`datum.replacement.package_candidates`, `datum.replacement.part_candidates`,
`datum.replacement.get_plan`, `datum.replacement.get_scoped_plan`, and
`datum.replacement.edit_scoped_plan`, while replacement apply aliases remain
hidden pending journal/proposal-backed mutation semantics.

Recent progress: manufacturing authoring compatibility aliases now carry
explicit deprecation criteria. Legacy panel-projection and manufacturing-plan
create/update/delete names map to public proposal-mediated
`datum.manufacturing.*` aliases, and legacy `*_proposal` names map directly to
their `datum.proposal.*` producers. The taxonomy and protocol catalog gates
fail if this family falls back to generic retained criteria.

Recent progress: OutputJob compatibility aliases now carry explicit
deprecation criteria. Legacy create/update/delete and Gerber-set names map to
proposal-mediated `datum.output_job.*`, legacy `*_proposal` names map directly
to their `datum.proposal.*` producers, and `run_output_job` maps to the
execution-class `datum.output_job.run` surface rather than a proposal write
alias.

Recent progress: route compatibility aliases now carry explicit deprecation
criteria. Legacy route proposal, strategy, batch-evaluation, artifact, fixture,
and apply names map to public `datum.route.*` replacements; public route apply
aliases remain locked as journal/proposal-backed write surfaces, while route
fixture generators remain explicitly generated-fixture-only.

Recent progress: proposal compatibility aliases now carry explicit deprecation
criteria. Legacy proposal create/show/preview/validate/review/apply names,
board replacement aliases, replacement apply aliases, and rule-proposal aliases
map to classified public `datum.proposal.*` replacements, including
proposal-metadata write, review-state write, and apply-gateway surfaces.

Recent progress: PCB compatibility aliases now carry explicit deprecation
criteria. Legacy flat board mutation names and hidden `datum.board.*` bridge
aliases map to public journaled `datum.pcb.*` replacements, while the taxonomy
gate continues to reject any public alias that dispatches to a fenced
non-journaled daemon write arm.

Recent progress: library compatibility aliases now carry explicit deprecation
criteria. Legacy pool-library lookup, typed unit/symbol/entity/package/part
authoring, model-blob, and pad-map names map to public `datum.library.*`
replacements; automated library writes remain governed by proposal policy where
the commit gateway requires reviewable proposals.

Recent progress: hidden flat session compatibility aliases now carry explicit
deprecation criteria. `open_project`, `close_project`, `save`, and
`validate_project` map to public `datum.session.*` replacements; public
`datum.session.save` remains compatibility-only because target authored writes
commit at apply time and Datum should not grow a new public save mutation for
authored operations.

Recent progress: the first native replacement proposal path now exists without
promoting those hidden apply aliases. `datum-eda proposal
create-board-component-replacement` builds a guarded `OperationBatch` for one
board component using `SetBoardPackagePart`, `SetBoardPackagePackage`, and
`SetBoardPackageValue`, including the same package materialization payloads used
by direct journaled component-package edits. Focused CLI coverage proves proposal
creation is non-mutating, then `proposal accept-apply` applies the replacement
through the proposal gateway and marks the proposal `Applied`. MCP now exposes
that same proposal-first path as
`datum.proposal.create_board_component_replacement`, while direct replacement
apply aliases stay hidden.

Recent progress: batch board-component replacement proposals now use the same
proposal-first path. `datum-eda proposal
create-board-component-replacements` accepts repeated replacement specs and
builds one guarded draft proposal for multiple package/part/value replacements
without mutating board state. Focused CLI coverage proves both components stay
unchanged until `proposal accept-apply`, then both replacements apply through
the proposal gateway. MCP exposes the same producer as flat
`create_board_component_replacements_proposal` and canonical
`datum.proposal.create_board_component_replacements`, with public write-surface
classification locked by taxonomy/catalog guards.

Recent progress: replacement-plan shaped board-component proposals now bridge
legacy selection payloads into the same proposal-first path. `datum-eda
proposal create-board-component-replacement-plan` accepts repeated selections
with `uuid`, optional `package_uuid`, optional `part_uuid`, and optional
`value`, maps them into guarded board component replacement operations, and
does not mutate board state until accepted/applied. MCP exposes the same
producer as flat `create_board_component_replacement_plan_proposal` and
canonical `datum.proposal.create_board_component_replacement_plan`; the old
direct apply-plan shape remains hidden until its mutation path is migrated.

Recent progress: SPEC_PARITY machine visibility now covers two previously
unguarded shipped slices. `standards_check_surface` freezes standards-aware
CheckFinding identity fields, standards finding codes, standards repair/check
CLI/MCP markers, zone-fill query exposure, and `SetCheckRun`.
`pool_library_surface` freezes pool/library Operation variants, ProjectCommands
variants, and public `datum.library.*` tools. `check_spec_parity.py` now passes
with 9 inventories.

Recent progress: board routing/net, netclass/dimension, and layout mutation
validation plus post-commit readbacks now use resolver-materialized board
state, and focused board-net, board-netclass, board-dimension, board-text, and
board-keepout coverage proves journal-created board objects remain queryable
and editable after promoted `board.json` is restored to stale pre-object
snapshots.

Recent progress: native schematic query/mutation/proposal entrypoints now load
resolver-materialized project and schematic roots before materialized sheet
helpers run, closing promoted-root bypasses across connectivity, text/drawing,
symbol, and schematic read/proposal surfaces. Pool replay now scopes
`SetPoolLibraryObject` and pool model attachment operations to the target
relative path, preventing later symbol edits from corrupting replayed unit
shards.

Recent progress: schematic wire, junction, and no-connect query surfaces now
read resolver-materialized sheet shards instead of promoted sheet JSON. Focused
regressions restore stale promoted sheet files after journaled create operations
and prove those connectivity objects remain queryable before deletion.
Schematic hierarchy/net/diagnostic construction now also materializes sheet
definitions through `ProjectResolver` instead of reading promoted definition
files directly; focused hierarchy coverage removes a promoted journal-created
definition file and still resolves the sheet-instance port link from the
journal.

Recent progress: native `project validate` now validates resolver-materialized
project, schematic, board, rules, sheet, and definition JSON shards instead of
promoted files. Focused validation coverage restores stale promoted schematic
root, sheet, and board files after journaled sheet/wire/board-text edits and
still reports a clean valid project, while pool model blob hash checks remain
filesystem-based generated/blob validation.

Recent progress: pool reference queries and pool library authoring entrypoints
now load resolver-materialized project manifests before inspecting pool refs.
Focused coverage restores stale pre-pool `project.json`, creates another typed
pool unit, proves no duplicate `AddProjectPoolRef` is journaled, and verifies
pool queries read the materialized pool reference. Pool-library mutation
preimage reads now also materialize project-owned pool objects through
`ProjectResolver` instead of promoted JSON reads, while external `--from-json`
inputs remain raw file reads. Focused coverage removes a promoted
journal-created `pool/parts/<uuid>.json` file, edits part metadata, and proves
the mutation uses the journal-materialized previous object.

Recent progress: route proposal export/apply and default-top-stackup authoring
now load resolver-materialized project/board state instead of stale promoted
roots. Mutating route proposal artifacts still require embedded accepted
proposal metadata, while verified no-op policy artifacts without draw-track
actions apply as zero-action artifacts. Focused route proposal, route apply,
board stackup, format, check, and full drift gates pass.

Recent progress: schematic sheet/root mutations now load resolver-materialized
schematic root state before create/delete/rename sheet, create definition, and
create/move/delete/bind/unbind sheet-instance operations. Resolver replay now
recovers journal-created schematic definition shards when a stale promoted
schematic root omits their root-map entries, matching the existing sheet-shard
recovery behavior, and definition shard replay now covers missing/unreadable
promoted files plus stale-promoted delete suppression. Focused sheet CLI tests,
schematic-definition replay tests, format, check, and full drift gates pass.

Recent progress: granular project-rule create/set/delete post-commit reports
now reload through resolver-materialized rules state. Focused rules coverage
restores stale promoted `rules.json` before set/delete and proves mutation
reports still return resolver-derived rule counts and rules-root revisions.
Rule payloads now carry explicit `object_revision` values: granular create
defaults missing revisions to `0`, granular set bumps the existing rule payload
and rule-domain object revision, and whole-root rules replacement normalizes
missing rule payload revisions to `0`.
Focused rules tests, format, check, and full drift gates pass.

Recent progress: journaled raw pool-library footprint create/set now validates
`pool/footprints` payloads through the current canonical package-compatible
geometry schema instead of accepting identity/path-only JSON. Malformed
footprint geometry is rejected before staging, while the existing pool-library
query fixture now emits package-compatible footprint payloads.

Recent progress: native project-rule payloads now have a shared substrate
validator for the persisted `rule_type` / `params` schema. Granular
create/set and whole-root rules replacement reject malformed rules before
journal staging, and `project validate` reports the same native rule schema
errors from resolver-materialized `rules/rules.json` instead of trying to parse
the legacy Rust `Rule` serde shape.

Recent progress: forward-annotation artifact export/selection/compare/filter/
apply and artifact-review import/replace now load resolver-materialized project
identity/state, and native project inspect now reports resolver-materialized
schematic/board/rules state. Focused coverage proves forward-annotation export
uses a journaled project name after stale promoted `project.json`, and inspect
reports a journal-created board pad after stale promoted `board.json`.
Forward-annotation artifact tests, inspect tests, format, check, and full drift
gates pass.

Recent progress: BOM/PnP export, validation, comparison, and panel-PnP report
surfaces now load resolver-materialized project/board state for report metadata
while row generation continues through the resolver-backed inventory model.
Focused BOM/PnP export coverage restores stale promoted `board.json` after
journaled component placement and proves exported rows plus report counts come
from materialized board state. BOM/PnP test groups, format, check, and full
drift gates pass.

Recent progress: manufacturing-set inspect, validate, and compare wrappers now
have stale-promoted-board regression coverage. The focused tests journal
board edits, restore stale promoted `board.json`, and prove wrapper metadata,
expected artifact sets, validation, and comparison outcomes are driven by
resolver-materialized board state.

Recent progress: production-object create idempotency now resolves journaled
state instead of promoted shard existence. Focused regressions remove promoted
ManufacturingPlan, PanelProjection, and OutputJob shards after their first
journaled create, rerun the same create command, and prove each path returns
the resolver-replayed object with `created: false` without appending a duplicate
journal transaction.

Recent progress: production authored-object semantic validation now runs at the
engine operation boundary. PanelProjection board instances must target the
project board, ManufacturingPlan and OutputJob board/panel targets must resolve
to the project board or an existing/current panel, optional variants must exist,
and OutputJobs cannot reference missing ManufacturingPlans. Promoted production
shard reads also reject filename/payload UUID mismatches with resolver
diagnostics and exclude the bad shard from the model.

Recent progress: artifact/check-run/ZoneFill generated-evidence persistence
helpers are no longer public substrate exports. `persist_artifact_metadata`,
`persist_output_job_run`, `persist_artifact_run`, `persist_check_run`, and
`persist_zone_fill` are internal `pub(super)` helpers, and the private-writer
guard now forbids those helper names in `substrate/mod.rs` while requiring
internal visibility. The former CLI filled-zone fixture now seeds evidence
through the real `project fill-zones` journaled command path, and the guard
rejects direct `persist_zone_fill` in CLI command/test files. Focused artifact
metadata/replay, ZoneFill DRC/fill/replay tests, format, check, private-writer
guard, and full drift gates pass.

Recent progress: artifact-only CLI generation now proves generated-evidence
journal recovery at the product boundary. The focused regression removes
promoted `.datum/artifacts/<id>.json` and `.datum/artifact_runs/<run>.json`
sidecars after `datum-eda artifact generate`, then verifies `ProjectResolver`
replays both `ArtifactMetadata` and `ArtifactRun` evidence from the journal
without mutating the authored `ModelRevision`.

Recent progress: resolver raw-load governance is now machine-checked and
zero-exception. The full drift gate runs
`scripts/check_resolver_raw_loads.py`, which fails on any CLI
`load_native_project(root)?` call site. The core resolver helper now seeds
`LoadedNativeProject` from resolver-materialized manifest, schematic, board, and
rules shards rather than loading promoted roots first. The KiCad footprint import
bootstrap exception was retired by loading pool refs from the
resolver-materialized project manifest; focused stale-promoted `project.json`
coverage proves a second import does not journal a duplicate `AddProjectPoolRef`.
The forward-annotation review fallback exception was retired by loading embedded
legacy review state from the resolver-materialized project manifest when no
review sidecar exists, with compatibility coverage proving the fallback remains
readable without creating a sidecar. Focused guard execution and the full drift
gate pass.

Recent progress: GUI agent entry points converged further onto the PTY
terminal. Terminal activity summary selection now focuses the terminal lane and
records a terminal-visible note instead of writing to the assistant transcript.
Closing the active terminal tab refreshes the surviving session context alias so
`.datum/gui-terminal-context.json` follows the active terminal session used by
terminal-launched agents. A new GUI convergence drift gate locks the AGENTS
terminal-prefill path, terminal activity focus behavior, and terminal-close
context refresh.

Recent progress: GUI and CLI terminal context envelopes now expose the latest
generated-evidence navigation pointers directly. GUI-written discovery files
carry `latest_artifact_id`, `latest_artifact_run_id`, and `latest_check_run_id`
from `ProductionStatus` / `CheckRunReviewState`; CLI `context get|refresh`
recomputes those same fields from `ProjectResolver`. Both GUI-written context
and CLI context refresh now also carry `latest_profile_id` and
`profile_latest_check_runs[]`, keeping terminal-launched agents aligned with
`datum-eda artifact list` and `datum-eda check list` without duplicating sorting
rules. GUI production visibility also prefers canonical
`latest_output_job_run_id` over the legacy aggregate `latest_run_id` fallback
when output-job row summaries are absent, and reconstructs latest
output-job/artifact IDs from matching `artifact_runs[]` evidence. GUI
production refresh now also uses canonical `latest_artifact_id` and
`latest_output_job_run_id` before output-job row-order fallbacks when selecting
the focused artifact drill-down, so the OUTPUTS dock follows current generated
evidence instead of the first artifact listed under an older job.
`datum-eda artifact list` latest artifact selection now uses explicit
generated-evidence ordering by artifact `model_revision` plus UUID tie-break
instead of resolver map order; focused coverage proves a newer low-UUID
artifact is selected over an older high-UUID artifact.

Recent progress: GUI-written terminal context `active_context_commands` now
include artifact/proposal discovery actions, check discovery/run actions,
focused artifact actions, OutputJob actions, and selected CheckFinding actions,
plus latest proposal review/apply actions and journal actions. Artifact and
proposal discovery expose always-present catalog-rendered
`datum-eda artifact list` and `datum-eda proposal list` commands when a project
root is known. Focused artifacts expose catalog-rendered
show/files/preview/validate commands through GUI and CLI context refresh
coverage, and contexts with distinct previous/latest artifact IDs expose
catalog-rendered `datum-eda artifact compare --before <previous> --after
<latest>` so terminal-launched agents can diff generated evidence without
rescanning artifact history. Latest OutputJob context now exposes
generate/start-run commands, and latest OutputJobRun context exposes cancel-run
commands. Terminal contexts now also expose `visible_proposal_ids`,
`latest_proposal_id`, and catalog-rendered proposal
show/preview/validate/defer/reject/review/accept-apply/apply commands for the
selected or latest draft/accepted/deferred proposal. Journal active commands now
include list/undo/redo everywhere and show-tip when resolver context has an
`accepted_transaction_tip`; GUI-written discovery now loads the resolver-backed
accepted transaction tip directly, so `journal_show_tip` is bound before a CLI
context refresh. Source-shard diagnostics are also first-class in active context:
GUI, CLI refresh, and MCP context envelopes expose `source_shards` as
`datum-eda project query <project-root> resolve-debug`, matching canonical
`datum.query.source_shards` resolver evidence while preserving the existing CLI
diagnostic command. When the GUI selection is a CheckFinding,
terminal-launched agents receive catalog-rendered
`datum-eda check list` history discovery even before a latest check run exists,
always-present `datum-eda check run <project-root>` and
`datum-eda check profiles <project-root>` actions for running and discovering
check profiles, and an always-present
`datum-eda check fill-zones <project-root>` action for refreshing ZoneFill
generated evidence from the same terminal/agent lane, plus focused
`datum-eda check waive` and `datum-eda check accept-deviation` commands with the
finding fingerprint already bound and only the human rationale left as an
explicit placeholder; when no finding is selected those fields remain `null`.
CLI `context get|refresh` now rebuilds the same `active_context_commands` shape
from the persisted discovery envelope, resolver-derived context, and preserved
selection context. MCP `datum.context.get` now has focused regression coverage
proving artifact, OutputJob, proposal, journal, and CheckFinding active commands
survive both the CLI bridge result and the agent-facing `tools/call` target
envelope.

Recent progress: source-shard taxonomy closure removed the remaining hand-built
`taxon=None` refs in resolver/replay paths. ImportMap, ProposalMetadata,
ComponentInstance, Relationship, VariantOverlay, production records, ZoneFill,
and schematic sheet payload refs now derive concrete taxons from the central
source-shard path taxonomy.

Recent progress: source-shard health now has explicit `Unknown` dirty-state
public coverage. Focused CLI regressions replace a journal-recovered
ArtifactMetadata promoted path with an unreadable directory and prove both
`project query <root> resolve-debug` and `datum-eda context refresh` report the
generated-evidence shard in the unknown bucket with path, kind, taxon,
authority, and dirty-state attention fields intact. ZoneFill, OutputJobRun,
ArtifactRun, and CheckRun generated evidence now have the same public
`resolve-debug` coverage: unreadable `.datum/zone_fills/<uuid>.json`,
`.datum/output_job_runs/<uuid>.json`, `.datum/artifact_runs/<uuid>.json`, and
`.datum/check_runs/<uuid>.json` promoted paths still resolve from journal
evidence and report their generated-evidence authority, concrete taxon, and
`Unknown` dirty state.

Recent progress: PTY terminal tabs now support explicit detach without killing
the underlying PTY. The `DETACH` dock control marks the active tab detached,
records lifecycle evidence, keeps process status `running`, rejects raw
keyboard/paste input while detached, and clicking the tab reattaches the same
session instead of spawning a replacement.

Recent progress: the embedded assistant bridge runtime has been retired. The
GUI no longer imports/spawns/polls/writes a hosted assistant subprocess,
`crates/gui-app/src/assistant_bridge.rs` and `scripts/datum_assistant_bridge.py`
were removed. The compatibility `DockTab::Assistant` no longer renders an
assistant transcript or accepts assistant text input; it aliases terminal-lane
rendering while the visible `AGENTS` hit target remains the terminal-agent
launcher. The dock now renders `AGENTS` as an inactive launcher rather than a
second active command lane, and the terminal lane explains that agent launches
prefill `codex`/`claude` commands into the PTY. The GUI convergence drift gate
now rejects reintroduced bridge artifacts, runtime ownership markers, assistant
transcript input, or assistant-lane rendering.

Recent progress: PTY terminal screen parsing now captures OSC `0`/`1`/`2`
shell title updates into `TerminalLaneState.title` and renders the active
PTY-provided session label in the terminal dock header. Unsupported OSC
commands remain swallowed without changing the title. Focused regressions prove
BEL-terminated, ST-terminated, icon-title, and split OSC title updates do not
leak payload bytes into visible terminal rows.

Recent progress: PTY terminal screen parsing now captures OSC `7` current
directory updates from `file://host/path` URIs into
`TerminalLaneState.current_working_directory`. The terminal context JSON exposes
that value as `terminal_sessions.active_working_directory`, and the terminal
dock header renders the same `CWD` value, so users and terminal-launched agents
see the shell's current directory instead of only the original project root.
Unsupported OSC 7 URI schemes remain swallowed without changing the tracked
directory.

Recent progress: PTY terminal screen parsing now preserves bare BEL alerts as
protocol state. Standalone BEL increments `TerminalLaneState.bell_count`
without visible byte leakage, the dock surfaces the current alert count in the
session header, and OSC BEL terminators remain control-string terminators rather
than terminal alerts.

Recent progress: PTY terminal screen parsing now preserves cursor row/column for
full-display erase controls `CSI 2J` and `CSI 3J`. These controls clear visible
rows without acting like terminal reset, and focused regressions prove
subsequent printable bytes land at the preserved cursor address.

Recent progress: the dormant protocol-only assistant state was removed from
`gui-protocol`. `WorkspaceUiState` now exposes the terminal lane without a
parallel `assistant` transcript/input payload, while `DockTab::Assistant`
remains only a compatibility alias rendered through the terminal lane. The GUI
convergence guard now rejects reintroduced `AssistantMessage`,
`AssistantLaneState`, or `assistant: AssistantLaneState` markers.

Recent progress: production authored-shard replay now has a dirty-state oracle.
`journal_replay_recovers_missing_production_shards` removes promoted
ManufacturingPlan, PanelProjection, and OutputJob files, resolves through the
journal, and asserts each recovered `SourceShardRef` keeps the exact production
relative path with `SourceShardDirtyState::Missing`.

Recent progress: authored identity/relationship replay now has the same
dirty-state oracle. ComponentInstance, Relationship, and VariantOverlay replay
coverage removes promoted `.datum` shards, resolves through the journal, and
asserts each recovered `SourceShardRef` keeps the exact identity/intent relative
path with `SourceShardDirtyState::Missing`.

Recent progress: the identity/relationship dirty-state oracle now reaches the
public resolver diagnostic surface. `project query <root> resolve-debug`
coverage removes promoted ComponentInstance, Relationship, and VariantOverlay
shards after journaled creation and asserts each recovered row reports
`AuthoredDesign`, its concrete taxon, and `Missing` dirty state.

Recent progress: GUI source-shard health now sees the same identity/intent
sidecar failures. `SourceShardStatusSummary` coverage removes promoted
ComponentInstance, Relationship, and VariantOverlay shards after journaled
creation and verifies the GUI attention rows preserve relative path, kind,
taxon, authority, and `missing` dirty state.

Recent progress: relationship and variant shard discovery now enforce the
shared source-shard schema-version guard. Promoted `.datum/relationships/*.json`
and `.datum/variants/*.json` shards with unsupported future `schema_version`
values are rejected as `invalid_relationship_shard` or
`invalid_variant_overlay_shard` before authored payloads enter the model.

Recent progress: Import Map sidecar discovery now enforces the shared
source-shard schema-version guard. Promoted `.datum/import_map/*.json` sidecars
with unsupported future `schema_version` values are rejected as
`invalid_import_map` before import-key mappings enter the model.

Recent progress: pool source-shard taxonomy is now enforced at the ownership
boundary. `SourceShardKind::Pool` accepts only the eight concrete pool
subdirectories (`units`, `symbols`, `entities`, `parts`, `packages`,
`footprints`, `padstacks`, and `pin_pad_maps`), rejects unknown `pool/*`
families, and focused source-shard metadata coverage proves each accepted pool
path maps to a concrete `SourceShardTaxon`.

Recent progress: PTY terminal parsing now supports `ESC M` Reverse Index. The
screen model moves the cursor up when away from the active top margin and
scrolls only the active scroll region downward when Reverse Index occurs at the
top margin. Focused regressions lock both behaviors.

Recent progress: PTY terminal parsing now supports CSI next/previous line
controls (`CSI E` / `CSI F`). Focused regressions prove the cursor moves by the
requested/default row count, resets to column zero, and creates destination rows
when moving down past current scrollback.

Recent progress: PTY terminal parsing now supports CSI vertical absolute and
relative positioning (`CSI d` / `CSI e`). Focused regressions prove both controls
preserve the current column while moving to or extending destination rows.

Recent progress: PTY terminal parsing now supports CSI vertical-position
backward (`CSI k`). Focused regressions prove explicit and default counts
preserve the current column while moving upward and saturating at the top row.

Recent progress: PTY terminal parsing now supports CSI horizontal absolute and
relative positioning (`CSI n \`` / `CSI n a`). Focused regressions prove
absolute column addressing overwrites the intended cell and relative horizontal
movement creates intervening blank cells before subsequent printable output.

Recent progress: PTY terminal parsing now supports CSI insert-line/delete-line
(`CSI L` / `CSI M`) operations. Focused regressions prove row shifts are bounded
to the active screen or scroll region and preserve rows outside the active
region.

Recent progress: PTY terminal parsing now supports CSI scroll-up/scroll-down
(`CSI S` / `CSI T`) operations. Focused regressions prove full-screen and
scroll-region bounded shifts clear introduced rows while preserving rows outside
the active region.

Recent progress: PTY terminal parsing now uses default 8-column horizontal tab
stops and supports CSI backtab (`CSI Z`). Focused regressions prove forward tab
spacing, previous-tab-stop overwrite behavior, and saturation at column zero.
The same tab-stop model now backs CSI cursor-forward-tab (`CSI I`) with explicit
repeat counts.

Recent progress: PTY terminal parsing now supports CSI repeat preceding
character (`CSI b`). Focused regressions prove repeat counts, no-op behavior
before any printable character, and terminal-width wrapping through the same
screen writer used by normal PTY printable bytes.

Recent progress: PTY terminal parsing now supports `ESC D` Index and `ESC E`
Next Line. Focused regressions prove Index moves down without resetting column,
Next Line moves down and resets to column zero, and Index at the bottom margin
scrolls only the active scroll region.

Recent progress: PTY terminal parsing now treats ASCII vertical tab (`VT`,
`0x0B`) and form feed (`FF`, `0x0C`) as line-feed controls that preserve the
active column, matching terminal emulator behavior for legacy line-advance
controls without leaking visible replacement bytes.

Recent progress: PTY terminal parsing now supports `ESC c` terminal reset.
Focused regressions prove reset clears visible rows, cursor position, saved
cursor, alternate-screen state, scroll region, pending wrap, and repeat-character
state before subsequent terminal output.

Recent progress: PTY terminal parsing now swallows VT charset designation
sequences such as `ESC ( B` and `ESC ) 0`. Focused regressions prove whole and
split charset designation sequences do not leak designation bytes into visible
terminal rows.

Recent progress: PTY terminal parsing now supports the DEC screen alignment
test (`ESC #8`). The parser fills the visible terminal grid with unstyled `E`
cells using the active protocol row/column geometry, while unsupported
`ESC #` intermediate controls remain non-leaking.

Recent progress: PTY terminal parsing now swallows non-printing ST-terminated
control strings for DCS (`ESC P`), SOS (`ESC X`), PM (`ESC ^`), and APC
(`ESC _`), plus two-byte `ESC #` intermediate sequences. Focused regressions
prove whole and split control strings, split ST terminators, and split `ESC #`
sequences do not leak payload/final bytes into visible terminal rows.

Recent progress: PTY terminal parsing now accepts 8-bit C1 control bytes for
CSI (`0x9b`), OSC (`0x9d`), DCS/SOS/PM/APC control strings
(`0x90`/`0x98`/`0x9e`/`0x9f`), ST (`0x9c`), Index (`0x84`), Next Line
(`0x85`), and Reverse Index (`0x8d`). Focused regressions prove these controls
dispatch like their ESC-prefixed forms, do not leak replacement characters or
payload bytes into terminal rows, and do not break split UTF-8 decoding.

Recent progress: PTY terminal parsing now supports DEC autowrap mode
(`CSI ?7h/l`). Autowrap defaults enabled, `?7l` disables right-margin wrapping
and clamps writes to the last visible column, `?7h` restores existing wrapping
behavior, `CSI b` repeat-character writes inherit the current mode, split
private-mode sequences do not leak bytes, and terminal reset restores autowrap.

Recent progress: PTY terminal parsing now supports DEC origin mode
(`CSI ?6h/l`). With origin mode enabled, cursor home and `CSI H/f` plus
`CSI d` vertical absolute positioning are relative to the active scroll-region
top margin and clamp to the bottom margin; disabling origin mode restores
absolute screen addressing. Split private-mode sequences remain non-leaking.

Recent progress: PTY terminal parsing now tracks xterm focus-event reporting
mode (`CSI ?1004h/l`) in `TerminalLaneState.focus_event_reporting` and surfaces
the active mode in the terminal dock header. Focused regressions prove enable,
disable, split private-mode input, and terminal reset behavior do not leak bytes
into terminal rows.

Recent progress: PTY terminal parsing now tracks xterm mouse reporting
negotiation in protocol state. The parser records normal, button-event, and
any-event modes (`CSI ?1000/1002/1003h/l`) plus UTF-8, SGR, and URXVT coordinate
encodings (`CSI ?1005/1006/1015h/l`) in `TerminalLaneState`, and the dock
surfaces the active mouse mode. Focused regressions prove mode precedence,
encoding precedence, disable/reset behavior, and split private-mode input do
not leak bytes.

Recent progress: PTY terminal session-history UX now has keyboard scrollback
controls. Terminal-focused `Shift+PageUp` / `Shift+PageDown` page through
scrollback, `Shift+Home` jumps to the oldest retained row, and `Shift+End`
returns to the live tail without writing those navigation keys to the PTY. The
terminal dock advertises the scrollback shortcut next to copy/paste.

Recent progress: PTY terminal input now handles Alt/meta character chords like
native terminal emulators. Printable terminal text still writes its UTF-8 bytes,
Alt-modified text writes an ESC-prefixed byte stream, ordinary Ctrl character
chords write ASCII control bytes for shell/readline apps, `Ctrl+Space` writes
NUL, and Ctrl+Alt character chords remain reserved instead of being downgraded
into ambiguous terminal text. Focused `terminal_input` coverage locks normal,
Alt, Unicode Alt, Alt+Space, Ctrl-letter, Ctrl-punctuation, Ctrl+Space, and
Ctrl+Alt cases. `Shift+Tab` now emits xterm reverse-tab (`CSI Z`) instead of a
literal tab, with focused coverage preserving plain Tab behavior. Home and End
now follow the same DEC application cursor-key mode as arrow keys, emitting SS3
`ESC O H/F` while active and normal CSI `ESC [ H/F` otherwise. Shift/Alt/Ctrl
arrow chords now emit xterm modifier-parameter sequences, so terminal apps can
distinguish navigation chords such as Ctrl+Left and Ctrl+Home from plain keys. Modified
Insert/Delete/PageUp/PageDown now use matching xterm tilde modifier sequences
such as Ctrl+PageDown `CSI 6;5~`. Modified `F1`-`F12` now preserve xterm
modifier parameters as well, using CSI forms for modified `F1`-`F4` and tilde
forms for modified `F5`-`F12`.

Recent progress: PTY terminal known-width screen-grid behavior now bounds
`CSI @` insert-character and `CSI P` delete-character to the visible terminal
columns. Insert-character shifts cells right only within the active width and
discards right-margin overflow; delete-character shifts cells left and pads the
right margin with blanks. Known-width horizontal cursor controls now also clamp
to the right margin for `CSI C`, `CSI G`, `CSI \``, `CSI a`, and `CSI H/f`
column addressing instead of allowing cursor positions beyond the visible grid.
PTY resize now also passes row geometry into the screen parser, and known-height
vertical cursor controls clamp to the bottom margin for `CSI B`, `CSI E`,
`CSI d`, `CSI e`, and `CSI H/f` row addressing. Known-height scroll-region setup
(`CSI top;bottom r`) now clamps an oversized bottom margin to the PTY bottom row
instead of growing the screen model beyond the visible grid. Known-width
erase-character (`CSI X`) now also blanks visible cells beyond the current row
text up to the right margin instead of ignoring cells that have not yet been
materialized as string content. Known-width erase-in-line (`CSI K`) now
materializes visible blank cells for modes 0/1/2 across the bounded terminal row
instead of truncating or clearing only the row string backing store.
Known-geometry erase-display (`CSI J`) now materializes visible blank cells
across bounded rows and columns for modes 0/1/2/3 while preserving cursor
position. Known-height line operations (`CSI L`, `CSI M`, `CSI S`, and
`CSI T`) now use the visible PTY screen region when no scroll region is active
instead of deriving the active region only from already-materialized backing
rows. Focused terminal-screen coverage locks these cases while retaining legacy
no-width/no-height parser behavior.

Recent progress: PTY terminal parsing now preserves cursor position for
whole-line erase `CSI 2K` and supports normal insert/replace mode via
`CSI 4h` / `CSI 4l`. Insert mode shares the printable/repeat-character cursor
path, split mode sequences do not leak bytes, terminal reset clears insert mode,
and focused terminal-screen coverage now includes 70 passing tests.

Recent progress: PTY terminal parsing now supports mutable horizontal tab
stops. `ESC H` sets a custom stop, `CSI g` clears the current stop including
default eight-column stops, `CSI 3g` clears all stops until a new stop or reset,
and terminal reset restores default stops. HT, forward-tab, and backtab all use
the shared tab-stop state, with focused coverage raising terminal-screen tests
to 77 passing cases.

Recent progress: PTY terminal styling now retains SGR background color and
inverse-video metadata in `TerminalStyleSpan` alongside foreground and bold.
The dock renders inverse/background spans with a visible color fallback, while
the protocol keeps the richer metadata needed for future true cell-background
rendering.

Recent progress: PTY terminal styling now retains extended SGR color metadata
for 256-color (`38;5;n` / `48;5;n`) and truecolor (`38;2;r;g;b` /
`48;2;r;g;b`) foreground/background spans. Malformed extended-color sequences
are ignored without clearing the active style.

Recent progress: PTY terminal styling now retains SGR dim and conceal metadata
in `TerminalStyleSpan`. Focused regressions prove `CSI 2/8 m` attributes
survive into protocol spans, `CSI 22 m` clears bold and dim together, and
`CSI 28 m` clears conceal without clearing unrelated style metadata.

Recent progress: PTY terminal styling now retains SGR blink metadata in
`TerminalStyleSpan`. Focused regressions prove both slow and rapid blink
controls (`CSI 5/6 m`) survive into protocol spans and `CSI 25 m` resets blink
without clearing unrelated style metadata.

Recent progress: PTY terminal styling now retains SGR overline metadata in
`TerminalStyleSpan`. Focused regressions prove `CSI 53 m` survives into
protocol spans and `CSI 55 m` resets overline without clearing underline,
strikethrough, or unrelated style metadata.

Recent progress: PTY terminal styling now retains SGR italic, underline, and
strikethrough metadata in `TerminalStyleSpan` alongside foreground/background,
bold, and inverse state. Focused regressions prove `CSI 3/4/9 m` attributes
survive into protocol spans and `CSI 23/24/29 m` reset only their matching
attribute.

Recent progress: PTY terminal cursor state is now protocol-visible. The screen
parser projects PTY cursor row/column plus DEC private cursor visibility
(`CSI ?25h/l`) and DECSCUSR cursor shape controls (`CSI Ps SP q`) into
`TerminalLaneState`, and the terminal dock renders block, underline, or bar
cursor glyphs from that protocol state only when visible. Focused regressions
prove cursor visibility toggles and split cursor-shape controls do not leak
escape bytes, terminal reset clears cursor style, and render contracts honor
the visibility/style fields.

Recent progress: PTY terminal parsing now supports legacy DEC private alternate-screen mode `CSI ?47h/l` alongside the existing `CSI ?1047h/l` and `CSI ?1049h/l` paths. Focused regressions prove the legacy mode restores the main buffer and split `?47` / `?1049` private-mode sequences do not leak bytes into visible terminal rows.

Recent progress: PTY terminal status responses now include xterm window-state
and position query handling. `CSI 11t` replies with `ESC [ 1 t` from the active
session screen state, while `CSI 13t` returns a deterministic `0,0` top-left
window position for Datum's embedded terminal surface, preserving visible
terminal rows and buffering split query bytes until the final `t`.

Recent progress: PTY terminal geometry responses now include xterm text-area
pixel and cell-size queries. `CSI 14t` derives deterministic total pixel size
from the protocol-visible row/column grid using stable 16x8 terminal cells, and
`CSI 15t` reports the same dimensions as root/screen pixel size because Datum's
embedded PTY surface does not expose a larger terminal desktop. `CSI 16t`
reports that same cell size directly, so shell applications can query pixel
geometry without requiring a platform font metric dependency.

Recent progress: PTY terminal geometry responses now also include xterm
full-screen character-size queries. `CSI 19t` returns the same active
row/column grid as the text-area `CSI 18t` response because Datum's embedded PTY
surface does not expose a larger enclosing terminal desktop.

Recent progress: PTY terminal status responses now include xterm icon-label and
window-title query handling. `CSI 20t` and `CSI 21t` reply with ST-terminated
OSC responses derived from `TerminalLaneState.title`, stripping control bytes
from the title payload before writing the response back to the PTY. `CSI 22t`
and `CSI 23t` now save and restore the protocol title without visible terminal
output, so full-screen terminal apps can bracket title changes without losing
the outer Datum terminal title.

Recent progress: GUI source-shard health now exposes the resolver
dirty/missing/unknown breakdown directly in the project panel instead of only an
aggregate attention count, making stale promoted files and missing
journal-recovered shards distinguishable in the live workspace. The
`SourceShardStatusSummary` now carries per-shard attention rows with relative
path, kind, authority, taxon, and dirty state, and both the project panel and
terminal context surface that drilldown for non-clean shards.

Recent progress: real CLI OutputJob execution now has broader
generated-evidence replay coverage. Focused `project run-output-job`
regressions create and run Gerber, single-scope drill, aggregate BOM/PnP, and
manufacturing-set OutputJobs, capture the authored `ModelRevision`, remove the
promoted `.datum/output_job_runs/<run>.json` shard, and prove
`ProjectResolver` recovers the run from the journal without mutating the
authored revision. The manufacturing-set path also now proves generated child
Gerber output does not author a separate Gerber OutputJob.

Recent progress: OutputJobRun and ArtifactRun generated evidence now has an
explicit payload schema contract. New run evidence serializes
`schema_version: 1`; resolver validation rejects unsupported future versions
before insertion; legacy run evidence missing the field still reads as version
1; and replayed/generated source-shard metadata preserves the version. Focused
engine and CLI regressions cover versioned persistence, legacy compatibility,
unsupported-version rejection, replay metadata, `project run-output-job`
output, and ad hoc `artifact generate/list/show` output.

Recent progress: OutputJobRun and ArtifactRun provenance validation is now
owned by the generated-evidence validator. Optional provenance remains allowed
to be absent, but if present, terminal session IDs, terminal context paths,
project roots, and source revisions must be nonblank/nonempty before the run
evidence can enter the resolver model.

Recent progress: ArtifactMetadata generator provenance is now enforced by the
generated-evidence validator. Blank `generator_version` values are rejected both
by helper persistence and resolver discovery, so artifact evidence cannot enter
the model without a concrete producer version string.

Recent progress: ArtifactMetadata generated evidence now has an explicit
payload schema contract. New artifact metadata serializes `schema_version: 1`;
legacy promoted metadata missing the field still reads as version 1; and
unsupported future payload versions are rejected by producer/model validation
before artifact evidence can enter the resolver model.

Recent progress: production/output artifact shard discovery now enforces the
shared source-shard schema-version guard. Promoted `.datum/output_job_runs`,
`.datum/artifact_runs`, `.datum/artifacts`, `.datum/check_runs`,
`.datum/manufacturing_plans`, `.datum/panel_projections`, and
`.datum/output_jobs` JSON shards with unsupported future `schema_version` values
are rejected before generated evidence or authored production payloads enter the
model.

Recent progress: authored production records now have an explicit payload schema
contract. New ManufacturingPlan, PanelProjection, and OutputJob payloads
serialize `schema_version: 1`; legacy promoted records missing the field still
read as version 1; and unsupported future versions are rejected by
staging/model validation and resolver discovery before production records enter
the model.

Recent progress: ZoneFill generated evidence now has an explicit payload schema
contract. New fills serialize `schema_version: 1`; resolver validation rejects
unsupported future versions before they can become renderable copper; legacy
fills missing the field still read as version 1; and replayed/generated source
shard metadata preserves the version. Focused engine and CLI regressions cover
versioned persistence, legacy compatibility, unsupported-version rejection,
journal replay metadata, query/fill command output, and standards repair
proposal payloads.

Recent progress: ZoneFill thermal-relief handling is no longer blanket
unsupported. The bounded solver can fill a thermal-relief zone when no same-net
pad/via anchors intersect the bounded fill, records that no spokes were needed,
and still refuses same-net pad/via thermal anchors until real thermal
spoke/isolation geometry exists.

Recent progress: proposal policy now blocks automated direct generated-evidence
writes. `CommitSource::Tool` and `CommitSource::Assistant` direct commits that
set or delete `OutputJobRun`, `ArtifactRun`, `CheckRun`, `ArtifactMetadata`, or
`ZoneFill` evidence return
`proposal_required_for_automated_generated_evidence_operation` before staging
or promoting files; focused engine coverage exercises all ten set/delete
operation variants.

Recent progress: generated-evidence staging now rejects stale scope before
promotion. Journaled `SetOutputJobRun`, `SetArtifactRun`, `SetCheckRun`,
`SetArtifactMetadata`, and `SetZoneFill` operations must match the current
project/model revision at commit time; deletes remain ID/schema validated so
stale evidence can still be removed. Focused engine coverage rejects stale
model revisions for all five evidence families and wrong-project run evidence,
while replay regressions still recover accepted historical evidence.

Recent progress: CheckFinding target identity is now enforced by the engine
generated-evidence validator, not only by CLI output tests. Persisted CheckRun
evidence with a legacy null `primary_target` is rejected as
`invalid_check_run`; valid findings must carry a target object with concrete
`kind` and nonblank `id`, and related targets are validated by the same rule.

Recent progress: CheckRun generated-evidence helper persistence now invokes the
same semantic validator used by resolver discovery before writing
`.datum/check_runs`. Invalid CheckRun evidence is rejected at producer time,
while focused regressions still manually seed invalid promoted shards to prove
resolver quarantine remains intact.

Recent progress: CheckRun generated evidence now has an explicit payload schema
contract. New CheckRuns serialize `schema_version: 1`; legacy promoted evidence
missing the field still reads as version 1; and unsupported future payload
versions are rejected by producer/model validation before CheckRun evidence can
enter the resolver model.

Recent progress: Lower engine DRC finding identity now carries the same
standards metadata expected by CheckFinding. `DrcViolation` includes optional
`standards_basis`, `rule_revision`, and `import_key`; versioned DRC
fingerprints fold those slots into the hash; standards-backed DRC producers set
the current process-aperture/geometry basis plus `rule_revision="v1"`; and
daemon live DRC CheckRun views preserve those fields.

Recent progress: DRC fingerprint waiver/deviation lifecycle coverage now matches
the ERC path for the native-project slice. Focused CLI regressions waive and
accept-deviate a `connectivity_unrouted_net` DRC finding through the journaled
`project waive-finding` and `project accept-deviation` commands, assert domain
readback remains `drc`, and prove waiver undo/redo clears and restores
`waiver_refs` on the normalized CheckRun finding.

Recent progress: ComponentInstance authored identity now includes optional
`part_ref` to a native pool part. Resolver validation rejects missing, stale, or
wrong-kind part refs; commit validation rejects stale part revisions before
journal staging; CLI bind/set expose `--part`; and flat plus canonical MCP
ComponentInstance writes forward `part` through to the CLI bridge. Focused
engine, CLI, and MCP regressions cover resolver/query behavior and dispatch.

Recent progress: promoted ComponentInstance shard discovery now enforces the
shared source-shard schema-version guard. `.datum/component_instances/*.json`
shards with unsupported future `schema_version` values are rejected as
`invalid_component_instance_shard` before authored component identity enters the
model.

Recent progress: forward-annotation audit/proposal matching now uses resolver
ComponentInstance symbol/package refs before falling back to reference matching
for uncovered legacy objects. When a ComponentInstance carries `part_ref`, that
part becomes the expected identity for part-mismatch detection. Focused
regressions prove refdes-only matches do not create update actions for
already-bound objects, mismatches target the ComponentInstance-bound board
package even when references differ, and stale schematic symbol part fields do
not override authored `part_ref`. Forward-annotation artifact compare now
classifies exact `action_id` matches as applicable and same symbol/component
UUID plus same action/reason as drifted identity when reference changes alter
the derived `action_id`. The legacy reference/action fallback was removed, so
same-ref/action replacements with different object UUID identity are stale
rather than drifted; filter/apply eligibility remains limited to exact
applicable `action_id` matches.

Recent progress: ComponentInstance role metadata is now authored substrate
rather than an inferred compatibility join. Persisted shards and query output
carry optional per-symbol and per-package `{role, label}` records keyed by the
referenced object UUID; resolver and commit validation reject role keys outside
the selected refs, blank/invalid role identifiers, and invalid labels. CLI
`bind-component-instance` / `set-component-instance` plus MCP flat/canonical
bindings expose `symbol_roles` and `package_roles`, and new CLI-authored
bindings write default stable roles when explicit metadata is omitted.

Recent progress: ComponentInstance role metadata now reaches assembly outputs.
BOM and PnP CSV rows include `component_instance_role` and
`component_instance_label` projected from the authored package role map, while
inspect/compare keep accepting legacy headers. Role/label changes are reported
as BOM/PnP drift, and output-job manufacturing-set BOM/PnP artifacts preserve
the same columns when generated from a stored variant context.

Recent progress: resolver-synthesized exact-match ComponentInstances are
retired. ComponentInstances now enter the model from authored shards or
journaled operations, while resolver diagnostics still report unmatched and
ambiguous schematic-board relationships. BOM/PnP identity projection,
forward-annotation authoritative matching, variant propagation, resolver/debug
counts, and ComponentInstance query surfaces all use authored
ComponentInstances only.

Recent progress: source-shard taxonomy now covers the current authored
identity/relationship sidecars, sidecar metadata, and generated-evidence shard
families instead of only pool and authored production records. The shared
source-shard builder path derives concrete taxons for ComponentInstance,
Relationship, VariantOverlay, ImportMap, ProposalMetadata,
ForwardAnnotationReview, OutputJobRun, ArtifactRun, CheckRun, ZoneFill, and
ArtifactMetadata; focused engine coverage locks the canonical ownership path
and taxon mapping. CLI `resolve-debug` coverage also proves missing
journal-recovered ArtifactMetadata and ForwardAnnotationReview shards expose
their concrete taxons publicly alongside dirty-state and authority.

1. Continue contract reconciliation for domains outside the just-updated
   schematic and rules/checks contracts as new substrate slices land.
2. Continue source-shard taxonomy closure and keep the new GUI dirty-state
   summary aligned as additional shard families become authoritative.
3. Continue PTY terminal closure with attach/detach semantics if needed plus
   remaining VT screen-grid behavior such as additional controls.
4. Move to the next substrate blocker. The current revision-guard slice has
   no known unclassified public set/delete surface in the implemented
   operation vocabulary; public production authoring aliases are now
   proposal-mediated.

## Completion Rule

Do not mark the active goal complete until every row above is either `Done` or
explicitly moved out of the production substrate goal by a ratified scope
decision. Current evidence supports continued implementation, not completion.
