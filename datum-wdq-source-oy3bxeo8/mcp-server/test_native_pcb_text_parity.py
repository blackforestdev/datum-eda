#!/usr/bin/env python3
"""Native MCP parity for PCB text authoring."""

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
    run_cli_json,
)


class TestNativePcbTextParity(unittest.TestCase):
    def test_pcb_text_tools_create_edit_delete_and_replay(self) -> None:
        with tempfile.TemporaryDirectory(prefix="datum-mcp-board-text-") as tmp:
            root = Path(tmp)
            run_cli_json(root, "project", "new", str(root), "--name", "Board Text Parity")
            env = {"DATUM_CLI_BIN": " ".join(shlex.quote(part) for part in datum_cli_prefix())}
            with patch.dict(os.environ, env, clear=False):
                host = StdioToolHost(EngineDaemonClient())
                text = call_tool(
                    host,
                    "datum.pcb.place_text",
                    {
                        "path": str(root),
                        "text": "PCB TOP",
                        "x_nm": 1000,
                        "y_nm": 2000,
                        "layer": 1,
                        "rotation_deg": 90,
                        "render_intent": "annotation",
                        "style": "regular",
                        "style_class": "fab-note",
                        "h_align": "center",
                        "v_align": "top",
                        "mirrored": True,
                        "keep_upright": True,
                        "line_spacing_ratio_ppm": 1350000,
                        "bold": True,
                        "italic": True,
                    },
                )["text_uuid"]
                texts = run_cli_json(root, "project", "query", str(root), "board-texts")
                self.assertEqual(len(texts), 1)
                self.assertEqual(texts[0]["uuid"], text)
                self.assertEqual(texts[0]["text"], "PCB TOP")
                self.assertEqual(texts[0]["position"], {"x": 1000, "y": 2000})
                self.assertEqual(texts[0]["render_intent"], "annotation")
                self.assertEqual(texts[0]["h_align"], "center")
                self.assertEqual(texts[0]["v_align"], "top")
                self.assertTrue(texts[0]["mirrored"])
                self.assertTrue(texts[0]["keep_upright"])
                self.assertTrue(texts[0]["bold"])
                self.assertTrue(texts[0]["italic"])
                operation = assert_latest_journal_operation(
                    self, host, str(root), "place board text", "create_board_text"
                )
                self.assertEqual(operation["text_id"], text)
                self.assertEqual(operation["text"], texts[0])
                edit = call_tool(
                    host,
                    "datum.pcb.edit_text",
                    {
                        "path": str(root),
                        "text": text,
                        "value": "PCB BOT",
                        "x_nm": 3000,
                        "y_nm": 4000,
                        "layer": 2,
                        "rotation_deg": 180,
                        "height_nm": 1200000,
                        "stroke_width_nm": 150000,
                        "family": "jetbrains_mono",
                        "style": "regular",
                        "style_class": "reference-label",
                        "h_align": "right",
                        "v_align": "bottom",
                        "mirrored": False,
                        "keep_upright": False,
                        "line_spacing_ratio_ppm": 900000,
                        "bold": False,
                        "italic": False,
                    },
                )
                self.assertEqual(edit["action"], "edit_board_text")
                texts = run_cli_json(root, "project", "query", str(root), "board-texts")
                self.assertEqual(texts[0]["text"], "PCB BOT")
                self.assertEqual(texts[0]["position"], {"x": 3000, "y": 4000})
                self.assertEqual(texts[0]["family_source"], "explicit")
                self.assertFalse(texts[0]["mirrored"])
                self.assertFalse(texts[0]["keep_upright"])
                self.assertFalse(texts[0]["bold"])
                self.assertFalse(texts[0]["italic"])
                operation = assert_latest_journal_operation(
                    self, host, str(root), "edit board text", "set_board_text"
                )
                self.assertEqual(operation["text"], texts[0])
                self.assertEqual(call_tool(host, "datum.journal.undo", {"path": str(root)})["status"], "applied")
                texts = run_cli_json(root, "project", "query", str(root), "board-texts")
                self.assertEqual(texts[0]["text"], "PCB TOP")
                self.assertEqual(call_tool(host, "datum.journal.redo", {"path": str(root)})["status"], "applied")
                self.assertEqual(
                    call_tool(host, "datum.pcb.delete_text", {"path": str(root), "text": text})["action"],
                    "delete_board_text",
                )
                self.assertEqual(run_cli_json(root, "project", "query", str(root), "board-texts"), [])
                operation = assert_latest_journal_operation(
                    self, host, str(root), "delete board text", "delete_board_text"
                )
                self.assertEqual(operation["text_id"], text)


if __name__ == "__main__":
    unittest.main()
