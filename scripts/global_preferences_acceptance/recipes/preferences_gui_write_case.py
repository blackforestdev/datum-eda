#!/usr/bin/env python3
"""Committed GP-CM05E GUI WRITE case recipe (family-dispatching; global-populated first).

Disjoint from ``recipes/preferences_gui_case.py`` (the READ-ONLY clean-install GUI recipe) and from
the CLI ``recipes/preferences_case.py`` (at its PM022 ceiling): the write-gate GUI families earn their
own committed recipe. The validator binds a case to exactly this invocation and replays it.

    python3 scripts/global_preferences_acceptance/recipes/preferences_gui_write_case.py \
        --fixture-json <canonical frozen fixture JSON>

Every parameter -- including the display coordinate -- is carried in that single fixture JSON. The
candidate GUI binary the report pins is provided out-of-band through the environment so the committed
argv stays reproducible, and the producer seeds the real on-disk precondition in the work root it
carries the recipe out-of-band too (both exactly as the CLI write families do):

    DATUM_EDA_GUI         the candidate ``datum-gui`` executable (gui surface)
    DATUM_EDA_WORK_ROOT   the producer's work root. For a WRITE family the producer has already seeded
                          the real precondition under it (``config/datum/preferences`` populated to
                          gen0) BEFORE this recipe runs, and leaves it afterward so the producer can
                          capture the real post-run store. The GUI launches with
                          XDG_CONFIG_HOME=<work_root>/config, so it reads that same seeded store.

Coverage families (dispatched on the fixture's coverage head; an unimplemented head is refused, never
faked):

    existing-project-stability / global-populated / default / gui   IMPLEMENTED (this unit)

The global-populated row is a PROJECT-GENESIS flow driven through the real New Project modal against a
POPULATED Global store, with the four governed assertions mirroring the CLI anchor
``existing_project_stability_global_populated_cli``:

    exact-eight-seeds                eight seeds taken from the populated Global store (system carries
                                     the user value imperial; source kind global with a real generation)
    resolver-valid-four-file-project the four-file Project the GUI created resolves
                                     (project.json + board/rules/schematic, and `project validate` == 0)
    immutable-receipt                the genesis receipt is a sealed, immutable genesis artifact -- NOT a
                                     mutable preference-journal commit. GUI SURFACE VARIANT (PM-ruled,
                                     same product claim, methodology divergence noted for audit): the CLI
                                     binds receipt.genesis_request_digest to the `project new --json`
                                     stdout envelope's digest, which the GUI genesis does not emit, and
                                     re-genesis idempotency is unreachable because the modal mints its own
                                     project-id/request-id (no id control to replay). This surface proves
                                     it two ways together: SHAPE (eight items + schema_name + four digest
                                     seals, 64-hex) AND BYTE-STABILITY -- the receipt block is re-read
                                     after the assertion-4 committed Global edit (gen0->gen1) and must be
                                     byte-identical, i.e. a later global edit does not rewrite the sealed
                                     historical genesis receipt.
    later-global-edits-no-change     a later Global edit (external governed APPLY, gen0->gen1) leaves the
                                     published Project byte-identical

The recipe reports honest observations and always exits 0 on a completed run: a false observation is
recorded as data, not turned into a command failure, so the validator (not the producer) adjudicates
it. Observations never retry to green.
"""
import argparse
import json
import os
import re
import subprocess
import sys
import time
import uuid
from pathlib import Path

SCHEMA = "datum.preferences.case-observations.v2"

# The four seal digests a genesis receipt carries (a mutable preference-journal commit carries none).
RECEIPT_SEALS = ("units_seed_catalog_digest", "profile_digest",
                 "genesis_request_digest", "seed_application_digest")
# The four-file resolver surface a valid native Project resolves from.
PROJECT_FILES = ("project.json", "board/board.json", "rules/rules.json", "schematic/schematic.json")
_DIGEST = re.compile(r"[0-9a-f]{64}")


def _carries_digest(value):
    """A non-empty string carrying a 64-hex SHA-256 digest (tolerating a ``sha256:`` prefix or any
    envelope the seal uses). Used to check each genesis-receipt seal is a real digest, not just set."""
    return isinstance(value, str) and bool(_DIGEST.search(value))

# Reuse the governed foreground-TTY helper for the later-edit APPLY mutation (keeps the pty logic in
# ONE audited place and ships with the recipe for reproducible replay), exactly as the CLI recipe does.
sys.path.insert(0, str(Path(__file__).resolve().parents[2]))
from global_preferences_acceptance.process import execute as _execute_tty  # noqa: E402


def _require_binary(variable):
    value = os.environ.get(variable)
    if not value or not Path(value).is_file():
        raise SystemExit(f"{variable} must point at the candidate binary for this surface")
    return value


def _work_root():
    """A WRITE family runs in the producer's seeded work root (carried out-of-band exactly like the
    surface binary) so the GUI reads the populated store and the producer captures the real post-run
    store. Absent means the producer did not stage the precondition -- a hard error, never a tempdir."""
    provided = os.environ.get("DATUM_EDA_WORK_ROOT")
    if not provided:
        raise SystemExit("DATUM_EDA_WORK_ROOT must be set by the producer for a GUI write family")
    work = Path(provided)
    require = work / "config" / "datum" / "preferences"
    if not require.is_dir():
        raise SystemExit(f"producer did not seed the populated Global store at {require}")
    return work


def _coordinate(fixture, expected_head):
    """Extract and validate the display coordinate (backend, scale, window) from the frozen fixture's
    coverage parameters, and confirm the fixture selects this recipe's expected coverage head. JSON
    turns the matrix tuple into a list; normalise it back to (str, float, (int, int))."""
    coverage = (fixture.get("parameters") or {}).get("coverage")
    if not isinstance(coverage, list) or len(coverage) != 5:
        raise SystemExit("fixture coverage must be a 5-tuple [case, variant, subcase, surface, coordinate]")
    head = tuple(coverage[:4])
    if head != expected_head:
        raise SystemExit(f"recipe bound to {expected_head}, fixture selects {head}")
    coordinate = coverage[4]
    if (not isinstance(coordinate, list) or len(coordinate) != 3
            or not isinstance(coordinate[0], str)
            or not isinstance(coordinate[1], (int, float))
            or not isinstance(coordinate[2], list) or len(coordinate[2]) != 2
            or not all(isinstance(dimension, int) for dimension in coordinate[2])):
        raise SystemExit("gui coordinate must be [backend, scale, [width, height]]")
    return coordinate[0], float(coordinate[1]), (coordinate[2][0], coordinate[2][1])


def _drive_new_project_genesis(qa, main, *, name, location):
    """Drive the real New Project modal to genesis a Project from the populated Global store, through
    genuine OS input (the PINNED, reachability-confirmed drive). SAFETY: the Location is overwritten
    off the real home to the isolated `location` BEFORE Create, so genesis never touches the real home.

    1. Ctrl+N opens the modal (workspace_keyboard.rs: ctrl+n -> NewProject), Project name focused.
    2. Type the project name; the Folder name auto-derives (Use project name checked).
    3. Focus Location, select-all (Ctrl+A), type the isolated destination -- overwrite the default home.
    4. Focus Create, Space -- genesis. The Project lands under `location`.
    """
    location.mkdir(parents=True, exist_ok=True)
    qa.key(main, "ctrl+n")
    qa.wait("New Project modal open", lambda: qa.window("New Project — Datum") is not None)
    qa.text(main, name)  # Project name is focused on open
    qa.focus_control(main, "Location")
    qa.key(main, "ctrl+a")
    qa.text(main, str(location))
    qa.focus_control(main, "Create")
    try:
        qa.capture("new-project-genesis-before-create", main)  # retained visual evidence
    except Exception:
        pass
    qa.key(main, "space")
    # Genesis writes exactly one project.json under the isolated location; wait for it, then return it.
    qa.wait("project genesis published", lambda: any(location.rglob("project.json")))
    projects = sorted(location.rglob("project.json"))
    return projects[0].parent if projects else None


def _run_gui_genesis(gui_binary, work, *, backend, scale, window, name):
    """Launch the candidate datum-gui against the seeded work root's config (XDG_CONFIG_HOME=
    work/config, the populated Global store) and drive the New Project modal to genesis. Returns the
    created Project directory (or None if genesis did not publish). Uses the governed owned-Weston
    capture envelope (StartupQA); scale 2.0 needs the envelope's scale/input extension and is refused
    here rather than mis-bound (the wayland/1.0 and x11/1.0 coordinates are blessed)."""
    if backend not in ("wayland", "x11") or float(scale) != 1.0:
        raise SystemExit(f"this capture envelope supports the wayland/1.0 and x11/1.0 coordinates "
                         f"(scale 2.0 pending its own feasibility probe); got {backend}/{scale}")
    repo = Path(__file__).resolve().parents[3]
    scripts_dir = str(repo / "scripts")
    if scripts_dir not in sys.path:
        sys.path.insert(0, scripts_dir)
    os.environ["DATUM_QA_BINARY_DIR"] = str(Path(gui_binary).resolve().parent)

    import gi  # noqa: E402
    gi.require_version("Atspi", "2.0")
    from prove_native_project_startup_gui import StartupQA  # noqa: E402

    # StartupQA(work) binds XDG_CONFIG_HOME=work/config -- the producer-seeded populated Global store.
    qa = StartupQA(work)
    # Project-free launch: neutralise the envelope's stale-parent probe vars so the GUI opens with NO
    # Project (genesis starts from a clean workspace reading the populated Global store).
    for key in ("DATUM_PROJECT_ROOT", "DATUM_DISCOVERY", "DATUM_AGENT_CREDENTIAL_FILE",
                "EDA_ENGINE_SOCKET", "DATUM_ENGINE_SOCKET", "DATUM_MCP_ENDPOINT",
                "DATUM_AGENT_PROVENANCE"):
        qa.env.pop(key, None)
    try:
        qa.start_envelope(output_size=(int(window[0]), int(window[1])), backend=backend)
        main = qa.launch("existing-project-stability-global-populated-gui")
        return _drive_new_project_genesis(qa, main, name=name, location=work / "genesis-dest")
    finally:
        try:
            qa.stop()
        except Exception:
            pass
        try:
            qa.close_envelope()
        except Exception:
            pass


def _project_digest_map(project_dir):
    """Map every regular file under the published Project to its sha256 -- a disk-level fingerprint
    used to prove the Project bytes are unchanged by a later Global edit."""
    import hashlib
    fingerprint = {}
    if project_dir and project_dir.exists():
        for path in sorted(project_dir.rglob("*")):
            if path.is_file() and not path.is_symlink():
                fingerprint[path.relative_to(project_dir).as_posix()] = \
                    hashlib.sha256(path.read_bytes()).hexdigest()
    return fingerprint


def _validate_project(cli_binary, work, project_dir):
    """Read-only resolver check: `datum-eda project validate <dir>` against the isolated config root.
    stdin is /dev/null (no foreground TTY), so validate stays deterministic and never blocks."""
    env = {**os.environ, "HOME": str(work), "XDG_CONFIG_HOME": str(work / "config")}
    env.pop("DATUM_GUI_PREFERENCES_PATH", None)
    proc = subprocess.run([str(cli_binary), "--format", "json", "project", "validate", str(project_dir)],
                          capture_output=True, text=True, stdin=subprocess.DEVNULL, env=env)
    return proc.returncode == 0


def _later_global_edit(cli_binary, work, generation):
    """Perform the later Global edit that must not perturb the published Project: a governed TTY-APPLY
    `preferences set datum.units.system metric` (gen0 -> gen1) against the seeded store. Returns True
    iff it committed a real change (so later-global-edits-no-change proves no-change under a REAL edit,
    not a no-op). The ROW's defining action is the GUI genesis; this edit is only the perturbation."""
    env = {k: v for k, v in os.environ.items()
           if k not in ("DATUM_GUI_PREFERENCES_PATH", "DATUM_ENGINE_SOCKET", "EDA_ENGINE_SOCKET")}
    env["HOME"] = str(work)
    env["XDG_CONFIG_HOME"] = str(work / "config")
    argv = ["--format", "json", "preferences", "set", "datum.units.system",
            "--value-json", json.dumps("metric"),
            "--expected", json.dumps({"kind": "generation", "generation": generation}),
            "--request-id", str(uuid.uuid5(uuid.NAMESPACE_URL,
                                           "existing-project-stability/global-populated/gui/later-edit")),
            "--reason", "GP-CM05E GUI later global edit"]
    run = _execute_tty(cli_binary, argv, env, timeout=30, confirmation=True)
    text = run["stdout"].decode("utf-8", "replace").replace("\r", "")
    start = text.find('{\n  "ok"')
    try:
        envelope = json.loads(text[start:]) if start >= 0 else None
    except json.JSONDecodeError:
        envelope = None
    committed = bool(run["exit_code"] == 0 and run["confirmed"] and not run["timed_out"]
                     and isinstance(envelope, dict) and envelope.get("ok") is True)
    changed = bool(committed and (((envelope or {}).get("result") or {}).get("result") or {})
                   .get("changed") is True)
    return changed


def existing_project_stability_global_populated_gui(fixture):
    """Genesis a Project from the POPULATED Global store through the GUI New Project modal, then observe
    the four governed assertions. The GUI is driven first (the defining action); the receipt/resolver
    observations are then read from the published Project, and the later Global edit proves the Project
    is byte-unchanged by it. Honest observations only -- nothing is faked or retried to green."""
    expected_head = ("existing-project-stability", "global-populated", "default", "gui")
    backend, scale, window = _coordinate(fixture, expected_head)
    gui_binary = _require_binary("DATUM_EDA_GUI")
    cli_binary = Path(gui_binary).resolve().parent / "datum-eda"
    if not cli_binary.is_file():
        raise SystemExit(f"candidate CLI companion missing for resolver validate/later-edit: {cli_binary}")
    work = _work_root()

    project_dir = _run_gui_genesis(gui_binary, work, backend=backend, scale=scale,
                                   window=window, name="Acceptance")

    exact_eight = resolver_valid = immutable_receipt = later_no_change = False
    if project_dir is not None and (project_dir / "project.json").is_file():
        try:
            manifest = json.loads((project_dir / "project.json").read_text())
        except (OSError, json.JSONDecodeError):
            manifest = {}
        receipt = manifest.get("project_units_seed_receipt") or {}
        items = receipt.get("items") or []
        source = receipt.get("source") or {}
        system = next((i for i in items if i.get("key") == "datum.units.system"), {})

        # exact-eight-seeds: eight seeds from the POPULATED Global store -- system carries the user
        # value (imperial), effective_source user, source kind global with a real generation.
        exact_eight = bool(len(items) == 8 and source.get("kind") == "global" and source.get("generation")
                           and system.get("copied_value") == "imperial"
                           and system.get("effective_source") == "user")

        # resolver-valid-four-file-project: the four files resolve + parse, and `project validate` == 0.
        files_ok = True
        for name in PROJECT_FILES:
            path = project_dir / name
            if not path.is_file():
                files_ok = False
                continue
            try:
                json.loads(path.read_text())
            except (OSError, json.JSONDecodeError):
                files_ok = False
        resolver_valid = bool(files_ok and _validate_project(cli_binary, work, project_dir))

        # immutable-receipt (GUI-surface variant; see the module caveat and the PM ruling): SHAPE --
        # eight items, a schema_name, all four digest seals present as non-empty strings each carrying a
        # 64-hex digest -- AND, because "looks sealed" is necessary but not sufficient, BYTE-STABILITY:
        # the receipt block is captured now and re-read after assertion-4's committed gen0->gen1 Global
        # edit, and must be byte-identical. That is the assertion's product meaning: a later committed
        # global edit does not rewrite the sealed historical genesis receipt (not a mutable journal row).
        shape_ok = bool(len(items) == 8 and isinstance(receipt.get("schema_name"), str)
                        and receipt.get("schema_name")
                        and all(_carries_digest(receipt.get(s)) for s in RECEIPT_SEALS))
        receipt_bytes_before = json.dumps(receipt, sort_keys=True)

        # later-global-edits-no-change: snapshot the Project, perform the later Global edit
        # (gen0 -> gen1, TTY APPLY), snapshot again -> the Project is byte-identical.
        before_project = _project_digest_map(project_dir)
        changed = _later_global_edit(cli_binary, work, source.get("generation"))
        after_project = _project_digest_map(project_dir)
        later_no_change = bool(changed and before_project and before_project == after_project)

        # Re-read the receipt block AFTER the committed perturbation and assert byte-stability. The
        # perturbation must have actually committed (`changed`) for this to be a real immutability
        # observation rather than a trivial no-op comparison.
        try:
            receipt_after = (json.loads((project_dir / "project.json").read_text())
                             .get("project_units_seed_receipt") or {})
        except (OSError, json.JSONDecodeError):
            receipt_after = None
        receipt_stable = bool(receipt_after is not None
                              and json.dumps(receipt_after, sort_keys=True) == receipt_bytes_before)
        immutable_receipt = bool(shape_ok and changed and receipt_stable)

    return {
        "exact-eight-seeds": bool(exact_eight),
        "resolver-valid-four-file-project": bool(resolver_valid),
        "immutable-receipt": bool(immutable_receipt),
        "later-global-edits-no-change": bool(later_no_change),
    }


# Coverage head -> observer. Only the implemented family is registered; an unimplemented head is
# refused by observe() (never fabricated to a passing row).
OBSERVERS = {
    ("existing-project-stability", "global-populated", "default", "gui"):
        existing_project_stability_global_populated_gui,
}


def observe(fixture):
    """Dispatch on the fixture's coverage head to the implemented family observer."""
    coverage = (fixture.get("parameters") or {}).get("coverage")
    if not isinstance(coverage, list) or len(coverage) != 5:
        raise SystemExit("fixture coverage must be a 5-tuple [case, variant, subcase, surface, coordinate]")
    head = tuple(coverage[:4])
    observer = OBSERVERS.get(head)
    if observer is None:
        raise SystemExit(f"no GUI write recipe is implemented for coverage head {head}")
    return observer(fixture)


def main(argv=None):
    parser = argparse.ArgumentParser(description="GP-CM05E GUI write case recipe (family-dispatching)")
    parser.add_argument("--fixture-json", required=True)
    args = parser.parse_args(argv)
    fixture = json.loads(args.fixture_json)
    start = time.monotonic()
    observations = observe(fixture)
    elapsed = time.monotonic() - start
    # `executed_tests` is the count the command-observation parser reads from this line; it is a count
    # of completed observations, not a pass/fail claim.
    print(f"Ran {len(observations)} tests in {elapsed:.3f}s", file=sys.stderr)
    print(json.dumps({"schema": SCHEMA, "executed_tests": len(observations),
                      "observations": observations}, sort_keys=True))
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
