# Horizon EDA — Architecture Analysis

## Summary
Analysis of Horizon EDA 2.7.0 source from https://github.com/horizon-eda/horizon
Commit: 46cc26c1 (2026-03-15)

## Technology Stack

| Component | Technology | Notes |
|-----------|-----------|-------|
| Language | C++17 | ~169,000 LOC |
| Build | Meson | Cross-platform |
| GUI | GTK3 + gtkmm | In maintenance mode (GTK4 is current) |
| Rendering | OpenGL (via libepoxy) | Custom triangle renderer, ~12,500 LOC |
| 3D | OpenGL + OpenCASCADE | Separate canvas3d module |
| Data storage | JSON files | nlohmann/json, all objects serializable |
| Pool index | SQLite3 | 10 tables, UUID-keyed, FTS for search |
| IPC | ZeroMQ | JSON messages between pool-mgr and editors |
| Version control | libgit2 | Native git integration in pool manager |
| Router | KiCad PNS (bundled) | Frozen at KiCad 6.0.4 (2022) |
| Geometry | clipper, poly2tri, delaunator | Bundled 3rd party |
| Python | CPython C API + py3cairo | Export-only module |
| Networking | libcurl | Pool downloads, stock info |
| PDF | podofo | PDF export |
| Archive | libarchive | Pool packaging |

## Source Tree (by size)

| Module | LOC | Role |
|--------|-----|------|
| imp/ | 26,922 | Interactive editor (GUI + tool integration) |
| core/ | 26,580 | Tool dispatch, document management, undo/redo |
| canvas/ | 12,459 | OpenGL rendering engine |
| board/ | 10,132 | Board data model + DRC rules |
| pool/ | 4,468 | Pool entity model (unit, entity, part, package) |
| schematic/ | 4,389 | Schematic data model |
| canvas3d/ | 3,164 | 3D viewer |
| export_odb/ | 2,945 | ODB++ export |
| common/ | 2,972 | Shared types and utilities |
| block/ | 2,220 | Electrical block (netlist) model |
| python_module/ | 1,583 | Python bindings (export + DRC only) |
| router/ | 1,416 | PNS interface adapter (3rd_party has the router) |
| rules/ | 1,354 | Rule matching and import/export |
| export_gerber/ | 976 | Gerber export |
| blocks/ | 604 | Hierarchical block management |
| export_step/ | 583 | STEP 3D export |

## Data Model — The Pool System

### Entity Hierarchy
```
Unit (electrical identity)
  └── Pin[] (with direction, swap_group, alternate_names)

Entity (multi-gate component)
  └── Gate[] → Unit reference

Package (physical footprint)
  └── Pad[] → Padstack reference
  └── Model[] (3D models)

Part (purchasable component — THE key abstraction)
  ├── → Entity reference
  ├── → Package reference
  ├── pad_map: Pad → (Gate, Pin) mapping
  ├── attributes: MPN, value, manufacturer, datasheet, description
  ├── parametric data (table-specific: resistance, capacitance, etc.)
  ├── orderable MPNs
  ├── flags: exclude_BOM, exclude_PnP, base_part
  └── → optional base Part (inheritance)
```

### Pool SQLite Schema (schema.sql)
- `units` (uuid, name, manufacturer, filename)
- `entities` (uuid, name, manufacturer, n_gates, prefix, filename)
- `symbols` (uuid, unit, name, filename)
- `packages` (uuid, name, manufacturer, n_pads, alternate_for, filename)
- `models` (package_uuid, model_uuid, model_filename)
- `parts` (uuid, MPN, manufacturer, entity, package, description, datasheet, parametric_table, base, flag_base_part)
- `orderable_MPNs` (part, uuid, MPN)
- `tags` (tag, uuid, type)
- `padstacks` (uuid, name, well_known_name, package, type)
- `frames`, `decals`, `dependencies`, `pools_included`
- `all_items_view` — unified view across all types
- `tags_view` — concatenated tags per item

### On-Disk Format
All design data stored as JSON files:
- `*.hprj` — project file
- `board.json` — PCB layout
- `blocks/*.json` — electrical blocks (netlists)
- `*.hsch` — schematics
- Pool items: individual JSON files in pool directory tree

## Tool System (181 ToolIDs)

### Tool Lifecycle
```
Core::tool_begin(ToolID, ToolArgs, ImpInterface*)
  → creates tool via factory
  → loads settings
  → sets ImpInterface
  → calls tool->begin(args)
  → returns ToolResponse

Core::tool_update(ToolArgs)
  → calls tool->update(args)
  → returns ToolResponse (NOP, END, COMMIT, REVERT)

On COMMIT: working document replaces saved document, rebuild triggered
On REVERT: working document restored from saved document
```

### ToolArgs (input to tools)
- `type`: NONE, MOVE, ACTION, LAYER_CHANGE, DATA
- `coords`: Coordi (x, y in nanometers)
- `selection`: set of SelectableRef (type + UUID + vertex)
- `action`: InToolActionID enum
- `target`: Target (specific object under cursor)
- `work_layer`: int
- `data`: optional ToolData pointer

### Tool Categories
**Non-interactive** (begin → immediate commit): DELETE, LOCK, UNLOCK, ANNOTATE,
SMASH, UNSMASH, SET_DIFFPAIR, CLEAR_DIFFPAIR, UPDATE_ALL_PLANES, FIX, UNFIX,
MERGE_DUPLICATE_JUNCTIONS

**Coordinate-interactive** (begin → series of MOVE/ACTION → commit): MOVE,
PLACE_JUNCTION, DRAW_LINE, DRAW_TRACK, PLACE_VIA, ROUTE_TRACK_INTERACTIVE,
ADD_COMPONENT, PLACE_TEXT

**Dialog-interactive** (begin → opens dialog → commit): EDIT_STACKUP,
MANAGE_NET_CLASSES, EDIT_PAD_PARAMETER_SET, MANAGE_POWER_NETS

## ZMQ IPC Protocol

### Current Messages
Editor → Project Manager: needs-save, ready, edit, show-in-pool-mgr,
preferences, update-pool

Project Manager → Editor: save, close, present, preferences, reload-pools,
pool-updated

Project Manager → Editor (design ops): highlight, place, reload-netlist,
reload-netlist-hint, backannotate, edit-meta

### Transport
- ZMQ PUB/SUB for broadcasts (project manager → all editors)
- ZMQ REQ/REP for point-to-point (editor → project manager)
- Messages: int prefix (PID or 0 for broadcast) + UUID cookie + JSON payload

## DRC Rules (13 types)
- clearance_copper
- clearance_copper_non_copper (silk, courtyard, etc.)
- clearance_copper_keepout
- clearance_same_net
- clearance_silk_exp_copper
- track_width
- hole_size
- via (size/drill)
- via_definitions
- diffpair
- plane
- board_connectivity
- preflight_checks
- net_ties
- thermals
- shorted_pads
- parameters
- layer_pair
- height_restrictions

## Export Capabilities
- Gerber (RS-274X / X2)
- ODB++
- PDF (schematic + board)
- STEP (3D model via OpenCASCADE)
- Pick and Place (CSV)
- BOM
- 3D Image (rendered PNG)

## What's Missing (vs. professional EDA)
- No interactive command line
- No autorouter (only interactive PNS)
- No BGA fanout automation
- No impedance calculator (rules exist, no solver)
- Python API is export-only (cannot modify designs)
- Router frozen at KiCad 6.0.4 (4 years behind)
- GTK3 (maintenance mode, should be GTK4)
- No Wayland-native rendering
- Single primary developer (bus factor = 1)
