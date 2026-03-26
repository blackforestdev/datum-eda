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
  3. (future M3+) save → engine writes to disk
  4. close_project → engine releases memory

Current slice is read/check-focused with explicit close-project lifecycle
control. Write operations and save remain deferred.
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

`open_project`, `close_project`, `search_pool`, `get_part`, `get_package`,
`get_board_summary`, `get_schematic_summary`, `get_sheets`, `get_symbols`,
`get_ports`, `get_labels`, `get_buses`, `get_bus_entries`, `get_noconnects`,
`get_symbol_fields`, `get_hierarchy`, `get_netlist`, `get_components`,
`get_net_info`, `get_schematic_net_info`, `get_connectivity_diagnostics`,
`get_design_rules`, `get_unrouted`, `get_check_report`, `run_erc`,
`run_drc`, `explain_violation`.

Methods listed later in this document without a current implementation note are
`Target M2+` and are not yet part of the enforced daemon/MCP contract.

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
Current implementation note: implemented in the current daemon/stdio host.

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
              "objects": [uuid] }
          ],
          "summary": { "errors": int, "warnings": int } }
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
Input:  { "reference": string, "x_mm": float, "y_mm": float,
          "rotation_deg": float | null }
Output: { "diff": json }
Error:  component_not_found, invalid_operation
```

#### `set_value`
```
Method: set_value
Input:  { "reference": string, "value": string }
Output: { "diff": json }
```

#### `set_reference`
```
Method: set_reference
Input:  { "reference": string, "new_reference": string }
Output: { "diff": json }
```

#### `assign_part`
```
Method: assign_part
Input:  { "reference": string, "part_uuid": uuid }
Output: { "diff": json }
Error:  component_not_found, part_not_found
```

#### `set_design_rule`
```
Method: set_design_rule
Input:  { "rule_type": string, "scope": json, "parameters": json,
          "priority": int, "name": string | null }
Output: { "rule_uuid": uuid, "diff": json }
```

#### `delete_track`
```
Method: delete_track
Input:  { "uuid": uuid }
Output: { "diff": json }
Error:  not_found
```

#### `undo`
```
Method: undo
Input:  {}
Output: { "diff": json, "description": string }
Error:  { "code": "nothing_to_undo" }
```

#### `redo`
```
Method: redo
Input:  {}
Output: { "diff": json, "description": string }
Error:  { "code": "nothing_to_redo" }
```

#### `save`
```
Method: save
Input:  { "path": string | null }    // null = save to original location
Output: { "path": string }
```

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
