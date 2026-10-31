"""Drift + fence regressions for the GP-CM05E coverage-disposition unit (the SURFACE_DISPOSITIONS
analog for the timed/native corpora).

Pins, both directions, that the strict required inventories equal the full matrix minus the governed
dispositions -- so the registry and the recompute can never silently drift -- and that every
disposition names real matrix coordinates/keys (no orphan disposition, no silent partial). The
full-matrix coordinate space and the matrix protocol's scale_factors are unchanged; only the strict
REQUIRED sets reduce (measurement 90->58, native 48->0). No validator is weakened here.
"""
import tempfile
import unittest

from global_preferences_acceptance.inputs import EvidenceError, canonical, digest
from global_preferences_acceptance.inventory import (
    ENV_BLOCKED_SCALE2_COORDINATES, MEASUREMENT_DISPOSITIONS, NATIVE_DISPOSITIONS)
from global_preferences_acceptance.matrix import gui_coordinates
from global_preferences_acceptance.measurements import (
    measurement_env_blocked_variants, measurement_inventory, measurement_matrix)
from global_preferences_acceptance.observations import (
    native_inventory, native_matrix, native_scope_exited)
from global_preferences_acceptance.report import validate_report
from global_preferences_acceptance.test_support import SyntheticBundle
from test_global_preferences_production_matrix import MATRIX


class MeasurementDispositionTests(unittest.TestCase):
    def test_counts(self):
        self.assertEqual(len(measurement_matrix()), 90)
        self.assertEqual(len(measurement_env_blocked_variants()), 32)
        self.assertEqual(len(measurement_inventory()), 58)

    def test_drift_both_directions(self):
        matrix, owed, strict = (measurement_matrix(), measurement_env_blocked_variants(),
                                measurement_inventory())
        self.assertEqual(matrix - owed, strict)                 # forward: matrix - disposed == strict
        self.assertEqual(matrix - strict, owed)                 # reverse: matrix - strict == disposed
        self.assertTrue(owed <= matrix)                         # disposed is a real subset
        self.assertEqual(strict & owed, set())                  # strict and disposed are disjoint

    def test_only_scale2_coordinates_are_owed(self):
        self.assertTrue(set(ENV_BLOCKED_SCALE2_COORDINATES) <= set(gui_coordinates()))
        self.assertTrue(all(scale == 2.0 for _, scale, _ in ENV_BLOCKED_SCALE2_COORDINATES))
        self.assertTrue(all(key[2] in set(ENV_BLOCKED_SCALE2_COORDINATES)
                            for key in measurement_env_blocked_variants()))
        self.assertTrue(all(key[2] is None or key[2][1] != 2.0 for key in measurement_inventory()))

    def test_fence_rejects_non_matrix_coordinate(self):
        # The fence reads the name bound in the measurements module namespace.
        import global_preferences_acceptance.measurements as meas
        from global_preferences_acceptance.inputs import EvidenceError
        saved = meas.ENV_BLOCKED_SCALE2_COORDINATES
        meas.ENV_BLOCKED_SCALE2_COORDINATES = saved + (("wayland", 9.0, (1, 1)),)
        try:
            with self.assertRaises(EvidenceError):
                measurement_env_blocked_variants()
        finally:
            meas.ENV_BLOCKED_SCALE2_COORDINATES = saved


class NativeDispositionTests(unittest.TestCase):
    def test_counts(self):
        self.assertEqual(len(native_matrix()), 48)
        self.assertEqual(len(native_scope_exited()), 48)
        self.assertEqual(len(native_inventory()), 0)

    def test_drift_both_directions(self):
        matrix, exited, strict = native_matrix(), native_scope_exited(), native_inventory()
        self.assertEqual(matrix - exited, strict)
        self.assertEqual(strict, set())
        self.assertTrue(exited <= matrix)


class RegistryIntegrityTests(unittest.TestCase):
    def test_measurement_disposition_fields(self):
        for key, disposition in MEASUREMENT_DISPOSITIONS.items():
            self.assertEqual(disposition["mode"], "env_blocked")
            self.assertTrue(disposition["citations"], f"{key}: env_blocked needs citations")
            self.assertEqual(disposition["owner_gate"], "GP-CM05A")
            self.assertTrue(disposition["qualification_followup"])

    def test_native_disposition_fields(self):
        for key, disposition in NATIVE_DISPOSITIONS.items():
            self.assertEqual(disposition["mode"], "scope_exit")
            self.assertTrue(disposition["citations"], f"{key}: scope_exit needs citations")
            self.assertEqual(disposition["owner_gate"], "GP-CM05A")
            self.assertTrue(disposition["qualification_followup"])


class DispositionedReportIsNonReadyTests(unittest.TestCase):
    """The intended consequence of native's structural scope-exit: while native_inventory() is empty,
    validate_native requires zero native observations yet validate_review (§9) requires a native
    replay category, so NO report can reach a readiness verdict -- the honest always-non-ready state
    the rider promises the owner. Built under the ACTIVE dispositions (no nominal lift)."""

    @classmethod
    def setUpClass(cls):
        cls.temporary = tempfile.TemporaryDirectory()
        cls.fixture = SyntheticBundle(cls.temporary.name, MATRIX)

    @classmethod
    def tearDownClass(cls):
        cls.temporary.cleanup()

    def test_full_synthetic_report_carries_the_dispositioned_shape(self):
        report = self.fixture.report
        self.assertEqual(report["native_observations"], [])            # native scope-exited
        self.assertEqual(len(report["measurements"]), 58)              # scale-2 env-blocked

    def test_validate_report_is_non_ready_while_native_scope_exited(self):
        # Even the origin-walled synthetic (test_only) path cannot pass: validate_review rejects the
        # §9 replay that now carries no native category.
        with self.assertRaisesRegex(EvidenceError, "replay incomplete"):
            validate_report(self.fixture.report, MATRIX, root=self.fixture.root,
                            bundle_root=self.fixture.directory, expected_revision=self.fixture.revision,
                            expected_environment_sha256=digest(canonical(self.fixture.report["environments"])),
                            test_only=True)


if __name__ == "__main__":
    unittest.main()
