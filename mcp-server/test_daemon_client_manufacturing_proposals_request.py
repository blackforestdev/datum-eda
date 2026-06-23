#!/usr/bin/env python3
"""Request construction tests for manufacturing proposal bridges."""

from __future__ import annotations

import subprocess
import unittest
from unittest.mock import patch

from server_runtime import EngineDaemonClient


class TestManufacturingProposalRequestBridges(unittest.TestCase):
    @patch("server_runtime.subprocess.run")
    def test_proposes_manufacturing_plan_create_via_cli(self, run_mock) -> None:
        run_mock.return_value = subprocess.CompletedProcess(
            args=[],
            returncode=0,
            stdout='{"contract":"proposal_create_v1","action":"propose_create_manufacturing_plan"}',
            stderr="",
        )
        response = EngineDaemonClient().create_manufacturing_plan_proposal(
            "/tmp/native-project",
            "fab/doa2526",
            "Reviewed Fabrication Plan",
            "default",
            "main-panel",
            "proposal-plan-create",
            "review manufacturing plan creation",
        )
        run_mock.assert_called_once_with(
            [
                "datum-eda",
                "--format",
                "json",
                "proposal",
                "create-manufacturing-plan",
                "/tmp/native-project",
                "--prefix",
                "fab/doa2526",
                "--name",
                "Reviewed Fabrication Plan",
                "--variant",
                "default",
                "--panel-projection",
                "main-panel",
                "--proposal",
                "proposal-plan-create",
                "--rationale",
                "review manufacturing plan creation",
            ],
            capture_output=True,
            text=True,
            check=False,
        )
        self.assertEqual(response.result["contract"], "proposal_create_v1")

    @patch("server_runtime.subprocess.run")
    def test_proposes_manufacturing_plan_update_via_cli(self, run_mock) -> None:
        run_mock.return_value = subprocess.CompletedProcess(
            args=[],
            returncode=0,
            stdout='{"contract":"proposal_create_v1","action":"propose_update_manufacturing_plan"}',
            stderr="",
        )
        response = EngineDaemonClient().update_manufacturing_plan_proposal(
            "/tmp/native-project",
            "22222222-2222-2222-2222-222222222222",
            "Reviewed Fabrication Plan",
            "fab/doa2526-r2",
            None,
            True,
            None,
            True,
            "proposal-plan-update",
            "review manufacturing plan update",
        )
        run_mock.assert_called_once_with(
            [
                "datum-eda",
                "--format",
                "json",
                "proposal",
                "update-manufacturing-plan",
                "/tmp/native-project",
                "--manufacturing-plan",
                "22222222-2222-2222-2222-222222222222",
                "--name",
                "Reviewed Fabrication Plan",
                "--prefix",
                "fab/doa2526-r2",
                "--clear-variant",
                "--clear-panel-projection",
                "--proposal",
                "proposal-plan-update",
                "--rationale",
                "review manufacturing plan update",
            ],
            capture_output=True,
            text=True,
            check=False,
        )
        self.assertEqual(response.result["contract"], "proposal_create_v1")

    @patch("server_runtime.subprocess.run")
    def test_proposes_manufacturing_plan_delete_via_cli(self, run_mock) -> None:
        run_mock.return_value = subprocess.CompletedProcess(
            args=[],
            returncode=0,
            stdout='{"contract":"proposal_create_v1","action":"propose_delete_manufacturing_plan"}',
            stderr="",
        )
        response = EngineDaemonClient().delete_manufacturing_plan_proposal(
            "/tmp/native-project",
            "22222222-2222-2222-2222-222222222222",
            "proposal-plan-delete",
            "review manufacturing plan deletion",
        )
        run_mock.assert_called_once_with(
            [
                "datum-eda",
                "--format",
                "json",
                "proposal",
                "delete-manufacturing-plan",
                "/tmp/native-project",
                "--manufacturing-plan",
                "22222222-2222-2222-2222-222222222222",
                "--proposal",
                "proposal-plan-delete",
                "--rationale",
                "review manufacturing plan deletion",
            ],
            capture_output=True,
            text=True,
            check=False,
        )
        self.assertEqual(response.result["contract"], "proposal_create_v1")

    @patch("server_runtime.subprocess.run")
    def test_proposes_panel_projection_create_via_cli(self, run_mock) -> None:
        run_mock.return_value = subprocess.CompletedProcess(
            args=[],
            returncode=0,
            stdout='{"contract":"proposal_create_v1","action":"propose_create_panel_projection"}',
            stderr="",
        )
        response = EngineDaemonClient().create_panel_projection_proposal(
            "/tmp/native-project",
            "main-panel",
            "Reviewed Main Panel",
            "main-board",
            1000,
            2000,
            90,
            "proposal-panel-create",
            "review panel creation",
        )
        run_mock.assert_called_once_with(
            [
                "datum-eda",
                "--format",
                "json",
                "proposal",
                "create-panel-projection",
                "/tmp/native-project",
                "--key",
                "main-panel",
                "--name",
                "Reviewed Main Panel",
                "--board",
                "main-board",
                "--x-nm",
                "1000",
                "--y-nm",
                "2000",
                "--rotation-deg",
                "90",
                "--proposal",
                "proposal-panel-create",
                "--rationale",
                "review panel creation",
            ],
            capture_output=True,
            text=True,
            check=False,
        )
        self.assertEqual(response.result["contract"], "proposal_create_v1")

    @patch("server_runtime.subprocess.run")
    def test_proposes_panel_projection_update_via_cli(self, run_mock) -> None:
        run_mock.return_value = subprocess.CompletedProcess(
            args=[],
            returncode=0,
            stdout='{"contract":"proposal_create_v1","action":"propose_update_panel_projection"}',
            stderr="",
        )
        response = EngineDaemonClient().update_panel_projection_proposal(
            "/tmp/native-project",
            "11111111-1111-1111-1111-111111111111",
            "Reviewed Main Panel",
            "main-board",
            3000,
            4000,
            180,
            "proposal-panel-update",
            "review panel update",
        )
        run_mock.assert_called_once_with(
            [
                "datum-eda",
                "--format",
                "json",
                "proposal",
                "update-panel-projection",
                "/tmp/native-project",
                "--panel-projection",
                "11111111-1111-1111-1111-111111111111",
                "--name",
                "Reviewed Main Panel",
                "--board",
                "main-board",
                "--x-nm",
                "3000",
                "--y-nm",
                "4000",
                "--rotation-deg",
                "180",
                "--proposal",
                "proposal-panel-update",
                "--rationale",
                "review panel update",
            ],
            capture_output=True,
            text=True,
            check=False,
        )
        self.assertEqual(response.result["contract"], "proposal_create_v1")

    @patch("server_runtime.subprocess.run")
    def test_proposes_panel_projection_delete_via_cli(self, run_mock) -> None:
        run_mock.return_value = subprocess.CompletedProcess(
            args=[],
            returncode=0,
            stdout='{"contract":"proposal_create_v1","action":"propose_delete_panel_projection"}',
            stderr="",
        )
        response = EngineDaemonClient().delete_panel_projection_proposal(
            "/tmp/native-project",
            "11111111-1111-1111-1111-111111111111",
            "proposal-panel-delete",
            "review panel deletion",
        )
        run_mock.assert_called_once_with(
            [
                "datum-eda",
                "--format",
                "json",
                "proposal",
                "delete-panel-projection",
                "/tmp/native-project",
                "--panel-projection",
                "11111111-1111-1111-1111-111111111111",
                "--proposal",
                "proposal-panel-delete",
                "--rationale",
                "review panel deletion",
            ],
            capture_output=True,
            text=True,
            check=False,
        )
        self.assertEqual(response.result["contract"], "proposal_create_v1")


if __name__ == "__main__":
    unittest.main()
