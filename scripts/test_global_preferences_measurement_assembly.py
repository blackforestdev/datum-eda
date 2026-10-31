"""Validator-grounded tests for the GP-CM05E Phase-A U4 measurement assembly core.

Covers the frozen-identity consumption that replaces the producer ``main()``'s HEAD/binary/matrix
recompute (:func:`frozen_common`), the U3 merge refusals applied to measurement rows (duplicate
coverage, identity mismatch, unfrozen environment, non-candidate binary), the pre-warm-once /
produce-per-budget / validate-before-merge wiring of :func:`assemble_measurements`, and the
placeholder lifecycle that keeps a partial measurement report non-ready.

Everything here is a SYNTHETIC identity/merge exercise (hand-built rows, spied producer builders)
-- never a product proof and never a live measurement campaign. The partially assembled report is
EXPLICITLY asserted to stay incomplete under the governed validators; no test weakens a validator,
and no path issues readiness.
"""
import json
import tempfile
import unittest
from pathlib import Path
from unittest.mock import MagicMock, patch

from global_preferences_acceptance import measurement_assembly as ma
from global_preferences_acceptance import measurements_producer as mp
from global_preferences_acceptance import producer
from global_preferences_acceptance import report as report_module
from global_preferences_acceptance import assemble
from global_preferences_acceptance.capture import Capture
from global_preferences_acceptance.environment_record import HEADLESS_ID, build_environments
from global_preferences_acceptance.inputs import Bundle, EvidenceError, canonical, digest, parse

REAL_ROOT = Path(__file__).resolve().parents[1]
NATIVE_SPEC = {"coordinate": "wayland/1.0/1024x900",
               "gpu": "Intel(R) HD Graphics P630 (KBL GT2)", "driver": "Mesa 25.0.7",
               "compositor": "owned nested Weston",
               "background_load": "quiescent -- floor-quiet, one capture process"}


class MeasurementAssemblyTests(unittest.TestCase):
    def setUp(self):
        temporary = tempfile.TemporaryDirectory()
        self.addCleanup(temporary.cleanup)
        self.root = Path(temporary.name)
        self.bundle_root = self.root / "bundle"
        self.bundle_root.mkdir()
        shared = Capture(self.bundle_root)
        self.manifest = shared.put([{"path": "scripts/fixture.py", "sha256": digest(b"# recipe\n")}])
        frozen_binaries = {role: {"path": "command-capture/" + digest(role.encode()),
                                  "sha256": digest(role.encode())}
                           for role in assemble.CANDIDATE_ROLES}
        self.candidate = {"revision": "a" * 40, "tree": "b" * 40, "cargo_lock_sha256": "c" * 64,
                          "toolchains": {"rustc": "rustc", "cargo": "cargo", "python": "python3"},
                          "build_commands": [{"synthetic": True}], "binaries": frozen_binaries}
        self.environments = build_environments(self.root, [NATIVE_SPEC])
        self.identity = assemble.freeze_identity(self.candidate, self.environments,
                                                 self.manifest, "d" * 64)
        self.role_shas = {role: ref["sha256"] for role, ref in frozen_binaries.items()}

    # --- synthetic row helper (merge inspects identity only; no trials needed) ---------------
    def row(self, budget_id, variant_id, *, environment_id=HEADLESS_ID, **overrides):
        role = ma.measurement_role(budget_id)
        base = {"budget_id": budget_id, "variant_id": variant_id, "environment_id": environment_id,
                "candidate_revision": self.candidate["revision"],
                "binary_sha256": self.role_shas[role],
                "input_manifest_sha256": self.manifest["sha256"],
                "matrix_sha256": "d" * 64, "trials": []}
        base.update(overrides)
        return base

    # --- frozen-identity consumption ----------------------------------------------------------
    def test_frozen_common_consumes_the_baton_verbatim(self):
        common = ma.frozen_common(self.identity)
        self.assertEqual(common, {"candidate_revision": self.candidate["revision"],
                                  "binary_sha256": self.role_shas["cli"],
                                  "input_manifest_sha256": self.manifest["sha256"],
                                  "matrix_sha256": "d" * 64,
                                  "environment_id": HEADLESS_ID})
        # The headless id is actually present in the frozen 21-field array (one binding).
        self.assertIn(HEADLESS_ID, {e["id"] for e in self.identity["environments"]})
        # The gui role selects a different candidate binary, still verbatim from the baton.
        self.assertEqual(ma.frozen_common(self.identity, role="gui",
                                          environment_id="native-wayland-1-1024x900")["binary_sha256"],
                         self.role_shas["gui"])

    def test_frozen_common_refuses_unfrozen_environment_or_missing_role(self):
        with self.assertRaisesRegex(EvidenceError, "not an id in the frozen"):
            ma.frozen_common(self.identity, environment_id="headless-not-selected")
        with self.assertRaisesRegex(EvidenceError, "no mcp-render binary"):
            ma.frozen_common(self.identity, role="mcp-render")

    # --- U3 merge semantics on measurement rows ----------------------------------------------
    def test_merge_binds_frozen_identity_on_rows(self):
        rows = [self.row("release-query", "describe/defaults"),
                self.row("storage-growth", "generation"),
                self.row("gui-feedback", "focus", environment_id="native-wayland-1-1024x900")]
        merged = ma.merge_measurements(self.identity, rows)
        self.assertEqual(len(merged), 3)
        self.assertEqual(merged[2]["binary_sha256"], self.role_shas["gui"])  # gui budget -> gui binary

    def test_merge_refuses_duplicate_row(self):
        duplicate = self.row("release-query", "describe/defaults")
        with self.assertRaisesRegex(EvidenceError, "duplicate measurement row"):
            ma.merge_measurements(self.identity, [duplicate, dict(duplicate)])

    def test_merge_refuses_identity_mismatch(self):
        for field, value, message in (
                ("candidate_revision", "e" * 40, "row identity differs"),
                ("input_manifest_sha256", "0" * 64, "row identity differs"),
                ("matrix_sha256", "f" * 64, "row identity differs"),
                ("environment_id", "unfrozen-environment", "not an id in the frozen"),
                ("binary_sha256", self.role_shas["gui"], "not the frozen candidate cli binary")):
            with self.subTest(field=field):
                tampered = self.row("release-query", "describe/defaults", **{field: value})
                with self.assertRaisesRegex(EvidenceError, message):
                    ma.merge_measurements(self.identity, [tampered])
        # A GUI-budget row must bind the gui binary, not the cli one (role resolution).
        with self.assertRaisesRegex(EvidenceError, "not the frozen candidate gui binary"):
            ma.merge_measurements(self.identity, [self.row(
                "gui-feedback", "focus", environment_id="native-wayland-1-1024x900",
                binary_sha256=self.role_shas["cli"])])

    # --- placeholder lifecycle: a partial corpus stays non-ready ------------------------------
    def test_partial_measurement_report_stays_non_ready(self):
        rows = ma.merge_measurements(self.identity, [self.row("release-query", "describe/defaults")])
        capture = Capture(self.bundle_root)
        report = assemble.assembled_report(capture, self.identity, case_results=(), measurements=rows)
        # The measurement feed landed in the skeleton, under product-execution origin.
        self.assertEqual(report["measurements"], rows)
        self.assertEqual(report["origin"]["kind"], "product-execution")
        self.assertEqual(report["case_results"], [])
        self.assertEqual(report["gate_results"], [])
        bundle = Bundle(self.bundle_root, report["artifacts"])
        # The explicit placeholders can never satisfy their governed validators.
        with self.assertRaises(EvidenceError):
            report_module.validate_review(bundle.json(report["independent_review"]), report, bundle)
        with self.assertRaises(EvidenceError):
            report_module.validate_release(bundle.json(report["release_notes"]), report, bundle)
        # The governed full validator refuses the partial report outright (review_only included):
        # a one-variant corpus is INCOMPLETE and never reaches any readiness state.
        matrix = parse((REAL_ROOT / producer.MATRIX_RELPATH).read_bytes(), "matrix")
        with self.assertRaises(EvidenceError):
            report_module.validate_report(
                report, matrix, root=REAL_ROOT, bundle_root=self.bundle_root,
                expected_revision=self.candidate["revision"],
                expected_environment_sha256=self.identity["environment_sha256"],
                review_only=True)

    # --- assemble_measurements wiring (no live campaign: builders are spied) ------------------
    def test_assemble_measurements_consumes_frozen_prewarms_once_validates_then_merges(self):
        produced_by = {}

        def fake_produce(selector, capture, binary, common, timeout, scratch, *, daemon_binary=None):
            produced_by[selector] = dict(common)  # record the identity the runner handed us
            capture.put(b"synthetic trial artifact for " + selector.encode())
            variant = {"release-query/defaults": "describe/defaults",
                       "storage": "generation"}[selector]
            budget = "release-query" if selector.startswith("release-query") else "storage-growth"
            return [self.row(budget, variant)]

        prewarm = MagicMock(return_value={"path": "command-capture/prewarm", "sha256": "0" * 64})
        validate = MagicMock()
        with patch.object(mp, "prewarm", prewarm), \
                patch.object(mp, "_produce", side_effect=fake_produce), \
                patch.object(ma.meas, "validate_trials", validate):
            report = ma.assemble_measurements(
                self.bundle_root, self.identity, binary=self.root / "bin" / "datum-eda",
                scratch=self.root / "scratch",
                budgets=("release-query/defaults", "storage"))

        # Pre-warm ran exactly once, with the FROZEN revision + cli binary sha (not a HEAD recompute).
        prewarm.assert_called_once()
        self.assertEqual(prewarm.call_args.args[2], self.candidate["revision"])
        self.assertEqual(prewarm.call_args.args[3], self.role_shas["cli"])
        # Each budget was produced once under the SAME frozen-derived common (verbatim, no recompute).
        self.assertEqual(set(produced_by), {"release-query/defaults", "storage"})
        for common in produced_by.values():
            self.assertEqual(common, ma.frozen_common(self.identity))
        # Every produced row was replayed through the governed validator BEFORE merge.
        self.assertEqual(validate.call_count, 2)
        # The merged rows reached the report skeleton with the placeholder lifecycle intact.
        self.assertEqual({r["variant_id"] for r in report["measurements"]},
                         {"describe/defaults", "generation"})
        self.assertEqual(report["origin"]["kind"], "product-execution")
        bundle = Bundle(self.bundle_root, report["artifacts"])
        self.assertEqual(bundle.json(report["independent_review"])["schema"],
                         assemble.PENDING_REVIEW_SCHEMA)
        self.assertEqual(bundle.json(report["release_notes"])["schema"],
                         assemble.PENDING_RELEASE_SCHEMA)

    def test_assemble_measurements_refuses_cross_budget_duplicate(self):
        def fake_produce(selector, capture, binary, common, timeout, scratch, *, daemon_binary=None):
            capture.put(b"artifact " + selector.encode())
            return [self.row("release-query", "describe/defaults")]  # same coverage from both

        with patch.object(mp, "prewarm", MagicMock()), \
                patch.object(mp, "_produce", side_effect=fake_produce), \
                patch.object(ma.meas, "validate_trials", MagicMock()):
            with self.assertRaisesRegex(EvidenceError, "duplicate measurement row"):
                ma.assemble_measurements(
                    self.bundle_root, self.identity, binary=self.root / "bin" / "datum-eda",
                    scratch=self.root / "scratch",
                    budgets=("release-query/defaults", "release-query/populated"))


if __name__ == "__main__":
    unittest.main()
