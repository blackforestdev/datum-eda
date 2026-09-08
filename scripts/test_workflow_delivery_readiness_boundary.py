"""Completed unenrolled readiness must validate authority via the real CLI."""

from copy import deepcopy
import json
import unittest

from project_status import render_block
import test_workflow_delivery_coverage_entrypoints as support
from workflow_delivery_selector import selector_failures
from workflow_delivery_tree import Tree


class ReadinessBoundaryTest(unittest.TestCase):
    def setUp(self):
        self.h = support.CoverageEntrypointsTest()
        self.h.setUp()
        self.addCleanup(self.h.doCleanups)
        self.f = self.h.f
        self.m = self.h.manifest
        self.item = self.m["frontier"][1]
        doc = self.item["governing_docs"][0]
        original = self.item["completion"]["steps"][0]
        ready = deepcopy(original)
        owner = deepcopy(original)
        owner.update(id="NEXT-C02", kind="owner_decision", depends_on=["NEXT-C01"])
        owner["requirement_refs"] = [{"path": doc, "marker": "NEXT-C02"}]
        owner["owner_input"] = {"response_format": "Fixture authorization only",
            "requests": [{"id": "AUTHORIZE", "question": "Authorize fixture?",
                          "recommended_response": "Fixture only",
                          "source_ref": {"path": doc, "marker": "AUTHORIZE"}}]}
        verify = deepcopy(original)
        verify.update(id="NEXT-C03", kind="execution", depends_on=["NEXT-C02"])
        verify["requirement_refs"] = [{"path": doc, "marker": "NEXT-C03"}]
        self.item["completion"].update(steps=[ready, owner, verify], delivery={
            "contract_path": "next.contract.json", "checkpoints": {
                "ready": "NEXT-C01", "activate": None, "verify": "NEXT-C03", "accept": None}})
        self.f.write(doc, self.f.root.joinpath(doc).read_bytes() + (
            "\n<!-- NEXT-READY-EVIDENCE -->\n<!-- REQ:NEXT:NEXT-C02 -->\n"
            "<!-- OWNER:NEXT:NEXT-C02:AUTHORIZE -->\n<!-- REQ:NEXT:NEXT-C03 -->\n").encode())
        issues = [json.loads(line) for line in self.f.root.joinpath(".beads/issues.jsonl").read_text().splitlines()]
        next(i for i in issues if i["id"] == "dat-next")["acceptance_criteria"] = (
            "NEXT-C01: Prepare\nNEXT-C02: Authorize\nNEXT-C03: Verify")
        self.f.write(".beads/issues.jsonl", ("\n".join(json.dumps(i) for i in issues) + "\n").encode())
        self.contract = deepcopy(self.f.contract)
        self.contract.update(id="next-contract", frontier_key="NEXT", issue_id="dat-next",
                             category="infrastructure", proof_path="missing-future-proof.json",
                             review_path="missing-future-review.json")
        for scenario in self.contract["scenarios"]:
            scenario["method"] = "infrastructure"
        self.f.save("next.contract.json", self.contract)
        policy = Tree(self.f.root).json("specs/workflow_delivery_policy.json")
        next(r for r in policy["coverage"]["rows"] if r["frontier_key"] == "NEXT")["category"] = "infrastructure"
        self.f.save("specs/workflow_delivery_policy.json", policy)
        self.save()
        self.f.git("commit", "-qm", "test(readiness): prepare unexecuted fixture contract\n\nSynthetic authority only; no production promotion.")
        self.h.authority = self.f.git("rev-parse", "HEAD").decode().strip()
        for key in ("AuthorityRef", "BaseRef"):
            self.f.git("config", "datum.workflowDelivery" + key, self.h.authority)
        # Candidate completes readiness, not execution or acceptance.
        ready.update(status="complete", completion_evidence=[
            {"kind": "document", "path": doc, "marker": "NEXT-READY-EVIDENCE"}])
        self.item.update(authorization="owner_decision", state="specified")
        self.item["completion"]["canonical_next_step_id"] = "NEXT-C02"
        self.save()

    def save(self):
        self.f.save("specs/active_frontier.json", self.m)
        self.f.write("specs/PROGRESS.md", render_block(self.m).encode())
        self.f.stage()

    def test_ready_requires_authority_but_not_future_proof(self):
        code, report = self.h.cli()
        self.assertEqual(0, code, report)
        self.assertEqual([], selector_failures(self.f.root, self.m))
        self.assertFalse(self.f.root.joinpath("missing-future-proof.json").exists())

    def test_unresolved_required_question_refuses_completed_readiness(self):
        self.contract["open_decisions"] = [{"id": "MISSING", "question": "Unresolved authority",
            "required_for_scenarios": [self.contract["scenarios"][0]["id"]], "disposition_ref": None}]
        self.f.save("next.contract.json", self.contract)
        self.f.stage()
        code, report = self.h.cli()
        self.assertEqual(1, code, report)
        self.assertEqual("WDQ-AUTHORITY", report["findings"][0]["code"])
        self.assertEqual("NEXT", report["findings"][0]["key"])
        self.assertTrue(any("mandatory owner question unresolved" in e
                            for e in selector_failures(self.f.root, self.m)))

    def test_missing_authority_marker_refuses_completed_readiness(self):
        self.contract["authority_refs"][0]["marker"] = "missing-reference"
        self.f.save("next.contract.json", self.contract)
        self.f.stage()
        code, report = self.h.cli()
        self.assertEqual(1, code, report)
        self.assertEqual("WDQ-AUTHORITY", report["findings"][0]["code"])

    def test_missing_units_answer_refuses_completed_readiness(self):
        del self.contract["foundation_answers"]["units_precision"]
        self.f.save("next.contract.json", self.contract)
        self.f.stage()
        code, report = self.h.cli()
        self.assertEqual(1, code, report)
        self.assertEqual("WDQ-COVERAGE", report["findings"][0]["code"])
        self.assertEqual("NEXT", report["findings"][0]["key"])

    def test_stale_owning_route_refuses_completed_readiness(self):
        self.f.write("research/source.md", b"Unreconciled source change.\n")
        self.f.stage()
        code, report = self.h.cli()
        self.assertEqual(1, code, report)
        self.assertEqual("WDQ-AUTHORITY", report["findings"][0]["code"])
        self.assertIn("owning route review is stale", report["findings"][0]["detail"])

    def test_completed_preflight_cannot_omit_its_delivery_declaration(self):
        del self.item["completion"]["delivery"]
        self.save()
        code, report = self.h.cli()
        self.assertEqual(1, code, report)
        self.assertIn("requires a delivery declaration", report["findings"][0]["detail"])

    def test_execution_requires_promoted_enrollment(self):
        owner = self.item["completion"]["steps"][1]
        owner.update(status="complete", completion_evidence=[{"kind": "review", **self.f.ref}])
        self.item.update(state="ready", authorization="execution")
        self.item["completion"]["canonical_next_step_id"] = "NEXT-C03"
        self.save()
        code, report = self.h.cli()
        self.assertEqual(1, code, report)
        self.assertIn("requires owner-promoted enrollment", report["findings"][0]["detail"])

    def test_product_classification_cannot_use_infrastructure_contract(self):
        policy = Tree(self.f.root).json("specs/workflow_delivery_policy.json")
        next(r for r in policy["coverage"]["rows"] if r["frontier_key"] == "NEXT")[
            "category"] = "product"
        self.f.save("specs/workflow_delivery_policy.json", policy)
        self.save()
        self.f.git("commit", "-qm", "fixture product classification with incorrect contract")
        self.h.authority = self.f.git("rev-parse", "HEAD").decode().strip()
        for key in ("AuthorityRef", "BaseRef"):
            self.f.git("config", "datum.workflowDelivery" + key, self.h.authority)
        code, report = self.h.cli()
        self.assertEqual(1, code, report)
        self.assertIn("category must match owner-promoted classification",
                      report["findings"][0]["detail"])

    def test_pending_planning_does_not_require_future_contract(self):
        self.item["completion"]["steps"][0].update(status="pending", completion_evidence=[])
        self.item.update(state="specified", authorization="planning")
        self.item["completion"]["canonical_next_step_id"] = "NEXT-C01"
        del self.item["completion"]["delivery"]
        self.save()
        code, report = self.h.cli()
        self.assertEqual(0, code, report)
        self.assertEqual([], selector_failures(self.f.root, self.m))


if __name__ == "__main__":
    unittest.main()
