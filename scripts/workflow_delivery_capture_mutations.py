"""Explicit synthetic refusal inputs, with the exact diagnostic each must reach."""

from workflow_delivery_io import canonical_json, parse_json
from workflow_delivery_environment_cases import FIXTURE_VARIANTS

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


def expected_detail(case_id):
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
