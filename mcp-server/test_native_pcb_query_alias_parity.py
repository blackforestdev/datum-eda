#!/usr/bin/env python3
"""Native readback parity for PCB layout query aliases."""

from __future__ import annotations

import os
import shlex
import tempfile
import unittest
from pathlib import Path
from unittest.mock import patch

from server_runtime import EngineDaemonClient, StdioToolHost
from test_native_pcb_stackup_parity import CUSTOM_STACKUP
from test_native_write_parity import call_tool, datum_cli_prefix, query_result, run_cli_json, seed_board_net


class TestNativePcbQueryAliasParity(unittest.TestCase):
    def test_pcb_layout_query_aliases_read_native_project_state(self) -> None:
        with tempfile.TemporaryDirectory(prefix="datum-mcp-pcb-query-alias-") as tmp:
            root = Path(tmp)
            run_cli_json(root, "project", "new", str(root), "--name", "PCB Query Alias Parity")
            env = {"DATUM_CLI_BIN": " ".join(shlex.quote(part) for part in datum_cli_prefix())}
            with patch.dict(os.environ, env, clear=False):
                host = StdioToolHost(EngineDaemonClient())
                call_tool(
                    host,
                    "datum.pcb.set_outline",
                    {"path": str(root), "vertices": ["0:0", "2000:0", "2000:1000", "0:1000"]},
                )
                call_tool(host, "datum.pcb.set_stackup", {"path": str(root), "layers": CUSTOM_STACKUP})
                keepout = call_tool(
                    host,
                    "datum.pcb.place_keepout",
                    {"path": str(root), "vertices": ["0:0", "100:0", "100:100"], "layers": [1], "kind": "copper"},
                )["keepout_uuid"]
                dimension = call_tool(
                    host,
                    "datum.pcb.place_dimension",
                    {"path": str(root), "from_x_nm": 0, "from_y_nm": 0, "to_x_nm": 1000, "to_y_nm": 500, "layer": 41},
                )["dimension_uuid"]
                text = call_tool(
                    host,
                    "datum.pcb.place_text",
                    {"path": str(root), "text": "REF**", "x_nm": 10, "y_nm": 20, "layer": 1},
                )["text_uuid"]
                run_cli_json(
                    root,
                    "project",
                    "create-pool-unit",
                    str(root),
                    "--pool",
                    "pool",
                    "--unit",
                    "11111111-1111-1111-1111-111111111111",
                    "--name",
                    "QueryAliasUnit",
                    "--manufacturer",
                    "Datum",
                )
                run_cli_json(
                    root,
                    "project",
                    "create-pool-symbol",
                    str(root),
                    "--pool",
                    "pool",
                    "--symbol",
                    "22222222-2222-2222-2222-222222222222",
                    "--unit",
                    "11111111-1111-1111-1111-111111111111",
                    "--name",
                    "QueryAliasSymbol",
                )
                sheet_uuid = "33333333-3333-3333-3333-333333333333"
                definition_uuid = "44444444-4444-4444-4444-444444444444"
                run_cli_json(
                    root,
                    "project",
                    "create-sheet",
                    str(root),
                    "--name",
                    "MCP Source Shard Sheet",
                    "--sheet",
                    sheet_uuid,
                )
                run_cli_json(
                    root,
                    "project",
                    "create-sheet-definition",
                    str(root),
                    "--root-sheet",
                    sheet_uuid,
                    "--name",
                    "MCP Source Shard Definition",
                    "--definition",
                    definition_uuid,
                )
                net = seed_board_net(host, root)
                self.assertEqual(len(query_result(host, "datum.query.board_outline", root)["vertices"]), 4)
                self.assertEqual(query_result(host, "datum.query.board_stackup", root)[1]["name"], "Core")
                self.assertEqual(query_result(host, "datum.query.board_keepouts", root)[0]["uuid"], keepout)
                self.assertEqual(query_result(host, "datum.query.board_dimensions", root)[0]["uuid"], dimension)
                self.assertEqual(query_result(host, "datum.query.board_texts", root)[0]["uuid"], text)
                self.assertEqual(query_result(host, "datum.query.board_nets", root)[0]["uuid"], net)
                self.assertEqual(query_result(host, "datum.query.board_net_classes", root)[0]["name"], "Default")
                source_shards = query_result(host, "datum.query.source_shards", root)["source_shards"]
                self.assertTrue(
                    any(
                        shard["path"] == "board/board.json"
                        and shard["kind"] == "BoardRoot"
                        and shard["dirty_state"] == "Clean"
                        for shard in source_shards
                    )
                )
                self.assertTrue(
                    any(
                        shard["path"] == "pool/symbols/22222222-2222-2222-2222-222222222222.json"
                        and shard["kind"] == "Pool"
                        and shard.get("taxon") == "PoolSymbol"
                        for shard in source_shards
                    )
                )
                self.assertTrue(
                    any(
                        shard["path"] == "schematic/definitions/44444444-4444-4444-4444-444444444444.json"
                        and shard["kind"] == "SchematicDefinition"
                        and shard["authority"] == "AuthoredDesign"
                        and shard["dirty_state"] == "Clean"
                        for shard in source_shards
                    )
                )
                (root / "pool/symbols/22222222-2222-2222-2222-222222222222.json").unlink()
                replayed_source_shards = query_result(host, "datum.query.source_shards", root)["source_shards"]
                self.assertTrue(
                    any(
                        shard["path"] == "pool/symbols/22222222-2222-2222-2222-222222222222.json"
                        and shard["kind"] == "Pool"
                        and shard.get("taxon") == "PoolSymbol"
                        and shard["authority"] == "AuthoredDesign"
                        and shard["dirty_state"] == "Missing"
                        for shard in replayed_source_shards
                    )
                )
                (root / "pool/symbols/22222222-2222-2222-2222-222222222222.json").mkdir()
                unknown_source_shards = query_result(host, "datum.query.source_shards", root)["source_shards"]
                self.assertTrue(
                    any(
                        shard["path"] == "pool/symbols/22222222-2222-2222-2222-222222222222.json"
                        and shard["kind"] == "Pool"
                        and shard.get("taxon") == "PoolSymbol"
                        and shard["authority"] == "AuthoredDesign"
                        and shard["dirty_state"] == "Unknown"
                        for shard in unknown_source_shards
                    )
                )
                definition_path = root / "schematic/definitions/44444444-4444-4444-4444-444444444444.json"
                definition_path.unlink()
                definition_path.mkdir()
                unknown_definition_shards = query_result(host, "datum.query.source_shards", root)["source_shards"]
                self.assertTrue(
                    any(
                        shard["path"] == "schematic/definitions/44444444-4444-4444-4444-444444444444.json"
                        and shard["kind"] == "SchematicDefinition"
                        and shard["authority"] == "AuthoredDesign"
                        and shard["dirty_state"] == "Unknown"
                        for shard in unknown_definition_shards
                    )
                )


if __name__ == "__main__":
    unittest.main()
