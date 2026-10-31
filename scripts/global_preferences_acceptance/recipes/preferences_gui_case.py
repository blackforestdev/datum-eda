#!/usr/bin/env python3
"""Committed GP-CM05E GUI case recipe: clean-install/missing-store/gui.

Disjoint from ``recipes/preferences_case.py`` on purpose: that CLI recipe is at
its PM022 size ceiling and must not grow, and the GUI surface earns its own
committed recipe. The validator binds a case to exactly this invocation and
replays it (``scripts/global_preferences_acceptance/observations.py``):

    python3 scripts/global_preferences_acceptance/recipes/preferences_gui_case.py \
        --fixture-json <canonical frozen fixture JSON>

Every parameter -- including the display coordinate -- is carried in that single
fixture JSON; no wrapper, dropped parameter or alternative recipe is accepted.
The candidate GUI binary the report pins (``candidate.binaries["gui"]``) is
provided out-of-band through the environment so the committed argv stays
reproducible:

    DATUM_EDA_GUI  -> the candidate ``datum-gui`` executable (gui surface)

Coverage row::

    ("clean-install", "missing-store", "default", "gui", (backend, scale, window))

Required assertions (a READ-ONLY family -- launching and viewing the Global
Preferences surface must create NOTHING):

    factory-defaults       the Global Preferences surface presents factory
                           defaults with no stored generation
    no-repository-created  no preferences repository is written to the store
    no-project-write       no native Project is written

These mirror the CLI anchor ``clean_install_missing_store_cli`` in the sibling
recipe: the two no-write assertions are honest post-run filesystem observations
over an isolated config/home; ``factory-defaults`` is the genuine-input GUI
observation.

The recipe reports honest observations and always exits 0 on a completed run: a
false observation is recorded as data, not turned into a command failure, so the
validator (not the producer) adjudicates it. Observations never retry to green.
"""
import argparse
import json
import os
import shutil
import sys
import tempfile
import time
from pathlib import Path

SCHEMA = "datum.preferences.case-observations.v2"

# Expected coverage identity this recipe is written for. The fixture must select
# exactly this case/variant/subcase/surface; the coordinate varies per row.
EXPECTED_HEAD = ("clean-install", "missing-store", "default", "gui")


def _require_binary(variable):
    value = os.environ.get(variable)
    if not value or not Path(value).is_file():
        raise SystemExit(f"{variable} must point at the candidate binary for this surface")
    return value


def _coordinate(fixture):
    """Extract and validate the display coordinate (backend, scale, window) from the frozen
    fixture's coverage parameters. JSON turns the matrix tuple into a list; normalise it back
    to (str, float, (int, int)) so the launch binds the exact row coordinate, never a default."""
    coverage = (fixture.get("parameters") or {}).get("coverage")
    if not isinstance(coverage, list) or len(coverage) != 5:
        raise SystemExit("fixture coverage must be a 5-tuple [case, variant, subcase, surface, coordinate]")
    head = tuple(coverage[:4])
    if head != EXPECTED_HEAD:
        raise SystemExit(f"recipe bound to {EXPECTED_HEAD}, fixture selects {head}")
    coordinate = coverage[4]
    if (not isinstance(coordinate, list) or len(coordinate) != 3
            or not isinstance(coordinate[0], str)
            or not isinstance(coordinate[1], (int, float))
            or not isinstance(coordinate[2], list) or len(coordinate[2]) != 2
            or not all(isinstance(dimension, int) for dimension in coordinate[2])):
        raise SystemExit("gui coordinate must be [backend, scale, [width, height]]")
    backend, scale, window = coordinate[0], float(coordinate[1]), (coordinate[2][0], coordinate[2][1])
    return backend, scale, window


def _no_repository_created(work):
    """No Global preferences repository is written under the isolated config root. Mirrors the CLI
    anchor's ``config/datum/preferences`` existence check: viewing defaults must not materialise a
    store."""
    return not (work / "config" / "datum" / "preferences").exists()


def _no_project_written(work):
    """A read-only preferences view writes no native Project anywhere under the isolated work root.
    Identical predicate to the sibling CLI recipe (any ``project.json`` is a written Project)."""
    return not any(path.name == "project.json" for path in work.rglob("*"))


VALUE_ROLES = ("combo box", "toggle button", "spin button", "slider", "check box", "radio button")
FACTORY_PROVENANCE = "Factory default · Global · this device"
ACTIVE_DESCRIPTOR_COUNT = 11  # PRODUCT_BOUNDARY.active_global_descriptors (Appearance + Units)


def _section_label(qa):
    """The currently-selected section's label, read inline from the AT-SPI section-nav panel."""
    for node in qa.nodes():
        try:
            if (node.get_role_name() == "panel"
                    and "section navigation" in dict(node.get_attributes()).get("description", "")):
                return node.get_name()
        except Exception:
            continue
    return None


def _section_controls(qa):
    """Every value-control in the currently-shown section as (name, provenance_description), read
    INLINE in one AT-SPI descent -- never hold a node reference (immediate-mode rebuilds per frame
    would stale it). The control's description carries the resolver provenance the surface shows."""
    controls = []
    for node in qa.nodes():
        try:
            if node.get_role_name() in VALUE_ROLES:
                controls.append((node.get_name(), dict(node.get_attributes()).get("description", "")))
        except Exception:
            continue
    return controls


def _run_gui_session(binary, work, *, backend, scale, window):
    """Launch the candidate datum-gui against the ISOLATED home/config, genuinely open the Global
    Preferences surface through real OS input, and observe whether it presents factory defaults.

    Uses the governed capture envelope ``scripts/prove_native_project_startup_gui.py::StartupQA``
    (owned Weston on the shared display -- the host desktop steals coordinate clicks, so the owned
    envelope is mandatory; never a bespoke weston launch). The envelope runs datum-gui at --scale=1
    as either a native Wayland client (backend=wayland) or an Xwayland/X11 client (backend=x11), so
    it binds the wayland/1.0 and x11/1.0 coordinates. scale 2.0 needs the envelope's scale/input
    extension, pending its own feasibility probe, and is refused here rather than mis-bound.

    factory-defaults is TRUE iff the dialog opens and every one of the 11 active descriptors
    (Appearance + Units) shows the surface's own "Factory default · Global · this device" provenance
    -- i.e. no User/Organization contribution and a readable (not preserved-unreadable) store. That
    provenance is the authoritative value the resolver projects onto the control; AT-SPI is read for
    tree/role/name/description (reliable), not a CHECKED state (unreliable under immediate mode).
    A failed read or short count returns False; nothing is faked or retried to green."""
    if backend not in ("wayland", "x11") or float(scale) != 1.0:
        raise SystemExit(f"this capture envelope supports the wayland/1.0 and x11/1.0 coordinates "
                         f"(scale 2.0 pending its own feasibility probe); got {backend}/{scale}")
    repo = Path(__file__).resolve().parents[3]
    scripts_dir = str(repo / "scripts")
    if scripts_dir not in sys.path:
        sys.path.insert(0, scripts_dir)
    os.environ["DATUM_QA_BINARY_DIR"] = str(Path(binary).resolve().parent)

    import gi  # noqa: E402
    gi.require_version("Atspi", "2.0")
    from prove_native_project_startup_gui import StartupQA  # noqa: E402

    qa = StartupQA(work)
    # Isolated, project-free launch: neutralise the envelope's stale-parent probe vars so the GUI
    # opens with NO Project, and let XDG_CONFIG_HOME=work/config be the (missing) store location.
    for key in ("DATUM_PROJECT_ROOT", "DATUM_DISCOVERY", "DATUM_AGENT_CREDENTIAL_FILE",
                "EDA_ENGINE_SOCKET", "DATUM_ENGINE_SOCKET", "DATUM_MCP_ENDPOINT",
                "DATUM_AGENT_PROVENANCE"):
        qa.env.pop(key, None)
    try:
        qa.start_envelope(output_size=(int(window[0]), int(window[1])), backend=backend)
        main = qa.launch("clean-install-missing-store-gui")
        qa.menu_item(main, "Edit", "Preferences", child="Global Preferences…")
        qa.wait("Global Preferences dialog open", lambda: any(
            node.get_role_name() == "dialog" and node.get_name() == "Global Preferences"
            for node in qa.nodes()))
        # Enumerate every section by cycling the section nav (ArrowRight) until it wraps around.
        collected = {}
        for _ in range(len(VALUE_ROLES) + 4):
            section = _section_label(qa)
            if section in collected:
                break
            collected[section] = _section_controls(qa)
            qa.key(main, "Right")
            time.sleep(0.4)
        try:
            qa.capture("global-preferences-factory-defaults", main)  # retained visual evidence
        except Exception:
            pass
        controls = [control for section in collected.values() for control in section]
        dialog_present = any(
            node.get_role_name() == "dialog" and node.get_name() == "Global Preferences"
            for node in qa.nodes())
        exactly_active_count = len(controls) == ACTIVE_DESCRIPTOR_COUNT
        all_factory = bool(controls) and all(
            FACTORY_PROVENANCE in description for _, description in controls)
        return bool(dialog_present and exactly_active_count and all_factory)
    finally:
        try:
            qa.stop()
        except Exception:
            pass
        try:
            qa.close_envelope()
        except Exception:
            pass


def observe(fixture):
    """Run the one GUI coverage row against an isolated home/config and return its three honest
    observations. The GUI session runs first (producing factory-defaults and exercising the real
    surface); the two no-write assertions are then read from the same isolated work root, so they
    are meaningful (the surface was genuinely driven) rather than trivially empty."""
    backend, scale, window = _coordinate(fixture)
    binary = _require_binary("DATUM_EDA_GUI")
    # Capture affordance: when DATUM_GP_GUI_EVIDENCE_ROOT is set the isolated work tree (holding the
    # weston-screenshooter frame and the final isolated config) is created under it and RETAINED as
    # durable packet evidence; otherwise a private temp dir is used and removed. Retention cannot
    # change any observation -- all three are computed from this run's real state before cleanup.
    evidence_root = os.environ.get("DATUM_GP_GUI_EVIDENCE_ROOT")
    if evidence_root:
        base = Path(evidence_root)
        base.mkdir(parents=True, exist_ok=True)
        work = Path(tempfile.mkdtemp(prefix="clean-install-missing-store-gui-", dir=str(base)))
        retain = True
    else:
        work = Path(tempfile.mkdtemp(prefix="gp-cm05e-case-clean-install-missing-store-gui-"))
        retain = False
    try:
        factory_defaults = _run_gui_session(binary, work, backend=backend, scale=scale, window=window)
        return {
            "factory-defaults": bool(factory_defaults),
            "no-repository-created": bool(_no_repository_created(work)),
            "no-project-write": bool(_no_project_written(work)),
        }
    finally:
        if not retain:
            shutil.rmtree(work, ignore_errors=True)


def main(argv=None):
    parser = argparse.ArgumentParser(description="GP-CM05E GUI case recipe (clean-install/missing-store/gui)")
    parser.add_argument("--fixture-json", required=True)
    args = parser.parse_args(argv)
    fixture = json.loads(args.fixture_json)
    start = time.monotonic()
    observations = observe(fixture)
    elapsed = time.monotonic() - start
    # `executed_tests` is the count the command-observation parser reads from this line; it is a
    # count of completed observations, not a pass/fail claim.
    print(f"Ran {len(observations)} tests in {elapsed:.3f}s", file=sys.stderr)
    print(json.dumps({"schema": SCHEMA, "executed_tests": len(observations),
                      "observations": observations}, sort_keys=True))
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
