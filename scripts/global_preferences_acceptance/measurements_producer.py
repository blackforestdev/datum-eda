#!/usr/bin/env python3
"""GP-CM05E raw-sample measurement PRODUCER (slice S2).

Drives the real release product surface and assembles raw timing/RSS samples into
the ``measurements.validate_trials`` schema (3 trials x 10 warm-ups + 100 samples,
each a typed sample plus a datum.preferences.measurement-capture.v2 evidence
record; one datum.preferences.measurement-trial.v2 log per trial). It self-
validates each produced measurement with the governed validator.

S2 covers the HEADLESS/CLI budgets. durable-mutation /daemon is S3; the GUI budgets
(gui-feedback, native-window-open, window-lifecycle) are S4. This module currently
implements the timed-query path and proves it on release-query/describe/defaults;
the remaining headless variants reuse the same assembler.

No retries-until-green: a failed or over-budget sample is preserved in its evidence
and surfaced by the validator; it is never re-run to obtain a passing value.
"""
import argparse
import json
import os
import sys
from pathlib import Path

HERE = Path(__file__).resolve()
ROOT = HERE.parents[2]
sys.path.insert(0, str(HERE.parents[1]))

from global_preferences_acceptance import measurements as meas  # noqa: E402
from global_preferences_acceptance import observations as obs  # noqa: E402
from global_preferences_acceptance.capture import Capture  # noqa: E402
from global_preferences_acceptance.inputs import Bundle, canonical, digest, parse  # noqa: E402
from global_preferences_acceptance.process import execute  # noqa: E402

MATRIX_RELPATH = "specs/global_preferences_production_acceptance_matrix.json"
CAPTURE_SCHEMA = "datum.preferences.measurement-capture.v2"
TRIAL_SCHEMA = "datum.preferences.measurement-trial.v2"
WARMUPS = 10
SAMPLES = 100

# Timed headless query variants (release-query budget): verb -> CLI argv tail.
RELEASE_QUERY_ARGV = {
    "describe": ["preferences", "describe"],
    "list": ["preferences", "list"],
    "get": ["preferences", "get", "datum.accessibility.reduced_motion"],
    "search": ["preferences", "search", "motion"],
    "explain": ["preferences", "explain", "datum.accessibility.reduced_motion"],
    "preview-project-units-seed": ["preferences", "preview-project-units-seed", "--source", "global"],
}


def _sha_file(path):
    import hashlib
    return hashlib.sha256(Path(path).read_bytes()).hexdigest()


def _isolated_env(config_root):
    env = {k: v for k, v in os.environ.items()
           if k not in {"XDG_CONFIG_HOME", "DATUM_GUI_PREFERENCES_PATH",
                        "DATUM_ENGINE_SOCKET", "EDA_ENGINE_SOCKET"}}
    env["XDG_CONFIG_HOME"] = str(config_root)
    return env


def _timed_sample(capture, identity, trial_index, phase, index, binary, argv, env, timeout):
    run = execute(Path(binary), ["--format", "json", *argv], env, timeout=timeout)
    ok = run["exit_code"] == 0 and not run["timed_out"]
    sample = {
        "kind": "timed",
        "index": index,
        "started_monotonic_ns": run["started_monotonic_ns"],
        "elapsed_ns": run["elapsed_ns"],
        "rss_kib": run["rss_kib"],
        "daemon_rss_kib": None,
        "launch_to_response_ns": None,
        "presented_elapsed_ns": None,
        "outcome": "success" if ok else "failure",
    }
    record = {"schema": CAPTURE_SCHEMA, "measurement": identity, "trial_index": trial_index,
              "phase": phase, "exit_code": run["exit_code"], "timed_out": run["timed_out"],
              "observation": dict(sample)}
    sample["evidence"] = capture.put(record)
    return sample


def build_timed_measurement(capture, *, budget_id, variant_id, argv, binary,
                            candidate_revision, binary_sha256, input_manifest_sha256,
                            matrix_sha256, environment_id, timeout, scratch):
    identity = {"budget_id": budget_id, "variant_id": variant_id, "environment_id": environment_id,
                "candidate_revision": candidate_revision, "binary_sha256": binary_sha256,
                "input_manifest_sha256": input_manifest_sha256, "matrix_sha256": matrix_sha256}
    trials = []
    for trial_index in range(3):
        config_root = scratch / f"{budget_id}-{variant_id.replace('/', '_')}-trial{trial_index}"
        config_root.mkdir(parents=True, exist_ok=True)
        env = _isolated_env(config_root)
        warmups = [_timed_sample(capture, identity, trial_index, "warmups", i, binary, argv, env, timeout)
                   for i in range(WARMUPS)]
        samples = [_timed_sample(capture, identity, trial_index, "samples", i, binary, argv, env, timeout)
                   for i in range(SAMPLES)]
        trial = {"index": trial_index, "warmup_count": WARMUPS, "warmups": warmups,
                 "baseline_rss_kib": None, "baseline_resources": None, "cold_open": None,
                 "samples": samples}
        trial_log = {**trial, "schema": TRIAL_SCHEMA, "measurement": identity}
        trial["log"] = capture.put(trial_log)
        trials.append(trial)
    return {**identity, "trials": trials}


def main(argv=None):
    import development_workspace as workspace
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--bundle-root", required=True, type=Path)
    parser.add_argument("--binary", type=Path, default=ROOT / "target/release/datum-eda")
    parser.add_argument("--variant", default="all",
                        help="release-query <verb>/defaults, or 'all' for every defaults verb (S2 headless timed subset)")
    parser.add_argument("--timeout", type=float, default=15)
    parser.add_argument("--scratch", type=Path, default=workspace.scratch("gp-cm05e-measurement-roots"))
    args = parser.parse_args(argv)

    binary = args.binary.resolve()
    if not (binary.is_file() and os.access(binary, os.X_OK)):
        parser.error("release binary must be executable")
    if args.variant == "all":
        verbs = list(RELEASE_QUERY_ARGV)
    else:
        verb, _, store = args.variant.partition("/")
        if verb not in RELEASE_QUERY_ARGV or store != "defaults":
            parser.error("release-query variant must be <verb>/defaults or 'all'")
        verbs = [verb]

    args.bundle_root.mkdir(parents=True, exist_ok=True)
    scratch = args.scratch.resolve()
    scratch.mkdir(parents=True, exist_ok=True)
    capture = Capture(args.bundle_root.resolve())

    import subprocess
    revision = subprocess.run(["git", "-C", str(ROOT), "rev-parse", "HEAD"],
                              capture_output=True, text=True, check=True).stdout.strip()
    binary_sha256 = _sha_file(binary)
    matrix_sha256 = digest(canonical(parse((ROOT / MATRIX_RELPATH).read_bytes(), "matrix")))
    input_manifest_sha256 = digest(canonical([{"path": str(binary), "sha256": binary_sha256}]))
    environment_id = "headless-linux-x86_64"

    produced = []
    for verb in verbs:
        produced.append(build_timed_measurement(
            capture, budget_id="release-query", variant_id=f"{verb}/defaults",
            argv=RELEASE_QUERY_ARGV[verb], binary=binary, candidate_revision=revision,
            binary_sha256=binary_sha256, input_manifest_sha256=input_manifest_sha256,
            matrix_sha256=matrix_sha256, environment_id=environment_id,
            timeout=args.timeout, scratch=scratch))

    bundle = Bundle(args.bundle_root.resolve(), list(capture.inventory.values()))
    for measurement in produced:
        try:
            meas.validate_trials(measurement, bundle)
        except Exception as error:  # preserve + diagnose; never re-roll
            print(f"MEASUREMENT VALIDATION FAILED ({measurement['variant_id']}, preserved): {error}",
                  file=sys.stderr)
            return 1
        p95 = [meas.percentile([s["elapsed_ns"] for s in t["samples"]]) for t in measurement["trials"]]
        print(f"release-query {measurement['variant_id']}: validate_trials ACCEPTED; "
              f"per-trial p95 ns = {p95} (budget p95 <= 50ms)")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
