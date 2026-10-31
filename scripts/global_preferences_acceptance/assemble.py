#!/usr/bin/env python3
"""GP-CM05E Phase-A U3: shared-identity freeze + the engine/CLI ONE-capture assembly core.

The Phase-A orchestrator's heart: freeze ONE shared identity (U2 candidate + U1 environments +
the U2 input-manifest closure + the matrix digest) into the `frozen.json` baton every producer
consumes verbatim, then run the engine/CLI case families under that identity into one shared
content-addressed bundle and merge their case rows into the ONE
``datum-global-preferences-production-evidence-v2`` report skeleton.

Audit-flagged integration seams this module closes:

* **Seam 1 (env-entry shape).** The frozen baton carries the full 21-field ``environments`` array
  VERBATIM -- it becomes ``report["environments"]`` and binds report.py's environment_sha256; no
  extra key is ever injected into it. The GUI/native producers' selection need is served two
  compatible ways that land on the same id: the frozen.json-only ``selection`` aid (one
  {coordinate, id} row per native entry, projected from that entry's own fields) and the
  deterministic id derivation ``native-<backend>-<int-scale>-<WxH>`` (asserted against every
  frozen native entry at freeze time). ONE binding, no fork: every per-row ``environment_id``
  must equal an id present in the 21-field array, enforced again at merge.
* **Seam 2 (recipe-in-manifest).** Every recipe a merged row's frozen fixture references must be
  PRESENT in the frozen input-manifest closure with its committed sha256; anything else refuses.

Placeholder lifecycle (the assembled report is a skeleton, never a readiness claim):
``gate_results`` = [] (the Phase-B runner overwrites it), ``independent_review`` = an EXPLICIT
pending-review reference the independent reviewer replaces, ``release_notes`` = an EXPLICIT
pending-handoff reference the U5 release unit replaces. Each placeholder document fails its
governed validator by construction, so a partially assembled report stays incomplete under
``validate_report``; no path in this module computes or issues readiness.

A family the producer has no handler for REFUSES (the gap is preserved and reported) -- it is
never silently skipped, so a partial selection cannot masquerade as the full corpus.
"""
import argparse
import json
import sys
from pathlib import Path

HERE = Path(__file__).resolve()
ROOT = HERE.parents[2]
sys.path.insert(0, str(HERE.parents[1]))  # put scripts/ on the path for the package

from global_preferences_acceptance import producer  # noqa: E402
from global_preferences_acceptance.capture import Capture  # noqa: E402
from global_preferences_acceptance.environment_record import _scale_label, environment_sha256  # noqa: E402
from global_preferences_acceptance.identity import validate_environments  # noqa: E402
from global_preferences_acceptance.inputs import (  # noqa: E402
    Bundle, canonical, closed, digest, parse, require, sha,
)
from global_preferences_acceptance.matrix import case_inventory  # noqa: E402

REPORT_SCHEMA = "datum-global-preferences-production-evidence-v2"
FROZEN_FIELDS = "candidate environments environment_sha256 input_manifest matrix_sha256 selection"
CANDIDATE_ROLES = ("cli", "gui", "daemon", "engine", "gui-render", "mcp")
PENDING_REVIEW_SCHEMA = "datum.preferences.pending-independent-review.v1"
PENDING_RELEASE_SCHEMA = "datum.preferences.pending-release-notes.v1"


def native_environment_id(entry):
    """The deterministic environment id a native producer may derive from its coordinate:
    ``native-<backend>-<int-scale>-<WxH>`` (U1's one construction path, via its own scale token).
    Used here only to ASSERT the frozen array's ids, never to mint one."""
    return (f"native-{entry['display_backend']}-{_scale_label(entry['scale_factor'])}-"
            f"{entry['window_size'][0]}x{entry['window_size'][1]}")


def selection_aid(environments):
    """Seam 1: the frozen.json-ONLY selection aid -- one {coordinate, id} row per native entry,
    PROJECTED from that entry's own 21 fields. Never copied into report["environments"]; the ONE
    binding stays the 21-field array's ids."""
    return [{"coordinate": [e["display_backend"], e["scale_factor"], list(e["window_size"])],
             "id": e["id"]}
            for e in environments if e["display_backend"] is not None]


def freeze_identity(candidate, environments, input_manifest_ref, matrix_sha256):
    """Assemble the frozen shared-identity baton. Every load-bearing field is carried verbatim;
    the only additions are the derived binding digest (environment_sha256) and the seam-1
    selection aid. FREEZE BEFORE TRIALS: this is the identity every producer consumes."""
    closed(candidate, "revision tree cargo_lock_sha256 toolchains build_commands binaries",
           "candidate")
    require(type(candidate["binaries"]) is dict and set(candidate["binaries"]) == set(CANDIDATE_ROLES),
            "the frozen candidate needs exactly the six role binaries")
    for role, reference in candidate["binaries"].items():
        sha(reference["sha256"], f"candidate binary {role}")
    validate_environments(environments)
    for entry in environments:
        if entry["display_backend"] is not None:
            require(entry["id"] == native_environment_id(entry),
                    "native environment id must equal the deterministic derivation "
                    f"native-<backend>-<int-scale>-<WxH>: {entry['id']!r}")
    closed(input_manifest_ref, "path sha256", "input manifest reference")
    sha(input_manifest_ref["sha256"], "input manifest")
    sha(matrix_sha256, "matrix digest")
    return {"candidate": candidate,
            "environments": environments,
            "environment_sha256": environment_sha256(environments),
            "input_manifest": input_manifest_ref,
            "matrix_sha256": matrix_sha256,
            "selection": selection_aid(environments)}


def write_frozen(path, frozen):
    closed(frozen, FROZEN_FIELDS, "frozen identity")
    Path(path).write_text(json.dumps(frozen, indent=2, sort_keys=True) + "\n")


def load_frozen(path):
    frozen = parse(Path(path).read_bytes(), "frozen identity")
    closed(frozen, FROZEN_FIELDS, "frozen identity")
    require(frozen["environment_sha256"] == environment_sha256(frozen["environments"]),
            "frozen environment_sha256 does not bind the frozen environments array")
    return frozen


def cli_engine_pairs():
    """Every (family, surface) coverage pair on the cli/engine surfaces, from the governed
    inventory (the selection axis for the assembly core; GUI/native arrive via the baton)."""
    return sorted({((k[0], k[1]), k[3]) for k in case_inventory() if k[3] in ("cli", "engine")})


def merge_case_results(identity, reports):
    """Merge per-family case reports produced under the ONE frozen identity.

    Refusals: a sub-report whose candidate/input_manifest/matrix/environments differ from the
    frozen identity; a duplicate coverage row; a row whose bound identity fields differ; a row
    whose environment_id is not an id in the frozen 21-field array (the seam-1 ONE binding); a
    row whose binary is not the frozen candidate binary for its surface; conflicting artifact
    bytes at one path. Returns (rows, artifacts)."""
    env_ids = {e["id"] for e in identity["environments"]}
    role_shas = {role: ref["sha256"] for role, ref in identity["candidate"]["binaries"].items()}
    expected = {"candidate": identity["candidate"], "input_manifest": identity["input_manifest"],
                "matrix_sha256": identity["matrix_sha256"],
                "environments": identity["environments"]}
    rows, seen, artifacts = [], set(), {}
    for report in reports:
        for name, value in expected.items():
            require(canonical(report[name]) == canonical(value),
                    f"sub-report {name} differs from the frozen shared identity")
        for row in report["case_results"]:
            key = (row["case_id"], row["variant_id"], row["subcase_id"], row["surface"],
                   row["environment_id"])
            require(key not in seen, f"duplicate case row refused: {key}")
            seen.add(key)
            require(row["candidate_revision"] == identity["candidate"]["revision"] and
                    row["input_manifest_sha256"] == identity["input_manifest"]["sha256"] and
                    row["matrix_sha256"] == identity["matrix_sha256"],
                    f"case row identity differs from the frozen shared identity: {key}")
            require(row["environment_id"] in env_ids,
                    "case row environment_id is not an id in the frozen 21-field environments "
                    f"array: {key}")
            require(row["surface"] in role_shas and
                    row["binary_sha256"] == role_shas[row["surface"]],
                    f"case row binary is not the frozen candidate binary for its surface: {key}")
            rows.append(row)
        for entry in report["artifacts"]:
            held = artifacts.get(entry["path"])
            require(held is None or held["sha256"] == entry["sha256"],
                    f"conflicting artifact content for {entry['path']}")
            artifacts.setdefault(entry["path"], entry)
    return rows, list(artifacts.values())


def verify_recipe_membership(bundle, input_manifest_ref, rows):
    """Seam 2: every recipe a merged row's frozen fixture references must be PRESENT in the frozen
    input-manifest closure with its committed sha256 (U2 retains committed blob digests, so an
    uncommitted, drifted or relocated recipe cannot match). Refuse otherwise. Runs at assembly and
    again over the full merged set at capture."""
    entries = bundle.json(input_manifest_ref)
    for row in rows:
        recipe = bundle.json(row["fixture"])["recipe"]
        require(recipe in entries,
                "row recipe is not in the frozen input-manifest closure with its committed "
                f"sha256: {recipe['path']} "
                f"({row.get('case_id') or row.get('scenario_id') or 'row'})")


def assembled_report(capture, identity, *, case_results, measurements=(), native_observations=(),
                     producer_session="dev-agent-02"):
    """The ONE merged report skeleton with the placeholder lifecycle. The placeholders are
    explicit documents that FAIL their governed validators by construction (closed-set schema
    refusal), so the report stays incomplete under validate_report until the Phase-B runner fills
    gate_results, the independent reviewer replaces independent_review and the U5 release unit
    replaces release_notes. Completeness is computed only by validate_report, never here."""
    review_ref = capture.put({
        "schema": PENDING_REVIEW_SCHEMA,
        "status": "pending-section-9-independent-review",
        "producer_session": producer_session,
        "note": "Explicit placeholder: the independent reviewer replaces this reference with the "
                "datum.preferences.independent-review.v2 document; validate_review refuses this "
                "placeholder by construction.",
    })
    release_ref = capture.put({
        "schema": PENDING_RELEASE_SCHEMA,
        "status": "pending-U5-release-handoff",
        "note": "Explicit placeholder: the U5 release-handoff unit replaces this reference with "
                "the datum.preferences.release-handoff.v2 bundle; validate_release refuses this "
                "placeholder by construction (it is NOT review_only-gated).",
    })
    return {
        "schema": REPORT_SCHEMA,
        "origin": {"kind": "product-execution", "producer_session": producer_session},
        "candidate": identity["candidate"],
        "matrix_sha256": identity["matrix_sha256"],
        "input_manifest": identity["input_manifest"],
        "environments": identity["environments"],
        "artifacts": list(capture.inventory.values()),
        "case_results": list(case_results),
        "measurements": list(measurements),
        "gate_results": [],
        "native_observations": list(native_observations),
        "independent_review": review_ref,
        "release_notes": release_ref,
    }


def assemble_cases(bundle_root, identity, *, cli_binary, engine_binary, pairs=None,
                   producer_session="dev-agent-02"):
    """Run engine/CLI case families under the ONE frozen identity into one shared bundle and
    return the merged (still-incomplete) report. Every produced row is replayed through the
    governed per-case validators before merging. A family without a producer handler refuses --
    the gap is preserved, never skipped."""
    selected = list(pairs) if pairs is not None else cli_engine_pairs()
    require(bool(selected) and set(selected) <= set(cli_engine_pairs()),
            "pairs must select real cli/engine family coverage")
    reports = []
    for family, surface in selected:
        report, binaries, environments, keys = producer.build_family(
            bundle_root, family=family, surface=surface,
            cli_binary=cli_binary, engine_binary=engine_binary, identity=identity)
        bundle = Bundle(bundle_root, report["artifacts"])
        for row, key in zip(report["case_results"], keys):
            producer.validate_case_row(bundle, row, key, report, binaries, environments)
        reports.append(report)
    rows, artifacts = merge_case_results(identity, reports)
    verify_recipe_membership(Bundle(bundle_root, artifacts), identity["input_manifest"], rows)
    capture = Capture(bundle_root, inventory=artifacts)
    return assembled_report(capture, identity, case_results=rows,
                            producer_session=producer_session)


def main(argv=None):
    parser = argparse.ArgumentParser(description=__doc__)
    sub = parser.add_subparsers(dest="mode", required=True)
    freeze = sub.add_parser("freeze-identity",
                            help="freeze the shared identity baton (U2 candidate + U1 "
                                 "environments + closure + matrix) into frozen.json")
    freeze.add_argument("--revision", required=True, help="the pinned candidate revision")
    freeze.add_argument("--bundle-root", required=True, type=Path,
                        help="the ONE shared content-addressed capture bundle")
    freeze.add_argument("--native-specs", type=Path,
                        help="JSON list of native {coordinate,gpu,driver,compositor,"
                             "background_load} specs (U1 shape)")
    freeze.add_argument("--target", type=Path,
                        help="project-backed proof target whose mount is recorded "
                             "(default: the bundle root)")
    freeze.add_argument("--out", required=True, type=Path, help="frozen.json output path")
    cases = sub.add_parser("assemble-cases",
                           help="run the engine/CLI families under frozen.json into the merged "
                                "(incomplete) report")
    cases.add_argument("--frozen", required=True, type=Path, help="the frozen identity baton")
    cases.add_argument("--bundle-root", required=True, type=Path)
    cases.add_argument("--report", required=True, type=Path, help="merged report output path")
    cases.add_argument("--cli-binary", required=True, type=Path)
    cases.add_argument("--engine-binary", required=True, type=Path)
    args = parser.parse_args(argv)
    args.bundle_root.mkdir(parents=True, exist_ok=True)
    bundle_root = args.bundle_root.resolve()

    if args.mode == "freeze-identity":
        from global_preferences_acceptance.candidate_identity import build_candidate
        from global_preferences_acceptance.environment_record import build_environments
        capture = Capture(bundle_root)
        candidate, manifest_ref = build_candidate(ROOT, capture, args.revision)
        native = json.loads(args.native_specs.read_text()) if args.native_specs else []
        environments = build_environments(args.target or bundle_root, native)
        matrix_sha256 = digest(canonical(parse(
            (ROOT / producer.MATRIX_RELPATH).read_bytes(), "matrix")))
        frozen = freeze_identity(candidate, environments, manifest_ref, matrix_sha256)
        write_frozen(args.out, frozen)
        print(f"frozen identity written: {args.out}")
        print(f"environment-sha256: {frozen['environment_sha256']}")
        print(f"input-manifest-sha256: {frozen['input_manifest']['sha256']}")
        return 0

    frozen = load_frozen(args.frozen)
    report = assemble_cases(bundle_root, frozen,
                            cli_binary=args.cli_binary.resolve(),
                            engine_binary=args.engine_binary.resolve())
    args.report.write_text(json.dumps(report, indent=2, sort_keys=True))
    print(f"Assembled {len(report['case_results'])} engine/CLI case rows under the frozen "
          "identity.")
    print("Report is INCOMPLETE by construction (native_observations/measurements/gate_results/"
          "independent_review/release_notes pending); NO readiness is issued here -- only "
          "validate_report computes completeness.")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
