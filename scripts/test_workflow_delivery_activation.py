"""Exercise owner activation only in disposable synthetic Git repositories."""

import json
import fcntl
import os
from pathlib import Path
import subprocess
import sys
import tempfile
import unittest

import test_workflow_delivery_activation_preflight as fixtures
from workflow_delivery_activation import COORDINATION
from workflow_delivery_capture_state import protected_state
from workflow_delivery_io import canonical_json, sha256
from workflow_delivery_support_bundle import prepare_bundle
from workflow_delivery_support_store import support_locations, proposed_local_trust


class ActivationCommandTest(unittest.TestCase):
    def setUp(self):
        self.fixture = fixtures.PromotionCandidateTest()
        self.fixture.setUp()
        self.addCleanup(self.fixture.doCleanups)
        self.f = self.fixture.f
        self.temp = tempfile.TemporaryDirectory()
        self.addCleanup(self.temp.cleanup)

    def prepare(self):
        self.live = Path(self.temp.name) / "main"
        self.f.git("branch", "-m", "fixture-candidate")
        self.f.git("branch", "main", self.fixture.base)
        self.f.git("worktree", "add", "--quiet", str(self.live), "main")
        candidate = self.fixture.candidate
        prepare_bundle(self.live, candidate)
        self.locations = support_locations(self.live, candidate)
        before = protected_state(self.live, ["src"])
        self.request = {"schema_version": 1, "kind": "datum.workflow-delivery.activation-request",
            "base": self.fixture.base, "candidate": candidate, "authority": candidate,
            "input_roots": ["src"], "prior_local_trust": before["local_trust"],
            "publication_review": self.fixture.review, "environment_path": "requested-environment.json",
            "coordination": COORDINATION,
            "proposed_local_trust": proposed_local_trust(self.live, authority=candidate,
                base=self.fixture.base, environment_path="requested-environment.json")}
        self.request_path = Path(self.temp.name) / "request.json"
        self.response = self.locations["store"] / "owner-responses/fixture.json"
        self.response.parent.mkdir(parents=True)
        self.output = self.locations["logs"] / "activation.json"
        self.save_inputs()

    def save_inputs(self):
        self.request_path.write_bytes(canonical_json(self.request))
        self.request_digest = sha256(self.request_path.read_bytes())
        self.response.write_bytes(canonical_json({"schema_version": 1,
            "kind": "datum.workflow-delivery.activation-response",
            "response": "WORKFLOW-DELIVERY-IMPLEMENTATION: approve ACTIVATE — " + self.request_digest,
            "source": "Synthetic fixture input, not real owner approval", "recorded_at": "2026-09-08"}))

    def run_cli(self, *, output=None, script=None):
        return subprocess.run([sys.executable, "-I", "-S", "-B",
            str(script or self.locations["scripts"] / "workflow_delivery_preflight_cli.py"), "--activate",
            "--root", str(self.live), "--request", str(self.request_path), "--request-sha256", self.request_digest,
            "--owner-response", str(self.response), "--owner-response-sha256", sha256(self.response.read_bytes()),
            "--output", str(output or self.output)], capture_output=True, timeout=60,
            env={"PATH": os.defpath, "GIT_CONFIG_GLOBAL": os.devnull, "GIT_CONFIG_NOSYSTEM": "1"})

    def test_success_publishes_exact_candidate_and_runs_installed_gates(self):
        self.prepare()
        result = self.run_cli()
        self.assertEqual(0, result.returncode, result.stdout + result.stderr)
        report = json.loads(result.stdout)
        self.assertTrue(report["activation_asserted"])
        state = protected_state(self.live, ["src"])
        self.assertEqual(self.fixture.candidate, state["head"])
        self.assertEqual(self.request["proposed_local_trust"], state["local_trust"])
        payload = json.loads(self.output.read_bytes())
        events = [json.loads(line) for line in Path(payload["progress_log"]).read_bytes().splitlines()]
        self.assertEqual("verified", events[-1]["stage"])
        self.assertEqual(2, sum(e["stage"] == "verification-result" for e in events))
        self.assertFalse(payload["activation"]["roadmap_acceptance_asserted"])
        # A completed invocation does not turn its stale request into a retry.
        result = self.run_cli(output=self.output.with_name("retry.json"))
        self.assertEqual(2, result.returncode, result.stdout + result.stderr)
        self.assertEqual(state, protected_state(self.live, ["src"]))

    def test_candidate_checkout_cannot_run_mutating_activation(self):
        self.prepare()
        before = protected_state(self.live, ["src"])
        result = self.run_cli(script=Path(__file__).with_name("workflow_delivery_preflight_cli.py"))
        self.assertEqual(2, result.returncode, result.stdout + result.stderr)
        self.assertIn(b"exact prepared Git-common runtime", result.stdout)
        self.assertEqual(before, protected_state(self.live, ["src"]))

    def test_missing_coordination_and_dirty_source_refuse_before_publication(self):
        self.prepare()
        self.request["coordination"] = "not coordinated"
        self.save_inputs()
        before = protected_state(self.live, ["src"])
        result = self.run_cli()
        self.assertEqual(2, result.returncode, result.stdout + result.stderr)
        self.assertIn(b"writer-pause", result.stdout)
        self.assertEqual(before, protected_state(self.live, ["src"]))
        (self.live / "src/read.py").write_bytes(b"uncommitted fixture work\n")
        before = protected_state(self.live, ["src"])
        result = self.run_cli(output=self.output.with_name("dirty.json"))
        self.assertEqual(2, result.returncode, result.stdout + result.stderr)
        self.assertEqual(before, protected_state(self.live, ["src"]))

    def test_undisposed_review_finding_blocks_actual_publication(self):
        path = self.f.root / self.f.contract["review_path"]
        review = json.loads(path.read_bytes())
        review["findings"] = [{"issue_id": "dat-next", "severity": "blocking", "disposition_ref": None}]
        self.f.save(self.f.contract["review_path"], review)
        self.fixture.commit_candidate()
        self.prepare()
        before = protected_state(self.live, ["src"])
        result = self.run_cli()
        self.assertEqual(2, result.returncode, result.stdout + result.stderr)
        self.assertIn(b"undisposed defect dat-next", result.stdout)
        self.assertEqual(before, protected_state(self.live, ["src"]))

    def test_live_activation_lock_refuses_another_process_without_publication(self):
        self.prepare()
        before = protected_state(self.live, ["src"])
        with (self.locations["store"] / "activation.lock").open("wb") as lock:
            fcntl.flock(lock.fileno(), fcntl.LOCK_EX | fcntl.LOCK_NB)
            result = self.run_cli()
        self.assertEqual(2, result.returncode, result.stdout + result.stderr)
        self.assertIn(b"another activation holds", result.stdout)
        self.assertEqual("not_started", json.loads(result.stdout)["state"])
        self.assertEqual(before, protected_state(self.live, ["src"]))

    def test_worktree_config_override_is_not_silently_ignored(self):
        self.prepare()
        self.f.git("config", "extensions.worktreeConfig", "true")
        before = protected_state(self.live, ["src"])
        result = self.run_cli()
        self.assertEqual(2, result.returncode, result.stdout + result.stderr)
        self.assertIn(b"worktree-specific configuration", result.stdout)
        self.assertEqual(before, protected_state(self.live, ["src"]))

    def test_interruption_retains_partial_state_and_stale_cli_retry_refuses(self):
        from workflow_delivery_activation import activate
        from workflow_delivery_activation_state import inspect_activation_state
        self.prepare()
        events = []
        def interrupt_after_first_config(event):
            events.append(event)
            if event["stage"] == "config-written":
                raise KeyboardInterrupt("Synthetic interruption after an actual config write")
        with self.assertRaises(KeyboardInterrupt):
            activate(self.live, self.request, record=interrupt_after_first_config)
        self.assertEqual("config-written", events[-1]["stage"])
        state = inspect_activation_state(self.live, base=self.request["base"], candidate=self.request["candidate"],
            input_roots=["src"], prior_trust=self.request["prior_local_trust"], proposed_trust=self.request["proposed_local_trust"])
        self.assertEqual("partial", state["state"])
        before = protected_state(self.live, ["src"])
        result = self.run_cli()
        self.assertEqual(2, result.returncode, result.stdout + result.stderr)
        self.assertEqual("partial", json.loads(result.stdout)["state"])
        self.assertEqual(before, protected_state(self.live, ["src"]))


if __name__ == "__main__":
    unittest.main()
