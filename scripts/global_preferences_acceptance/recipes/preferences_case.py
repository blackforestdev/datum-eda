#!/usr/bin/env python3
"""Committed GP-CM05E case recipe: run the real product surface for one coverage
row and emit a ``datum.preferences.case-observations.v2`` record.

The validator (``scripts/global_preferences_acceptance/observations.py``) binds
each case to exactly this invocation and replays it:

    python3 scripts/global_preferences_acceptance/recipes/preferences_case.py \
        --fixture-json <canonical frozen fixture JSON>

Every parameter is carried in that single fixture JSON; no wrapper, dropped
parameter or alternative recipe is accepted. The surface binary the report pins
(``candidate.binaries[role]``) is provided out-of-band through the environment so
the committed argv stays reproducible; replay supplies the matching build:

    DATUM_EDA_CLI  -> the candidate ``datum-eda`` CLI executable (cli surface)

The recipe reports honest observations and always exits 0 on a completed run: a
false observation is recorded as data, not turned into a command failure, so the
validator (not the producer) adjudicates it. Observations never retry to green.
"""
import argparse
import json
import os
import shutil
import subprocess
import sys
import tempfile
import time
from pathlib import Path

SCHEMA = "datum.preferences.case-observations.v2"


def _require_binary(variable):
    value = os.environ.get(variable)
    if not value or not Path(value).is_file():
        raise SystemExit(f"{variable} must point at the candidate binary for this surface")
    return value


def _run_cli(binary, args, *, work):
    """Run the real CLI against an isolated config root; capture typed JSON."""
    env = {
        **os.environ,
        "HOME": str(work),
        "XDG_CONFIG_HOME": str(work / "config"),
    }
    env.pop("DATUM_GUI_PREFERENCES_PATH", None)
    return subprocess.run(
        [binary, "--format", "json", *args],
        capture_output=True,
        text=True,
        env=env,
    )


def _no_project_written(work):
    """A read-only query writes no native Project anywhere under the work root."""
    return not any(path.name == "project.json" for path in work.rglob("*"))


def clean_install_missing_store_cli():
    """Fresh install, no Global repository: the CLI must report factory defaults,
    create no repository, and write no Project."""
    binary = _require_binary("DATUM_EDA_CLI")
    work = Path(tempfile.mkdtemp(prefix="gp-cm05e-case-clean-install-"))
    try:
        proc = _run_cli(binary, ["preferences", "describe"], work=work)
        repository = work / "config" / "datum" / "preferences"
        no_repository = not repository.exists()
        no_project = _no_project_written(work)
        factory_defaults = False
        if proc.returncode == 0:
            try:
                report = json.loads(proc.stdout)
            except json.JSONDecodeError:
                report = None
            if isinstance(report, dict):
                context = report.get("context") or {}
                factory_defaults = bool(
                    report.get("ok") is True
                    and context.get("repository_status") == "defaults_only"
                    and context.get("generation") is None
                    and context.get("active_descriptor_count") == 11
                    and context.get("reserved_descriptor_count") == 45
                )
        return {
            "factory-defaults": factory_defaults,
            "no-repository-created": bool(no_repository),
            "no-project-write": bool(no_project),
        }
    finally:
        shutil.rmtree(work, ignore_errors=True)


# (case, variant, surface) -> observation producer. Later GP-CM05E slices extend
# this map; each family runs the real product surface and records honest results.
DISPATCH = {
    ("clean-install", "missing-store", "cli"): clean_install_missing_store_cli,
}


def observe(fixture):
    coverage = fixture["parameters"]["coverage"]
    case, variant, _subcase, surface, _coordinate = coverage
    handler = DISPATCH.get((case, variant, surface))
    if handler is None:
        raise SystemExit(f"recipe family not implemented: {case}/{variant}/{surface}")
    return handler()


def main(argv=None):
    parser = argparse.ArgumentParser(description="GP-CM05E case recipe")
    parser.add_argument("--fixture-json", required=True)
    args = parser.parse_args(argv)
    fixture = json.loads(args.fixture_json)
    start = time.monotonic()
    observations = observe(fixture)
    elapsed = time.monotonic() - start
    # Real observational checks completed; report the count the capture layer
    # parses for executed_tests. This is a count, not a pass/fail claim.
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
