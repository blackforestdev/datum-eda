"""Completed authoring evidence through actual schema-2 entry points."""

import hashlib
import unittest

import test_workflow_delivery_readiness_boundary as support
from workflow_delivery_selector import selector_failures
from workflow_delivery_tree import Tree
from workflow_delivery_clause_test_support import inventory, matrix, refresh


class SpecificationBoundaryTest(unittest.TestCase):
    def setUp(self):
        self.r = support.ReadinessBoundaryTest()
        self.r.setUp()
        self.addCleanup(self.r.doCleanups)
        self.h, self.f = self.r.h, self.r.f
        item = self.r.item
        item["completion"]["steps"][2]["kind"] = "planning"
        del item["completion"]["delivery"]
        policy = Tree(self.f.root).json("specs/workflow_delivery_policy.json")
        policy["coverage"]["rows"][1]["category"] = "specification"
        self.inventory = inventory(self.f, item, policy)
        self.f.save("specs/workflow_delivery_policy.json", policy)
        self.step = item["completion"]["steps"][0]
        self.step.update(status="pending", completion_evidence=[])
        item.update(authorization="planning")
        item["completion"]["canonical_next_step_id"] = "NEXT-C01"
        self.r.save()
        self.f.git("commit", "-qm", "test(specification): pin pending authoring fixture\n\nSynthetic policy, not product authority.")
        self.h.authority = self.f.git("rev-parse", "HEAD").decode().strip()
        for key in ("AuthorityRef", "BaseRef"):
            self.f.git("config", "datum.workflowDelivery" + key, self.h.authority)
        self.matrix, self.matrix_ref = matrix(self.f, item, self.step, self.inventory["clauses"])
        self.step.update(status="complete", completion_evidence=[self.matrix_ref])
        item.update(authorization="owner_decision")
        item["completion"]["canonical_next_step_id"] = "NEXT-C02"
        self.r.save()

    def test_reviewed_output_passes_without_future_product_proof(self):
        code, report = self.h.cli()
        self.assertEqual(0, code, report)
        self.assertEqual([], selector_failures(self.f.root, self.r.m))

    def test_commit_alone_does_not_establish_authored_output(self):
        self.step["completion_evidence"] = [{"kind": "commit", "revision": self.h.authority}]
        self.r.save()
        code, report = self.h.cli()
        self.assertEqual(1, code, report)
        self.assertIn("requires governed output", report["findings"][0]["detail"])

    def test_governed_output_without_route_refuses(self):
        self.step["completion_evidence"] = [{"kind": "document",
            "path": self.r.item["governing_docs"][0], "marker": "NEXT-READY-EVIDENCE"}]
        self.r.save()
        code, report = self.h.cli()
        self.assertEqual(1, code, report)
        self.assertIn("requires an owning evidence route", report["findings"][0]["detail"])

    def test_stale_source_refuses_and_unstaged_repair_cannot_bless_index(self):
        old = self.f.root.joinpath("research/source.md").read_bytes()
        self.f.write("research/source.md", b"Unreviewed specification input.\n")
        self.f.stage()
        self.f.write("research/source.md", old)
        code, report = self.h.cli()
        self.assertEqual(1, code, report)
        self.assertIn("owning route review is stale", report["findings"][0]["detail"])
        self.assertEqual([], selector_failures(self.f.root, self.r.m))

    def test_stale_source_refuses_selector_too(self):
        self.f.write("research/source.md", b"Unreviewed source.\n")
        self.f.stage()
        self.assertTrue(any("owning route review is stale" in e
                            for e in selector_failures(self.f.root, self.r.m)))

    def save_matrix(self):
        self.f.save(self.matrix_ref["path"], self.matrix)
        refresh(self.f)
        self.r.save()

    def assert_refusal(self, detail):
        code, report = self.h.cli()
        self.assertEqual(1, code, report)
        self.assertIn(detail, report["findings"][0]["detail"])

    def promote_inventory(self):
        # Synthetic owner-selected inventory change, never a candidate bypass.
        self.f.save("docs/specification-inventory.json", self.inventory)
        refresh(self.f)
        self.r.save()
        self.f.git("commit", "-qm", "fixture reviewed inventory update")
        self.h.authority = self.f.git("rev-parse", "HEAD").decode().strip()
        for key in ("AuthorityRef", "BaseRef"):
            self.f.git("config", "datum.workflowDelivery" + key, self.h.authority)

    def test_ordinary_document_cannot_substitute_for_matrix(self):
        self.step["completion_evidence"] = [{"kind": "document", **self.f.ref}]
        self.r.save()
        self.assert_refusal("exactly one clause/disposition matrix")

    def test_candidate_cannot_shrink_promoted_inventory(self):
        self.inventory["clauses"].pop()
        self.f.save("docs/specification-inventory.json", self.inventory)
        refresh(self.f)
        self.r.save()
        self.assert_refusal("changed after owner promotion")

    def test_omitted_clause_refuses_even_with_fresh_review_digest(self):
        second = dict(self.inventory["clauses"][0], id="SECOND")
        self.inventory["clauses"].append(second)
        self.promote_inventory()
        self.assert_refusal("dispose every clause")

    def test_duplicate_disposition_refuses(self):
        self.matrix["dispositions"].append(dict(self.matrix["dispositions"][0]))
        self.save_matrix()
        self.assert_refusal("duplicate identity")

    def test_unknown_clause_refuses(self):
        self.matrix["dispositions"][0]["clause_id"] = "UNKNOWN"
        self.save_matrix()
        self.assert_refusal("unknown clause")

    def test_wrong_step_identity_refuses(self):
        self.matrix["step_id"] = "NEXT-C03"
        self.save_matrix()
        self.assert_refusal("identity mismatch")

    def test_mechanism_cannot_be_only_documented(self):
        self.inventory["clauses"][0]["requires_owner_decision"] = True
        self.promote_inventory()
        self.assert_refusal("explicit owner ratification")

    def test_pending_mechanism_has_an_explicit_downstream_owner_boundary(self):
        self.inventory["clauses"][0]["requires_owner_decision"] = True
        self.promote_inventory()
        self.matrix["dispositions"][0].update(disposition="pending_owner", owner_step_id="NEXT-C02")
        self.save_matrix()
        code, report = self.h.cli()
        self.assertEqual(0, code, report)
        self.assertEqual([], selector_failures(self.f.root, self.r.m))

    def test_pending_mechanism_cannot_point_to_a_planning_step(self):
        self.matrix["dispositions"][0].update(disposition="pending_owner", owner_step_id="NEXT-C03")
        self.save_matrix()
        self.assert_refusal("pending explicit owner-decision")

    def test_ordinary_document_cannot_ratify_a_mechanism(self):
        self.matrix["dispositions"][0]["disposition"] = "ratified"
        self.save_matrix()
        self.assert_refusal("numbered decision evidence")

    def test_missing_disposition_output_marker_refuses(self):
        self.matrix["dispositions"][0]["evidence_refs"][0]["marker"] = "ABSENT"
        self.save_matrix()
        self.assert_refusal("marker must occur exactly once")

    def test_promoted_inventory_must_cover_all_planned_requirements(self):
        self.inventory["clauses"].pop()
        self.promote_inventory()
        self.assert_refusal("cover every completion-step requirement")

    def add_decision(self):
        path = "docs/decisions/PRODUCT_MECHANICS_999_CLAUSE_FIXTURE.md"
        raw = b"<!-- RATIFIED-FIXTURE -->\nSynthetic controlling decision.\n"
        self.f.write(path, raw)
        governed = Tree(self.f.root).json("specs/spec_governance_manifest.json")
        governed["entries"][path] = {"class": "doctrine", "controlling": True}
        self.f.save("specs/spec_governance_manifest.json", governed)
        routes = Tree(self.f.root).json("specs/evidence_traceability_manifest.json")
        routes["routes"].append({"id": "ratification-fixture", "sources": [],
            "consumers": [path], "reviewed_digest": hashlib.sha256(
                path.encode() + b"\0" + raw + b"\0").hexdigest()})
        self.f.save("specs/evidence_traceability_manifest.json", routes)
        self.matrix["dispositions"][0].update(disposition="ratified", evidence_refs=[
            {"path": path, "marker": "RATIFIED-FIXTURE"}])
        self.save_matrix()

    def test_candidate_cannot_self_ratify_with_new_numbered_doctrine(self):
        self.add_decision()
        self.assert_refusal("owner-promoted controlling doctrine")

    def test_existing_promoted_controlling_doctrine_can_resolve_clause(self):
        self.inventory["clauses"][0]["requires_owner_decision"] = True
        self.add_decision()
        self.promote_inventory()
        code, report = self.h.cli()
        self.assertEqual(0, code, report)
        self.assertEqual([], selector_failures(self.f.root, self.r.m))

    def test_candidate_cannot_complete_the_owner_boundary(self):
        owner = self.r.item["completion"]["steps"][1]
        owner.update(status="complete", completion_evidence=[{"kind": "review", **self.f.ref}])
        self.r.item.update(authorization="planning")
        self.r.item["completion"]["canonical_next_step_id"] = "NEXT-C03"
        self.r.save()
        self.assert_refusal("owner decision requires exact owner promotion")


if __name__ == "__main__":
    unittest.main()
