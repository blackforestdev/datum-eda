"""Run the inspection script; publication and live trust are never changed."""

import json
import os
from pathlib import Path
import subprocess
import sys
import tempfile
import unittest

from workflow_delivery_capture_state import protected_state
from workflow_delivery_io import canonical_json, sha256
from workflow_delivery_test_support import Fixture
from workflow_delivery_trust_test_support import accepted_fixture
from workflow_delivery_support_bundle import prepare_bundle
from workflow_delivery_support_store import support_locations, proposed_local_trust
from workflow_delivery_publication_delta import publication_delta


class PreflightCliTest(unittest.TestCase):
    def setUp(self):
        self.f = Fixture()
        self.addCleanup(self.f.close)
        self.f.git("branch", "-M", "main")
        self.temp = tempfile.TemporaryDirectory()
        self.addCleanup(self.temp.cleanup)
        self.store = Path(self.temp.name)
        _, _, _, self.f.head = accepted_fixture(self.f)
        prepare_bundle(self.f.root, self.f.head)
        tree = self.f.git("rev-parse", "HEAD^{tree}").decode().strip()
        candidate = self.f.git("commit-tree", tree, "-p", self.f.head, "-m", "synthetic candidate").decode().strip()
        self.request = {"schema_version": 1, "kind": "datum.workflow-delivery.preflight-request",
                        "base": self.f.head, "candidate": candidate, "authority": self.f.head, "input_roots": ["src"],
                        "prior_local_trust": protected_state(self.f.root, ["src"])["local_trust"]}
        delta = publication_delta(self.f.root, base=self.f.head, candidate=candidate)
        self.request["publication_review"] = {"delta_sha256": sha256(canonical_json(delta)),
                                              "paths": delta["touched_paths"]}
        self.request["environment_path"] = "evidence/environment.json"
        self.request["proposed_local_trust"] = proposed_local_trust(self.f.root,
            authority=self.f.head, base=self.f.head, environment_path=self.request["environment_path"])
        self.path = self.store / "request.json"
        self.path.write_bytes(canonical_json(self.request))
        self.output = self.f.root / ".git/datum-wdq/logs/preflight.json"
        self.script = Path(__file__).with_name("workflow_delivery_preflight_cli.py")
        self.env = {"PATH": os.defpath, "HOME": str(self.store), "XDG_CONFIG_HOME": str(self.store),
                    "GIT_CONFIG_NOSYSTEM": "1", "GIT_CONFIG_GLOBAL": os.devnull}

    def run_cli(self, *, output=None, digest=None, isolated=True, extra=(), mode="--inspect"):
        return subprocess.run([sys.executable, *(["-I", "-S", "-B"] if isolated else []),
            str(self.script), mode, "--root", str(self.f.root), "--request", str(self.path),
            "--request-sha256", digest or sha256(self.path.read_bytes()),
            "--output", str(output or self.output), *extra], cwd=self.store,
            env=self.env, capture_output=True, timeout=30)

    def owner_response(self):
        response = self.f.root / ".git/datum-wdq/owner-responses/fixture.json"
        response.parent.mkdir(parents=True)
        raw = canonical_json({"schema_version": 1, "kind": "datum.workflow-delivery.activation-response",
            "response": "WORKFLOW-DELIVERY-IMPLEMENTATION: approve ACTIVATE — " + sha256(self.path.read_bytes()),
            "source": "Synthetic test input, not owner approval", "recorded_at": "2026-09-08"})
        response.write_bytes(raw)
        return response, ["--owner-response", str(response), "--owner-response-sha256", sha256(raw)]

    def test_separate_response_is_compared_without_claiming_owner_identity(self):
        response, arguments = self.owner_response()
        raw = response.read_bytes()
        before = protected_state(self.f.root, ["src"])
        result = self.run_cli(extra=arguments)
        self.assertEqual(0, result.returncode, result.stdout + result.stderr)
        payload = json.loads(self.output.read_bytes())
        self.assertTrue(payload["owner_response"]["response_matches_request"])
        self.assertFalse(payload["owner_response"]["owner_identity_verified"])
        self.assertFalse(payload["publication_authorized"])
        self.assertEqual(raw, response.read_bytes())
        self.assertEqual(before, protected_state(self.f.root, ["src"]))

    def test_response_for_another_request_refuses_without_publication(self):
        _, arguments = self.owner_response()
        # JSON whitespace changes are still distinct externally hashed request bytes.
        self.path.write_bytes(self.path.read_bytes() + b"\n")
        before = protected_state(self.f.root, ["src"])
        result = self.run_cli(extra=arguments)
        self.assertEqual(2, result.returncode, result.stdout + result.stderr)
        self.assertIn(b"exact activation request", result.stdout)
        self.assertFalse(json.loads(self.output.read_bytes())["publication_authorized"])
        self.assertEqual(before, protected_state(self.f.root, ["src"]))

    def test_response_cannot_be_taken_from_candidate_or_redirected_storage(self):
        response, arguments = self.owner_response()
        candidate_file = self.f.root / "candidate-response.json"
        candidate_file.write_bytes(response.read_bytes())
        redirected = response.with_name("redirected.json")
        redirected.symlink_to(response)
        for index, path in enumerate((candidate_file, redirected)):
            with self.subTest(path=path):
                result = self.run_cli(extra=[arguments[0], str(path), *arguments[2:]],
                                      output=self.output.with_name(f"response-path-{index}.json"))
                self.assertEqual(2, result.returncode, result.stdout + result.stderr)
                self.assertIn(b"separate nonredirected JSON", result.stdout)

    def test_response_path_without_external_digest_is_an_invocation_failure(self):
        response, _ = self.owner_response()
        result = self.run_cli(extra=["--owner-response", str(response)])
        self.assertEqual(2, result.returncode, result.stdout + result.stderr)
        self.assertIn(b"required together", result.stdout)
        self.assertFalse(self.output.exists())

    def test_promotion_inspection_requires_separate_response_and_exact_mode(self):
        result = self.run_cli(mode="--inspect-promotion")
        self.assertEqual(2, result.returncode, result.stdout + result.stderr)
        self.assertIn(b"separately selected exact owner response", result.stdout)
        _, arguments = self.owner_response()
        result = self.run_cli(mode="--inspect-promotion", extra=arguments)
        self.assertEqual(2, result.returncode, result.stdout + result.stderr)
        self.assertIn(b"closed preflight request", result.stdout)
        self.assertFalse(self.output.exists())

    def test_promotion_inspection_cannot_use_a_different_authority_candidate(self):
        self.request["kind"] = "datum.workflow-delivery.promotion-inspection-request"
        self.path.write_bytes(canonical_json(self.request))
        _, arguments = self.owner_response()
        before = protected_state(self.f.root, ["src"])
        result = self.run_cli(mode="--inspect-promotion", extra=arguments)
        self.assertEqual(2, result.returncode, result.stdout + result.stderr)
        self.assertIn(b"authority must be the exact complete candidate", result.stdout)
        payload = json.loads(self.output.read_bytes())
        self.assertFalse(payload["ok"])
        self.assertFalse(payload["publication_authorized"])
        self.assertEqual(before, protected_state(self.f.root, ["src"]))

    def test_success_retains_exact_log_and_leaves_protected_state_unchanged(self):
        before = protected_state(self.f.root, ["src"])
        result = self.run_cli()
        self.assertEqual(0, result.returncode, result.stdout + result.stderr)
        report = json.loads(result.stdout)
        self.assertTrue(report["ok"])
        self.assertEqual(sha256(self.output.read_bytes()), report["log_sha256"])
        payload = json.loads(self.output.read_bytes())
        self.assertEqual(self.request, payload["request"])
        self.assertFalse(payload["publication_authorized"])
        self.assertEqual(self.f.head, payload["support_bundle"]["authority"])
        self.assertEqual(before, protected_state(self.f.root, ["src"]))

    def test_partial_activation_state_is_visible_and_retained_without_repair(self):
        from copy import deepcopy
        self.request["kind"] = "datum.workflow-delivery.activation-state-request"
        del self.request["publication_review"]
        del self.request["environment_path"]
        proposed = deepcopy(self.request["prior_local_trust"])
        proposed["datum.workflowDeliveryAuthorityRef"] = [self.request["candidate"]]
        self.request["proposed_local_trust"] = proposed
        self.path.write_bytes(canonical_json(self.request))
        self.f.git("merge", "--ff-only", self.request["candidate"])
        before = protected_state(self.f.root, ["src"])
        result = self.run_cli(mode="--inspect-state")
        self.assertEqual(2, result.returncode, result.stdout + result.stderr)
        report = json.loads(result.stdout)
        self.assertEqual("partial", report["state"])
        self.assertTrue(report["observation_completed"])
        self.assertFalse(report["activation_asserted"])
        payload = json.loads(self.output.read_bytes())
        self.assertEqual("partial", payload["activation_state"]["state"])
        self.assertEqual(before, protected_state(self.f.root, ["src"]))

    def test_dirty_failure_is_visible_and_retained_without_repair(self):
        self.f.write("src/read.py", b"dirty input\n")
        before = protected_state(self.f.root, ["src"])
        result = self.run_cli()
        self.assertEqual(2, result.returncode)
        self.assertIn(b"dirty", result.stdout)
        self.assertFalse(json.loads(self.output.read_bytes())["ok"])
        self.assertEqual(before, protected_state(self.f.root, ["src"]))

    def test_unreviewed_history_is_logged_and_never_published(self):
        self.request["publication_review"]["delta_sha256"] = "0" * 64
        self.path.write_bytes(canonical_json(self.request))
        before = protected_state(self.f.root, ["src"])
        result = self.run_cli()
        self.assertEqual(2, result.returncode, result.stdout + result.stderr)
        self.assertIn(b"review digest differs", result.stdout)
        payload = json.loads(self.output.read_bytes())
        self.assertFalse(payload["ok"])
        self.assertFalse(payload["publication_authorized"])
        self.assertEqual(before, protected_state(self.f.root, ["src"]))

    def test_each_replacement_trust_value_is_bound_to_the_request(self):
        from copy import deepcopy
        original = deepcopy(self.request)
        before = protected_state(self.f.root, ["src"])
        for index, key in enumerate(original["proposed_local_trust"]):
            with self.subTest(key=key):
                request = deepcopy(original)
                request["proposed_local_trust"][key] = ["unreviewed-replacement"]
                self.path.write_bytes(canonical_json(request))
                output = self.output.with_name(f"replacement-{index}.json")
                result = self.run_cli(output=output)
                self.assertEqual(2, result.returncode, result.stdout + result.stderr)
                self.assertIn(b"proposed local trust differs", result.stdout)
                self.assertFalse(json.loads(output.read_bytes())["publication_authorized"])
                self.assertEqual(before, protected_state(self.f.root, ["src"]))

    def test_missing_environment_does_not_pass_from_a_consistent_config_map(self):
        self.request["environment_path"] = "missing-environment.json"
        self.request["proposed_local_trust"] = proposed_local_trust(self.f.root,
            authority=self.f.head, base=self.f.head, environment_path=self.request["environment_path"])
        self.path.write_bytes(canonical_json(self.request))
        before = protected_state(self.f.root, ["src"])
        result = self.run_cli()
        self.assertEqual(2, result.returncode, result.stdout + result.stderr)
        self.assertIn(b"missing-environment.json", result.stdout)
        self.assertFalse(json.loads(self.output.read_bytes())["ok"])
        self.assertEqual(before, protected_state(self.f.root, ["src"]))

    def test_candidate_only_environment_change_is_not_selected_authority(self):
        path = self.request["environment_path"]
        original = (self.f.root / path).read_bytes()
        value = json.loads(original)
        value["os"] = "candidate-only environment"
        self.f.save(path, value)
        self.f.stage()
        tree = self.f.git("write-tree").decode().strip()
        candidate = self.f.git("commit-tree", tree, "-p", self.f.head,
                               "-m", "synthetic changed environment").decode().strip()
        self.f.write(path, original)
        self.f.stage()
        self.request["candidate"] = candidate
        delta = publication_delta(self.f.root, base=self.f.head, candidate=candidate)
        self.request["publication_review"] = {"delta_sha256": sha256(canonical_json(delta)),
                                              "paths": delta["touched_paths"]}
        self.path.write_bytes(canonical_json(self.request))
        before = protected_state(self.f.root, ["src"])
        result = self.run_cli()
        self.assertEqual(2, result.returncode, result.stdout + result.stderr)
        self.assertIn(b"environment differs", result.stdout)
        self.assertFalse(json.loads(self.output.read_bytes())["ok"])
        self.assertEqual(before, protected_state(self.f.root, ["src"]))

    def test_detached_checkout_refusal_is_visible_retained_and_nonmutating(self):
        self.f.git("checkout", "--detach", "-q", self.f.head)
        before = protected_state(self.f.root, ["src"])
        result = self.run_cli()
        self.assertEqual(2, result.returncode, result.stdout + result.stderr)
        self.assertIn(b"attached main checkout", result.stdout)
        payload = json.loads(self.output.read_bytes())
        self.assertFalse(payload["ok"])
        self.assertFalse(payload["publication_authorized"])
        self.assertFalse(payload["activation_asserted"])
        self.assertEqual(before, protected_state(self.f.root, ["src"]))

    def test_existing_log_is_never_overwritten(self):
        self.assertEqual(0, self.run_cli().returncode)
        original = self.output.read_bytes()
        self.assertEqual(2, self.run_cli().returncode)
        self.assertEqual(original, self.output.read_bytes())

    def test_wrong_digest_external_output_and_nonisolated_python_refuse(self):
        for kwargs in ({"digest": "0" * 64}, {"output": self.store / "outside.json"}, {"isolated": False}):
            with self.subTest(kwargs=kwargs):
                result = self.run_cli(**kwargs)
                self.assertEqual(2, result.returncode)
                self.assertFalse(json.loads(result.stdout)["ok"])
                self.assertFalse(self.output.exists())

    def test_activation_cannot_be_combined_with_inspection(self):
        result = self.run_cli(extra=["--activate"])
        self.assertEqual(2, result.returncode)
        self.assertIn(b"not allowed with argument --inspect", result.stderr)
        self.assertFalse(self.output.exists())

    def test_missing_or_changed_proposed_runner_is_logged_without_repair(self):
        runner = support_locations(self.f.root, self.f.head)["runner"]
        original = runner.read_bytes()
        runner.chmod(0o644)
        runner.write_bytes(original + b"\n# altered proposed runner\n")
        runner.chmod(0o444)
        result = self.run_cli()
        self.assertEqual(2, result.returncode)
        self.assertIn(b"pinned Git authority", result.stdout)
        self.assertEqual(original + b"\n# altered proposed runner\n", runner.read_bytes())
        # This file is solely inside this freshly created fixture bundle.
        runner.unlink()
        result = self.run_cli(output=self.output.with_name("missing-runner.json"))
        self.assertEqual(2, result.returncode)
        self.assertIn(b"runtime module set differs", result.stdout)
        self.assertFalse(runner.exists())

    def test_missing_or_moving_authority_never_falls_back_to_candidate(self):
        for authority in ("HEAD", "0" * 40):
            self.request["authority"] = authority
            self.path.write_bytes(canonical_json(self.request))
            result = self.run_cli()
            self.assertEqual(2, result.returncode)
            self.assertFalse(json.loads(result.stdout)["ok"])
            self.assertFalse(self.output.exists())

    def test_malformed_request_is_rejected_before_log_creation(self):
        for change in ({"schema_version": True}, {"extra": "not allowed"}, {"input_roots": [False]}):
            with self.subTest(change=change):
                self.path.write_bytes(canonical_json(dict(self.request, **change)))
                result = self.run_cli()
                self.assertEqual(2, result.returncode)
                self.assertFalse(json.loads(result.stdout)["ok"])
                self.assertFalse(self.output.exists())

    def test_incomplete_script_copy_returns_structured_import_failure(self):
        copy = self.store / "standalone-preflight.py"
        copy.write_bytes(self.script.read_bytes())
        bootstrap = self.script.with_name("workflow_delivery_source_only.py")
        (self.store / bootstrap.name).write_bytes(bootstrap.read_bytes())
        self.script = copy
        result = self.run_cli()
        self.assertEqual(2, result.returncode)
        report = json.loads(result.stdout)
        self.assertFalse(report["ok"])
        self.assertEqual("ModuleNotFoundError", report["findings"][0]["type"])
        self.assertFalse(self.output.exists())

    def test_missing_source_bootstrap_refuses_without_cache_fallback(self):
        copy = self.store / "standalone-preflight.py"
        copy.write_bytes(self.script.read_bytes())
        self.script = copy
        result = self.run_cli()
        self.assertEqual(2, result.returncode)
        report = json.loads(result.stdout)
        self.assertFalse(report["ok"])
        self.assertEqual("FileNotFoundError", report["findings"][0]["type"])
        self.assertIn("workflow_delivery_source_only.py", report["findings"][0]["detail"])
        self.assertFalse(self.output.exists())


if __name__ == "__main__":
    unittest.main()
