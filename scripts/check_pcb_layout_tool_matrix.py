#!/usr/bin/env python3
"""Lock the PCB layout contract's 13 logical tool buckets.

The public surface has more than 13 concrete verbs because CRUD/property
spelling is user-friendly at the CLI/MCP boundary. This gate keeps that
concrete surface tied to the lean logical matrix in
docs/contracts/PCB_LAYOUT_TOOL_CONTRACT.md.
"""

from __future__ import annotations

import json
import re
import sys
from pathlib import Path


ROOT = Path(__file__).resolve().parents[1]
CATALOG = ROOT / "mcp-server/datum_tool_catalog.json"
PROJECT_ARGS = ROOT / "crates/cli/src/args/project.rs"
OPERATIONS = ROOT / "crates/engine/src/substrate/operation.rs"


def read(path: Path) -> str:
    return path.read_text(encoding="utf-8")


def catalog_verbs() -> dict[str, dict]:
    payload = json.loads(read(CATALOG))
    return {str(verb["name"]): verb for verb in payload["verbs"]}


def fail(message: str) -> None:
    print(f"check_pcb_layout_tool_matrix: {message}", file=sys.stderr)
    raise SystemExit(1)


def require_names(names: set[str], required: set[str], bucket: str) -> None:
    missing = sorted(required - names)
    if missing:
        fail(f"{bucket} missing public tool(s): {', '.join(missing)}")


def require_markers(text: str, markers: tuple[str, ...], bucket: str, source: str) -> None:
    missing = [marker for marker in markers if marker not in text]
    if missing:
        fail(f"{bucket} missing {source} marker(s): {', '.join(missing)}")


def main() -> None:
    verbs = catalog_verbs()
    names = set(verbs)
    project_args = read(PROJECT_ARGS)
    operations = read(OPERATIONS)

    buckets: dict[str, set[str]] = {
        "place_component": {"datum.pcb.place_component"},
        "move_rotate_flip_component": {
            "datum.pcb.move_component",
            "datum.pcb.rotate_component",
            "datum.pcb.flip_component",
        },
        "align_distribute": {"datum.pcb.align_components"},
        "draw_track_delete_track": {"datum.pcb.draw_track", "datum.pcb.delete_track"},
        "place_via_delete_via": {"datum.pcb.place_via", "datum.pcb.delete_via"},
        "author_zone_edit_delete": {
            "datum.pcb.place_zone",
            "datum.pcb.edit_zone",
            "datum.pcb.delete_zone",
        },
        "fill_zones": {"datum.check.fill_zones"},
        "author_keepout": {
            "datum.pcb.place_keepout",
            "datum.pcb.edit_keepout",
            "datum.pcb.delete_keepout",
        },
        "set_board_outline": {"datum.pcb.set_outline"},
        "edit_stackup": {"datum.pcb.set_stackup", "datum.pcb.add_default_top_stackup"},
        "set_design_rules_set_net_class": {
            "datum.pcb.place_net_class",
            "datum.pcb.edit_net_class",
            "datum.pcb.delete_net_class",
        },
        "inspect_edit": {
            "datum.pcb.edit_pad",
            "datum.pcb.edit_track",
            "datum.pcb.lock_component",
            "datum.pcb.unlock_component",
        },
        "run_drc": {"datum.check.run"},
    }
    for bucket, required in buckets.items():
        require_names(names, required, bucket)

    require_markers(
        operations,
        (
            "CreateBoardPackage",
            "SetBoardPackagePosition",
            "SetBoardPackageRotation",
            "SetComponentSide",
            "CreateBoardTrack",
            "DeleteBoardTrack",
            "CreateBoardVia",
            "DeleteBoardVia",
            "CreateBoardZone",
            "SetBoardZone",
            "DeleteBoardZone",
            "SetZoneFill",
            "DeleteZoneFill",
            "CreateBoardKeepout",
            "SetBoardKeepout",
            "DeleteBoardKeepout",
            "SetBoardOutline",
            "SetBoardStackup",
            "SetProjectRule",
            "SetBoardNetClass",
            "SetBoardPackageLocked",
        ),
        "operation_coverage",
        "Operation",
    )
    require_markers(
        project_args,
        ("AlignBoardComponents", "SetBoardComponentLocked", "ClearBoardComponentLocked"),
        "cli_coverage",
        "ProjectCommands",
    )

    align = verbs["datum.pcb.align_components"]
    if align["dispatch"]["method"] != "align_board_components":
        fail("align_distribute dispatch method must be align_board_components")
    if not any(token.get("repeated") == "--component" for token in align["dispatch"]["argv"]):
        fail("align_distribute must use one repeated --component parameter")
    mode_schema = align["inputSchema"]["properties"]["mode"]
    expected_modes = {
        "left",
        "right",
        "top",
        "bottom",
        "hcenter",
        "vcenter",
        "distribute-h",
        "distribute-v",
    }
    if set(mode_schema.get("enum", [])) != expected_modes:
        fail("align_distribute mode enum drifted")

    forbidden = {
        "align_left",
        "align_right",
        "align_top",
        "align_bottom",
        "align_hcenter",
        "align_vcenter",
        "distribute_h",
        "distribute_v",
    }
    redundant = sorted(
        name for name in names if name.startswith("datum.pcb.") and name.split(".")[-1] in forbidden
    )
    if redundant:
        fail(f"align/distribute must stay one mode-parameterized tool, found: {redundant}")

    pcb_fill_tools = sorted(
        name for name in names if name.startswith("datum.pcb.") and "fill" in name
    )
    if pcb_fill_tools:
        fail(f"fill_zones is a datum.check derived-state tool, not datum.pcb: {pcb_fill_tools}")

    print(
        "check_pcb_layout_tool_matrix: 13 logical PCB buckets covered; "
        "align/distribute is a single mode-parameterized tool"
    )


if __name__ == "__main__":
    main()
