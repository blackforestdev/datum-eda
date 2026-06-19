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

The target public surface has exactly seven shared tool classes:

| Class | CLI shape | MCP shape | Mutates source? |
|-------|-----------|-----------|-----------------|
| Context / session | `datum-eda context get|refresh` | `datum.context.get`, `datum.context.refresh` | No |
| Query | `datum-eda query <family>` | `datum.query.*` | No |
| Check | `datum-eda check run|show` | `datum.check.*` | No source mutation |
| Proposal | `datum-eda proposal create|show|validate|reject|defer|apply` | `datum.proposal.*` | Apply only |
| Commit / apply gateway | internal `commit()` | reached through `datum.proposal.apply` or policy-gated direct commit | Yes |
| Artifact | `datum-eda artifact generate|show|compare|validate` | `datum.artifact.*` | No |
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

CLI:

- `datum-eda context get`
- `datum-eda context refresh`

Returns the `DatumContextEnvelope`: session id, actor, capabilities, project
identity, model revision, variant, output context, projection/selection/cursor
context, visible findings/artifacts, provenance seed, and expiry/refresh
metadata.

Context/session is capability state. It is not an `Operation`, is not
journaled, and must not grant direct filesystem write access.

### 2. Query

MCP:

- `datum.query.*`

CLI:

- `datum-eda query <family> [--domain ...] [--at <model_revision>]`

Queries are read-only. Existing `get_*`, `search_*`, report, relationship,
pool, schematic, PCB, rules, manufacturing, artifact, transaction, and
provenance reads fold into query families rather than new peer methods.

Target query responses must be revision-pinned and must prefer stable IDs over
names. Cross-domain joins use product-mechanics identity such as
`ComponentInstance`; agents must not reconstruct joins by refdes/name/path
string matching.

### 3. Check

MCP:

- `datum.check.run`
- `datum.check.show`

CLI:

- `datum-eda check run --domain <erc|drc|standards|process|manufacturing|all> --profile <id>`
- `datum-eda check show <check_run_id>`

Checks do not mutate source design state. Persisted `CheckRun` records are
derived evidence with revision, variant, checker version, profile, status, and
stable finding fingerprints. Finding explanations and suggested next actions
belong on `CheckFinding`; standalone index-addressed explain methods are
compatibility only.

### 4. Proposal

MCP:

- `datum.proposal.create`
- `datum.proposal.show`
- `datum.proposal.validate`
- `datum.proposal.reject`
- `datum.proposal.defer`
- `datum.proposal.apply`

CLI:

- `datum-eda proposal create|show|validate|reject|defer|apply`

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

- `datum.artifact.generate`
- `datum.artifact.show`
- `datum.artifact.compare`
- `datum.artifact.validate`

CLI:

- `datum-eda artifact generate --include <scope>[,<scope>...]`
- `datum-eda artifact show <artifact_id>`
- `datum-eda artifact compare <before> <after>`
- `datum-eda artifact validate <artifact_id>`

Artifacts are derived projections, never source authority and never committed
through `commit()`. Format and scope are parameters (`--include`), not separate
peer tool families. Panel/manufacturing/output-job context is represented in
`output_context`.

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
- `run_erc` -> `datum.check.run` with `domain = "erc"`
- `run_drc` -> `datum.check.run` with `domain = "drc"`

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

## Current Compatibility

This section is authoritative for the current implemented daemon/MCP method
names. It preserves implementation truth; it is not the target surface.

Current implementation note: implemented in the current daemon/stdio host.

### Current Implemented Methods (2026-03-25)

`open_project`, `close_project`, `save`, `delete_track`, `delete_via`,
`delete_component`, `move_component`, `rotate_component`, `set_value`,
`set_reference`, `assign_part`, `set_package`, `set_package_with_part`,
`replace_component`, `replace_components`, `apply_component_replacement_plan`,
`apply_component_replacement_policy`, `apply_scoped_component_replacement_policy`,
`apply_scoped_component_replacement_plan`, `edit_scoped_component_replacement_plan`,
`set_net_class`, `set_design_rule`, `undo`, `redo`, `search_pool`, `get_part`,
`get_package`,
`get_package_change_candidates`, `get_part_change_candidates`,
`get_component_replacement_plan`, `get_scoped_component_replacement_plan`,
`get_board_summary`, `get_components`,
`get_netlist`, `get_schematic_summary`, `get_sheets`, `get_labels`,
`get_symbols`, `get_symbol_fields`, `get_ports`, `get_buses`,
`get_bus_entries`, `get_noconnects`, `get_hierarchy`, `get_net_info`,
`get_unrouted`, `get_schematic_net_info`, `get_check_report`,
`get_connectivity_diagnostics`, `get_design_rules`, `run_erc`, `run_drc`,
`explain_violation`,
`validate_project`,
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

#### `delete_track`

Per-method write compatibility; target routes through proposal/apply or policy-gated direct commit.

#### `delete_via`

Per-method write compatibility; target routes through proposal/apply or policy-gated direct commit.

#### `delete_component`

Per-method write compatibility; target routes through proposal/apply or policy-gated direct commit.

#### `move_component`

Per-method write compatibility; target routes through proposal/apply or policy-gated direct commit.

#### `rotate_component`

Per-method write compatibility; target routes through proposal/apply or policy-gated direct commit.

#### `set_value`

Per-method write compatibility; target routes through proposal/apply or policy-gated direct commit.

#### `set_reference`

Per-method write compatibility; target routes through proposal/apply or policy-gated direct commit.

#### `assign_part`

Per-method write compatibility; target routes through proposal/apply or policy-gated direct commit.

#### `set_package`

Per-method write compatibility; target routes through proposal/apply or policy-gated direct commit.

#### `set_package_with_part`

Per-method write compatibility; target routes through proposal/apply or policy-gated direct commit.

#### `replace_component`

Per-method write compatibility; target routes through proposal/apply or policy-gated direct commit.

#### `replace_components`

Per-method write compatibility; target routes through proposal/apply or policy-gated direct commit.

#### `apply_component_replacement_plan`

Per-method write compatibility; target routes through proposal/apply or policy-gated direct commit.

#### `apply_component_replacement_policy`

Per-method write compatibility; target routes through proposal/apply or policy-gated direct commit.

#### `apply_scoped_component_replacement_policy`

Per-method write compatibility; target routes through proposal/apply or policy-gated direct commit.

#### `apply_scoped_component_replacement_plan`

Per-method write compatibility; target routes through proposal/apply or policy-gated direct commit.

#### `edit_scoped_component_replacement_plan`

Per-method write compatibility; target routes through proposal/apply or policy-gated direct commit.

#### `set_net_class`

Per-method write compatibility; target routes through proposal/apply or policy-gated direct commit.

#### `set_design_rule`

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

Query compatibility.

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

Check compatibility.

#### `run_drc`

Check compatibility.

#### `explain_violation`

Check compatibility.

#### `validate_project`

Query/check compatibility until resolver-backed validation lands.

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
| `get_check_report`, `run_erc`, `run_drc`, `explain_violation` | Check compatibility |
| Replacement plan reads and edits | Query/proposal compatibility depending on mutation |
| `export_route_path_proposal`, `route_proposal`, `review_route_proposal`, `route_proposal_explain`, `route_strategy_*` | Proposal/query compatibility |
| Batch route strategy artifact methods | Artifact/query/check compatibility, depending on operation |
| `export_route_proposal`, `inspect_route_proposal_artifact`, `revalidate_route_proposal_artifact` | Artifact/proposal compatibility |
| `route_apply`, `route_apply_selected`, `apply_route_proposal_artifact` | Apply compatibility; target must route through `commit()` |
| `delete_*`, `move_component`, `rotate_component`, `set_*`, `assign_part`, package/replacement/rule writes | Per-method write compatibility; target must route through `OperationBatch` + `commit()` |
| `undo`, `redo` | Journal compatibility; target must route through compensating `OperationBatch` + `commit()` |
| `validate_project` | Query/check compatibility until resolver-backed validation lands |

### Current Transport Notes

The current daemon transport may return JSON-RPC numeric error codes plus
message text instead of the target envelope. Current methods may depend on one
MCP-server-held open project. Those behaviors are compatibility behavior only.

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
