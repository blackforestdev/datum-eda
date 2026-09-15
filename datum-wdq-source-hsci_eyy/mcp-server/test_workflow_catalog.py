#!/usr/bin/env python3
"""Governed canonical workflow inventory tests."""

from __future__ import annotations

import copy
import json
from pathlib import Path
import unittest

from discovery_scope import DiscoveryScope
from resources_catalog import DatumResourceCatalog
from workflow_catalog import load_workflow_catalog, validate_workflow_catalog


class TestWorkflowCatalog(unittest.TestCase):
    def test_catalog_binds_every_governed_workflow_surface(self) -> None:
        catalog = load_workflow_catalog()
        by_id = {workflow["id"]: workflow for workflow in catalog["workflows"]}

        self.assertEqual(
            by_id["apply-approved-proposal"]["required_capability"],
            "apply-approved",
        )
        self.assertEqual(
            by_id["apply-approved-proposal"]["review_gate"],
            "accepted-proposal-and-matching-revision",
        )
        self.assertIn(
            "datum.prepare-proposal", by_id["prepare-proposal"]["mcp_prompts"]
        )
        self.assertIn(
            "datum://context/pinned/{context_id}",
            by_id["refresh-stale-context"]["mcp_resources"],
        )
        self.assertIn(
            "datum-eda agent launch <adapter> <project-root> --resume",
            by_id["resume-agent-work"]["cli"],
        )

    def test_unknown_or_privately_invented_projection_fails_validation(self) -> None:
        catalog = load_workflow_catalog()
        mutated = copy.deepcopy(catalog)
        mutated["workflows"][0]["mcp_tools"].append("vendor.private.magic")

        with self.assertRaisesRegex(ValueError, "unknown MCP tools"):
            validate_workflow_catalog(mutated)

    def test_catalog_is_discoverable_as_a_stable_read_only_resource(self) -> None:
        resources = DatumResourceCatalog(
            DiscoveryScope(
                path=Path("/tmp/discovery.json"),
                schema="datum_terminal_context_v1",
                project_root=Path("/tmp/project"),
                terminal_session_id="session-test",
                agent_launch_id=None,
                context_id="context-test",
                document={"model_revision": "revision-test"},
            )
        )

        self.assertIn(
            "datum://workflows",
            {item["uri"] for item in resources.list_resources()},
        )
        content = resources.read("datum://workflows")["contents"][0]
        self.assertEqual(content["mimeType"], "application/json")
        self.assertEqual(json.loads(content["text"])["catalog_version"], 1)

    def test_missing_review_or_evidence_contract_fails_validation(self) -> None:
        catalog = load_workflow_catalog()
        missing_gate = copy.deepcopy(catalog)
        missing_gate["workflows"][2]["review_gate"] = ""
        with self.assertRaisesRegex(ValueError, "review_gate"):
            validate_workflow_catalog(missing_gate)

        missing_evidence = copy.deepcopy(catalog)
        missing_evidence["workflows"][2]["evidence"] = []
        with self.assertRaisesRegex(ValueError, "evidence may not be empty"):
            validate_workflow_catalog(missing_evidence)


if __name__ == "__main__":
    unittest.main()
