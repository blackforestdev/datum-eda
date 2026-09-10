"""Five-phase review enforcement through real staged CLI and selector."""

from contextlib import redirect_stdout
from copy import deepcopy
import io
import json
import unittest

from check_workflow_delivery import main
from project_status import render_block
from workflow_delivery_checkpoints import delivery_shape
from workflow_delivery_io import DeliveryInputError, canonical_json
from workflow_delivery_native_test_support import environment, typed_proof
from workflow_delivery_authority import authority_sha256
from workflow_delivery_proof import packet_sha256
from workflow_delivery_selector import selector_failures
from workflow_delivery_test_support import Fixture
from workflow_delivery_tree import Tree
from workflow_delivery_trust_test_support import accepted_fixture


class ReviewCheckpointTest(unittest.TestCase):
    def setUp(self):
        self.f = Fixture()
        self.addCleanup(self.f.close)
        _, self.proof, self.review, _ = accepted_fixture(self.f)
        self.m = Tree(self.f.root).json("specs/active_frontier.json")
        self.item = self.m["frontier"][0]
        self.item.update(state="ready", authorization="execution")
        del self.item["landing_commit"]
        completion = self.item["completion"]
        completion["delivery"]["schema_version"] = 2
        completion["delivery"]["checkpoints"]["review"] = "D"
        completion["canonical_next_step_id"] = "D"
        step = deepcopy(completion["steps"][2])
        step.update(id="D", status="pending", depends_on=["V"], completion_evidence=[])
        step["requirement_refs"][0]["marker"] = "D"
        completion["steps"].insert(3, step)
        self.review_step = step
        self.accept_step = completion["steps"][-1]
        self.accept_step.update(status="pending", depends_on=["D"], completion_evidence=[])
        self.doc = self.item["governing_docs"][0]
        self.f.write(self.doc, self.f.root.joinpath(self.doc).read_bytes() +
                     b"\n<!-- REQ:TASK:D -->\n<!-- REVIEW-TASK-D -->\n")
        self.issues = [json.loads(line) for line in
                       self.f.root.joinpath(".beads/issues.jsonl").read_text().splitlines()]
        self.issue = next(i for i in self.issues if i["id"] == "dat-test")
        self.issue.update(status="open", acceptance_criteria="R: Ready\nI: Activate\nV: Verify\nD: Review\nA: Accept")
        self.review["owner_receipt"] = None
        self.f.save("requested-environment.json", environment())
        self.save()
        self.f.git("commit", "-qm", "fixture promote five-phase mapping before review")
        self.authority = self.f.git("rev-parse", "HEAD").decode().strip()
        for key in ("AuthorityRef", "BaseRef"):
            self.f.git("config", "datum.workflowDelivery" + key, self.authority)
        self.f.git("config", "datum.workflowDeliveryEnvironmentPath", "requested-environment.json")
        self.review_step.update(status="complete", completion_evidence=[
            {"kind": "review", "path": self.doc, "marker": "REVIEW-TASK-D"}])
        self.item.update(state="specified", authorization="owner_decision")
        completion["canonical_next_step_id"] = "A"
        self.save()

    def save(self):
        self.f.save("specs/active_frontier.json", self.m)
        self.f.write("specs/PROGRESS.md", render_block(self.m).encode())
        self.f.write(".beads/issues.jsonl", b"".join(canonical_json(i) for i in self.issues))
        self.f.save(self.f.contract["review_path"], self.review)
        self.f.stage()

    def cli(self):
        before = self.f.snapshot()
        output = io.StringIO()
        with redirect_stdout(output):
            code = main(["--root", str(self.f.root), "--enforce", "--staged",
                         "--authority-ref", self.authority, "--base-ref", self.authority,
                         "--environment-path", "requested-environment.json"])
        self.assertEqual(before, self.f.snapshot())
        return code, json.loads(output.getvalue())

    def test_completed_review_passes_without_asserting_owner_acceptance(self):
        code, report = self.cli()
        self.assertEqual(0, code, report)
        self.assertIn("TASK: review", report["checks"])
        self.assertFalse(report["acceptance_asserted"])
        self.assertEqual([], selector_failures(self.f.root, self.m))

    def test_self_review_refuses_at_review_checkpoint(self):
        self.review["reviewer_session"] = "writer"
        self.save()
        code, report = self.cli()
        self.assertEqual(1, code, report)
        self.assertEqual("WDQ-REVIEW", report["findings"][0]["code"])
        self.assertTrue(any("not independent" in e for e in selector_failures(self.f.root, self.m)))

    def test_pending_owner_defect_disposition_does_not_hide_review_findings(self):
        self.review["findings"] = [{"issue_id": "dat-next", "severity": "blocking",
                                    "disposition_ref": None}]
        self.save()
        code, report = self.cli()
        self.assertEqual(0, code, report)
        self.assertFalse(report["acceptance_asserted"])

    def test_cannot_combine_review_and_acceptance_from_unreviewed_base(self):
        self.accept_step.update(status="complete", completion_evidence=[
            {"kind": "review", "path": self.doc, "marker": "EVIDENCE-TASK-A"}])
        self.item.update(state="landed", authorization="none", landing_commit=self.authority)
        self.item["completion"]["canonical_next_step_id"] = None
        self.issue["status"] = "closed"
        self.save()
        code, report = self.cli()
        self.assertEqual(1, code, report)
        self.assertIn("pending/unreviewed baseline", report["findings"][0]["detail"])

    def test_mapping_version_is_explicit_and_closed(self):
        for version in (True, 1, 3, "2"):
            with self.subTest(version=version):
                item = deepcopy(self.item)
                item["completion"]["delivery"]["schema_version"] = version
                with self.assertRaises(DeliveryInputError):
                    delivery_shape(item, self.f.contract)

    def test_review_is_distinct_required_execution_checkpoint(self):
        for change in (None, "V", "A"):
            with self.subTest(review=change):
                item = deepcopy(self.item)
                item["completion"]["delivery"]["checkpoints"]["review"] = change
                with self.assertRaises(DeliveryInputError):
                    delivery_shape(item, self.f.contract)

    def test_infrastructure_also_requires_review_without_product_acceptance(self):
        item = deepcopy(self.item)
        points = item["completion"]["delivery"]["checkpoints"]
        points.update(activate=None, accept=None)
        contract = deepcopy(self.f.contract)
        contract["category"] = "infrastructure"
        delivery_shape(item, contract)
        points["review"] = None
        with self.assertRaises(DeliveryInputError):
            delivery_shape(item, contract)

    def test_infrastructure_review_runs_actual_replay_without_acceptance_receipt(self):
        self.infrastructure_replay()

    def test_headless_infrastructure_review_checks_producer_and_reviewer_environment(self):
        from test_workflow_delivery_headless import environment as headless_environment
        self.infrastructure_replay(headless_environment())

    def infrastructure_replay(self, requested=None):
        self.f.contract["category"] = "infrastructure"
        self.f.contract["scenarios"][0]["method"] = "infrastructure"
        self.f.save("contract.json", self.f.contract)
        self.item["completion"]["delivery"]["checkpoints"].update(activate=None, accept=None)
        self.proof, _, _, _ = typed_proof(self.f)
        replay, _, _, _ = typed_proof(self.f, "reviewer")
        if requested is not None:
            self.f.save("requested-environment.json", requested)
            self.proof["environment"] = self.f.blob(
                "evidence/headless-producer.json", canonical_json(requested))
            replay["environment"] = self.f.blob(
                "evidence/headless-reviewer.json", canonical_json(requested))
        self.review["replay"] = self.f.blob("docs/reviews/replay.json", canonical_json(replay))
        self.f.save(self.f.contract["proof_path"], self.proof)
        self.review["packet_sha256"] = packet_sha256(self.f.contract, self.proof,
            authority_sha256(Tree(self.f.root), self.f.contract))
        self.save()
        self.f.git("commit", "-qm", "fixture exact infrastructure review authority")
        self.authority = self.f.git("rev-parse", "HEAD").decode().strip()
        for key in ("AuthorityRef", "BaseRef"):
            self.f.git("config", "datum.workflowDelivery" + key, self.authority)
        code, report = self.cli()
        self.assertEqual(0, code, report)
        self.assertIn("TASK: review", report["checks"])
        self.assertFalse(report["acceptance_asserted"])
        self.assertEqual([], selector_failures(self.f.root, self.m))


if __name__ == "__main__":
    unittest.main()
