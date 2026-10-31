"""Validator-grounded tests for the GP-CM05E Phase-A U5 release-handoff assembly core.

Covers the frozen-identity consumption (environment_ids/binaries/boundaries carried verbatim),
the malformed-support and dangling-reference refusals, acceptance by the governed
``report.validate_release``, the governed validator's own refusals on a tampered handoff, and the
placeholder-lifecycle bar: attaching the real release notes closes ONLY the release leg, so the
report still stays non-ready under ``validate_report``.

Everything here is a SYNTHETIC identity/binding exercise -- the eight support documents and the
clean-user launch proof are synthetic placeholder blobs (their authored content and the real
launch capture are owner/campaign legs, never produced here). No test weakens a validator and no
path issues readiness.
"""
import json
import tempfile
import unittest
from pathlib import Path

from global_preferences_acceptance import assemble
from global_preferences_acceptance import producer
from global_preferences_acceptance import release_handoff as rh
from global_preferences_acceptance import report as report_module
from global_preferences_acceptance.capture import Capture
from global_preferences_acceptance.environment_record import build_environments
from global_preferences_acceptance.inputs import Bundle, EvidenceError, canonical, digest, parse
from global_preferences_acceptance.inventory import PRODUCT_BOUNDARY

REAL_ROOT = Path(__file__).resolve().parents[1]
NATIVE_SPEC = {"coordinate": "wayland/1.0/1024x900",
               "gpu": "Intel(R) HD Graphics P630 (KBL GT2)", "driver": "Mesa 25.0.7",
               "compositor": "owned nested Weston",
               "background_load": "quiescent -- floor-quiet, one capture process"}


class ReleaseHandoffTests(unittest.TestCase):
    def setUp(self):
        temporary = tempfile.TemporaryDirectory()
        self.addCleanup(temporary.cleanup)
        self.root = Path(temporary.name)
        self.bundle_root = self.root / "bundle"
        self.bundle_root.mkdir()
        self.capture = Capture(self.bundle_root)
        self.manifest = self.capture.put([{"path": "scripts/fixture.py",
                                           "sha256": digest(b"# recipe\n")}])
        frozen_binaries = {role: {"path": "command-capture/" + digest(role.encode()),
                                  "sha256": digest(role.encode())}
                           for role in assemble.CANDIDATE_ROLES}
        self.candidate = {"revision": "a" * 40, "tree": "b" * 40, "cargo_lock_sha256": "c" * 64,
                          "toolchains": {"rustc": "rustc", "cargo": "cargo", "python": "python3"},
                          "build_commands": [{"synthetic": True}], "binaries": frozen_binaries}
        self.environments = build_environments(self.root, [NATIVE_SPEC])
        self.identity = assemble.freeze_identity(self.candidate, self.environments,
                                                 self.manifest, "d" * 64)
        # Synthetic retained support/launch artifacts (content is an owner/campaign leg).
        self.support = {name: self.capture.put(f"synthetic {name}".encode())
                        for name in rh.SUPPORT_CATEGORIES}
        self.launch = self.capture.put(b"synthetic clean-user offline launch proof")

    def assemble_doc(self):
        ref = rh.assemble_release_handoff(self.capture, self.identity, support=self.support,
                                          clean_user_launch=self.launch)
        return ref, Bundle(self.bundle_root, list(self.capture.inventory.values())).json(ref)

    # --- frozen-identity consumption ----------------------------------------------------------
    def test_assemble_binds_identity_and_boundary_verbatim(self):
        _, doc = self.assemble_doc()
        self.assertEqual(set(doc), set("schema candidate_revision input_manifest_sha256 "
                                       "environment_ids binaries boundaries support "
                                       "clean_user_launch".split()))
        self.assertEqual(doc["schema"], rh.RELEASE_SCHEMA)
        self.assertEqual(doc["candidate_revision"], self.candidate["revision"])
        self.assertEqual(doc["input_manifest_sha256"], self.manifest["sha256"])
        self.assertEqual(doc["environment_ids"], [e["id"] for e in self.environments])
        self.assertEqual(doc["binaries"], self.candidate["binaries"])
        self.assertEqual(canonical(doc["boundaries"]), canonical(PRODUCT_BOUNDARY))
        self.assertEqual(set(doc["support"]), set(rh.SUPPORT_CATEGORIES))

    def test_assemble_refuses_malformed_support(self):
        short = {k: v for k, v in self.support.items() if k != "recovery"}
        with self.assertRaisesRegex(EvidenceError, "exactly the eight governed categories"):
            rh.assemble_release_handoff(self.capture, self.identity, support=short,
                                        clean_user_launch=self.launch)
        extra = {**self.support, "unexpected": self.launch}
        with self.assertRaisesRegex(EvidenceError, "exactly the eight governed categories"):
            rh.assemble_release_handoff(self.capture, self.identity, support=extra,
                                        clean_user_launch=self.launch)

    def test_assemble_refuses_dangling_reference(self):
        dangling = {"path": "command-capture/" + "0" * 64, "sha256": "0" * 64}
        with self.assertRaisesRegex(EvidenceError, "support reference does not resolve"):
            rh.assemble_release_handoff(self.capture, self.identity,
                                        support={**self.support, "storage": dangling},
                                        clean_user_launch=self.launch)
        with self.assertRaisesRegex(EvidenceError, "launch proof does not resolve"):
            rh.assemble_release_handoff(self.capture, self.identity, support=self.support,
                                        clean_user_launch=dangling)

    # --- the governed validator is the real gate ----------------------------------------------
    def _report_with_release(self):
        report = assemble.assembled_report(self.capture, self.identity,
                                           case_results=(), measurements=())
        report = rh.attach_release_handoff(report, self.capture, self.identity,
                                           support=self.support, clean_user_launch=self.launch)
        return report, Bundle(self.bundle_root, report["artifacts"])

    def test_validate_release_accepts_the_assembled_handoff(self):
        report, bundle = self._report_with_release()
        # The governed validator accepts the assembled handoff against the matching report.
        report_module.validate_release(bundle.json(report["release_notes"]), report, bundle)

    def test_validate_release_refuses_tampered_handoff(self):
        report, _ = self._report_with_release()
        doc = Bundle(self.bundle_root, report["artifacts"]).json(report["release_notes"])
        for mutate, message in (
                (lambda d: d.update(environment_ids=d["environment_ids"][:-1]), "release environments differ"),
                (lambda d: d.update(binaries={}), "distributed binary identities differ"),
                (lambda d: d["boundaries"].update(active_global_descriptors=999), "release boundary differs"),
                (lambda d: d.update(candidate_revision="e" * 40), "release identity differs")):
            with self.subTest(message=message):
                tampered = json.loads(json.dumps(doc))
                mutate(tampered)
                ref = self.capture.put(tampered)
                bundle = Bundle(self.bundle_root, list(self.capture.inventory.values()))
                with self.assertRaisesRegex(EvidenceError, message):
                    report_module.validate_release(bundle.json(ref), report, bundle)

    # --- placeholder lifecycle: closing the release leg does NOT make the report ready ---------
    def test_attached_report_still_non_ready(self):
        report, bundle = self._report_with_release()
        # The release leg is closed: its reference is the real handoff, not the pending placeholder.
        self.assertEqual(bundle.json(report["release_notes"])["schema"], rh.RELEASE_SCHEMA)
        self.assertNotEqual(bundle.json(report["release_notes"])["schema"],
                            assemble.PENDING_RELEASE_SCHEMA)
        # ...but independent_review is still the pending placeholder, and the corpus is empty, so the
        # governed full validator still refuses: closing one leg never issues readiness.
        self.assertEqual(bundle.json(report["independent_review"])["schema"],
                         assemble.PENDING_REVIEW_SCHEMA)
        matrix = parse((REAL_ROOT / producer.MATRIX_RELPATH).read_bytes(), "matrix")
        with self.assertRaises(EvidenceError):
            report_module.validate_report(
                report, matrix, root=REAL_ROOT, bundle_root=self.bundle_root,
                expected_revision=self.candidate["revision"],
                expected_environment_sha256=self.identity["environment_sha256"],
                review_only=True)


if __name__ == "__main__":
    unittest.main()
