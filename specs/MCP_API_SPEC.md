# MCP API Specification

Status: target implementation spec with current compatibility contract.

This document defines the target Datum MCP surface under the product-mechanics
scope and records the current implemented daemon/MCP method list for
compatibility. It intentionally separates:

- **Target Surface**: the implementation direction for MCP and CLI.
- **Current Compatibility**: methods that exist today and must keep working
  until compatibility aliases are retired.

The target surface is derived from
`docs/contracts/AI_CLI_MCP_TOOL_SURFACE.md`. That contract is the shared
authority for the seven tool classes and the product vocabulary used here.

## Transport

```text
MCP Client <-> MCP Server (Python, stdio) <-> Engine (Rust, JSON-RPC over Unix socket)
```

The MCP server remains a thin translation layer. Target MCP tools translate
stable `datum.*` requests into engine operations, queries, checks, proposal
workflows, artifact generation, or journal reads. The MCP server must not
become an alternate state store and must not write project files directly.

## Target Surface

CLI and MCP are isomorphic:

- CLI: `datum-eda <group> <verb> ...`
- MCP: `datum.<group>.<verb>`

Every CLI command with `--json` returns the same schema as the matching MCP
tool. Human-readable CLI output is presentation only.

The target public surface has exactly seven shared contract classes. Public MCP
methods may use granular canonical `datum.<family>.*` prefixes such as
`datum.pcb.*`, `datum.schematic.*`, `datum.library.*`, and `datum.route.*`
when they are typed realizations of these classes. Those granular prefixes are
canonical public families, not flat legacy aliases. The drift gate
`scripts/check_mcp_public_taxonomy.py` locks the current public prefix
inventory and fails if non-`datum.*` names, hidden compatibility aliases, or
non-journaled write bypasses re-enter `tools/list`.

The seven contract classes are:

| Class | CLI shape | MCP shape | Mutates source? |
|-------|-----------|-----------|-----------------|
| Context / session | `datum-eda context get|refresh|session-events` | `datum.context.get`, `datum.context.refresh`, `datum.context.session_events` | No |
| Query | `datum-eda query <family>` | `datum.query.*` | No |
| Check | `datum-eda check run|show` | `datum.check.*` | No source mutation |
| Proposal | `datum-eda proposal create|show|validate|reject|defer|apply` | `datum.proposal.*` | Apply only |
| Commit / apply gateway | internal `commit()` | reached through `datum.proposal.apply` or policy-gated direct commit | Yes |
| Artifact | current: `datum-eda artifact list|generate|show|files|compare|validate|export-manufacturing-set|validate-manufacturing-set`; generic generate currently supports `gerber-set`, `manufacturing-set`, `bom`, `pnp`, `drill`, and `all` scopes | `datum.artifact.*` | No |
| Journal | `datum-eda journal list|show|undo|redo` | `datum.journal.*` | Undo/redo only |

Per-domain docs may add typed `Operation` variants, query families, check
profiles, artifact include values, and proposal producers. They must not add
peer MCP tool classes or private write paths.

## Target JSON Envelope

All target MCP responses use a normalized envelope:

```json
{
  "ok": true,
  "schema": {
    "name": "datum.<group>.<verb>",
    "version": 1
  },
  "context": {
    "project_id": "uuid",
    "model_revision": "rev",
    "variant": "default",
    "output_context": null
  },
  "result": {}
}
```

Errors use the same envelope shape with `ok: false`:

```json
{
  "ok": false,
  "schema": {
    "name": "datum.<group>.<verb>",
    "version": 1
  },
  "context": {
    "project_id": "uuid",
    "model_revision": "rev",
    "variant": "default",
    "output_context": null
  },
  "error": {
    "code": "stale_model_revision",
    "message": "Request was prepared against an older model revision",
    "details": {
      "prepared_against": "rev-41",
      "current": "rev-42"
    }
  }
}
```

Envelope principles:

- `schema.name` is the MCP method name; `schema.version` is incremented only
  for incompatible payload changes.
- `context.project_id`, `context.model_revision`, `context.variant`, and
  `context.output_context` are present whenever the request is project-backed.
- `result` is tool-specific and never changes the meaning of the envelope.
- `error.code` is symbolic and stable. JSON-RPC numeric transport codes are
  transport details, not the public Datum error contract.
- Compatibility aliases may return legacy payloads during migration, but new
  `datum.*` tools must return the target envelope.
- Current MCP `tools/call` handling wraps canonical `datum.*` success payloads
  in this envelope at the MCP boundary. `datum.check.*`, `datum.artifact.*`,
  `datum.proposal.*`, `datum.journal.*`, `datum.component_instance.*`, and the
  first resolver-backed `datum.query.*` set now receive first-pass
  tool-specific normalized `result` payloads. These expose stable check-run,
  finding, artifact, file, validation, production projection, proposal,
  transaction, component-instance, relationship, variant, import-map, panel, and
  manufacturing-plan fields while retaining the original CLI/daemon payload
  under `result.raw`. During migration it also copies raw result fields at the
  top level for compatibility with older callers. Tool-level failures for
  canonical `datum.*` calls return `ok: false` envelopes instead of JSON-RPC
  transport errors.

## Model Context Resolution

Target tools resolve project-backed context through `ProjectResolver`, not
through ad-hoc paths or one mutable MCP-global project slot.

Each project-backed request accepts enough context to resolve:

- `project_id` or `project_root`
- `model_revision` or explicit `latest` read intent
- `variant`
- `output_context` for projection/artifact/manufacturing scopes

`ProjectResolver` is responsible for locating the model store, validating the
requested revision, resolving variant and output context, and returning the
canonical `DatumContextEnvelope`. Tools must use that envelope for all reads,
checks, proposals, artifact generation, and journal operations.

Target rules:

- Queries may read `latest` or a pinned `model_revision`; responses report the
  actual revision used.
- Proposals must be prepared against an explicit `model_revision`.
- Applies must reject stale proposals unless the proposal is explicitly
  rebased and revalidated.
- Artifacts must record the exact `project_id`, `model_revision`, `variant`,
  `output_context`, generator version, and content hashes.
- Check runs must record the exact `project_id`, `model_revision`, `variant`,
  check profile, checker version, and finding fingerprints.

## Target Tools

### 1. Context / Session

MCP:

- `datum.context.get`
- `datum.context.refresh`
- `datum.context.session_events`
- `datum.context.session_activity`

CLI:

- `datum-eda context get`
- `datum-eda context refresh`
- `datum-eda context session-events`
- `datum-eda context session-activity`

`get` and `refresh` return the `DatumContextEnvelope`: session id, actor,
capabilities, project identity, model revision, variant, output context,
projection/selection/cursor context, visible findings/artifacts, provenance
seed, and expiry/refresh metadata. `session-events` returns
`datum_tool_session_events_v1` with the resolved context path, tool-session
event log path, returned `event_count`, `matched_event_count`, raw
`total_event_count`, applied `filters`, optional `limit`, and parsed JSONL
events. `session-events` accepts exact-match `event_kind`, `origin`, and
`command_id` filters; `event_kind` matches the JSONL field named `event`.
`limit` returns the newest N matching events after filters while preserving
chronological order in the returned slice. `terminal_command_handoff` events
carry `origin`, stable `command_id`, optional `mcp_alias`, `handoff_mode`
(`prefill|execute`), command text, and timestamp. `session-activity` returns
`datum_tool_session_activity_summary_v1` over the same filter/limit semantics:
raw/matched/activity counts, first/last occurrence time, event-kind/origin/
handoff-mode counts, and compact per-command activity summaries.

Context/session is capability state. It is not an `Operation`, is not
journaled, and must not grant direct filesystem write access.

Current implementation:

- `datum.context.get`, `datum.context.refresh`, `datum.context.session_events`,
  and `datum.context.session_activity` are exposed as MCP tools and bridge to
  `datum-eda context get|refresh|session-events|session-activity`.
- The current context payload remains compatible with the GUI terminal discovery
  envelope (`datum_terminal_context_v1`) and now carries the first
  `DatumContextEnvelope` fields: actor type, capabilities, project root/id/name,
  model revision when resolvable, accepted transaction tip, visible artifact and
  check-run ids, output context ids, provenance seed, expiry placeholder, and
  refresh command. GUI terminals write authoritative per-session discovery under
  `.datum/terminal-contexts/<session>.json` and preserve
  `.datum/gui-terminal-context.json` as a latest-session compatibility alias.
  `context get` returns the normalized envelope without mutating it; `context
  refresh` writes the enriched envelope back to the resolved context file while
  preserving GUI-owned typed selection/cursor/projection/session fields; and
  `context session-events` reads `.datum/tool-sessions/<session>.events.jsonl`
  as parsed, non-journaled tool-session provenance.
  GUI-authored context files also include first launch-time
  `selection_context`, `cursor_context`, and `projection_context` snapshots.
  The shared GUI protocol now defines typed context/session structs for these
  fields, GUI terminals persist `DatumToolSessionMetadata` under
  `.datum/tool-sessions/<session>.json`, and GUI selection, cursor/hover, dock,
  and frame-affecting session events rewrite the same per-session context file.
  The same discovery envelope includes `agent_commands` templates for
  terminal-launched optional agents (`codex`, `claude`, `aider`) plus context
  inspection/refresh helpers; these are shell-launch hints only and do not
  create a privileged assistant mutation path. The GUI `AGENTS` dock entry now
  routes to the PTY and prints those launch hints rather than making the
  assistant lane the canonical agent entry point.
  Production command templates in that discovery envelope now advertise
  OutputJob, ManufacturingPlan, and PanelProjection create/update/delete as
  `datum-eda project ... --as-proposal` commands, so terminal-launched agents
  discover proposal drafting rather than direct production source mutation.
  Richer engine-owned session policy/expiry semantics remain target-scope.

### 2. Query

MCP:

- `datum.query.*`

CLI:

- `datum-eda query <family> [--domain ...] [--at <model_revision>]`

Queries are read-only. Existing `get_*`, `search_*`, report, relationship,
pool, schematic, PCB, rules, manufacturing, artifact, transaction, and
provenance reads fold into query families rather than new peer methods.

Current implementation:

- Current native-project CLI aliases include `datum-eda query summary <root>`,
  `datum-eda query component-instances <root>`, schematic read aliases
  `sheets`, `symbols`, `labels`, `ports`, `buses`, `bus-entries`,
  `noconnects`, `hierarchy`, `schematic-nets`, and
  `connectivity-diagnostics`, plus `datum-eda query relationships <root>`,
  `datum-eda query variants <root>`, `datum-eda query import-map <root>`,
  `datum-eda query zone-fills <root>`, `datum-eda query panel-projections
  <root>`, `datum-eda query manufacturing-plans <root>`, and `datum-eda query
  output-jobs <root>`.
  Legacy imported-design reads remain available as `datum-eda query imported
  <file> <what>` and through the historical `datum-eda query <file> <what>`
  compatibility parser.
- The first compatibility-backed query aliases are exposed:
  `datum.check.run`, `datum.query.board_summary`,
  `datum.query.components`, `datum.query.netlist`,
  `datum.query.schematic_summary`, `datum.query.sheets`,
  `datum.query.symbols`, `datum.query.symbol_fields`, `datum.query.labels`,
  `datum.query.ports`, `datum.query.buses`, `datum.query.bus_entries`,
  `datum.query.noconnects`, `datum.query.hierarchy`,
  `datum.query.schematic_nets`, `datum.query.connectivity_diagnostics`,
  `datum.query.design_rules`, `datum.query.zone_fills`,
  `datum.query.source_shards`, `datum.query.component_instances`, `datum.query.relationships`,
  `datum.query.variants`, `datum.query.import_map`,
  `datum.query.panel_projections`, `datum.query.manufacturing_plans`,
  `datum.query.output_jobs`,
  `datum.proposal.list`, and `datum.proposal.show`.
  `datum.query.source_shards` mirrors resolver source-shard metadata including
  `dirty_state` and concrete `taxon` values for typed pool shards and authored
  production records (`ManufacturingPlan`, `PanelProjection`, `OutputJob`).
- These aliases dispatch to the existing compatibility implementations
  (`get_check_run`, `get_board_summary`, `get_components`, `get_netlist`,
  `get_schematic_summary`, `get_sheets`, `get_symbols`,
  `get_symbol_fields`, `get_labels`, `get_ports`, `get_buses`,
  `get_bus_entries`, `get_noconnects`, native path-aware
  `get_project_hierarchy` with legacy `get_hierarchy` fallback,
  `get_schematic_net_info`, `get_connectivity_diagnostics`,
  `get_design_rules`, `get_source_shards`, `get_zone_fills`, `get_component_instances`,
  `get_relationships`, `get_variants`, `get_import_map`,
  `get_panel_projections`, `get_manufacturing_plans`, `get_proposals`, and
  `show_proposal`) until the full `datum.*` families replace flat method
  names.
- The first ZoneFill producer aliases are exposed as flat `fill_zones` and
  canonical `datum.check.fill_zones`. They bridge to canonical CLI
  `datum-eda check fill-zones`, which currently persists honest
  `Unsupported` generated evidence rather than pretending a copper-fill
  solver exists.
- ComponentInstance write aliases now include `datum.component_instance.bind`,
  `datum.component_instance.set`, and `datum.component_instance.delete`. These
  dispatch to journal-backed compatibility implementations
  (`bind_component_instance`, `set_component_instance`, and
  `delete_component_instance`).
- Check aliases now also include `datum.check.repair_standards`,
  `datum.check.waive`, `datum.check.accept_deviation`, and
  `datum.check.explain_violation`, all dispatched to the existing
  check/proposal/journal compatibility implementations as applicable.
- Proposal aliases now also include `datum.proposal.validate`,
  `datum.proposal.review`, `datum.proposal.defer`, `datum.proposal.reject`,
  `datum.proposal.accept_apply`, and `datum.proposal.apply`, all dispatched to
  the existing proposal compatibility implementations. Producer-specific
  proposal aliases now also exist under the same family:
  `datum.proposal.create_board_component_replacement`,
  `datum.proposal.create_board_component_replacements`,
  `datum.proposal.create_board_component_replacement_plan`,
  `datum.proposal.create_pool_library_object`,
  `datum.proposal.create_pool_unit`,
  `datum.proposal.create_pool_symbol`,
  `datum.proposal.create_pool_entity`,
  `datum.proposal.create_pool_padstack`,
  `datum.proposal.create_pool_package`,
  `datum.proposal.create_pool_footprint`,
  `datum.proposal.create_pool_pin_pad_map`,
  `datum.proposal.set_pool_pin_pad_map`,
  `datum.proposal.set_pool_footprint_pad`,
  `datum.proposal.set_pool_footprint_courtyard_rect`,
  `datum.proposal.set_pool_footprint_courtyard_polygon`,
  `datum.proposal.add_pool_footprint_silkscreen_line`,
  `datum.proposal.add_pool_footprint_silkscreen_rect`,
  `datum.proposal.add_pool_footprint_silkscreen_circle`,
  `datum.proposal.add_pool_footprint_silkscreen_polygon`,
  `datum.proposal.set_pool_package_pad`,
  `datum.proposal.set_pool_package_courtyard_rect`,
  `datum.proposal.set_pool_package_courtyard_polygon`,
  `datum.proposal.create_panel_projection`,
  `datum.proposal.update_panel_projection`,
  `datum.proposal.delete_panel_projection`,
  `datum.proposal.create_manufacturing_plan`,
  `datum.proposal.update_manufacturing_plan`,
  `datum.proposal.delete_manufacturing_plan`,
  `datum.proposal.create_output_job`,
  `datum.proposal.update_output_job`, and
  `datum.proposal.delete_output_job`.

Target query responses must be revision-pinned and must prefer stable IDs over
names. Cross-domain joins use product-mechanics identity such as
`ComponentInstance`; agents must not reconstruct joins by refdes/name/path
string matching.

### 3. Check

MCP:

- `datum.check.run`
- `datum.check.list`
- `datum.check.show`
- `datum.check.profiles`
- `datum.check.explain_violation`

CLI:

- `datum-eda check run --domain <erc|drc|standards|process|manufacturing|all> --profile <id>`
- `datum-eda check show <check_run_id>`
- Current native-project implementation supports `datum-eda check run <root>`,
  `datum-eda check run <root> --profile native-combined`,
  `datum-eda check list <root>`,
  `datum-eda check show <root> --check-run <check_run_id>`,
  `datum-eda check profiles <root>`,
  `datum-eda check repair-standards <root>`,
  `datum-eda check waive <root> --fingerprint <sha256:...> --rationale <text>`,
  and `datum-eda check accept-deviation <root> --fingerprint <sha256:...>
  --rationale <text>`. Legacy imported-design checking is retained under
  `datum-eda check imported <file>`.

Checks do not mutate source design state. Persisted `CheckRun` records are
derived evidence with revision, variant, checker version, profile, status, and
stable finding fingerprints. Finding explanations and suggested next actions
belong on `CheckFinding`; standalone index-addressed explain methods are
compatibility only. Current native-project check runs preserve compatibility
`proposal_refs` and also expose structured `proposal_links` at run and finding
level with proposal status, source, rationale, current validation snapshot, and
canonical `datum-eda proposal ...` command templates for show, preview,
validate, review, and apply actions.
Canonical `datum.check.repair_standards` returns the standard `datum.*`
envelope with normalized `check_run_id`, `proposal_count`, and `proposals[]`
fields. Each proposal entry exposes the stable proposal id, repair kind,
affected pad/track/via/net-class/zone ids when applicable, finding
fingerprints, repair codes, prepared-against revision, current-revision
readiness, apply blocker codes, operation count, and raw compatibility payload.

### 4. Proposal

MCP:

- `datum.proposal.create`
- `datum.proposal.list`
- `datum.proposal.show`
- `datum.proposal.preview`
- `datum.proposal.validate`
- `datum.proposal.reject`
- `datum.proposal.defer`
- `datum.proposal.review`
- `datum.proposal.accept_apply`
- `datum.proposal.apply`

CLI:

- `datum-eda proposal create|list|show|preview|validate|review|reject|defer|accept-apply|apply`
- Current native-project implementation supports `datum-eda proposal list
  <root>`, `datum-eda proposal create <root> --batch <operation-batch.json>
  --rationale <text>`, `datum-eda proposal show <root> --proposal <uuid>`,
  `datum-eda proposal preview <root> --proposal <uuid>`,
  `datum-eda proposal validate <root> --proposal <uuid>`,
  `datum-eda proposal review <root> --proposal <uuid> --status
  <accepted|deferred|rejected>`, `datum-eda proposal defer <root> --proposal
  <uuid>`, `datum-eda proposal reject <root> --proposal <uuid>`,
  `datum-eda proposal accept-apply <root> --proposal <uuid>`, and
  `datum-eda proposal apply <root> --proposal <uuid>`. `accept-apply` is a
  convenience composition of accepted review plus revision-guarded apply; it is
  not a direct mutation bypass. `proposal preview` dry-runs the stored batch on
  a cloned resolved model and returns `proposal_preview_v1` with classified
  diff, affected objects, and validation state; it writes no shards. Generic
  proposal creation writes draft sidecar metadata only and does not mutate
  source design state.

Proposals own a typed `OperationBatch`, rationale, affected object IDs,
expected result, prepared-against `model_revision`, assumptions, risks, checks,
and session provenance. Proposal creation never mutates source design state.

High-risk, cross-domain, batch, destructive, generated, checker-authored, or
assistant-authored changes must go through proposal review unless an explicit
direct-commit policy grants otherwise.

### 5. Commit / Apply Gateway

There is exactly one mutation gateway: `commit(OperationBatch)`.

`commit()` is an internal engine contract, not a standalone general user verb.
It is reached by accepted proposal apply and by narrowly policy-gated direct
authoring edits.

Target commit rules:

- Every source mutation is a typed `OperationBatch`.
- Every committed batch produces object revisions, a new `model_revision`, and
  one journal transaction.
- Applies reject stale base revisions unless explicitly rebased and revalidated.
- Applies validate object revisions, affected IDs, capability, provenance, and
  acceptance policy.
- AI/assistant/checker/import-repair paths may propose but must not silently
  bypass proposal policy.

No private mutation paths are allowed. Per-method writes, direct shard writes,
derived-cache writes as source, direct `write_canonical_json`, per-domain
save/write methods, or MCP handlers that mutate outside `commit()` are target
contract violations.

### 6. Artifact

MCP:

- Current:
  - `datum.artifact.list`
  - `datum.artifact.generate`
  - `datum.artifact.show`
  - `datum.artifact.files`
  - `datum.artifact.preview`
  - `datum.artifact.compare`
  - `datum.artifact.validate`
  - `datum.artifact.export_manufacturing_set`
  - `datum.artifact.validate_manufacturing_set`

CLI:

- Current:
  - `datum-eda artifact list <root>`
  - `datum-eda artifact generate <root> --output-dir <dir> --include <gerber-set|manufacturing-set|bom|pnp|drill|all>[,<scope>...] [--prefix <prefix>]`
  - `datum-eda artifact generate <root> --output-job <uuid> [--output-dir <dir>]`
  - `datum-eda artifact show <root> --artifact <artifact_id>`
  - `datum-eda artifact files <root> --artifact <artifact_id>`
  - `datum-eda artifact preview <root> --artifact <artifact_id> [--artifact-dir <dir>] --file <relative_path>`
  - `datum-eda artifact compare <root> --before <artifact_id> --after <artifact_id>`
  - `datum-eda artifact validate <root> --artifact <artifact_id>`
  - `datum-eda artifact export-manufacturing-set <root> --output-dir <dir> [--prefix <prefix>] [--include <scope>[,<scope>...]] [--output-job <uuid>|--job <name>] [--variant <uuid>]`
  - `datum-eda artifact validate-manufacturing-set <root> --output-dir <dir> [--prefix <prefix>] [--include <scope>[,<scope>...]] [--output-job <uuid>|--job <name>] [--variant <uuid>]`

Artifacts are derived projections, never source authority and never committed
through `commit()`. Format and scope are parameters (`--include`), not separate
peer tool families. The current generic generator accepts the implemented
`gerber-set`, `manufacturing-set`, `bom`, `pnp`, `drill`, and `all` scopes.
The `bom` and `pnp` scopes each produce one CSV artifact. The `drill` scope
produces a drill CSV plus Excellon drill artifact and carries the Excellon
production projection proof. Generic artifact validation performs semantic
comparison for the `bom`, `pnp`, and `drill` scopes and persists their
`ArtifactMetadata.validation_state`. `artifact generate --output-job` executes
an authored `OutputJob`, uses the job's stored include/prefix/variant/output
directory unless an output-directory override is supplied, and records one
`OutputJobRun` for the logical command. Panel/manufacturing/output-job context
is represented in `output_context`.

Current OutputJob authoring also supports
`datum-eda project create-output-job <root> --prefix <prefix> --include <gerber-set|manufacturing-set|bom|pnp|drill|all>[,<scope>...]`.
The compatibility MCP bridge exposes the same operation as
`create_output_job`; the older `create_gerber_output_job` remains for Gerber-set
compatibility. Generated BOM/PnP/drill artifacts attach to a matching authored
OutputJob when scope and prefix match, including jobs whose stored include list
contains multiple scopes.
Direct manufacturing-set export, validate, compare, manifest, and inspect
compatibility commands accept `--include`, `--output-job <uuid>`, and exact-name
`--job <name>`; when a job is supplied, its prefix, variant, and stored include
list are used unless an explicit CLI flag overrides them. Duplicate job names
are rejected as ambiguous.

Every artifact records project id, model revision, variant, output context,
generator version, projection revision, validation/equivalence state, and
per-file content hashes.

### 7. Journal

MCP:

- `datum.journal.list`
- `datum.journal.show`
- `datum.journal.undo`
- `datum.journal.redo`

CLI:

- `datum-eda journal list|show|undo|redo`
- Current native-project implementation supports `datum-eda journal list
  <root>`, `datum-eda journal show <root> --transaction <uuid>`,
  `datum-eda journal undo <root>`, and `datum-eda journal redo <root>`.

Journal is global and project-wide. `list` and `show` are read-only. `undo` and
`redo` are compensating `OperationBatch` commits through the same `commit()`
gateway, not private reversal paths.

## OperationBatch / Proposal / Apply Rules

`OperationBatch` is the only source mutation payload. It contains:

- batch id
- prepared-against `project_id`, `model_revision`, and `variant`
- typed operations
- affected object IDs and expected object revisions
- actor/session/provenance
- validation requirements
- optional proposal id and acceptance record

Proposal lifecycle:

1. `datum.proposal.create` produces a proposal with an `OperationBatch` and a
   machine-readable preview/diff.
2. `datum.proposal.validate` checks base revision, object revisions,
   capabilities, checks, and policy.
3. `datum.proposal.apply` revalidates the proposal and then calls `commit()`.
4. `datum.journal.list|show` expose the resulting transaction and proposal
   lineage.

Apply rejection reasons include stale model revision, missing acceptance,
capability denial, invalid operation, object revision mismatch, failed required
check, proposal conflict, and unsupported operation.

Current implementation:

- Generic proposal validation is backed by the engine-level
  `ProposalApplyValidation` object.
- Generic proposal creation accepts an `OperationBatch`, stamps or validates
  its prepared model revision guard, dry-runs the batch for operation
  validation/affected-object preview, and writes draft proposal metadata.
- `project validate-proposal` returns stable `blocker_codes` and structured
  blocker objects with `code` and `message`.
- `project review-proposal --status accepted` refuses stale proposals and
  proposals whose embedded `OperationBatch` lacks the prepared model revision
  guard; deferred and rejected reviews remain allowed for stale drafts.
- Current blocker codes include `missing_acceptance`, `stale_model_revision`,
  and `missing_revision_guard`.

## Compatibility Alias Policy

Compatibility aliases exist to keep current clients working while the target
surface is implemented. Aliases must be thin adapters over target tools once
the target tool exists.

Required aliases:

- `open_project` -> `datum.context.get` / `datum.context.refresh` through
  `ProjectResolver`
- `close_project` -> session/context release or no-op once context is
  resolver-backed rather than one global open project
- `save` -> compatibility wrapper only; target writes are committed at apply
  time and do not require a public save mutation
- `run_erc` -> live `check_run_v1` compatibility envelope with `profile_id = "erc"`;
  persisted generated evidence remains `datum.check.run`
- `run_drc` -> live `check_run_v1` compatibility envelope with `profile_id = "drc"`;
  persisted generated evidence remains `datum.check.run`

Per-method write compatibility:

- Existing mutation methods such as move, rotate, set field/value/reference,
  delete, assign part/package, replacement, rule edits, routing apply, undo,
  and redo may remain as compatibility methods during migration.
- Once `OperationBatch` and `commit()` exist, each compatibility write method
  must translate into a typed operation or accepted proposal and must enter
  through `commit()`.
- No compatibility method may retain a private mutation path after the target
  commit gateway is available for its operation type.
- New write capabilities must not be introduced as flat compatibility methods;
  they must be implemented as typed operations under proposal/apply or
  policy-gated direct commit.

Alias retirement policy:

- Compatibility aliases are documented in this section only.
- Target docs and examples use `datum-eda` CLI and `datum.*` MCP names.
- Aliases must preserve current behavior until a migration plan declares them
  deprecated and then removed.
- Alias payloads may remain legacy during transition, but target `datum.*`
  payloads must use the target JSON envelope.
- Every hidden compatibility alias registered by the MCP catalog must carry
  machine-readable retirement metadata:
  `x_compatibility_visibility = "hidden"`,
  `x_canonical_replacements`, `x_retirement_status`, and
  `x_retirement_criteria`.
- `x_canonical_replacements` is a non-empty list of public replacement tools or
  explicit `pending:<datum.* surface>` migration targets. Public replacements
  must appear in `tools/list`; pending targets mark gaps that still require a
  public canonical surface before the alias can move past
  `retained_until_migration_plan`.
- `x_retirement_status` is one of `retained_until_migration_plan`,
  `deprecated`, or `scheduled_for_removal`. New hidden aliases start as
  `retained_until_migration_plan`; moving to `deprecated` requires a named
  canonical `datum.*` replacement and documentation update; moving to
  `scheduled_for_removal` requires confirmation that current supported clients
  no longer need the alias.
- Retirement criteria must state the removal condition. The default condition
  is: remove after the canonical `datum.*` replacement is public,
  journal/proposal-backed where it mutates source, documented here, and current
  client compatibility requirements no longer require the alias.
- The first deprecated hidden families are legacy session, check,
  check/explain, journal, artifact/output, ComponentInstance, pool lookup,
  read-only query, replacement-planning, manufacturing-authoring, OutputJob,
  route, proposal, PCB, and library aliases. Each has a public
  `datum.session.*`, `datum.check.*`,
  `datum.journal.*`, `datum.artifact.*`, `datum.component_instance.*`,
  `datum.pool.*`, `datum.query.*`, `datum.replacement.*`,
  `datum.manufacturing.*`, `datum.output_job.*`, `datum.route.*`,
  `datum.proposal.*`, `datum.pcb.*`, or `datum.library.*` replacement and
  alias-specific retirement criteria.
- The hidden flat session aliases (`open_project`, `close_project`, `save`,
  `validate_project`) are deprecated in favor of public `datum.session.*`
  replacements. `datum.session.save` remains an imported-design compatibility
  write-back surface only; target authored writes commit at apply time and no
  new public save mutation should be introduced.
- `scripts/check_mcp_public_taxonomy.py` enforces both the public prefix
  inventory and hidden-alias retirement metadata. A hidden alias without this
  metadata is a spec drift failure, even if it is not advertised by
  `tools/list`.

## Current Compatibility

This section is authoritative for the current implemented daemon/MCP method
names. It preserves implementation truth; it is not the target surface.

Current implementation note: implemented in the current daemon/stdio host.

### Current Implemented Methods (2026-06-28)

`close_project`, `edit_scoped_component_replacement_plan`,
`explain_violation`, `get_board_summary`, `get_bus_entries`, `get_buses`,
`get_check_report`, `get_component_replacement_plan`, `get_components`,
`get_connectivity_diagnostics`, `get_design_rules`, `get_hierarchy`,
`get_labels`, `get_net_info`, `get_netlist`, `get_noconnects`,
`get_package`, `get_package_change_candidates`, `get_part`,
`get_part_change_candidates`, `get_ports`, `get_schematic_net_info`,
`get_schematic_summary`, `get_scoped_component_replacement_plan`,
`get_sheets`, `get_symbol_fields`, `get_symbols`, `get_unrouted`,
`open_project`, `redo`, `run_drc`, `run_erc`, `save`, `search_pool`,
`undo`, `validate_project`, `get_check_run`, `get_check_runs`,
`show_check_run`, `get_check_profiles`, `get_zone_fills`, `fill_zones`,
`generate_standards_repair_proposals`, `waive_finding`, `accept_deviation`,
`get_journal_list`, `get_journal_transaction`, `journal_undo`,
`journal_redo`, `get_panel_projections`, `create_panel_projection`,
`create_panel_projection_proposal`,
`update_panel_projection`,
`update_panel_projection_proposal`,
`delete_panel_projection`,
`delete_panel_projection_proposal`,
`get_manufacturing_plans`,
`create_manufacturing_plan`,
`create_manufacturing_plan_proposal`,
`update_manufacturing_plan`,
`update_manufacturing_plan_proposal`,
`delete_manufacturing_plan`,
`delete_manufacturing_plan_proposal`,
`get_output_jobs`,
`generate_artifacts`,
`get_artifacts`,
`show_artifact`,
`get_artifact_files`,
`preview_artifact_file`,
`compare_artifacts`,
`validate_artifact`,
`create_gerber_output_job`,
`create_output_job`,
`create_output_job_proposal`,
`update_output_job`,
`update_output_job_proposal`,
`run_output_job`,
`start_output_job_run`,
`cancel_output_job_run`,
`delete_output_job`,
`delete_output_job_proposal`,
`export_manufacturing_set`,
`validate_manufacturing_set`,
`get_component_instances`,
`bind_component_instance`,
`set_component_instance`,
`delete_component_instance`,
`get_pool_library_objects`,
`show_pool_library_object`,
`get_pool_model_blobs`,
`gc_pool_model_blobs`,
`create_pool_library_object`,
`create_pool_unit`,
`set_pool_unit_pin`,
`create_pool_symbol`,
`add_pool_symbol_line`,
`add_pool_symbol_rect`,
`add_pool_symbol_circle`,
`add_pool_symbol_polygon`,
`add_pool_symbol_arc`,
`add_pool_symbol_text`,
`set_pool_symbol_pin_anchor`,
`create_pool_entity`,
`create_pool_padstack`,
`create_pool_package`,
`create_pool_footprint`,
`set_pool_footprint_pad`,
`set_pool_footprint_courtyard_rect`,
`set_pool_footprint_courtyard_polygon`,
`add_pool_footprint_silkscreen_line`,
`add_pool_footprint_silkscreen_rect`,
`add_pool_footprint_silkscreen_circle`,
`add_pool_footprint_silkscreen_polygon`,
`set_pool_package_pad`,
`set_pool_package_courtyard_rect`,
`set_pool_package_courtyard_polygon`,
`add_pool_package_silkscreen_line`,
`add_pool_package_silkscreen_rect`,
`add_pool_package_silkscreen_circle`,
`add_pool_package_silkscreen_polygon`,
`add_pool_package_silkscreen_arc`,
`add_pool_package_silkscreen_text`,
`add_pool_package_model_3d`,
`set_pool_package_body_heights`,
`create_pool_part`,
`set_pool_part_metadata`,
`set_pool_part_parametric`,
`set_pool_part_orderable_mpns`,
`set_pool_part_tags`,
`set_pool_part_packaging_options`,
`set_pool_part_supply_chain`,
`set_pool_part_behavioural_models`,
`attach_pool_part_model`,
`detach_pool_part_model`,
`set_pool_part_thermal`,
`set_pool_part_pad_map_entry`,
`set_pool_part_pad_map`,
`create_pool_pin_pad_map`,
`set_pool_pin_pad_map`,
`set_pool_library_object`,
`delete_pool_library_object`,
`get_relationships`,
`get_variants`,
`get_import_map`,
`create_proposal`,
`create_draw_wire_proposal`,
`create_place_label_proposal`,
`create_place_symbol_proposal`,
`create_board_component_replacement_proposal`,
`create_board_component_replacements_proposal`,
`create_board_component_replacement_plan_proposal`,
`create_pool_pin_pad_map_proposal`,
`set_pool_pin_pad_map_proposal`,
`get_proposals`,
`show_proposal`,
`preview_proposal`,
`validate_proposal`,
`defer_proposal`,
`reject_proposal`,
`review_proposal`,
`accept_apply_proposal`,
`apply_proposal`,
`export_route_path_proposal`,
`route_proposal`,
`review_route_proposal`,
`route_proposal_explain`,
`route_strategy_report`,
`route_strategy_compare`,
`route_strategy_delta`,
`write_route_strategy_curated_fixture_suite`,
`capture_route_strategy_curated_baseline`,
`route_strategy_batch_evaluate`,
`inspect_route_strategy_batch_result`,
`validate_route_strategy_batch_result`,
`compare_route_strategy_batch_result`,
`gate_route_strategy_batch_result`,
`summarize_route_strategy_batch_results`,
`export_route_proposal`,
`route_apply`,
`route_apply_selected`,
`inspect_route_proposal_artifact`,
`revalidate_route_proposal_artifact`,
`apply_route_proposal_artifact`.

### Current Method Sections

These sections preserve per-method compatibility anchors required by drift
gates. They are not target tool-class definitions.

#### `open_project`

Context/session compatibility.

#### `close_project`

Context/session compatibility.

#### `save`

Legacy save compatibility; target commits at apply time.

Retired private-writer compatibility methods such as flat board component,
track, via, net-class, design-rule, and replacement apply writes are no longer
daemon-dispatched compatibility anchors. Their public replacements are the
canonical `datum.pcb.*`, `datum.proposal.*`, and journal-backed CLI paths
described below.

#### `place_zone` / `edit_zone` / `delete_zone`

Canonical native-project MCP aliases `datum.pcb.place_zone`,
`datum.pcb.edit_zone`, and `datum.pcb.delete_zone` require explicit project
`path` arguments and dispatch through journaled native CLI authoring via
`datum-eda project place-board-zone`, `edit-board-zone`, and
`delete-board-zone`. Zone filling remains a generated-evidence check surface
under `datum.check.fill_zones`.

#### `place_pad` / `edit_pad` / `delete_pad` / `set_pad_net` / `clear_pad_net`

Canonical native-project MCP aliases `datum.pcb.place_pad`, `edit_pad`,
`delete_pad`, `set_pad_net`, and `clear_pad_net` require explicit project
`path` arguments and dispatch through journaled native CLI authoring via the
matching `datum-eda project ...board-pad...` commands.

#### `place_net` / `edit_net` / `delete_net`

Canonical native-project MCP aliases `datum.pcb.place_net`, `edit_net`, and
`delete_net` require explicit project `path` arguments and dispatch through
journaled native CLI authoring via the matching `datum-eda project
...board-net` commands.
`datum.pcb.place_net` and `datum.pcb.edit_net` expose the same per-net
controlled-impedance metadata as the native CLI: `impedance_target_ohms`,
`impedance_tolerance_pct`, and `controlled_dielectric_layer`; `edit_net` also
accepts `clear_controlled_impedance` to remove the metadata. These map directly
to the matching `place-board-net` / `edit-board-net` flags.

#### `place_net_class` / `edit_net_class` / `delete_net_class`

Canonical native-project MCP aliases `datum.pcb.place_net_class`,
`edit_net_class`, and `delete_net_class` require explicit project `path`
arguments and dispatch through journaled native CLI authoring via the matching
`datum-eda project ...board-net-class` commands.

#### `set_board_name` / `set_outline` / `set_stackup` / `add_default_top_stackup`

Canonical native-project MCP aliases `datum.pcb.set_board_name`,
`set_outline`, `set_stackup`, and `add_default_top_stackup` require explicit
project `path` arguments and dispatch through journaled native CLI authoring via
the matching `datum-eda project set-board-name`, `set-board-outline`,
`set-board-stackup`, and `add-default-top-stackup` commands.
`datum.pcb.set_stackup.layers[]` accepts the same layer tuple syntax as the CLI:
`id:name:type:thickness_nm` for legacy callers or
`id:name:type:thickness_nm:dielectric_constant:loss_tangent:copper_weight_oz:roughness_um:material_name`
for material-aware callers. Empty optional material slots remain unset.

#### `place_keepout` / `edit_keepout` / `delete_keepout`

Canonical native-project MCP aliases `datum.pcb.place_keepout`,
`edit_keepout`, and `delete_keepout` require explicit project `path` arguments
and dispatch through journaled native CLI authoring via the matching
`datum-eda project ...board-keepout` commands.

#### `place_dimension` / `edit_dimension` / `delete_dimension`

Canonical native-project MCP aliases `datum.pcb.place_dimension`,
`edit_dimension`, and `delete_dimension` require explicit project `path`
arguments and dispatch through journaled native CLI authoring via the matching
`datum-eda project ...board-dimension` commands. `edit_dimension` maps
`clear_text=true` to the CLI `--clear-text` flag.

#### `place_text` / `edit_text` / `delete_text`

Canonical native-project MCP aliases `datum.pcb.place_text`, `edit_text`, and
`delete_text` require explicit project `path` arguments and dispatch through
journaled native CLI authoring via the matching `datum-eda project
...board-text` commands. `place_text` exposes the authored text, position,
layer, rotation, size, stroke, alignment, style, mirror/upright, line-spacing,
bold, and italic fields; `edit_text` exposes the same fields as optional
replacements using the CLI `--text <uuid>` target plus `--value` for content.

#### `create_sheet` / `delete_sheet` / `rename_sheet` / `create_sheet_definition` / `create_sheet_instance` / `delete_sheet_instance` / `move_sheet_instance` / `bind_sheet_instance_port` / `unbind_sheet_instance_port` / `draw_wire` / `delete_wire` / `place_junction` / `delete_junction` / `place_noconnect` / `delete_noconnect` / `place_label` / `rename_label` / `delete_label` / `place_port` / `edit_port` / `delete_port` / `create_bus` / `edit_bus_members` / `place_bus_entry` / `delete_bus_entry` / `place_text` / `edit_text` / `delete_text` / drawing aliases / symbol lifecycle aliases

Canonical native-project MCP aliases `datum.schematic.create_sheet`,
`delete_sheet`, `rename_sheet`, `create_sheet_definition`,
`create_sheet_instance`, `delete_sheet_instance`, `move_sheet_instance`,
`bind_sheet_instance_port`, `unbind_sheet_instance_port`, `draw_wire`,
`delete_wire`, `place_junction`, `delete_junction`, `place_noconnect`, and
`delete_noconnect` require explicit project `path` arguments and dispatch
through journaled native CLI authoring via the matching
`datum-eda project create-sheet`, `delete-sheet`, `rename-sheet`,
`create-sheet-definition`, `create-sheet-instance`, `delete-sheet-instance`,
`move-sheet-instance`, `bind-sheet-instance-port`, `unbind-sheet-instance-port`,
`draw-wire`, `delete-wire`, `place-junction`, `delete-junction`,
`place-noconnect`, and `delete-noconnect` commands. `delete_sheet` cascades the
sheet payload as one journaled sheet-delete operation, preserving undo/redo as
a whole-sheet restore rather than a sequence of private child writes.
`create_sheet_definition` creates the definition shard and updates the
schematic root definitions map as one journaled transaction.
`create_sheet_instance`, `delete_sheet_instance`, and `move_sheet_instance`
update the schematic root instances array as undoable journaled transactions.
`bind_sheet_instance_port` and `unbind_sheet_instance_port` update the
instance's persisted parent-sheet port list; path-aware hierarchy queries derive
links from those persisted bindings and matching child hierarchical labels.
Canonical label aliases
`datum.schematic.place_label`, `rename_label`, and `delete_label` dispatch
through the matching journaled `place-label`, `rename-label`, and
`delete-label` commands; `place_label` accepts `kind` as an optional enum-like
string matching the CLI value names. Port aliases `datum.schematic.place_port`,
`edit_port`, and `delete_port` dispatch through the matching journaled
`place-port`, `edit-port`, and `delete-port` commands; `edit_port` accepts
optional `name`, `direction`, `x_nm`, and `y_nm` replacements. Bus aliases
`datum.schematic.create_bus`, `edit_bus_members`, `place_bus_entry`, and
`delete_bus_entry` dispatch through the matching journaled `create-bus`,
`edit-bus-members`, `place-bus-entry`, and `delete-bus-entry` commands;
`create_bus` and `edit_bus_members` serialize `members[]` as repeated
`--member` flags, and `place_bus_entry` accepts optional `wire`. Schematic
text aliases `datum.schematic.place_text`, `edit_text`, and `delete_text`
dispatch through the matching journaled `place-schematic-text`,
`edit-schematic-text`, and `delete-schematic-text` commands; `edit_text`
accepts optional `value`, `x_nm`, `y_nm`, and `rotation_deg` replacements.
Drawing aliases `datum.schematic.place_drawing_line`, `place_drawing_rect`,
`place_drawing_circle`, `place_drawing_arc`, `edit_drawing_line`,
`edit_drawing_rect`, `edit_drawing_circle`, `edit_drawing_arc`, and
`delete_drawing` dispatch through the matching journaled drawing commands.
Symbol lifecycle aliases `datum.schematic.place_symbol`, `move_symbol`,
`rotate_symbol`, `mirror_symbol`, `delete_symbol`, `set_symbol_reference`,
`set_symbol_value`, `set_symbol_display_mode`,
`set_symbol_hidden_power_behavior`, `set_symbol_unit`, `clear_symbol_unit`,
`set_symbol_gate`, `clear_symbol_gate`, `set_symbol_entity`,
`clear_symbol_entity`, `set_symbol_part`, `clear_symbol_part`,
`set_symbol_lib_id`, `clear_symbol_lib_id`, `set_pin_override`,
`clear_pin_override`, `add_symbol_field`, `edit_symbol_field`, and
`delete_symbol_field` dispatch through the matching journaled symbol
commands.

#### `place_component`

Canonical native-project MCP alias `datum.pcb.place_component` requires
`path`, `part`, `package`, `reference`, `value`, `x_nm`, `y_nm`, and `layer`;
it dispatches through journaled native CLI authoring via
`datum-eda project place-board-component`.

Component delete, move, rotate, flip, reference, value, part, and package edits
are no longer flat daemon methods. Their canonical MCP aliases are
native-project scoped and dispatch through journaled CLI authoring via the
matching `datum-eda project ...board-component...` commands.

#### `lock_component` / `unlock_component`

Canonical native-project MCP aliases `datum.pcb.lock_component` and
`datum.pcb.unlock_component` require `path` and `component`; they dispatch
through journaled native CLI authoring via
`datum-eda project set-board-component-locked` and
`datum-eda project clear-board-component-locked`.

#### `edit_scoped_component_replacement_plan`

Per-method write compatibility; target routes through proposal/apply or policy-gated direct commit.

#### `undo`

Journal compatibility.

#### `redo`

Journal compatibility.

#### `search_pool`

Query compatibility.

#### `get_part`

Query compatibility.

#### `get_package`

Query compatibility.

#### `get_package_change_candidates`

Query compatibility.

#### `get_part_change_candidates`

Query compatibility.

#### `get_component_replacement_plan`

Query compatibility.

#### `get_scoped_component_replacement_plan`

Query compatibility.

#### `get_board_summary`

Query compatibility.

#### `get_components`

Query compatibility.

#### `get_netlist`

Query compatibility.

#### `get_schematic_summary`

Query compatibility.

#### `get_sheets`

Query compatibility.

#### `get_labels`

Query compatibility.

#### `get_symbols`

Query compatibility.

#### `get_symbol_fields`

Query compatibility.

#### `get_ports`

Query compatibility.

#### `get_buses`

Query compatibility.

#### `get_bus_entries`

Query compatibility.

#### `get_noconnects`

Query compatibility.

#### `get_hierarchy`

Query compatibility. Canonical `datum.query.hierarchy` accepts an optional
native project `path`; when present it dispatches through
`datum-eda project query <path> hierarchy`, otherwise it falls back to the
legacy open-session `get_hierarchy` compatibility method.

#### `get_net_info`

Query compatibility.

#### `get_unrouted`

Query compatibility.

#### `get_schematic_net_info`

Query compatibility.

#### `get_check_report`

Check compatibility.

#### `get_connectivity_diagnostics`

Query compatibility.

#### `get_design_rules`

Query compatibility.

#### `run_erc`

Check compatibility. Returns a live non-persisted `check_run_v1` envelope with
normalized ERC findings and raw legacy ERC findings under `raw_report.erc`.

#### `run_drc`

Check compatibility. Returns a live non-persisted `check_run_v1` envelope with
normalized DRC findings and the raw legacy DRC report under `raw_report.drc`.

#### `explain_violation`

Hidden check compatibility alias. The public canonical MCP tool is
`datum.check.explain_violation`. Both names dispatch to the same implementation
and accept `domain` plus preferred `fingerprint` or legacy `index`. Fingerprint
requests resolve against the current live CheckRun finding set; index remains
for older positional callers.

#### `validate_project`

Query/check compatibility until resolver-backed validation lands.

#### `get_check_run`

Check/evidence compatibility. Runs the native project check surface through the
CLI bridge and persists a resolver-owned `CheckRun` generated-evidence artifact.

#### `get_check_runs`

Check/evidence compatibility. Lists resolver-discovered persisted `CheckRun`
generated-evidence artifacts through the CLI bridge.

#### `show_check_run`

Check/evidence compatibility. Shows one resolver-discovered persisted
`CheckRun` generated-evidence artifact by UUID through the CLI bridge.

#### `get_check_profiles`

Check/profile compatibility. Lists the supported native-project CheckRun
profiles visible to agents. Current implementation exposes the single
`native-combined` compatibility profile. `get_check_run` / `datum.check.run`
accept optional `profile=native-combined` and reject unsupported profile ids
instead of silently substituting another profile.

#### `get_zone_fills`

Zone-fill query compatibility. Returns resolver-derived native board zone-fill
state so agents can distinguish authored zones from generated filled copper.

#### `fill_zones`

Zone-fill producer compatibility. Runs the canonical `datum-eda check
fill-zones` CLI bridge with optional `zone` and `net` filters. Current
implementation emits `ZoneFill{Filled}` only for closed, non-degenerate,
no-thermal zones on otherwise empty boards; harder cases persist
`ZoneFill{Unsupported}` generated evidence with explanatory provenance. The
CLI bridge commits journaled `SetZoneFill` generated-evidence operations, so
MCP-triggered fills are visible in journal show/list and participate in
undo/redo.

#### `generate_standards_repair_proposals`

Proposal/check compatibility. Generates draft repair proposals from persisted
standards/process-aperture, dimension-rule, and ZoneFill honesty findings.
Current repair families emit `SetBoardPad` proposals for pad mask/paste
aperture findings, direct geometry proposals for dimension-rule findings:
`SetBoardTrack` for `track_width_below_min` and `SetBoardVia` for
`via_hole_out_of_range` / `via_annular_below_min`, and `SetZoneFill` proposals
for supported `zone_fill_unfilled` / `zone_fill_stale` findings. Unsupported
ZoneFill cases remain findings without fake copper proposals. Target routes
resulting edits through reviewed proposals and `commit()`.

#### `waive_finding`

Check/journal compatibility. Authors a fingerprint-scoped native check finding
waiver through the CLI bridge and native project journal. Parameters are
`path`, `fingerprint`, `rationale`, and optional `created_by`. The canonical
CLI/MCP aliases are now `datum-eda check waive` and `datum.check.waive`; this
flat method remains as a compatibility name.

#### `accept_deviation`

Check/journal compatibility. Authors a fingerprint-scoped accepted deviation
through the CLI bridge and native project journal. Parameters are `path`,
`fingerprint`, `rationale`, and optional `accepted_by`. The canonical CLI/MCP
aliases are now `datum-eda check accept-deviation` and
`datum.check.accept_deviation`; this flat method remains as a compatibility
name.

#### `get_journal_list`

Journal compatibility. Returns the resolver-backed native project transaction
journal summary and undo/redo availability. The canonical aliases are
`datum-eda journal list` and `datum.journal.list`; this flat method remains as
a compatibility name.

#### `get_journal_transaction`

Journal compatibility. Returns one full native project transaction record by
transaction UUID. The canonical aliases are `datum-eda journal show` and
`datum.journal.show`; this flat method remains as a compatibility name.

#### `journal_undo`

Journal compatibility. Applies one native project undo through the resolver
journal as a compensating transaction. The canonical aliases are
`datum-eda journal undo` and `datum.journal.undo`; this flat method remains as
a compatibility name.

#### `journal_redo`

Journal compatibility. Applies one native project redo through the resolver
journal as a compensating transaction. The canonical aliases are
`datum-eda journal redo` and `datum.journal.redo`; this flat method remains as
a compatibility name.

#### `get_panel_projections`

PanelProjection query compatibility.

#### `create_panel_projection`

PanelProjection authoring compatibility for production-only panel targets.

#### `create_panel_projection_proposal`

PanelProjection proposal-authoring compatibility. Builds the same
`CreatePanelProjection` OperationBatch as `create_panel_projection`, persists a
draft proposal metadata, and does not create the panel shard until review/apply.

#### `update_panel_projection`

PanelProjection authoring compatibility. Updates one authored panel projection
through the CLI bridge and the journaled substrate path, preserving object
identity while bumping `object_revision`.

#### `update_panel_projection_proposal`

PanelProjection proposal-authoring compatibility. Builds the same
`SetPanelProjection` OperationBatch as `update_panel_projection`, persists a
draft proposal metadata, and does not mutate the panel shard until review/apply.

#### `delete_panel_projection`

PanelProjection authoring compatibility. Deletes one authored panel projection
through the CLI bridge and the journaled substrate path; deletion is refused
while a ManufacturingPlan still targets the panel projection.

#### `delete_panel_projection_proposal`

PanelProjection proposal-authoring compatibility. Builds the same
`DeletePanelProjection` OperationBatch as `delete_panel_projection`, persists a
draft proposal metadata, and does not remove the panel shard until review/apply.

#### `get_manufacturing_plans`

ManufacturingPlan query compatibility.

#### `create_manufacturing_plan`

ManufacturingPlan authoring compatibility; may link to a PanelProjection.

#### `create_manufacturing_plan_proposal`

ManufacturingPlan proposal-authoring compatibility. Builds the same
`CreateManufacturingPlan` OperationBatch as `create_manufacturing_plan`,
persists draft proposal metadata, and does not create the plan shard until
review/apply.

#### `update_manufacturing_plan`

ManufacturingPlan authoring compatibility. Updates one authored manufacturing
plan through the CLI bridge and the journaled substrate path, preserving object
identity while bumping `object_revision`.

#### `update_manufacturing_plan_proposal`

ManufacturingPlan proposal-authoring compatibility. Builds the same
`SetManufacturingPlan` OperationBatch as `update_manufacturing_plan`, persists a
draft proposal metadata, and does not mutate the plan shard until review/apply.

#### `delete_manufacturing_plan`

ManufacturingPlan authoring compatibility. Deletes one authored manufacturing
plan through the CLI bridge and the journaled substrate path; deletion is
refused while an OutputJob still references the manufacturing plan.

#### `delete_manufacturing_plan_proposal`

ManufacturingPlan proposal-authoring compatibility. Builds the same
`DeleteManufacturingPlan` OperationBatch as `delete_manufacturing_plan`,
persists draft proposal metadata, and does not remove the plan shard until
review/apply.

#### `get_output_jobs`

OutputJob/query compatibility.

#### `create_gerber_output_job`

OutputJob authoring compatibility; optional `manufacturing_plan` links the
Gerber job to ManufacturingPlan intent. Target routes through `OperationBatch`
and `commit()`.

#### `create_output_job`

OutputJob authoring compatibility for one or more comma-separated artifact
include scopes (`gerber-set`, `manufacturing-set`, `bom`, `pnp`, `drill`, or
`all`); optional
`manufacturing_plan` links the job to ManufacturingPlan intent. This routes
through the same journaled `OperationBatch` substrate path as the Gerber
compatibility command.

#### `create_output_job_proposal`

OutputJob proposal-authoring compatibility. Builds the same `CreateOutputJob`
OperationBatch as `create_output_job`, persists draft proposal metadata through
the generic proposal gateway, and does not create the OutputJob shard until the
proposal is accepted and applied.

#### `update_output_job`

OutputJob authoring compatibility. Updates one OutputJob name or
manufacturing-plan link through the CLI bridge and the journaled substrate path.

#### `update_output_job_proposal`

OutputJob proposal-authoring compatibility. Builds the same `SetOutputJob`
OperationBatch as `update_output_job`, persists draft proposal metadata through
the generic proposal gateway, and does not mutate the OutputJob shard until the
proposal is accepted and applied.

#### `run_output_job`

OutputJob execution compatibility. Executes one authored OutputJob by resolving
its stored include scope list, prefix, and preferred output directory, then routes
generation through the existing generated-evidence artifact path. A caller may
provide a one-shot `output_dir` override; otherwise the authored OutputJob
`output_dir` is used before the default generated-artifacts directory. Failed
generation attempts return structured `output_job_run_v1` JSON with `status:
failed` and are still persisted as generated evidence instead of being hidden as
transport errors. Each persisted run receives a monotonic per-job
`run_sequence`; repeated executions of the same authored job produce distinct
run records, and output-job queries use `run_sequence` rather than UUID lexical
order to identify the latest run.
For generic multi-scope jobs, `run_output_job` persists one aggregate
`OutputJobRun` for the logical command rather than one run per generated
artifact; multi-artifact aggregate runs leave `artifact_id` null and report the
generated scopes in the run log and artifact report.

#### `start_output_job_run`

OutputJob execution-lifecycle compatibility. Persists a generated-evidence
`OutputJobRun` with `status: running` for one authored OutputJob without
changing `ModelRevision`. The lifecycle record uses the same per-job
`run_sequence` ordering contract as completed or failed executions.

#### `cancel_output_job_run`

OutputJob execution-lifecycle compatibility. Marks one existing generated
evidence `OutputJobRun` as `status: canceled`, preserving the run id and making
the canceled state query-visible through `get_output_jobs`.

#### `delete_output_job`

OutputJob authoring compatibility. Deletes one authored OutputJob through the
CLI bridge and the journaled substrate path.

#### `delete_output_job_proposal`

OutputJob proposal-authoring compatibility. Builds the same `DeleteOutputJob`
OperationBatch as `delete_output_job`, persists draft proposal metadata through
the generic proposal gateway, and does not remove the OutputJob shard until the
proposal is accepted and applied.

#### `get_artifacts`

Generated-evidence compatibility. Lists resolver-discovered artifact metadata
and ad hoc `ArtifactRun` generated-evidence records for a native project through
the canonical artifact CLI bridge.

#### `generate_artifacts`

Generated-evidence compatibility. Generates supported derived production
artifacts for a native project through the canonical artifact CLI bridge.
Current include scopes are `gerber-set`, `manufacturing-set`, `bom`, `pnp`,
`drill`, and `all`. The same method can execute an authored `OutputJob` by
passing `output_job`, in which case direct include/prefix arguments are not
used and the response is the `output_job_run_v1` execution envelope.

#### `show_artifact`

Generated-evidence compatibility. Returns one resolver-discovered artifact
metadata record by artifact id through the canonical artifact CLI bridge,
including sequence-ordered ad hoc `ArtifactRun` history when the artifact was
generated outside an authored OutputJob.

#### `get_artifact_files`

Generated-evidence compatibility. Returns the generated file path/hash list and
production projection proof list for one resolver-discovered artifact through
the canonical artifact CLI bridge. It does not read or mutate output files.

#### `preview_artifact_file`

Generated-artifact preview compatibility. Verifies that a requested safe
relative file path belongs to the resolver-discovered artifact, reads it from
either the caller-supplied artifact directory or persisted
`ArtifactMetadata.output_dir`, checks the bytes against the artifact metadata
hash, and returns a read-only semantic preview envelope. Current preview
readers cover supported RS-274X Gerber inspection, Excellon drill tool/hit
inspection, bounded Gerber/Excellon preview primitives in nanometer
coordinates, and CSV row/header summaries for BOM, PnP, and drill tables.
Older artifact metadata without `output_dir` still requires an explicit
`artifact_dir` override.

#### `compare_artifacts`

Generated-evidence compatibility. Compares two resolver-discovered artifact
metadata records by kind, model revision, validation state, file path/hash set,
and production projection proof set through the canonical artifact CLI bridge.

#### `validate_artifact`

Generated-evidence compatibility. Validates one resolver-discovered artifact
metadata record by artifact id through the canonical artifact CLI bridge. This
checks project/model ownership, safe relative artifact paths, file hash fields,
projection hash fields, and the persisted validation state. It does not rehash
exported production files because current artifact metadata stores relative
artifact paths; byte-level validation remains on output-directory scoped
commands such as `validate_manufacturing_set`.

#### `export_manufacturing_set`

Manufacturing artifact compatibility. Exports the supported manufacturing set
through the canonical artifact CLI bridge and returns resolver-owned artifact
metadata, manifest path, and OutputJobRun evidence.

#### `validate_manufacturing_set`

Manufacturing artifact compatibility. Validates an exported manufacturing set
against current project state and persisted artifact file hashes; validation
failures still return structured JSON evidence for agent review.

#### `get_component_instances`

ComponentInstance query compatibility. Returns authored ComponentInstance
records plus resolver-bound symbol/package references for a native project.

#### `bind_component_instance`

ComponentInstance write compatibility. Creates a journaled binding between one
schematic symbol and one board package through the native project commit path.

#### `set_component_instance`

ComponentInstance write compatibility. Updates one journaled symbol/package
binding through the native project commit path.

#### `delete_component_instance`

ComponentInstance write compatibility. Deletes one journaled ComponentInstance
binding through the native project commit path.

#### `get_pool_library_objects`

Native pool-library query compatibility. Lists resolver-discovered
pool-library objects from project pool refs, with optional pool, kind, object,
and payload filters.

#### `show_pool_library_object`

Native pool-library query compatibility. Shows one resolver-discovered
pool-library object with its materialized payload.

#### `get_pool_model_blobs`

Native pool-library model-blob query compatibility. Lists
content-addressed `pool/models/<role>/<sha256>.<ext>` files, recomputes
SHA-256, reports deterministic model UUIDs, and includes discovered
`Part.behavioural_models` attachment references plus explicit `referenced` /
`orphaned` lifecycle flags. Canonical alias: `datum.library.pool_models`.

#### `gc_pool_model_blobs`

Native pool-library model-blob lifecycle compatibility. Invokes
`datum-eda project gc-pool-models` in dry-run mode unless `apply` is true and
reports the native `native_project_pool_model_gc_v1` plan/delete/skip contract.
The CLI remains the authority for conservative deletion rules: only orphaned
regular hash-matching blobs may be deleted; referenced blobs, hash-mismatched
blobs, non-regular files, and AMI bundle roles are preserved.

#### `create_pool_library_object`

Native pool-library authoring compatibility. Creates one pool-library object
through the native project commit path using CLI-computed
`<pool>/<kind>/<uuid>.json` object paths.

#### `create_pool_unit`

Native pool-library typed authoring compatibility. Creates one schema-versioned
native `Unit` shard through the same journaled pool-library commit path without
requiring callers to author raw JSON.

#### `set_pool_unit_pin`

Native pool-library typed authoring compatibility. Sets one `Unit.pins` entry,
validating a non-empty pin name, the canonical pin direction enum, and duplicate
pin identity before committing through the journaled pool-library set path.

#### `create_pool_symbol`

Native pool-library typed authoring compatibility. Creates one schema-versioned
native `Symbol` shard through the same journaled pool-library commit path and
rejects symbols whose referenced `Unit` is not present in the resolved model.

#### `add_pool_symbol_line`

Native pool-library typed authoring compatibility. Appends one validated
`Primitive::Line` entry to `Symbol.drawings`, rejecting zero-length lines and
non-positive stroke widths before committing through the journaled pool-library
set path. Canonical alias: `datum.library.add_symbol_line`.

#### `add_pool_symbol_rect`

Native pool-library typed authoring compatibility. Appends one validated
`Primitive::Rect` entry to `Symbol.drawings`, rejecting zero-area bounds and
non-positive stroke widths before committing through the journaled pool-library
set path. Canonical alias: `datum.library.add_symbol_rect`.

#### `add_pool_symbol_circle`

Native pool-library typed authoring compatibility. Appends one validated
`Primitive::Circle` entry to `Symbol.drawings`, rejecting non-positive radius
or stroke width before committing through the journaled pool-library set path.
Canonical alias: `datum.library.add_symbol_circle`.

#### `add_pool_symbol_polygon`

Native pool-library typed polygon/polyline authoring compatibility. Appends one
validated `Primitive::Polygon` entry to `Symbol.drawings`, persisting vertices,
`closed`, and stroke width. Vertices are supplied as `x,y;x,y;...`; malformed
vertices, non-positive stroke widths, fewer than two vertices, and closed
polygons with fewer than three vertices are rejected before committing through
the journaled pool-library set path. Canonical alias:
`datum.library.add_symbol_polygon`.

#### `add_pool_symbol_arc`

Native pool-library typed authoring compatibility. Appends one validated
`Primitive::Arc` entry to `Symbol.drawings`, rejecting non-positive radius
or stroke width while persisting center, radius, start angle, end angle, and
stroke width before committing through the journaled pool-library set path.
Canonical alias: `datum.library.add_symbol_arc`.

#### `add_pool_symbol_text`

Native pool-library typed authoring compatibility. Appends one validated
`Primitive::Text` entry to `Symbol.drawings`, rejecting blank text while
persisting authored position and rotation before committing through the
journaled pool-library set path. Canonical alias:
`datum.library.add_symbol_text`.

#### `set_pool_symbol_pin_anchor`

Native pool-library typed authoring compatibility. Sets or replaces one
`Symbol.pin_anchors` entry for a referenced unit pin, rejecting symbols whose
unit is missing and pins that are not present in that unit before committing
through the journaled pool-library set path. Canonical alias:
`datum.library.set_symbol_pin_anchor`.

#### `create_pool_entity`

Native pool-library typed authoring compatibility. Creates one schema-versioned
native `Entity` shard with an initial gate over an existing `Unit` and `Symbol`,
and rejects mismatched symbol/unit combinations before committing the shard.

#### `create_pool_padstack`

Native pool-library typed authoring compatibility. Creates one schema-versioned
native `Padstack` shard with optional circle/rect aperture geometry and optional
drill diameter through the journaled pool-library commit path.

#### `create_pool_package`

Native pool-library typed package authoring. Creates one schema-versioned
native `Package` body shard through the journaled pool-library commit path.
Optional `pad`/`padstack` inputs remain accepted as legacy land-pattern
compatibility input, but new land-pattern authoring should use first-class
`Footprint` tools.

#### `create_pool_footprint`

Native pool-library typed authoring compatibility. Creates one
schema-versioned native `Footprint` shard tied to an existing `Package` through
the journaled pool-library commit path. Canonical alias:
`datum.library.create_footprint`.

#### `set_pool_footprint_pad`

Native pool-library typed first-class `Footprint` authoring. Sets one `Footprint.pads`
entry, validating the referenced padstack, non-empty pad name, and positive
layer before committing through the journaled pool-library set path. Canonical
alias: `datum.library.set_footprint_pad`.

#### `set_pool_footprint_courtyard_rect`

Native pool-library typed first-class `Footprint` authoring. Sets a
rectangular `Footprint.courtyard` polygon from validated min/max nanometer
bounds before committing through the journaled pool-library set path.
Canonical alias: `datum.library.set_footprint_courtyard_rect`.

#### `set_pool_footprint_courtyard_polygon`

Native pool-library typed first-class `Footprint` authoring. Sets a closed
`Footprint.courtyard` polygon from `x,y;x,y;...` vertices before committing
through the journaled pool-library set path. Malformed vertices and polygons
with fewer than three vertices are rejected. Canonical alias:
`datum.library.set_footprint_courtyard_polygon`.

#### `add_pool_footprint_silkscreen_line`

Native pool-library typed first-class `Footprint` authoring. Appends one
`Primitive::Line` entry to `Footprint.silkscreen`, validating distinct
endpoints and positive stroke width before committing through the journaled
pool-library set path. Canonical alias:
`datum.library.add_footprint_silkscreen_line`.

#### `add_pool_footprint_silkscreen_rect`

Native pool-library typed first-class `Footprint` authoring. Appends one
`Primitive::Rect` entry to `Footprint.silkscreen`, validating nonzero bounds and
positive stroke width before committing through the journaled pool-library set
path. Canonical alias: `datum.library.add_footprint_silkscreen_rect`.

#### `add_pool_footprint_silkscreen_circle`

Native pool-library typed first-class `Footprint` authoring. Appends one
`Primitive::Circle` entry to `Footprint.silkscreen`, validating positive radius
and stroke width before committing through the journaled pool-library set path.
Canonical alias: `datum.library.add_footprint_silkscreen_circle`.

#### `add_pool_footprint_silkscreen_polygon`

Native pool-library typed first-class `Footprint` authoring. Appends one
`Primitive::Polygon` entry to `Footprint.silkscreen`, validating semicolon
separated `x,y` vertices, closed/open state, minimum vertex count, and positive
stroke width before committing through the journaled pool-library set path.
Canonical alias: `datum.library.add_footprint_silkscreen_polygon`.

#### `set_pool_package_pad`

Native pool-library typed authoring compatibility. Legacy package-named path
that requires exactly one first-class `Footprint` for the target `Package` in
the requested pool, then sets one `Footprint.pads` entry through the journaled
pool-library set path. The shim validates the referenced padstack, non-empty
pad name, positive layer, and duplicate pad identity on the target Footprint.
New land-pattern pads should be authored directly with `set_pool_footprint_pad`.

#### `set_pool_package_courtyard_rect`

Native pool-library typed authoring compatibility. Legacy package-named path
that requires exactly one first-class `Footprint` for the target `Package` in
the requested pool, then sets a rectangular `Footprint.courtyard` polygon from
validated min/max nanometer bounds through the journaled pool-library set path.

#### `set_pool_package_courtyard_polygon`

Native pool-library typed authoring compatibility. Legacy package-named path
that requires exactly one first-class `Footprint` for the target `Package` in
the requested pool, then sets a closed `Footprint.courtyard` polygon from
`x,y;x,y;...` vertices through the journaled pool-library set path. Malformed
vertices and polygons with fewer than three vertices are rejected. Canonical alias:
`datum.library.set_package_courtyard_polygon`.

#### `add_pool_package_silkscreen_line`

Native pool-library typed authoring compatibility. Legacy package-named path
that requires exactly one first-class `Footprint` for the target `Package` in
the requested pool, then appends one validated `Primitive::Line` entry to
`Footprint.silkscreen`, rejecting zero-length lines and non-positive stroke
widths before committing through the journaled pool-library set path.

#### `add_pool_package_silkscreen_rect`

Native pool-library typed authoring compatibility. Legacy package-named path
that requires exactly one first-class `Footprint` for the target `Package` in
the requested pool, then appends one validated `Primitive::Rect` entry to
`Footprint.silkscreen`, rejecting zero-area bounds and non-positive stroke
widths before committing through the journaled pool-library set path.

#### `add_pool_package_silkscreen_circle`

Native pool-library typed authoring compatibility. Legacy package-named path
that requires exactly one first-class `Footprint` for the target `Package` in
the requested pool, then appends one validated `Primitive::Circle` entry to
`Footprint.silkscreen`, rejecting non-positive radius or stroke width before
committing through the journaled pool-library set path.

#### `add_pool_package_silkscreen_polygon`

Native pool-library typed authoring compatibility. Legacy package-named path
that requires exactly one first-class `Footprint` for the target `Package` in
the requested pool, then appends one validated `Primitive::Polygon` entry to
`Footprint.silkscreen`, persisting vertices, `closed`, and stroke width.
Vertices are supplied as `x,y;x,y;...`; malformed vertices, non-positive stroke
widths, fewer than two vertices, and closed polygons with fewer than three
vertices are rejected before committing through the journaled pool-library set
path. Canonical alias:
`datum.library.add_package_silkscreen_polygon`.

#### `add_pool_package_silkscreen_arc`

Native pool-library typed authoring compatibility. Legacy package-named path
that requires exactly one first-class `Footprint` for the target `Package` in
the requested pool, then appends one validated `Primitive::Arc` entry to
`Footprint.silkscreen`, persisting center, radius, start angle, end angle, and
stroke width while rejecting non-positive radius or stroke width before
committing through the journaled pool-library set path.
Canonical alias: `datum.library.add_package_silkscreen_arc`.

#### `add_pool_package_silkscreen_text`

Native pool-library typed authoring compatibility. Legacy package-named path
that requires exactly one first-class `Footprint` for the target `Package` in
the requested pool, then appends one validated `Primitive::Text` entry to
`Footprint.silkscreen`, rejecting blank text while persisting position and
rotation before committing through the journaled pool-library set path.
Canonical alias:
`datum.library.add_package_silkscreen_text`.

#### `add_pool_package_model_3d`

Native pool-library typed authoring compatibility. Appends one `ModelRef` to
`Package.models_3d` from `--model-path`, typed `ModelFormat`, and typed
`Transform3D` fields or JSON, persisting the path, format, transform, and
empty provenance slot. Blank, absolute, traversal, or uninferable model paths,
unsupported formats, malformed transform JSON, and nonpositive scales are
rejected before committing through the journaled pool-library set path.
Canonical alias:
`datum.library.add_package_model_3d`.

#### `set_pool_package_body_heights`

Native pool-library typed authoring compatibility. Sets or clears
`Package.body_height_nm` and `Package.body_height_mounted_nm` through the
journaled pool-library set path, rejecting empty no-op requests and
nonpositive heights before committing. Canonical alias:
`datum.library.set_package_body_heights`.

#### `create_pool_part`

Native pool-library typed authoring compatibility. Creates one schema-versioned
native `Part` shard binding an existing `Entity` to an existing `Package` with
an initially empty `pad_map`.

#### `set_pool_part_metadata`

Native pool-library typed authoring compatibility. Updates supplied basic
`Part` metadata fields (`mpn`, `manufacturer`, `manufacturer_jep106`, `value`,
`description`, `datasheet`, and `lifecycle`) on an existing part while
preserving omitted fields. The lifecycle value is validated against the
canonical lifecycle enum, `manufacturer_jep106` is constrained to the valid
11-bit JEP106 range, and empty no-op requests are rejected before committing
through the journaled pool-library set path. Canonical alias:
`datum.library.set_part_metadata`.

#### `set_pool_part_parametric`

Native pool-library typed authoring compatibility. Sets `Part.parametric`
fields through `merge` or `replace` mode using repeatable `key=value`
entries, rejecting malformed entries, blank keys, duplicate request keys, and
unsupported modes before committing through the journaled pool-library set path.
Canonical alias: `datum.library.set_part_parametric`.

#### `set_pool_part_packaging_options`

Native pool-library typed authoring compatibility. Sets
`Part.packaging_options` through `merge` or `replace` mode using repeatable
JSON packaging option objects that match `PackagingKind` / `PackagingOption`
from `ENGINE_SPEC.md` §1.2. Semantic schema validation remains in the CLI /
engine path; MCP forwards option payload strings without rewriting them.
Canonical alias: `datum.library.set_part_packaging_options`.

#### `set_pool_part_supply_chain`

Native pool-library typed authoring compatibility. Sets or clears the derived
`Part.supply_chain_offers` cache and `Part.last_supply_chain_check` timestamp
through the journaled pool-library set path. MCP forwards repeatable
`SupplyOffer` JSON payload strings and the checked-at timestamp string to the
CLI/engine path for schema and quality validation. Canonical alias:
`datum.library.set_part_supply_chain`.

#### `set_pool_part_behavioural_models`

Native pool-library typed authoring compatibility. Sets
`Part.behavioural_models` through `merge` or `replace` mode using repeatable
JSON `ModelAttachment` objects. Semantic schema validation remains in the CLI /
engine path; MCP forwards attachment payload strings without rewriting them.
This is the metadata-list substrate for model attachments, not yet the
file-copying attach/detach operation. Canonical alias:
`datum.library.set_part_behavioural_models`.

#### `attach_pool_part_model`

Native pool-library typed authoring compatibility. Reads a vendor behavioural
model file, stores it under the content-addressed `pool/models/<role>/`
directory, creates deterministic model / attachment UUIDs from the model hash
and target part, and appends the resulting `ModelAttachment` to
`Part.behavioural_models` through the journaled pool-library set path. Undo
currently reverts the part attachment list while retaining the content-addressed
pool blob through typed `DetachPoolPartModel` inverse operations; full blob
lifecycle cleanup remains pending. Canonical alias:
`datum.library.attach_part_model`.

#### `detach_pool_part_model`

Native pool-library typed authoring compatibility. Removes behavioural model
attachments from `Part.behavioural_models` by exact attachment UUID or model
UUID through the same journaled pool-library set path used by attach. Undo
restores the removed attachment list entry while retaining any
content-addressed `pool/models` blob through typed `AttachPoolPartModel`
inverse operations; blob garbage collection remains pending. Canonical alias:
`datum.library.detach_part_model`.

The native CLI read side now exposes `project query <root> pool-models` for
content-addressed model blob verification and attachment-reference discovery.
Native project validation also fails provenance-backed behavioural-model
attachments when the referenced blob is missing, the content-addressed filename
does not match the file bytes, or the attachment model UUID diverges from the
deterministic UUID derived from the referenced SHA-256.
MCP now exposes flat `get_pool_model_blobs` and canonical
`datum.library.pool_models` aliases over the same CLI verification surface;
flat `gc_pool_model_blobs` exposes the matching conservative CLI cleanup
action.

#### `set_pool_part_thermal`

Native pool-library typed authoring compatibility. Sets or clears
`Part.thermal` through the journaled pool-library set path. Supplied thermal
numeric fields are forwarded to the CLI as JSON-number strings for validation
against `ThermalSpec`; `--clear` removes the thermal object before any supplied
replacement fields are applied. Canonical alias:
`datum.library.set_part_thermal`.

#### `set_pool_part_orderable_mpns`

Native pool-library typed authoring compatibility. Sets `Part.orderable_mpns`
in `merge` or full `replace` mode from repeatable MPN values, rejecting blank
values, duplicate request MPNs, and unsupported modes before committing through
the journaled pool-library set path. Canonical alias:
`datum.library.set_part_orderable_mpns`.

#### `set_pool_part_tags`

Native pool-library typed authoring compatibility. Sets `Part.tags` in `merge`
or full `replace` mode from repeatable tag values, rejecting blank values,
duplicate request tags, and unsupported modes before committing through the
journaled pool-library set path. Canonical alias:
`datum.library.set_part_tags`.

#### `set_pool_part_pad_map_entry`

Native pool-library typed authoring compatibility. Accepts one legacy-shaped
package pad / entity gate / unit pin request, requires the part to name a
`Part.default_pin_pad_map`, and bridges the update into that first-class
`pool/pin_pad_maps` record before committing through the journaled pool-library
set path. It does not write `Part.pad_map`.

#### `set_pool_part_pad_map`

Native pool-library typed authoring compatibility. Accepts multiple
legacy-shaped pad/gate/pin requests in `merge` or full `replace` mode, requires
the part to name a `Part.default_pin_pad_map`, validates every package pad,
entity gate, and gate unit pin, and writes the equivalent pin/pad mappings to
the first-class `pool/pin_pad_maps` record. It does not write `Part.pad_map`.

#### `create_pool_pin_pad_map`

Native pool-library typed authoring. Creates one first-class `PinPadMap`
record in `pool/pin_pad_maps` from repeatable pin/pad mappings and can bind it
as the part default in the same journaled commit with `set_default`.

#### `set_pool_pin_pad_map`

Native pool-library typed authoring. Updates one first-class `PinPadMap`
mapping set in `merge` or full `replace` mode through the journaled
pool-library set path.

#### `set_pool_library_object`

Native pool-library authoring compatibility. Replaces one pool-library object
through the native project commit path, preserving object identity, bumping the
object revision, and using the previous payload as the undo inverse.

#### `delete_pool_library_object`

Native pool-library authoring compatibility. Deletes one pool-library object
through the native project commit path using CLI-computed
`<pool>/<kind>/<uuid>.json` object paths.

#### `get_relationships`

Relationship query compatibility. Returns authored relationship records plus
resolver-derived relationship status for a native project.

#### `get_variants`

Variant query compatibility. Returns authored sparse variant overlays plus
resolver-derived population/applicability for a native project.

#### `get_import_map`

Import provenance query compatibility. Returns resolver-validated import-key
identity mappings for a native project without changing model revision.

#### `get_proposals`

Proposal query compatibility. Returns resolver-discovered proposal records for
a native project without applying or changing proposal status.

#### `create_proposal`

Proposal authoring compatibility. Creates draft proposal metadata from an
OperationBatch JSON file after stamping or validating the prepared model
revision guard and dry-running the batch for operation validation. It does not
mutate source design state.

#### `create_draw_wire_proposal`

Schematic authoring proposal compatibility. Creates draft proposal metadata
that contains a revision-guarded OperationBatch for drawing one schematic wire
on a sheet. It maps to `datum-eda proposal create-draw-wire` and returns the
`proposal_create_v1` contract with a `propose_draw_wire` action. It does not
mutate source design state until the proposal is accepted/applied through the
proposal gateway.

#### `create_place_label_proposal`

Schematic authoring proposal compatibility. Creates draft proposal metadata
that contains a revision-guarded OperationBatch for placing one schematic label
on a sheet. It maps to `datum-eda proposal create-place-label` and returns the
`proposal_create_v1` contract with a `propose_place_label` action. It does not
mutate source design state until the proposal is accepted/applied through the
proposal gateway.

#### `create_place_symbol_proposal`

Schematic authoring proposal compatibility. Creates draft proposal metadata
that contains a revision-guarded OperationBatch for placing one schematic
symbol, including optional pool-library materialization inputs. It maps to
`datum-eda proposal create-place-symbol` and returns the `proposal_create_v1`
contract with a `propose_place_symbol` action. It does not mutate source design
state until the proposal is accepted/applied through the proposal gateway.

#### `create_board_component_replacement_proposal`

Board authoring proposal compatibility. Creates draft proposal metadata that
contains a revision-guarded OperationBatch for replacing one native-project
board component package, part, and/or value. It maps to `datum-eda proposal
create-board-component-replacement` and returns the `proposal_create_v1`
contract with a `propose_board_component_replacement` action. It does not mutate
source design state until the proposal is accepted/applied through the proposal
gateway.

#### `create_board_component_replacements_proposal`

Board authoring proposal compatibility. Creates one draft proposal record that
contains a revision-guarded OperationBatch for replacing multiple
native-project board component package, part, and/or value sets. It maps to
`datum-eda proposal create-board-component-replacements` and returns the
`proposal_create_v1` contract with a `propose_board_component_replacement`
action. It does not mutate source design state until the proposal is
accepted/applied through the proposal gateway.

#### `create_board_component_replacement_plan_proposal`

Board authoring proposal compatibility. Creates one draft proposal record from
legacy replacement-plan shaped selections (`uuid`, optional `package_uuid`,
optional `part_uuid`, optional `value`) and maps them to the same
revision-guarded board component replacement OperationBatch used by native
proposal authoring. It maps to `datum-eda proposal
create-board-component-replacement-plan` and returns the `proposal_create_v1`
contract with a `propose_board_component_replacement` action. It does not
mutate source design state until the proposal is accepted/applied through the
proposal gateway.

#### `create_pool_pin_pad_map_proposal`

Native library authoring proposal compatibility. Creates draft proposal
metadata containing a revision-guarded OperationBatch for one first-class
`PinPadMap` object, with optional same-batch `Part.default_pin_pad_map`
binding. It maps to `datum-eda proposal create-pool-pin-pad-map` and returns
the `proposal_create_v1` contract with a
`create_pool_pin_pad_map_proposal` action. It does not mutate source design
state until the proposal is accepted/applied through the proposal gateway.

#### `set_pool_pin_pad_map_proposal`

Native library authoring proposal compatibility. Creates draft proposal
metadata containing a revision-guarded OperationBatch for updating one
first-class `PinPadMap` mapping table in `merge` or `replace` mode. It maps to
`datum-eda proposal set-pool-pin-pad-map` and returns the
`proposal_create_v1` contract with a `set_pool_pin_pad_map_proposal` action.
It does not mutate source design state until the proposal is accepted/applied
through the proposal gateway.

#### `show_proposal`

Proposal inspection compatibility. Returns one resolver-discovered proposal
record plus current validation state without applying or changing proposal
status.

#### `preview_proposal`

Proposal preview compatibility. Dry-runs one resolver-discovered proposal's
embedded OperationBatch on a cloned resolved model and returns
`proposal_preview_v1` with classified `CommitDiff`, affected objects, and
current validation state. It does not write source shards, generated evidence,
or proposal status.

#### `validate_proposal`

Proposal validation compatibility. Reports whether one persisted proposal was
prepared against the current model revision, whether its embedded batch carries
the prepared revision guard, and whether it can be applied now.

#### `defer_proposal`

Proposal review compatibility. Persists `deferred` review status for one draft
proposal record without applying it.

#### `reject_proposal`

Proposal review compatibility. Persists `rejected` review status for one draft
proposal record without applying it.

#### `review_proposal`

Proposal review compatibility. Persists `accepted`, `deferred`, or `rejected`
review status for one draft proposal record without applying it.

#### `accept_apply_proposal`

Proposal review/apply compatibility. Persists accepted review status for one
draft proposal record and then applies it through the generic revision-guarded
proposal gateway.

#### `apply_proposal`

Proposal apply compatibility. Applies one accepted persisted proposal through
the generic revision-guarded proposal gateway.

#### `export_route_path_proposal`

Artifact/query compatibility.

#### `route_proposal`

Proposal/query compatibility.

#### `review_route_proposal`

Proposal/query compatibility.

#### `route_proposal_explain`

Proposal/query compatibility.

#### `route_strategy_report`

Proposal/query compatibility.

#### `route_strategy_compare`

Proposal/query compatibility.

#### `route_strategy_delta`

Proposal/query compatibility.

#### `write_route_strategy_curated_fixture_suite`

Artifact/query compatibility.

#### `capture_route_strategy_curated_baseline`

Artifact/query compatibility.

#### `route_strategy_batch_evaluate`

Proposal/query compatibility.

#### `inspect_route_strategy_batch_result`

Artifact/query compatibility.

#### `validate_route_strategy_batch_result`

Artifact/query compatibility.

#### `compare_route_strategy_batch_result`

Artifact/query compatibility.

#### `gate_route_strategy_batch_result`

Artifact/query compatibility.

#### `summarize_route_strategy_batch_results`

Artifact/query compatibility.

#### `export_route_proposal`

Artifact/query compatibility.

#### `route_apply`

Apply compatibility; target routes through `OperationBatch` and `commit()`.

#### `route_apply_selected`

Apply compatibility; target routes through `OperationBatch` and `commit()`.

#### `inspect_route_proposal_artifact`

Artifact/query compatibility.

#### `revalidate_route_proposal_artifact`

Artifact/query compatibility.

#### `apply_route_proposal_artifact`

Apply compatibility; target routes through `OperationBatch` and `commit()`.

### Current Method Classification

| Current method family | Target class |
|-----------------------|--------------|
| `open_project`, `close_project` | Context/session compatibility |
| `save` | Legacy compatibility; no target public save |
| `search_pool`, `get_part`, `get_package`, `get_*summary`, `get_*info`, `get_*candidates`, schematic and board reads | Query |
| `get_check_report`, `get_check_run`, `get_zone_fills`, `fill_zones`, `run_erc`, `run_drc`, `explain_violation` | Check/derived-state compatibility |
| Replacement plan reads and edits | Query/proposal compatibility depending on mutation |
| `datum.check.repair_standards`, `datum.check.waive`, `datum.check.accept_deviation`, plus flat `generate_standards_repair_proposals`, `waive_finding`, `accept_deviation` | Check-derived proposal/waiver/deviation compatibility |
| `datum.journal.list`, `datum.journal.show`, `datum.journal.undo`, `datum.journal.redo`, plus flat `get_journal_list`, `get_journal_transaction`, `journal_undo`, `journal_redo`, `undo`, `redo` | Journal compatibility; undo/redo route through compensating `OperationBatch` + `commit()` |
| `get_panel_projections`, `create_panel_projection`, `create_panel_projection_proposal`, `update_panel_projection`, `update_panel_projection_proposal`, `delete_panel_projection`, `delete_panel_projection_proposal`, `get_manufacturing_plans`, `create_manufacturing_plan`, `create_manufacturing_plan_proposal`, `update_manufacturing_plan`, `update_manufacturing_plan_proposal`, `delete_manufacturing_plan`, `delete_manufacturing_plan_proposal`, `get_output_jobs`, `create_gerber_output_job`, `create_output_job`, `create_output_job_proposal`, `update_output_job`, `update_output_job_proposal`, `run_output_job`, `start_output_job_run`, `cancel_output_job_run`, `delete_output_job`, `delete_output_job_proposal`, `generate_artifacts`, `get_artifact_files`, `preview_artifact_file`, `export_manufacturing_set`, `validate_manufacturing_set` | Panel/manufacturing/OutputJob/artifact compatibility |
| `get_relationships`, `get_variants` | Native relationship/variant substrate compatibility |
| `get_import_map` | Native import provenance substrate compatibility |
| `create_proposal`, `create_draw_wire_proposal`, `create_place_label_proposal`, `create_place_symbol_proposal`, `create_board_component_replacement_proposal`, `create_board_component_replacements_proposal`, `create_board_component_replacement_plan_proposal`, `get_proposals` | Native proposal substrate compatibility |
| `show_proposal`, `preview_proposal`, `validate_proposal`, `defer_proposal`, `reject_proposal`, `review_proposal`, `accept_apply_proposal`, `apply_proposal` | Native proposal inspect/preview/validate/defer/reject/review/apply compatibility |
| `export_route_path_proposal`, `route_proposal`, `review_route_proposal`, `route_proposal_explain`, `route_strategy_*` | Proposal/query compatibility |
| Batch route strategy artifact methods | Artifact/query/check compatibility, depending on operation |
| `export_route_proposal`, `inspect_route_proposal_artifact`, `revalidate_route_proposal_artifact` | Artifact/proposal compatibility |
| `route_apply`, `route_apply_selected`, `apply_route_proposal_artifact` | Apply compatibility; target must route through `commit()` |
| `delete_*`, `move_component`, `rotate_component`, `set_*`, `assign_part`, package/replacement/rule writes | Per-method write compatibility; target must route through `OperationBatch` + `commit()` |
| `validate_project` | Query/check compatibility until resolver-backed validation lands |

### Current Transport Notes

Compatibility tool names may still return JSON-RPC numeric error codes plus
message text instead of the target envelope. Canonical `datum.*` `tools/call`
responses return the target success/error envelope at the MCP boundary. Current
legacy methods may depend on one MCP-server-held open project. Those behaviors
are compatibility behavior only.

## Error Codes

Target symbolic error codes include:

| Code | Meaning |
|------|---------|
| `no_project_context` | Project-backed operation could not resolve context |
| `project_not_found` | Project id or root cannot be resolved |
| `invalid_model_revision` | Requested model revision does not exist |
| `stale_model_revision` | Request/proposal was prepared against an old revision |
| `variant_not_found` | Requested variant cannot be resolved |
| `output_context_not_found` | Requested output/manufacturing context cannot be resolved |
| `not_found` | Referenced object does not exist |
| `invalid_operation` | Operation failed validation |
| `capability_denied` | Session lacks the required capability |
| `proposal_required` | Mutation must go through proposal review |
| `proposal_not_found` | Proposal id cannot be resolved |
| `proposal_conflict` | Proposal conflicts with current model state |
| `missing_acceptance` | Apply requires acceptance not present in request/proposal |
| `missing_revision_guard` | Proposal operation batch lacks the prepared model revision guard |
| `check_failed` | Required pre-apply check failed |
| `artifact_error` | Artifact generation, validation, or comparison failed |
| `journal_error` | Journal read/write/undo/redo failed |
| `engine_error` | Internal engine error |

Legacy codes such as `no_project_open`, `project_already_open`,
`import_error`, `net_not_found`, `component_not_found`, `part_not_found`,
`unsupported_scope`, `connectivity_error`, `erc_error`, `drc_error`, and
`export_error` may appear from current compatibility methods until aliases are
rewired to target envelopes.

## Implementation Invariants

- `datum-eda` is the canonical CLI executable name. Bare `eda` and bare
  `datum` CLI names are legacy/noncanonical.
- `datum.*` is the canonical MCP namespace.
- There is one operation vocabulary, one proposal lifecycle, one commit/apply
  gateway, one artifact projection contract, and one project-wide journal.
- The MCP server must not mutate files, shards, caches, or journals directly.
- Derived outputs and check findings are never source authority.
- Selection/cursor/editor state is context/query data, not an operation.
- Compatibility methods must not expand the public surface; they are migration
  shims for existing clients.

## Open Implementation Questions

- Exact `ProjectResolver` request fields and lookup precedence for
  `project_id` vs `project_root` need an implementation-level schema.
- Direct-commit-by-policy capability boundaries need owner approval before any
  assistant/script path can bypass proposal review.
- Alias deprecation timing is not defined here; it needs a migration plan after
  target `datum.*` methods exist.
- Target `OperationBatch` variant inventory belongs in the domain contracts and
  still needs to be reconciled with current per-method writes.
