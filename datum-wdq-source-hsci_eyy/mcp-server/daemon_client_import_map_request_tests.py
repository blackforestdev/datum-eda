#!/usr/bin/env python3
"""EngineDaemonClient import map CLI bridge tests."""

from __future__ import annotations

import subprocess
import unittest
from unittest.mock import patch

from server_runtime import EngineDaemonClient


class TestDaemonClientImportMapRequests(unittest.TestCase):
    @patch("server_runtime.subprocess.run")
    def test_lists_import_map_via_cli(self, run_mock) -> None:
        run_mock.return_value = subprocess.CompletedProcess(
            args=[],
            returncode=0,
            stdout='{"contract":"import_map_query_v1","import_map_count":1}',
            stderr="",
        )
        response = EngineDaemonClient().get_import_map("/tmp/native-project")
        run_mock.assert_called_once_with(
            ["datum-eda", "--format", "json", "query", "import-map", "/tmp/native-project"],
            capture_output=True,
            text=True,
            check=False,
        )
        self.assertEqual(response.result["contract"], "import_map_query_v1")
