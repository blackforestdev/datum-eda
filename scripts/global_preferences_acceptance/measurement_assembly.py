#!/usr/bin/env python3
"""GP-CM05E Phase-A U4: the measurement ASSEMBLY CORE.

The measurement analog of :func:`assemble.assemble_cases`. It consumes the ONE frozen
shared-identity baton (``assemble.load_frozen``) VERBATIM, drives the headless measurement
budgets through the *existing* ``measurements_producer`` builders -- release-query, durable
(standalone + daemon), project-genesis/validation and storage-growth, every one reached through
the single ``measurements_producer._produce`` runner -- replays each produced row through the governed
validator (``measurements.validate_trials``), merges the rows under the frozen identity with the
U3 merge refusals, and feeds them into the ONE ``assembled_report`` skeleton whose placeholder
lifecycle keeps the report non-ready.

**Frozen-identity consumption, not recompute.** The producer's ``main()`` recomputes the row
identity from the live checkout: the git ``HEAD`` revision, the on-disk binary sha256, the matrix
digest, a synthetic input manifest and a literal ``headless-linux-x86_64`` environment id. This
module REPLACES that block with consumption of the frozen baton -- ``candidate.revision``,
``candidate.binaries[role].sha256``, ``input_manifest.sha256``, ``matrix_sha256`` and an id drawn
from the frozen 21-field ``environments`` array are all carried verbatim, never recomputed. The
producer builders themselves are reused unchanged (no cloned runner); only the identity they are
handed changes. The global pre-warm still runs exactly once before the first measured budget.

**No readiness here.** A partial corpus -- fewer than the governed 90 measurement variants, or the
still-pending gate/review/release placeholders -- is refused by ``validate_report``. This module
only assembles the skeleton; completeness is computed solely by the governed validator.
"""
import sys
from pathlib import Path

HERE = Path(__file__).resolve()
sys.path.insert(0, str(HERE.parents[1]))

from global_preferences_acceptance import measurements as meas  # noqa: E402
from global_preferences_acceptance import measurements_producer as mp  # noqa: E402
from global_preferences_acceptance.assemble import assembled_report, load_frozen  # noqa: E402
from global_preferences_acceptance.capture import Capture  # noqa: E402
from global_preferences_acceptance.environment_record import HEADLESS_ID  # noqa: E402
from global_preferences_acceptance.inputs import Bundle, require  # noqa: E402

# The GUI measurement budgets, exactly as ``observations.identity`` resolves the gui surface
# (kept local to this unit's two files -- no governed-inventory edit). A row on any other budget
# binds the cli surface.
GUI_BUDGETS = {"gui-feedback", "native-window-open", "window-lifecycle"}

# The headless budget selectors ``measurements_producer._produce`` understands, in corpus order. The GUI
# budgets (gui-feedback/native-window-open/window-lifecycle) arrive via the frontend native
# harness on the same frozen baton and are assembled by that lane, never cloned here.
HEADLESS_BUDGETS = ("release-query/defaults", "release-query/populated",
                    "durable/set", "durable/reset", "daemon/set", "daemon/reset",
                    "project", "storage")


def measurement_role(budget_id):
    """The candidate-binary role a measurement row binds, mirroring ``observations.identity``: the
    GUI budgets run on the ``gui`` surface, every headless budget on the ``cli`` surface."""
    return "gui" if budget_id in GUI_BUDGETS else "cli"


def frozen_common(frozen, *, role="cli", environment_id=HEADLESS_ID):
    """The producer ``common`` kwargs derived from the frozen baton VERBATIM -- the frozen-identity
    replacement for the producer ``main()``'s recompute-from-HEAD/binary/matrix block. ``role``
    selects the candidate binary; ``environment_id`` must be an id present in the frozen 21-field
    ``environments`` array (the headless id by default). Nothing here is computed from the live
    checkout."""
    binaries = frozen["candidate"]["binaries"]
    require(role in binaries, f"frozen candidate has no {role} binary")
    require(environment_id in {entry["id"] for entry in frozen["environments"]},
            f"environment id is not an id in the frozen 21-field environments array: {environment_id!r}")
    return {"candidate_revision": frozen["candidate"]["revision"],
            "binary_sha256": binaries[role]["sha256"],
            "input_manifest_sha256": frozen["input_manifest"]["sha256"],
            "matrix_sha256": frozen["matrix_sha256"],
            "environment_id": environment_id}


def merge_measurements(identity, measurements):
    """Merge measurement rows produced under the ONE frozen identity -- the U3 merge semantics
    (:func:`assemble.merge_case_results`) applied to measurement rows.

    Refusals: a duplicate ``(budget_id, variant_id, environment_id)`` coverage row; a row whose
    candidate/input-manifest/matrix identity differs from the frozen baton; a row whose
    ``environment_id`` is not an id in the frozen 21-field ``environments`` array; a row whose
    ``binary_sha256`` is not the frozen candidate binary for the row's budget-derived surface role
    (``observations.identity``'s gui/cli resolution). Returns the merged rows."""
    env_ids = {entry["id"] for entry in identity["environments"]}
    role_shas = {role: ref["sha256"] for role, ref in identity["candidate"]["binaries"].items()}
    rows, seen = [], set()
    for row in measurements:
        key = (row["budget_id"], row["variant_id"], row["environment_id"])
        require(key not in seen, f"duplicate measurement row refused: {key}")
        seen.add(key)
        require(row["candidate_revision"] == identity["candidate"]["revision"] and
                row["input_manifest_sha256"] == identity["input_manifest"]["sha256"] and
                row["matrix_sha256"] == identity["matrix_sha256"],
                f"measurement row identity differs from the frozen shared identity: {key}")
        require(row["environment_id"] in env_ids,
                "measurement row environment_id is not an id in the frozen 21-field environments "
                f"array: {key}")
        role = measurement_role(row["budget_id"])
        require(role in role_shas and row["binary_sha256"] == role_shas[role],
                f"measurement row binary is not the frozen candidate {role} binary: {key}")
        rows.append(row)
    return rows


def assemble_measurements(bundle_root, identity, *, binary, scratch, budgets=HEADLESS_BUDGETS,
                          daemon_binary=None, timeout=15, validate=True, prewarm_once=True,
                          producer_session="dev-agent-02"):
    """Drive the selected headless measurement budgets under the ONE frozen identity into one
    shared content-addressed bundle and return the merged (still-incomplete) measurement report
    skeleton.

    The producer's builders are reused unchanged through ``measurements_producer._produce`` (no
    cloned runner); the only substitution is frozen-identity consumption (:func:`frozen_common`) in
    place of the producer ``main()``'s HEAD/binary/matrix/env recompute. The global pre-warm runs
    exactly once before the first budget (``prewarm_once``), never per budget. Every produced row is
    replayed through the governed ``measurements.validate_trials`` before merge. No readiness is
    issued."""
    bundle_root = Path(bundle_root).resolve()
    bundle_root.mkdir(parents=True, exist_ok=True)
    scratch = Path(scratch).resolve()
    scratch.mkdir(parents=True, exist_ok=True)
    capture = Capture(bundle_root)
    common = frozen_common(identity, role="cli")  # every headless budget is the cli surface
    if prewarm_once:
        mp.prewarm(capture, binary, common["candidate_revision"], common["binary_sha256"],
                   timeout, scratch)
    produced = []
    for selector in budgets:
        produced.extend(mp._produce(selector, capture, binary, common, timeout, scratch,
                                    daemon_binary=daemon_binary))
    if validate:
        bundle = Bundle(bundle_root, list(capture.inventory.values()))
        for measurement in produced:
            meas.validate_trials(measurement, bundle)
    rows = merge_measurements(identity, produced)
    return assembled_report(capture, identity, case_results=(), measurements=rows,
                            producer_session=producer_session)


def main(argv=None):
    import argparse
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--frozen", required=True, type=Path, help="the frozen identity baton")
    parser.add_argument("--bundle-root", required=True, type=Path)
    parser.add_argument("--report", required=True, type=Path, help="merged report output path")
    parser.add_argument("--binary", required=True, type=Path,
                        help="the frozen candidate cli release binary (its sha256 must match "
                             "the frozen candidate)")
    parser.add_argument("--daemon-binary", type=Path, help="eda-engine-daemon (daemon/* budgets)")
    parser.add_argument("--budget", action="append", choices=list(HEADLESS_BUDGETS),
                        help="headless budget selector(s); default = the full headless corpus")
    parser.add_argument("--scratch", required=True, type=Path)
    parser.add_argument("--timeout", type=float, default=15)
    args = parser.parse_args(argv)

    import json
    identity = load_frozen(args.frozen)
    report = assemble_measurements(
        args.bundle_root, identity, binary=args.binary.resolve(),
        daemon_binary=args.daemon_binary.resolve() if args.daemon_binary else None,
        scratch=args.scratch, budgets=tuple(args.budget) if args.budget else HEADLESS_BUDGETS,
        timeout=args.timeout)
    args.report.write_text(json.dumps(report, indent=2, sort_keys=True))
    print(f"Assembled {len(report['measurements'])} measurement rows under the frozen identity.")
    print("Report is INCOMPLETE by construction (the full 90-variant corpus, case_results, "
          "gate_results, independent_review and release_notes are pending); NO readiness is "
          "issued here -- only validate_report computes completeness.")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
