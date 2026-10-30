#!/usr/bin/env python3
"""GP-CM05E GUI one-pattern PRODUCER: clean-install/missing-store/gui.

A disjoint driver that reuses the surface-generic ``producer.produce_case`` (made
surface-keyed by bc73e90e) to produce exactly ONE GUI coverage row against ONE
display coordinate, then replays the governed per-case validator against it.
``producer.py`` (build_s1 / build_family / main) and ``recipes/preferences_case.py``
are untouched; this family lives entirely in its own recipe + driver.

The row is the PM-authorized single GUI pattern. It is read-only: the surface
must create no preferences repository and write no Project, so ``after == before
== empty`` (``capture_after=None``), exactly like the CLI read-only families.

    python3 scripts/global_preferences_acceptance/gui_producer.py \
        --bundle-root <dir> --report <out.json> \
        --gui-binary <candidate datum-gui> [--coordinate wayland/1.0/1024x900]

candidate.revision is stamped to the current git HEAD (producer._git_head): build
the candidate datum-gui at the HEAD you run this against so the per-row revision
is honest (the full GUI corpus reruns at the pinned S6 candidate separately).
"""
import argparse
import json
import sys
from pathlib import Path

HERE = Path(__file__).resolve()
ROOT = HERE.parents[2]
sys.path.insert(0, str(HERE.parents[1]))  # put scripts/ on the path for the package

from global_preferences_acceptance.capture import Capture  # noqa: E402
from global_preferences_acceptance.inputs import canonical, digest, parse, require  # noqa: E402
from global_preferences_acceptance.matrix import case_inventory  # noqa: E402
from global_preferences_acceptance.producer import (  # noqa: E402
    MATRIX_RELPATH, _empty_state, _git_head, _sha_file, produce_case, validate_case,
)

GUI_RECIPE_RELPATH = "scripts/global_preferences_acceptance/recipes/preferences_gui_case.py"
# PM-authorized single pattern/coordinate. The default is the simplest reliable row; any coordinate
# passed must be a real case_inventory() key for this family.
DEFAULT_COORDINATE = ("wayland", 1.0, (1024, 900))


def _parse_coordinate(text):
    """Parse 'backend/scale/WxH' into (backend, float_scale, (w, h))."""
    try:
        backend, scale, window = text.split("/")
        width, height = window.lower().split("x")
        return backend, float(scale), (int(width), int(height))
    except ValueError:
        raise SystemExit("--coordinate must be backend/scale/WIDTHxHEIGHT, e.g. wayland/1.0/1024x900")


def _environment_id(coordinate):
    backend, scale, window = coordinate
    return f"{backend}-{scale:g}-{window[0]}x{window[1]}"


def build_gui_case(bundle_root, *, gui_binary, coordinate):
    """Produce the one clean-install/missing-store/gui row for `coordinate`."""
    key = ("clean-install", "missing-store", "default", "gui", coordinate)
    require(key in case_inventory(), f"coordinate {coordinate} is not a real GUI case for this family")

    capture = Capture(bundle_root)
    revision = _git_head()
    gui_sha = _sha_file(gui_binary)
    candidate = {"revision": revision,
                 "binaries": {"gui": {"path": str(gui_binary), "sha256": gui_sha}}}
    binaries = {"gui": gui_sha}
    recipe_ref = {"path": GUI_RECIPE_RELPATH, "sha256": _sha_file(ROOT / GUI_RECIPE_RELPATH)}
    input_manifest_ref = capture.put([recipe_ref])
    matrix_sha256 = digest(canonical(parse((ROOT / MATRIX_RELPATH).read_bytes(), "matrix")))
    # Exactly one environment, whose coordinate equals this row's coordinate: produce_case refuses
    # any other count (the coordinate-matching rule, verified inert for CLI coordinate-None cases).
    environments = {_environment_id(coordinate): coordinate}

    row = produce_case(capture, key=key, candidate=candidate, binaries=binaries,
                       environments=environments, input_manifest=input_manifest_ref,
                       matrix_sha256=matrix_sha256, surface_binaries={"gui": str(gui_binary)},
                       initial_state=_empty_state, capture_after=None, setup=None,
                       bundle_root=bundle_root, recipe_relpath=GUI_RECIPE_RELPATH)
    report = {
        "origin": {"kind": "product-execution", "producer_session": "frontend-dev-01"},
        "candidate": candidate,
        "input_manifest": input_manifest_ref,
        "matrix_sha256": matrix_sha256,
        "environments": environments,
        "artifacts": list(capture.inventory.values()),
        "case_results": [row],
    }
    return report, binaries, environments, key


def main(argv=None):
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--bundle-root", required=True, type=Path,
                        help="directory to hold the content-addressed capture bundle")
    parser.add_argument("--report", required=True, type=Path,
                        help="path to write the produced case report JSON")
    parser.add_argument("--gui-binary", required=True, type=Path,
                        help="candidate datum-gui executable (gui surface)")
    parser.add_argument("--coordinate", default=None,
                        help="display coordinate backend/scale/WIDTHxHEIGHT "
                             "(default wayland/1.0/1024x900)")
    args = parser.parse_args(argv)
    args.bundle_root.mkdir(parents=True, exist_ok=True)
    bundle_root = args.bundle_root.resolve()
    gui_binary = args.gui_binary.resolve()
    coordinate = _parse_coordinate(args.coordinate) if args.coordinate else DEFAULT_COORDINATE

    report, binaries, environments, key = build_gui_case(
        bundle_root, gui_binary=gui_binary, coordinate=coordinate)
    args.report.write_text(json.dumps(report, indent=2, sort_keys=True))

    print("Produced case:", key)
    for arow in report["case_results"][0]["assertions"]:
        print(f"  {arow['id']}: observed={arow['observed']}")

    try:
        validate_case(bundle_root, report, key, binaries, environments)
    except Exception as error:  # preserve + diagnose; never re-roll to green
        print(f"VALIDATION FAILED (preserved for diagnosis): {error}", file=sys.stderr)
        return 1
    print("GUI case VALIDATOR-ACCEPTED by the governed per-case validators.")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
