"""Malformed authority INPUT recipes; never owner promotion or proof repair."""

from copy import deepcopy
from workflow_delivery_readiness_input import READINESS_CASES
from workflow_delivery_review_input import REPORT_CASES, REPORT_FIXTURE, REVIEW_INPUT_CASES
from workflow_delivery_coverage_input import COVERAGE_INPUT_CASES
from workflow_delivery_specification_input import SPECIFICATION_INPUT_CASES
from workflow_delivery_review_phase_input import REVIEW_PHASE_CASES

FIXTURE_VARIANTS = {
    "INFRA-S06-09.missing-key": "nonempty array required",
    "INFRA-S06-09.duplicate-key": "duplicate identity",
    "INFRA-S06-09.unknown-key": "must match every trusted enrolled key exactly",
    "INFRA-S06-12.selection-version": "integer environment selection version 2 required",
    "INFRA-S06-12.selection-kind": "environment selection kind required",
    "INFRA-S06-12.selection-fields": "closed fields required",
}
MIXED_VARIANT = "INFRA-S06-08.mixed"
SWAPPED_VARIANT = "INFRA-S06-10.swapped"
PRODUCT_HEADLESS_VARIANT = "INFRA-S06-15.product-headless"
HEADLESS_VARIANTS = {
    "INFRA-S06-12.environment-version": ("environment", "integer headless environment version 2 required"),
    "INFRA-S06-12.environment-kind": ("environment.kind", "expected one of"),
    "INFRA-S06-12.environment-fields": ("environment", "closed fields required"),
    "INFRA-S06-13.toolchain": ("environment.toolchain", "environment/build toolchain mismatch"),
    "INFRA-S06-13.interpreter": ("environment.tools.interpreter", "observed interpreter differs from build receipt binary"),
    "INFRA-S06-14.pipes-dimensions": ("environment.terminal_size", "pipes have no terminal dimensions"),
    "INFRA-S06-14.pty-dimensions": ("environment.terminal_size", "PTY requires observed positive columns/rows"),
}
MIXED_VARIANTS = {MIXED_VARIANT, SWAPPED_VARIANT, PRODUCT_HEADLESS_VARIANT} | set(HEADLESS_VARIANTS)
PREPARED_VARIANTS = set(FIXTURE_VARIANTS) | MIXED_VARIANTS | set(READINESS_CASES) | set(REVIEW_INPUT_CASES)
PREPARED_VARIANTS.update(COVERAGE_INPUT_CASES)
PREPARED_VARIANTS.update(SPECIFICATION_INPUT_CASES)
PREPARED_VARIANTS.update(REVIEW_PHASE_CASES)


def malformed_selection(selection, variant):
    value = deepcopy(selection)
    if variant == "INFRA-S06-09.missing-key":
        value["environments"] = []
    elif variant == "INFRA-S06-09.duplicate-key":
        value["environments"].append(deepcopy(value["environments"][0]))
    elif variant == "INFRA-S06-09.unknown-key":
        value["environments"][0]["frontier_key"] = "UNKNOWN"
    elif variant == "INFRA-S06-12.selection-version":
        value["schema_version"] = True
    elif variant == "INFRA-S06-12.selection-kind":
        value["kind"] = "invalid-selection-kind"
    elif variant == "INFRA-S06-12.selection-fields":
        value["unexpected"] = "synthetic malformed authority input"
    else:
        raise ValueError("explicit implemented environment fixture variant required")
    return value


def case_uses_prepared_input(case_id, recipe):
    variant = recipe.get("fixture_variant")
    expected = REPORT_FIXTURE if case_id in REPORT_CASES else case_id if case_id in PREPARED_VARIANTS else None
    if variant != expected:
        raise ValueError("case does not match retained fixture variant")
    return expected is not None
