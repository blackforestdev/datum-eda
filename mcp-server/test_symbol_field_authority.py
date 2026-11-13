"""Scoped authority and native guard continuity for the shared Symbol writer."""

import json
import subprocess
import unittest
from unittest.mock import patch

from server_runtime import EngineDaemonClient
from test_agent_capability import ScopedHostFixture


TOOL = "datum.schematic.set_symbol_field"
TIP = "10000000-0000-0000-0000-000000000001"
OTHER_TIP = "20000000-0000-0000-0000-000000000002"


class WriterFixture(ScopedHostFixture):
    def __init__(self, capabilities, *, tip=TIP, allowlist=None):
        self.tip = tip
        super().__init__(capabilities, unattended_tools=allowlist)
        self.host._daemon = EngineDaemonClient()

    def _write(self, path, value):
        value = dict(value)
        if "accepted_transaction_tip" in value:
            value["accepted_transaction_tip"] = self.tip
        ScopedHostFixture._write(path, value)

    def arguments(self):
        return {
            "path": str(self.root),
            "symbol": "unbound-symbol",
            "field": "value",
            "value": "updated value",
            "expected_model_revision": "revision-authority",
            "expected_tip_transaction": "none" if self.tip is None else self.tip,
            "context_id": "context-authority",
            "accepted_transaction_tip": self.tip,
        }


class SymbolFieldAuthority(unittest.TestCase):
    def setUp(self):
        self.fixtures = []

    def tearDown(self):
        for fixture in self.fixtures:
            fixture.close()

    def fixture(self, capabilities=None, **kwargs):
        if capabilities is None:
            capabilities = ["inspect", "propose", "apply-approved", "unattended"]
            kwargs.setdefault("allowlist", [TOOL])
        fixture = WriterFixture(capabilities, **kwargs)
        self.fixtures.append(fixture)
        return fixture

    def tools(self, fixture):
        response = fixture.host.handle_message(
            {"jsonrpc": "2.0", "id": 2, "method": "tools/list"}
        )
        return {tool["name"]: tool for tool in response["result"]["tools"]}

    def call(self, fixture, arguments):
        with patch(
            "server_runtime.subprocess.run",
            return_value=subprocess.CompletedProcess([], 0, stdout="{}", stderr=""),
        ) as run:
            result = fixture.call(TOOL, arguments)
        return result, run

    def test_inspect_propose_and_reviewed_apply_cannot_dispatch_direct_writer(self):
        for capabilities in [
            ["inspect"],
            ["inspect", "propose"],
            ["inspect", "propose", "apply-approved"],
        ]:
            with self.subTest(capabilities=capabilities):
                fixture = self.fixture(capabilities)
                self.assertNotIn(TOOL, self.tools(fixture))
                # The old path dispatched even without these broker-only fields.
                arguments = fixture.arguments()
                del arguments["context_id"]
                del arguments["accepted_transaction_tip"]
                result, run = self.call(fixture, arguments)
                self.assertFalse(result["ok"])
                self.assertEqual(result["error"]["code"], "capability_denied")
                run.assert_not_called()

    def test_unattended_authority_requires_this_exact_tool_allowlist(self):
        fixture = self.fixture(
            ["inspect", "propose", "apply-approved", "unattended"],
            allowlist=["datum.proposal.accept_apply"],
        )
        self.assertNotIn(TOOL, self.tools(fixture))
        result, run = self.call(fixture, fixture.arguments())
        self.assertFalse(result["ok"])
        self.assertEqual(result["error"]["code"], "capability_denied")
        run.assert_not_called()

    def test_authorized_uuid_and_empty_journal_guards_reach_native_adapter(self):
        for tip in [TIP, None]:
            with self.subTest(tip=tip):
                fixture = self.fixture(tip=tip)
                tools = self.tools(fixture)
                self.assertEqual(tools[TOOL]["x_datum_required_capability"], "unattended")
                native_schema = tools[TOOL]["inputSchema"]["properties"]
                self.assertEqual(native_schema["expected_model_revision"]["minLength"], 1)
                arguments = fixture.arguments()
                result, run = self.call(fixture, arguments)
                self.assertTrue(result["ok"], result)
                run.assert_called_once()
                argv = run.call_args.args[0]
                self.assertIn("set-symbol-field", argv)
                self.assertEqual(
                    argv[argv.index("--expected-model-revision") + 1],
                    arguments["expected_model_revision"],
                )
                self.assertEqual(
                    argv[argv.index("--expected-tip-transaction") + 1],
                    arguments["expected_tip_transaction"],
                )
                self.assertNotIn("--context-id", argv)
                self.assertNotIn("--accepted-transaction-tip", argv)

    def test_native_and_broker_tips_must_agree_before_writer_dispatch(self):
        for broker, native in [
            (TIP, OTHER_TIP), (TIP, "none"), (None, TIP),
            (TIP, "not-a-uuid"), (TIP, ""), (TIP, None),
        ]:
            with self.subTest(broker=broker, native=native):
                fixture = self.fixture(tip=broker)
                arguments = fixture.arguments() | {"expected_tip_transaction": native}
                result, run = self.call(fixture, arguments)
                self.assertFalse(result["ok"])
                self.assertEqual(result["error"]["code"], "stale_context")
                run.assert_not_called()
        fixture = self.fixture()
        arguments = fixture.arguments()
        del arguments["expected_tip_transaction"]
        result, run = self.call(fixture, arguments)
        self.assertFalse(result["ok"])
        run.assert_not_called()

    def test_uuid_identity_comparison_preserves_native_spelling(self):
        tip = "abcdefab-cdef-abcd-efab-cdefabcdefab"
        fixture = self.fixture(tip=tip)
        result, run = self.call(
            fixture, fixture.arguments() | {"expected_tip_transaction": tip.upper()}
        )
        self.assertTrue(result["ok"], result)
        self.assertIn(tip.upper(), run.call_args.args[0])

    def test_missing_or_stale_broker_context_refuses_before_writer(self):
        fixture = self.fixture()
        for field in ["context_id", "expected_model_revision", "accepted_transaction_tip"]:
            with self.subTest(missing=field):
                arguments = fixture.arguments()
                del arguments[field]
                result, run = self.call(fixture, arguments)
                self.assertFalse(result["ok"])
                run.assert_not_called()
        for change in [
            {"context_id": "different-context"},
            {"expected_model_revision": "different-revision"},
            {"accepted_transaction_tip": OTHER_TIP},
        ]:
            with self.subTest(change=change):
                result, run = self.call(fixture, fixture.arguments() | change)
                self.assertFalse(result["ok"])
                run.assert_not_called()
        live = json.loads(fixture.live_path.read_text())
        fixture._write(fixture.live_path, live | {"model_revision": "advanced-revision"})
        result, run = self.call(fixture, fixture.arguments())
        self.assertFalse(result["ok"])
        run.assert_not_called()


if __name__ == "__main__":
    unittest.main()
