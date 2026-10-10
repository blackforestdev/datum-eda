# Source context, checks and production artifacts

> Detailed evidence relocated from `specs/PROGRESS.md`. The tracker remains the implementation-status entry point; the generated Active Frontier alone selects work. Dated history below does not grant execution or establish a fresh PASS.

Source-shard taxonomy correction:
- Remaining hand-built resolver/replay refs for ImportMap, ProposalMetadata,
  ComponentInstance, Relationship, VariantOverlay, production records, ZoneFill,
  and schematic sheet payloads now derive concrete taxons through the central
  source-shard path taxonomy instead of constructing `taxon=None` refs.

Context envelope correction:
- `datum-eda context get|refresh|session-events|session-activity` no longer returns only the raw GUI discovery
  file. The CLI now enriches compatible `datum_terminal_context_v1` discovery
  payloads with the first `DatumContextEnvelope` fields: `actor_type`,
  `capabilities`, resolver-derived `project_id` / `project_name` /
  `model_revision` when a project root resolves, `accepted_transaction_tip`,
  `visible_artifact_ids`, `visible_output_job_ids`,
  `visible_artifact_file_paths`, latest output-job/run/artifact IDs,
  latest artifact/artifact-run IDs, focused artifact/file defaults,
  `source_shard_status`, `latest_check_run_id`, `latest_profile_id`,
  `profile_latest_check_runs`, `visible_check_run_ids`, `output_context`,
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
  document the richer envelope and event stream; focused MCP coverage now proves
  `datum.context.get` preserves artifact, OutputJob, proposal, journal, and CheckFinding `active_context_commands` through the
  agent-facing target envelope. GUI protocol now defines shared typed session/context
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
  Multi-unit ComponentInstance refs now survive bind/set/delete undo and query.
  ComponentInstance shards now carry optional `part_ref` identity to a current
  native `pool/parts` object; resolver validation rejects stale/missing/non-part
  refs, commit validation rejects stale part revisions before staging, CLI
  bind/set expose `--part`, and MCP flat/canonical ComponentInstance write
  tools forward the same field to the CLI bridge. Forward-annotation
  audit/proposal matching now uses resolver ComponentInstance symbol/package
  refs before legacy reference fallback for uncovered objects, and treats
  `part_ref` as the expected part identity when present. Regressions prove
  refdes-only matches no longer create update actions for already-bound objects
  and mismatches target the ComponentInstance-bound package even when references
  differ. Forward-annotation artifact compare now classifies exact `action_id`
  matches as applicable and same symbol/component UUID identity plus same
  action/reason as `drifted` when a reference rename changes the derived
  `action_id`, without broadening filter/apply eligibility. The legacy
  reference/action fallback was removed, so same-ref/action replacements with
  different object UUID identity are stale rather than drifted.
  ComponentInstance shards now carry per-symbol and per-package `{role, label}`
  metadata keyed by referenced object UUID; resolver and commit validation reject
  role keys outside the selected refs, blank/invalid role identifiers, and
  invalid labels, while CLI/MCP bind/set can author `symbol_roles` and
  `package_roles` and default new bindings to stable role records. BOM/PnP
  assembly rows now project package role/label into `component_instance_role`
  and `component_instance_label`, compare reports role/label drift, inspect
  accepts both new and legacy CSV headers, and output-job manufacturing-set
  artifacts preserve the same columns through stored variant runs. Resolver
  synthesized exact-match joins are retired; authored ComponentInstances are
  the only source of stable BOM/PnP identity, forward-annotation `part_ref`
  authority, component-scoped variant propagation, resolver/debug counts, and
  ComponentInstance query rows, while package/symbol fitted-state entries
  remain valid.
  Automated cross-domain identity/intent writes are now proposal-required at the
  commit gateway: `Tool`/`Assistant` provenance batches that directly
  create/set/delete ComponentInstance, Relationship, or VariantOverlay shards
  are rejected with
  `proposal_required_for_automated_cross_domain_identity_operation`, keeping
  schematic/board/part bindings, design-intent relationships, and variant
  overlays on the reviewable proposal path. Automated pool/library writes are
  now covered by the same gateway policy: `Tool`/`Assistant` provenance batches
  that directly create/delete typed pool packages or padstacks, create/set/delete
  generic pool library objects, or attach/detach pool part models are rejected
  with `proposal_required_for_automated_library_operation`, so AI/tool-authored
  symbols, packages, padstacks, parts, pin maps, and model attachments cannot
  bypass proposal review before authored pool shards are staged. The MCP
  runtime now preserves that source boundary by launching CLI-backed library
  authoring methods with `DATUM_COMMIT_SOURCE=tool`, preventing canonical
  `datum.library.*` aliases from silently downgrading automation into
  `CommitSource::Cli`. Positive library proposal producers now exist for raw
  objects and typed semantic unit/symbol/entity/padstack/package/package-pad/package-courtyard paths: `datum-eda proposal
  create-pool-library-object` / `datum.proposal.create_pool_library_object` and
  `datum-eda proposal create-pool-unit` / `datum.proposal.create_pool_unit` and
  `datum-eda proposal create-pool-symbol` / `datum.proposal.create_pool_symbol`
  and `datum-eda proposal create-pool-entity` /
  `datum.proposal.create_pool_entity` and `datum-eda proposal
  create-pool-padstack` / `datum.proposal.create_pool_padstack` and
  `datum-eda proposal create-pool-package` /
  `datum.proposal.create_pool_package` and `datum-eda proposal
  set-pool-package-pad` / `datum.proposal.set_pool_package_pad` and
  `datum-eda proposal set-pool-footprint-courtyard-rect` /
  `datum.proposal.set_pool_footprint_courtyard_rect` and `datum-eda proposal
  set-pool-footprint-courtyard-polygon` /
  `datum.proposal.set_pool_footprint_courtyard_polygon` and
  `datum-eda proposal set-pool-package-courtyard-rect` /
  `datum.proposal.set_pool_package_courtyard_rect` and `datum-eda proposal
  set-pool-package-courtyard-polygon` /
  `datum.proposal.set_pool_package_courtyard_polygon` create non-mutating draft
  proposals, then `proposal accept-apply` writes the object through the generic
  proposal gateway.
- The deviation lifecycle is no longer pending for the current compatibility
  slice. `project accept-deviation` and MCP `accept_deviation` now author
  fingerprint-scoped accepted deviations through the native journal; CheckRun
  readback reports `status=accepted_deviation` and `deviation_refs`.
- DRC fingerprint lifecycle coverage now matches the ERC path for the current
  native-project slice. Focused CLI regressions waive and accept-deviate a
  `connectivity_unrouted_net` DRC finding through the journaled
  `project waive-finding` / `project accept-deviation` commands, prove domain
  readback is `drc`, and prove waiver undo/redo clears and restores
  `waiver_refs` on the normalized CheckRun finding.
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
  command templates including preview. ZoneFill standards-repair coverage now
  also proves generated `SetZoneFill` repair proposals link back into the
  CheckRun and finding `proposal_links` graph, so supported unfilled/stale
  copper-evidence repairs are visible to agents through the same proposal
  discovery path as process-aperture repairs. Canonical MCP
  `datum.check.repair_standards` now normalizes the repair output into
  `check_run_id`, `proposal_count`, and proposal readiness/affected-object
  fields while preserving the raw compatibility payload. Live and persisted CheckRun payloads now also carry
  deterministic `profile_basis` and `coverage` entries so evaluated,
  profile-filtered, and not-yet-implemented rule families are explicit instead
  of inferred from missing findings; canonical MCP normalization and GUI
  protocol preserve those fields. Standards profile basis rows, standards coverage rows, and generated standards findings now also carry typed v1 `standards_basis_detail` evidence with `basis_id`, registry-entry reference, selection scope, basis kind, disposition, and provenance while preserving flat `standards_basis` compatibility. The current process-aperture and ZoneFill honesty basis details are resolved from an engine-owned v1 check standards-basis registry seam instead of CLI-local fallback constructors, and unknown basis IDs are not silently coerced into a known basis family. Engine validation rejects malformed typed basis details before helper persistence or resolver acceptance. Finding target projection now maps common scalar payload identifiers such as `artifact_id`, `zone_id`, and `pad_id` to concrete domain target kinds while preserving `objects[]` / `object_uuids[]` compatibility for legacy DRC/ERC payloads. Live and persisted `CheckFinding` records now also carry
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
  catalog and matching MCP aliases. GUI-written and CLI-refreshed terminal
  contexts now also expose `active_context_commands.artifact_list` and
  `active_context_commands.proposal_list` as always-available
  `datum-eda artifact list <project-root>` and
  `datum-eda proposal list <project-root>` discovery commands when the project
  root is known. When a previous artifact and current/latest artifact are both
  known, contexts also expose `active_context_commands.artifact_compare` as
  `datum-eda artifact compare <project-root> --before <previous> --after
  <latest>`, so terminal-launched agents can compare generated evidence without
  rescanning artifact history. GUI-written, CLI-refreshed, and MCP-preserved
  context envelopes also expose `active_context_commands.source_shards` as
  `datum-eda project query <project-root> resolve-debug`, giving terminal-launched
  agents a direct resolver/source-shard diagnostic command that mirrors the
  canonical `datum.query.source_shards` read model. They also expose `active_context_commands.check_run`,
  `active_context_commands.check_list`,
  `active_context_commands.check_profiles`, and
  `active_context_commands.check_fill_zones` as always-available
  `datum-eda check run|list|profiles|fill-zones <project-root>` commands, so
  terminal-launched agents can discover generated evidence, run checks, inspect
  profiles, and refresh ZoneFill generated evidence before a latest run or
  selected finding exists. Remaining CheckRun UX gaps are
  deeper profile configuration and richer first-class GUI review/apply widgets;
  the canonical waive/deviation taxonomy now exists for the current
  native-project slice.
- ERC `object_uuids[]` are no longer only preserved as raw compatibility
  payload. Live native `project query <root> erc` now projects those UUIDs into
  normalized `object_uuid` `primary_target` / `related_targets` fields while
  still preserving the raw array in payload/evidence. Focused ERC regression
  coverage fails if an ERC finding falls back to `erc/unknown` when object UUID
  evidence is present.
- Engine-daemon `run_erc` and `run_drc` now return live non-persisted
  `check_run_v1` envelopes instead of top-level raw arrays/reports. The raw
  compatibility payloads remain nested under `raw_report.erc` and
  `raw_report.drc`, while normalized findings carry target objects,
  deterministic fingerprints, summaries, profile basis, and coverage. The
  lower in-memory engine API remains raw; canonical persisted generated
  evidence remains `datum-eda check run` / `datum.check.run`.
- Persisted check-history list UX now exposes `latest_check_run_id`,
  `latest_profile_id`, `profile_latest_check_runs[]`, and per-row
  `latest_for_profile` flags from `datum-eda check list`, so GUI surfaces and
  terminal-launched agents can select the current run per profile without
  duplicating resolver sort semantics. GUI-written terminal context and CLI
  `context get|refresh` now carry the same latest CheckRun profile summary into
  `datum_check_context_v1` / top-level terminal-agent envelopes.
- Engine-daemon and MCP `explain_violation` now accept a stable finding
  `fingerprint` and resolve it against the current live CheckRun finding set.
  Legacy positional `index` remains accepted for compatibility, but new callers
  can avoid index drift when explaining ERC/DRC findings.
- Canonical MCP `datum.check.explain_violation` is now public and dispatches to
  the same fingerprint-capable implementation, while hidden flat
  `explain_violation` now declares `datum.check.explain_violation` as its real
  canonical replacement instead of a pending migration target. The MCP taxonomy
  guard now locks 338 public tools, 525 registered tools, and 187 hidden
  compatibility aliases.
- Lower engine DRC fingerprints now use
  `datum-eda:drc-violation-fingerprint:v2` material that includes
  `standards_basis`, `rule_revision`, and `import_key` slots. Standards-backed
  DRC producers stamp the current process-aperture/geometry basis plus
  `rule_revision="v1"` before fingerprinting, and daemon DRC CheckRun views
  preserve those fields instead of flattening them to null.

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
  `datum-eda artifact list` now exposes top-level `latest_artifact_id`,
  `latest_artifact_run_id`, and `latest_output_job_run_id` pointers so GUI
  surfaces and terminal-launched agents can select current generated evidence
  without duplicating artifact/run sorting. Latest artifact selection is now
  explicitly keyed by artifact `model_revision` plus UUID tie-break rather than
  resolver map order, with focused coverage proving a newer low-UUID artifact
  wins over an older high-UUID artifact.
  The GUI terminal discovery envelope and CLI `context get|refresh` now carry
  `latest_artifact_id`, `latest_artifact_run_id`, and `latest_check_run_id`
  alongside the existing latest output-job fields so terminal-launched agents
  can navigate generated evidence and check history from one context file.
  GUI production visibility now prefers canonical artifact-list
  `latest_output_job_run_id` over the legacy aggregate `latest_run_id` fallback
  when no output-job row carries its own latest run summary, and reconstructs
  latest output-job/artifact IDs from matching `artifact_runs[]` evidence.
  GUI production refresh now uses canonical `latest_artifact_id` /
  `latest_output_job_run_id` navigation context before falling back to output-job
  row order, so the OUTPUTS dock focuses the current generated evidence instead
  of the first artifact listed under an older job.
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
  Artifact-only CLI generation now has replay coverage matching the engine
  oracle: the focused regression removes promoted `.datum/artifacts/<id>.json`
  and `.datum/artifact_runs/<run>.json` sidecars after `datum-eda artifact
  generate`, then proves `ProjectResolver` recovers both generated-evidence
  records from the journal while the authored `ModelRevision` remains stable.
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
