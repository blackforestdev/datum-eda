"""Shared field dispatch, guard presence, empty values and caller provenance."""
import os
import subprocess
import unittest
from unittest.mock import patch

from server_runtime import EngineDaemonClient
from tool_dispatch import dispatch_tool_call
from tools_catalog_data import TOOL_BY_NAME


class SymbolFieldCallers(unittest.TestCase):
    def request(self):
        return {"path": "/project", "symbol": "symbol-id", "field": "value",
                "expected_model_revision": "observed-revision",
                "expected_tip_transaction": "none", "value": ""}

    @patch.dict(os.environ, {}, clear=True)
    @patch("server_runtime.subprocess.run")
    def test_empty_value_clear_and_occurrence_use_real_registered_method(self, run):
        run.return_value = subprocess.CompletedProcess([], 0, stdout="{}", stderr="")
        request = self.request()
        dispatch_tool_call(EngineDaemonClient(), "datum.schematic.set_symbol_field", request)
        argv = run.call_args.args[0]
        self.assertEqual(argv, ["datum-eda", "--format", "json", "project", "set-symbol-field",
            "--root", "/project", "--symbol", "symbol-id", "--field", "value",
            "--expected-model-revision", "observed-revision", "--expected-tip-transaction", "none",
            "--value", ""])
        self.assertEqual(run.call_args.kwargs["env"]["DATUM_COMMIT_SOURCE"], "tool")
        self.assertEqual(run.call_args.kwargs["env"]["DATUM_TOOL_SURFACE"], "mcp")
        request.pop("value")
        request.update(clear=True, instance_path=["first", "second"])
        dispatch_tool_call(EngineDaemonClient(), "datum.schematic.set_symbol_field", request)
        argv = run.call_args.args[0]
        self.assertNotIn("--value", argv)
        self.assertEqual(argv[-5:], ["--clear", "--instance-path", "first", "--instance-path", "second"])

    @patch("server_runtime.subprocess.run")
    def test_missing_null_and_blank_guards_cannot_spawn_writer(self, run):
        for key in ["expected_model_revision", "expected_tip_transaction"]:
            for value in [None, "", "  "]:
                request = self.request()
                request[key] = value
                with self.assertRaises(ValueError):
                    dispatch_tool_call(EngineDaemonClient(), "datum.schematic.set_symbol_field", request)
            request = self.request()
            request.pop(key)
            with self.assertRaises(KeyError):
                dispatch_tool_call(EngineDaemonClient(), "datum.schematic.set_symbol_field", request)
        run.assert_not_called()

    def test_catalog_preserves_compatibility_and_bounds_supported_selectors(self):
        schema = TOOL_BY_NAME["datum.schematic.set_symbol_field"]["inputSchema"]
        self.assertEqual(schema["properties"]["field"]["enum"], ["reference", "value"])
        self.assertIn("expected_model_revision", schema["required"])
        self.assertIn("expected_tip_transaction", schema["required"])
        for name in ["set_symbol_reference", "set_symbol_value", "edit_symbol_field"]:
            self.assertIn("datum.schematic." + name, TOOL_BY_NAME)


if __name__ == "__main__":
    unittest.main()
