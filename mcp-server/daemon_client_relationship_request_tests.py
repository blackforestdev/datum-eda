#!/usr/bin/env python3
"""EngineDaemonClient relationship/variant CLI bridge tests."""

from __future__ import annotations

import subprocess
import unittest
from unittest.mock import patch

from server_runtime import EngineDaemonClient


class TestDaemonClientRelationshipRequests(unittest.TestCase):
    @patch("server_runtime.subprocess.run")
    def test_lists_component_instances_via_cli(self, run_mock) -> None:
        run_mock.return_value = subprocess.CompletedProcess(
            args=[],
            returncode=0,
            stdout='{"contract":"component_instances_query_v1","component_instance_count":1}',
            stderr="",
        )
        response = EngineDaemonClient().get_component_instances("/tmp/native-project")
        run_mock.assert_called_once_with(
            ["datum-eda", "--format", "json", "query", "component-instances", "/tmp/native-project"],
            capture_output=True,
            text=True,
            check=False,
        )
        self.assertEqual(response.result["contract"], "component_instances_query_v1")

    @patch("server_runtime.subprocess.run")
    def test_binds_component_instance_via_cli(self, run_mock) -> None:
        run_mock.return_value = subprocess.CompletedProcess(
            args=[],
            returncode=0,
            stdout='{"action":"bind_component_instance","component_instance":"ci-test"}',
            stderr="",
        )
        response = EngineDaemonClient().bind_component_instance(
            "/tmp/native-project",
            "sym-test",
            "pkg-test",
            "ci-test",
            part="part-test",
        )
        run_mock.assert_called_once_with(
            [
                "datum-eda",
                "--format",
                "json",
                "project",
                "bind-component-instance",
                "/tmp/native-project",
                "--symbol",
                "sym-test",
                "--package",
                "pkg-test",
                "--part",
                "part-test",
                "--component-instance",
                "ci-test",
            ],
            capture_output=True,
            text=True,
            check=False,
        )
        self.assertEqual(response.result["component_instance"], "ci-test")

    @patch("server_runtime.subprocess.run")
    def test_binds_multi_symbol_component_instance_via_cli(self, run_mock) -> None:
        run_mock.return_value = subprocess.CompletedProcess(
            args=[],
            returncode=0,
            stdout='{"action":"bind_component_instance","component_instance":"ci-test"}',
            stderr="",
        )
        EngineDaemonClient().bind_component_instance(
            "/tmp/native-project",
            None,
            "pkg-test",
            "ci-test",
            symbols=["sym-a", "sym-b"],
        )
        run_mock.assert_called_once_with(
            [
                "datum-eda",
                "--format",
                "json",
                "project",
                "bind-component-instance",
                "/tmp/native-project",
                "--symbol",
                "sym-a",
                "--symbol",
                "sym-b",
                "--package",
                "pkg-test",
                "--component-instance",
                "ci-test",
            ],
            capture_output=True,
            text=True,
            check=False,
        )

    @patch("server_runtime.subprocess.run")
    def test_binds_component_instance_roles_via_cli(self, run_mock) -> None:
        run_mock.return_value = subprocess.CompletedProcess(
            args=[],
            returncode=0,
            stdout='{"action":"bind_component_instance","component_instance":"ci-test"}',
            stderr="",
        )
        EngineDaemonClient().bind_component_instance(
            "/tmp/native-project",
            "sym-test",
            "pkg-test",
            symbol_roles={"sym-test": {"role": "logical_unit", "label": "A"}},
            package_roles={"pkg-test": {"role": "physical_package"}},
        )
        run_mock.assert_called_once_with(
            [
                "datum-eda",
                "--format",
                "json",
                "project",
                "bind-component-instance",
                "/tmp/native-project",
                "--symbol",
                "sym-test",
                "--package",
                "pkg-test",
                "--symbol-role",
                "sym-test=logical_unit:A",
                "--package-role",
                "pkg-test=physical_package",
            ],
            capture_output=True,
            text=True,
            check=False,
        )

    @patch("server_runtime.subprocess.run")
    def test_sets_component_instance_via_cli(self, run_mock) -> None:
        run_mock.return_value = subprocess.CompletedProcess(
            args=[],
            returncode=0,
            stdout='{"action":"set_component_instance","component_instance":"ci-test"}',
            stderr="",
        )
        response = EngineDaemonClient().set_component_instance(
            "/tmp/native-project",
            "ci-test",
            "sym-next",
            "pkg-next",
            part="part-next",
        )
        run_mock.assert_called_once_with(
            [
                "datum-eda",
                "--format",
                "json",
                "project",
                "set-component-instance",
                "/tmp/native-project",
                "--component-instance",
                "ci-test",
                "--symbol",
                "sym-next",
                "--package",
                "pkg-next",
                "--part",
                "part-next",
            ],
            capture_output=True,
            text=True,
            check=False,
        )
        self.assertEqual(response.result["component_instance"], "ci-test")

    @patch("server_runtime.subprocess.run")
    def test_sets_multi_symbol_component_instance_via_cli(self, run_mock) -> None:
        run_mock.return_value = subprocess.CompletedProcess(
            args=[],
            returncode=0,
            stdout='{"action":"set_component_instance","component_instance":"ci-test"}',
            stderr="",
        )
        EngineDaemonClient().set_component_instance(
            "/tmp/native-project",
            "ci-test",
            None,
            "pkg-next",
            symbols=["sym-a", "sym-b"],
        )
        run_mock.assert_called_once_with(
            [
                "datum-eda",
                "--format",
                "json",
                "project",
                "set-component-instance",
                "/tmp/native-project",
                "--component-instance",
                "ci-test",
                "--symbol",
                "sym-a",
                "--symbol",
                "sym-b",
                "--package",
                "pkg-next",
            ],
            capture_output=True,
            text=True,
            check=False,
        )

    @patch("server_runtime.subprocess.run")
    def test_deletes_component_instance_via_cli(self, run_mock) -> None:
        run_mock.return_value = subprocess.CompletedProcess(
            args=[],
            returncode=0,
            stdout='{"action":"delete_component_instance","component_instance":"ci-test"}',
            stderr="",
        )
        response = EngineDaemonClient().delete_component_instance("/tmp/native-project", "ci-test")
        run_mock.assert_called_once_with(
            [
                "datum-eda",
                "--format",
                "json",
                "project",
                "delete-component-instance",
                "/tmp/native-project",
                "--component-instance",
                "ci-test",
            ],
            capture_output=True,
            text=True,
            check=False,
        )
        self.assertEqual(response.result["component_instance"], "ci-test")

    @patch("server_runtime.subprocess.run")
    def test_lists_relationships_via_cli(self, run_mock) -> None:
        run_mock.return_value = subprocess.CompletedProcess(
            args=[],
            returncode=0,
            stdout='{"contract":"relationships_query_v1","relationship_count":1}',
            stderr="",
        )
        response = EngineDaemonClient().get_relationships("/tmp/native-project")
        run_mock.assert_called_once_with(
            ["datum-eda", "--format", "json", "query", "relationships", "/tmp/native-project"],
            capture_output=True,
            text=True,
            check=False,
        )
        self.assertEqual(response.result["contract"], "relationships_query_v1")

    @patch("server_runtime.subprocess.run")
    def test_lists_variants_via_cli(self, run_mock) -> None:
        run_mock.return_value = subprocess.CompletedProcess(
            args=[],
            returncode=0,
            stdout='{"contract":"variants_query_v1","variant_count":1}',
            stderr="",
        )
        response = EngineDaemonClient().get_variants("/tmp/native-project")
        run_mock.assert_called_once_with(
            ["datum-eda", "--format", "json", "query", "variants", "/tmp/native-project"],
            capture_output=True,
            text=True,
            check=False,
        )
        self.assertEqual(response.result["contract"], "variants_query_v1")
