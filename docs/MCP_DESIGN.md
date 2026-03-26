# MCP Server Design

> **Status**: Non-normative design rationale.  
> Controlling contracts are `specs/MCP_API_SPEC.md`,
> `specs/PROGRAM_SPEC.md`, and `specs/ENGINE_SPEC.md`.
> If this document conflicts with `specs/`, `specs/` win.
> Tool-availability notes below include historical planning text; current
> method availability is tracked in `specs/MCP_API_SPEC.md` and
> `specs/PROGRESS.md`.

## Overview

The MCP server provides AI agents with structured access to the design
engine. It is a Python process communicating with the Rust engine via
JSON-RPC over a Unix socket. All tools map 1:1 to engine API methods.

## Architecture

```
Claude Code / AI Agent
        │
        │ MCP Protocol (stdio)
        ▼
┌─────────────────────────────┐
│  mcp-server (Python)        │
│  • Translates MCP calls     │
│    to engine API calls      │
│  • Formats results for AI   │
│  • Stateful: holds open     │
│    project reference        │
└──────────────┬──────────────┘
               │ JSON-RPC / Unix socket
               ▼
┌─────────────────────────────┐
│  Engine (Rust process)      │
│  • Pool (SQLite + JSON)     │
│  • Design state (in memory) │
│  • Operations + DRC         │
│  • Import/export            │
└─────────────────────────────┘
```

## Configuration

```json
// ~/.claude/settings.json
{
  "mcpServers": {
    "eda": {
      "command": "python3",
      "args": ["/path/to/mcp-server/server.py"],
      "env": {
        "EDA_ENGINE_SOCKET": "/path/to/eda.sock"
      }
    }
  }
}
```

The current Python MCP process connects to `eda-engine-daemon` over the Unix
socket in `EDA_ENGINE_SOCKET`. It does not call the CLI binary directly.

---

## M2 Tools (v1: Analysis + Checking)

Available when: engine can import and query designs.

Representative implemented daemon/MCP subset for the current `M2` slice:
- `open_project`
- `get_board_summary`
- `get_schematic_summary`
- `get_sheets`
- `get_components`
- `get_net_info`
- `get_unrouted`
- `get_schematic_net_info`
- `get_labels`
- `get_symbols`
- `get_ports`
- `get_buses`
- `get_hierarchy`
- `get_noconnects`
- `get_connectivity_diagnostics`
- `get_check_report`
- `run_erc`
- `run_drc`

Methods that were previously staged later in planning but are now implemented
in the current slice include:
- `close_project`
- `search_pool`, `get_part`, `get_package`
- `get_netlist`, `get_design_rules`
- `explain_violation`

### Project Management

#### `open_project`
Import a KiCad or Eagle design.
```json
Input:  {"path": "/path/to/board.kicad_pcb"}
Output: {"kind": "kicad_board",
         "source": "/path/to/board.kicad_pcb",
         "counts": {...},
         "warnings": [],
         "metadata": {...}}
```

#### `close_project`
Close the currently open project.
Current `M2` availability is defined by `specs/MCP_API_SPEC.md` and
`specs/PROGRESS.md`; this method is implemented in the current slice.

### Pool Queries

#### `search_pool`
Search components by keyword, parametric value, or tag.
```json
Input:  {"query": "100nF 0402 ceramic", "limit": 20}
Output: [{"uuid": "...", "mpn": "GRM155R71C104KA88J",
          "manufacturer": "Murata", "description": "..."}]
```
Implementation: SQLite FTS on pool index.
Current `M2` availability is defined by `specs/MCP_API_SPEC.md` and
`specs/PROGRESS.md`; this method is implemented in the current slice.

#### `get_part`
Full part details — entity, package, pad map, parametrics.
```json
Input:  {"uuid": "..."}
Output: {"mpn": "...", "entity": {...}, "package": {...},
         "pad_map": {...}, "parametric": {"capacitance": "100nF"}}
```
Current `M2` availability is defined by `specs/MCP_API_SPEC.md` and
`specs/PROGRESS.md`; this method is implemented in the current slice.

#### `get_package`
Package geometry — pads, courtyard, silkscreen.
Current `M2` availability is defined by `specs/MCP_API_SPEC.md` and
`specs/PROGRESS.md`; this method is implemented in the current slice.

### Design Queries

Implemented now through the current daemon/stdio host:
- `get_board_summary`
- `get_schematic_summary`
- `get_components`
- `get_net_info`
- `get_unrouted`
- `get_schematic_net_info`
- `get_labels`
- `get_symbols`
- `get_ports`
- `get_buses`
- `get_hierarchy`
- `get_noconnects`
- `get_connectivity_diagnostics`
- `run_erc`
- `get_check_report`

Also implemented in the current slice:
- `get_netlist`
- `get_design_rules`
- `explain_violation`

#### `get_board_summary`
Board dimensions, layer count, component count, net count, routed %.
```json
Output: {"width_mm": 53.8, "height_mm": 37.5, "layers": 2,
         "components": 24, "nets": 18, "routed_pct": 100.0,
         "unrouted_count": 0}
```

#### `get_netlist`
Full connectivity — nets with connected component pins.
```json
Output: {"nets": [
  {"uuid": "...", "name": "VCC", "class": "power",
   "pins": [{"component": "U1", "pin": "VDD"},
            {"component": "C1", "pin": "1"}]},
  ...
]}
```
Current `M2` availability is defined by `specs/MCP_API_SPEC.md` and
`specs/PROGRESS.md`; this method is implemented in the current slice.

#### `get_components`
All placed components with reference, value, part, position, layer.

#### `get_net_info`
Board net inventory and routing metrics for the currently open board.
```json
Input:  {}
Output: [{"name": "VCC", "class": "power", "pins": 6,
          "tracks": 12, "vias": 2, "zones": 1,
          "routed_length_mm": 47.3}]
```

#### `get_unrouted`
Imported board airwire inventory for the currently open board.
```json
Input:  {}
Output: [{"net_name": "SIG",
          "from": {"component_ref": "R1", "pin_name": "1"},
          "to": {"component_ref": "R2", "pin_name": "1"},
          "distance_nm": 20000000}]
```

#### `get_schematic_summary`
Imported schematic summary for the open project.

#### `get_schematic_net_info`
Schematic net inventory for the currently open schematic.
```json
Input:  {}
Output: [{"name": "SCL", "labels": 1, "ports": 0, "pins": 1},
         {"name": "VCC", "labels": 1, "ports": 0, "pins": 0,
          "semantic_class": "power"}]
```

#### `get_labels`
Imported schematic labels for the open project.

#### `get_symbols`
Imported placed schematic symbols for the open project.

#### `get_ports`
Imported schematic interface ports for the open project.

#### `get_buses`
Imported schematic buses for the open project.

#### `get_hierarchy`
Imported schematic sheet-instance hierarchy for the open project.

#### `get_noconnects`
Imported no-connect markers for the open project.

#### `get_check_report`
Unified checking surface for imported designs. This is the MCP-facing
projection of the engine `CheckReport` type and should stay shape-aligned
with the engine and `specs/MCP_API_SPEC.md`.

- Board projects: structural connectivity diagnostics in the current
  `M1/M2` slice, including empty-copper, via-only, and partially-routed nets.
- Schematic projects: connectivity diagnostics plus ERC findings.
- Includes top-level summary status and grouped `by_code` counts so MCP
  consumers and CI agents can assess health without scanning every finding.

#### `get_connectivity_diagnostics`
Raw structural diagnostics for the currently open board or schematic.
Board diagnostics currently include empty-copper and partially-routed cases.

#### `run_erc`
Raw ERC findings for the currently open schematic.

#### `get_design_rules`
All configured rules with scopes and values.
Current `M2` availability is defined by `specs/MCP_API_SPEC.md` and
`specs/PROGRESS.md`; this method is implemented in the current slice.

#### `get_unrouted`
Nets with unrouted connections (airwires).

### DRC

#### `run_drc`
Run design rule checks. Returns structured violations.
```json
Input:  {"rules": ["clearance_copper", "track_width", "connectivity"]}
Output: {"passed": false, "violations": [
  {"rule": "clearance_copper", "severity": "error",
   "message": "Track-to-track clearance 0.08mm < 0.1mm minimum",
   "location": {"x": 25400000, "y": 18300000},
   "objects": ["track-uuid-1", "track-uuid-2"]}
]}
```
Implemented in the current daemon/stdio host with the current engine DRC
subset (connectivity + clearance checks).

#### `explain_violation`
Natural language explanation of a DRC violation with context.
```json
Input:  {"violation_index": 0}
Output: {"explanation": "Two tracks on the Top layer are 0.08mm apart,
          but the clearance rule for net class 'default' requires 0.1mm.
          The tracks belong to nets VCC and GND near component U1.",
         "suggestion": "Move track segment or increase spacing."}
```
Current `M2` availability is defined by `specs/MCP_API_SPEC.md` and
`specs/PROGRESS.md`; this method is implemented in the current slice.

---

## M3 Tools (Write Operations on Imported Designs)

Available when: engine supports operations with undo.

#### `move_component`
```json
Input:  {"reference": "U1", "x_mm": 25.0, "y_mm": 15.0, "rotation": 90}
Output: {"diff": {"modified": [["component", "uuid-of-U1"]]}}
```

#### `set_value`
```json
Input:  {"reference": "R1", "value": "4.7k"}
```

#### `set_reference`
```json
Input:  {"old": "R1", "new": "R101"}
```

#### `assign_part`
Assign a pool part to a component.
```json
Input:  {"reference": "R1", "part_uuid": "..."}
```

#### `delete_track`
Remove a track segment by UUID.

#### `set_design_rule`
Add or modify a design rule.
```json
Input:  {"rule_type": "clearance",
         "scope": {"NetClass": "uuid-of-highspeed"},
         "value_nm": 150000,
         "priority": 10}
```

#### `undo` / `redo`
Undo or redo the last operation.

#### `save`
Save modifications back to original format (KiCad write-back).

---

## M4 Tools (Native Authoring + Export)

Available when: engine supports native project creation and export.

#### `create_project`
Create a new empty project.

#### `place_symbol` / `draw_wire` / `annotate`
Schematic editing operations.

#### `place_component` / `add_track` / `add_via` / `add_zone`
Board editing operations.

#### `sync_schematic_to_board`
Generate ECO (list of changes). Returns per-change accept/reject list.

#### `export_gerber`
```json
Input:  {"output_dir": "/path/to/gerbers",
         "layers": ["top_copper", "bottom_copper", "top_silk", ...]}
```

#### `export_bom`
```json
Input:  {"format": "csv", "output": "/path/to/bom.csv"}
```

#### `export_drill` / `export_pnp`
Manufacturing outputs.

---

## M5-M6 Tools (Layout Engine)

Available when: placement + routing engine exists.

#### `suggest_placement`
AI-assisted placement proposal for unplaced or all components.
```json
Input:  {"components": ["U1", "U2", "R1", "R2", "C1"],
         "strategy": "minimize_wire_length"}
Output: {"proposal_id": "...",
         "placements": [{"ref": "U1", "x": 25.0, "y": 15.0, "rot": 0}, ...],
         "score": 0.73,
         "warnings": ["C1 placed 4mm from U1 VDD — target was 2mm"]}
```

#### `route_net`
Propose routing for a single net.
```json
Input:  {"net": "SPI_CLK", "strategy": "shortest"}
Output: {"proposal_id": "...", "tracks": [...], "vias": [...],
         "length_mm": 12.4, "clearance_ok": true}
```

#### `route_all`
Propose routing for all unrouted nets.

#### `accept_proposal` / `reject_proposal`
Accept or reject a layout proposal.

#### `analyze_layout`
AI analysis of current placement + routing quality.
```json
Output: {"score": 0.81,
         "issues": [
           "C3 should be closer to U2 pin 14 for decoupling",
           "Net CLK has 3 unnecessary vias",
           "Analog ground plane split under R7 by digital trace"],
         "suggestions": [...]}
```

---

## Error Handling

All tools return structured errors:
```json
{"error": "net_not_found",
 "message": "Net 'VCC_5V' does not exist in the design",
 "context": {"available_nets": ["VCC_3V3", "GND", "VBUS"]}}
```

## State Management

The MCP server is stateful — it holds a reference to one open project.
Operations are sequential (no concurrent modification). The engine
process manages undo/redo state. The MCP server is a thin translation
layer, not a state manager.
