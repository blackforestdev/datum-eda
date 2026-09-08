"""Real entry-point refusals for coordinated external and dormant work."""

from copy import deepcopy
from datetime import datetime, timedelta, timezone
import json
import unittest

from project_status import render_block
import test_workflow_delivery_coverage_entrypoints as support
from workflow_delivery_selector import selector_failures
from workflow_delivery_tree import Tree


class ExternalBoundaryTest(unittest.TestCase):
    def setUp(self):
        self.h = support.CoverageEntrypointsTest()
        self.h.setUp()
        self.addCleanup(self.h.doCleanups)
        self.f, self.m = self.h.f, self.h.manifest
        self.item = self.m["frontier"][1]
        step = self.item["completion"]["steps"][0]
        step.update(kind="execution", status="in_progress")
        second = deepcopy(step)
        second.update(id="NEXT-C02", status="pending", depends_on=["NEXT-C01"])
        doc = self.item["governing_docs"][0]
        second["requirement_refs"] = [{"path": doc, "marker": "NEXT-C02"}]
        self.item["completion"]["steps"].append(second)
        self.f.write(doc, self.f.root.joinpath(doc).read_bytes() + b"\n<!-- REQ:NEXT:NEXT-C02 -->\n")
        now = datetime.now(timezone.utc).replace(microsecond=0) - timedelta(seconds=1)
        stamp = lambda value: value.isoformat().replace("+00:00", "Z")
        self.item.update(state="in_progress", authorization="execution", claim={
            "agent": "codex", "harness": "codex-cli", "session": "external-fixture",
            "worktree": str(self.f.root), "head": self.h.authority, "scope": ["external/owned"],
            "claimed_at": stamp(now), "heartbeat_at": stamp(now),
            "expires_at": stamp(now + timedelta(hours=1))})
        self.issues = [json.loads(line) for line in
                       self.f.root.joinpath(".beads/issues.jsonl").read_text().splitlines()]
        self.issue = next(i for i in self.issues if i["id"] == "dat-next")
        self.issue.update(status="in_progress", assignee="codex",
                          acceptance_criteria="NEXT-C01: Current\nNEXT-C02: Later")
        self.policy = Tree(self.f.root).json("specs/workflow_delivery_policy.json")
        self.policy["coverage"]["rows"][1].update(category="external_lane",
            boundary_ref=self.f.ref, external_handoff_ref=self.f.ref)
        self.policy["coverage"]["production_roots"].append("external")
        self.policy["coverage"]["source_scopes"] = [{"frontier_key": "NEXT",
            "step_ids": ["NEXT-C01"], "paths": ["external/owned"], "boundary_ref": self.f.ref}]
        self.promote()

    def save(self):
        self.f.save("specs/active_frontier.json", self.m)
        self.f.write("specs/PROGRESS.md", render_block(self.m).encode())
        self.f.write(".beads/issues.jsonl", ("\n".join(json.dumps(i) for i in self.issues) + "\n").encode())
        self.f.save("specs/workflow_delivery_policy.json", self.policy)
        self.f.stage()

    def promote(self):
        self.save()
        self.f.git("commit", "-qm", "fixture exact coordinated boundary\n\nSynthetic trust only.")
        self.h.authority = self.f.git("rev-parse", "HEAD").decode().strip()
        for key in ("AuthorityRef", "BaseRef"):
            self.f.git("config", "datum.workflowDelivery" + key, self.h.authority)

    def refuse(self, text):
        code, report = self.h.cli()
        self.assertEqual(1, code, report)
        self.assertIn(text, report["findings"][0]["detail"])

    def test_exact_external_scope_allows_source_without_claiming_enrollment(self):
        self.f.write("external/owned/input.py", b"bounded external fixture\n")
        self.save()
        code, report = self.h.cli()
        self.assertEqual(0, code, report)
        self.assertEqual([], selector_failures(self.f.root, self.m))
        self.assertNotIn("NEXT", {r["frontier_key"] for r in self.policy["enrolled"]})

    def test_external_scope_does_not_allow_neighbor_paths(self):
        self.f.write("external/owned_elsewhere/input.py", b"outside handoff\n")
        self.save()
        self.refuse("no promoted scope")

    def test_external_permission_does_not_waive_affected_enrolled_proof(self):
        self.policy["coverage"]["source_scopes"][0]["paths"].append("src/read.py")
        self.promote()
        self.f.write("src/read.py", b"changed shared input\n")
        self.save()
        code, report = self.h.cli()
        self.assertEqual(1, code, report)
        self.assertEqual("TASK", report["findings"][0]["key"])
        self.assertIn("changed relevant inputs", report["findings"][0]["detail"])

    def test_expired_external_claim_refuses_actual_cli(self):
        self.item["claim"]["expires_at"] = self.item["claim"]["heartbeat_at"]
        self.save()
        self.refuse("claim")

    def test_missing_external_claim_refuses_actual_cli(self):
        del self.item["claim"]
        self.save()
        self.refuse("requires a claim")

    def test_tracker_assignee_mismatch_refuses_actual_cli(self):
        self.issue["assignee"] = "different-owner"
        self.save()
        self.refuse("does not match")

    def test_selected_requirement_cannot_be_rewritten(self):
        self.item["completion"]["steps"][0]["action"] = "Expanded unauthorized action"
        self.save()
        self.refuse("selected requirement/authority changed")

    def test_unselected_future_step_cannot_be_rewritten(self):
        self.item["completion"]["steps"][1]["action"] = "Expanded future action"
        self.save()
        self.refuse("crossed the promoted boundary")

    def test_external_outcome_cannot_expand(self):
        self.item["completion"]["outcome"] = "Accept all unrelated product work"
        self.save()
        self.refuse("completion boundary changed")

    def test_external_prerequisite_cannot_change_without_coordination(self):
        self.item["dependencies"] = ["dat-test"]
        self.issue["dependencies"] = [{"depends_on_id": "dat-test", "type": "blocks"}]
        self.m["frontier"][0]["completion"]["post_completion"]["unblocks_issue_ids"] = ["dat-next"]
        self.m["frontier"][0]["unblocks"] = ["dat-next"]
        self.save()
        self.refuse("governing boundary changed")

    def test_external_boundary_cannot_advance_to_next_step(self):
        first, second = self.item["completion"]["steps"]
        first.update(status="complete", completion_evidence=[{"kind": "document", **self.f.ref}])
        second["status"] = "in_progress"
        self.item["completion"]["canonical_next_step_id"] = "NEXT-C02"
        self.save()
        self.refuse("boundary changed")


class DeferredBoundaryTest(unittest.TestCase):
    def setUp(self):
        self.e = ExternalBoundaryTest()
        self.e.setUp()
        self.addCleanup(self.e.doCleanups)
        self.cold = deepcopy(self.e.item)
        self.cold.update(key="COLD", issue_id="dat-cold", order=2, state="deferred",
                         authorization="none", canonical_next=False)
        del self.cold["claim"]
        step = self.cold["completion"]["steps"][0]
        step.update(id="COLD-C01", status="pending")
        step["requirement_refs"][0]["marker"] = "COLD-C01"
        self.cold["completion"].update(steps=[step], canonical_next_step_id="COLD-C01")
        self.e.m["frontier"].append(self.cold)
        doc = self.cold["governing_docs"][0]
        self.e.f.write(doc, self.e.f.root.joinpath(doc).read_bytes() + b"\n<!-- REQ:COLD:COLD-C01 -->\n")
        self.issue = {"id": "dat-cold", "status": "deferred", "labels": ["roadmap:deferred"],
                      "acceptance_criteria": "COLD-C01: Deferred work"}
        self.e.issues.append(self.issue)
        self.e.policy["coverage"]["rows"].append({"frontier_key": "COLD", "issue_id": "dat-cold",
            "category": "deferred", "boundary_ref": self.e.f.ref, "external_handoff_ref": None})
        self.e.promote()

    def test_dormant_item_passes_without_future_contract(self):
        code, report = self.e.h.cli()
        self.assertEqual(0, code, report)

    def promote_completed_history(self):
        step = self.cold["completion"]["steps"][0]
        step.update(status="complete", completion_evidence=[{"kind": "document", **self.e.f.ref}])
        self.cold["completion"]["canonical_next_step_id"] = None
        self.e.promote()

    def test_deferred_hold_preserves_prior_completed_execution(self):
        self.promote_completed_history()
        code, report = self.e.h.cli()
        self.assertEqual(0, code, report)
        self.assertEqual([], selector_failures(self.e.f.root, self.e.m))

    def test_deferred_hold_cannot_complete_pending_work(self):
        step = self.cold["completion"]["steps"][0]
        step.update(status="complete", completion_evidence=[{"kind": "document", **self.e.f.ref}])
        self.cold["completion"]["canonical_next_step_id"] = None
        self.e.save()
        self.e.refuse("deferred work requires reclassification")

    def test_deferred_completed_requirement_cannot_change(self):
        self.promote_completed_history()
        self.cold["completion"]["steps"][0]["action"] = "Expanded historical scope"
        self.e.save()
        self.e.refuse("deferred work requires reclassification")

    def test_deferred_pending_requirement_cannot_change(self):
        self.cold["completion"]["steps"][0]["action"] = "Expanded pending scope"
        self.e.save()
        self.e.refuse("deferred work requires reclassification")

    def test_deferred_classification_preserves_planned_none_placeholder(self):
        self.cold["state"] = "planned"
        self.issue.update(status="open", labels=["roadmap:frontier"])
        self.e.promote()
        code, report = self.e.h.cli()
        self.assertEqual(0, code, report)
        self.assertEqual([], selector_failures(self.e.f.root, self.e.m))

    def test_dormant_item_cannot_reactivate_under_old_classification(self):
        self.cold.update(state="ready", authorization="execution")
        self.issue["status"] = "open"
        self.issue["labels"] = ["roadmap:frontier"]
        self.e.save()
        self.e.refuse("deferred work requires reclassification")

    def test_ready_none_cannot_escape_deferred_classification(self):
        self.cold["state"] = "ready"
        self.issue["status"] = "open"
        self.issue["labels"] = ["roadmap:frontier"]
        self.e.save()
        self.e.refuse("deferred work requires reclassification")


if __name__ == "__main__":
    unittest.main()
