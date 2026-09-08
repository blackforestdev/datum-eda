"""Explicit synthetic refusal inputs, with the exact diagnostic each must reach."""

from workflow_delivery_io import canonical_json, parse_json
from workflow_delivery_capture_history import BASE_CASES, HISTORY_CASES
from workflow_delivery_capture_snapshots import SNAPSHOT_CASES
from workflow_delivery_interruption_case import INTERRUPTION_CASE
from workflow_delivery_readiness_input import READINESS_CASES
from workflow_delivery_review_input import REVIEW_INPUT_CASES
from workflow_delivery_coverage_input import COVERAGE_INPUT_CASES
from workflow_delivery_specification_input import SPECIFICATION_INPUT_CASES
from workflow_delivery_review_phase_input import REVIEW_PHASE_CASES
from workflow_delivery_environment_cases import FIXTURE_VARIANTS, HEADLESS_VARIANTS, MIXED_VARIANT, SWAPPED_VARIANT

POLICY = "specs/workflow_delivery_policy.json"
CASES = {
    "INFRA-S01-01": (None, None),
    "INFRA-S02-09.outside-scope": ("src/unscoped.py", "WDQ-COVERAGE"),
    "INFRA-S02-10.category": (POLICY, "WDQ-POLICY"),
    "INFRA-S02-10.scope": (POLICY, "WDQ-POLICY"),
    "INFRA-S02-10.enrollment-sessions": (POLICY, "WDQ-POLICY"),
    "INFRA-S06-02.cli-gate": ("scripts/check_workflow_delivery.py", "WDQ-POLICY"),
    "INFRA-S06-02.selector-gate": ("scripts/workflow_delivery_selector.py", "WDQ-POLICY"),
    "INFRA-S06-11.selection": ("requested-environment.json", "WDQ-ENVIRONMENT"),
    "INFRA-S06-11.blob": ("selected-environment.json", "WDQ-ENVIRONMENT"),
}
CASES.update({key: ("requested-environment.json", "WDQ-ENVIRONMENT") for key in FIXTURE_VARIANTS})
CASES.update(HISTORY_CASES)
CASES.update({key: (path, code) for key, (path, code, _) in BASE_CASES.items()})
CASES.update({key: (path, code) for key, (path, code, _) in REVIEW_PHASE_CASES.items()})
CASES.update({key: (path, code) for key, (path, code, _) in SPECIFICATION_INPUT_CASES.items()})
CASES.update({key: (path, code) for key, (path, code, _) in COVERAGE_INPUT_CASES.items()})
CASES.update({key: (path, code) for key, (path, code, _) in READINESS_CASES.items()})
CASES.update({key: (path, code) for key, (path, code, _) in REVIEW_INPUT_CASES.items()})
CASES.update(SNAPSHOT_CASES)
CASES[INTERRUPTION_CASE] = (None, None)
CASES[MIXED_VARIANT] = (None, None)
# This prepared-input case names the diagnostic environment context, not a
# candidate file to mutate. The exact selected bytes live in its retained bundle.
CASES[SWAPPED_VARIANT] = ("environment", "WDQ-ENVIRONMENT")
CASES.update({key: (path, "WDQ-ENVIRONMENT") for key, (path, _) in HEADLESS_VARIANTS.items()})


def expected_detail(case_id):
    if case_id in BASE_CASES:
        return BASE_CASES[case_id][2]
    if case_id in REVIEW_PHASE_CASES:
        return REVIEW_PHASE_CASES[case_id][2]
    if case_id in SPECIFICATION_INPUT_CASES:
        return SPECIFICATION_INPUT_CASES[case_id][2]
    if case_id in COVERAGE_INPUT_CASES:
        return COVERAGE_INPUT_CASES[case_id][2]
    if case_id in REVIEW_INPUT_CASES:
        return REVIEW_INPUT_CASES[case_id][2]
    if case_id in READINESS_CASES:
        return READINESS_CASES[case_id][2]
    if case_id == INTERRUPTION_CASE:
        return None
    if case_id in HISTORY_CASES:
        return "production change has no promoted scope"
    if case_id in HEADLESS_VARIANTS:
        return HEADLESS_VARIANTS[case_id][1]
    if case_id == MIXED_VARIANT:
        return None
    if case_id == SWAPPED_VARIANT:
        return "headless environment cannot substitute for product native evidence"
    if case_id in FIXTURE_VARIANTS:
        return FIXTURE_VARIANTS[case_id]
    if case_id.startswith("INFRA-S02-10."):
        return "policy change requires owner-controlled promotion"
    if case_id.startswith("INFRA-S06-02."):
        return "gate/wiring change requires owner-controlled promotion"
    return {"INFRA-S01-01": None,
            "INFRA-S02-09.outside-scope": "production change has no promoted scope",
            "INFRA-S06-11.selection": "environment selection change requires owner promotion",
            "INFRA-S06-11.blob": "artifact hash mismatch"}[case_id]


def changed_bytes(case_id, raw):
    target, _ = CASES[case_id]
    if target == "src/unscoped.py":
        return b"# Synthetic unpermitted implementation input.\n"
    if case_id.startswith("INFRA-S06-02."):
        # Modified fixture gate is data being rejected, never the executed runner.
        return raw + b"\n# Synthetic candidate-only gate change.\n"
    value = parse_json(raw, target)
    if case_id == "INFRA-S02-10.category":
        value["coverage"]["rows"][0]["category"] = "product"
    elif case_id == "INFRA-S02-10.scope":
        value["coverage"]["source_scopes"] = [{"frontier_key": "NEXT", "step_ids": ["NEXT-C01"],
            "paths": ["src"], "boundary_ref": value["decision_ref"]}]
    elif case_id == "INFRA-S02-10.enrollment-sessions":
        value["enrolled"][0]["implementation_sessions"] = ["candidate-only-session"]
    elif case_id == "INFRA-S06-11.selection":
        value["environments"] = []
    elif case_id == "INFRA-S06-11.blob":
        value["toolchain"] = "synthetic candidate-only toolchain"
    else:
        raise ValueError("no implemented mutation for " + case_id)
    return canonical_json(value)
