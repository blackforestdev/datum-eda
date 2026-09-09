"""Workspace classification primitives; integration and authority stay separate."""

import copy
import hashlib
import importlib.util
import marshal
from pathlib import Path
import py_compile
import stat
import sys
import tempfile
import unittest
from unittest.mock import patch

from workflow_delivery_workspace import WorkspaceInputs, read_stable, regular_file, workspace_policy_shape


class WorkspaceInputsTest(unittest.TestCase):
    def setUp(self):
        self.temp = tempfile.TemporaryDirectory(prefix="datum-workspace-")
        self.addCleanup(self.temp.cleanup)
        self.root = Path(self.temp.name)
        self.policy = {"schema_version": 1, "kind": "datum.workflow-delivery.workspace-inputs",
                       "local_files": [], "python_caches": "verified-source-v1",
                       "beads_runtime": "standard-v1"}
        self.tracked = {"scripts/example.py", ".beads/issues.jsonl", ".beads/config.yaml"}
        self.source = self.write("scripts/example.py", b"value = 42\n")

    def write(self, name, raw):
        path = self.root / name
        path.parent.mkdir(parents=True, exist_ok=True)
        path.write_bytes(raw)
        return path

    def reader(self, *, isolated=True):
        return WorkspaceInputs(self.root, self.policy, self.tracked, cache_isolated=isolated)

    def cache(self, *, optimize=0):
        py_compile.compile(str(self.source), doraise=True, optimize=optimize)
        path = Path(importlib.util.cache_from_source(str(self.source), optimization=optimize or ""))
        return path, path.relative_to(self.root).as_posix()

    def pin(self, name, category="owner_local"):
        path = self.root / name
        self.policy["local_files"].append({"path": name, "category": category,
            "sha256": hashlib.sha256(path.read_bytes()).hexdigest(),
            "size": path.stat().st_size, "mode": stat.S_IMODE(path.stat().st_mode)})

    def test_unknown_and_tracked_paths_remain_inputs(self):
        for name in ("scripts/new.py", "scripts/generated.rs", ".beads/evil.py", ".beads/config.yaml"):
            self.write(name, b"data\n")
            self.assertEqual("input", self.reader().classify(name))
        self.pin("scripts/example.py")
        self.assertEqual("input", self.reader().classify("scripts/example.py"))

    def test_generated_caches_require_source_only_startup(self):
        path, name = self.cache()
        before = path.read_bytes()
        self.assertEqual("input", self.reader(isolated=False).classify(name))
        self.assertEqual("python_cache", self.reader().classify(name))
        self.assertEqual(before, path.read_bytes())

    def test_all_normal_optimization_levels_and_recurring_generation(self):
        for optimize in (0, 1, 2):
            path, name = self.cache(optimize=optimize)
            self.assertEqual("python_cache", self.reader().classify(name))
            path.unlink()
            path, name = self.cache(optimize=optimize)
            self.assertEqual("python_cache", self.reader().classify(name))

    def test_forged_valid_header_is_an_input_and_never_executes(self):
        path, name = self.cache()
        sentinel = self.root / "payload-executed"
        code = compile(f"open({str(sentinel)!r}, 'w').write('bad')\n", str(self.source), "exec")
        path.write_bytes(path.read_bytes()[:16] + marshal.dumps(code))
        self.assertEqual("input", self.reader().classify(name))
        self.assertFalse(sentinel.exists())

    def test_malformed_headers_trailing_data_and_executable_cache_refuse(self):
        path, name = self.cache()
        original = path.read_bytes()
        for changed in (b"bad", b"xxxx" + original[4:], original[:4] + (8).to_bytes(4, "little") + original[8:], original + b"trailing"):
            path.write_bytes(changed)
            self.assertEqual("input", self.reader().classify(name))
        path.write_bytes(original)
        path.chmod(0o755)
        self.assertEqual("input", self.reader().classify(name))

    def test_new_orphan_and_stale_payload_remain_inputs(self):
        path, name = self.cache()
        self.source.write_text("value = 43\n")
        self.assertEqual("input", self.reader().classify(name))
        self.source.unlink()
        self.assertEqual("input", self.reader().classify(name))

    def test_untracked_source_does_not_make_cache_eligible(self):
        _, name = self.cache()
        self.tracked.clear()
        self.assertEqual("input", self.reader().classify(name))

    def test_cache_lookalikes_and_other_interpreter_tags_remain_inputs(self):
        path, name = self.cache()
        tag = sys.implementation.cache_tag
        for other in (f"scripts/cache/example.{tag}.pyc", name.replace(tag, tag + "0"),
                      "scripts/__pycache__/helper.py", "scripts/__pycache__/example.pyc"):
            self.write(other, path.read_bytes())
            self.assertEqual("input", self.reader().classify(other))

    def test_exact_owner_file_pin_never_authorizes_changes(self):
        name = "scripts/local-gui.sh"
        path = self.write(name, b"#!/bin/sh\nexit 0\n")
        path.chmod(0o775)
        self.pin(name)
        reader = self.reader()
        self.assertEqual("owner_local", reader.classify(name))
        self.write("scripts/neighbor.sh", path.read_bytes())
        self.assertEqual("input", reader.classify("scripts/neighbor.sh"))
        path.chmod(0o755)
        self.assertEqual("input", reader.classify(name))
        path.chmod(0o775)
        path.write_bytes(b"#!/bin/sh\nexit 1\n")
        self.assertEqual("input", reader.classify(name))

    def test_explicit_legacy_cache_pin_requires_isolation_and_exact_bytes(self):
        path, name = self.cache()
        self.source.unlink()
        self.pin(name, "legacy_python_cache")
        reader = self.reader()
        self.assertEqual("input", self.reader(isolated=False).classify(name))
        self.assertEqual("legacy_python_cache", reader.classify(name))
        path.write_bytes(path.read_bytes() + b"changed")
        self.assertEqual("input", reader.classify(name))

    def test_pinned_identity_requires_exact_size_not_only_an_upper_bound(self):
        _, cache_name = self.cache()
        self.write("scripts/local.sh", b"value")
        self.pin("scripts/local.sh")
        self.pin(cache_name, "legacy_python_cache")
        self.policy["python_caches"] = "none"
        for row in self.policy["local_files"]:
            row["size"] += 1
        reader = self.reader()
        self.assertEqual("input", reader.classify("scripts/local.sh"))
        self.assertEqual("input", reader.classify(cache_name))

    def test_legacy_pin_may_transition_only_to_fully_verified_derived_cache(self):
        path, name = self.cache()
        self.pin(name, "legacy_python_cache")
        reader = self.reader()
        raw = path.read_bytes()
        path.write_bytes(raw[:8] + b"12345678" + raw[16:])
        self.assertEqual("python_cache", reader.classify(name))
        path.write_bytes(raw[:16] + b"unverified replacement")
        self.assertEqual("input", reader.classify(name))

    def test_beads_runtime_can_change_but_canonical_and_unknown_files_stay_inputs(self):
        for name in (".beads/beads.db", ".beads/beads.db-wal", ".beads/.write.lock",
                     ".beads/.br_history/issues.20260909_120000_123456789.jsonl"):
            path = self.write(name, b"runtime")
            self.assertEqual("beads_runtime", self.reader().classify(name))
            path.write_bytes(b"updated runtime")
            self.assertEqual("beads_runtime", self.reader().classify(name))
            path.chmod(0o755)
            self.assertEqual("input", self.reader().classify(name))
        for name in (".beads/issues.jsonl", ".beads/config.yaml", ".beads/new.db", ".beads/.br_history/evil.py"):
            self.write(name, b"data")
            self.assertEqual("input", self.reader().classify(name))

    def test_symlink_files_and_parent_redirects_are_not_exempt(self):
        target = self.write("other", b"data")
        path = self.root / ".beads/beads.db"
        path.parent.mkdir()
        path.symlink_to(target)
        self.assertEqual("input", self.reader().classify(".beads/beads.db"))
        (self.root / "redirect").symlink_to(self.root / "scripts", target_is_directory=True)
        self.assertEqual("input", self.reader().classify("redirect/example.py"))

    def test_policy_is_a_snapshot_not_mutable_caller_permission(self):
        self.write("scripts/local.sh", b"old")
        self.pin("scripts/local.sh")
        reader = self.reader()
        self.write("scripts/local.sh", b"new")
        self.policy["local_files"][0]["sha256"] = hashlib.sha256(b"new").hexdigest()
        self.assertEqual("input", reader.classify("scripts/local.sh"))

    def test_replaced_path_during_read_is_not_an_unchanged_file(self):
        name = "scripts/local.sh"
        path = self.write(name, b"old")
        replacement = self.write("replacement", b"new")
        captured = regular_file(self.root, name)
        original_fstat = __import__("os").fstat
        calls = 0

        def replace_after_read(fd):
            nonlocal calls
            result = original_fstat(fd)
            calls += 1
            if calls == 2:
                replacement.replace(path)
            return result

        with patch("workflow_delivery_workspace.os.fstat", side_effect=replace_after_read):
            self.assertIsNone(read_stable(captured, limit=3))
        self.assertEqual(b"new", path.read_bytes())

    def test_closed_policy_rejects_ambiguous_pins_and_broad_exemptions(self):
        self.write("scripts/local.sh", b"value")
        self.pin("scripts/local.sh")
        bad = []
        for key, value in (("schema_version", True), ("python_caches", "ignore-all"), ("unknown", True)):
            item = copy.deepcopy(self.policy); item[key] = value; bad.append(item)
        for key, value in (("path", ".git/config"), ("path", ".beads/issues.jsonl"), ("size", True),
                           ("mode", True), ("sha256", "invalid"), ("category", "all")):
            item = copy.deepcopy(self.policy); item["local_files"][0][key] = value; bad.append(item)
        item = copy.deepcopy(self.policy); item["local_files"].append(item["local_files"][0]); bad.append(item)
        for item in bad:
            with self.assertRaises(ValueError):
                workspace_policy_shape(item)


if __name__ == "__main__":
    unittest.main()
