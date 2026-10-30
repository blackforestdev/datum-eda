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
import shutil
import stat
import subprocess
import sys
import uuid
from pathlib import Path

HERE = Path(__file__).resolve()
ROOT = HERE.parents[2]
sys.path.insert(0, str(HERE.parents[1]))  # put scripts/ on the path for the package

from global_preferences_acceptance import observations as obs  # noqa: E402
from global_preferences_acceptance.capture import Capture  # noqa: E402
from global_preferences_acceptance.inputs import Bundle, canonical, digest, parse, read_file, require  # noqa: E402
from global_preferences_acceptance.matrix import case_inventory  # noqa: E402
from global_preferences_acceptance.process import execute  # noqa: E402

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


def _coverage_root(key):
    case, variant, subcase, surface, _coordinate = key
    return f"{case}/{variant}/{subcase}/{surface}"


def _empty_state(key):
    """Empty store state -- the frozen `before` for a fresh-install family (and the `after` of a
    read-only one, whose observations prove nothing was created)."""
    return {"schema": STATE_SCHEMA, "root": _coverage_root(key), "files": []}


def _snapshot_dir(capture, root_label, directory, prefix):
    """Capture every regular file under `directory` into a real state.v2 manifest (links rejected),
    prefixing each path so it sorts under the validator's virtual root. This is how a mutating
    family reports its REAL post-run state instead of the read-only after==before shortcut."""
    files = []
    directory = Path(directory)
    if directory.exists():
        for path in sorted(directory.rglob("*")):
            mode = path.lstat().st_mode
            require(not stat.S_ISLNK(mode), "captured state contains an unexpected symlink")
            if stat.S_ISDIR(mode):
                continue
            require(stat.S_ISREG(mode), "captured state entry is not a regular file")
            raw = path.read_bytes()
            reference = capture.put(raw)
            files.append({"path": prefix + path.relative_to(directory).as_posix(),
                          "kind": "file", "bytes": len(raw), "sha256": reference["sha256"]})
    return {"schema": STATE_SCHEMA, "root": root_label,
            "files": sorted(files, key=lambda entry: entry["path"])}


def produce_case(capture, *, key, candidate, binaries, environments, input_manifest,
                 matrix_sha256, surface_binaries, initial_state=None, capture_after=None,
                 setup=None, bundle_root=None):
    """Build one case_result row by running the committed recipe for its surface.

    `before` is either initial_state(key) (a pure manifest) or, when `setup` is given, the snapshot of
    a real on-disk state the producer builds in the work root BEFORE the recipe runs (e.g. a seeded +
    corrupted store). When capture_after is given the producer captures the REAL post-run state as
    `after`. AFTER-STATE CAVEAT (audit, binding): after==before (capture_after=None) is correct ONLY
    for a READ-ONLY family whose observations prove nothing changed; a MUTATING family MUST pass
    capture_after so `after` reflects reality -- copying after:=before in a write family misreports it.
    """
    case, variant, subcase, surface, _coordinate = key
    required = case_inventory()[key]  # ordered assertion ids for this coverage row

    # Carry the pinned surface binary -- and, for a work-root family, the work root -- out-of-band so
    # the committed argv stays stable and reproducible. The work root is created BEFORE `setup`/the
    # recipe so a setup hook can build the real on-disk starting state the recipe then observes.
    os.environ["DATUM_EDA_CLI"] = surface_binaries["cli"]
    work_root = None
    if capture_after is not None or setup is not None:
        require(bundle_root is not None, "a work-root family needs a bundle_root for its work tree")
        work_root = Path(bundle_root) / ".work" / "_".join(str(p) for p in (case, variant, subcase, surface))
        if work_root.exists():
            shutil.rmtree(work_root)
        work_root.mkdir(parents=True)
        os.environ["DATUM_EDA_WORK_ROOT"] = str(work_root)
    else:
        os.environ.pop("DATUM_EDA_WORK_ROOT", None)

    before_state = setup(capture, key, work_root) if setup is not None else (initial_state or _empty_state)(key)
    before_ref = capture.put(before_state)

    recipe_ref = {"path": RECIPE_RELPATH, "sha256": _sha_file(ROOT / RECIPE_RELPATH)}
    fixture = {
        "schema": FIXTURE_SCHEMA,
        "source_revision": candidate["revision"],
        "recipe": recipe_ref,
        "parameters": {"coverage": list(key)},
        "expected_assertions": {aid: {"operator": "equal", "value": True} for aid in required},
        "initial_state": before_ref,
    }
    fixture_ref = capture.put(fixture)

    command = obs.case_command(fixture)
    cmd_row = capture.command(command, cwd=ROOT, candidate=candidate["revision"],
                              inputs=input_manifest["sha256"])

    raw = json.loads(_read_artifact(capture, cmd_row["log"]))
    case_output = json.loads(_read_artifact(capture, raw["stdout"]))
    observed = case_output["observations"]

    after_ref = (capture_after(capture, key, work_root, before_state)
                 if capture_after is not None else before_ref)

    environment_id = next(iter(environments))
    row = {
        "case_id": case, "variant_id": variant, "subcase_id": subcase, "surface": surface,
        "environment_id": environment_id,
        "binary_sha256": binaries[surface],
        "matrix_sha256": matrix_sha256,
        "fixture": fixture_ref,
        "before": before_ref,
        "after": after_ref,
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


def _seed_corrupt_store(work_root, variant):
    """Create a real single-generation preferences store via one governed TTY-APPLY mutation, drop the
    transient writer lease (a corrupt store at rest holds no live writer), then apply the variant's
    corruption to real bytes. Returns the repository path."""
    config = work_root / "config"
    env = {k: v for k, v in os.environ.items()
           if k not in ("XDG_CONFIG_HOME", "DATUM_ENGINE_SOCKET", "EDA_ENGINE_SOCKET",
                        "DATUM_GUI_PREFERENCES_PATH", "DATUM_EDA_WORK_ROOT")}
    env["XDG_CONFIG_HOME"] = str(config)
    argv = ["--format", "json", "preferences", "set", "datum.accessibility.reduced_motion",
            "--value-json", "true", "--expected", json.dumps({"kind": "missing"}),
            "--request-id", str(uuid.uuid5(uuid.NAMESPACE_URL, f"corrupt-store/{variant}")),
            "--reason", "GP-CM05E corrupt-store fixture seed"]
    run = execute(Path(os.environ["DATUM_EDA_CLI"]), argv, env, timeout=30, confirmation=True)
    require(run["exit_code"] == 0 and run["confirmed"] and not run["timed_out"],
            f"corrupt-store seed mutation failed/unconfirmed: {variant}")
    repo = config / "datum" / "preferences"
    require(repo.is_dir(), "seed did not create a preferences repository")
    lock = repo / "writer.lock"
    if lock.exists():
        lock.unlink()
    head = repo / "head.json"
    if variant == "truncated-head":
        head.write_bytes(head.read_bytes()[:120])
    elif variant == "missing-head":
        head.unlink()
    elif variant == "missing-generation":
        shutil.rmtree(repo / "generations")
    elif variant == "wrong-digest":
        data = json.loads(head.read_text())
        data["generation"]["canonical_manifest_digest"] = "sha256:" + "0" * 64
        head.write_text(json.dumps(data))
    elif variant == "unpublished-generation":
        data = json.loads(head.read_text())
        data["generation"]["generation"] = 5
        data["generation"]["committed_order"] = 5
        head.write_text(json.dumps(data))
    else:
        raise ValueError(f"unknown corrupt-store variant {variant}")
    return repo


def _family_handlers(family):
    """Return (initial_state_fn, capture_after_fn, setup_fn) for a case family. capture_after None =
    read-only (after==before). setup None = `before` is a pure manifest; otherwise setup builds the
    real on-disk starting state in the work root and returns its snapshot as `before`."""
    case, variant = family
    # Project-genesis families: the mutating post-state is the published Project; the Global store
    # stays absent (asserted per-variant). capture_after snapshots the REAL project (after != before).
    if case == "existing-project-stability" and variant in ("factory", "global-missing"):
        def capture_after(capture, key, work_root, before_state):
            return capture.put(_snapshot_dir(capture, before_state["root"],
                                             Path(work_root) / "project", "project/"))
        return _empty_state, capture_after, None
    # Read-only query/parity families: no store is written, so after==before=empty is honest and the
    # recipe's own observations (no-write / absent store) carry the proof. capture_after stays None.
    if case == "surface-parity-negative" and variant in (
            "active-inventory", "all-query-schemas", "all-stable-refusals",
            "malformed-requests", "reserved-keys"):
        return _empty_state, None, None
    if case == "clean-install" and variant == "reopen":
        return _empty_state, None, None
    # Corrupt-store: the producer seeds + corrupts a REAL store (setup) as `before`; the read-only
    # describe must not repair it, so `after` is the real post-describe STORE snapshot (audit
    # guardrail #2 -- the mutated surface is the store dir, not a project) and bytes must match.
    if case == "corrupt-store" and variant in (
            "truncated-head", "missing-head", "missing-generation", "wrong-digest",
            "unpublished-generation"):
        def setup(capture, key, work_root, _variant=variant):
            repo = _seed_corrupt_store(Path(work_root), _variant)
            return _snapshot_dir(capture, _coverage_root(key), repo, "preferences/")

        def capture_after(capture, key, work_root, before_state):
            repo = Path(work_root) / "config" / "datum" / "preferences"
            return capture.put(_snapshot_dir(capture, before_state["root"], repo, "preferences/"))
        return None, capture_after, setup
    raise ValueError(f"case family not implemented: {family}")


def build_family(bundle_root, *, cli_binary, family):
    """Produce every CLI case of one family into a self-contained report + capture bundle."""
    capture = Capture(bundle_root)
    revision = _git_head()
    cli_sha = _sha_file(cli_binary)
    candidate = {"revision": revision,
                 "binaries": {"cli": {"path": str(cli_binary), "sha256": cli_sha}}}
    binaries = {"cli": cli_sha}
    recipe_ref = {"path": RECIPE_RELPATH, "sha256": _sha_file(ROOT / RECIPE_RELPATH)}
    input_manifest_ref = capture.put([recipe_ref])
    matrix_sha256 = digest(canonical(parse((ROOT / MATRIX_RELPATH).read_bytes(), "matrix")))
    environments = {"headless-linux-x86_64": None}
    initial_state, capture_after, setup = _family_handlers(family)
    keys = sorted(k for k in case_inventory() if (k[0], k[1]) == family and k[3] == "cli")
    require(bool(keys), f"no CLI cases for family {family}")
    rows = [produce_case(capture, key=key, candidate=candidate, binaries=binaries,
                         environments=environments, input_manifest=input_manifest_ref,
                         matrix_sha256=matrix_sha256, surface_binaries={"cli": str(cli_binary)},
                         initial_state=initial_state, capture_after=capture_after,
                         setup=setup, bundle_root=bundle_root) for key in keys]
    report = {
        "origin": {"kind": "product-execution", "producer_session": "dev-agent-02"},
        "candidate": candidate,
        "input_manifest": input_manifest_ref,
        "matrix_sha256": matrix_sha256,
        "environments": environments,
        "artifacts": list(capture.inventory.values()),
        "case_results": rows,
    }
    return report, binaries, environments, keys


def validate_case_row(bundle, row, key, report, binaries, environments):
    """Replay the governed per-case validators against one produced case row."""
    coordinate = obs.identity(row, report, environments, binaries)
    observed_key = (row["case_id"], row["variant_id"], row["subcase_id"], row["surface"], coordinate)
    require(observed_key == key, f"produced coverage {observed_key} != target {key}")
    required = case_inventory()[key]
    frozen = obs.fixture(row["fixture"], bundle, required, report, key)
    require(row["command"] == obs.case_command(frozen), "case command does not replay the frozen fixture recipe")
    require(row["before"] == frozen["initial_state"], "case before-state is not the frozen fixture state")
    obs.assertions(row["assertions"], frozen["expected_assertions"], bundle, obs.observation_context(row))
    obs.state_manifest(row["before"], bundle)
    obs.state_manifest(row["after"], bundle)
    obs.command_observation(row, bundle, tests=row["executed_tests"])


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
    parser.add_argument("--family",
                        help="produce a whole CLI case family as case/variant "
                             "(e.g. existing-project-stability/factory); omit for the S1 pattern case")
    args = parser.parse_args(argv)
    args.bundle_root.mkdir(parents=True, exist_ok=True)
    bundle_root = args.bundle_root.resolve()
    cli_binary = args.cli_binary.resolve()

    if args.family:
        family = tuple(args.family.split("/"))
        if len(family) != 2:
            parser.error("--family must be exactly case/variant")
        report, binaries, environments, keys = build_family(bundle_root, cli_binary=cli_binary, family=family)
        args.report.write_text(json.dumps(report, indent=2, sort_keys=True))
        print(f"Produced family {family[0]}/{family[1]}: {len(keys)} CLI case(s)")
        bundle = Bundle(bundle_root, report["artifacts"])
        for row, key in zip(report["case_results"], keys):
            for arow in row["assertions"]:
                print(f"  {key[1]}/{key[2]} {arow['id']}: observed={arow['observed']}")
            try:
                validate_case_row(bundle, row, key, report, binaries, environments)
            except Exception as error:  # preserve + diagnose; never re-roll to green
                print(f"VALIDATION FAILED ({key}, preserved): {error}", file=sys.stderr)
                return 1
        print(f"Family {family[0]}/{family[1]} VALIDATOR-ACCEPTED ({len(keys)} case(s)).")
        return 0

    report, binaries, environments = build_s1(bundle_root, cli_binary=cli_binary)
    args.report.write_text(json.dumps(report, indent=2, sort_keys=True))

    observed = report["case_results"][0]["assertions"]
    print("Produced case:", S1_KEY)
    for row in observed:
        print(f"  {row['id']}: observed={row['observed']}")

    try:
        validate_case(bundle_root, report, S1_KEY, binaries, environments)
    except Exception as error:  # preserve + diagnose; never re-roll to green
        print(f"VALIDATION FAILED (preserved for diagnosis): {error}", file=sys.stderr)
        return 1
    print("S1 capture VALIDATOR-ACCEPTED by the governed per-case validators.")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
