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
the candidate at the HEAD you run this against so the per-row revision is honest
(the full GUI corpus reruns at the pinned S6 candidate separately). The capture
envelope's StartupQA hashes THREE same-directory candidate companions for
provenance -- datum-gui, datum-eda and eda-engine-daemon -- so build all three in
one guarded invocation of ``gates.release_build()`` (``-p datum-eda-cli
-p datum-gui-app -p eda-engine-daemon``), not ``--bin datum-gui`` alone; this
driver preflights their presence and refuses with an actionable missing-list.
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
# StartupQA.__init__ (prove_native_project_startup_gui.py) hashes all three same-directory candidate
# companions for provenance; a missing one aborts the recipe subprocess with an OPAQUE empty-stdout
# JSONDecodeError in produce_case. Preflight them here with an actionable missing-list instead.
CANDIDATE_COMPANIONS = ("datum-gui", "datum-eda", "eda-engine-daemon")


def _preflight_candidate_companions(gui_binary):
    """Fail fast (before Capture/recipe/envelope startup) if any StartupQA candidate companion is
    missing from the candidate directory, naming the canonical all-three build command."""
    directory = Path(gui_binary).resolve().parent
    missing = [name for name in CANDIDATE_COMPANIONS if not (directory / name).is_file()]
    if missing:
        raise SystemExit(
            f"candidate directory {directory} is missing StartupQA companion binaries: "
            f"{', '.join(missing)}. Build all three at the pinned revision in one guarded invocation "
            f"-- gates.release_build() (-p datum-eda-cli -p datum-gui-app -p eda-engine-daemon) -- "
            f"not --bin datum-gui alone.")


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


def _coordinate_tuple(raw):
    """Normalise a JSON coordinate [backend, scale, [w, h]] to (backend, float, (w, h))."""
    backend, scale, window = raw
    return (backend, float(scale), (int(window[0]), int(window[1])))


def _derive_native_id(coordinate):
    """The deterministic native environment id ``native-<backend>-<int-scale>-<WxH>`` a producer may
    derive from its coordinate -- the same construction as assemble.native_environment_id /
    environment_record._scale_label (int-if-whole scale token). Used ONLY to re-assert the frozen
    selection id consumer-side, never to mint one."""
    backend, scale, window = coordinate
    scale_token = str(int(scale)) if float(scale) == int(scale) else str(scale)
    return f"native-{backend}-{scale_token}-{window[0]}x{window[1]}"


def _consume_identity(identity, gui_binary, coordinate):
    """Bind this row to dev's FROZEN shared identity, computing NOTHING (F1 reference wiring).

    Seam 1: the frozen baton's 21-field ``environments`` array carries NO per-entry ``coordinate`` key
    (its closed field set + environment_sha256 binding forbid injecting one). The coordinate->id
    projection lives in the frozen.json ``selection`` aid, one {coordinate, id} row per native entry;
    this consumer matches there, then RE-ASSERTS the resolved id against the 21-field array's ids (the
    seam-1 ONE binding, re-checked consumer-side) and against the deterministic native derivation.
    candidate / input_manifest / matrix_sha256 are carried verbatim; the provided gui binary is
    verified to be the pinned candidate (no substitution). Returns the same tuple the self-compute path
    produces."""
    candidate = identity["candidate"]
    gui_sha = candidate["binaries"]["gui"]["sha256"]
    if _sha_file(gui_binary) != gui_sha:
        raise SystemExit(
            f"--gui-binary sha256 != candidate.binaries.gui.sha256 ({gui_sha}); the frozen identity "
            "pins the gui binary -- no substitution")
    binaries = {"gui": gui_sha}
    matched = [row for row in identity["selection"]
               if _coordinate_tuple(row["coordinate"]) == coordinate]
    if len(matched) != 1:
        raise SystemExit(
            f"frozen identity selection must carry exactly one row for coordinate {coordinate}; "
            f"found {len(matched)}")
    environment_id = matched[0]["id"]
    if environment_id not in {entry["id"] for entry in identity["environments"]}:
        raise SystemExit(
            f"selection id {environment_id!r} is not an id in the frozen 21-field environments array "
            "(seam-1 one-binding violation)")
    if environment_id != _derive_native_id(coordinate):
        raise SystemExit(
            f"selection id {environment_id!r} != deterministic native id "
            f"{_derive_native_id(coordinate)!r} for coordinate {coordinate}")
    environments = {environment_id: coordinate}
    return candidate, binaries, identity["input_manifest"], identity["matrix_sha256"], environments


def build_gui_case(bundle_root, *, gui_binary, coordinate, identity=None):
    """Produce the one clean-install/missing-store/gui row for `coordinate`.

    With `identity` (dev's FROZEN shared identity -- F1 reference wiring) the row BINDS to it and
    computes nothing, so every surface in the assembled report shares one identity source (no drift).
    Without it, the standalone/debug path self-computes candidate/manifest/matrix/environment from the
    local tree (legacy; one surface at a time)."""
    key = ("clean-install", "missing-store", "default", "gui", coordinate)
    require(key in case_inventory(), f"coordinate {coordinate} is not a real GUI case for this family")
    _preflight_candidate_companions(gui_binary)

    capture = Capture(bundle_root)
    if identity is not None:
        candidate, binaries, input_manifest_ref, matrix_sha256, environments = _consume_identity(
            identity, gui_binary, coordinate)
    else:
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
    if identity is not None and "environment_sha256" in identity:
        report["environment_sha256"] = identity["environment_sha256"]
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
    parser.add_argument("--identity", default=None, type=Path,
                        help="dev's FROZEN shared identity JSON (candidate / environments / "
                             "environment_sha256 / input_manifest / matrix_sha256). When given the row "
                             "BINDS to it, computing nothing (the Phase-A reference wiring); its "
                             "input_manifest content must be resolvable in the shared --bundle-root.")
    args = parser.parse_args(argv)
    args.bundle_root.mkdir(parents=True, exist_ok=True)
    bundle_root = args.bundle_root.resolve()
    gui_binary = args.gui_binary.resolve()
    coordinate = _parse_coordinate(args.coordinate) if args.coordinate else DEFAULT_COORDINATE
    identity = json.loads(args.identity.read_text()) if args.identity else None

    report, binaries, environments, key = build_gui_case(
        bundle_root, gui_binary=gui_binary, coordinate=coordinate, identity=identity)
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
