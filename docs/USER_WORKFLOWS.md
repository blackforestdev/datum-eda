# User Workflows

> **Status**: Non-normative design rationale.
> The controlling CLI surface is `specs/PROGRAM_SPEC.md`.
> The controlling MCP surface is `specs/MCP_API_SPEC.md`.
> Command names and MCP tool names used in this document are illustrative
> examples of how the formal API surface might be exercised. Where this
> document uses a command or tool name not present in the controlling
> specs, it is marked **[aspirational]** and represents a design direction,
> not a committed contract.
> Scope terminology follows `specs/PROGRAM_SPEC.md` §Scope Integrity Terms.

## Purpose
Documents end-to-end usage scenarios for each milestone. These workflows
validate that the architectural decisions serve real engineering tasks
and help define what the tool should feel like to use. They are not
API specifications.

---

## 1. Target Users

### Primary: Professional EE on Linux
An electrical engineer who:
- Works on Linux (by choice or workplace requirement)
- May have existing KiCad/Eagle designs and needs migration continuity
- Wants programmatic access to design data
- Values CLI and scriptability over GUI clicking
- May already use AI coding/assistant tools in their workflow
- Needs CI/CD integration for design verification

### Secondary: AI Agent (autonomous or semi-autonomous)
An MCP-compatible AI agent/client that:
- Opens designs via MCP
- Queries design data to answer engineering questions
- Runs ERC/DRC and explains results
- Proposes design modifications
- Operates under human review (proposes, doesn't auto-commit)

### Tertiary: Hardware CI/CD Pipeline
An automated system that:
- Runs ERC/DRC on every commit to a hardware repository
- Fails the build on violations
- Generates reports for design review
- Tracks design metrics over time

### Not the v1 target
- Students learning PCB design (need a GUI first)
- Hobbyists who want a free KiCad alternative (we're not replacing KiCad in v1)
- Altium/OrCAD users who need Windows (we're Linux-only)

---

## 2. M2 Workflows (v1: Analysis + Automation)

These workflows use commands and MCP tools defined in `specs/PROGRAM_SPEC.md`
and `specs/MCP_API_SPEC.md`.

### 2.1 "I have a KiCad project, what's in it?"

```bash
# Import and query (CLI names per specs/PROGRAM_SPEC.md §M2)
$ tool import board.kicad_pcb
$ tool query board.kicad_pcb --summary
$ tool query board.kicad_pcb --nets
$ tool query board.kicad_pcb --components
```

### 2.2 "Run DRC/ERC on my design in CI"

> `tool erc` is in the current implemented slice.
> `tool drc` is now wired in the current slice; full M2 exit semantics
> (corpus-backed thresholds and complete rule set) remain in progress.

```bash
# DRC and ERC per specs/PROGRAM_SPEC.md
$ tool drc board.kicad_pcb          # exit code 1 if violations
$ tool erc schematic.kicad_sch      # exit code 1 if violations
$ tool drc board.kicad_pcb --format json > drc-report.json
```

Exit codes (per `specs/PROGRAM_SPEC.md`):
- 0 = pass
- 1 = violations
- 2 = error

### 2.3 "AI, tell me about this design" (MCP workflow)

Using MCP tools defined in `specs/MCP_API_SPEC.md`:

```
AI client → MCP: open_project({ path: "amplifier.kicad_pcb" })
AI client → MCP: get_board_summary({})
AI client → MCP: get_net_info({})
AI client → MCP: get_unrouted({})
```

The AI agent uses the structured responses to compose a human-readable
answer about the design.

### 2.4 "AI, run ERC and explain what's wrong" (MCP workflow)

```
AI client → MCP: open_project({ path: "sensor.kicad_sch" })
AI client → MCP: run_erc({})
AI client → MCP: get_check_report({})
```

`explain_violation` is part of the current `M2` MCP surface. The agent can use
that method directly, and can still fall back to `run_erc` findings plus
`get_check_report` summary/detail data when needed.

### 2.5 "Search the pool for a part"

```bash
$ tool pool search "100nF 0402 ceramic"
```

---

## 3. M3 Workflows (Write Operations)

These workflows use MCP tools defined in `specs/MCP_API_SPEC.md` §M3.

### 3.1 "AI, reorganize the component placement"

```
AI client → MCP: move_component({ reference: "U1", x_mm: 25.0, y_mm: 15.0, rotation_deg: 90 })
AI client → MCP: move_component({ reference: "C1", x_mm: 24.0, y_mm: 16.5 })
AI client → MCP: run_drc({})
AI client → MCP: save({})
```

### 3.2 "Batch modify via CLI"

```bash
# Per specs/PROGRAM_SPEC.md §M3
$ tool modify board.kicad_pcb --move U1 25.0,15.0 --rotate U1 90
$ tool modify board.kicad_pcb --undo
$ tool modify board.kicad_pcb --save
```

---

## 4. M4 Workflows (Native Authoring)

> **Note**: M4 CLI and MCP command names below are **[aspirational]**.
> The formal M4 API surface in `specs/MCP_API_SPEC.md` §M4 defines the
> controlling tool names. The workflows below illustrate the intended
> user experience, not the exact command surface.

### 4.1 "Create a design from scratch" **[aspirational]**

The M4 workflow enables creating a project, placing symbols, wiring,
annotating, forward-annotating to board, and exporting manufacturing
files — all via CLI or MCP, no GUI required.

Current live slice:
- `eda project new <dir> [--name <project-name>]` creates a deterministic native project scaffold with `project.json`, `schematic/schematic.json`, `board/board.json`, and `rules/rules.json`.
- `eda project inspect <dir>` validates that native scaffold and reports the resolved schema, UUIDs, paths, and current object counts.
- `eda project query <dir> summary` reports the current native schematic/board/rule summary from the scaffold.
- `eda project query <dir> design-rules` reports the current native rules payload.
- `eda project place-symbol <dir> --sheet <uuid> --reference <text> --value <text> [--lib-id <text>] --x-nm <i64> --y-nm <i64> [--rotation-deg <i32>] [--mirrored]` places a native schematic symbol into a referenced sheet file.
- `eda project move-symbol <dir> --symbol <uuid> --x-nm <i64> --y-nm <i64>` repositions an existing native schematic symbol.
- `eda project rotate-symbol <dir> --symbol <uuid> --rotation-deg <i32>` updates the stored rotation for an existing native schematic symbol.
- `eda project mirror-symbol <dir> --symbol <uuid>` toggles the stored mirrored state for an existing native schematic symbol.
- `eda project delete-symbol <dir> --symbol <uuid>` removes an existing native schematic symbol.
- `eda project set-symbol-reference <dir> --symbol <uuid> --reference <text>` updates the stored reference designator for an existing native schematic symbol.
- `eda project set-symbol-value <dir> --symbol <uuid> --value <text>` updates the stored value text for an existing native schematic symbol.
- `eda project set-symbol-unit <dir> --symbol <uuid> --unit <text>` records the current native unit-selection token for an existing schematic symbol.
- `eda project clear-symbol-unit <dir> --symbol <uuid>` clears the stored native unit-selection token for an existing schematic symbol.
- `eda project set-symbol-gate <dir> --symbol <uuid> --gate <uuid>` records the current native gate-selection UUID for an existing schematic symbol.
- `eda project clear-symbol-gate <dir> --symbol <uuid>` clears the stored native gate-selection UUID for an existing schematic symbol.
- `eda project set-symbol-display-mode <dir> --symbol <uuid> --mode <library-default|show-hidden-pins|hide-optional-pins>` updates the stored native symbol display mode.
- `eda project set-symbol-hidden-power-behavior <dir> --symbol <uuid> --behavior <source-defined-implicit|explicit-power-object|preserved-as-imported-metadata>` updates the stored native hidden-power handling mode for an existing symbol.
- `eda project set-pin-override <dir> --symbol <uuid> --pin <uuid> --visible <true|false> [--x-nm <i64> --y-nm <i64>]` updates one stored per-pin display override on a native symbol.
- `eda project clear-pin-override <dir> --symbol <uuid> --pin <uuid>` removes one stored per-pin display override from a native symbol.
- `eda project add-symbol-field <dir> --symbol <uuid> --key <text> --value <text> [--hidden] [--x-nm <i64> --y-nm <i64>]` adds a native symbol field.
- `eda project edit-symbol-field <dir> --field <uuid> [--key <text>] [--value <text>] [--visible <true|false>] [--x-nm <i64> --y-nm <i64>]` edits an existing native symbol field.
- `eda project delete-symbol-field <dir> --field <uuid>` removes an existing native symbol field.
- `eda project query <dir> symbols` reports the current native symbol inventory from referenced sheet files.
- `eda project query <dir> symbol-fields --symbol <uuid>` reports the current field list for one native symbol.
- `eda project query <dir> symbol-semantics --symbol <uuid>` reports the current native gate/unit selection state for one native symbol.
- `eda project query <dir> symbol-pins --symbol <uuid>` reports the stored pin list and any per-pin override state for one native symbol.
- `eda project place-text <dir> --sheet <uuid> --text <text> --x-nm <i64> --y-nm <i64> [--rotation-deg <i32>]` places a native schematic text object.
- `eda project edit-text <dir> --text <uuid> [--value <text>] [--x-nm <i64> --y-nm <i64>] [--rotation-deg <i32>]` edits an existing native schematic text object.
- `eda project delete-text <dir> --text <uuid>` removes an existing native schematic text object.
- `eda project query <dir> texts` reports the current native text inventory from referenced sheet files.
- `eda project place-drawing-line <dir> --sheet <uuid> --from-x-nm <i64> --from-y-nm <i64> --to-x-nm <i64> --to-y-nm <i64>` places a native schematic drawing line.
- `eda project place-drawing-rect <dir> --sheet <uuid> --min-x-nm <i64> --min-y-nm <i64> --max-x-nm <i64> --max-y-nm <i64>` places a native schematic drawing rectangle.
- `eda project place-drawing-circle <dir> --sheet <uuid> --center-x-nm <i64> --center-y-nm <i64> --radius-nm <i64>` places a native schematic drawing circle.
- `eda project place-drawing-arc <dir> --sheet <uuid> --center-x-nm <i64> --center-y-nm <i64> --radius-nm <i64> --start-angle-mdeg <i64> --end-angle-mdeg <i64>` places a native schematic drawing arc.
- `eda project edit-drawing-line <dir> --drawing <uuid> [--from-x-nm <i64> --from-y-nm <i64>] [--to-x-nm <i64> --to-y-nm <i64>]` edits an existing native schematic drawing line.
- `eda project edit-drawing-rect <dir> --drawing <uuid> ...` edits an existing native schematic drawing rectangle.
- `eda project edit-drawing-circle <dir> --drawing <uuid> ...` edits an existing native schematic drawing circle.
- `eda project edit-drawing-arc <dir> --drawing <uuid> ...` edits an existing native schematic drawing arc.
- `eda project delete-drawing <dir> --drawing <uuid>` removes an existing native schematic drawing primitive.
- `eda project query <dir> drawings` reports the current native drawing inventory from referenced sheet files.
- `eda project place-label <dir> --sheet <uuid> --name <text> --x-nm <i64> --y-nm <i64>` writes the first native authored schematic object into a referenced sheet file.
- `eda project rename-label <dir> --label <uuid> --name <text>` renames an existing native schematic label.
- `eda project delete-label <dir> --label <uuid>` removes an existing native schematic label.
- `eda project draw-wire <dir> --sheet <uuid> --from-x-nm <i64> --from-y-nm <i64> --to-x-nm <i64> --to-y-nm <i64>` adds a native schematic wire to a referenced sheet file.
- `eda project delete-wire <dir> --wire <uuid>` removes an existing native schematic wire.
- `eda project place-junction <dir> --sheet <uuid> --x-nm <i64> --y-nm <i64>` places a native schematic junction in a referenced sheet file.
- `eda project delete-junction <dir> --junction <uuid>` removes an existing native schematic junction.
- `eda project place-port <dir> --sheet <uuid> --name <text> --direction <input|output|bidirectional|passive> --x-nm <i64> --y-nm <i64>` places a native hierarchical port.
- `eda project edit-port <dir> --port <uuid> [--name <text>] [--direction <...>] [--x-nm <i64>] [--y-nm <i64>]` edits an existing native hierarchical port.
- `eda project delete-port <dir> --port <uuid>` removes an existing native hierarchical port.
- `eda project create-bus <dir> --sheet <uuid> --name <text> --member <text>...` creates a native schematic bus.
- `eda project edit-bus-members <dir> --bus <uuid> --member <text>...` edits the member list for an existing native schematic bus.
- `eda project place-bus-entry <dir> --sheet <uuid> --bus <uuid> [--wire <uuid>] --x-nm <i64> --y-nm <i64>` places a native bus entry.
- `eda project delete-bus-entry <dir> --bus-entry <uuid>` removes an existing native bus entry.
- `eda project place-noconnect <dir> --sheet <uuid> --symbol <uuid> --pin <uuid> --x-nm <i64> --y-nm <i64>` places a native no-connect marker.
- `eda project delete-noconnect <dir> --noconnect <uuid>` removes an existing native no-connect marker.

MCP tools for this flow are defined in `specs/MCP_API_SPEC.md` §M4:
`place_symbol`, `draw_wire`, `place_label`, `annotate`,
`sync_schematic_to_board`, `export_gerber`, `export_bom`, etc.

### 4.2 "AI builds a complete design" **[aspirational]**

An AI agent uses the M4 MCP surface to:
1. Create a project
2. Search the pool for needed parts
3. Place symbols and draw wires on schematic sheets
4. Run ERC to verify the schematic
5. Forward-annotate to create a board
6. Place components on the board
7. Run DRC to verify placement
8. Export manufacturing files

This represents the full "design without a GUI" ambition of the project.

---

## 5. CI/CD Integration Patterns

### 5.1 Pre-commit hook

```bash
#!/bin/bash
# Uses M2 CLI surface
for pcb in $(git diff --cached --name-only -- '*.kicad_pcb'); do
    tool drc "$pcb"
    if [ $? -ne 0 ]; then
        echo "DRC failed on $pcb"
        exit 1
    fi
done
for sch in $(git diff --cached --name-only -- '*.kicad_sch'); do
    tool erc "$sch"
    if [ $? -ne 0 ]; then
        echo "ERC failed on $sch"
        exit 1
    fi
done
```

### 5.2 Design metrics tracking

```bash
$ tool query board.kicad_pcb --summary --format json
```

JSON output enables parsing by CI systems for metric tracking.

---

## 6. Scripting Workflows (Python via PyO3) **[aspirational]**

The Python scripting interface (PyO3) is planned for M4+. The engine
API methods (`specs/ENGINE_SPEC.md` §5) will be exposed as Python
functions. Example patterns:

- Batch parametric analysis (iterate components, check power ratings)
- Custom DRC rules (check decoupling cap proximity)
- Design data extraction for external analysis tools

These are design directions, not committed Python API surfaces.
