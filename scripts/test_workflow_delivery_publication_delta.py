"""Publication preparation must retain actual history, not only the final tree."""

from copy import deepcopy
import unittest

from workflow_delivery_publication_delta import publication_delta, verify_publication_delta
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
