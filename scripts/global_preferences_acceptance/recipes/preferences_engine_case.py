#!/usr/bin/env python3
"""Committed GP-CM05E ENGINE-surface case recipe: run the engine-surface
acceptance driver for one coverage row and emit a
``datum.preferences.case-observations.v2`` record.

Some resolver states — managed provider generation states — are reachable only
inside the engine; no CLI/MCP/GUI adapter can inject them (acceptance contract:
managed fixtures cannot activate a managed provider). This recipe is the engine
surface's committed entrypoint, disjoint from the CLI recipe
(``preferences_case.py``) so the CLI, engine and GUI surfaces evolve in separate
lanes. The validator binds each engine case to exactly this invocation:

    python3 scripts/global_preferences_acceptance/recipes/preferences_engine_case.py \
        --fixture-json <canonical frozen fixture JSON>

The pinned engine binary (``candidate.binaries["engine"]``) is provided
out-of-band so the committed argv stays reproducible; replay supplies the match:

    DATUM_EDA_ENGINE  -> the candidate engine acceptance driver executable

The engine driver injects a TEST PRECONDITION (a pure resolver evaluation; no
store is written) and prints the honest ``{assertion_id: bool}`` dict. This
recipe wraps it UNCHANGED into the case-observations.v2 record and always exits 0
on a completed run: a false observation is recorded as data, not a command
failure, so the validator adjudicates it. Observations never retry to green.
"""
import argparse
import json
import os
import subprocess
import sys
import time
from pathlib import Path

SCHEMA = "datum.preferences.case-observations.v2"

# Engine-surface coverage the driver implements. Managed-offline is a pure
# resolver read (no store write); its per-state variants resolve an authenticated
# managed Pin whose generation is inactive and observe it never participates.
ENGINE_COVERAGE = {
    ("managed-offline", "inactive"),
    ("managed-offline", "expired"),
    ("managed-offline", "unavailable"),
    ("managed-offline", "offline"),
    # Write family: the driver's observe phase runs the recovery operation against
    # the store the producer's engine setup phase seeded (via DATUM_EDA_WORK_ROOT).
    ("backup-restore", "restore"),
    ("backup-restore", "exact-backup"),
    ("backup-restore", "reverse-restore"),
    ("backup-restore", "stale-preview"),
    ("backup-restore", "incomplete-backup"),
}


def _require_binary(variable):
    value = os.environ.get(variable)
    if not value or not Path(value).is_file():
        raise SystemExit(f"{variable} must point at the candidate binary for this surface")
    return value


def observe(fixture):
    """Invoke the engine acceptance driver for the fixture's coverage row and return its honest
    observation dict (the driver emits only the dict; this recipe supplies the schema wrapper)."""
    case, variant, _subcase, surface, _coordinate = fixture["parameters"]["coverage"]
    if surface != "engine":
        raise SystemExit(f"engine recipe invoked for non-engine surface: {surface}")
    if (case, variant) not in ENGINE_COVERAGE:
        raise SystemExit(f"engine recipe family not implemented: {case}/{variant}")
    binary = _require_binary("DATUM_EDA_ENGINE")
    proc = subprocess.run([binary, "--case", case, "--variant", variant],
                          capture_output=True, text=True)
    if proc.returncode != 0:
        raise SystemExit(f"engine acceptance driver failed ({case}/{variant}): {proc.stderr.strip()}")
    try:
        observations = json.loads(proc.stdout)
    except json.JSONDecodeError as error:
        raise SystemExit(f"engine driver emitted non-JSON observations ({case}/{variant}): {error}")
    if not isinstance(observations, dict) or not observations:
        raise SystemExit(f"engine driver observations are not a non-empty object ({case}/{variant})")
    if not all(isinstance(v, bool) for v in observations.values()):
        raise SystemExit(f"engine driver observations are not all booleans ({case}/{variant})")
    return observations


def main(argv=None):
    parser = argparse.ArgumentParser(description="GP-CM05E engine-surface case recipe")
    parser.add_argument("--fixture-json", required=True)
    args = parser.parse_args(argv)
    fixture = json.loads(args.fixture_json)
    start = time.monotonic()
    observations = observe(fixture)
    elapsed = time.monotonic() - start
    # A count the capture layer parses for executed_tests; not a pass/fail claim.
    print(f"Ran {len(observations)} tests in {elapsed:.3f}s", file=sys.stderr)
    sys.stdout.write(
        json.dumps(
            {
                "schema": SCHEMA,
                "executed_tests": len(observations),
                "observations": observations,
            },
            sort_keys=True,
            separators=(",", ":"),
            ensure_ascii=False,
        )
    )
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
