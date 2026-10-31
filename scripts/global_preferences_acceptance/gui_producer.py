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
import functools
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
    MATRIX_RELPATH, _coverage_root, _empty_state, _git_head, _seed_populated_store, _sha_file,
    _snapshot_dir, produce_case, validate_case,
)

GUI_RECIPE_RELPATH = "scripts/global_preferences_acceptance/recipes/preferences_gui_case.py"
GUI_WRITE_RECIPE_RELPATH = "scripts/global_preferences_acceptance/recipes/preferences_gui_write_case.py"
GUI_A11Y_RECIPE_RELPATH = "scripts/global_preferences_acceptance/recipes/preferences_gui_a11y_case.py"
GUI_A11Y_NAV_RECIPE_RELPATH = "scripts/global_preferences_acceptance/recipes/preferences_gui_a11y_nav_case.py"
GUI_A11Y_MODAL_RECIPE_RELPATH = "scripts/global_preferences_acceptance/recipes/preferences_gui_a11y_modal_case.py"
# The four read-only accessibility-interaction nav scenarios the dispatching nav recipe handles.
GUI_A11Y_NAV_VARIANTS = ("keyboard-all-controls", "search-and-clear", "explanation-and-return",
                         "native-close-reopen")
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


def _candidate_bindings(capture, gui_binary, coordinate, identity, recipe_relpath):
    """Resolve (candidate, binaries, input_manifest_ref, matrix_sha256, environments) for one row.

    With `identity` (dev's FROZEN shared identity -- F1 reference wiring) BIND to it and compute nothing,
    so every surface in the assembled report shares one identity source (no drift). Without it, the
    standalone/debug path self-computes from the local tree (legacy; one surface at a time)."""
    if identity is not None:
        candidate, binaries, input_manifest_ref, matrix_sha256, environments = _consume_identity(
            identity, gui_binary, coordinate)
        # Register the bound input_manifest in this run's inventory so the row's artifacts are
        # self-contained and validate_case's Bundle(report["artifacts"]) can resolve it (the blob
        # lives in the shared bundle_root; produce_case re-verifies its bytes against the reference).
        # Mirrors the headless identity path (producer.build_family, producer.py:467) -- the GUI
        # identity path previously omitted this, so per-row validate_case could not resolve the
        # external input_manifest reference.
        capture.inventory[input_manifest_ref["path"]] = dict(input_manifest_ref)
        return candidate, binaries, input_manifest_ref, matrix_sha256, environments
    revision = _git_head()
    gui_sha = _sha_file(gui_binary)
    candidate = {"revision": revision,
                 "binaries": {"gui": {"path": str(gui_binary), "sha256": gui_sha}}}
    binaries = {"gui": gui_sha}
    recipe_ref = {"path": recipe_relpath, "sha256": _sha_file(ROOT / recipe_relpath)}
    input_manifest_ref = capture.put([recipe_ref])
    matrix_sha256 = digest(canonical(parse((ROOT / MATRIX_RELPATH).read_bytes(), "matrix")))
    # Exactly one environment, whose coordinate equals this row's coordinate (produce_case refuses any
    # other count -- the coordinate-matching rule).
    environments = {_environment_id(coordinate): coordinate}
    return candidate, binaries, input_manifest_ref, matrix_sha256, environments


def _assemble_report(candidate, input_manifest_ref, matrix_sha256, environments, capture, row, identity):
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
    return report


def build_gui_case(bundle_root, *, gui_binary, coordinate, identity=None):
    """Produce the clean-install/missing-store/gui row (READ-ONLY family: after==before, no store)."""
    key = ("clean-install", "missing-store", "default", "gui", coordinate)
    require(key in case_inventory(), f"coordinate {coordinate} is not a real GUI case for this family")
    _preflight_candidate_companions(gui_binary)
    capture = Capture(bundle_root)
    candidate, binaries, input_manifest_ref, matrix_sha256, environments = _candidate_bindings(
        capture, gui_binary, coordinate, identity, GUI_RECIPE_RELPATH)
    row = produce_case(capture, key=key, candidate=candidate, binaries=binaries,
                       environments=environments, input_manifest=input_manifest_ref,
                       matrix_sha256=matrix_sha256, surface_binaries={"gui": str(gui_binary)},
                       initial_state=_empty_state, capture_after=None, setup=None,
                       bundle_root=bundle_root, recipe_relpath=GUI_RECIPE_RELPATH)
    report = _assemble_report(candidate, input_manifest_ref, matrix_sha256, environments, capture, row, identity)
    return report, binaries, environments, key


def _gui_write_setup(capture, key, work_root):
    """Seed the real on-disk precondition BEFORE the GUI recipe runs: populate the Global store to
    gen0 (datum.units.system=imperial via a governed TTY-APPLY, writer lease dropped) so the GUI
    genesis reads a POPULATED store. Mirrors producer._family_handlers' global-populated setup exactly
    -- this is the store-seed MECHANICS reused, not a reimplementation. Returns the `before` snapshot
    (store@gen0); the later Global edit the recipe performs takes it to gen1 so after != before."""
    repo = _seed_populated_store(Path(work_root))
    return _snapshot_dir(capture, _coverage_root(key), repo, "preferences/")


def _gui_write_capture_after(capture, key, work_root, before_state):
    """Snapshot the REAL post-run Global Preferences store (after != before for this WRITE family).
    The recipe's later Global edit commits gen0->gen1 under DATUM_EDA_WORK_ROOT/config; capture_after
    captures the generations/ tree so the row's `after` reflects the committed mutation (the
    AFTER-STATE CAVEAT: a mutating family must never report after==before)."""
    repo = Path(work_root) / "config" / "datum" / "preferences"
    return capture.put(_snapshot_dir(capture, before_state["root"], repo, "preferences/"))


def build_gui_write_case(bundle_root, *, gui_binary, coordinate, identity=None):
    """Produce the existing-project-stability/global-populated/gui WRITE row -- the write-gate pattern.

    A PROJECT-GENESIS flow against a POPULATED Global store: setup seeds the store (gen0); the recipe
    drives the real New Project modal to genesis a Project (eight seeds taken, the four-file resolver
    Project published, the immutable genesis receipt), then performs a later Global edit (gen0->gen1,
    external governed APPLY) and observes the published Project is byte-unchanged by it. before=store
    @gen0, after=store@gen1 -> after != before with a real new generation (the write-family guardrail,
    829668b4 bar on the GUI surface; NOT the corrupt-store preserve-equality shape)."""
    key = ("existing-project-stability", "global-populated", "default", "gui", coordinate)
    require(key in case_inventory(), f"coordinate {coordinate} is not a real GUI case for this family")
    _preflight_candidate_companions(gui_binary)
    capture = Capture(bundle_root)
    candidate, binaries, input_manifest_ref, matrix_sha256, environments = _candidate_bindings(
        capture, gui_binary, coordinate, identity, GUI_WRITE_RECIPE_RELPATH)
    row = produce_case(capture, key=key, candidate=candidate, binaries=binaries,
                       environments=environments, input_manifest=input_manifest_ref,
                       matrix_sha256=matrix_sha256, surface_binaries={"gui": str(gui_binary)},
                       capture_after=_gui_write_capture_after, setup=_gui_write_setup,
                       bundle_root=bundle_root, recipe_relpath=GUI_WRITE_RECIPE_RELPATH)
    report = _assemble_report(candidate, input_manifest_ref, matrix_sha256, environments, capture, row, identity)
    return report, binaries, environments, key


def _gui_a11y_capture_after(capture, key, work_root, before_state):
    """Fold BOTH the post-run store (prefix 'preferences/') AND the recipe-written F4-DESIGN §5 typed
    accessibility-capture payload (prefix 'accessibility-capture/') into ONE after-state manifest. The
    CASE-row schemas are all closed (assertion evidence = {schema,context,id,observed}; the row's fields
    are fixed; stdout = the 5 bools), so the PRIMARY typed evidence (ordered event list + priorities +
    Status-node desc + control table + focus-walk + Orca spoken) has no richer slot -- its content-
    addressed, row-bound home is row['after'] (state.v2, each file sha-bound)."""
    root = before_state["root"]
    store = _snapshot_dir(capture, root, Path(work_root) / "config" / "datum" / "preferences", "preferences/")
    evidence = _snapshot_dir(capture, root, Path(work_root) / "accessibility-capture", "accessibility-capture/")
    files = sorted(store["files"] + evidence["files"], key=lambda entry: entry["path"])
    return capture.put({"schema": store["schema"], "root": root, "files": files})


def build_gui_a11y_case(bundle_root, *, gui_binary, coordinate, identity=None):
    """Produce the accessibility-interaction/set-reset-refusal/gui row -- the §6-N06 a11y-capture anchor.

    The recipe drives the real GP dialog (SET + RESET commit generations; a held-writer-lease REFUSAL is
    refused with NO new generation) through the shared 3-channel accessibility_capture backbone, and
    observes the five matrix assertions (native-at-spi-evidence / correct-focus /
    correct-name-role-value-state / no-trap / truthful-announcement). before=empty store, after=the
    SET/RESET generations -> after != before (write-family guardrail); the refusal's no-write is proven
    by the unchanged generation count INSIDE the run, captured in the diagnostics."""
    key = ("accessibility-interaction", "set-reset-refusal", "default", "gui", coordinate)
    require(key in case_inventory(), f"coordinate {coordinate} is not a real GUI case for this family")
    _preflight_candidate_companions(gui_binary)
    capture = Capture(bundle_root)
    candidate, binaries, input_manifest_ref, matrix_sha256, environments = _candidate_bindings(
        capture, gui_binary, coordinate, identity, GUI_A11Y_RECIPE_RELPATH)
    row = produce_case(capture, key=key, candidate=candidate, binaries=binaries,
                       environments=environments, input_manifest=input_manifest_ref,
                       matrix_sha256=matrix_sha256, surface_binaries={"gui": str(gui_binary)},
                       capture_after=_gui_a11y_capture_after, setup=None,
                       bundle_root=bundle_root, recipe_relpath=GUI_A11Y_RECIPE_RELPATH)
    report = _assemble_report(candidate, input_manifest_ref, matrix_sha256, environments, capture, row, identity)
    return report, binaries, environments, key


def build_gui_a11y_nav_case(bundle_root, *, gui_binary, coordinate, identity=None, variant):
    """Produce one of the four READ-ONLY accessibility-interaction nav rows (keyboard-all-controls /
    search-and-clear / explanation-and-return / native-close-reopen) -- thin personas over the shared
    accessibility_capture backbone via the dispatching nav recipe. These never mutate the Global store,
    so the after-state's preferences/ subtree stays EMPTY (no-write proven by absence); _gui_a11y_capture_after
    still folds the F4-DESIGN section-5 typed payload (prefix accessibility-capture/) into row["after"]."""
    require(variant in GUI_A11Y_NAV_VARIANTS, f"{variant} is not a nav scenario")
    key = ("accessibility-interaction", variant, "default", "gui", coordinate)
    require(key in case_inventory(), f"coordinate {coordinate} is not a real GUI case for this family")
    _preflight_candidate_companions(gui_binary)
    capture = Capture(bundle_root)
    candidate, binaries, input_manifest_ref, matrix_sha256, environments = _candidate_bindings(
        capture, gui_binary, coordinate, identity, GUI_A11Y_NAV_RECIPE_RELPATH)
    row = produce_case(capture, key=key, candidate=candidate, binaries=binaries,
                       environments=environments, input_manifest=input_manifest_ref,
                       matrix_sha256=matrix_sha256, surface_binaries={"gui": str(gui_binary)},
                       capture_after=_gui_a11y_capture_after, setup=None,
                       bundle_root=bundle_root, recipe_relpath=GUI_A11Y_NAV_RECIPE_RELPATH)
    report = _assemble_report(candidate, input_manifest_ref, matrix_sha256, environments, capture, row, identity)
    return report, binaries, environments, key


def build_gui_a11y_modal_case(bundle_root, *, gui_binary, coordinate, identity=None):
    """Produce the accessibility-interaction/new-project-modes/gui row -- the New Project MODAL persona
    over the shared accessibility_capture backbone. READ-ONLY on the Global store (mode exploration
    creates no Project and writes no store), so the after-state's preferences/ subtree stays EMPTY and
    the §5 payload ships an explicit no-write fingerprint (store absent); _gui_a11y_capture_after folds
    the payload into row["after"]. Lands on the mv1k modal-announcement fix (dat-7ynp)."""
    key = ("accessibility-interaction", "new-project-modes", "default", "gui", coordinate)
    require(key in case_inventory(), f"coordinate {coordinate} is not a real GUI case for this family")
    _preflight_candidate_companions(gui_binary)
    capture = Capture(bundle_root)
    candidate, binaries, input_manifest_ref, matrix_sha256, environments = _candidate_bindings(
        capture, gui_binary, coordinate, identity, GUI_A11Y_MODAL_RECIPE_RELPATH)
    row = produce_case(capture, key=key, candidate=candidate, binaries=binaries,
                       environments=environments, input_manifest=input_manifest_ref,
                       matrix_sha256=matrix_sha256, surface_binaries={"gui": str(gui_binary)},
                       capture_after=_gui_a11y_capture_after, setup=None,
                       bundle_root=bundle_root, recipe_relpath=GUI_A11Y_MODAL_RECIPE_RELPATH)
    report = _assemble_report(candidate, input_manifest_ref, matrix_sha256, environments, capture, row, identity)
    return report, binaries, environments, key


FAMILY_BUILDERS = {
    "clean-install": build_gui_case,
    "global-populated": build_gui_write_case,
    "set-reset-refusal": build_gui_a11y_case,
    "new-project-modes": build_gui_a11y_modal_case,
    **{_variant: functools.partial(build_gui_a11y_nav_case, variant=_variant)
       for _variant in GUI_A11Y_NAV_VARIANTS},
}


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
    parser.add_argument("--family", default="clean-install", choices=sorted(FAMILY_BUILDERS),
                        help="GUI case family: clean-install (read-only), global-populated (write) or set-reset-refusal (a11y capture)")
    args = parser.parse_args(argv)
    args.bundle_root.mkdir(parents=True, exist_ok=True)
    bundle_root = args.bundle_root.resolve()
    gui_binary = args.gui_binary.resolve()
    coordinate = _parse_coordinate(args.coordinate) if args.coordinate else DEFAULT_COORDINATE
    identity = json.loads(args.identity.read_text()) if args.identity else None

    report, binaries, environments, key = FAMILY_BUILDERS[args.family](
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
