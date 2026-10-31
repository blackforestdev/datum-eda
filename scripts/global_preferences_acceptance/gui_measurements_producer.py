"""GP-CM05E Unit M: the S4/GUI measurement producer (gui-feedback / native-window-open /
window-lifecycle) -- the U4 analog for the three GUI budgets excluded at
``measurements_producer.py`` line 17-18.

This is the TOOL only: frozen-identity consumption, per-budget orchestration, the ratified
sample/trial seal (reused verbatim from :mod:`measurements_producer`), the governed
:func:`measurements.validate_trials`, and a standalone merge step. The RUNNER (frontend) supplies a
:class:`GuiDrive` -- the owned-Weston launch, the interaction, the present-watch wiring and the
``/proc`` accounting on its own envelope/display -- so this module is blind to the display and the
envelope (the :mod:`gui_producer` parameterization precedent). The live drive is never exercised
here; the focused tests inject a synthetic GuiDrive.

Per-corpus evidence stays separate: :func:`build_gui_measurements` emits a standalone GUI measurement
report, and :func:`merge_measurement_corpus` is the explicit governed merge with the headless corpus
(dup / identity / env / binary refusals via :func:`measurement_assembly.merge_measurements`), so the
26-headless and 64-GUI corpora remain independently auditable until compose consumes the merged 90.
"""
from pathlib import Path

from . import measurements as meas
from . import measurements_producer as mp
from .assemble import assembled_report, load_frozen
from .capture import Capture
from .environment_record import _scale_label
from .inputs import Bundle
from .inventory import TIMED_VARIANTS
from .matrix import gui_coordinates
from .measurement_assembly import GUI_BUDGETS, frozen_common, merge_measurements

TRIALS = 3


def coordinate_environment_id(coordinate):
    """The frozen 21-field environments id for a GUI display coordinate, built by the one governed
    construction in :func:`environment_record.native_environment` (never a parallel formula)."""
    backend, scale, window = coordinate
    return f"native-{backend}-{_scale_label(scale)}-{window[0]}x{window[1]}"


class GuiDrive:
    """Runner-supplied drive. The tool calls one method per execution (warm-up, sample or cold-open);
    the runner owns the candidate-gui launch, the interaction, the present-watch (PresentCompleteNotify
    primary, image-poll fallback) and the ``/proc`` + window/lease accounting on its envelope/display.

    Each method returns a plain result dict. Timed results (gui-feedback, native-window-open) carry
    ``exit_code``, ``timed_out``, ``started_monotonic_ns``, ``elapsed_ns``, ``rss_kib`` and
    ``presented_elapsed_ns``. Lifecycle results (window-lifecycle) carry ``exit_code``, ``timed_out``,
    ``started_monotonic_ns``, ``post_close_rss_kib``, ``orphan_windows``, ``orphan_writer_leases`` and
    the ``owned_windows`` / ``writer_leases`` resource lists ([{"identity", "owner"}, ...]).
    ``started_monotonic_ns`` must be strictly increasing across every execution of a coordinate."""

    def interact_and_present(self, *, variant, coordinate, trial_index, phase, index):
        raise NotImplementedError  # gui-feedback

    def open_and_present(self, *, coordinate, trial_index, phase, index):
        raise NotImplementedError  # native-window-open (phase "cold_open" for the cold launch)

    def open_close_cycle(self, *, coordinate, trial_index, phase, index):
        raise NotImplementedError  # window-lifecycle


def _timed_row(capture, identity, trial_index, phase, index, result):
    """Seal one timed GUI sample: reuse the ratified timed observation, add the presented frame."""
    observation = mp._timed_observation(result)
    observation["presented_elapsed_ns"] = result["presented_elapsed_ns"]
    return mp._retain(capture, identity, trial_index, phase, index, result, observation)


def _lifecycle_row(capture, identity, trial_index, phase, index, result):
    """Seal one window-lifecycle sample (open/close cycle -> post-close RSS + orphan/resource state)."""
    ok = result["exit_code"] == 0 and not result["timed_out"]
    observation = {"kind": "lifecycle",
                   "started_monotonic_ns": result["started_monotonic_ns"],
                   "post_close_rss_kib": result["post_close_rss_kib"],
                   "orphan_windows": result["orphan_windows"],
                   "orphan_writer_leases": result["orphan_writer_leases"],
                   "owned_windows": result["owned_windows"],
                   "writer_leases": result["writer_leases"],
                   "outcome": "success" if ok else "failure"}
    return mp._retain(capture, identity, trial_index, phase, index, result, observation)


def _seal_trial(capture, identity, trial_index, warmups, samples, *, baseline_rss_kib=None,
                baseline_resources=None, cold_open=None):
    """Build and seal one measurement-trial.v2 envelope with GUI-specific baseline/cold-open fields."""
    trial = {"index": trial_index, "warmup_count": mp.WARMUPS, "warmups": warmups,
             "baseline_rss_kib": baseline_rss_kib, "baseline_resources": baseline_resources,
             "cold_open": cold_open, "samples": samples}
    trial_log = {**trial, "schema": mp.TRIAL_SCHEMA, "measurement": identity}
    trial["log"] = capture.put(trial_log)
    return trial


def _build_gui_feedback(capture, common, coordinate, *, variant, drive):
    identity = {"budget_id": "gui-feedback", "variant_id": variant, **common}
    trials = []
    for trial_index in range(TRIALS):
        warmups = [_timed_row(capture, identity, trial_index, "warmups", i,
                              drive.interact_and_present(variant=variant, coordinate=coordinate,
                                                         trial_index=trial_index, phase="warmups", index=i))
                   for i in range(mp.WARMUPS)]
        samples = [_timed_row(capture, identity, trial_index, "samples", i,
                              drive.interact_and_present(variant=variant, coordinate=coordinate,
                                                         trial_index=trial_index, phase="samples", index=i))
                   for i in range(mp.SAMPLES)]
        trials.append(_seal_trial(capture, identity, trial_index, warmups, samples))
    return {**identity, "trials": trials}


def _build_native_window_open(capture, common, coordinate, *, drive):
    identity = {"budget_id": "native-window-open", "variant_id": "open", **common}
    trials = []
    for trial_index in range(TRIALS):
        cold_open = _timed_row(capture, identity, trial_index, "cold_open", 0,
                               drive.open_and_present(coordinate=coordinate, trial_index=trial_index,
                                                      phase="cold_open", index=0))
        warmups = [_timed_row(capture, identity, trial_index, "warmups", i,
                              drive.open_and_present(coordinate=coordinate, trial_index=trial_index,
                                                     phase="warmups", index=i))
                   for i in range(mp.WARMUPS)]
        samples = [_timed_row(capture, identity, trial_index, "samples", i,
                              drive.open_and_present(coordinate=coordinate, trial_index=trial_index,
                                                     phase="samples", index=i))
                   for i in range(mp.SAMPLES)]
        trials.append(_seal_trial(capture, identity, trial_index, warmups, samples, cold_open=cold_open))
    return {**identity, "trials": trials}


def _build_window_lifecycle(capture, common, coordinate, *, drive):
    identity = {"budget_id": "window-lifecycle", "variant_id": "cycles", **common}
    trials = []
    for trial_index in range(TRIALS):
        warmups = [_lifecycle_row(capture, identity, trial_index, "warmups", i,
                                  drive.open_close_cycle(coordinate=coordinate, trial_index=trial_index,
                                                         phase="warmups", index=i))
                   for i in range(mp.WARMUPS)]
        samples = [_lifecycle_row(capture, identity, trial_index, "samples", i,
                                  drive.open_close_cycle(coordinate=coordinate, trial_index=trial_index,
                                                         phase="samples", index=i))
                   for i in range(mp.SAMPLES)]
        # The warmed baseline is the final warm-up observation -- the governed validator requires the
        # trial baseline to equal warmups[-1]'s post-close RSS and resources (no separately-claimed
        # baseline); the drive must warm the process before the first warm-up it reports here.
        last = warmups[-1]
        trials.append(_seal_trial(capture, identity, trial_index, warmups, samples,
                                  baseline_rss_kib=last["post_close_rss_kib"],
                                  baseline_resources={"owned_windows": last["owned_windows"],
                                                      "writer_leases": last["writer_leases"]}))
    return {**identity, "trials": trials}


def build_gui_measurements(bundle_root, identity, *, drive, coordinates=None, budgets=GUI_BUDGETS,
                           validate=True, producer_session="dev-agent-01"):
    """Drive the selected GUI budgets for the supplied display coordinates under the ONE frozen
    identity into the shared content-addressed bundle and return a standalone GUI measurement report.

    ``coordinates`` defaults to the full ``gui_coordinates()`` matrix; the runner passes the subset
    its envelope can capture (coordinates it cannot reach stay OWED and are disclosed at compose, like
    the OWED cases). Each coordinate binds its ``native-*`` frozen environment id via
    :func:`frozen_common` (role ``gui``). Every produced row is replayed through the governed
    :func:`measurements.validate_trials`; a failed/over-budget row raises and the run stops with the
    evidence preserved (one-attempt; never re-rolled). No readiness is issued."""
    bundle_root = Path(bundle_root).resolve()
    bundle_root.mkdir(parents=True, exist_ok=True)
    capture = Capture(bundle_root)
    coordinates = tuple(coordinates) if coordinates is not None else gui_coordinates()
    produced = []
    for coordinate in coordinates:
        common = frozen_common(identity, role="gui",
                               environment_id=coordinate_environment_id(coordinate))
        if "gui-feedback" in budgets:
            for variant in TIMED_VARIANTS["gui-feedback"]:
                produced.append(_build_gui_feedback(capture, common, coordinate, variant=variant, drive=drive))
        if "native-window-open" in budgets:
            produced.append(_build_native_window_open(capture, common, coordinate, drive=drive))
        if "window-lifecycle" in budgets:
            produced.append(_build_window_lifecycle(capture, common, coordinate, drive=drive))
    if validate:
        bundle = Bundle(bundle_root, list(capture.inventory.values()))
        for measurement in produced:
            meas.validate_trials(measurement, bundle)
    rows = merge_measurements(identity, produced)
    return assembled_report(capture, identity, case_results=(), measurements=rows,
                            producer_session=producer_session)


def merge_measurement_corpus(bundle_root, identity, reports, *, producer_session="dev-agent-01"):
    """The explicit governed merge step: combine the measurement rows of several per-corpus reports
    (the 26-headless + the 64-GUI) over the ONE shared bundle into the full measurement report compose
    consumes. :func:`measurement_assembly.merge_measurements` re-asserts the frozen identity and
    refuses a duplicate ``(budget_id, variant_id, environment_id)``, an identity/env mismatch, or a
    row whose binary is not the frozen candidate binary for its surface role -- the cross-corpus seam
    audit probes. Both inputs are retained unchanged as separate evidence."""
    bundle_root = Path(bundle_root).resolve()
    rows = []
    for report in reports:
        rows.extend(report["measurements"])
    merged = merge_measurements(identity, rows)
    inventory = [{"path": "command-capture/" + p.name, "sha256": p.name}
                 for p in (bundle_root / "command-capture").iterdir()]
    capture = Capture(bundle_root, inventory=inventory)
    return assembled_report(capture, identity, case_results=(), measurements=merged,
                            producer_session=producer_session)


def main(argv=None):
    """CLI for the drive-free merge step (the live GUI capture is the runner importing
    :func:`build_gui_measurements` with its own GuiDrive)."""
    import argparse
    import json

    parser = argparse.ArgumentParser(description=__doc__)
    sub = parser.add_subparsers(dest="command", required=True)
    merge = sub.add_parser("merge", help="merge per-corpus measurement reports into the full corpus")
    merge.add_argument("--frozen", required=True, type=Path, help="the frozen identity baton")
    merge.add_argument("--bundle-root", required=True, type=Path, help="the shared content-addressed bundle")
    merge.add_argument("--report", required=True, action="append", type=Path,
                       help="a per-corpus measurement report (repeat for headless + gui)")
    merge.add_argument("--out", required=True, type=Path, help="merged measurement report output")
    args = parser.parse_args(argv)

    identity = load_frozen(str(args.frozen))
    reports = [json.loads(path.read_text()) for path in args.report]
    merged = merge_measurement_corpus(args.bundle_root, identity, reports)
    args.out.write_text(json.dumps(merged, indent=2, sort_keys=True))
    print(f"merged {sum(len(r['measurements']) for r in reports)} rows -> "
          f"{len(merged['measurements'])} measurement variants -> {args.out}")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
