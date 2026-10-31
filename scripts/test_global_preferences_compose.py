"""Validator-grounded tests for the GP-CM05E Phase-A U6 composition core.

The composition consumes three per-corpus reports (case / measurement / native) produced under the
ONE frozen identity and composes them into the single complete v2 report, replaying the governed
per-corpus validators and gating each sub-report's identity against the frozen baton.

Cross-check: the complete synthetic corpora from test_support.SyntheticBundle are split into three
per-corpus sub-reports and recomposed; the composed (product-execution) report must carry exactly
the same rows and satisfy the same governed validators as the monolithic synthetic report, while
its origin stays DISTINCT (product-execution vs synthetic-test, per the settled synthetic ruling).
The refusals are falsifiable: a drifted identity, a partial corpus, a tampered row binary and a
duplicate row each refuse loudly -- never a silent partial PASS. No test weakens a validator and no
path issues readiness.
"""
from copy import deepcopy
import tempfile
import unittest

from global_preferences_acceptance import compose
from global_preferences_acceptance.identity import validate_environments
from global_preferences_acceptance.inputs import Bundle, EvidenceError, canonical, digest
from global_preferences_acceptance.observations import validate_cases
from global_preferences_acceptance.report import validate_report
from global_preferences_acceptance.test_support import SyntheticBundle
from test_global_preferences_production_matrix import MATRIX


class CompositionTests(unittest.TestCase):
    @classmethod
    def setUpClass(cls):
        cls.temporary = tempfile.TemporaryDirectory()
        cls.fixture = SyntheticBundle(cls.temporary.name, MATRIX)
        report = cls.fixture.report
        cls.identity = {
            "candidate": report["candidate"],
            "input_manifest": report["input_manifest"],
            "matrix_sha256": report["matrix_sha256"],
            "environments": report["environments"],
            "environment_sha256": digest(canonical(report["environments"])),
            "selection": [],
        }

    @classmethod
    def tearDownClass(cls):
        cls.temporary.cleanup()

    def sub(self, corpus):
        """A per-corpus sub-report: the shared frozen identity + artifacts + one corpus's rows,
        exactly the single-corpus shape assemble.py / measurement_assembly.py / the native capture
        each write."""
        base = {k: self.identity[k] for k in ("candidate", "input_manifest", "matrix_sha256",
                                              "environments")}
        base["artifacts"] = self.fixture.report["artifacts"]
        base[corpus] = self.fixture.report[corpus]
        return base

    def compose(self, *, case=None, measurement=None, native=None):
        return compose.compose_report(
            self.fixture.directory, self.identity,
            case_report=case if case is not None else self.sub("case_results"),
            measurement_report=measurement if measurement is not None else self.sub("measurements"),
            native_report=native if native is not None else self.sub("native_observations"),
            producer_session="dev-agent-01")

    def test_composes_the_three_corpora_verbatim_under_one_identity(self):
        composed = self.compose()
        # The rows are carried verbatim from each per-corpus report.
        self.assertEqual(composed["case_results"], self.fixture.report["case_results"])
        self.assertEqual(composed["measurements"], self.fixture.report["measurements"])
        self.assertEqual(composed["native_observations"], self.fixture.report["native_observations"])
        # One complete report in the closed v2 shape, frozen identity verbatim.
        self.assertEqual(set(composed), set(
            "schema origin candidate matrix_sha256 input_manifest environments artifacts "
            "case_results measurements gate_results native_observations independent_review "
            "release_notes".split()))
        self.assertEqual(composed["schema"], "datum-global-preferences-production-evidence-v2")
        self.assertEqual(canonical(composed["candidate"]), canonical(self.identity["candidate"]))
        # Distinct origins per the settled synthetic ruling: the composed product report is
        # product-execution; the synthetic source stays synthetic-test.
        self.assertEqual(composed["origin"]["kind"], "product-execution")
        self.assertEqual(self.fixture.report["origin"]["kind"], "synthetic-test")
        # The composed corpora satisfy the same governed validator the synthetic report does.
        bundle = Bundle(self.fixture.directory, composed["artifacts"])
        environments = validate_environments(composed["environments"])
        binaries = {k: v["sha256"] for k, v in composed["candidate"]["binaries"].items()}
        validate_cases(composed, bundle, environments, binaries)

    def test_composed_report_stays_non_ready_until_phase_b_legs(self):
        composed = self.compose()
        # Complete corpora, but gate_results/review/release are still pending placeholders, so the
        # governed full validator refuses -- composition issues no readiness.
        self.assertEqual(composed["gate_results"], [])
        with self.assertRaises(EvidenceError):
            validate_report(composed, MATRIX, root=self.fixture.root,
                            bundle_root=self.fixture.directory,
                            expected_revision=self.fixture.revision,
                            expected_environment_sha256=self.identity["environment_sha256"],
                            review_only=True)

    def test_refuses_a_sub_report_whose_identity_drifts_from_frozen(self):
        drifted = self.sub("case_results")
        drifted["matrix_sha256"] = "0" * 64
        with self.assertRaisesRegex(EvidenceError, "differs from the frozen shared identity"):
            self.compose(case=drifted)

    def test_refuses_a_partial_case_corpus(self):
        partial = self.sub("case_results")
        partial["case_results"] = partial["case_results"][:-1]
        with self.assertRaisesRegex(EvidenceError, "count differs"):
            self.compose(case=partial)

    def test_refuses_a_partial_measurement_corpus(self):
        partial = self.sub("measurements")
        partial["measurements"] = partial["measurements"][:-1]
        with self.assertRaisesRegex(EvidenceError, "measurement count differs"):
            self.compose(measurement=partial)

    def test_refuses_a_partial_native_corpus(self):
        partial = self.sub("native_observations")
        partial["native_observations"] = partial["native_observations"][:-1]
        with self.assertRaisesRegex(EvidenceError, "native observation count differs"):
            self.compose(native=partial)

    def test_refuses_a_tampered_row_binary(self):
        tampered = deepcopy(self.sub("case_results"))
        tampered["case_results"][0]["binary_sha256"] = "0" * 64
        with self.assertRaises(EvidenceError):
            self.compose(case=tampered)

    def test_refuses_a_duplicate_native_row(self):
        # Keep the count at the required inventory size but make one scenario/env appear twice, so
        # the dedup refusal fires rather than the count check.
        duplicated = deepcopy(self.sub("native_observations"))
        duplicated["native_observations"][1] = deepcopy(duplicated["native_observations"][0])
        with self.assertRaisesRegex(EvidenceError, "duplicate native scenario"):
            self.compose(native=duplicated)


if __name__ == "__main__":
    unittest.main()
