#!/usr/bin/env python3
"""Native MCP parity for schematic drawing shapes beyond lines."""

from __future__ import annotations

import os
import shlex
import tempfile
import unittest
from pathlib import Path
from unittest.mock import patch

from server_runtime import EngineDaemonClient, StdioToolHost
from test_native_write_parity import (
    assert_latest_journal_operation,
    call_tool,
    datum_cli_prefix,
    query_result,
    run_cli_json,
)


def drawing_by_uuid(drawings: list[dict], drawing: str) -> dict:
    return next(item for item in drawings if item["uuid"] == drawing)


class TestNativeSchematicDrawingShapeParity(unittest.TestCase):
    def make_host(self, root: Path) -> tuple[StdioToolHost, str]:
        run_cli_json(root, "project", "new", str(root), "--name", "Drawing Shape Parity")
        env = {"DATUM_CLI_BIN": " ".join(shlex.quote(part) for part in datum_cli_prefix())}
        patcher = patch.dict(os.environ, env, clear=False)
        patcher.start()
        self.addCleanup(patcher.stop)
        host = StdioToolHost(EngineDaemonClient())
        sheet = call_tool(host, "datum.schematic.create_sheet", {"path": str(root), "name": "Main"})["sheet_uuid"]
        return host, sheet

    def test_rect_drawing_tools_call_writes_deletes_and_replays(self) -> None:
        with tempfile.TemporaryDirectory(prefix="datum-mcp-drawing-rect-") as tmp:
            root = Path(tmp)
            host, sheet = self.make_host(root)
            drawing = call_tool(
                host,
                "datum.schematic.place_drawing_rect",
                {"path": str(root), "sheet": sheet, "min_x_nm": 0, "min_y_nm": 0, "max_x_nm": 100, "max_y_nm": 100},
            )["drawing_uuid"]
            rect = drawing_by_uuid(query_result(host, "datum.query.schematic_drawings", root), drawing)
            self.assertEqual(rect["kind"], "rect")
            self.assertEqual(rect["min"], {"x": 0, "y": 0})
            self.assertEqual(rect["max"], {"x": 100, "y": 100})
            operation = assert_latest_journal_operation(
                self, host, str(root), "place schematic drawing rect", "create_schematic_drawing"
            )
            self.assertEqual(operation["drawing_id"], drawing)
            self.assertEqual(
                call_tool(
                    host,
                    "datum.schematic.edit_drawing_rect",
                    {"path": str(root), "drawing": drawing, "max_x_nm": 200, "max_y_nm": 150},
                )["action"],
                "edit_drawing_rect",
            )
            rect = drawing_by_uuid(query_result(host, "datum.query.schematic_drawings", root), drawing)
            self.assertEqual(rect["max"], {"x": 200, "y": 150})
            operation = assert_latest_journal_operation(
                self, host, str(root), "edit schematic drawing rect", "set_schematic_drawing"
            )
            self.assertEqual(operation["drawing_id"], drawing)
            self.assertEqual(call_tool(host, "datum.journal.undo", {"path": str(root)})["status"], "applied")
            rect = drawing_by_uuid(query_result(host, "datum.query.schematic_drawings", root), drawing)
            self.assertEqual(rect["max"], {"x": 100, "y": 100})
            self.assertEqual(call_tool(host, "datum.journal.redo", {"path": str(root)})["status"], "applied")
            self.assertEqual(
                call_tool(host, "datum.schematic.delete_drawing", {"path": str(root), "drawing": drawing})["action"],
                "delete_drawing",
            )
            self.assertEqual(query_result(host, "datum.query.schematic_drawings", root), [])
            operation = assert_latest_journal_operation(
                self, host, str(root), "delete schematic drawing", "delete_schematic_drawing"
            )
            self.assertEqual(operation["drawing_id"], drawing)

    def test_circle_drawing_tools_call_writes_deletes_and_replays(self) -> None:
        with tempfile.TemporaryDirectory(prefix="datum-mcp-drawing-circle-") as tmp:
            root = Path(tmp)
            host, sheet = self.make_host(root)
            drawing = call_tool(
                host,
                "datum.schematic.place_drawing_circle",
                {"path": str(root), "sheet": sheet, "center_x_nm": 50, "center_y_nm": 60, "radius_nm": 25},
            )["drawing_uuid"]
            circle = drawing_by_uuid(query_result(host, "datum.query.schematic_drawings", root), drawing)
            self.assertEqual(circle["kind"], "circle")
            self.assertEqual(circle["center"], {"x": 50, "y": 60})
            self.assertEqual(circle["radius"], 25)
            operation = assert_latest_journal_operation(
                self, host, str(root), "place schematic drawing circle", "create_schematic_drawing"
            )
            self.assertEqual(operation["drawing_id"], drawing)
            self.assertEqual(
                call_tool(
                    host,
                    "datum.schematic.edit_drawing_circle",
                    {"path": str(root), "drawing": drawing, "center_x_nm": 70, "center_y_nm": 80, "radius_nm": 50},
                )["action"],
                "edit_drawing_circle",
            )
            circle = drawing_by_uuid(query_result(host, "datum.query.schematic_drawings", root), drawing)
            self.assertEqual(circle["center"], {"x": 70, "y": 80})
            self.assertEqual(circle["radius"], 50)
            operation = assert_latest_journal_operation(
                self, host, str(root), "edit schematic drawing circle", "set_schematic_drawing"
            )
            self.assertEqual(operation["drawing_id"], drawing)
            self.assertEqual(call_tool(host, "datum.journal.undo", {"path": str(root)})["status"], "applied")
            circle = drawing_by_uuid(query_result(host, "datum.query.schematic_drawings", root), drawing)
            self.assertEqual(circle["radius"], 25)
            self.assertEqual(call_tool(host, "datum.journal.redo", {"path": str(root)})["status"], "applied")
            self.assertEqual(
                call_tool(host, "datum.schematic.delete_drawing", {"path": str(root), "drawing": drawing})["action"],
                "delete_drawing",
            )
            self.assertEqual(query_result(host, "datum.query.schematic_drawings", root), [])

    def test_arc_drawing_tools_call_writes_deletes_and_replays(self) -> None:
        with tempfile.TemporaryDirectory(prefix="datum-mcp-drawing-arc-") as tmp:
            root = Path(tmp)
            host, sheet = self.make_host(root)
            drawing = call_tool(
                host,
                "datum.schematic.place_drawing_arc",
                {
                    "path": str(root),
                    "sheet": sheet,
                    "center_x_nm": 50,
                    "center_y_nm": 60,
                    "radius_nm": 25,
                    "start_angle_mdeg": 0,
                    "end_angle_mdeg": 90000,
                },
            )["drawing_uuid"]
            arc = drawing_by_uuid(query_result(host, "datum.query.schematic_drawings", root), drawing)
            self.assertEqual(arc["kind"], "arc")
            self.assertEqual(arc["arc"]["center"], {"x": 50, "y": 60})
            self.assertEqual(arc["arc"]["radius"], 25)
            self.assertEqual(arc["arc"]["end_angle"], 90000)
            operation = assert_latest_journal_operation(
                self, host, str(root), "place schematic drawing arc", "create_schematic_drawing"
            )
            self.assertEqual(operation["drawing_id"], drawing)
            self.assertEqual(
                call_tool(
                    host,
                    "datum.schematic.edit_drawing_arc",
                    {"path": str(root), "drawing": drawing, "radius_nm": 75, "end_angle_mdeg": 180000},
                )["action"],
                "edit_drawing_arc",
            )
            arc = drawing_by_uuid(query_result(host, "datum.query.schematic_drawings", root), drawing)
            self.assertEqual(arc["arc"]["radius"], 75)
            self.assertEqual(arc["arc"]["end_angle"], 180000)
            operation = assert_latest_journal_operation(
                self, host, str(root), "edit schematic drawing arc", "set_schematic_drawing"
            )
            self.assertEqual(operation["drawing_id"], drawing)
            self.assertEqual(call_tool(host, "datum.journal.undo", {"path": str(root)})["status"], "applied")
            arc = drawing_by_uuid(query_result(host, "datum.query.schematic_drawings", root), drawing)
            self.assertEqual(arc["arc"]["radius"], 25)
            self.assertEqual(call_tool(host, "datum.journal.redo", {"path": str(root)})["status"], "applied")
            self.assertEqual(
                call_tool(host, "datum.schematic.delete_drawing", {"path": str(root), "drawing": drawing})["action"],
                "delete_drawing",
            )
            self.assertEqual(query_result(host, "datum.query.schematic_drawings", root), [])


if __name__ == "__main__":
    unittest.main()
