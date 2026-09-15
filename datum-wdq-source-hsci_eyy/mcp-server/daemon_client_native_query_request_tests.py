#!/usr/bin/env python3
"""Daemon-client request tests for native object query aliases."""

from __future__ import annotations

import subprocess
import unittest
from unittest.mock import patch

from server_runtime import EngineDaemonClient


def assert_project_query(test: unittest.TestCase, run_mock, method: str, query: str) -> None:
    run_mock.return_value = subprocess.CompletedProcess(
        args=[],
        returncode=0,
        stdout='[{"uuid":"object-test"}]',
        stderr="",
    )
    response = getattr(EngineDaemonClient(), method)("/tmp/native-project")
    test.assertEqual(response.result, [{"uuid": "object-test"}])
    run_mock.assert_called_once_with(
        [
            "datum-eda",
            "--format",
            "json",
            "project",
            "query",
            "/tmp/native-project",
            query,
        ],
        capture_output=True,
        text=True,
        check=False,
    )


class TestDaemonClientNativeQueryRequests(unittest.TestCase):
    @patch("server_runtime.subprocess.run")
    def test_schematic_wires_query_uses_project_cli(self, run_mock) -> None:
        run_mock.return_value = subprocess.CompletedProcess(
            args=[],
            returncode=0,
            stdout='[{"uuid":"wire-test"}]',
            stderr="",
        )
        response = EngineDaemonClient().get_schematic_wires("/tmp/native-project")
        self.assertEqual(response.result, [{"uuid": "wire-test"}])
        run_mock.assert_called_once_with(
            [
                "datum-eda",
                "--format",
                "json",
                "project",
                "query",
                "/tmp/native-project",
                "wires",
            ],
            capture_output=True,
            text=True,
            check=False,
        )

    @patch("server_runtime.subprocess.run")
    def test_board_tracks_query_uses_project_cli(self, run_mock) -> None:
        run_mock.return_value = subprocess.CompletedProcess(
            args=[],
            returncode=0,
            stdout='[{"uuid":"track-test"}]',
            stderr="",
        )
        response = EngineDaemonClient().get_board_tracks("/tmp/native-project")
        self.assertEqual(response.result, [{"uuid": "track-test"}])
        run_mock.assert_called_once_with(
            [
                "datum-eda",
                "--format",
                "json",
                "project",
                "query",
                "/tmp/native-project",
                "board-tracks",
            ],
            capture_output=True,
            text=True,
            check=False,
        )

    @patch("server_runtime.subprocess.run")
    def test_schematic_junctions_query_uses_project_cli(self, run_mock) -> None:
        assert_project_query(self, run_mock, "get_schematic_junctions", "junctions")

    @patch("server_runtime.subprocess.run")
    def test_schematic_labels_query_uses_project_cli(self, run_mock) -> None:
        assert_project_query(self, run_mock, "get_schematic_labels", "labels")

    @patch("server_runtime.subprocess.run")
    def test_schematic_ports_query_uses_project_cli(self, run_mock) -> None:
        assert_project_query(self, run_mock, "get_schematic_ports", "ports")

    @patch("server_runtime.subprocess.run")
    def test_schematic_noconnects_query_uses_project_cli(self, run_mock) -> None:
        assert_project_query(self, run_mock, "get_schematic_noconnects", "noconnects")

    @patch("server_runtime.subprocess.run")
    def test_schematic_buses_query_uses_project_cli(self, run_mock) -> None:
        assert_project_query(self, run_mock, "get_schematic_buses", "buses")

    @patch("server_runtime.subprocess.run")
    def test_schematic_bus_entries_query_uses_project_cli(self, run_mock) -> None:
        assert_project_query(self, run_mock, "get_schematic_bus_entries", "bus-entries")

    @patch("server_runtime.subprocess.run")
    def test_schematic_texts_query_uses_project_cli(self, run_mock) -> None:
        assert_project_query(self, run_mock, "get_schematic_texts", "texts")

    @patch("server_runtime.subprocess.run")
    def test_schematic_drawings_query_uses_project_cli(self, run_mock) -> None:
        assert_project_query(self, run_mock, "get_schematic_drawings", "drawings")

    @patch("server_runtime.subprocess.run")
    def test_board_vias_query_uses_project_cli(self, run_mock) -> None:
        assert_project_query(self, run_mock, "get_board_vias", "board-vias")

    @patch("server_runtime.subprocess.run")
    def test_board_pads_query_uses_project_cli(self, run_mock) -> None:
        assert_project_query(self, run_mock, "get_board_pads", "board-pads")

    @patch("server_runtime.subprocess.run")
    def test_board_zones_query_uses_project_cli(self, run_mock) -> None:
        assert_project_query(self, run_mock, "get_board_zones", "board-zones")
    @patch("server_runtime.subprocess.run")
    def test_board_layout_queries_use_project_cli(self, run_mock) -> None:
        for method, query in [
            ("get_board_texts", "board-texts"),
            ("get_board_keepouts", "board-keepouts"),
            ("get_board_outline", "board-outline"),
            ("get_board_stackup", "board-stackup"),
            ("get_board_dimensions", "board-dimensions"),
            ("get_board_nets", "board-nets"),
            ("get_board_net_classes", "board-net-classes"),
        ]:
            with self.subTest(method=method):
                run_mock.reset_mock()
                assert_project_query(self, run_mock, method, query)


if __name__ == "__main__":
    unittest.main()
