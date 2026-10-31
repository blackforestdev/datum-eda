"""Validator-grounded tests for the GP-CM05E Unit M S4/GUI measurement producer.

Covers the frozen-identity consumption for the GUI surface (role ``gui`` + native environment id per
display coordinate), the three GUI budgets' seal shapes replayed through the GOVERNED
:func:`measurements.validate_trials` (gui-feedback present-after-interaction, native-window-open
cold-open + present-within-open, window-lifecycle warmed-baseline + no-leak), the one-attempt
preserve-on-fail (an over-budget / leaking drive makes the governed validator raise and the run
stop), and the explicit cross-corpus merge step's refusals on real row shapes (duplicate coverage
key, frozen-identity mismatch) -- the seam audit will probe.

Everything here injects a SYNTHETIC GuiDrive returning hand-built observations. No live GUI is
driven (that is the frontend runner's capture under the window), no validator is weakened, and the
assembled report is asserted to stay non-ready.
"""
import tempfile
import unittest
from unittest.mock import patch
from pathlib import Path

from global_preferences_acceptance import assemble
from global_preferences_acceptance import gui_measurements_producer as gm
from global_preferences_acceptance import measurements as meas
from global_preferences_acceptance.capture import Capture
from global_preferences_acceptance.environment_record import build_environments
from global_preferences_acceptance.inputs import Bundle, EvidenceError, digest

NATIVE_SPEC = {"coordinate": "wayland/1.0/1024x900",
               "gpu": "Intel(R) HD Graphics P630 (KBL GT2)", "driver": "Mesa 25.0.7",
               "compositor": "owned nested Weston",
               "background_load": "quiescent -- floor-quiet, one capture process"}
COORDINATE = ("wayland", 1.0, (1024, 900))
NATIVE_ID = "native-wayland-1-1024x900"


class FakeGuiDrive(gm.GuiDrive):
    """A synthetic runner drive. One strictly-increasing monotonic clock for every execution of the
    coordinate; in-budget timed latencies with the correct presentation ordering; a stable warmed
    window/lease inventory so the lifecycle baseline equals the final warm-up. ``timed_elapsed_ns`` /
    ``orphan_windows`` can be overridden to exercise the one-attempt preserve-on-fail path."""

    def __init__(self, *, timed_elapsed_ns=10_000_000, open_elapsed_ns=100_000_000,
                 post_close_rss_kib=10_000, orphan_windows=0, orphan_writer_leases=0):
        self._clock = 1
        self._timed = timed_elapsed_ns
        self._open = open_elapsed_ns
        self._rss = post_close_rss_kib
        self._orphan_windows = orphan_windows
        self._orphan_leases = orphan_writer_leases
        self._owned = [{"identity": "window/main", "owner": "datum-gui"}]
        self._leases = []

    def _tick(self):
        start = self._clock
        self._clock += 1_000
        return start

    def interact_and_present(self, *, variant, coordinate, trial_index, phase, index):
        start = self._tick()
        return {"exit_code": 0, "timed_out": False, "started_monotonic_ns": start,
                "elapsed_ns": self._timed, "rss_kib": 20_000,
                "presented_elapsed_ns": self._timed + 2_000_000}  # present AFTER interaction

    def open_and_present(self, *, coordinate, trial_index, phase, index):
        start = self._tick()
        return {"exit_code": 0, "timed_out": False, "started_monotonic_ns": start,
                "elapsed_ns": self._open, "rss_kib": 40_000,
                "presented_elapsed_ns": self._open - 5_000_000}  # present WITHIN the open interval

    def open_close_cycle(self, *, coordinate, trial_index, phase, index):
        start = self._tick()
        return {"exit_code": 0, "timed_out": False, "started_monotonic_ns": start,
                "post_close_rss_kib": self._rss, "orphan_windows": self._orphan_windows,
                "orphan_writer_leases": self._orphan_leases,
                "owned_windows": list(self._owned), "writer_leases": list(self._leases)}


class GuiMeasurementProducerTests(unittest.TestCase):
    def setUp(self):
        # FakeGuiDrive exercises mechanics; it supplies no Git/product proof.
        guard = patch.object(gm, "validate_live_inputs")
        self.live_guard = guard.start()
        self.addCleanup(guard.stop)
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
        self.gui_sha = frozen_binaries["gui"]["sha256"]

    def build(self, *, drive=None, budgets=gm.GUI_BUDGETS):
        return gm.build_gui_measurements(self.bundle_root, self.identity,
                                         drive=drive or FakeGuiDrive(), coordinates=[COORDINATE],
                                         budgets=budgets, producer_session="dev-agent-01")

    def test_changed_inputs_refuse_before_gui_drive(self):
        self.live_guard.side_effect = EvidenceError("current proof input changed")
        with patch.object(FakeGuiDrive, "interact_and_present") as feedback:
            with self.assertRaisesRegex(EvidenceError, "current proof input changed"):
                self.build()
            feedback.assert_not_called()

    def test_changed_inputs_during_gui_drive_refuse_report(self):
        self.live_guard.side_effect = [None, EvidenceError("current proof input changed")]
        with self.assertRaisesRegex(EvidenceError, "current proof input changed"):
            self.build(budgets={"gui-feedback"})

    # --- coordinate -> frozen environment id --------------------------------------------------
    def test_coordinate_environment_id(self):
        self.assertEqual(gm.coordinate_environment_id(COORDINATE), NATIVE_ID)
        self.assertIn(NATIVE_ID, {e["id"] for e in self.identity["environments"]})

    # --- frozen-identity consumption: gui role + native env -----------------------------------
    def test_rows_bind_gui_role_and_native_environment(self):
        report = self.build(budgets={"gui-feedback"})
        self.assertEqual(len(report["measurements"]), 6)  # six gui-feedback variants, one coordinate
        for row in report["measurements"]:
            self.assertEqual(row["binary_sha256"], self.gui_sha)
            self.assertEqual(row["environment_id"], NATIVE_ID)
            self.assertEqual(row["candidate_revision"], self.candidate["revision"])

    # --- the three budgets' seal shapes pass the governed validator ---------------------------
    def test_gui_feedback_seal_passes_validator(self):
        report = self.build(budgets={"gui-feedback"})
        bundle = Bundle(self.bundle_root, self._inventory())
        for row in report["measurements"]:
            meas.validate_trials(row, bundle)  # must not raise
            self.assertTrue(all(t["cold_open"] is None and t["baseline_rss_kib"] is None
                                for t in row["trials"]))

    def test_native_window_open_cold_open(self):
        report = self.build(budgets={"native-window-open"})
        (row,) = report["measurements"]
        self.assertEqual(row["variant_id"], "open")
        for trial in row["trials"]:
            self.assertEqual(trial["cold_open"]["index"], 0)
        meas.validate_trials(row, Bundle(self.bundle_root, self._inventory()))  # must not raise

    def test_window_lifecycle_baseline_is_final_warmup(self):
        report = self.build(budgets={"window-lifecycle"})
        (row,) = report["measurements"]
        for trial in row["trials"]:
            self.assertEqual(trial["baseline_rss_kib"], trial["warmups"][-1]["post_close_rss_kib"])
            self.assertEqual(trial["baseline_resources"]["owned_windows"],
                             trial["warmups"][-1]["owned_windows"])
        meas.validate_trials(row, Bundle(self.bundle_root, self._inventory()))  # must not raise

    # --- one-attempt preserve-on-fail: the governed validator raises and the run stops --------
    def test_over_budget_timed_drive_raises(self):
        slow = FakeGuiDrive(timed_elapsed_ns=120_000_000)  # 120 ms > gui-feedback 100 ms max
        with self.assertRaises(EvidenceError):
            self.build(drive=slow, budgets={"gui-feedback"})

    def test_leaking_lifecycle_drive_raises(self):
        leaking = FakeGuiDrive(orphan_windows=1)  # an orphaned window must fail the governed validator
        with self.assertRaises(EvidenceError):
            self.build(drive=leaking, budgets={"window-lifecycle"})

    # --- the full standalone report stays non-ready -------------------------------------------
    def test_full_report_is_non_ready(self):
        report = self.build()
        self.assertEqual(len(report["measurements"]), 8)  # 6 + 1 + 1 for one coordinate
        self.assertNotEqual(report.get("status"), "ready")

    # --- the explicit cross-corpus merge step's refusals on real row shapes -------------------
    def _row(self, budget_id, variant_id, **overrides):
        from global_preferences_acceptance.measurement_assembly import measurement_role
        role = measurement_role(budget_id)
        base = {"budget_id": budget_id, "variant_id": variant_id,
                "environment_id": NATIVE_ID if role == "gui" else "headless-linux-x86_64",
                "candidate_revision": self.candidate["revision"],
                "binary_sha256": self.candidate["binaries"][role]["sha256"],
                "input_manifest_sha256": self.manifest["sha256"],
                "matrix_sha256": "d" * 64, "trials": []}
        base.update(overrides)
        return base

    def test_merge_refuses_duplicate_cross_corpus_key(self):
        gui = {"measurements": [self._row("gui-feedback", "focus")]}
        duplicate = {"measurements": [self._row("gui-feedback", "focus")]}
        with self.assertRaises(EvidenceError):
            gm.merge_measurement_corpus(self.bundle_root, self.identity, [gui, duplicate])

    def test_merge_refuses_identity_mismatch(self):
        headless = {"measurements": [self._row("release-query", "get/defaults")]}
        gui = {"measurements": [self._row("gui-feedback", "focus", matrix_sha256="e" * 64)]}
        with self.assertRaises(EvidenceError):
            gm.merge_measurement_corpus(self.bundle_root, self.identity, [headless, gui])

    def _inventory(self):
        return [{"path": "command-capture/" + p.name, "sha256": p.name}
                for p in (self.bundle_root / "command-capture").iterdir()]


if __name__ == "__main__":
    unittest.main()
