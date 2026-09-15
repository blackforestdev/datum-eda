#!/usr/bin/env python3
"""Fake daemon client import map responses for MCP tests."""

from __future__ import annotations

from server_runtime import JsonRpcResponse


class FakeDaemonClientImportMapMixin:
    def get_import_map(self, path: str) -> JsonRpcResponse:
        self.calls.append(("get_import_map", path))
        entry = {
            "import_id": "kicad:board:root",
            "id": "kicad:board:root",
            "project_id": "project-test",
            "model_revision": "model-rev-test",
            "source": "fixture.kicad_pcb",
            "source_kind": "kicad_board",
            "source_hash": "fixture-source-hash",
            "target_id": "board-root",
            "target_kind": "board",
            "status": "mapped",
        }
        return JsonRpcResponse(
            "2.0",
            142,
            {
                "contract": "import_map_query_v1",
                "project_root": path,
                "project_id": "project-test",
                "model_revision": "model-rev-test",
                "import_map_count": 1,
                "entries": {"kicad:board:root": entry},
                "import_map_entries": [entry],
            },
            None,
        )
