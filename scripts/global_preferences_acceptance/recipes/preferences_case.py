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
import uuid
from pathlib import Path

SCHEMA = "datum.preferences.case-observations.v2"


def _require_binary(variable):
    value = os.environ.get(variable)
    if not value or not Path(value).is_file():
        raise SystemExit(f"{variable} must point at the candidate binary for this surface")
    return value


def _work_dir(prefix):
    """Return (work_root, cleanup). When the producer provides DATUM_EDA_WORK_ROOT the recipe runs
    there and LEAVES it so the producer can capture the real post-run state (mutating families); the
    path is carried out-of-band exactly like DATUM_EDA_CLI, so the committed fixture argv stays stable.
    Otherwise it uses a private temp dir it removes itself (read-only families need no post-state)."""
    provided = os.environ.get("DATUM_EDA_WORK_ROOT")
    if provided:
        work = Path(provided)
        work.mkdir(parents=True, exist_ok=True)
        return work, (lambda: None)
    work = Path(tempfile.mkdtemp(prefix=prefix))
    return work, (lambda: shutil.rmtree(work, ignore_errors=True))


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


def existing_project_stability_factory_cli():
    """Genesis from the factory units profile on the CLI surface: exactly eight sealed seed values,
    an immutable digest-sealed units receipt, and no read or write of the Global preferences
    repository. Runs the real `project new --units-source factory`; the producer provides the work
    root out-of-band (DATUM_EDA_WORK_ROOT) and captures the real published Project as the case's
    post-state. Observations are honest semantics, never returncode==0."""
    binary = _require_binary("DATUM_EDA_CLI")
    work, cleanup = _work_dir("gp-cm05e-case-eps-factory-")
    try:
        destination = work / "project"
        seed = "existing-project-stability/factory/cli"
        project_id = str(uuid.uuid5(uuid.NAMESPACE_URL, seed + "/project"))
        request_id = str(uuid.uuid5(uuid.NAMESPACE_URL, seed + "/request"))
        proc = _run_cli(binary, ["project", "new", "--name", "Acceptance",
                                 "--project-id", project_id, "--request-id", request_id,
                                 "--units-source", "factory", "--json", str(destination)], work=work)
        repository = work / "config" / "datum" / "preferences"
        no_preference = not repository.exists()
        exact_eight = immutable_receipt = False
        report = None
        if proc.returncode == 0:
            try:
                report = json.loads(proc.stdout)
            except json.JSONDecodeError:
                report = None
        if isinstance(report, dict) and report.get("ok") is True:
            result = report.get("result") or {}
            receipt = result.get("units_receipt") or {}
            items = receipt.get("items") or []
            eight_items = len(items) == 8
            seeded_from_factory = all(
                isinstance(item, dict) and item.get("effective_source") == "factory_profile"
                and item.get("provenance") for item in items)
            exact_eight = bool(eight_items and seeded_from_factory)
            # A factory seed must not consult the Global store: the receipt source is the factory
            # profile, so an absent repository plus a factory-sourced receipt is a truthful
            # no-read-or-write observation (not merely an absent file).
            no_preference = bool(no_preference and (receipt.get("source") or {}).get("kind") == "factory"
                                 and seeded_from_factory)
            seals = ("units_seed_catalog_digest", "profile_digest",
                     "genesis_request_digest", "seed_application_digest")
            immutable_receipt = bool(
                eight_items and receipt.get("schema_name")
                and all(isinstance(receipt.get(name), str) and receipt.get(name) for name in seals)
                and receipt.get("genesis_request_digest") == result.get("genesis_request_digest"))
        return {
            "exact-eight-seeds": bool(exact_eight),
            "no-preference-read-or-write": bool(no_preference),
            "immutable-receipt": bool(immutable_receipt),
        }
    finally:
        cleanup()


# (case, variant, surface) -> observation producer. Later GP-CM05E slices extend
# this map; each family runs the real product surface and records honest results.
#
# COPY RULES for new families (audit-binding, checked per family):
#   1. Encode the family's REAL semantic observation, not merely returncode == 0.
#   2. A MUTATING family must make the producer capture the REAL post-run state
#      manifest for `after` and assert on it — the read-only after==before shortcut
#      in producer.py is valid only for non-writing cases like this one.
#   3. recipe.sha256 is a committed-provenance seal (bytes pinned by the candidate
#      revision), not an exec-time hash gate; never rely on it to validate at run.
DISPATCH = {
    ("clean-install", "missing-store", "cli"): clean_install_missing_store_cli,
    ("existing-project-stability", "factory", "cli"): existing_project_stability_factory_cli,
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
