"""Scoped journal authority through the real host and native CLI adapter.

Adapter controls intercept subprocesses. Native controls require an explicitly
configured DATUM_CLI_BIN and exercise cloned canonical two-commit history.
"""
import json
import os
import shutil
import tempfile
import uuid
from pathlib import Path
import subprocess
import unittest
from unittest.mock import patch

from server_runtime import EngineDaemonClient
from stdio_tool_host import StdioToolHost
from test_agent_capability import ScopedHostFixture


TOOLS = ("datum.journal.undo", "datum.journal.redo")
TIP = "10000000-0000-0000-0000-000000000001"
OTHER_TIP = "20000000-0000-0000-0000-000000000002"
CAPABILITIES = ["inspect", "propose", "apply-approved", "unattended"]


class JournalFixture(ScopedHostFixture):
    def __init__(self, capabilities, *, tip=TIP, allowlist=None):
        self.tip = tip
        if allowlist is None:
            allowlist = TOOLS if "unattended" in capabilities else ()
        super().__init__(capabilities, unattended_tools=list(allowlist))
        self.host._daemon = EngineDaemonClient()

    def _write(self, path, value):
        value = dict(value)
        if "accepted_transaction_tip" in value:
            value["accepted_transaction_tip"] = self.tip
        ScopedHostFixture._write(path, value)

    def arguments(self):
        return {"path": str(self.root), "context_id": "context-authority",
                "expected_model_revision": "revision-authority",
                "accepted_transaction_tip": self.tip}


class JournalAuthority(unittest.TestCase):
    def setUp(self):
        self.fixtures = []

    def tearDown(self):
        for fixture in self.fixtures:
            fixture.close()

    def fixture(self, capabilities=None, **kwargs):
        f = JournalFixture(CAPABILITIES if capabilities is None else capabilities, **kwargs)
        self.fixtures.append(f)
        return f

    def call(self, fixture, tool, arguments):
        with patch("server_runtime.subprocess.run", return_value=
                   subprocess.CompletedProcess([], 0, stdout="{}", stderr="")) as run:
            result = fixture.call(tool, arguments)
        return result, run

    def test_scoped_catalog_requires_context_but_manual_guards_stay_optional(self):
        fixture = self.fixture()
        response = fixture.host.handle_message({"jsonrpc": "2.0", "id": 2, "method": "tools/list"})
        scoped = {t["name"]: t for t in response["result"]["tools"]}
        from tools_catalog_data import TOOL_BY_NAME
        for tool in TOOLS:
            self.assertEqual(TOOL_BY_NAME[tool]["x_public_write_surface_class"],
                             "journaled_history_compensation")
            self.assertTrue({"context_id", "expected_model_revision", "accepted_transaction_tip"}
                            .issubset(scoped[tool]["inputSchema"]["required"]))
            self.assertEqual(TOOL_BY_NAME[tool]["inputSchema"]["required"], ["path"])

    def test_current_context_reaches_adapter_with_both_native_guards(self):
        for tool in TOOLS:
            for explicit in (False, True):
                with self.subTest(tool=tool, explicit=explicit):
                    fixture = self.fixture()
                    arguments = fixture.arguments()
                    if explicit:
                        arguments["expected_tip_transaction"] = TIP
                    before = dict(arguments)
                    result, run = self.call(fixture, tool, arguments)
                    self.assertTrue(result["ok"], result)
                    run.assert_called_once()
                    argv = run.call_args.args[0]
                    self.assertEqual(argv[-4:], ["--expected-model-revision", "revision-authority",
                                                "--expected-tip-transaction", TIP])
                    self.assertNotIn("--context-id", argv)
                    self.assertNotIn("--accepted-transaction-tip", argv)
                    self.assertEqual(arguments, before)

    def test_missing_stale_or_conflicting_context_refuses_before_dispatch(self):
        cases = ("context_id", "expected_model_revision", "accepted_transaction_tip",
                 "wrong_context", "wrong_revision", "advanced_revision", "advanced_tip")
        for tool in TOOLS:
            for case in cases:
                with self.subTest(tool=tool, case=case):
                    fixture = self.fixture()
                    arguments = fixture.arguments()
                    if case in arguments:
                        del arguments[case]
                    elif case == "wrong_context":
                        arguments["context_id"] = "other-context"
                    elif case == "wrong_revision":
                        arguments["expected_model_revision"] = "other-revision"
                    else:
                        live = json.loads(fixture.live_path.read_text())
                        live["model_revision" if case == "advanced_revision" else
                             "accepted_transaction_tip"] = (
                                 "newer-revision" if case == "advanced_revision" else OTHER_TIP)
                        fixture.live_path.write_text(json.dumps(live))
                    result, run = self.call(fixture, tool, arguments)
                    self.assertFalse(result["ok"], result)
                    self.assertIn(result["error"]["code"], ("context_fence_required", "stale_context"))
                    run.assert_not_called()

    def test_native_tip_conflicts_cannot_override_the_broker_pin(self):
        for tool in TOOLS:
            for tip in (None, "none", OTHER_TIP, "not-a-uuid"):
                with self.subTest(tool=tool, tip=tip):
                    fixture = self.fixture()
                    arguments = fixture.arguments() | {"expected_tip_transaction": tip}
                    result, run = self.call(fixture, tool, arguments)
                    self.assertFalse(result["ok"], result)
                    self.assertEqual(result["error"]["code"], "stale_context")
                    run.assert_not_called()

    def test_empty_or_invalid_pinned_history_never_becomes_unguarded_dispatch(self):
        for tool in TOOLS:
            for tip in (None, "", "not-a-uuid"):
                with self.subTest(tool=tool, tip=tip):
                    fixture = self.fixture(tip=tip)
                    result, run = self.call(fixture, tool, fixture.arguments())
                    self.assertFalse(result["ok"], result)
                    run.assert_not_called()

    def test_inspect_and_missing_exact_unattended_allowlist_stay_denied(self):
        for tool in TOOLS:
            for capabilities, allowlist in ((["inspect"], ()),
                    (["inspect", "propose"], ()),
                    (["inspect", "propose", "apply-approved"], ()),
                    (CAPABILITIES, ("datum.schematic.set_symbol_field",))):
                with self.subTest(tool=tool, capabilities=capabilities, allowlist=allowlist):
                    fixture = self.fixture(capabilities, allowlist=allowlist)
                    result, run = self.call(fixture, tool, fixture.arguments())
                    self.assertFalse(result["ok"], result)
                    self.assertEqual(result["error"]["code"], "capability_denied")
                    run.assert_not_called()

    def test_unscoped_manual_adapter_keeps_optional_guard_semantics(self):
        for tool in TOOLS:
            for guarded in (False, True):
                with self.subTest(tool=tool, guarded=guarded):
                    fixture = self.fixture()
                    fixture.host = StdioToolHost(EngineDaemonClient())
                    arguments = {"path": str(fixture.root)}
                    if guarded:
                        arguments |= {"expected_model_revision": "manual-revision",
                                      "expected_tip_transaction": OTHER_TIP}
                    result, run = self.call(fixture, tool, arguments)
                    self.assertTrue(result["ok"], result)
                    argv = run.call_args.args[0]
                    if guarded:
                        self.assertEqual(argv[-4:], ["--expected-model-revision", "manual-revision",
                                                    "--expected-tip-transaction", OTHER_TIP])
                    else:
                        self.assertNotIn("--expected-model-revision", argv)
                        self.assertNotIn("--expected-tip-transaction", argv)


@unittest.skipUnless(os.environ.get("DATUM_CLI_BIN"), "requires an explicit prebuilt native CLI")
class NativeJournalAuthority(unittest.TestCase):
    """Real host, adapter and native guarded history; never falls back to Cargo."""

    @classmethod
    def setUpClass(cls):
        from test_native_write_parity import run_cli_json
        cls.cli = staticmethod(run_cli_json)
        cls.temp = tempfile.TemporaryDirectory(prefix="datum-journal-native-")
        cls.seed = Path(cls.temp.name) / "seed"
        cls.seed.mkdir()
        cls.cli(cls.seed, "project", "new", str(cls.seed), "--name", "Journal authority")
        cls.text = cls.cli(cls.seed, "project", "place-board-text", str(cls.seed),
                           "--text", "FIRST", "--x-nm", "1000", "--y-nm", "2000",
                           "--layer", "1")["text_uuid"]
        cls.cli(cls.seed, "project", "edit-board-text", str(cls.seed),
                "--text", cls.text, "--value", "SECOND")
        journal = cls.cli(cls.seed, "journal", "list", str(cls.seed))
        if journal["count"] != 2:
            raise AssertionError(journal)

    @classmethod
    def tearDownClass(cls):
        cls.temp.cleanup()

    def setUp(self):
        self.fixtures = []

    def tearDown(self):
        for fixture in self.fixtures:
            fixture.close()

    def fixture(self, capabilities=None):
        f = JournalFixture(CAPABILITIES if capabilities is None else capabilities)
        self.fixtures.append(f)
        shutil.copytree(self.seed, f.root, dirs_exist_ok=True)
        self.repin(f)
        return f

    def journal(self, f):
        return self.cli(f.root, "journal", "list", str(f.root))

    def repin(self, f):
        from discovery_scope import load_discovery_scope
        j = self.journal(f)
        f.tip = j["transactions"][-1]["transaction_id"]
        for path in (f.live_path, f.pinned_path, f.discovery_path):
            document = json.loads(path.read_text())
            document.update(model_revision=j["model_revision"], accepted_transaction_tip=f.tip)
            path.write_text(json.dumps(document))
        environment = {"DATUM_AGENT_CREDENTIAL_FILE": str(f.root / ".agent-credential.json"),
                       "DATUM_AGENT_LAUNCH_ID": "agent-launch-test", "DATUM_AGENT_ADAPTER_ID": "test-agent"}
        with patch.dict(os.environ, environment, clear=False):
            f.scope = load_discovery_scope(f.discovery_path)
            f.host = StdioToolHost(EngineDaemonClient(), f.scope)
        f.native_arguments = {"path": str(f.root), "context_id": "context-authority",
                              "expected_model_revision": j["model_revision"],
                              "accepted_transaction_tip": f.tip}
        return j

    def snapshot(self, f):
        # Refusals may append audit events, but must preserve all native files.
        excluded = {"live.json", "pinned.json", "discovery.json", "agent-authority.json",
                    ".agent-credential.json"}
        return {str(p.relative_to(f.root)): p.read_bytes() for p in f.root.rglob("*")
                if p.is_file() and str(p.relative_to(f.root)) not in excluded
                and not p.is_relative_to(f.root / ".datum" / "tool-sessions")}

    def text_value(self, f):
        return self.cli(f.root, "project", "query", str(f.root), "board-texts")[0]["text"]

    def advance(self, f, *, metadata=False):
        if not metadata:
            self.cli(f.root, "project", "edit-board-text", str(f.root),
                     "--text", self.text, "--value", "THIRD")
        else:
            j = self.journal(f)
            transaction = self.cli(f.root, "journal", "show", str(f.root),
                                   "--transaction", j["transactions"][-1]["transaction_id"])["transaction"]
            batch = {"batch_id": str(uuid.uuid4()), "expected_model_revision": j["model_revision"],
                     "provenance": {"actor": "journal-authority-test", "source": "test",
                                    "reason": "Interleaved proposal metadata"},
                     "operations": transaction["operations"]}
            path = Path(self.temp.name) / "proposal-batch.json"
            path.write_text(json.dumps(batch))
            self.cli(f.root, "proposal", "create", str(f.root), "--batch", str(path),
                     "--rationale", "Tip-only history guard control")

    def test_fresh_scoped_undo_redo_targets_the_pinned_transaction(self):
        f = self.fixture()
        before = self.journal(f)
        result = f.call(TOOLS[0], f.native_arguments)
        self.assertTrue(result["ok"], result)
        self.assertEqual(result["transaction"]["undo_of"], before["transactions"][-1]["transaction_id"])
        self.assertTrue(result["guard"]["checked"])
        self.assertEqual(result["guard"]["expected_tip_transaction"], f.tip)
        self.assertEqual(result["guard"]["expected_model_revision"], before["model_revision"])
        self.assertEqual(self.text_value(f), "FIRST")
        stale_bytes = self.snapshot(f)
        stale = f.call(TOOLS[1], f.native_arguments)
        self.assertFalse(stale["ok"], stale)
        self.assertEqual(self.snapshot(f), stale_bytes)
        self.repin(f)
        redone = f.call(TOOLS[1], f.native_arguments)
        self.assertTrue(redone["ok"], redone)
        self.assertEqual(redone["transaction"]["redo_of"], result["transaction"]["transaction_id"])
        self.assertEqual(self.text_value(f), "SECOND")
        self.assertEqual(self.journal(f)["count"], before["count"] + 2)

    def test_real_missing_stale_conflicting_and_inspect_refusals_preserve_native_bytes(self):
        for tool in TOOLS:
            for case in ("missing", "stale", "conflict", "inspect"):
                with self.subTest(tool=tool, case=case):
                    f = self.fixture(["inspect"] if case == "inspect" else None)
                    args = dict(f.native_arguments)
                    if case == "missing":
                        del args["accepted_transaction_tip"]
                    elif case == "stale":
                        self.advance(f)
                        # Simulate the real live context refresh after the human edit.
                        live = json.loads(f.live_path.read_text())
                        j = self.journal(f)
                        live.update(model_revision=j["model_revision"],
                                    accepted_transaction_tip=j["transactions"][-1]["transaction_id"])
                        f.live_path.write_text(json.dumps(live))
                    elif case == "conflict":
                        args["expected_tip_transaction"] = OTHER_TIP
                    before = self.snapshot(f)
                    with patch("server_runtime.subprocess.run", wraps=subprocess.run) as run:
                        result = f.call(tool, args)
                    self.assertFalse(result["ok"], result)
                    run.assert_not_called()
                    self.assertEqual(self.snapshot(f), before)

    def test_native_guards_refuse_real_preflight_dispatch_races(self):
        from stdio_tool_host import dispatch_tool_call
        for tool in TOOLS:
            for metadata in (False, True):
                with self.subTest(tool=tool, metadata=metadata):
                    f = self.fixture()
                    if tool == TOOLS[1]:
                        self.assertTrue(f.call(TOOLS[0], f.native_arguments)["ok"])
                        self.repin(f)
                    before = self.journal(f)
                    interleaved = []
                    def dispatch(daemon, name, arguments):
                        self.advance(f, metadata=metadata)
                        advanced = self.journal(f)
                        self.assertNotEqual(advanced["transactions"][-1]["transaction_id"], f.tip)
                        if metadata:
                            self.assertEqual(advanced["model_revision"], before["model_revision"])
                        self.assertEqual(arguments["expected_model_revision"], before["model_revision"])
                        self.assertEqual(arguments["expected_tip_transaction"], f.tip)
                        interleaved.append(self.snapshot(f))
                        return dispatch_tool_call(daemon, name, arguments)
                    with patch("stdio_tool_host.dispatch_tool_call", side_effect=dispatch) as run:
                        result = f.call(tool, f.native_arguments)
                    run.assert_called_once()
                    self.assertFalse(result["ok"], result)
                    self.assertIn("expected tip transaction" if metadata else "expected model revision",
                                  result["error"]["message"])
                    self.assertEqual(len(interleaved), 1)
                    self.assertEqual(self.snapshot(f), interleaved[0])

    def test_real_unscoped_manual_history_keeps_unguarded_semantics(self):
        f = self.fixture()
        f.host = StdioToolHost(EngineDaemonClient())
        for tool, value in zip(TOOLS, ("FIRST", "SECOND")):
            result = f.call(tool, {"path": str(f.root)})
            self.assertTrue(result["ok"], result)
            self.assertFalse(result["guard"]["checked"])
            self.assertEqual(self.text_value(f), value)


if __name__ == "__main__":
    unittest.main()
