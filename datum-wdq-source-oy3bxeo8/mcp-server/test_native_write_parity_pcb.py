#!/usr/bin/env python3
"""PCB-focused canonical MCP write parity against real native projects."""

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
    seed_board_net,
)


class TestNativeWriteParityPcb(unittest.TestCase):
    def test_pcb_draw_track_tools_call_writes_model_and_journal(self) -> None:
        with tempfile.TemporaryDirectory(prefix="datum-mcp-track-") as tmp:
            root = Path(tmp)
            run_cli_json(root, "project", "new", str(root), "--name", "Track Parity")
            env = {"DATUM_CLI_BIN": " ".join(shlex.quote(part) for part in datum_cli_prefix())}
            with patch.dict(os.environ, env, clear=False):
                host = StdioToolHost(EngineDaemonClient())
                net = seed_board_net(host, root)
                track = call_tool(
                    host,
                    "datum.pcb.draw_track",
                    {
                        "path": str(root),
                        "net": net,
                        "from_x_nm": 1000,
                        "from_y_nm": 2000,
                        "to_x_nm": 3000,
                        "to_y_nm": 4000,
                        "width_nm": 250000,
                        "layer": 1,
                    },
                )["track_uuid"]

                tracks = query_result(host, "datum.query.board_tracks", root)
                self.assertEqual(len(tracks), 1)
                self.assertEqual(tracks[0]["uuid"], track)
                self.assertEqual(tracks[0]["net"], net)
                self.assertEqual(tracks[0]["from"], {"x": 1000, "y": 2000})
                self.assertEqual(tracks[0]["to"], {"x": 3000, "y": 4000})
                self.assertEqual(tracks[0]["width"], 250000)
                self.assertEqual(tracks[0]["layer"], 1)
                operation = assert_latest_journal_operation(
                    self, host, str(root), "draw board track", "create_board_track"
                )
                self.assertEqual(operation["track_id"], track)
                self.assertEqual(operation["track"]["uuid"], track)
                self.assertEqual(operation["track"]["net"], net)
                self.assertEqual(operation["track"]["from"], {"x": 1000, "y": 2000})
                self.assertEqual(operation["track"]["to"], {"x": 3000, "y": 4000})
                self.assertEqual(operation["track"]["width"], 250000)
                self.assertEqual(operation["track"]["layer"], 1)
                undo = call_tool(host, "datum.journal.undo", {"path": str(root)})
                self.assertEqual(undo["action"], "undo")
                self.assertEqual(undo["status"], "applied")
                self.assertEqual(query_result(host, "datum.query.board_tracks", root), [])
                redo = call_tool(host, "datum.journal.redo", {"path": str(root)})
                self.assertEqual(redo["action"], "redo")
                self.assertEqual(redo["status"], "applied")
                redone_tracks = query_result(host, "datum.query.board_tracks", root)
                self.assertEqual(len(redone_tracks), 1)
                self.assertEqual(redone_tracks[0]["uuid"], track)
                self.assertEqual(
                    call_tool(host, "datum.pcb.delete_track", {"path": str(root), "track": track})["action"],
                    "delete_board_track",
                )
                self.assertEqual(query_result(host, "datum.query.board_tracks", root), [])
                operation = assert_latest_journal_operation(
                    self, host, str(root), "delete board track", "delete_board_track"
                )
                self.assertEqual(operation["track_id"], track)
                self.assertEqual(call_tool(host, "datum.journal.undo", {"path": str(root)})["status"], "applied")
                self.assertEqual(query_result(host, "datum.query.board_tracks", root)[0]["uuid"], track)
                self.assertEqual(call_tool(host, "datum.journal.redo", {"path": str(root)})["status"], "applied")
                self.assertEqual(query_result(host, "datum.query.board_tracks", root), [])

    def test_pcb_via_tools_call_writes_deletes_and_replays(self) -> None:
        with tempfile.TemporaryDirectory(prefix="datum-mcp-via-") as tmp:
            root = Path(tmp)
            run_cli_json(root, "project", "new", str(root), "--name", "Via Parity")
            env = {"DATUM_CLI_BIN": " ".join(shlex.quote(part) for part in datum_cli_prefix())}
            with patch.dict(os.environ, env, clear=False):
                host = StdioToolHost(EngineDaemonClient())
                net = seed_board_net(host, root)
                via = call_tool(
                    host,
                    "datum.pcb.place_via",
                    {
                        "path": str(root),
                        "net": net,
                        "x_nm": 5000,
                        "y_nm": 6000,
                        "drill_nm": 300000,
                        "diameter_nm": 600000,
                        "from_layer": 1,
                        "to_layer": 2,
                    },
                )["via_uuid"]
                vias = query_result(host, "datum.query.board_vias", root)
                self.assertEqual(len(vias), 1)
                self.assertEqual(vias[0]["uuid"], via)
                self.assertEqual(vias[0]["net"], net)
                self.assertEqual(vias[0]["position"], {"x": 5000, "y": 6000})
                self.assertEqual(vias[0]["drill"], 300000)
                self.assertEqual(vias[0]["diameter"], 600000)
                operation = assert_latest_journal_operation(
                    self, host, str(root), "place board via", "create_board_via"
                )
                self.assertEqual(operation["via_id"], via)
                self.assertEqual(operation["via"]["position"], {"x": 5000, "y": 6000})
                self.assertEqual(call_tool(host, "datum.journal.undo", {"path": str(root)})["status"], "applied")
                self.assertEqual(query_result(host, "datum.query.board_vias", root), [])
                self.assertEqual(call_tool(host, "datum.journal.redo", {"path": str(root)})["status"], "applied")
                self.assertEqual(query_result(host, "datum.query.board_vias", root)[0]["uuid"], via)
                self.assertEqual(
                    call_tool(host, "datum.pcb.delete_via", {"path": str(root), "via": via})["action"],
                    "delete_board_via",
                )
                self.assertEqual(query_result(host, "datum.query.board_vias", root), [])
                operation = assert_latest_journal_operation(
                    self, host, str(root), "delete board via", "delete_board_via"
                )
                self.assertEqual(operation["via_id"], via)

    def test_pcb_pad_tools_call_writes_deletes_and_replays(self) -> None:
        with tempfile.TemporaryDirectory(prefix="datum-mcp-pad-") as tmp:
            root = Path(tmp)
            run_cli_json(root, "project", "new", str(root), "--name", "Pad Parity")
            env = {"DATUM_CLI_BIN": " ".join(shlex.quote(part) for part in datum_cli_prefix())}
            with patch.dict(os.environ, env, clear=False):
                host = StdioToolHost(EngineDaemonClient())
                net = seed_board_net(host, root)
                package = "11111111-1111-4111-8111-111111111111"
                pad = call_tool(
                    host,
                    "datum.pcb.place_pad",
                    {
                        "path": str(root),
                        "package": package,
                        "name": "1",
                        "x_nm": 7000,
                        "y_nm": 8000,
                        "layer": 3,
                        "diameter_nm": 700000,
                        "net": net,
                    },
                )["pad_uuid"]
                pads = query_result(host, "datum.query.board_pads", root)
                self.assertEqual(len(pads), 1)
                self.assertEqual(pads[0]["uuid"], pad)
                self.assertEqual(pads[0]["package"], package)
                self.assertEqual(pads[0]["name"], "1")
                self.assertEqual(pads[0]["net"], net)
                self.assertEqual(pads[0]["position"], {"x": 7000, "y": 8000})
                self.assertEqual(pads[0]["diameter"], 700000)
                operation = assert_latest_journal_operation(
                    self, host, str(root), "place board pad", "create_board_pad"
                )
                self.assertEqual(operation["pad_id"], pad)
                self.assertEqual(operation["pad"]["package"], package)
                self.assertEqual(operation["pad"]["position"], {"x": 7000, "y": 8000})
                self.assertEqual(call_tool(host, "datum.journal.undo", {"path": str(root)})["status"], "applied")
                self.assertEqual(query_result(host, "datum.query.board_pads", root), [])
                self.assertEqual(call_tool(host, "datum.journal.redo", {"path": str(root)})["status"], "applied")
                self.assertEqual(query_result(host, "datum.query.board_pads", root)[0]["uuid"], pad)
                self.assertEqual(
                    call_tool(host, "datum.pcb.delete_pad", {"path": str(root), "pad": pad})["action"],
                    "delete_board_pad",
                )
                self.assertEqual(query_result(host, "datum.query.board_pads", root), [])
                operation = assert_latest_journal_operation(
                    self, host, str(root), "delete board pad", "delete_board_pad"
                )
                self.assertEqual(operation["pad_id"], pad)


if __name__ == "__main__":
    unittest.main()
