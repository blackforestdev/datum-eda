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


def _eps_genesis(units_source, seed):
    """Run the real `project new --units-source <...>` against an out-of-band work root (so the
    producer can capture the published Project as the case post-state) and return
    (report-or-None, result, units_receipt, items, repository_absent)."""
    binary = _require_binary("DATUM_EDA_CLI")
    work, cleanup = _work_dir(f"gp-cm05e-case-eps-{units_source}-")
    try:
        destination = work / "project"
        project_id = str(uuid.uuid5(uuid.NAMESPACE_URL, seed + "/project"))
        request_id = str(uuid.uuid5(uuid.NAMESPACE_URL, seed + "/request"))
        proc = _run_cli(binary, ["project", "new", "--name", "Acceptance",
                                 "--project-id", project_id, "--request-id", request_id,
                                 "--units-source", units_source, "--json", str(destination)], work=work)
        repository_absent = not (work / "config" / "datum" / "preferences").exists()
        report = None
        if proc.returncode == 0:
            try:
                report = json.loads(proc.stdout)
            except json.JSONDecodeError:
                report = None
        result = report.get("result") if isinstance(report, dict) and report.get("ok") is True else None
        receipt = (result or {}).get("units_receipt") or {}
        items = receipt.get("items") or []
        return report, result, receipt, items, repository_absent
    finally:
        cleanup()


def existing_project_stability_factory_cli():
    """Genesis from the factory units profile on the CLI surface: exactly eight sealed seed values,
    an immutable digest-sealed units receipt, and no read or write of the Global preferences
    repository. Observations are honest semantics, never returncode==0."""
    _report, result, receipt, items, repository_absent = _eps_genesis(
        "factory", "existing-project-stability/factory/cli")
    exact_eight = immutable_receipt = False
    no_preference = repository_absent
    if result is not None:
        eight_items = len(items) == 8
        seeded_from_factory = all(
            isinstance(item, dict) and item.get("effective_source") == "factory_profile"
            and item.get("provenance") for item in items)
        exact_eight = bool(eight_items and seeded_from_factory)
        # A factory seed must not consult the Global store: the receipt source is the factory
        # profile, so an absent repository plus a factory-sourced receipt is a truthful
        # no-read-or-write observation (not merely an absent file).
        no_preference = bool(repository_absent and (receipt.get("source") or {}).get("kind") == "factory"
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


def existing_project_stability_global_missing_cli():
    """Genesis with --units-source global while NO Global store exists: exactly eight seeds, the
    factory contribution disclosed in the receipt (global source, no generation, factory_default
    items), and still no Global store created. Honest semantics, never returncode==0."""
    _report, result, receipt, items, repository_absent = _eps_genesis(
        "global", "existing-project-stability/global-missing/cli")
    exact_eight = disclosed = False
    if result is not None:
        eight_items = len(items) == 8
        exact_eight = bool(eight_items)
        source = receipt.get("source") or {}
        # With no Global store the global source resolves to the factory default and the receipt
        # DISCLOSES that contribution: global kind, no generation, a defaults profile, and every
        # seed effective_source factory_default with provenance naming it.
        disclosed = bool(
            eight_items and source.get("kind") == "global" and source.get("generation") is None
            and source.get("defaults_profile_id")
            and all(isinstance(item, dict) and item.get("effective_source") == "factory_default"
                    and isinstance(item.get("provenance"), str) and "factory_default" in item["provenance"]
                    for item in items))
    return {
        "exact-eight-seeds": bool(exact_eight),
        "factory-contribution-disclosed": bool(disclosed),
        "no-preference-store": bool(repository_absent),
    }


ACTIVE_SECTIONS = {"appearance", "units"}
RESERVED_SENTINEL = "datum.pcb.route_profile"  # a known reserved descriptor (reserved_preference_key)
QUERY_VERBS = (["describe"], ["list"], ["get", "datum.units.system"], ["search", "unit"],
               ["explain", "datum.units.system"], ["preview-project-units-seed", "--source", "global"])


def _query_context(binary, work, argv):
    """Run one read-only query verb; return its parsed envelope (or None) without writing a store."""
    proc = _run_cli(binary, ["preferences", *argv], work=work)
    if proc.returncode != 0:
        return None
    try:
        return json.loads(proc.stdout)
    except json.JSONDecodeError:
        return None


def surface_parity_negative_active_inventory_cli():
    """The CLI query surface exposes exactly the active inventory: 11 active descriptors, the 45
    reserved descriptors absent from the listing, exactly the two active sections, and one shared
    service (identical active catalog digest + counts across every query verb). Read-only: no store
    is written, so the empty before-state is also the truthful after-state. Honest semantics only."""
    binary = _require_binary("DATUM_EDA_CLI")
    work, cleanup = _work_dir("gp-cm05e-case-spn-active-inventory-")
    try:
        describe = _query_context(binary, work, ["describe"])
        eleven_active = reserved_absent = two_sections = same_service = False
        if isinstance(describe, dict) and describe.get("ok") is True:
            context = describe.get("context") or {}
            inner = ((describe.get("result") or {}).get("result") or {}).get("result") or {}
            sections = inner.get("sections") or []
            controls = inner.get("controls") or []
            section_ids = {s.get("id") for s in sections if isinstance(s, dict)}
            control_keys = [c.get("key") for c in controls if isinstance(c, dict)]
            control_sections = {c.get("section_id") for c in controls if isinstance(c, dict)}

            eleven_active = bool(context.get("active_descriptor_count") == 11 and len(controls) == 11)
            # The 45 reserved descriptors exist but are ABSENT from the active surface: the listing is
            # exactly the 11 active controls, all within the two active sections, and a known reserved
            # key does not appear among them.
            reserved_absent = bool(
                context.get("reserved_descriptor_count") == 45 and len(controls) == 11
                and control_sections <= ACTIVE_SECTIONS and RESERVED_SENTINEL not in control_keys)
            two_sections = bool(len(sections) == 2 and section_ids == ACTIVE_SECTIONS
                                and control_sections <= ACTIVE_SECTIONS)

            # same-service: every query verb answers from one service -- identical active catalog
            # digest and identical active/reserved counts.
            digests, actives, reserveds, all_ok = set(), set(), set(), True
            for argv in QUERY_VERBS:
                envelope = describe if argv == ["describe"] else _query_context(binary, work, argv)
                if not (isinstance(envelope, dict) and envelope.get("ok") is True):
                    all_ok = False
                    break
                ctx = envelope.get("context") or {}
                digests.add(ctx.get("active_catalog_digest"))
                actives.add(ctx.get("active_descriptor_count"))
                reserveds.add(ctx.get("reserved_descriptor_count"))
            same_service = bool(all_ok and len(digests) == 1 and None not in digests
                                and actives == {11} and reserveds == {45})
        return {
            "11-active": bool(eleven_active),
            "45-reserved-absent": bool(reserved_absent),
            "two-sections": bool(two_sections),
            "same-service": bool(same_service),
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
    ("existing-project-stability", "global-missing", "cli"): existing_project_stability_global_missing_cli,
    ("surface-parity-negative", "active-inventory", "cli"): surface_parity_negative_active_inventory_cli,
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
