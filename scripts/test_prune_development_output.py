"""Safety regression checks for released development-output pruning."""

from pathlib import Path
import errno
import json
import os
import shutil
import subprocess
import sys
import tempfile
import time
from types import SimpleNamespace
import unittest
from unittest.mock import patch

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

    def test_compiler_mode_ignores_unrelated_app_names_but_checks_selected_maps(self):
        with tempfile.TemporaryDirectory() as directory:
            root = Path(directory)
            process = root / "proc/77"
            process.mkdir(parents=True)
            (process / "fd").mkdir()
            (process / "cmdline").write_bytes(b"/usr/bin/unrelated\0")
            (process / "status").write_text("State:\tS (sleeping)\n")
            (process / "cwd").symlink_to("/unrelated")
            (process / "exe").symlink_to("/usr/bin/unrelated")
            selected = root / "selected-output"
            (process / "maps").write_text(f"0-1 r-xp 0 00:00 0 {selected}\n")
            listing = Path.iterdir
            def entries(path):
                return iter([process]) if path == Path("/proc") else listing(path)
            def named(*, builds_only=False):
                return [] if builds_only else ["unrelated EDA daemon"]
            with patch.object(prune.cache, "active_processes", side_effect=named), \
                    patch.object(Path, "iterdir", entries):
                with self.assertRaisesRegex(guard.ResourcePolicyError, "active output mapping"):
                    prune.refuse_open_output([selected], check_mappings=True, builds_only=True)
                (process / "maps").write_text("0-1 r-xp 0 00:00 0 /unrelated/output\n")
                prune.refuse_open_output([selected], check_mappings=True, builds_only=True)
                with self.assertRaisesRegex(guard.ResourcePolicyError, "build/application is active"):
                    prune.refuse_open_output([selected])

    def test_only_exact_own_batch_descriptors_are_exempt_other_live_users_refuse(self):
        with tempfile.TemporaryDirectory() as directory:
            selected = Path(directory) / "selected.rlib"
            selected.write_bytes(b"!<arch>\ncompiler cache")
            own = os.open(selected, os.O_RDONLY)
            listing = Path.iterdir
            processes = [Path("/proc") / str(os.getpid())]
            def entries(path):
                return iter(processes) if path == Path("/proc") else listing(path)
            child = None
            try:
                with patch.object(prune.cache, "active_processes", return_value=[]), \
                        patch.object(Path, "iterdir", entries):
                    prune.refuse_open_output([selected], builds_only=True, admitted_fds=frozenset([own]))
                    extra = os.open(selected, os.O_RDONLY)
                    try:
                        with self.assertRaisesRegex(guard.ResourcePolicyError, "active output reference"):
                            prune.refuse_open_output([selected], builds_only=True, admitted_fds=frozenset([own]))
                    finally:
                        os.close(extra)
                    ready = Path(directory) / "ready"
                    code = ("import os,time;from pathlib import Path;"
                            f"fd=os.open({str(selected)!r},os.O_RDONLY);os.dup2(fd,{own});"
                            f"Path({str(ready)!r}).write_text('ready');time.sleep(30)")
                    child = subprocess.Popen([sys.executable, "-c", code])
                    deadline = time.monotonic() + 5
                    while not ready.exists() and time.monotonic() < deadline:
                        self.assertIsNone(child.poll())
                        time.sleep(.01)
                    self.assertTrue(ready.exists())
                    processes.append(Path("/proc") / str(child.pid))
                    with self.assertRaisesRegex(guard.ResourcePolicyError, "active output reference"):
                        prune.refuse_open_output([selected], builds_only=True, admitted_fds=frozenset([own]))
            finally:
                if child is not None:
                    child.terminate()
                    child.wait(timeout=5)
                os.close(own)

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


class CompilerCachePruneTests(unittest.TestCase):
    def setUp(self):
        temp = tempfile.TemporaryDirectory()
        self.addCleanup(temp.cleanup)
        self.root = Path(temp.name)
        subprocess.run(["git", "init", "-q", self.root], check=True)
        for profile in ("debug", "release"):
            directory = self.root / "target" / profile
            directory.mkdir(parents=True)
            for name in prune.BINS:
                shutil.copy2(sys.executable, directory / name)
        self.deps = self.root / "target/debug/deps"
        self.deps.mkdir()
        self.old = self.deps / "libobsolete-0123456789abcdef.rlib"
        self.old.write_bytes(b"!<arch>\nrebuildable archive")
        self.proof = self.deps / "proof-0123456789abcdef"
        shutil.copy2(sys.executable, self.proof)
        self.alias = self.deps / "current-0123456789abcdef"
        os.link(self.root / "target/debug/datum-gui", self.alias)
        self.dependency = self.deps / "old.d"
        self.dependency.write_text("retained dependency evidence")
        self.raw = self.deps / "proof.log"
        self.raw.write_text("raw evidence")
        fixture = self.deps / "native"
        fixture.mkdir()
        (fixture / "project.json").write_text("{}")
        self.native = fixture / "library.rmeta"
        self.native.write_bytes(b"rustfixture bytes")
        self.library = self.deps / "liblive-0123456789abcdef.so"
        shutil.copy2(sys.executable, self.library)
        self.keep = [str(self.proof.relative_to(self.root))]

    def plan(self):
        with patch.object(prune.guard, "load_policy", return_value=SimpleNamespace(lock_path=self.root / "lock")), \
                patch.object(prune.cache, "active_processes", return_value=[]):
            return prune.compiler_plan(self.root, self.keep)

    def apply(self, plan, probe=None):
        with patch.object(prune.guard, "load_policy", return_value=SimpleNamespace(lock_path=self.root / "lock")), \
                patch.object(prune.cache, "active_processes", return_value=[]), \
                patch.object(prune, "refuse_open_output", side_effect=probe):
            return prune.apply_compiler_plan(self.root, plan)

    def test_only_generated_files_removed_runtime_aliases_and_evidence_exact(self):
        plan = self.plan()
        self.assertEqual([x["path"] for x in plan["selected"]], [str(self.old.relative_to(self.root))])
        kept = [self.proof, self.alias, self.dependency, self.raw, self.native, self.library]
        before = {path: path.read_bytes() for path in kept}
        normal = {path: path.read_bytes() for profile in ("debug", "release")
                  for path in (self.root / "target" / profile).iterdir() if path.is_file()}
        result = self.apply(plan)
        self.assertEqual(result["status"], "complete_current_runtime_and_evidence_retained")
        self.assertFalse(self.old.exists())
        for path, data in {**before, **normal}.items():
            self.assertEqual(path.read_bytes(), data)
        self.assertEqual(self.alias.stat().st_ino, (self.root / "target/debug/datum-gui").stat().st_ino)

    def test_changed_input_refuses_before_unlink(self):
        plan = self.plan()
        self.old.write_bytes(b"!<arch>\nchanged cache")
        with self.assertRaisesRegex(guard.ResourcePolicyError, "changed"):
            self.apply(plan)
        self.assertTrue(self.old.exists())

    def test_raw_metadata_and_malformed_object_lookalikes_are_retained(self):
        raw = self.deps / "notes.rmeta"
        raw.write_bytes(b"rust notes are raw evidence, not compiler metadata")
        unsupported = self.deps / "unsupported.rmeta"
        unsupported.write_bytes(b"rust\x00\x00\x00\x09other version")
        malformed = self.deps / "notes.o"
        header = bytearray(64)
        header[:6] = b"\x7fELF\x02\x01"
        header[16:18] = (1).to_bytes(2, "little")
        malformed.write_bytes(header)
        program = self.deps / "notes-0123456789abcdef"
        broken = bytearray(120)
        broken[:6] = b"\x7fELF\x02\x01"
        broken[16:18] = (2).to_bytes(2, "little")
        broken[32:40] = (64).to_bytes(8, "little")
        broken[54:56] = (56).to_bytes(2, "little")
        broken[56:58] = (1).to_bytes(2, "little")
        broken[64:68] = (3).to_bytes(4, "little")
        program.write_bytes(broken)
        valid = self.deps / "libgenerated-0123456789abcdef.rmeta"
        valid.write_bytes(b"rust\x00\x00\x00\x0agenerated metadata")
        plan = self.plan()
        selected = {x["path"] for x in plan["selected"]}
        self.assertIn(str(valid.relative_to(self.root)), selected)
        for path in (raw, unsupported, malformed, program):
            self.assertNotIn(str(path.relative_to(self.root)), selected)
        self.apply(plan)
        self.assertFalse(valid.exists())
        for path in (raw, unsupported, malformed, program):
            self.assertTrue(path.exists())

    def test_tracked_input_and_active_mapping_are_not_cleaned(self):
        plan = self.plan()
        subprocess.run(["git", "add", str(self.old.relative_to(self.root))], cwd=self.root, check=True)
        with self.assertRaisesRegex(guard.ResourcePolicyError, "tracked"):
            self.apply(plan)
        subprocess.run(["git", "reset", "-q"], cwd=self.root, check=True)
        def active(*args, **kwargs):
            raise guard.ResourcePolicyError("active output mapping")
        with self.assertRaisesRegex(guard.ResourcePolicyError, "active output mapping"):
            self.apply(plan, active)
        self.assertTrue(self.old.exists())

    def test_symlink_and_escaped_plan_refuse_without_touching_data(self):
        plan = self.plan()
        plan["selected"][0]["path"] = "../outside"
        with self.assertRaisesRegex(guard.ResourcePolicyError, "escapes"):
            self.apply(plan)
        self.old.unlink()
        self.old.symlink_to(self.raw)
        with self.assertRaisesRegex(guard.ResourcePolicyError, "unknown compiler cache entry"):
            self.plan()
        self.assertEqual(self.raw.read_text(), "raw evidence")

    def test_build_activity_and_protected_changes_refuse(self):
        plan = self.plan()
        with patch.object(prune.guard, "load_policy", return_value=SimpleNamespace(lock_path=self.root / "lock")), \
                patch.object(prune.cache, "active_processes", return_value=["cargo"]):
            with self.assertRaisesRegex(guard.ResourcePolicyError, "idle"):
                prune.apply_compiler_plan(self.root, plan)
        self.proof.write_bytes(b"\x7fELFchanged protected proof")
        with self.assertRaisesRegex(guard.ResourcePolicyError, "changed"):
            self.apply(plan)
        self.assertTrue(self.old.exists())

    def test_real_running_and_held_program_admission(self):
        image = self.deps / "obsolete-0123456789abcdef"
        shutil.copy2(sys.executable, image)
        plan = self.plan()
        process = subprocess.Popen([str(image), "-c", "import time; time.sleep(30)"])
        try:
            with self.assertRaises(OSError) as caught:
                self.apply(plan)
            self.assertEqual(caught.exception.errno, errno.ETXTBSY)
            self.assertTrue(self.old.exists())
        finally:
            process.terminate()
            process.wait(timeout=5)
        def attempt_exec(*args, **kwargs):
            with self.assertRaises(OSError) as caught:
                subprocess.run([str(image), "-c", "pass"], check=True)
            self.assertEqual(caught.exception.errno, errno.ETXTBSY)
        self.apply(plan, attempt_exec)
        self.assertFalse(image.exists())


if __name__ == "__main__":
    unittest.main()
