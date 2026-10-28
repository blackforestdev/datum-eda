#!/usr/bin/env python3
"""GP-CM05E production-execution case PRODUCER (slice S1 pattern).

Runs a committed case recipe against the real product surface, captures the run
into a content-addressed bundle, assembles the ``datum-global-preferences-
production-evidence-v2`` case_result row, and self-validates it with the governed
per-case validators in ``observations.py``. This is the producer half the
acceptance contract (sections 10-11) obligates; the validator was always designed
to receive it. S1 establishes the pattern on the simplest family
(``clean-install/missing-store/cli``) before the other families copy it.

No retries-until-green: a false or failed observation is preserved and reported,
never re-rolled. The matrix is specification, not a target to massage.
"""
import argparse
import hashlib
import json
import os
import subprocess
import sys
from pathlib import Path

HERE = Path(__file__).resolve()
ROOT = HERE.parents[2]
sys.path.insert(0, str(HERE.parents[1]))  # put scripts/ on the path for the package

from global_preferences_acceptance import observations as obs  # noqa: E402
from global_preferences_acceptance.capture import Capture  # noqa: E402
from global_preferences_acceptance.inputs import Bundle, canonical, digest, parse, read_file  # noqa: E402
from global_preferences_acceptance.matrix import case_inventory  # noqa: E402

RECIPE_RELPATH = "scripts/global_preferences_acceptance/recipes/preferences_case.py"
MATRIX_RELPATH = "specs/global_preferences_production_acceptance_matrix.json"
STATE_SCHEMA = "datum.preferences.state.v2"
FIXTURE_SCHEMA = "datum.preferences.fixture.v2"
ASSERTION_SCHEMA = "datum.preferences.assertion.v2"

# S1 target coverage row (non-GUI, non-daemon): simplest pattern-establishing case.
S1_KEY = ("clean-install", "missing-store", "default", "cli", None)


def _sha_file(path):
    return hashlib.sha256(Path(path).read_bytes()).hexdigest()


def _git_head():
    out = subprocess.run(["git", "-C", str(ROOT), "rev-parse", "HEAD"],
                         capture_output=True, text=True, check=True)
    return out.stdout.strip()


def _read_artifact(capture, reference):
    return read_file(capture.root, reference["path"])


def produce_case(capture, *, key, candidate, binaries, environments,
                 input_manifest, matrix_sha256, surface_binaries):
    """Build one case_result row by running the committed recipe for its surface."""
    case, variant, subcase, surface, _coordinate = key
    required = case_inventory()[key]  # ordered assertion ids for this coverage row

    # Missing-store starts from (and ends at) an empty preferences repository.
    state = {"schema": STATE_SCHEMA,
             "root": f"{case}/{variant}/{subcase}/{surface}", "files": []}
    state_ref = capture.put(state)

    recipe_ref = {"path": RECIPE_RELPATH, "sha256": _sha_file(ROOT / RECIPE_RELPATH)}
    fixture = {
        "schema": FIXTURE_SCHEMA,
        "source_revision": candidate["revision"],
        "recipe": recipe_ref,
        "parameters": {"coverage": list(key)},
        "expected_assertions": {aid: {"operator": "equal", "value": True} for aid in required},
        "initial_state": state_ref,
    }
    fixture_ref = capture.put(fixture)

    command = obs.case_command(fixture)
    # Carry the pinned surface binary out-of-band so the committed argv is stable.
    os.environ["DATUM_EDA_CLI"] = surface_binaries["cli"]
    cmd_row = capture.command(command, cwd=ROOT, candidate=candidate["revision"],
                              inputs=input_manifest["sha256"])

    raw = json.loads(_read_artifact(capture, cmd_row["log"]))
    case_output = json.loads(_read_artifact(capture, raw["stdout"]))
    observed = case_output["observations"]

    environment_id = next(iter(environments))
    row = {
        "case_id": case, "variant_id": variant, "subcase_id": subcase, "surface": surface,
        "environment_id": environment_id,
        "binary_sha256": binaries[surface],
        "matrix_sha256": matrix_sha256,
        "fixture": fixture_ref,
        "before": state_ref,
        "after": state_ref,
        "executed_tests": raw["executed_tests"],
        **{k: cmd_row[k] for k in
           ("command", "exit_code", "candidate_revision", "input_manifest_sha256", "log")},
    }

    context = obs.observation_context(row)
    assertion_rows = []
    for aid in required:
        value = observed[aid]
        evidence = capture.put({"schema": ASSERTION_SCHEMA, "context": context,
                                "id": aid, "observed": value})
        assertion_rows.append({"id": aid, "expected": {"operator": "equal", "value": True},
                               "observed": value, "evidence": evidence})
    row["assertions"] = assertion_rows
    return row


def validate_case(bundle_root, report, key, binaries, environments):
    """Replay the governed per-case validators against the produced capture."""
    bundle = Bundle(bundle_root, report["artifacts"])
    row = report["case_results"][0]
    coordinate = obs.identity(row, report, environments, binaries)
    observed_key = (row["case_id"], row["variant_id"], row["subcase_id"], row["surface"], coordinate)
    if observed_key != key:
        raise AssertionError(f"produced coverage {observed_key} != target {key}")
    required = case_inventory()[key]
    frozen = obs.fixture(row["fixture"], bundle, required, report, key)
    if row["command"] != obs.case_command(frozen):
        raise AssertionError("case command does not replay the frozen fixture recipe")
    if row["before"] != frozen["initial_state"]:
        raise AssertionError("case before-state is not the frozen fixture state")
    obs.assertions(row["assertions"], frozen["expected_assertions"], bundle, obs.observation_context(row))
    obs.state_manifest(row["before"], bundle)
    obs.state_manifest(row["after"], bundle)
    obs.command_observation(row, bundle, tests=row["executed_tests"])


def build_s1(bundle_root, *, cli_binary):
    capture = Capture(bundle_root)
    revision = _git_head()
    cli_sha = _sha_file(cli_binary)
    candidate = {"revision": revision,
                 "binaries": {"cli": {"path": str(cli_binary), "sha256": cli_sha}}}
    binaries = {"cli": cli_sha}
    recipe_ref = {"path": RECIPE_RELPATH, "sha256": _sha_file(ROOT / RECIPE_RELPATH)}
    input_manifest_ref = capture.put([recipe_ref])
    # The contract binds matrix_sha256 to the canonical matrix digest (report.py),
    # not the raw file bytes; match it now so the same report assembles under the
    # full validator in later slices.
    matrix_sha256 = digest(canonical(parse((ROOT / MATRIX_RELPATH).read_bytes(), "matrix")))
    environments = {"headless-linux-x86_64": None}

    row = produce_case(capture, key=S1_KEY, candidate=candidate, binaries=binaries,
                       environments=environments, input_manifest=input_manifest_ref,
                       matrix_sha256=matrix_sha256, surface_binaries={"cli": str(cli_binary)})

    report = {
        "origin": {"kind": "product-execution", "producer_session": "dev-agent-02"},
        "candidate": candidate,
        "input_manifest": input_manifest_ref,
        "matrix_sha256": matrix_sha256,
        "environments": environments,
        "artifacts": list(capture.inventory.values()),
        "case_results": [row],
    }
    return report, binaries, environments


def main(argv=None):
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--bundle-root", required=True, type=Path,
                        help="directory to hold the content-addressed capture bundle")
    parser.add_argument("--report", required=True, type=Path,
                        help="path to write the produced case report JSON")
    parser.add_argument("--cli-binary", required=True, type=Path,
                        help="candidate datum-eda CLI executable (cli surface)")
    args = parser.parse_args(argv)
    args.bundle_root.mkdir(parents=True, exist_ok=True)

    report, binaries, environments = build_s1(args.bundle_root.resolve(), cli_binary=args.cli_binary.resolve())
    args.report.write_text(json.dumps(report, indent=2, sort_keys=True))

    observed = report["case_results"][0]["assertions"]
    print("Produced case:", S1_KEY)
    for row in observed:
        print(f"  {row['id']}: observed={row['observed']}")

    try:
        validate_case(args.bundle_root.resolve(), report, S1_KEY, binaries, environments)
    except Exception as error:  # preserve + diagnose; never re-roll to green
        print(f"VALIDATION FAILED (preserved for diagnosis): {error}", file=sys.stderr)
        return 1
    print("S1 capture VALIDATOR-ACCEPTED by the governed per-case validators.")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
