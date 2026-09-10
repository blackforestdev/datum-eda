"""Publication preparation must retain actual history, not only the final tree."""

from copy import deepcopy
import unittest

from workflow_delivery_publication_delta import publication_delta, verify_publication_delta, verify_publication_review, inspect_activation_response
from workflow_delivery_io import canonical_json, sha256
from workflow_delivery_test_support import Fixture


class PublicationDeltaTest(unittest.TestCase):
    def setUp(self):
        self.f = Fixture()
        self.addCleanup(self.f.close)
        self.base = self.f.head

    def commit(self):
        self.f.stage()
        self.f.git("commit", "-qm", "test fixture publication input")
        return self.f.git("rev-parse", "HEAD").decode().strip()

    def delta(self, candidate):
        return publication_delta(self.f.root, base=self.base, candidate=candidate)

    def test_net_zero_history_retains_blobs_and_modes(self):
        original = (self.f.root / "src/read.py").read_bytes()
        self.f.write("src/read.py", b"intermediate source\n")
        first = self.commit()
        self.f.write("src/read.py", original)
        candidate = self.commit()
        result = self.delta(candidate)
        self.assertEqual([], result["net_changes"])
        self.assertEqual(["src/read.py"], result["touched_paths"])
        self.assertEqual([first, candidate], [row["commit"] for row in result["commits"]])
        change = result["commits"][0]["edges"][0]["changes"][0]
        self.assertEqual("100644", change["after"]["mode"])
        self.assertEqual(b"intermediate source\n", self.f.git("cat-file", "blob", change["after"]["oid"]))
        self.assertFalse(result["publication_authorized"])
        self.assertEqual(result, verify_publication_delta(self.f.root, result, base=self.base, candidate=candidate))
        review = {"delta_sha256": sha256(canonical_json(result)), "paths": result["touched_paths"]}
        self.assertEqual(review, verify_publication_review(result, review))
        for paths in ([], ["src"], ["src/read.py", "src/extra.py"], ["src/read.py", "src/read.py"]):
            with self.subTest(paths=paths), self.assertRaises(ValueError):
                verify_publication_review(result, dict(review, paths=paths))
        altered = deepcopy(result)
        altered["commits"][0]["edges"][0]["changes"] = []
        with self.assertRaisesRegex(ValueError, "differs"):
            verify_publication_delta(self.f.root, altered, base=self.base, candidate=candidate)
        altered = deepcopy(result)
        altered["publication_authorized"] = 0
        with self.assertRaisesRegex(ValueError, "differs"):
            verify_publication_delta(self.f.root, altered, base=self.base, candidate=candidate)

    def test_same_base_and_moving_refs_refuse(self):
        with self.assertRaisesRegex(ValueError, "must differ"):
            self.delta(self.base)
        with self.assertRaisesRegex(ValueError, "full pinned"):
            self.delta("HEAD")

    def test_review_cannot_bind_different_history_with_the_same_final_tree(self):
        self.f.write("src/read.py", b"candidate\n")
        candidate = self.commit()
        first = self.delta(candidate)
        tree = self.f.git("rev-parse", candidate + "^{tree}").decode().strip()
        later = self.f.git("commit-tree", tree, "-p", candidate,
                           "-m", "different exact history").decode().strip()
        second = self.delta(later)
        self.assertEqual(first["net_changes"], second["net_changes"])
        self.assertEqual(first["touched_paths"], second["touched_paths"])
        review = {"delta_sha256": sha256(canonical_json(first)), "paths": first["touched_paths"]}
        with self.assertRaisesRegex(ValueError, "digest differs"):
            verify_publication_review(second, review)

    def test_review_is_closed_and_paths_are_literal_sorted_file_names(self):
        self.f.write("src/read.py", b"candidate\n")
        result = self.delta(self.commit())
        review = {"delta_sha256": sha256(canonical_json(result)), "paths": result["touched_paths"]}
        for change in ({"delta_sha256": True}, {"delta_sha256": "bad"}, {"paths": "src/read.py"},
                       {"paths": [False]}, {"paths": ["../src/read.py"]}, {"paths": ["/src/read.py"]},
                       {"paths": ["src/z.py", "src/read.py"]}, {"owner_approved": True}):
            with self.subTest(change=change), self.assertRaises(ValueError):
                verify_publication_review(result, dict(review, **change))

    def test_external_response_matches_only_one_packet_without_claiming_authorship(self):
        request = "a" * 64
        value = {"schema_version": 1, "kind": "datum.workflow-delivery.activation-response",
                 "response": "WORKFLOW-DELIVERY-IMPLEMENTATION: approve ACTIVATE — " + request,
                 "source": "Synthetic input; not an actual owner response", "recorded_at": "2026-09-08"}
        raw = canonical_json(value)
        result = inspect_activation_response(raw, response_sha256=sha256(raw), request_sha256=request)
        self.assertTrue(result["response_matches_request"])
        for field in ("publication_authorized", "activation_asserted", "owner_identity_verified"):
            self.assertFalse(result[field])
        for change in ({"schema_version": True}, {"response": "approved"},
                       {"response": "WORKFLOW-DELIVERY-IMPLEMENTATION: defer — later"},
                       {"source": ""}, {"recorded_at": None}, {"owner_approved": True}):
            altered = canonical_json(dict(value, **change))
            with self.subTest(change=change), self.assertRaises(ValueError):
                inspect_activation_response(altered, response_sha256=sha256(altered), request_sha256=request)
        with self.assertRaisesRegex(ValueError, "exact activation request"):
            inspect_activation_response(raw, response_sha256=sha256(raw), request_sha256="b" * 64)
        with self.assertRaisesRegex(ValueError, "externally selected digest"):
            inspect_activation_response(raw + b"\n", response_sha256=sha256(raw), request_sha256=request)

    def test_merge_retains_each_parent_edge(self):
        self.f.write("src/read.py", b"side branch source\n")
        side = self.commit()
        original = self.f.git("rev-parse", self.base + "^{tree}").decode().strip()
        merge = self.f.git("commit-tree", original, "-p", self.base, "-p", side,
                           "-m", "synthetic merge").decode().strip()
        result = self.delta(merge)
        self.assertEqual([], result["net_changes"])
        self.assertEqual([self.base, side], result["commits"][-1]["parents"])
        self.assertEqual([], result["commits"][-1]["edges"][0]["changes"])
        self.assertEqual("src/read.py", result["commits"][-1]["edges"][1]["changes"][0]["path"])
        with self.assertRaisesRegex(ValueError, "every touched file"):
            verify_publication_review(result, {"delta_sha256": sha256(canonical_json(result)), "paths": []})

    def test_nonancestor_and_artifact_selected_pins_refuse(self):
        self.f.write("src/read.py", b"candidate\n")
        candidate = self.commit()
        tree = self.f.git("rev-parse", self.base + "^{tree}").decode().strip()
        sibling = self.f.git("commit-tree", tree, "-p", self.base, "-m", "sibling").decode().strip()
        with self.assertRaisesRegex(ValueError, "must descend"):
            publication_delta(self.f.root, base=sibling, candidate=candidate)
        value = self.delta(candidate)
        value["base"] = candidate
        with self.assertRaisesRegex(ValueError, "differs"):
            verify_publication_delta(self.f.root, value, base=self.base, candidate=candidate)


if __name__ == "__main__":
    unittest.main()
