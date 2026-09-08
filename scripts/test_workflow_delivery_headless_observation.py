"""Actual isolated interpreter/Git observations; no fabricated tool records."""

import hashlib
import json
import os
from pathlib import Path
import subprocess
import sys
import unittest


class HeadlessObservationTest(unittest.TestCase):
    def invoke(self, *, isolated=True, recipe=None, tty_input=False, include_hook_tools=False):
        directory = str(Path(__file__).resolve().parent)
        code = ("import sys,json; sys.path.insert(0," + repr(directory) + "); "
                "from workflow_delivery_headless_observation import observe_pipes_environment; "
                "print(json.dumps(observe_pipes_environment(reproduction_commands=" +
                repr(["Run the reviewed fixture capture recipe"] if recipe is None else recipe) +
                ", include_hook_tools=" + repr(include_hook_tools) + ")))" )
        descriptors = os.openpty() if tty_input else None
        try:
            return subprocess.run([sys.executable, *(["-I", "-S", "-B"] if isolated else []), "-c", code],
                                  input=None if tty_input else b"", stdin=descriptors[1] if descriptors else None,
                                  capture_output=True, timeout=20, env={"PATH": os.defpath, "LC_ALL": "C"})
        finally:
            if descriptors:
                for descriptor in descriptors:
                    os.close(descriptor)

    def test_measured_pipes_tools_and_raw_version_result(self):
        run = self.invoke()
        self.assertEqual(0, run.returncode, run.stderr)
        value = json.loads(run.stdout)
        self.assertFalse(value["selection_authorized"])
        self.assertFalse(value["input_closure_complete"])
        environment = value["environment"]
        self.assertEqual("pipes", environment["transport"])
        self.assertIsNone(environment["terminal_size"])
        for tool in environment["tools"]:
            self.assertEqual(hashlib.sha256(Path(tool["path"]).read_bytes()).hexdigest(), tool["sha256"])
        git = next(tool for tool in environment["tools"] if tool["name"] == "git")
        observed = value["git_version_invocation"]
        self.assertEqual([git["path"], "--version"], observed["argv"])
        self.assertEqual(git["version"], bytes.fromhex(observed["stdout_hex"]).decode().strip())
        self.assertEqual(0, observed["returncode"])
        self.assertTrue(value["runtime"]["modules"])

    def test_nonisolated_and_missing_recipe_refuse(self):
        for arguments in ({"isolated": False}, {"recipe": []}, {"recipe": [False]}, {"include_hook_tools": "yes"}):
            with self.subTest(arguments=arguments):
                result = self.invoke(**arguments)
                self.assertNotEqual(0, result.returncode)
                self.assertEqual(b"", result.stdout)

    def test_explicit_hook_tools_retain_actual_versions_and_hashes(self):
        run = self.invoke(include_hook_tools=True)
        self.assertEqual(0, run.returncode, run.stderr)
        value = json.loads(run.stdout)
        tools = {tool["name"]: tool for tool in value["environment"]["tools"]}
        self.assertEqual({"interpreter", "git", "bash", "realpath", "env"}, set(tools))
        for name, invocation in value["hook_tool_version_invocations"].items():
            tool = tools[name]
            self.assertEqual([tool["path"], "--version"], invocation["argv"])
            self.assertEqual(0, invocation["returncode"])
            self.assertEqual(tool["version"], bytes.fromhex(invocation["stdout_hex"]).decode().strip())
            self.assertEqual(hashlib.sha256(Path(tool["path"]).read_bytes()).hexdigest(), tool["sha256"])
        self.assertFalse(value["input_closure_complete"])
        self.assertFalse(value["selection_authorized"])

    def test_actual_tty_input_is_not_mislabelled_as_pipes(self):
        result = self.invoke(tty_input=True)
        self.assertNotEqual(0, result.returncode)
        self.assertEqual(b"", result.stdout)
        self.assertIn(b"TTY or mixed stdio", result.stderr)


if __name__ == "__main__":
    unittest.main()
