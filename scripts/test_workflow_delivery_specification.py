"""Completed authoring evidence through actual schema-2 entry points."""

import unittest

import test_workflow_delivery_readiness_boundary as support
from workflow_delivery_selector import selector_failures
from workflow_delivery_tree import Tree


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
        self.step.update(status="complete", completion_evidence=[
            {"kind": "document", **self.f.ref}])
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


if __name__ == "__main__":
    unittest.main()
