#!/usr/bin/env python3
"""EngineDaemonClient proposal CLI bridge tests."""

from __future__ import annotations

import subprocess
import unittest
from unittest.mock import patch

from server_runtime import EngineDaemonClient


class TestDaemonClientProposalRequests(unittest.TestCase):
    @patch("server_runtime.subprocess.run")
    def test_creates_proposal_via_cli(self, run_mock) -> None:
        run_mock.return_value = subprocess.CompletedProcess(
            args=[],
            returncode=0,
            stdout='{"contract":"proposal_create_v1","proposal_id":"proposal-test"}',
            stderr="",
        )
        response = EngineDaemonClient().create_proposal(
            "/tmp/native-project",
            "/tmp/batch.json",
            "review batch",
            "proposal-test",
            "assistant",
            ["check-test"],
            ["sha256:test"],
        )
        run_mock.assert_called_once_with(
            ["datum-eda", "--format", "json", "proposal", "create", "/tmp/native-project", "--batch", "/tmp/batch.json", "--rationale", "review batch", "--proposal", "proposal-test", "--source", "assistant", "--check-run", "check-test", "--finding-fingerprint", "sha256:test"],
            capture_output=True,
            text=True,
            check=False,
        )
        self.assertEqual(response.result["contract"], "proposal_create_v1")

    @patch("server_runtime.subprocess.run")
    def test_creates_draw_wire_proposal_via_cli(self, run_mock) -> None:
        run_mock.return_value = subprocess.CompletedProcess(args=[], returncode=0, stdout='{"contract":"proposal_create_v1","action":"propose_draw_wire"}', stderr="")
        response = EngineDaemonClient().create_draw_wire_proposal("/tmp/native-project", "sheet-uuid", 0, 10, 100, 110, "proposal-wire", "review wire")
        run_mock.assert_called_once_with(
            ["datum-eda", "--format", "json", "proposal", "create-draw-wire", "/tmp/native-project", "--sheet", "sheet-uuid", "--from-x-nm", "0", "--from-y-nm", "10", "--to-x-nm", "100", "--to-y-nm", "110", "--proposal", "proposal-wire", "--rationale", "review wire"],
            capture_output=True,
            text=True,
            check=False,
        )
        self.assertEqual(response.result["action"], "propose_draw_wire")

    @patch("server_runtime.subprocess.run")
    def test_creates_place_label_proposal_via_cli(self, run_mock) -> None:
        run_mock.return_value = subprocess.CompletedProcess(args=[], returncode=0, stdout='{"contract":"proposal_create_v1","action":"propose_place_label"}', stderr="")
        response = EngineDaemonClient().create_place_label_proposal("/tmp/native-project", "sheet-uuid", "VCC", 100, 200, "power", "proposal-label", "review label")
        run_mock.assert_called_once_with(
            ["datum-eda", "--format", "json", "proposal", "create-place-label", "/tmp/native-project", "--sheet", "sheet-uuid", "--name", "VCC", "--x-nm", "100", "--y-nm", "200", "--kind", "power", "--proposal", "proposal-label", "--rationale", "review label"],
            capture_output=True,
            text=True,
            check=False,
        )
        self.assertEqual(response.result["action"], "propose_place_label")

    @patch("server_runtime.subprocess.run")
    def test_creates_place_symbol_proposal_via_cli(self, run_mock) -> None:
        run_mock.return_value = subprocess.CompletedProcess(args=[], returncode=0, stdout='{"contract":"proposal_create_v1","action":"propose_place_symbol"}', stderr="")
        response = EngineDaemonClient().create_place_symbol_proposal("/tmp/native-project", "sheet-uuid", "U1", "OPA", 100, 200, "Device:R", 90, True, "proposal-symbol", "review symbol")
        run_mock.assert_called_once_with(
            ["datum-eda", "--format", "json", "proposal", "create-place-symbol", "/tmp/native-project", "--sheet", "sheet-uuid", "--reference", "U1", "--value", "OPA", "--x-nm", "100", "--y-nm", "200", "--lib-id", "Device:R", "--rotation-deg", "90", "--mirrored", "--proposal", "proposal-symbol", "--rationale", "review symbol"],
            capture_output=True,
            text=True,
            check=False,
        )
        self.assertEqual(response.result["action"], "propose_place_symbol")

    @patch("server_runtime.subprocess.run")
    def test_creates_board_component_replacement_proposal_via_cli(self, run_mock) -> None:
        run_mock.return_value = subprocess.CompletedProcess(args=[], returncode=0, stdout='{"contract":"proposal_create_v1","action":"propose_board_component_replacement"}', stderr="")
        response = EngineDaemonClient().create_board_component_replacement_proposal("/tmp/native-project", "component-uuid", "package-uuid", "part-uuid", "10k", "proposal-replace", "review replacement")
        run_mock.assert_called_once_with(
            ["datum-eda", "--format", "json", "proposal", "create-board-component-replacement", "/tmp/native-project", "--component", "component-uuid", "--package", "package-uuid", "--part", "part-uuid", "--value", "10k", "--proposal", "proposal-replace", "--rationale", "review replacement"],
            capture_output=True,
            text=True,
            check=False,
        )
        self.assertEqual(response.result["action"], "propose_board_component_replacement")

    @patch("server_runtime.subprocess.run")
    def test_creates_board_component_replacements_proposal_via_cli(self, run_mock) -> None:
        run_mock.return_value = subprocess.CompletedProcess(args=[], returncode=0, stdout='{"contract":"proposal_create_v1","action":"propose_board_component_replacement"}', stderr="")
        replacements = [
            {"component": "component-u1", "package": "package-u1", "part": "part-u1", "value": "10k"},
            {"component": "component-u2", "part": "part-u2", "value": "22k"},
        ]
        response = EngineDaemonClient().create_board_component_replacements_proposal("/tmp/native-project", replacements, "proposal-replacements", "review replacements")
        run_mock.assert_called_once_with(
            [
                "datum-eda",
                "--format",
                "json",
                "proposal",
                "create-board-component-replacements",
                "/tmp/native-project",
                "--replacement",
                '{"component":"component-u1","package":"package-u1","part":"part-u1","value":"10k"}',
                "--replacement",
                '{"component":"component-u2","part":"part-u2","value":"22k"}',
                "--proposal",
                "proposal-replacements",
                "--rationale",
                "review replacements",
            ],
            capture_output=True,
            text=True,
            check=False,
        )
        self.assertEqual(response.result["action"], "propose_board_component_replacement")

    @patch("server_runtime.subprocess.run")
    def test_creates_board_component_replacement_plan_proposal_via_cli(self, run_mock) -> None:
        run_mock.return_value = subprocess.CompletedProcess(args=[], returncode=0, stdout='{"contract":"proposal_create_v1","action":"propose_board_component_replacement"}', stderr="")
        selections = [
            {"uuid": "component-u1", "package_uuid": "package-u1", "part_uuid": "part-u1", "value": "10k"},
            {"uuid": "component-u2", "part_uuid": "part-u2"},
        ]
        response = EngineDaemonClient().create_board_component_replacement_plan_proposal("/tmp/native-project", selections, "proposal-plan", "review plan")
        run_mock.assert_called_once_with(
            [
                "datum-eda",
                "--format",
                "json",
                "proposal",
                "create-board-component-replacement-plan",
                "/tmp/native-project",
                "--selection",
                '{"uuid":"component-u1","package_uuid":"package-u1","part_uuid":"part-u1","value":"10k"}',
                "--selection",
                '{"uuid":"component-u2","part_uuid":"part-u2"}',
                "--proposal",
                "proposal-plan",
                "--rationale",
                "review plan",
            ],
            capture_output=True,
            text=True,
            check=False,
        )
        self.assertEqual(response.result["action"], "propose_board_component_replacement")

    @patch("server_runtime.subprocess.run")
    def test_binds_component_instance_proposal_via_cli(self, run_mock) -> None:
        run_mock.return_value = subprocess.CompletedProcess(args=[], returncode=0, stdout='{"contract":"proposal_create_v1","action":"propose_bind_component_instance"}', stderr="")
        response = EngineDaemonClient().bind_component_instance_proposal(
            "/tmp/native-project",
            None,
            "pkg-test",
            "ci-test",
            ["sym-a", "sym-b"],
            "part-test",
            {"sym-a": {"role": "logical_unit", "label": "A"}},
            {"pkg-test": {"role": "physical_package", "label": "main"}},
            "proposal-bind",
            "review bind",
        )
        run_mock.assert_called_once_with(
            [
                "datum-eda",
                "--format",
                "json",
                "proposal",
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
                "--part",
                "part-test",
                "--symbol-role",
                "sym-a=logical_unit:A",
                "--package-role",
                "pkg-test=physical_package:main",
                "--proposal",
                "proposal-bind",
                "--rationale",
                "review bind",
            ],
            capture_output=True,
            text=True,
            check=False,
        )
        self.assertEqual(response.result["action"], "propose_bind_component_instance")

    @patch("server_runtime.subprocess.run")
    def test_sets_component_instance_proposal_via_cli(self, run_mock) -> None:
        run_mock.return_value = subprocess.CompletedProcess(args=[], returncode=0, stdout='{"contract":"proposal_create_v1","action":"propose_set_component_instance"}', stderr="")
        response = EngineDaemonClient().set_component_instance_proposal(
            "/tmp/native-project",
            "ci-test",
            "sym-next",
            "pkg-next",
            proposal="proposal-set",
            rationale="review set",
        )
        run_mock.assert_called_once_with(
            [
                "datum-eda",
                "--format",
                "json",
                "proposal",
                "set-component-instance",
                "/tmp/native-project",
                "--component-instance",
                "ci-test",
                "--symbol",
                "sym-next",
                "--package",
                "pkg-next",
                "--proposal",
                "proposal-set",
                "--rationale",
                "review set",
            ],
            capture_output=True,
            text=True,
            check=False,
        )
        self.assertEqual(response.result["action"], "propose_set_component_instance")

    @patch("server_runtime.subprocess.run")
    def test_deletes_component_instance_proposal_via_cli(self, run_mock) -> None:
        run_mock.return_value = subprocess.CompletedProcess(args=[], returncode=0, stdout='{"contract":"proposal_create_v1","action":"propose_delete_component_instance"}', stderr="")
        response = EngineDaemonClient().delete_component_instance_proposal(
            "/tmp/native-project",
            "ci-test",
            "proposal-delete",
            "review delete",
        )
        run_mock.assert_called_once_with(
            [
                "datum-eda",
                "--format",
                "json",
                "proposal",
                "delete-component-instance",
                "/tmp/native-project",
                "--component-instance",
                "ci-test",
                "--proposal",
                "proposal-delete",
                "--rationale",
                "review delete",
            ],
            capture_output=True,
            text=True,
            check=False,
        )
        self.assertEqual(response.result["action"], "propose_delete_component_instance")

    @patch("server_runtime.subprocess.run")
    def test_creates_pool_pin_pad_map_proposal_via_cli(self, run_mock) -> None:
        run_mock.return_value = subprocess.CompletedProcess(args=[], returncode=0, stdout='{"contract":"proposal_create_v1","action":"create_pool_pin_pad_map_proposal"}', stderr="")
        response = EngineDaemonClient().create_pool_pin_pad_map_proposal("/tmp/native-project", "map-uuid", "part-uuid", ["pin-uuid:pad-uuid"], "footprint-uuid", True, "pool", "proposal-pin-pad-map", "review pin pad map")
        run_mock.assert_called_once_with(
            ["datum-eda", "--format", "json", "proposal", "create-pool-pin-pad-map", "/tmp/native-project", "--map", "map-uuid", "--part", "part-uuid", "--footprint", "footprint-uuid", "--entry", "pin-uuid:pad-uuid", "--set-default", "--pool", "pool", "--proposal", "proposal-pin-pad-map", "--rationale", "review pin pad map"],
            capture_output=True,
            text=True,
            check=False,
        )
        self.assertEqual(response.result["action"], "create_pool_pin_pad_map_proposal")

    @patch("server_runtime.subprocess.run")
    def test_sets_pool_pin_pad_map_proposal_via_cli(self, run_mock) -> None:
        run_mock.return_value = subprocess.CompletedProcess(args=[], returncode=0, stdout='{"contract":"proposal_create_v1","action":"set_pool_pin_pad_map_proposal"}', stderr="")
        response = EngineDaemonClient().set_pool_pin_pad_map_proposal("/tmp/native-project", "map-uuid", "replace", ["pad-uuid:gate-uuid:pin-uuid"], "pool", "proposal-pin-pad-map-set", "review pin pad map update")
        run_mock.assert_called_once_with(
            ["datum-eda", "--format", "json", "proposal", "set-pool-pin-pad-map", "/tmp/native-project", "--map", "map-uuid", "--mode", "replace", "--entry", "pad-uuid:gate-uuid:pin-uuid", "--pool", "pool", "--proposal", "proposal-pin-pad-map-set", "--rationale", "review pin pad map update"],
            capture_output=True,
            text=True,
            check=False,
        )
        self.assertEqual(response.result["action"], "set_pool_pin_pad_map_proposal")

    @patch("server_runtime.subprocess.run")
    def test_lists_proposals_via_cli(self, run_mock) -> None:
        run_mock.return_value = subprocess.CompletedProcess(
            args=[],
            returncode=0,
            stdout='{"contract":"proposals_query_v1","proposal_count":1}',
            stderr="",
        )
        response = EngineDaemonClient().get_proposals("/tmp/native-project")
        run_mock.assert_called_once_with(
            ["datum-eda", "--format", "json", "proposal", "list", "/tmp/native-project"],
            capture_output=True,
            text=True,
            check=False,
        )
        self.assertEqual(response.result["contract"], "proposals_query_v1")

    @patch("server_runtime.subprocess.run")
    def test_reviews_proposal_via_cli(self, run_mock) -> None:
        run_mock.return_value = subprocess.CompletedProcess(args=[], returncode=0, stdout='{"action":"review_proposal","status":"accepted"}', stderr="")
        response = EngineDaemonClient().review_proposal("/tmp/native-project", "proposal-test", "accepted")
        run_mock.assert_called_once_with(
            ["datum-eda", "--format", "json", "proposal", "review", "/tmp/native-project", "--proposal", "proposal-test", "--status", "accepted"],
            capture_output=True,
            text=True,
            check=False,
        )
        self.assertEqual(response.result["status"], "accepted")

    @patch("server_runtime.subprocess.run")
    def test_shows_proposal_via_cli(self, run_mock) -> None:
        run_mock.return_value = subprocess.CompletedProcess(args=[], returncode=0, stdout='{"contract":"proposal_show_v1"}', stderr="")
        response = EngineDaemonClient().show_proposal("/tmp/native-project", "proposal-test")
        run_mock.assert_called_once_with(
            ["datum-eda", "--format", "json", "proposal", "show", "/tmp/native-project", "--proposal", "proposal-test"],
            capture_output=True,
            text=True,
            check=False,
        )
        self.assertEqual(response.result["contract"], "proposal_show_v1")

    @patch("server_runtime.subprocess.run")
    def test_previews_proposal_via_cli(self, run_mock) -> None:
        run_mock.return_value = subprocess.CompletedProcess(args=[], returncode=0, stdout='{"contract":"proposal_preview_v1"}', stderr="")
        response = EngineDaemonClient().preview_proposal("/tmp/native-project", "proposal-test")
        run_mock.assert_called_once_with(
            ["datum-eda", "--format", "json", "proposal", "preview", "/tmp/native-project", "--proposal", "proposal-test"],
            capture_output=True,
            text=True,
            check=False,
        )
        self.assertEqual(response.result["contract"], "proposal_preview_v1")

    @patch("server_runtime.subprocess.run")
    def test_validates_proposal_via_cli(self, run_mock) -> None:
        run_mock.return_value = subprocess.CompletedProcess(args=[], returncode=0, stdout='{"contract":"proposal_validation_v1","can_apply":false}', stderr="")
        response = EngineDaemonClient().validate_proposal("/tmp/native-project", "proposal-test")
        run_mock.assert_called_once_with(
            ["datum-eda", "--format", "json", "proposal", "validate", "/tmp/native-project", "--proposal", "proposal-test"],
            capture_output=True,
            text=True,
            check=False,
        )
        self.assertEqual(response.result["contract"], "proposal_validation_v1")

    @patch("server_runtime.subprocess.run")
    def test_defers_proposal_via_cli(self, run_mock) -> None:
        run_mock.return_value = subprocess.CompletedProcess(args=[], returncode=0, stdout='{"action":"review_proposal","status":"deferred"}', stderr="")
        response = EngineDaemonClient().defer_proposal("/tmp/native-project", "proposal-test")
        run_mock.assert_called_once_with(
            ["datum-eda", "--format", "json", "proposal", "defer", "/tmp/native-project", "--proposal", "proposal-test"],
            capture_output=True,
            text=True,
            check=False,
        )
        self.assertEqual(response.result["status"], "deferred")

    @patch("server_runtime.subprocess.run")
    def test_applies_proposal_via_cli(self, run_mock) -> None:
        run_mock.return_value = subprocess.CompletedProcess(args=[], returncode=0, stdout='{"action":"apply_proposal","status":"applied"}', stderr="")
        response = EngineDaemonClient().apply_proposal("/tmp/native-project", "proposal-test")
        run_mock.assert_called_once_with(
            ["datum-eda", "--format", "json", "proposal", "apply", "/tmp/native-project", "--proposal", "proposal-test"],
            capture_output=True,
            text=True,
            check=False,
        )
        self.assertEqual(response.result["status"], "applied")
