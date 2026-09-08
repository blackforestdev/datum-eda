#!/usr/bin/env python3
"""Check the R02 planning inventory, not runtime enforcement or adoption.

Use the recorded baseline so another product lane's later progress cannot turn
a truthful historical proposal into a failing live-status assertion.
"""

import json
from pathlib import Path
import re
import subprocess
import unittest


ROOT = Path(__file__).resolve().parents[1]
PROPOSAL = "docs/reviews/workflow-delivery-rollout/proposal.json"


class WorkflowRolloutProposalTest(unittest.TestCase):
    @classmethod
    def setUpClass(cls):
        cls.proposal = json.loads((ROOT / PROPOSAL).read_text())
        revision = cls.proposal["baseline_ref"]
        if not re.fullmatch(r"[0-9a-f]{40}|[0-9a-f]{64}", revision):
            raise ValueError("proposal must pin a full baseline commit")
        raw = subprocess.check_output(
            ["git", "show", f"{revision}:specs/active_frontier.json"],
            cwd=ROOT, text=True,
        )
        cls.items = {i["key"]: i for i in json.loads(raw)["frontier"]}

    def test_every_baseline_identity_is_classified_once(self):
        rows = self.proposal["classification_proposal"]
        self.assertEqual(len(self.items), len(rows))
        self.assertEqual(set(self.items), {r["frontier_key"] for r in rows})
        allowed = {"historical", "specification", "product", "infrastructure",
                   "deferred", "external_lane"}
        for row in rows:
            self.assertEqual({"frontier_key", "issue_id", "category"}, set(row))
            self.assertIn(row["category"], allowed)
            item = self.items[row["frontier_key"]]
            self.assertEqual(item["issue_id"], row["issue_id"])
            self.assertEqual(item["state"] == "landed", row["category"] == "historical")

    def test_three_distinct_real_consumers_use_existing_ordered_steps(self):
        cohorts = self.proposal["enrollment_intents"]
        self.assertEqual({"UVT-S5A-BUILD", "GUI-WRITE-PATH", "NATIVE-AUTHORING"},
                         {c["frontier_key"] for c in cohorts})
        self.assertEqual(3, len(cohorts))
        for cohort in cohorts:
            item = self.items[cohort["frontier_key"]]
            self.assertEqual(item["issue_id"], cohort["issue_id"])
            steps = item["completion"]["steps"]
            ids = [s["id"] for s in steps]
            points = cohort["checkpoints"]
            self.assertEqual({"ready", "activate", "verify", "review", "accept"},
                             set(points))
            self.assertEqual(5, len(set(points.values())))
            ordered = [points[k] for k in ("ready", "activate", "verify", "review", "accept")]
            positions = [ids.index(s) for s in ordered]
            self.assertEqual(sorted(positions), positions)
            self.assertEqual(["planning", "execution", "execution", "execution",
                              "owner_decision"], [steps[p]["kind"] for p in positions])
            authorization = ids.index(cohort["authorization_step"])
            self.assertEqual("owner_decision", steps[authorization]["kind"])
            self.assertLess(positions[0], authorization)
            self.assertLess(authorization, positions[1])
            self.assertEqual(2, cohort["mapping_version"])
            self.assertTrue(cohort["contract_path"].startswith("specs/workflow_delivery/"))
            self.assertEqual(len(cohort["scenario_groups"]),
                             len(set(cohort["scenario_groups"])))

    def test_proposal_does_not_manufacture_execution_or_adoption(self):
        p = self.proposal
        self.assertEqual("rollout_proposal", p["record_type"])
        self.assertIsNone(p["candidate_ref"])
        for key in ("implementation_authorized", "promotion_asserted", "acceptance_asserted"):
            self.assertIs(p[key], False)
        self.assertEqual([], p["source_scopes_proposed"])
        for cohort in p["enrollment_intents"]:
            self.assertIsNone(cohort["product_owner_session"])
            self.assertIsNone(cohort["independent_reviewer_session"])
            self.assertIs(cohort["readiness_asserted"], False)
            self.assertEqual([], cohort["adoption_events"])

    def test_preferences_is_a_coordination_boundary_not_adoption(self):
        external = self.proposal["external_lane"]
        self.assertEqual("GLOBAL-PREFERENCES-COMPLETION", external["frontier_key"])
        self.assertEqual("GP-CM05V", external["owner_boundary"])
        self.assertIs(external["handoff_received"], False)
        self.assertIs(external["source_scope_authorized"], False)
        self.assertNotIn(external["frontier_key"],
                         {c["frontier_key"] for c in self.proposal["enrollment_intents"]})

    def test_storage_proposal_is_not_another_documents_bundle(self):
        self.assertEqual("<resolved-git-common-dir>/datum-wdq",
                         self.proposal["support_store_proposal"])
        self.assertEqual("specs/WORKFLOW_DELIVERY_ROLLOUT_IMPLEMENTATION.md",
                         self.proposal["authority_plan"])


if __name__ == "__main__":
    unittest.main()
