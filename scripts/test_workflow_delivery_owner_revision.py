"""Owner-directed corrections through real CLI/selector views; no product proof."""

from copy import deepcopy
from datetime import datetime, timedelta, timezone
import json
import unittest

from project_status import render_block
import test_workflow_delivery_coverage_entrypoints as support
from workflow_delivery_selector import selector_failures
from workflow_delivery_tree import Tree
from workflow_delivery_owner_revision import validate_owner_revision
from workflow_delivery_io import DeliveryInputError


class OwnerRevisionTest(unittest.TestCase):
    def setUp(self):
        self.h = support.CoverageEntrypointsTest()
        self.h.setUp()
        self.addCleanup(self.h.doCleanups)
        self.f, self.m = self.h.f, self.h.manifest
        self.item = self.m["frontier"][1]
        self.doc = self.item["governing_docs"][0]
        self.item.update(authorization="owner_decision", state="specified", claim=None)
        first = self.item["completion"]["steps"][0]
        first.update(kind="owner_decision", status="pending",
                     owner_input=self.owner_input("NEXT-C01", self.doc))
        self.f.write(self.doc, self.f.root.joinpath(self.doc).read_bytes() +
                     b"\n<!-- OWNER:NEXT:NEXT-C01:NEXT-C01 -->\n")
        self.issues = [json.loads(line) for line in
                       self.f.root.joinpath(".beads/issues.jsonl").read_text().splitlines()]
        self.issue = next(i for i in self.issues if i["id"] == "dat-next")
        self.policy = Tree(self.f.root).json("specs/workflow_delivery_policy.json")
        self.policy["coverage"]["rows"][1].update(category="external_lane",
                                                  external_handoff_ref=self.f.ref)
        self.promote()
        self.source = deepcopy(self.item)
        self.source_commit = self.h.authority
        self.record_ref = {"path": "docs/revision.json", "marker": "REVISION-NEXT"}
        self.decision_ref = {"path": "docs/decisions/PRODUCT_MECHANICS_043_FIXTURE.md",
                             "marker": "<!-- REVISION-AUTHORITY -->"}
        self.f.write(self.decision_ref["path"], b"<!-- REVISION-AUTHORITY -->\nSynthetic doctrine.\n")
        self.instruction_ref = {"path": "docs/owner-revision.md", "marker": "<!-- OWNER-REVISION -->"}
        self.instructions = "Revise the specification; preserve review and do not accept the product."
        self.f.write(self.instruction_ref["path"],
                     (self.instruction_ref["marker"] + "\n" + self.instructions + "\n").encode())
        self.correction_doc = "docs/correction.md"
        self.f.write(self.correction_doc,
                     b"<!-- REQ:NEXT:NEXT-R -->\nCorrect bounded specification.\n"
                     b"<!-- REQ:NEXT:NEXT-RV -->\nReview corrected specification.\n"
                     b"<!-- OWNER:NEXT:NEXT-RV:NEXT-RV -->\n"
                     b"<!-- CORRECTION-PROOF -->\nSynthetic authorship evidence.\n")
        target = deepcopy(self.source)
        target.update(authorization="planning", state="specified", claim=None)
        target["governing_docs"] += [self.record_ref["path"], self.instruction_ref["path"],
                                      self.correction_doc, self.decision_ref["path"]]
        first = target["completion"]["steps"][0]
        first.update(status="complete", completion_evidence=[{"kind": "review", **self.instruction_ref}])
        correction = deepcopy(first)
        correction.pop("owner_input")
        correction.update(id="NEXT-R", kind="planning", status="pending", action="Correct specification",
                          depends_on=["NEXT-C01"], completion_evidence=[], requirement_refs=[
                              {"path": self.correction_doc, "marker": "NEXT-R"}])
        review = deepcopy(first)
        review.update(id="NEXT-RV", status="pending", action="Review correction", depends_on=["NEXT-R"],
                      completion_evidence=[], requirement_refs=[
                          {"path": self.correction_doc, "marker": "NEXT-RV"}],
                      owner_input=self.owner_input("NEXT-RV", self.correction_doc))
        target["completion"]["steps"] += [correction, review]
        target["completion"]["canonical_next_step_id"] = "NEXT-R"
        self.record = {"schema_version": 1, "kind": "owner_directed_revision", "marker": "REVISION-NEXT",
                       "frontier_key": "NEXT", "issue_id": "dat-next", "disposition": "revise",
                       "decision_ref": self.decision_ref,
                       "owner_instruction": self.instructions, "owner_instruction_ref": self.instruction_ref,
                       "provenance": {"source": "Synthetic owner fixture", "recorded_at": "2026-09-16T00:00:00Z"},
                       "source_commit": self.source_commit, "source_step_id": "NEXT-C01",
                       "correction_step_id": "NEXT-R", "review_step_id": "NEXT-RV", "target": target}
        self.f.save(self.record_ref["path"], self.record)
        governance = Tree(self.f.root).json("specs/spec_governance_manifest.json")
        for path in (self.record_ref["path"], self.instruction_ref["path"], self.correction_doc):
            governance["entries"][path] = {"class": "governed"}
        governance["entries"][self.decision_ref["path"]] = {"class": "doctrine"}
        self.f.save("specs/spec_governance_manifest.json", governance)
        self.policy["coverage"]["owner_revisions"] = [{"frontier_key": "NEXT", "record_ref": self.record_ref}]
        self.promote()  # Synthetic external selection, never a real owner promotion.

    @staticmethod
    def owner_input(step, doc):
        return {"response_format": "approve or revise", "requests": [{"id": step,
                "question": "Is this ready?", "recommended_response": "Review before deciding",
                "source_ref": {"path": doc, "marker": step}}]}

    def save(self):
        self.f.save("specs/active_frontier.json", self.m)
        self.f.write("specs/PROGRESS.md", render_block(self.m).encode())
        self.f.write(".beads/issues.jsonl", ("\n".join(json.dumps(i) for i in self.issues) + "\n").encode())
        self.f.save("specs/workflow_delivery_policy.json", self.policy)
        self.f.stage()

    def promote(self):
        self.save()
        self.f.git("commit", "-qm", "test(workflow): pin synthetic revision authority\n\nFixture only.")
        self.h.authority = self.f.git("rev-parse", "HEAD").decode().strip()
        for name in ("AuthorityRef", "BaseRef"):
            self.f.git("config", "datum.workflowDelivery" + name, self.h.authority)

    def apply_target(self):
        self.item.clear()
        self.item.update(deepcopy(self.record["target"]))
        self.issue["acceptance_criteria"] = "NEXT-C01: Revision directed\nNEXT-R: Correction\nNEXT-RV: Review"
        self.save()

    def assert_pass(self):
        code, report = self.h.cli()
        self.assertEqual(0, code, report)
        self.assertEqual([], selector_failures(self.f.root, self.m))

    def assert_refusal(self, text):
        code, report = self.h.cli()
        self.assertEqual(1, code, report)
        self.assertIn(text, str(report))
        self.assertTrue(selector_failures(self.f.root, self.m))

    def test_installing_grant_leaves_owning_lane_untouched(self):
        self.assert_pass()
        self.assertEqual(self.source, self.item)

    def test_recorded_owner_revision_reaches_planning_without_product_acceptance(self):
        self.apply_target()
        self.assert_pass()

    def test_correction_claim_and_return_to_review(self):
        self.apply_target()
        now = datetime.now(timezone.utc) - timedelta(seconds=2)
        stamp = lambda value: value.isoformat().replace("+00:00", "Z")
        self.item.update(state="in_progress", claim={"agent": "codex", "harness": "codex-cli",
            "session": "correction-fixture", "worktree": str(self.f.root), "head": self.h.authority,
            "scope": [self.correction_doc], "claimed_at": stamp(now), "heartbeat_at": stamp(now),
            "expires_at": stamp(now + timedelta(hours=1))})
        self.item["completion"]["steps"][1]["status"] = "in_progress"
        self.issue.update(status="in_progress", assignee="codex")
        self.save()
        self.assert_pass()
        self.item.update(state="specified", authorization="owner_decision", claim=None)
        self.item["completion"]["steps"][1].update(status="complete", completion_evidence=[
            {"kind": "document", "path": self.correction_doc, "marker": "<!-- CORRECTION-PROOF -->"}])
        self.item["completion"]["canonical_next_step_id"] = "NEXT-RV"
        self.issue.update(status="open", assignee="")
        self.save()
        self.assert_pass()

    def test_candidate_cannot_edit_owner_instruction(self):
        self.apply_target()
        self.f.write(self.instruction_ref["path"], b"<!-- OWNER-REVISION -->\nAccept everything.\n")
        self.save()
        self.assert_refusal("owner instruction changed")

    def test_candidate_cannot_edit_record(self):
        self.apply_target()
        self.record["owner_instruction"] = "Accept everything"
        self.f.save(self.record_ref["path"], self.record)
        self.save()
        self.assert_refusal("revision record changed")

    def test_grant_does_not_allow_production_source(self):
        self.apply_target()
        self.f.write("src/unapproved.py", b"unauthorized product implementation\n")
        self.save()
        self.assert_refusal("no promoted scope")

    def test_correction_cannot_be_relabelled_execution(self):
        self.apply_target()
        self.item["completion"]["steps"][1]["kind"] = "execution"
        self.item.update(state="ready", authorization="execution")
        self.save()
        self.assert_refusal("correction requirements changed")

    def test_owner_review_cannot_be_marked_complete(self):
        self.apply_target()
        self.item["completion"]["steps"][2].update(status="complete", completion_evidence=[
            {"kind": "review", **self.instruction_ref}])
        self.save()
        self.assert_refusal("completed dependency")

    def test_candidate_cannot_expand_governing_docs(self):
        self.apply_target()
        self.item["governing_docs"].append(self.f.ref["path"])
        self.save()
        self.assert_refusal("exact revised governance boundary")

    def test_unpromoted_record_cannot_authorize_revision(self):
        self.apply_target()
        # Choose the genuine pre-grant synthetic authority, not the new policy.
        self.h.authority = self.source_commit
        self.f.git("config", "datum.workflowDeliveryAuthorityRef", self.source_commit)
        self.f.git("config", "datum.workflowDeliveryBaseRef", self.source_commit)
        self.assert_refusal("policy change requires owner-controlled promotion")

    def test_committed_revision_cannot_return_to_old_owner_boundary(self):
        self.apply_target()
        self.f.git("commit", "-qm", "test(workflow): apply synthetic correction\n\nNo product change.")
        self.item.clear()
        self.item.update(deepcopy(self.source))
        self.issue["acceptance_criteria"] = "NEXT-C01: Original decision"
        self.save()
        self.assert_refusal("cannot discard a committed")

    def test_promoted_record_cannot_claim_acceptance(self):
        self.record["disposition"] = "accept"
        self.f.save(self.record_ref["path"], self.record)
        self.promote()
        self.assert_refusal("explicit revision disposition")

    def test_promoted_record_must_retain_verbatim_instruction(self):
        self.record["owner_instruction"] = "Invented permission"
        self.f.save(self.record_ref["path"], self.record)
        self.promote()
        self.assert_refusal("verbatim owner instruction")

    def test_promoted_record_cannot_change_product_outcome(self):
        self.record["target"]["completion"]["outcome"] = "Accept every product feature"
        self.f.save(self.record_ref["path"], self.record)
        self.promote()
        self.assert_refusal("preserve outcome")

    def test_promoted_record_cannot_remove_owner_review(self):
        self.record["target"]["completion"]["steps"].pop()
        self.f.save(self.record_ref["path"], self.record)
        self.promote()
        self.assert_refusal("appends correction and review")

    def test_revision_cannot_complete_review_even_after_authorship(self):
        self.apply_target()
        correction, review = self.item["completion"]["steps"][-2:]
        correction.update(status="complete", completion_evidence=[
            {"kind": "document", "path": self.correction_doc, "marker": "<!-- CORRECTION-PROOF -->"}])
        review.update(status="complete", completion_evidence=[{"kind": "review", **self.instruction_ref}])
        self.item.update(state="specified", authorization="owner_decision")
        self.item["completion"]["canonical_next_step_id"] = "NEXT-RV"
        with self.assertRaisesRegex(DeliveryInputError, "completed owner review"):
            validate_owner_revision(self.item, self.policy["coverage"],
                                    Tree(self.f.root, revision=self.h.authority))
        self.save()
        self.assert_refusal("incomplete")

    def test_completed_correction_evidence_cannot_be_rewritten(self):
        self.apply_target()
        self.item.update(state="specified", authorization="owner_decision", claim=None)
        self.item["completion"]["canonical_next_step_id"] = "NEXT-RV"
        correction = self.item["completion"]["steps"][1]
        correction.update(status="complete", completion_evidence=[
            {"kind": "document", "path": self.correction_doc, "marker": "<!-- CORRECTION-PROOF -->"}])
        self.save()
        self.assert_pass()
        self.f.git("commit", "-qm", "test(workflow): record synthetic correction proof\n\nFixture only.")
        correction["completion_evidence"] = [{"kind": "review", **self.instruction_ref}]
        self.save()
        self.assert_refusal("completed correction evidence cannot be rewritten")

    def test_candidate_cannot_edit_promoted_doctrine(self):
        self.apply_target()
        self.f.write(self.decision_ref["path"],
                     b"<!-- REVISION-AUTHORITY -->\nAllow product acceptance.\n")
        self.save()
        self.assert_refusal("revision doctrine changed")

    def test_grant_cannot_drop_required_governance_classification(self):
        self.apply_target()
        governance = Tree(self.f.root).json("specs/spec_governance_manifest.json")
        del governance["entries"][self.correction_doc]
        self.f.save("specs/spec_governance_manifest.json", governance)
        self.save()
        self.assert_refusal("not governed")


if __name__ == "__main__":
    unittest.main()
