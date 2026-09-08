"""Read-only proposed support layout tests; no live hook installation."""

from pathlib import Path
import tempfile
import unittest

from workflow_delivery_support_store import support_locations, validate_support_entry, proposed_local_trust
from workflow_delivery_capture_state import TRUST_KEYS, protected_state
from workflow_delivery_test_support import Fixture


class SupportStoreTest(unittest.TestCase):
    def setUp(self):
        self.f = Fixture()
        self.addCleanup(self.f.close)

    def locations(self):
        return support_locations(self.f.root, self.f.head)

    def test_resolution_does_not_create_directories_or_change_git(self):
        before = {str(p): p.read_bytes() for p in self.f.root.rglob("*") if p.is_file()}
        locations = self.locations()
        self.assertEqual(self.f.root / ".git/datum-wdq/trusted" / self.f.head / "scripts/check_workflow_delivery.py",
                         locations["runner"])
        self.assertFalse(locations["store"].exists())
        self.assertEqual(before, {str(p): p.read_bytes() for p in self.f.root.rglob("*") if p.is_file()})

    def test_linked_worktree_uses_the_shared_git_common_store(self):
        expected = self.locations()
        with tempfile.TemporaryDirectory() as temporary:
            linked = Path(temporary) / "linked"
            self.f.git("worktree", "add", "--detach", str(linked), self.f.head)
            self.assertEqual(expected, support_locations(linked, self.f.head))

    def test_proposed_configuration_is_exact_and_does_not_install_itself(self):
        before = protected_state(self.f.root, ["src"])
        value = proposed_local_trust(self.f.root, authority=self.f.head, base=self.f.head,
                                     environment_path="specs/example.environments.json")
        self.assertEqual(set(TRUST_KEYS), set(value))
        self.assertEqual([str(self.locations()["runner"])], value["datum.workflowDeliveryRunnerPath"])
        self.assertEqual([str(self.locations()["hooks"])], value["core.hooksPath"])
        self.assertEqual([self.f.head], value["datum.workflowDeliveryAuthorityRef"])
        self.assertEqual([self.f.head], value["datum.workflowDeliveryBaseRef"])
        self.assertEqual(["specs/example.environments.json"], value["datum.workflowDeliveryEnvironmentPath"])
        self.assertEqual(before, protected_state(self.f.root, ["src"]))
        self.assertFalse(self.locations()["store"].exists())

    def test_proposal_rejects_ambiguous_pins_and_non_repository_environment_paths(self):
        for base in ("HEAD", "0" * 40):
            with self.assertRaises((ValueError, RuntimeError)):
                proposed_local_trust(self.f.root, authority=self.f.head, base=base,
                                     environment_path="environment.json")
        for path in ("../environment.json", "/environment.json", ".git/config", "", False):
            with self.subTest(path=path), self.assertRaises(ValueError):
                proposed_local_trust(self.f.root, authority=self.f.head, base=self.f.head,
                                     environment_path=path)

    def test_moving_unknown_or_noncommit_authority_refuses(self):
        for authority in ("HEAD", "0" * 40):
            with self.assertRaises((ValueError, RuntimeError)):
                support_locations(self.f.root, authority)
        self.f.git("tag", "-a", "fixture-tag", "-m", "Synthetic tag, not a commit authority")
        tag = self.f.git("rev-parse", "fixture-tag").decode().strip()
        with self.assertRaisesRegex(ValueError, "exact commit"):
            support_locations(self.f.root, tag)

    def test_symlinked_store_refuses_even_when_target_is_inside_checkout(self):
        (self.f.root / "redirected").mkdir()
        (self.f.root / ".git/datum-wdq").symlink_to(self.f.root / "redirected", target_is_directory=True)
        with self.assertRaisesRegex(ValueError, "symlink"):
            self.locations()

    def test_file_in_directory_location_refuses(self):
        (self.f.root / ".git/datum-wdq").write_text("not a directory")
        with self.assertRaises((ValueError, OSError)):
            self.locations()

    def test_exact_existing_runner_and_hook_paths_without_activation(self):
        locations = self.locations()
        for entry in ("runner", "hook"):
            path = locations[entry]
            path.parent.mkdir(parents=True, exist_ok=True)
            path.write_text("fixture bytes, not a trusted executable")
            self.assertEqual(path, validate_support_entry(self.f.root, self.f.head, path, entry=entry))
        self.assertNotIn(b"hooksPath", (self.f.root / ".git/config").read_bytes())

    def test_missing_external_relative_and_symlinked_entries_refuse(self):
        runner = self.locations()["runner"]
        for path in (runner, self.f.root / "scripts/check_workflow_delivery.py", Path("relative.py")):
            with self.assertRaises(ValueError):
                validate_support_entry(self.f.root, self.f.head, path, entry="runner")
        runner.parent.mkdir(parents=True)
        runner.symlink_to(self.f.root / "src/read.py")
        with self.assertRaisesRegex(ValueError, "nonredirected"):
            validate_support_entry(self.f.root, self.f.head, runner, entry="runner")


if __name__ == "__main__":
    unittest.main()
