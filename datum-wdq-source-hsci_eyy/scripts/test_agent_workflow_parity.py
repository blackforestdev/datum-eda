#!/usr/bin/env python3
"""Hermetic mutations for the agent workflow projection gate."""

from __future__ import annotations

import json
from pathlib import Path
import shutil
import tempfile
import unittest

from check_agent_workflow_parity import check


ROOT = Path(__file__).resolve().parents[1]
FILES = (
    "AGENTS.md",
    "CLAUDE.md",
    ".cursor/rules/datum-workflows.mdc",
    "crates/cli/src/agent_adapters.rs",
    "mcp-server/workflow_catalog.json",
    "mcp-server/workflow_projections.json",
    "crates/gui-app/src/terminal_agent_launch_tests.rs",
    "scripts/agent_mcp_adapter_probe.py",
    "scripts/run_agent_launch_pty_proof.sh",
)


class TestAgentWorkflowParity(unittest.TestCase):
    def fixture(self) -> Path:
        root = Path(tempfile.mkdtemp(prefix="datum-workflow-parity-"))
        self.addCleanup(shutil.rmtree, root)
        for relative in FILES:
            target = root / relative
            target.parent.mkdir(parents=True, exist_ok=True)
            shutil.copyfile(ROOT / relative, target)
        return root

    def test_current_projection_passes(self) -> None:
        self.assertEqual(check(self.fixture()), [])

    def test_missing_pointer_marker_fails(self) -> None:
        root = self.fixture()
        path = root / "AGENTS.md"
        path.write_text(path.read_text().replace("DATUM-WORKFLOW-CATALOG", "REMOVED"))
        self.assertTrue(any("canonical marker" in error for error in check(root)))

    def test_vendor_projection_cannot_redefine_capability(self) -> None:
        root = self.fixture()
        path = root / "mcp-server/workflow_projections.json"
        document = json.loads(path.read_text())
        document["adapters"][0]["required_capability"] = "unattended"
        path.write_text(json.dumps(document))
        self.assertTrue(any("redefines workflow semantics" in error for error in check(root)))

    def test_adapter_instruction_drift_fails(self) -> None:
        root = self.fixture()
        path = root / "mcp-server/workflow_projections.json"
        document = json.loads(path.read_text())
        document["adapters"][1]["adapter_instruction_files"] = ["AGENTS.md"]
        path.write_text(json.dumps(document))
        self.assertTrue(any("AgentAdapter instruction files" in error for error in check(root)))

    def test_production_workflow_phase_removal_fails(self) -> None:
        root = self.fixture()
        path = root / "scripts/agent_mcp_adapter_probe.py"
        path.write_text(path.read_text().replace('"datum.proposal.apply"', '"removed.apply"'))
        self.assertTrue(any("workflow proof lacks" in error for error in check(root)))

    def test_production_pty_proof_removal_fails(self) -> None:
        root = self.fixture()
        path = root / "crates/gui-app/src/terminal_agent_launch_tests.rs"
        path.write_text(
            path.read_text().replace(
                "governed_agents_complete_production_workflow_through_owned_pty",
                "removed_workflow_proof",
            )
        )
        self.assertTrue(any("workflow proof lacks" in error for error in check(root)))


if __name__ == "__main__":
    unittest.main()
