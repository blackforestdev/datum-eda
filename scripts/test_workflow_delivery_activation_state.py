"""Exercise partial publication observation in owned synthetic repositories."""

from copy import deepcopy
import unittest
from unittest.mock import patch

from workflow_delivery_activation_state import inspect_activation_state
from workflow_delivery_capture_state import protected_state
from workflow_delivery_test_support import Fixture


class ActivationStateTest(unittest.TestCase):
    def setUp(self):
        self.f = Fixture()
        self.addCleanup(self.f.close)
        self.f.git("branch", "-M", "main")
        self.base = self.f.head
        tree = self.f.git("rev-parse", "HEAD^{tree}").decode().strip()
        self.candidate = self.f.git("commit-tree", tree, "-p", self.base,
                                    "-m", "Owned publication fixture").decode().strip()
        self.prior = protected_state(self.f.root, ["src"])["local_trust"]
        self.proposed = deepcopy(self.prior)
        self.proposed["datum.workflowDeliveryAuthorityRef"] = [self.candidate]
        self.proposed["core.hooksPath"] = ["/explicit/fixture/hooks"]

    def observe(self):
        before = protected_state(self.f.root, ["src"])
        result = inspect_activation_state(self.f.root, base=self.base, candidate=self.candidate,
            input_roots=["src"], prior_trust=self.prior, proposed_trust=self.proposed)
        self.assertEqual(before, protected_state(self.f.root, ["src"]))
        self.assertFalse(result["activation_asserted"])
        self.assertFalse(result["repair_performed"])
        self.assertFalse(result["publication_authorized"])
        return result

    def test_untouched_base(self):
        self.assertEqual("not_started", self.observe()["state"])

    def test_published_candidate_with_old_trust_is_partial(self):
        self.f.git("merge", "--ff-only", self.candidate)
        self.assertEqual("partial", self.observe()["state"])

    def test_each_interrupted_configuration_prefix_and_final_unverified_state(self):
        self.f.git("merge", "--ff-only", self.candidate)
        self.f.git("config", "--local", "datum.workflowDeliveryAuthorityRef", self.candidate)
        self.assertEqual("partial", self.observe()["state"])
        self.f.git("config", "--local", "core.hooksPath", "/explicit/fixture/hooks")
        self.assertEqual("candidate_and_configuration_match_unverified", self.observe()["state"])

    def test_configuration_before_publication_is_partial(self):
        self.f.git("config", "--local", "datum.workflowDeliveryAuthorityRef", self.candidate)
        self.assertEqual("partial", self.observe()["state"])

    def test_unknown_or_duplicate_trust_is_diverged(self):
        self.f.git("config", "--local", "datum.workflowDeliveryAuthorityRef", "unexpected")
        self.assertEqual("diverged", self.observe()["state"])
        self.f.git("config", "--local", "datum.workflowDeliveryAuthorityRef", self.candidate)
        self.f.git("config", "--local", "--add", "datum.workflowDeliveryAuthorityRef", self.candidate)
        self.assertEqual("diverged", self.observe()["state"])

    def test_hidden_dirty_bytes_are_not_reported_as_not_started(self):
        self.f.git("update-index", "--assume-unchanged", "src/read.py")
        self.f.write("src/read.py", b"hidden dirty bytes\n")
        self.assertEqual("diverged", self.observe()["state"])

    def test_detached_checkout_is_diverged(self):
        self.f.git("checkout", "--detach", "-q", self.base)
        self.assertEqual("diverged", self.observe()["state"])

    def test_concurrent_state_change_is_reported_not_repaired(self):
        before = protected_state(self.f.root, ["src"])
        after = deepcopy(before)
        after["head"] = self.candidate
        with patch("workflow_delivery_activation_state.protected_state", side_effect=[before, after]):
            result = self.observe()
        self.assertEqual("diverged", result["state"])
        self.assertIn("protected state changed during observation", result["findings"])


if __name__ == "__main__":
    unittest.main()
