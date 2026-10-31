#!/usr/bin/env python3
"""GP-CM05E Phase-A U5: the release-handoff ASSEMBLY CORE.

Assembles the ``datum.preferences.release-handoff.v2`` bundle that ``report.validate_release``
demands, and slots it into an assembled report's ``release_notes`` in place of the pending
placeholder. Because ``validate_release`` runs UNCONDITIONALLY inside ``validate_report`` (it is
NOT review_only-gated), the handoff is the hard Phase-B prerequisite: no report reaches any
readiness state until this reference is real and its referenced artifacts resolve.

**This unit binds references; it never authors content.** The frozen shared identity supplies the
load-bearing fields VERBATIM -- ``candidate_revision``, ``input_manifest_sha256``, the ordered
``environment_ids``, the six-role ``binaries`` and the governed ``boundaries`` (the ratified
``inventory.PRODUCT_BOUNDARY``). The eight support documents and the clean-user offline launch
proof are passed in as content-addressed references the caller has already retained; this module
checks their shape and that they resolve, and refuses a malformed support map. The AUTHORED
CONTENT of those support documents and the CAPTURE of the clean-user launch are separate
owner/campaign legs -- release-notes prose and any escalation/recovery wording are owner-reserved
(PM037 factory-default wording is untouchable), and the launch capture is a retained-evidence leg
(the frontend native lane when it is GUI-observable). Nothing here drafts owner wording.
"""
import sys
from pathlib import Path

HERE = Path(__file__).resolve()
sys.path.insert(0, str(HERE.parents[1]))

from global_preferences_acceptance.inputs import require  # noqa: E402
from global_preferences_acceptance.inventory import PRODUCT_BOUNDARY  # noqa: E402

RELEASE_SCHEMA = "datum.preferences.release-handoff.v2"
# The eight support categories report.validate_release requires, each mapped to a nonempty
# retained artifact (contract section 11; schema closed-set). Order fixed for a stable digest.
SUPPORT_CATEGORIES = ("settings_locations", "diagnostics", "errors", "recovery",
                      "compatibility", "storage", "retained_history", "known_exclusions")


def _resolves(capture, reference):
    """True iff ``reference`` is a {path, sha256} already retained in ``capture`` with that digest
    (so a dangling or drifted support/launch reference refuses here, not only at validate_release)."""
    require(type(reference) is dict and set(reference) == {"path", "sha256"},
            "a handoff reference must be a {path, sha256} artifact reference")
    held = capture.inventory.get(reference["path"])
    return held is not None and held["sha256"] == reference["sha256"]


def assemble_release_handoff(capture, identity, *, support, clean_user_launch):
    """Assemble the release-handoff.v2 document from the frozen identity and the caller's support /
    launch references, retain it in ``capture`` and return its reference.

    The identity fields are carried VERBATIM (environments/binaries/boundaries are exactly what the
    report carries, so ``validate_release`` matches). Refusals: a support map that is not exactly
    the eight categories; a support or launch reference that does not resolve in the capture. No
    content is authored -- the referenced documents are the caller's retained artifacts."""
    require(set(support) == set(SUPPORT_CATEGORIES),
            "support packet must map exactly the eight governed categories: "
            f"{', '.join(SUPPORT_CATEGORIES)}")
    for name in SUPPORT_CATEGORIES:
        require(_resolves(capture, support[name]),
                f"support reference does not resolve in the retained bundle: {name}")
    require(_resolves(capture, clean_user_launch),
            "clean-user offline launch proof does not resolve in the retained bundle")
    document = {
        "schema": RELEASE_SCHEMA,
        "candidate_revision": identity["candidate"]["revision"],
        "input_manifest_sha256": identity["input_manifest"]["sha256"],
        "environment_ids": [entry["id"] for entry in identity["environments"]],
        "binaries": identity["candidate"]["binaries"],
        "boundaries": PRODUCT_BOUNDARY,
        "support": {name: support[name] for name in SUPPORT_CATEGORIES},
        "clean_user_launch": clean_user_launch,
    }
    return capture.put(document)


def attach_release_handoff(report, capture, identity, *, support, clean_user_launch):
    """Replace an assembled report's PENDING release-notes placeholder with the real handoff
    reference, refreshing the artifact inventory so the handoff document (and the support/launch
    artifacts it binds) are carried. Returns the updated report. The rest of the placeholder
    lifecycle (gate_results, independent_review) is untouched -- this closes ONLY the release leg,
    never readiness."""
    require(report.get("release_notes") is not None, "report has no release-notes slot to replace")
    release_ref = assemble_release_handoff(capture, identity, support=support,
                                           clean_user_launch=clean_user_launch)
    return {**report, "release_notes": release_ref,
            "artifacts": list(capture.inventory.values())}
