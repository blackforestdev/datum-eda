"""Exercise owner activation only in disposable synthetic Git repositories."""

import json
import fcntl
import os
import signal
from pathlib import Path
import subprocess
import sys
import tempfile
import time
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

    def cli_command(self, *, output=None, script=None):
        return [sys.executable, "-I", "-S", "-B",
            str(script or self.locations["scripts"] / "workflow_delivery_preflight_cli.py"), "--activate",
            "--root", str(self.live), "--request", str(self.request_path), "--request-sha256", self.request_digest,
            "--owner-response", str(self.response), "--owner-response-sha256", sha256(self.response.read_bytes()),
            "--output", str(output or self.output)]

    def run_cli(self, *, output=None, script=None):
        return subprocess.run(self.cli_command(output=output, script=script), capture_output=True, timeout=60,
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

    def signal_during_git(self, number):
        self.prepare()
        ready, release = (Path(self.temp.name) / name for name in ("git-child-ready", "release-child"))
        hook = self.locations["common"] / "hooks/post-merge"
        hook.write_text(f"#!{sys.executable}\n" +
            "import os, time\nfrom pathlib import Path\n" +
            f"ready = Path({str(ready)!r})\nrelease = Path({str(release)!r})\n" +
            "temporary = ready.with_suffix('.tmp')\ntemporary.write_text(str(os.getpid()))\ntemporary.replace(ready)\ndeadline = time.monotonic() + 20\n" +
            "while not release.exists() and time.monotonic() < deadline:\n    time.sleep(0.02)\n" +
            "raise SystemExit(0 if release.exists() else 2)\n")
        hook.chmod(0o755)
        process = subprocess.Popen(self.cli_command(), stdout=subprocess.PIPE, stderr=subprocess.PIPE,
            env={"PATH": os.defpath, "GIT_CONFIG_GLOBAL": os.devnull, "GIT_CONFIG_NOSYSTEM": "1"})
        try:
            deadline = time.monotonic() + 15
            while not ready.exists() and process.poll() is None and time.monotonic() < deadline:
                time.sleep(0.02)
            self.assertTrue(ready.exists(), "actual post-merge child did not become live")
            self.assertIsNone(process.poll())
            os.kill(int(ready.read_text()), 0)
            process.send_signal(number)
            progress = self.output.with_suffix(".events.jsonl")
            deadline = time.monotonic() + 5
            events = []
            while time.monotonic() < deadline:
                events = [json.loads(line) for line in progress.read_bytes().splitlines(keepends=True)
                          if line.endswith(b"\n")]
                if any(e["stage"] == "interrupt-waiting-for-child" for e in events):
                    break
                time.sleep(0.02)
            waiting = next(e for e in events if e["stage"] == "interrupt-waiting-for-child")
            self.assertEqual(number, waiting["signal"])
            self.assertIsNone(process.poll())
            os.kill(waiting["pid"], 0)  # Confirm the exact Git child is still alive.
            with (self.locations["store"] / "activation.lock").open("rb") as lock:
                with self.assertRaises(BlockingIOError):
                    fcntl.flock(lock.fileno(), fcntl.LOCK_EX | fcntl.LOCK_NB)
            self.assertFalse(any(e["stage"] == "config-starting" for e in events))
            release.write_text("release the owned fixture child\n")
            stdout, stderr = process.communicate(timeout=20)
            self.assertEqual(2, process.returncode, stdout + stderr)
            report = json.loads(stdout)
            self.assertFalse(report["activation_asserted"])
            self.assertEqual("partial", report["state"])
            with self.assertRaises(ProcessLookupError):
                os.kill(waiting["pid"], 0)
            state = protected_state(self.live, ["src"])
            self.assertEqual(self.fixture.candidate, state["head"])
            self.assertEqual(self.request["prior_local_trust"], state["local_trust"])
            events = [json.loads(line) for line in progress.read_bytes().splitlines()]
            self.assertEqual("child-finished", events[-1]["stage"])
        finally:
            release.write_text("fixture cleanup release\n")
            if process.poll() is None:
                process.terminate()
                process.communicate(timeout=25)

    def test_sigterm_during_live_git_waits_with_lock_then_reports_partial(self):
        self.signal_during_git(signal.SIGTERM)

    def test_sigint_during_live_git_waits_with_lock_then_reports_partial(self):
        self.signal_during_git(signal.SIGINT)

    def test_child_timeout_reaps_only_its_owned_process(self):
        from workflow_delivery_activation import ActivationSignals
        events = []
        with ActivationSignals(events.append) as signals:
            with self.assertRaises(subprocess.TimeoutExpired):
                signals.run(self.f.root, [sys.executable, "-I", "-S", "-B", "-c",
                                         "import time; time.sleep(10)"], timeout=0.05)
        self.assertEqual("child-stopped-after-error", events[-1]["stage"])
        self.assertEqual(-signal.SIGKILL, events[-1]["exit_code"])
        with self.assertRaises(ProcessLookupError):
            os.kill(events[-1]["pid"], 0)

    def test_child_retains_lock_after_abrupt_parent_death(self):
        lock_path = Path(self.temp.name) / "owned.lock"
        ready = Path(self.temp.name) / "started.json"
        program = (
            "import sys, os, fcntl, json\nfrom pathlib import Path\n"
            "sys.path.insert(0, sys.argv[1])\n"
            "from workflow_delivery_activation import ActivationSignals\n"
            "fd = os.open(sys.argv[2], os.O_RDWR | os.O_CREAT, 0o600)\n"
            "fcntl.flock(fd, fcntl.LOCK_EX)\n"
            "def record(event):\n"
            "    if event['stage'] == 'child-started':\n"
            "        target = Path(sys.argv[3]); temporary = target.with_suffix('.tmp')\n"
            "        temporary.write_text(json.dumps(event)); temporary.replace(target)\n"
            "with ActivationSignals(record) as signals:\n"
            "    signals.lock_fd = fd\n"
            "    signals.run(sys.argv[4], [sys.executable, '-I', '-S', '-B', '-c', 'import time; time.sleep(2)'])\n")
        parent = subprocess.Popen([sys.executable, "-I", "-S", "-B", "-c", program,
            str(Path(__file__).parent), str(lock_path), str(ready), str(self.f.root)],
            stdout=subprocess.PIPE, stderr=subprocess.PIPE)
        try:
            deadline = time.monotonic() + 5
            while not ready.exists() and parent.poll() is None and time.monotonic() < deadline:
                time.sleep(0.02)
            self.assertTrue(ready.exists())
            child_pid = json.loads(ready.read_bytes())["pid"]
            os.kill(child_pid, 0)
            parent.kill()  # Exact owned fixture PID; SIGKILL cannot be handled.
            parent.communicate(timeout=5)
            self.assertEqual(-signal.SIGKILL, parent.returncode)
            with lock_path.open("rb") as lock:
                with self.assertRaises(BlockingIOError):
                    fcntl.flock(lock.fileno(), fcntl.LOCK_EX | fcntl.LOCK_NB)
                deadline = time.monotonic() + 5
                while True:
                    try:
                        fcntl.flock(lock.fileno(), fcntl.LOCK_EX | fcntl.LOCK_NB)
                        break
                    except BlockingIOError:
                        self.assertLess(time.monotonic(), deadline, "owned child did not release its lock")
                        time.sleep(0.02)
        finally:
            if parent.poll() is None:
                parent.terminate()
                parent.communicate(timeout=10)

    def test_child_start_log_failure_still_stops_and_reaps_child(self):
        from workflow_delivery_activation import ActivationSignals
        events = []
        def failed_log(event):
            events.append(event)
            if event["stage"] == "child-started":
                raise OSError("Synthetic progress log write failure")
        with ActivationSignals(failed_log) as signals:
            with self.assertRaisesRegex(OSError, "progress log write failure"):
                signals.run(self.f.root, [sys.executable, "-I", "-S", "-B", "-c",
                                         "import time; time.sleep(10)"])
        self.assertEqual("child-stopped-after-error", events[-1]["stage"])
        with self.assertRaises(ProcessLookupError):
            os.kill(events[-1]["pid"], 0)


if __name__ == "__main__":
    unittest.main()
