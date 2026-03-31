# MCP API Specification

## Transport

```
MCP Client ←→ MCP Server (Python, stdio) ←→ Engine (Rust, JSON-RPC over Unix socket)
```

The MCP server is a thin translation layer. It holds a reference to one
open engine session. It does not cache state — all queries go to the engine.

## Contract Mode

This document tracks two layers:
- **Current implementation contract (authoritative for code today)**:
  daemon JSON-RPC methods that are implemented and exercised by MCP tests.
- **Target M2 contract (planned hardening)**:
  richer normalized envelopes and full tool catalog parity.

Unless explicitly marked as `Target M2`, method definitions in this document
describe the **current implementation contract**.

## Session Model

The MCP server is **stateful**: it holds one open project at a time.

```
Session lifecycle:
  1. open_project → engine loads design into memory
  2. [queries, checks — any number, any order]
  3. save → engine writes to disk (optional)
  4. close_project → engine releases memory

Current implementation includes read/check and current M3 write operations
with explicit close-project lifecycle control.
```

Operations are **not idempotent**. `move_component` applied twice moves
the component twice. The MCP server does not deduplicate.

Undo/redo operates on the engine's transaction stack.

---

## Error Schema

All errors use this structure:
```json
{
  "error": {
    "code": "net_not_found",
    "message": "Net 'VCC_5V' does not exist in the design",
    "context": {
      "available_nets": ["VCC_3V3", "GND", "VBUS"]
    }
  }
}
```

Implementation note: the current daemon transport returns JSON-RPC numeric
codes plus message text. Symbolic string codes in this section are the target
normalized MCP surface.

### Error Codes

| Code | Meaning |
|------|---------|
| `no_project_open` | Operation requires an open project |
| `project_already_open` | Must close current project first |
| `import_error` | File could not be parsed |
| `not_found` | Referenced object does not exist |
| `net_not_found` | Net name or UUID not found |
| `component_not_found` | Component reference or UUID not found |
| `part_not_found` | Part UUID not in pool |
| `invalid_operation` | Operation failed validation |
| `unsupported_scope` | Rule uses expression node not yet implemented |
| `connectivity_error` | Connectivity graph could not be resolved safely |
| `erc_error` | ERC engine error (not a violation — an execution failure) |
| `drc_error` | DRC engine error (not a violation — an execution failure) |
| `export_error` | Export failed |
| `engine_error` | Internal engine error |

---

## M2 Tools (v1)

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

Methods in later `M4+` sections are target-state and are not part of the
current enforced daemon/MCP contract.

### Native CLI Parity Tracking (2026-03-29)

The native CLI/engine surface has advanced beyond the currently implemented
daemon/MCP contract. That is intentional for now: MCP parity is tracked here,
but deferred unless a milestone explicitly reopens MCP implementation work.

Parity policy for current development:
- native slices may land without same-slice MCP implementation
- new native contracts must be tracked here as one of:
  - `deferred_mcp_parity`
  - `implemented_in_mcp`

Native validation parity note (2026-03-30):
- `project validate <dir>` is now exposed through MCP as `validate_project`.
- MCP preserves the CLI validation contract exactly:
  - same JSON report shape
  - same valid/invalid semantics
  - invalid native projects may return CLI exit status `1`, but MCP still
    returns the structured validation payload
  - `not_planned_for_mcp`
- MCP implementation authority remains the current method list above; the
  entries below are tracking records, not claims of implemented MCP support

Currently tracked native contracts that are not implemented in MCP:

#### Implemented M5/M6 native parity exceptions

- `project export-route-path-proposal <dir> --net <uuid> --from-anchor <pad_uuid> --to-anchor <pad_uuid> --candidate <accepted_candidate> [--policy <policy>] --out <path>`
  - exposed in MCP as `export_route_path_proposal`
  - orthogonal-graph export responses include recorded segment-level
    ranked-path evidence
- `project route-proposal <dir> --net <uuid> --from-anchor <pad_uuid> --to-anchor <pad_uuid> [--profile <profile>]`
  - exposed in MCP as `route_proposal`
- `project route-strategy-report <dir> --net <uuid> --from-anchor <pad_uuid> --to-anchor <pad_uuid> [--objective <objective>]`
  - exposed in MCP as `route_strategy_report`
  - accepted objective set currently reuses the selector profile vocabulary:
    - `default`
    - `authored-copper-priority`
- `project route-strategy-compare <dir> --net <uuid> --from-anchor <pad_uuid> --to-anchor <pad_uuid>`
  - exposed in MCP as `route_strategy_compare`
  - compares only the accepted objective/profile set above and recommends one
    profile under a deterministic baseline-preserving rule
- `project route-strategy-delta <dir> --net <uuid> --from-anchor <pad_uuid> --to-anchor <pad_uuid>`
  - exposed in MCP as `route_strategy_delta`
  - compares only the same accepted objective/profile set above and emits one
    bounded explicit delta classification:
    - `same_outcome`
    - `different_candidate_family`
    - `different_policy_same_family`
    - `proposal_available_only_under_authored_copper_priority`
    - `no_proposal_under_any_profile`
- `project write-route-strategy-curated-fixture-suite --out-dir <path> [--manifest <path>]`
  - exposed in MCP as `write_route_strategy_curated_fixture_suite`
  - writes one deterministic curated native-project fixture suite plus a
    compatible `native_route_strategy_batch_requests` manifest for repeated
    evidence gathering
  - the current curated suite covers:
    - same-outcome baseline route selection
    - profile divergence between `default` and
      `authored-copper-priority`
    - no-proposal-under-any-profile
    - one cross-layer routable same-outcome case
- `project capture-route-strategy-curated-baseline --out-dir <path> [--manifest <path>] [--result <path>]`
  - exposed in MCP as `capture_route_strategy_curated_baseline`
  - materializes the curated fixture suite, runs the existing batch evaluator,
    and saves one reusable `native_route_strategy_batch_result_artifact`
    baseline file
  - default saved paths are inside `--out-dir`:
    - `route-strategy-batch-requests.json`
    - `route-strategy-batch-result.json`
  - the repo currently checks in one baseline capture under:
    - `crates/test-harness/testdata/quality/route_strategy_curated_baseline_v1`
  - the normal verification harness is:
    - `python3 scripts/check_route_strategy_evidence.py`
- `project route-strategy-batch-evaluate --requests <path>`
  - exposed in MCP as `route_strategy_batch_evaluate`
  - `--requests` points to one versioned JSON manifest of explicit route
    requests with:
    - `request_id`
    - `fixture_id`
    - `project_root`
    - `net_uuid`
    - `from_anchor_pad_uuid`
    - `to_anchor_pad_uuid`
  - the batch report reuses the existing `route_strategy_report`,
    `route_strategy_compare`, and `route_strategy_delta` logic per request and
    returns both per-request evidence and aggregate summary counts
  - the batch-evaluate JSON output is the saved artifact format with:
    - `kind = native_route_strategy_batch_result_artifact`
    - `version = 1`
- `project inspect-route-strategy-batch-result <path>`
  - exposed in MCP as `inspect_route_strategy_batch_result`
  - reports artifact identity/version, aggregate summary, recommendation and
    delta distributions, per-request outcomes, and malformed entries
- `project validate-route-strategy-batch-result <path>`
  - exposed in MCP as `validate_route_strategy_batch_result`
  - reports structural validity, version compatibility, missing required
    fields, summary/result count consistency, and malformed entries
- `project compare-route-strategy-batch-result <before> <after>`
  - exposed in MCP as `compare_route_strategy_batch_result`
  - compares saved artifacts only and treats them as compatible only when both
    use `kind = native_route_strategy_batch_result_artifact`, `version = 1`,
    and the same request-manifest kind/version
  - reports aggregate count deltas, added/removed/common request ids, and
    request-id keyed changes in recommended profile, delta classification, and
    selected live outcome with one bounded summary classification:
    - `identical`
    - `aggregate_only_changed`
    - `per_request_outcomes_changed`
    - `incompatible_artifacts`
- `project gate-route-strategy-batch-result <before> <after> [--policy <policy>]`
  - exposed in MCP as `gate_route_strategy_batch_result`
  - evaluates the saved-artifact comparison result only; it does not rerun any
    live strategy logic
  - accepted explicit gate policy set:
    - `strict_identical`
    - `allow_aggregate_only`
    - `fail_on_recommendation_change`
  - reports selected policy, pass/fail result, comparison classification,
    specific reasons, threshold/count facts, and summary counts of changed
    recommendations, changed delta classifications, and changed per-request
    outcomes
  - native CLI exit-code contract:
    - `0` when the selected gate policy passes
    - `2` when the selected gate policy fails
- `project summarize-route-strategy-batch-results [--dir <path> | --artifact <path> ...] [--baseline <path> --policy <policy>]`
  - exposed in MCP as `summarize_route_strategy_batch_results`
  - summarizes saved artifacts only; it does not rerun live route evaluation
  - accepts either one directory scan or an explicit artifact list
  - reports per-artifact identity/version, filesystem-derived run ordering
    when available, request counts, recommendation/delta distributions, and
    structural validation state
  - when `--baseline` is provided, attaches one optional baseline gate summary
    for each non-baseline artifact using the existing accepted gate policies
- `project route-proposal-explain <dir> --net <uuid> --from-anchor <pad_uuid> --to-anchor <pad_uuid> [--profile <profile>]`
  - exposed in MCP as `route_proposal_explain`
- `project export-route-proposal <dir> --net <uuid> --from-anchor <pad_uuid> --to-anchor <pad_uuid> --out <path> [--profile <profile>]`
  - exposed in MCP as `export_route_proposal`
- accepted selector profile set for those selector-backed surfaces:
  - `default`
  - `authored-copper-priority`
- `project route-apply <dir> --net <uuid> --from-anchor <pad_uuid> --to-anchor <pad_uuid> --candidate <accepted_candidate> [--policy <policy>]`
  - exposed in MCP as `route_apply`
- `project route-apply-selected <dir> --net <uuid> --from-anchor <pad_uuid> --to-anchor <pad_uuid> [--profile <profile>]`
  - exposed in MCP as `route_apply_selected`
- `project inspect-route-proposal-artifact <path>`
  - exposed in MCP as `inspect_route_proposal_artifact`
  - orthogonal-graph inspection responses include recorded segment-level
    ranked-path evidence
- `project revalidate-route-proposal-artifact <dir> --artifact <path>`
  - exposed in MCP as `revalidate_route_proposal_artifact`
  - orthogonal-graph revalidation responses include segment-level ranked-path
    evidence in addition to top-level drift classification
- `project apply-route-proposal-artifact <dir> --artifact <path>`
  - exposed in MCP as `apply_route_proposal_artifact`

#### Deferred M4 native parity

- `project query <dir> pools`
- `project query <dir> board-net --net <uuid>`
- `project query <dir> board-net-class --net-class <uuid>`
- `project query <dir> board-component-models-3d --component <uuid>`
- `project query <dir> board-component-pads --component <uuid>`
- `project query <dir> board-component-silkscreen --component <uuid>`
- `project query <dir> board-component-mechanical --component <uuid>`
- `project report-manufacturing <dir> [--prefix <text>]`
- `project export-manufacturing-set <dir> --output-dir <path> [--prefix <text>]`
- `project inspect-manufacturing-set <dir> --output-dir <path> [--prefix <text>]`
- `project validate-manufacturing-set <dir> --output-dir <path> [--prefix <text>]`
- `project compare-manufacturing-set <dir> --output-dir <path> [--prefix <text>]`
- `project manifest-manufacturing-set <dir> --output-dir <path> [--prefix <text>]`
- `project inspect-bom <path>`
- `project validate-bom <dir> --bom <path>`
- `project inspect-pnp <path>`
- `project validate-pnp <dir> --pnp <path>`
- `project inspect-drill <path>`
- `project validate-drill <dir> --drill <path>`
- `project compare-drill <dir> --drill <path>`
- `project export-gerber-set <dir> --output-dir <path> [--prefix <text>]`
- `project validate-gerber-set <dir> --output-dir <path> [--prefix <text>]`
- `project compare-gerber-set <dir> --output-dir <path> [--prefix <text>]`

#### Deferred M5 native parity

- `project query <dir> routing-substrate`
- `project query <dir> route-preflight --net <uuid>`
- `project query <dir> route-corridor --net <uuid>`
- `project query <dir> route-path-candidate --net <uuid> --from-anchor <pad_uuid> --to-anchor <pad_uuid>`
- same-layer `route-path-candidate-orthogonal-graph` responses now also
  include `segment_evidence`
- `project query <dir> route-path-candidate-explain --net <uuid> --from-anchor <pad_uuid> --to-anchor <pad_uuid>`
- same-layer `route-path-candidate-orthogonal-graph-explain` responses now
  also include `segment_evidence`
- `project query <dir> route-path-candidate-via --net <uuid> --from-anchor <pad_uuid> --to-anchor <pad_uuid>`
- `project query <dir> route-path-candidate-via-explain --net <uuid> --from-anchor <pad_uuid> --to-anchor <pad_uuid>`
- `project query <dir> route-path-candidate-two-via --net <uuid> --from-anchor <pad_uuid> --to-anchor <pad_uuid>`
- `project query <dir> route-path-candidate-two-via-explain --net <uuid> --from-anchor <pad_uuid> --to-anchor <pad_uuid>`
- `project query <dir> route-path-candidate-three-via --net <uuid> --from-anchor <pad_uuid> --to-anchor <pad_uuid>`
- `project query <dir> route-path-candidate-three-via-explain --net <uuid> --from-anchor <pad_uuid> --to-anchor <pad_uuid>`
- `project query <dir> route-path-candidate-four-via --net <uuid> --from-anchor <pad_uuid> --to-anchor <pad_uuid>`
- `project query <dir> route-path-candidate-four-via-explain --net <uuid> --from-anchor <pad_uuid> --to-anchor <pad_uuid>`
- `project query <dir> route-path-candidate-five-via --net <uuid> --from-anchor <pad_uuid> --to-anchor <pad_uuid>`
- `project query <dir> route-path-candidate-five-via-explain --net <uuid> --from-anchor <pad_uuid> --to-anchor <pad_uuid>`
- `project query <dir> route-path-candidate-six-via --net <uuid> --from-anchor <pad_uuid> --to-anchor <pad_uuid>`
- `project query <dir> route-path-candidate-six-via-explain --net <uuid> --from-anchor <pad_uuid> --to-anchor <pad_uuid>`
- `project query <dir> route-path-candidate-authored-via-chain --net <uuid> --from-anchor <pad_uuid> --to-anchor <pad_uuid>`
- `project query <dir> route-path-candidate-authored-via-chain-explain --net <uuid> --from-anchor <pad_uuid> --to-anchor <pad_uuid>`
- `project query <dir> route-path-candidate-orthogonal-dogleg --net <uuid> --from-anchor <pad_uuid> --to-anchor <pad_uuid>`
- `project query <dir> route-path-candidate-orthogonal-dogleg-explain --net <uuid> --from-anchor <pad_uuid> --to-anchor <pad_uuid>`
- `project query <dir> route-path-candidate-orthogonal-two-bend --net <uuid> --from-anchor <pad_uuid> --to-anchor <pad_uuid>`
- `project query <dir> route-path-candidate-orthogonal-two-bend-explain --net <uuid> --from-anchor <pad_uuid> --to-anchor <pad_uuid>`
- `project query <dir> route-path-candidate-orthogonal-graph --net <uuid> --from-anchor <pad_uuid> --to-anchor <pad_uuid>`
- `project query <dir> route-path-candidate-orthogonal-graph-explain --net <uuid> --from-anchor <pad_uuid> --to-anchor <pad_uuid>`
- `project query <dir> route-path-candidate-orthogonal-graph-via --net <uuid> --from-anchor <pad_uuid> --to-anchor <pad_uuid>`
- `project query <dir> route-path-candidate-orthogonal-graph-via-explain --net <uuid> --from-anchor <pad_uuid> --to-anchor <pad_uuid>`
- `project query <dir> route-path-candidate-orthogonal-graph-two-via --net <uuid> --from-anchor <pad_uuid> --to-anchor <pad_uuid>`
- `project query <dir> route-path-candidate-orthogonal-graph-two-via-explain --net <uuid> --from-anchor <pad_uuid> --to-anchor <pad_uuid>`
- `project query <dir> route-path-candidate-orthogonal-graph-three-via --net <uuid> --from-anchor <pad_uuid> --to-anchor <pad_uuid>`
- `project query <dir> route-path-candidate-orthogonal-graph-three-via-explain --net <uuid> --from-anchor <pad_uuid> --to-anchor <pad_uuid>`
- `project query <dir> route-path-candidate-orthogonal-graph-four-via --net <uuid> --from-anchor <pad_uuid> --to-anchor <pad_uuid>`
- `project query <dir> route-path-candidate-orthogonal-graph-four-via-explain --net <uuid> --from-anchor <pad_uuid> --to-anchor <pad_uuid>`
- `project query <dir> route-path-candidate-orthogonal-graph-five-via --net <uuid> --from-anchor <pad_uuid> --to-anchor <pad_uuid>`
- `project query <dir> route-path-candidate-orthogonal-graph-five-via-explain --net <uuid> --from-anchor <pad_uuid> --to-anchor <pad_uuid>`
- `project query <dir> route-path-candidate-orthogonal-graph-six-via --net <uuid> --from-anchor <pad_uuid> --to-anchor <pad_uuid>`
- `project query <dir> route-path-candidate-orthogonal-graph-six-via-explain --net <uuid> --from-anchor <pad_uuid> --to-anchor <pad_uuid>`
- `project query <dir> route-path-candidate-authored-copper-graph --net <uuid> --from-anchor <pad_uuid> --to-anchor <pad_uuid> --policy <policy>`
- `project query <dir> route-path-candidate-authored-copper-graph-explain --net <uuid> --from-anchor <pad_uuid> --to-anchor <pad_uuid> --policy <policy>`
- `project query <dir> route-path-candidate-authored-copper-graph-zone-aware --net <uuid> --from-anchor <pad_uuid> --to-anchor <pad_uuid>`
- `project query <dir> route-path-candidate-authored-copper-graph-zone-aware-explain --net <uuid> --from-anchor <pad_uuid> --to-anchor <pad_uuid>`
- `project query <dir> route-path-candidate-authored-copper-graph-zone-obstacle-aware --net <uuid> --from-anchor <pad_uuid> --to-anchor <pad_uuid>`
- `project query <dir> route-path-candidate-authored-copper-graph-zone-obstacle-aware-explain --net <uuid> --from-anchor <pad_uuid> --to-anchor <pad_uuid>`
- `project query <dir> route-path-candidate-authored-copper-graph-obstacle-aware --net <uuid> --from-anchor <pad_uuid> --to-anchor <pad_uuid>`
- `project query <dir> route-path-candidate-authored-copper-graph-obstacle-aware-explain --net <uuid> --from-anchor <pad_uuid> --to-anchor <pad_uuid>`
- `project export-route-path-proposal <dir> --net <uuid> --from-anchor <pad_uuid> --to-anchor <pad_uuid> --candidate authored-copper-plus-one-gap --out <path>`

### Project

#### `open_project`
```
Method: open_project
Input:  { "path": string }          // .kicad_pcb, .kicad_sch, .brd, .sch
Output: { "kind": string,
          "source": string,
          "counts": {
            "units": int,
            "symbols": int,
            "entities": int,
            "padstacks": int,
            "packages": int,
            "parts": int
          },
          "warnings": [string],
          "metadata": { key: string } }
Error:  import_error
```
Target M2 note: this may be normalized later into a richer project/session
envelope (project id, format split, board/schematic summary subobjects).

#### `close_project`
```
Method: close_project
Input:  {}
Output: { "ok": true }
Error:  no_project_open
```
Current implementation note: implemented in the current daemon/stdio host.

### Pool

#### `search_pool`
```
Method: search_pool
Input:  { "query": string,
          "type": "part" | "package" | "entity" | null,
          "limit": int (default 20) }
Output: { "results": [
            { "uuid": uuid, "type": "part",
              "mpn": string, "manufacturer": string,
              "description": string, "package": string }
          ] }
```
Current implementation note: implemented in the current daemon/stdio host
with keyword query-only parameters.

#### `get_part`
```
Method: get_part
Input:  { "uuid": uuid }
Output: { "uuid": uuid, "mpn": string, "manufacturer": string,
          "value": string, "description": string, "datasheet": string,
          "entity": { "name": string, "prefix": string,
                      "gates": [{ "name": string, "pins": [string] }] },
          "package": { "name": string, "pads": int },
          "parametric": { key: value, ... },
          "lifecycle": "active" | "nrnd" | "eol" | "obsolete" | "unknown" }
Error:  part_not_found
```
Current implementation note: implemented in the current daemon/stdio host.

#### `get_package`
```
Method: get_package
Input:  { "uuid": uuid }
Output: { "uuid": uuid, "name": string,
          "pads": [{ "name": string, "x_mm": float, "y_mm": float,
                     "layer": string }],
          "courtyard_mm": { "width": float, "height": float } }
Error:  not_found
```

#### `get_package_change_candidates`
```
Method: get_package_change_candidates
Input:  { "uuid": string }   // component UUID
Output: { "component_uuid": string,
          "current_part_uuid": string|null,
          "current_package_uuid": string,
          "current_package_name": string,
          "current_value": string,
          "status": "no_known_part"|"no_compatible_packages"|"candidates_available",
          "ambiguous_package_count": int,
          "candidates": [
            { "package_uuid": string,
              "package_name": string,
              "compatible_part_uuid": string,
              "compatible_part_value": string,
              "pin_names": [string] }
          ] }
Error:  component_not_found
```
Current implementation note: implemented in the current daemon/stdio host.

#### `get_part_change_candidates`
```
Method: get_part_change_candidates
Input:  { "uuid": string }   // component UUID
Output: { "component_uuid": string,
          "current_part_uuid": string|null,
          "current_package_uuid": string,
          "current_package_name": string,
          "current_value": string,
          "status": "no_known_part"|"no_compatible_parts"|"candidates_available",
          "candidates": [
            { "part_uuid": string,
              "package_uuid": string,
              "package_name": string,
              "value": string,
              "mpn": string,
              "manufacturer": string,
              "pin_names": [string] }
          ] }
Error:  component_not_found
```
Current implementation note: implemented in the current daemon/stdio host.

#### `get_component_replacement_plan`
```
Method: get_component_replacement_plan
Input:  { "uuid": string }   // component UUID
Output: { "component_uuid": string,
          "current_reference": string,
          "current_value": string,
          "current_part_uuid": string|null,
          "current_package_uuid": string,
          "current_package_name": string,
          "package_change": json,
          "part_change": json }
Error:  component_not_found
```
Current implementation note: implemented in the current daemon/stdio host.

#### `get_scoped_component_replacement_plan`
```
Method: get_scoped_component_replacement_plan
Input:  { "scope": {
            "reference_prefix": string|null,
            "value_equals": string|null,
            "current_package_uuid": string|null,
            "current_part_uuid": string|null
          },
          "policy": "best_compatible_package" | "best_compatible_part" }
Output: { "scope": json,
          "policy": string,
          "replacements": [
            { "component_uuid": string,
              "current_reference": string,
              "current_value": string,
              "current_part_uuid": string|null,
              "current_package_uuid": string,
              "target_part_uuid": string,
              "target_package_uuid": string,
              "target_value": string,
              "target_package_name": string }
          ] }
Error:  invalid_params | component_not_found | part_not_found | package_not_found
```
Current implementation note: implemented in the current daemon/stdio host as a read-only preview of the exact replacements a scoped policy would apply.

#### `edit_scoped_component_replacement_plan`
```
Method: edit_scoped_component_replacement_plan
Input:  { "plan": json,
          "exclude_component_uuids": [string],
          "overrides": [
            { "component_uuid": string,
              "target_package_uuid": string,
              "target_part_uuid": string }
          ] }
Output: { "scope": json,
          "policy": string,
          "replacements": [json] }
Error:  invalid_params
```
Current implementation note: implemented in the current daemon/stdio host for
plan post-processing (exclude/override) before apply.

#### `replace_components`
```
Method: replace_components
Input:  { "replacements": [
            { "uuid": string, "package_uuid": string, "part_uuid": string }
          ] }
Output: { "diff": { "created": [], "modified": [{ "object_type": "component", "uuid": string }], "deleted": [] },
          "description": string }
Error:  invalid_params | component_not_found | part_not_found | package_not_found
```
Current implementation note: implemented in the current daemon/stdio host as a single transaction / single undo step for batch component replacement.

#### `apply_component_replacement_plan`
```
Method: apply_component_replacement_plan
Input:  { "replacements": [
            { "uuid": string,
              "package_uuid": string|null,
              "part_uuid": string|null }
          ] }
Output: { "diff": { "created": [], "modified": [{ "object_type": "component", "uuid": string }], "deleted": [] },
          "description": string }
Error:  invalid_params | component_not_found | part_not_found | package_not_found
```
Current implementation note: implemented in the current daemon/stdio host for plan-driven replacement selection; callers may provide a package selector, a part selector, or both for each component.

#### `apply_component_replacement_policy`
```
Method: apply_component_replacement_policy
Input:  { "replacements": [
            { "uuid": string,
              "policy": "best_compatible_package" | "best_compatible_part" }
          ] }
Output: { "diff": { "created": [], "modified": [{ "object_type": "component", "uuid": string }], "deleted": [] },
          "description": string }
Error:  invalid_params | component_not_found | part_not_found | package_not_found
```
Current implementation note: implemented in the current daemon/stdio host for deterministic best-candidate replacement selection from the current replacement plan.

#### `apply_scoped_component_replacement_policy`
```
Method: apply_scoped_component_replacement_policy
Input:  { "scope": {
            "reference_prefix": string|null,
            "value_equals": string|null,
            "current_package_uuid": string|null,
            "current_part_uuid": string|null
          },
          "policy": "best_compatible_package" | "best_compatible_part" }
Output: { "diff": { "created": [], "modified": [{ "object_type": "component", "uuid": string }], "deleted": [] },
          "description": string }
Error:  invalid_params | component_not_found | part_not_found | package_not_found
```
Current implementation note: implemented in the current daemon/stdio host for deterministic best-candidate replacement selection over a scoped component filter.

#### `apply_scoped_component_replacement_plan`
```
Method: apply_scoped_component_replacement_plan
Input:  { "plan": {
            "scope": json,
            "policy": "best_compatible_package" | "best_compatible_part",
            "replacements": [
              { "component_uuid": string,
                "current_reference": string,
                "current_value": string,
                "current_part_uuid": string|null,
                "current_package_uuid": string,
                "target_part_uuid": string,
                "target_package_uuid": string,
                "target_value": string,
                "target_package_name": string }
            ]
          } }
Output: { "diff": { "created": [], "modified": [{ "object_type": "component", "uuid": string }], "deleted": [] },
          "description": string }
Error:  invalid_params | component_not_found | part_not_found | package_not_found
```
Current implementation note: implemented in the current daemon/stdio host to apply a previously previewed scoped replacement plan without re-resolving policy; the current CLI slice consumes this via `--apply-scoped-replacement-plan-file` and still requires matching pool libraries to be loaded.

### Design Queries

#### `get_schematic_summary`
```
Method: get_schematic_summary
Input:  {}
Output: { "sheets": int, "symbols": int, "nets": int,
          "labels": int, "ports": int, "buses": int }
Error:  no_project_open
```

#### `get_sheets`
```
Method: get_sheets
Input:  {}
Output: { "sheets": [
            { "uuid": uuid, "name": string, "symbols": int,
              "ports": int, "labels": int, "buses": int }
          ] }
Error:  no_project_open
```
Current implementation note: implemented in the current daemon/stdio host.

#### `get_symbols`
```
Method: get_symbols
Input:  { "sheet": uuid | null }
Output: { "symbols": [
            { "uuid": uuid, "reference": string, "value": string,
              "x_mm": float, "y_mm": float, "rotation_deg": float,
              "mirrored": bool, "part_uuid": uuid | null,
              "entity_uuid": uuid | null, "gate_uuid": uuid | null }
          ] }
Error:  no_project_open
```
Current implementation note: the current daemon/stdio host exposes the
all-sheets form only and passes `sheet = null` to the engine.

#### `get_ports`
```
Method: get_ports
Input:  { "sheet": uuid | null }
Output: { "ports": [
            { "uuid": uuid, "name": string, "direction": string,
              "x_mm": float, "y_mm": float }
          ] }
Error:  no_project_open
```
Current implementation note: the current daemon/stdio host exposes the
all-sheets form only and passes `sheet = null` to the engine.

#### `get_labels`
```
Method: get_labels
Input:  { "sheet": uuid | null }
Output: { "labels": [
            { "uuid": uuid, "sheet": uuid,
              "kind": "local" | "global" | "hierarchical" | "power",
              "name": string, "x_mm": float, "y_mm": float }
          ] }
Error:  no_project_open
```
Current implementation note: the current daemon/stdio host exposes the
all-sheets form only and passes `sheet = null` to the engine.

#### `get_buses`
```
Method: get_buses
Input:  { "sheet": uuid | null }
Output: { "buses": [
            { "uuid": uuid, "sheet": uuid, "name": string,
              "members": [string] }
          ] }
Error:  no_project_open
```
Current implementation note: the current daemon/stdio host exposes the
all-sheets form only and passes `sheet = null` to the engine.

#### `get_bus_entries`
```
Method: get_bus_entries
Input:  { "sheet": uuid | null }
Output: { "bus_entries": [
            { "uuid": uuid, "sheet": uuid, "bus_uuid": uuid,
              "wire_uuid": uuid | null, "x_mm": float, "y_mm": float }
          ] }
Error:  no_project_open
```
Current implementation note: the current daemon/stdio host exposes the
all-sheets form only and passes `sheet = null` to the engine.

#### `get_noconnects`
```
Method: get_noconnects
Input:  { "sheet": uuid | null }
Output: { "markers": [
            { "uuid": uuid, "sheet": uuid, "symbol_uuid": uuid,
              "pin_uuid": uuid, "x_mm": float, "y_mm": float }
          ] }
Error:  no_project_open
```
Current implementation note: the current daemon/stdio host exposes the
all-sheets form only and passes `sheet = null` to the engine.

#### `get_symbol_fields`
```
Method: get_symbol_fields
Input:  { "symbol_uuid": uuid }
Output: { "fields": [
            { "uuid": uuid, "key": string, "value": string,
              "visible": bool, "x_mm": float | null, "y_mm": float | null }
          ] }
Error:  no_project_open, not_found
```
Current implementation note: implemented in the current daemon/stdio host.

#### `get_hierarchy`
```
Method: get_hierarchy
Input:  {}
Output: { "instances": [
            { "uuid": uuid, "definition_uuid": uuid, "parent_sheet_uuid": uuid | null,
              "name": string, "x_mm": float, "y_mm": float }
          ],
          "links": [
            { "parent_sheet_uuid": uuid, "child_sheet_uuid": uuid,
              "parent_port_uuid": uuid, "child_port_uuid": uuid, "net_uuid": uuid }
          ] }
Error:  no_project_open
```
Current implementation note: implemented in the current daemon/stdio host.

#### `get_board_summary`
```
Method: get_board_summary
Input:  {}
Output: { "width_mm": float, "height_mm": float,
          "layers": int, "components": int, "nets": int,
          "tracks": int, "vias": int, "zones": int,
          "routed_pct": float, "unrouted_count": int }
Error:  no_project_open
```

#### `get_netlist`
```
Method: get_netlist
Input:  {}
Output: { "nets": [
            { "uuid": uuid, "name": string, "class": string | null,
              "pins": [{ "component": string, "pin": string }],
              "routed_pct": float | null,
              "labels": int | null, "ports": int | null,
              "sheets": [string] | null, "semantic_class": string | null }
          ] }
```
Current implementation note: implemented in the current daemon/stdio host.
Current payload is a canonical net inventory view:
- board projects: includes routing metrics (`routed_pct`) from board nets.
- schematic projects: includes schematic connectivity counts (`labels`, `ports`,
  `sheets`, `semantic_class`) with `routed_pct = null`.

#### `get_components`
```
Method: get_components
Input:  {}
Output: { "components": [
            { "uuid": uuid, "reference": string, "value": string,
              "part_mpn": string | null,
              "x_mm": float, "y_mm": float, "rotation_deg": float,
              "layer": "top" | "bottom", "locked": bool }
          ] }
```
Current implementation note: implemented in the current daemon/stdio host.

#### `get_net_info`
```
Method: get_net_info
Input:  { "net": string }           // name or UUID
Output: { "uuid": uuid, "name": string, "class": string,
          "pins": [{ "component": string, "pin": string }],
          "tracks": int, "vias": int,
          "routed_length_mm": float, "routed_pct": float }
Error:  net_not_found
```

This is the board-domain net query.
Current implementation note: the current daemon/stdio host exposes the full
board net inventory for the open design rather than a single-net selector.

#### `get_schematic_net_info`
```
Method: get_schematic_net_info
Input:  { "net": string }           // name or UUID
Output: { "uuid": uuid, "name": string, "class": string | null,
          "pins": [{ "component": string, "pin": string }],
          "labels": int, "ports": int, "sheets": [string],
          "semantic_class": string | null }
Error:  net_not_found
```

This is the schematic-domain net query. It is intentionally distinct from
`get_net_info` and must not expose board-routing metrics.
Current implementation note: the current daemon/stdio host exposes the full
schematic net inventory for the open design rather than a single-net selector.

#### `get_design_rules`
```
Method: get_design_rules
Input:  {}
Output: { "rules": [
            { "uuid": uuid, "name": string, "type": string,
              "scope": json, "priority": int, "enabled": bool,
              "parameters": json }
          ] }
```
Current implementation note: implemented in the current daemon/stdio host
using the current rule-evaluator subset payload. Later milestones may expand
projection richness, but the current method is part of the active M2 slice.

#### `get_unrouted`
```
Method: get_unrouted
Input:  {}
Output: { "airwires": [
            { "net_name": string,
              "from": { "component_ref": string, "pin_name": string },
              "to": { "component_ref": string, "pin_name": string },
              "distance_nm": int }
          ] }
```
Current implementation note: implemented in the current daemon/stdio host for
board projects. Target M2 may add a normalized `distance_mm` view while
retaining deterministic nm precision in engine-domain payloads.

#### `get_connectivity_diagnostics`
```
Method: get_connectivity_diagnostics
Input:  {}
Output: { "diagnostics": [
            { "kind": string, "severity": "error" | "warning" | "info",
              "message": string, "objects": [uuid] }
          ] }
Error:  no_project_open, connectivity_error
```
Current implementation note: implemented in the current daemon/stdio host.

#### `get_check_report`
```
Method: get_check_report
Input:  {}
Output:
  Board:
    { "domain": "board",
      "summary": { "status": "ok" | "info" | "warning" | "error",
                   "errors": int, "warnings": int, "infos": int, "waived": int,
                   "by_code": [{ "code": string, "count": int }] },
      "diagnostics": [
        { "kind": string, "severity": "error" | "warning" | "info",
          "message": string, "objects": [uuid] }
      ] }

  Schematic:
    { "domain": "schematic",
      "summary": { "status": "ok" | "info" | "warning" | "error",
                   "errors": int, "warnings": int, "infos": int, "waived": int,
                   "by_code": [{ "code": string, "count": int }] },
      "diagnostics": [
        { "kind": string, "severity": "error" | "warning" | "info",
          "message": string, "objects": [uuid] }
      ],
      "erc": [
        { "id": uuid,
          "code": string,
          "severity": "error" | "warning" | "info",
          "message": string,
          "net_name": string | null,
          "component": string | null,
          "pin": string | null,
          "objects": [{ "kind": string, "key": string }],
          "object_uuids": [uuid],
          "waived": bool }
      ] }
Error:  no_project_open, connectivity_error, erc_error
```
Current implementation note: implemented in the current daemon/stdio host.

### DRC

### ERC

#### `run_erc`
```
Method: run_erc
Input:  {}
Output: [
  { "id": uuid,
    "code": string,
    "severity": "error" | "warning" | "info",
    "message": string,
    "net_name": string | null,
    "component": string | null,
    "pin": string | null,
    "objects": [{ "kind": string, "key": string }],
    "object_uuids": [uuid],
    "waived": bool }
]
Error:  erc_error
```
Target M2 note: a wrapped summary envelope (`passed`, grouped counts) may be
added later, but current daemon/MCP contract is the raw ERC finding list.

#### `run_drc`
```
Method: run_drc
Input:  {}
Output: { "passed": bool,
          "violations": [
            { "id": uuid,
              "code": string,
              "rule_type": string,
              "severity": "error" | "warning",
              "message": string,
              "location": { "x_nm": int, "y_nm": int, "layer": int | null } | null,
              "objects": [uuid],
              "waived": bool }
          ],
          "summary": { "errors": int, "warnings": int, "waived": int } }
```
Current implementation note: implemented in the current daemon/stdio host.
Target M2 note: optional rule filtering and normalized unit views may be added
as the DRC catalog expands.

#### `explain_violation`
```
Method: explain_violation
Input:  { "domain": "erc" | "drc", "index": int }
Output: { "explanation": string,
          "rule_detail": string,
          "objects_involved": [{ "type": string, "uuid": uuid, "description": string }],
          "suggestion": string }
```
Current implementation note: implemented in the current daemon/stdio host.

---

## M3 Tools (Write Operations)

#### `move_component`
```
Method: move_component
Input:  { "uuid": uuid, "x_mm": float, "y_mm": float,
          "rotation_deg": float | null }
Output: { "diff": json }
Error:  component_not_found, invalid_operation
```
Current implementation note: implemented in the current daemon/stdio host.

#### `rotate_component`
```
Method: rotate_component
Input:  { "uuid": uuid, "rotation_deg": float }
Output: { "diff": json }
Error:  component_not_found, invalid_operation
```
Current implementation note: implemented in the current daemon/stdio host.

#### `set_value`
```
Method: set_value
Input:  { "uuid": uuid, "value": string }
Output: { "diff": json }
```
Current implementation note: implemented in the current daemon/stdio host.

#### `set_reference`
```
Method: set_reference
Input:  { "uuid": uuid, "reference": string }
Output: { "diff": json }
```
Current implementation note: implemented in the current daemon/stdio host.

#### `assign_part`
```
Method: assign_part
Input:  { "uuid": uuid, "part_uuid": uuid }
Output: { "diff": json }
Error:  component_not_found, part_not_found
```
Current implementation note: implemented in the current daemon/stdio host.

#### `set_package`
```
Method: set_package
Input:  { "uuid": uuid, "package_uuid": uuid }
Output: { "diff": json }
Error:  component_not_found, package_not_found, invalid_operation
```
Current implementation note: implemented in the current daemon/stdio host.

#### `set_package_with_part`
```
Method: set_package_with_part
Input:  { "uuid": uuid, "package_uuid": uuid, "part_uuid": uuid }
Output: { "diff": json }
Error:  component_not_found, package_not_found, part_not_found, invalid_operation
```
Current implementation note: implemented in the current daemon/stdio host.

#### `replace_component`
```
Method: replace_component
Input:  { "uuid": uuid, "package_uuid": uuid, "part_uuid": uuid }
Output: { "diff": json, "description": string }
Error:  component_not_found, part_not_found, package_not_found, invalid_operation
```
Current implementation note: implemented in the current daemon/stdio host.

#### `replace_components`
```
Method: replace_components
Input:  { "replacements": [
            { "uuid": uuid, "package_uuid": uuid, "part_uuid": uuid }
          ] }
Output: { "diff": json, "description": string }
Error:  component_not_found, part_not_found, package_not_found, invalid_operation
```
Current implementation note: implemented in the current daemon/stdio host as a
single transaction / single undo step.

#### `apply_component_replacement_plan`
```
Method: apply_component_replacement_plan
Input:  { "replacements": [
            { "uuid": uuid,
              "package_uuid": uuid | null,
              "part_uuid": uuid | null }
          ] }
Output: { "diff": json, "description": string }
Error:  component_not_found, part_not_found, package_not_found, invalid_operation
```
Current implementation note: implemented in the current daemon/stdio host.

#### `apply_scoped_component_replacement_policy`
```
Method: apply_scoped_component_replacement_policy
Input:  { "scope": {
            "reference_prefix": string | null,
            "value_equals": string | null,
            "current_package_uuid": uuid | null,
            "current_part_uuid": uuid | null
          },
          "policy": "best_compatible_package" | "best_compatible_part" }
Output: { "diff": json, "description": string }
Error:  component_not_found, part_not_found, package_not_found, invalid_operation
```
Current implementation note: implemented in the current daemon/stdio host.

#### `set_net_class`
```
Method: set_net_class
Input:  { "net_uuid": uuid,
          "class_name": string,
          "clearance": int,
          "track_width": int,
          "via_drill": int,
          "via_diameter": int,
          "diffpair_width": int | null,
          "diffpair_gap": int | null }
Output: { "diff": json }
Error:  net_not_found, invalid_operation
```
Current implementation note: implemented in the current daemon/stdio host.

#### `set_design_rule`
```
Method: set_design_rule
Input:  { "rule_type": string, "scope": json, "parameters": json,
          "priority": int, "name": string | null }
Output: { "rule_uuid": uuid, "diff": json }
```
Current implementation note: implemented in the current daemon/stdio host.

#### `delete_component`
```
Method: delete_component
Input:  { "uuid": uuid }
Output: { "diff": json }
Error:  not_found
```
Current implementation note: implemented in the current daemon/stdio host.

#### `delete_track`
```
Method: delete_track
Input:  { "uuid": uuid }
Output: { "diff": json }
Error:  not_found
```
Current implementation note: implemented in the current daemon/stdio host.

#### `delete_via`
```
Method: delete_via
Input:  { "uuid": uuid }
Output: { "diff": json }
Error:  not_found
```
Current implementation note: implemented in the current daemon/stdio host.

#### `undo`
```
Method: undo
Input:  {}
Output: { "diff": json, "description": string }
Error:  { "code": "nothing_to_undo" }
```
Current implementation note: implemented in the current daemon/stdio host.

#### `redo`
```
Method: redo
Input:  {}
Output: { "diff": json, "description": string }
Error:  { "code": "nothing_to_redo" }
```
Current implementation note: implemented in the current daemon/stdio host.

#### `save`
```
Method: save
Input:  { "path": string | null }    // null = save to original location
Output: { "path": string }
```
Current implementation note: implemented in the current daemon/stdio host.

---

## M4 Tools (Native Authoring + Export)

#### `place_symbol`
```
Method: place_symbol
Input:  { "sheet": uuid, "entity_uuid": uuid | null, "part_uuid": uuid | null,
          "reference": string, "x_mm": float, "y_mm": float,
          "rotation_deg": float, "mirrored": bool }
Output: { "symbol_uuid": uuid, "diff": json }
```

#### `move_symbol`
```
Method: move_symbol
Input:  { "uuid": uuid, "x_mm": float, "y_mm": float, "rotation_deg": float | null }
Output: { "diff": json }
```

#### `draw_wire`
```
Method: draw_wire
Input:  { "sheet": uuid, "from": { "x_mm": float, "y_mm": float },
          "to": { "x_mm": float, "y_mm": float } }
Output: { "wire_uuid": uuid, "diff": json }
```

#### `place_junction`
```
Method: place_junction
Input:  { "sheet": uuid, "x_mm": float, "y_mm": float }
Output: { "junction_uuid": uuid, "diff": json }
```

#### `place_label`
```
Method: place_label
Input:  { "sheet": uuid, "kind": "local" | "global" | "hierarchical" | "power",
          "name": string, "x_mm": float, "y_mm": float }
Output: { "label_uuid": uuid, "diff": json }
```

#### `create_sheet_instance`
```
Method: create_sheet_instance
Input:  { "parent_sheet": uuid | null, "name": string,
          "x_mm": float, "y_mm": float }
Output: { "sheet_instance_uuid": uuid, "sheet_uuid": uuid, "diff": json }
```

#### `place_hierarchical_port`
```
Method: place_hierarchical_port
Input:  { "sheet": uuid, "name": string, "direction": string,
          "x_mm": float, "y_mm": float }
Output: { "port_uuid": uuid, "diff": json }
```

#### `create_bus`
```
Method: create_bus
Input:  { "sheet": uuid, "name": string, "members": [string] }
Output: { "bus_uuid": uuid, "diff": json }
```

#### `place_noconnect`
```
Method: place_noconnect
Input:  { "sheet": uuid, "symbol_uuid": uuid, "pin_uuid": uuid,
          "x_mm": float, "y_mm": float }
Output: { "marker_uuid": uuid, "diff": json }
```

#### `set_field_value`
```
Method: set_field_value
Input:  { "symbol_uuid": uuid, "field": string, "value": string }
Output: { "diff": json }
```

#### `annotate`
```
Method: annotate
Input:  { "scope": "project" | "sheet", "sheet": uuid | null }
Output: { "diff": json, "renamed": [{ "old": string, "new": string }] }
```

#### `assign_gate`
```
Method: assign_gate
Input:  { "symbol_uuid": uuid, "gate_uuid": uuid }
Output: { "diff": json }
```

#### `rename_label`
```
Method: rename_label
Input:  { "uuid": uuid, "name": string }
Output: { "diff": json }
```

#### `edit_bus_members`
```
Method: edit_bus_members
Input:  { "uuid": uuid, "members": [string] }
Output: { "diff": json }
```

#### `place_bus_entry`
```
Method: place_bus_entry
Input:  { "sheet": uuid, "bus_uuid": uuid, "wire_uuid": uuid | null,
          "x_mm": float, "y_mm": float }
Output: { "bus_entry_uuid": uuid, "diff": json }
```

#### `move_field`
```
Method: move_field
Input:  { "field_uuid": uuid, "x_mm": float | null, "y_mm": float | null }
Output: { "diff": json }
```

#### `set_field_visibility`
```
Method: set_field_visibility
Input:  { "field_uuid": uuid, "visible": bool }
Output: { "diff": json }
```

#### `sync_schematic_to_board`
```
Method: sync_schematic_to_board
Input:  {}
Output: { "eco_id": uuid, "changes": [json] }
```

#### `export_gerber`
```
Method: export_gerber
Input:  { "output_dir": string, "layers": [string] }
Output: { "files": [string] }
Error:  export_error
```

#### `export_bom`
```
Method: export_bom
Input:  { "format": "csv" | "json", "output": string }
Output: { "path": string }
Error:  export_error
```

#### `export_drill`
```
Method: export_drill
Input:  { "output_dir": string }
Output: { "files": [string] }
Error:  export_error
```

#### `export_pnp`
```
Method: export_pnp
Input:  { "format": "csv", "output": string }
Output: { "path": string }
Error:  export_error
```

---

## M5-M6 Tools (Layout Engine)

#### `suggest_placement`
```
Method: suggest_placement
Input:  { "components": [string] | null,  // null = all unplaced
          "strategy": "minimize_wire_length" | "group_by_function" | null }
Output: { "proposal_id": uuid,
          "placements": [{ "reference": string, "x_mm": float, "y_mm": float,
                           "rotation_deg": float, "layer": string }],
          "score": float,
          "constraint_report": json,
          "warnings": [string] }
```

#### `route_net`
```
Method: route_net
Input:  { "net": string }
Output: { "proposal_id": uuid,
          "tracks": [json], "vias": [json],
          "length_mm": float, "constraint_report": json }
```

#### `accept_proposal`
```
Method: accept_proposal
Input:  { "proposal_id": uuid, "items": [uuid] | null }  // null = accept all
Output: { "diff": json }
```

#### `reject_proposal`
```
Method: reject_proposal
Input:  { "proposal_id": uuid }
Output: { "ok": true }
```

#### `analyze_layout`
```
Method: analyze_layout
Input:  {}
Output: { "score": float,
          "issues": [{ "severity": string, "message": string,
                       "location": json, "suggestion": string }] }
```
