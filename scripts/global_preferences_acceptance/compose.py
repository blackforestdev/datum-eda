#!/usr/bin/env python3
"""GP-CM05E Phase-A U6: the production COMPOSITION core.

Compose the three per-corpus reports produced under the ONE frozen identity -- the engine/CLI case
rows (U3/assemble, including frontend's GUI-surface rows), the native_observation rows (frontend's
native capture), and the measurement rows (U4/measurement_assembly) -- into the single complete
``datum-global-preferences-production-evidence-v2`` report that Phase-B's ``validate_report``
consumes.

Consume VERBATIM, never re-produce. Each per-corpus report is read; its top-level identity is gated
against the frozen baton (so a sub-report built under a different candidate/environments/manifest/
matrix refuses); its artifacts merge with conflict detection; its rows are replayed through the
GOVERNED per-corpus validators (``observations.validate_cases`` / ``validate_native`` and the
governed measurement check mirrored from ``report.validate_report``), so a partial, duplicate,
identity-mismatched, unfrozen-environment, non-candidate-binary or otherwise malformed corpus
REFUSES LOUDLY -- never a silent partial PASS. The merged rows feed one ``assembled_report`` whose
placeholder lifecycle (gate_results / independent_review / release_notes) stays pending until the
Phase-B runner, the independent reviewer and U5's release handoff fill their real legs; composition
issues no readiness.

native_observations get their PRODUCTION assembly path here (the same consume-and-validate pattern);
at HEAD they had no producer outside the synthetic ``test_support`` fixture.
"""
import argparse
import json
import sys
from pathlib import Path

HERE = Path(__file__).resolve()
sys.path.insert(0, str(HERE.parents[1]))

from global_preferences_acceptance import measurements as meas  # noqa: E402
from global_preferences_acceptance import observations as obs  # noqa: E402
from global_preferences_acceptance.assemble import (  # noqa: E402
    assembled_report, load_frozen, verify_recipe_membership,
)
from global_preferences_acceptance.capture import Capture  # noqa: E402
from global_preferences_acceptance.identity import validate_environments  # noqa: E402
from global_preferences_acceptance.inputs import Bundle, canonical, parse, require  # noqa: E402

IDENTITY_FIELDS = ("candidate", "input_manifest", "matrix_sha256", "environments")
MEASUREMENT_FIELDS = ("budget_id variant_id environment_id candidate_revision binary_sha256 "
                      "input_manifest_sha256 matrix_sha256 trials")


def require_frozen_identity(report, identity, label):
    """Gate a per-corpus sub-report's load-bearing identity against the frozen baton: candidate,
    input manifest, matrix digest and the 21-field environments array must all match verbatim, so
    every composed corpus is bound to the ONE shared identity (the cross-corpus merge refusal)."""
    for name in IDENTITY_FIELDS:
        require(canonical(report[name]) == canonical(identity[name]),
                f"{label} {name} differs from the frozen shared identity")


def merge_artifacts(reports):
    """Union the per-corpus artifact inventories, refusing conflicting bytes at one path."""
    artifacts = {}
    for report in reports:
        for entry in report["artifacts"]:
            held = artifacts.get(entry["path"])
            require(held is None or held["sha256"] == entry["sha256"],
                    f"conflicting artifact content for {entry['path']}")
            artifacts.setdefault(entry["path"], entry)
    return list(artifacts.values())


def validate_measurement_corpus(report, bundle, environments, binaries):
    """Replay the governed measurement check (mirrors report.validate_report's measurement section:
    closed-shape rows, the complete 90-variant inventory with no duplicate, per-row identity, and
    the governed validate_trials). A partial or malformed measurement corpus refuses here."""
    expected = meas.measurement_inventory()
    rows = report["measurements"]
    require(len(rows) == len(expected), "measurement count differs")
    seen = set()
    for row in rows:
        require(set(row) == set(MEASUREMENT_FIELDS.split()), "measurement row shape differs")
        coordinate = obs.identity(row, report, environments, binaries)
        key = (row["budget_id"], row["variant_id"], coordinate)
        require(key in expected and key not in seen, "unknown or duplicate measurement variant")
        seen.add(key)
        meas.validate_trials(row, bundle)
    require(seen == expected, f"missing measurement variants: {len(expected - seen)}")


def compose_report(bundle_root, identity, *, case_report, measurement_report, native_report,
                   producer_session="dev-agent-02"):
    """Compose the three per-corpus reports into the one complete (still-pending-leg) v2 report under
    the frozen identity. Each corpus is identity-gated and replayed through its governed validator
    before it is composed; the placeholder lifecycle is preserved. No readiness is issued."""
    bundle_root = Path(bundle_root).resolve()
    for report, label in ((case_report, "case sub-report"),
                          (measurement_report, "measurement sub-report"),
                          (native_report, "native sub-report")):
        require_frozen_identity(report, identity, label)
    artifacts = merge_artifacts([case_report, measurement_report, native_report])
    bundle = Bundle(bundle_root, artifacts)
    environments = validate_environments(identity["environments"])
    binaries = {role: ref["sha256"] for role, ref in identity["candidate"]["binaries"].items()}

    case_rows = list(case_report["case_results"])
    native_rows = list(native_report["native_observations"])
    measurements = list(measurement_report["measurements"])

    # Replay the GOVERNED per-corpus validators against each sub-report (whose identity now equals
    # the frozen baton). Completeness + per-row identity/binary/env + well-formedness are enforced
    # here; a partial or tampered corpus refuses loudly rather than composing a silent partial.
    obs.validate_cases(case_report, bundle, environments, binaries)
    obs.validate_native(native_report, bundle, environments, binaries)
    validate_measurement_corpus(measurement_report, bundle, environments, binaries)

    # Seam 2 (recipe-in-manifest) over the composed case + native fixtures.
    verify_recipe_membership(bundle, identity["input_manifest"], case_rows + native_rows)

    capture = Capture(bundle_root, inventory=artifacts)
    return assembled_report(capture, identity, case_results=case_rows, measurements=measurements,
                            native_observations=native_rows, producer_session=producer_session)


def _read(path):
    return parse(Path(path).read_bytes(), f"per-corpus report {path}")


def main(argv=None):
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--frozen", required=True, type=Path, help="the frozen identity baton")
    parser.add_argument("--bundle-root", required=True, type=Path,
                        help="the ONE shared content-addressed bundle all corpora were produced into")
    parser.add_argument("--case-report", required=True, type=Path)
    parser.add_argument("--measurement-report", required=True, type=Path)
    parser.add_argument("--native-report", required=True, type=Path)
    parser.add_argument("--report", required=True, type=Path, help="composed report output path")
    parser.add_argument("--producer-session", default="dev-agent-02")
    args = parser.parse_args(argv)

    identity = load_frozen(args.frozen)
    composed = compose_report(
        args.bundle_root, identity,
        case_report=_read(args.case_report),
        measurement_report=_read(args.measurement_report),
        native_report=_read(args.native_report),
        producer_session=args.producer_session)
    args.report.write_text(json.dumps(composed, indent=2, sort_keys=True))
    print(f"Composed the complete v2 report: {len(composed['case_results'])} case rows, "
          f"{len(composed['measurements'])} measurements, "
          f"{len(composed['native_observations'])} native observations.")
    print("Placeholder lifecycle preserved (gate_results/independent_review/release_notes pending); "
          "NO readiness is issued here -- only validate_report computes completeness.")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
