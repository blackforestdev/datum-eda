"""Synthetic environment refusal tests, never observed infrastructure proof."""

from copy import deepcopy
import unittest

from workflow_delivery_headless import headless_shape, validate_headless
from workflow_delivery_checkpoints import validate_delivery
from workflow_delivery_io import DeliveryInputError, canonical_json, sha256
from workflow_delivery_native import validate_environment
from workflow_delivery_native_test_support import typed_proof
from workflow_delivery_test_support import Fixture
from workflow_delivery_tree import Tree


def environment():
    return {"schema_version": 2, "kind": "headless", "os": "fixture-linux",
            "toolchain": "fixture-python", "transport": "pipes", "terminal_size": None,
            "input_method": "fixture arguments and pipes", "reproduction_commands": ["fixture run"],
            "tools": [{"name": name, "path": "/nonexistent/fixture-" + name,
                       "sha256": sha256(b"fixture interpreter"), "version": "fixture version"}
                      for name in ("interpreter", "git")]}


class HeadlessEnvironmentTest(unittest.TestCase):
    def reject_shape(self, value):
        with self.assertRaises(DeliveryInputError) as caught:
            headless_shape(value)
        self.assertEqual("WDQ-ENVIRONMENT", caught.exception.code)

    def test_pipes_require_no_invented_display(self):
        value = environment()
        self.assertIs(value, headless_shape(value))
        value["terminal_size"] = [80, 24]
        self.reject_shape(value)

    def test_pty_dimensions_are_positive_integer_character_counts(self):
        value = environment()
        value.update(transport="pty", terminal_size=[80, 24])
        headless_shape(value)
        for size in (None, [], [80], [80, 24, 1], [True, 24], [80.0, 24], [0, 24], [-1, 24]):
            with self.subTest(size=size):
                value["terminal_size"] = size
                self.reject_shape(value)

    def test_version_kind_and_fields_are_closed(self):
        for change in ({"schema_version": True}, {"schema_version": 2.0},
                       {"schema_version": 3}, {"kind": "gui"}, {"transport": "invented"},
                       {"scale": 1}, {"window_size": [1280, 768]}, {"os": " "}):
            with self.subTest(change=change):
                value = environment()
                value.update(change)
                self.reject_shape(value)

    def test_required_tools_are_unique_and_have_valid_metadata(self):
        values = []
        value = environment(); value["tools"] = []; values.append(value)
        value = environment(); value["tools"].pop(); values.append(value)
        value = environment(); value["tools"].append(deepcopy(value["tools"][0])); values.append(value)
        for field, invalid in (("sha256", "bad"), ("version", ""), ("path", ""), ("extra", True)):
            value = environment(); value["tools"][0][field] = invalid; values.append(value)
        for value in values:
            self.reject_shape(value)

    def test_reproduction_commands_are_nonempty_text(self):
        for commands in ([], [""], [False], "not an array"):
            value = environment()
            value["reproduction_commands"] = commands
            self.reject_shape(value)

    def test_paths_are_metadata_and_are_never_executed_or_opened(self):
        value = environment()
        validate_headless(value, deepcopy(value), category="infrastructure",
                          toolchain="fixture-python", binary_sha256=sha256(b"fixture interpreter"))

    def test_product_or_unknown_category_cannot_use_headless(self):
        value = environment()
        for category in ("product", None, "unknown"):
            with self.assertRaises(DeliveryInputError) as caught:
                validate_headless(value, value, category=category,
                    toolchain="fixture-python", binary_sha256=sha256(b"fixture interpreter"))
            self.assertEqual("WDQ-ENVIRONMENT", caught.exception.code)

    def test_recorded_and_requested_objects_match_exactly(self):
        value = environment()
        requested = deepcopy(value)
        requested["input_method"] = "different capture"
        with self.assertRaises(DeliveryInputError):
            validate_headless(value, requested, category="infrastructure",
                toolchain="fixture-python", binary_sha256=sha256(b"fixture interpreter"))

    def test_build_toolchain_and_interpreter_digest_are_bound(self):
        value = environment()
        for toolchain, binary in (("other", sha256(b"fixture interpreter")),
                                  ("fixture-python", sha256(b"other interpreter"))):
            with self.assertRaises(DeliveryInputError):
                validate_headless(value, value, category="infrastructure",
                                  toolchain=toolchain, binary_sha256=binary)

    def test_existing_environment_entrypoint_reads_hashed_headless_artifact(self):
        f = Fixture()
        self.addCleanup(f.close)
        proof = f.proof()
        value = environment()
        proof["environment"] = f.blob("evidence/headless.json", canonical_json(value))
        f.stage()
        before = f.snapshot()
        validate_environment(Tree(f.root), proof, value, contract=f.contract)
        self.assertEqual(before, f.snapshot())
        with self.assertRaises(DeliveryInputError):
            validate_environment(Tree(f.root), proof, value, contract={"category": "product"})
        with self.assertRaises(DeliveryInputError):
            validate_environment(Tree(f.root), proof, value)

    def test_producer_verification_checkpoint_passes_contract_category(self):
        f = Fixture()
        self.addCleanup(f.close)
        proof, _, _, _ = typed_proof(f)
        value = environment()
        proof["environment"] = f.blob("evidence/headless.json", canonical_json(value))
        f.save(f.contract["proof_path"], proof)
        f.stage()
        item = {"key": "TASK", "issue_id": "dat-test", "authorization": "none",
                "completion": {"canonical_next_step_id": None,
                    "delivery": {"contract_path": "contract.json", "checkpoints": {
                        "ready": "R", "activate": None, "verify": "V", "accept": None}},
                    "steps": [
                        {"id": "R", "kind": "planning", "status": "complete", "depends_on": []},
                        {"id": "V", "kind": "execution", "status": "complete", "depends_on": ["R"]}]}}
        self.assertEqual("verify", validate_delivery(Tree(f.root), item, environment=value))


if __name__ == "__main__":
    unittest.main()
