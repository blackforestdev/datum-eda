"""Safety regression checks for released development-output pruning."""

from pathlib import Path
import tempfile
import unittest

import prune_development_output as prune
import run_cargo_guarded as guard


class DevelopmentPruneTests(unittest.TestCase):
    def test_alternate_runtime_cannot_keep_stale_destination_dependencies(self):
        with tempfile.TemporaryDirectory() as directory:
            root = Path(directory)
            debug = root / "debug"
            debug.mkdir()
            binary = debug / "datum-eda"
            binary.write_text("runtime")
            dependencies = binary.with_suffix(".d")
            dependencies.write_text("old source basis")
            self.assertEqual(prune.retained_dependency_records(debug, [binary]), [dependencies])
            self.assertEqual(prune.retained_dependency_records(root / "alternate", [binary]), [])
            self.assertIn(dependencies, prune.removable_entries(debug, [binary]))

    def test_preserving_output_root_protects_all_active_task_content(self):
        with tempfile.TemporaryDirectory() as directory:
            root = Path(directory) / ".datum"
            root.mkdir()
            (root / "active.log").write_text("unreleased proof")
            self.assertEqual(prune.removable_entries(root, [root]), [])
            self.assertTrue((root / "active.log").exists())

    def test_nondumpable_user_supervisor_does_not_exempt_workspace_programs(self):
        self.assertTrue(prune.is_user_service_manager([b"/usr/lib/systemd/systemd", b"--user", b"--deserialize=33"]))
        for command in ([b"target/systemd", b"--user"],
                        [b"/usr/lib/systemd/systemd", b"--system"],
                        [b"/usr/lib/systemd/systemd", b"--user", b"--exec=target/worker"]):
            self.assertFalse(prune.is_user_service_manager(command))
        with tempfile.TemporaryDirectory() as directory:
            parent = Path(directory) / "10"
            child = Path(directory) / "11"
            parent.mkdir()
            child.mkdir()
            (child / "status").write_text("PPid:\t10\n")
            (parent / "cmdline").write_bytes(b"target/worker\0")
            self.assertFalse(prune.is_user_service_process(child, [b"(sd-pam)"]))
            (parent / "cmdline").write_bytes(b"/usr/lib/systemd/systemd\0--user\0")
            self.assertTrue(prune.is_user_service_process(child, [b"(sd-pam)"]))

    def test_nested_projects_and_runtime_files_are_not_selected(self):
        with tempfile.TemporaryDirectory() as directory:
            root = Path(directory)
            fixture = root / "target/old-source/source/project"
            fixture.mkdir(parents=True)
            (fixture / "project.json").write_text("{}")
            (fixture / "journal").write_text("owned design data")
            debug = root / "target/debug"
            debug.mkdir()
            binary = debug / "datum-gui"
            binary.write_text("runtime")
            cache = debug / "incremental"
            cache.mkdir()
            (cache / "object").touch()
            projects = prune.project_roots(root)
            self.assertEqual(projects, [fixture])
            selected = prune.removable_entries(root / "target", projects + [binary])
            self.assertEqual(selected, [cache])
            self.assertTrue((fixture / "journal").exists())

    def test_baseline_named_directory_is_preserved_without_worktree_registration(self):
        with tempfile.TemporaryDirectory() as directory:
            root = Path(directory)
            project = root / "target/s5a-manufacturing-baseline/authored"
            project.mkdir(parents=True)
            (project / "project.json").write_text("{}")
            self.assertEqual(prune.project_roots(root), [project])
            self.assertEqual(prune.removable_entries(root / "target", [project]), [])

    def test_runtime_copy_refuses_symlink_destinations_and_temporary_paths(self):
        with tempfile.TemporaryDirectory() as directory:
            root = Path(directory)
            debug = root / "debug"
            debug.mkdir()
            external = root / "private"
            external.write_text("keep")
            source = root / "runtime"
            source.write_bytes(b"runtime")
            destination = debug / "datum-gui"
            for link in (destination, debug / "datum-gui.maintenance-new"):
                link.symlink_to(external)
                with self.assertRaises(guard.ResourcePolicyError):
                    prune.copy_runtime(source, destination)
                self.assertEqual(external.read_text(), "keep")
                link.unlink()
            debug.rmdir()
            debug.symlink_to(root, target_is_directory=True)
            with self.assertRaises(guard.ResourcePolicyError):
                prune.copy_runtime(source, destination)

    def test_preview_enumerates_all_selected_output(self):
        with tempfile.TemporaryDirectory() as directory:
            root = Path(directory)
            roots = [root / "target", root / ".datum", root / "wdq"]
            for base in roots:
                base.mkdir()
            cache = roots[0] / "cache"
            cache.mkdir()
            kept = roots[1] / "project"
            kept.mkdir()
            self.assertEqual(prune.deletion_plan(roots, [kept]), [cache, roots[2]])

    def test_symlink_is_selected_without_traversing_external_data(self):
        with tempfile.TemporaryDirectory() as directory:
            root = Path(directory)
            output = root / "target"
            output.mkdir()
            external = root / "external"
            external.mkdir()
            (external / "project.json").write_text("private")
            link = output / "external-link"
            link.symlink_to(external, target_is_directory=True)
            self.assertEqual(prune.removable_entries(output, []), [link])
            self.assertEqual((external / "project.json").read_text(), "private")
            with self.assertRaises(guard.ResourcePolicyError):
                prune.removable_entries(link, [])


if __name__ == "__main__":
    unittest.main()
