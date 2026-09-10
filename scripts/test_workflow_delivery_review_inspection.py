"""Synthetic pre-review source inspection, never rollout acceptance evidence."""

from copy import deepcopy
from datetime import datetime, timedelta, timezone
import json
from pathlib import Path
import subprocess
import sys
import tempfile
import unittest

import test_workflow_delivery_initial_migration as migration_support
from workflow_delivery_activation_preflight import ROLLOUT
from workflow_delivery_capture_state import protected_state
from workflow_delivery_checkpoints import validate_delivery
from workflow_delivery_io import canonical_json, sha256
from workflow_delivery_review_inspection import inspect_review_candidate
from workflow_delivery_tree import Tree
from workflow_delivery_trust import Trust


class ReviewInspectionTest(unittest.TestCase):
    def setUp(self):
        self.fixture = migration_support.InitialMigrationTest()
        self.addCleanup(self.fixture.doCleanups)
        self.fixture.setUp()
        self.case = self.fixture.case
        self.f = self.case.f
        self.prepare("WDQ-COMPAT")

    def prepare(self, selected):
        manifest = Tree(self.f.root).json("specs/active_frontier.json")
        item = manifest["frontier"][0]
        now = datetime.now(timezone.utc) - timedelta(seconds=5)
        stamp = lambda value: value.isoformat().replace("+00:00", "Z")
        item.update(state="in_progress", authorization="execution", claim={
            "agent": "codex", "harness": "codex-cli", "session": "synthetic-review",
            "worktree": str(self.f.root), "head": self.case.base, "scope": ["synthetic review fixture"],
            "claimed_at": stamp(now), "heartbeat_at": stamp(now),
            "expires_at": stamp(now + timedelta(hours=1))})
        item["completion"]["canonical_next_step_id"] = selected
        for step in item["completion"]["steps"]:
            if step["id"] in ("WDQ-COMPAT", "WDQ-RECHECK"):
                step["status"] = ("in_progress" if step["id"] == selected else
                                  "complete" if selected == "WDQ-RECHECK" else "pending")
                if step["status"] != "complete":
                    step["completion_evidence"] = []
                else:
                    step["completion_evidence"] = [{"kind": "document", "path": item["governing_docs"][0],
                                                    "marker": "DONE-" + step["id"]}]
        issues = [json.loads(line) for line in (self.f.root / ".beads/issues.jsonl").read_bytes().splitlines()]
        issues[0].update(status="in_progress", assignee="codex")
        self.f.write(".beads/issues.jsonl", b"".join(canonical_json(i) for i in issues))
        policy = Tree(self.f.root).json("specs/workflow_delivery_policy.json")
        legacy = {k: deepcopy(v) for k, v in policy.items() if k != "coverage"}
        legacy.update(schema_version=1, enrolled=[deepcopy(policy["enrolled"][0])])
        baseline = deepcopy(manifest)
        del baseline["frontier"][0]["completion"]["delivery"]
        self.fixture.save_manifest(baseline)
        self.f.save("specs/workflow_delivery_policy.json", legacy)
        self.case.commit_candidate()
        self.case.base = self.case.candidate
        self.fixture.save_manifest(manifest)
        self.f.save("specs/workflow_delivery_policy.json", policy)
        for path in (self.f.contract["proof_path"], self.f.contract["review_path"]):
            if (self.f.root / path).exists():
                self.f.git("rm", "--", path)
        self.case.commit_candidate()

    def inspect(self):
        return inspect_review_candidate(self.f.root, base=self.case.base, candidate=self.case.candidate,
            authority=self.case.candidate, environment_path="requested-environment.json",
            publication_review=self.case.review)

    def test_missing_future_proof_allowed_only_for_non_authorizing_review(self):
        before = protected_state(self.f.root, ["src"])
        result = self.inspect()
        self.assertEqual("WDQ-COMPAT", result["selected_step"])
        self.assertEqual(["PILOT: verify"], result["checks"])
        for field in ("evidence_validated", "publication_authorized", "activation_asserted"):
            self.assertIs(result[field], False)
        self.assertEqual(before, protected_state(self.f.root, ["src"]))
        with self.assertRaisesRegex(ValueError, "claim-free WDQ-I04"):
            self.case.inspect()

    def test_recheck_can_inspect_before_its_own_review_packet_exists(self):
        self.prepare("WDQ-RECHECK")
        self.assertEqual("WDQ-RECHECK", self.inspect()["selected_step"])
        with self.assertRaisesRegex(ValueError, "claim-free WDQ-I04"):
            self.case.inspect()

    def test_candidate_cannot_rewrite_its_own_claim(self):
        manifest = Tree(self.f.root).json("specs/active_frontier.json")
        manifest["frontier"][0]["claim"]["scope"] = ["rewritten scope"]
        self.fixture.save_manifest(manifest)
        self.case.commit_candidate()
        with self.assertRaisesRegex(ValueError, "cannot advance or rewrite"):
            self.inspect()

    def test_candidate_cannot_change_another_lane(self):
        manifest = Tree(self.f.root).json("specs/active_frontier.json")
        manifest["frontier"][1]["title"] += " changed"
        self.fixture.save_manifest(manifest)
        self.case.commit_candidate()
        with self.assertRaisesRegex(ValueError, "another Frontier lane"):
            self.inspect()

    def test_unreviewed_production_path_refuses(self):
        self.f.write("scripts/unowned.py", b"# synthetic out-of-scope source\n")
        self.case.commit_candidate()
        with self.assertRaisesRegex(ValueError, "lacks exact reviewed scope"):
            self.inspect()

    def test_existing_enrollment_still_requires_its_proof(self):
        self.f.git("rm", "--", "docs/reviews/pilot-proof.json")
        self.case.commit_candidate()
        with self.assertRaisesRegex(ValueError, "required artifact is not in candidate Git tree"):
            self.inspect()

    def test_stale_claim_refuses(self):
        manifest = Tree(self.f.root).json("specs/active_frontier.json")
        manifest["frontier"][0]["claim"]["expires_at"] = "2020-01-01T01:00:00Z"
        self.fixture.save_manifest(manifest)
        self.case.commit_candidate()
        with self.assertRaisesRegex(ValueError, "claim"):
            self.inspect()

    def test_ordinary_changed_input_readiness_still_demands_proof(self):
        self.f.write("src/read.py", b"# changed synthetic input\n")
        self.case.commit_candidate()
        tree = Tree(self.f.root, revision=self.case.candidate)
        trust = Trust(tree, self.case.candidate, self.case.base)
        item = tree.json("specs/active_frontier.json")["frontier"][0]
        self.assertEqual(ROLLOUT, item["key"])
        with self.assertRaisesRegex(ValueError, "required artifact is not in candidate Git tree"):
            validate_delivery(tree, item, phase="ready", trust=trust)

    def test_review_cli_preserves_main_and_cannot_be_used_as_activation_request(self):
        from workflow_delivery_support_bundle import prepare_bundle
        from workflow_delivery_support_store import support_locations, proposed_local_trust
        temp = tempfile.TemporaryDirectory()
        self.addCleanup(temp.cleanup)
        live = Path(temp.name) / "main"
        # Branch operations are confined to the disposable synthetic fixture.
        self.f.git("branch", "-m", "fixture-candidate")
        self.f.git("branch", "main", self.case.base)
        self.f.git("worktree", "add", "--quiet", str(live), "main")
        prepare_bundle(live, self.case.candidate)
        locations = support_locations(live, self.case.candidate)
        before = protected_state(live, ["src"])
        request = {"schema_version": 1, "kind": "datum.workflow-delivery.review-inspection-request",
            "base": self.case.base, "candidate": self.case.candidate, "authority": self.case.candidate,
            "input_roots": ["src"], "prior_local_trust": before["local_trust"],
            "publication_review": self.case.review, "environment_path": "requested-environment.json",
            "proposed_local_trust": proposed_local_trust(live, authority=self.case.candidate,
                base=self.case.base, environment_path="requested-environment.json")}
        path = Path(temp.name) / "request.json"
        path.write_bytes(canonical_json(request))
        output = locations["logs"] / "review-fixture.json"
        arguments = ["--root", str(live), "--request", str(path),
                     "--request-sha256", sha256(path.read_bytes()), "--output", str(output)]
        result = subprocess.run([sys.executable, "-I", "-S", "-B",
            str(Path(__file__).with_name("workflow_delivery_preflight_cli.py")),
            "--inspect-review", *arguments], capture_output=True, timeout=30)
        self.assertEqual(0, result.returncode, result.stdout + result.stderr)
        payload = json.loads(output.read_bytes())
        self.assertTrue(payload["ok"])
        self.assertFalse(payload["evidence_validated"])
        self.assertFalse(payload["activation_asserted"])
        self.assertFalse(payload["publication_authorized"])
        self.assertEqual("WDQ-COMPAT", payload["review_inspection"]["selected_step"])
        self.assertEqual(before, protected_state(live, ["src"]))
        response = locations["store"] / "owner-responses/fixture.json"
        response.parent.mkdir(parents=True)
        response.write_bytes(canonical_json({"schema_version": 1,
            "kind": "datum.workflow-delivery.activation-response",
            "response": "WORKFLOW-DELIVERY-IMPLEMENTATION: approve ACTIVATE — " + sha256(path.read_bytes()),
            "source": "Synthetic input, not actual owner approval", "recorded_at": "2026-09-10"}))
        arguments[-1] = str(locations["logs"] / "refused-activation.json")
        refused = subprocess.run([sys.executable, "-I", "-S", "-B",
            str(locations["scripts"] / "workflow_delivery_preflight_cli.py"), "--activate", *arguments,
            "--owner-response", str(response), "--owner-response-sha256", sha256(response.read_bytes())],
            capture_output=True, timeout=30)
        self.assertEqual(2, refused.returncode, refused.stdout + refused.stderr)
        self.assertIn(b"preflight request", refused.stdout)
        self.assertEqual(before, protected_state(live, ["src"]))


if __name__ == "__main__":
    unittest.main()
