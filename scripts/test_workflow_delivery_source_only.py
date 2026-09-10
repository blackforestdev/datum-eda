"""Actual child-process cache isolation, not workspace classification proof."""

import importlib.util
import marshal
from pathlib import Path
import py_compile
import shutil
import subprocess
import sys
import tempfile
import unittest


BOOTSTRAP = Path(__file__).with_name("workflow_delivery_source_only.py")


class SourceOnlyTest(unittest.TestCase):
    def setUp(self):
        self.tmp = tempfile.TemporaryDirectory(prefix="datum-cache-proof-")
        self.addCleanup(self.tmp.cleanup)
        self.root = Path(self.tmp.name)
        self.source = self.root / "example.py"
        self.source.write_text("value = 'source'\n")
        py_compile.compile(str(self.source), doraise=True)
        self.cache = Path(importlib.util.cache_from_source(str(self.source)))
        self.sentinel = self.root / "cache-executed"

    def run_child(self, *, protected=True):
        # Execute the bootstrap as source, never import it through cache lookup.
        code = "import sys\n"
        if protected:
            code += (
                "namespace = {'__name__': '_datum_source_bootstrap'}\n"
                f"with open({str(BOOTSTRAP)!r}, 'rb') as stream: raw = stream.read()\n"
                f"exec(compile(raw, {str(BOOTSTRAP)!r}, 'exec'), namespace)\n"
                f"namespace['install']({str(self.root)!r})\n"
            )
        code += f"sys.path.insert(0, {str(self.root)!r})\nimport example\nprint(example.value)\n"
        return subprocess.run([sys.executable, "-I", "-S", "-B", "-c", code],
                              capture_output=True, text=True, check=False)

    def forge_cache(self):
        payload = compile(
            f"open({str(self.sentinel)!r}, 'w').write('executed')\nvalue = 'forged'\n",
            str(self.source), "exec")
        self.cache.write_bytes(self.cache.read_bytes()[:16] + marshal.dumps(payload))

    def test_forged_valid_header_executes_without_source_isolation(self):
        self.forge_cache()
        result = self.run_child(protected=False)
        self.assertEqual(0, result.returncode, result.stderr)
        self.assertEqual("forged\n", result.stdout)
        self.assertTrue(self.sentinel.exists())

    def test_forged_valid_header_cannot_execute_under_source_isolation(self):
        self.forge_cache()
        before = self.cache.read_bytes()
        result = self.run_child()
        self.assertEqual(0, result.returncode, result.stderr)
        self.assertEqual("source\n", result.stdout)
        self.assertFalse(self.sentinel.exists())
        self.assertEqual(before, self.cache.read_bytes())

    def test_normal_cache_is_preserved_and_source_is_used(self):
        before = self.cache.read_bytes()
        result = self.run_child()
        self.assertEqual(0, result.returncode, result.stderr)
        self.assertEqual("source\n", result.stdout)
        self.assertEqual(before, self.cache.read_bytes())

    def test_missing_cache_is_not_created(self):
        self.cache.unlink()
        result = self.run_child()
        self.assertEqual(0, result.returncode, result.stderr)
        self.assertFalse(self.cache.exists())

    def test_sourceless_legacy_bytecode_refuses(self):
        legacy = self.root / "example.pyc"
        legacy.write_bytes(self.cache.read_bytes())
        self.source.unlink()
        result = self.run_child()
        self.assertNotEqual(0, result.returncode)
        self.assertIn("refuses sourceless repository bytecode", result.stderr)

    def test_malformed_cache_is_not_consumed_or_repaired(self):
        self.cache.write_bytes(b"not a valid cache")
        result = self.run_child()
        self.assertEqual(0, result.returncode, result.stderr)
        self.assertEqual("source\n", result.stdout)
        self.assertEqual(b"not a valid cache", self.cache.read_bytes())


class RealEntrypointCacheTest(unittest.TestCase):
    def test_real_selector_cli_and_preflight_do_not_execute_forged_cache(self):
        cases = [
            ("project_status.py", "project_task_details.py", ["--help"], 0),
            ("project_status.py", "workflow_delivery_source_only.py", ["--help"], 0),
            ("check_workflow_delivery.py", "workflow_delivery_authority.py", ["--help"], 0),
            ("workflow_delivery_preflight_cli.py", "workflow_delivery_activation_preflight.py", None, 2),
        ]
        for entry, imported, arguments, expected_exit in cases:
            with self.subTest(entry=entry), tempfile.TemporaryDirectory(prefix="datum-entry-cache-") as directory:
                root = Path(directory)
                scripts = root / "scripts"
                scripts.mkdir()
                # Actual implementation files, not stubbed handlers or imports.
                for path in BOOTSTRAP.parent.glob("*.py"):
                    shutil.copy2(path, scripts / path.name)
                source = scripts / imported
                py_compile.compile(str(source), doraise=True)
                cache = Path(importlib.util.cache_from_source(str(source)))
                sentinel = root / "cache-executed"
                payload = compile(f"open({str(sentinel)!r}, 'w').write('executed')\n", str(source), "exec")
                forged = cache.read_bytes()[:16] + marshal.dumps(payload)
                cache.write_bytes(forged)
                if arguments is None:
                    request = root / "request.json"
                    request.write_text("{}\n")
                    arguments = ["--inspect", "--root", str(root), "--request", str(request),
                                 "--request-sha256", "0" * 64, "--output", str(root / "result.json")]
                flags = ["-I", "-S", "-B"] if entry == "workflow_delivery_preflight_cli.py" else ["-B"]
                result = subprocess.run([sys.executable, *flags, str(scripts / entry), *arguments],
                                        capture_output=True, text=True, check=False)
                if entry == "workflow_delivery_preflight_cli.py":
                    self.assertIn("request bytes differ", result.stdout)
                self.assertEqual(expected_exit, result.returncode, result.stdout + result.stderr)
                self.assertFalse(sentinel.exists(), result.stdout + result.stderr)
                self.assertEqual(forged, cache.read_bytes())


if __name__ == "__main__":
    unittest.main()
