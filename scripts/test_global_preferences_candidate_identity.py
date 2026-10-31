"""Validator-grounded tests for the GP-CM05E Phase-A U2 candidate-identity builder.

The deterministic identity + input-manifest closure are asserted here against the SAME sources
validate_candidate (identity.py) consumes -- git identity and required_paths -- and against
report.py's binding digest. The release build + six-role binary retention are capture-time
orchestration (build_candidate) whose full acceptance is validate_candidate at the real capture on
the re-pinned, clean, orchestrator-inclusive candidate; they are not reproduced synthetically here.
No readiness is issued by this unit."""
import json
import tempfile
import unittest
from pathlib import Path

from global_preferences_acceptance import candidate_identity as ci
from global_preferences_acceptance.capture import Capture
from global_preferences_acceptance.identity import git, required_paths
from global_preferences_acceptance.inputs import canonical, digest

ROOT = str(Path(__file__).resolve().parents[1])


def _head(root):
    return git(root, "rev-parse", "--verify", "HEAD^{commit}").decode().strip()


class CandidateIdentityTests(unittest.TestCase):
    def test_roles_are_exactly_the_six_validate_candidate_requires(self):
        self.assertEqual(set(ci.ROLES), {"cli", "gui", "daemon", "engine", "gui-render", "mcp"})

    def test_revision_identity_matches_git(self):
        rid = ci.revision_identity(ROOT, "HEAD")
        self.assertEqual(set(rid), {"revision", "tree", "cargo_lock_sha256", "toolchains"})
        self.assertEqual(rid["revision"], _head(ROOT))
        self.assertEqual(rid["tree"], git(ROOT, "rev-parse", _head(ROOT) + "^{tree}").decode().strip())
        self.assertEqual(rid["cargo_lock_sha256"], digest(git(ROOT, "show", _head(ROOT) + ":Cargo.lock")))
        self.assertEqual(set(rid["toolchains"]), {"rustc", "cargo", "python"})
        for value in rid["toolchains"].values():
            self.assertTrue(value and value.strip())

    def test_input_manifest_covers_the_required_closure_with_committed_digests(self):
        _roots, required = required_paths(ROOT, _head(ROOT))
        with tempfile.TemporaryDirectory() as d:
            ref = ci.input_manifest(ROOT, Capture(d), "HEAD")
            entries = json.loads((Path(d) / ref["path"]).read_bytes())
        self.assertEqual([e["path"] for e in entries], required)
        self.assertEqual(sorted(required), required, "closure must be sorted")
        self.assertEqual(len(set(required)), len(required), "closure must be unique")
        for entry in entries[:50]:  # digests are the committed blob hashes (spot-check for speed)
            self.assertEqual(entry["sha256"], digest(git(ROOT, "show", _head(ROOT) + ":" + entry["path"])))

    def test_input_manifest_sha_is_the_canonical_binding_digest(self):
        with tempfile.TemporaryDirectory() as d:
            ref = ci.input_manifest(ROOT, Capture(d), "HEAD")
            entries = json.loads((Path(d) / ref["path"]).read_bytes())
        # report.py / identity.py bind observations to input_manifest["sha256"]; it must be the
        # canonical digest of the retained entries, and the no-retain helper must agree.
        self.assertEqual(ref["sha256"], digest(canonical(entries)))
        self.assertEqual(ci.input_manifest_sha256(ROOT, "HEAD"), ref["sha256"])

    def test_candidate_dict_assembles_to_validate_candidate_key_set(self):
        # Shape conformance without the capture-time build: a candidate assembled from the real
        # identity plus a placeholder build observation + six role refs has exactly the closed key
        # set validate_candidate requires (revision tree cargo_lock_sha256 toolchains build_commands
        # binaries), with the six roles and three toolchains.
        rid = ci.revision_identity(ROOT, "HEAD")
        candidate = {**rid,
                     "build_commands": [{"command": [], "candidate_revision": rid["revision"],
                                         "input_manifest_sha256": "0" * 64, "exit_code": 0,
                                         "log": {"path": "p", "sha256": "0" * 64}}],
                     "binaries": {role: {"path": "p", "sha256": "0" * 64} for role in ci.ROLES}}
        self.assertEqual(set(candidate),
                         {"revision", "tree", "cargo_lock_sha256", "toolchains", "build_commands", "binaries"})
        self.assertEqual(set(candidate["binaries"]), set(ci.ROLES))


if __name__ == "__main__":
    unittest.main()
