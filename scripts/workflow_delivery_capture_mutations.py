"""Explicit synthetic refusal inputs, with the exact diagnostic each must reach."""

from workflow_delivery_io import canonical_json, parse_json
from workflow_delivery_capture_history import BASE_CASES, CURRENT_HISTORY_CASE, HISTORY_CASES
from workflow_delivery_capture_snapshots import SNAPSHOT_CASES
from workflow_delivery_interruption_case import INTERRUPTION_CASE
from workflow_delivery_readiness_input import READINESS_CASES
from workflow_delivery_review_input import REPORT_CASES, REPORT_FIXTURE, REPORT_ONLY_CASE, REVIEW_INPUT_CASES
from workflow_delivery_coverage_input import COVERAGE_INPUT_CASES
from workflow_delivery_specification_input import SPECIFICATION_INPUT_CASES
from workflow_delivery_review_phase_input import REVIEW_PHASE_CASES
from workflow_delivery_environment_cases import FIXTURE_VARIANTS, HEADLESS_VARIANTS, MIXED_VARIANT, SWAPPED_VARIANT, PRODUCT_HEADLESS_VARIANT

POLICY = "specs/workflow_delivery_policy.json"
CLI_ERROR_CASES = {"INFRA-S01-02.invalid-argument", "INFRA-S01-02.conflicting-views"}
TEXT_CASES = {"INFRA-S01-04.valid", "INFRA-S01-04.refused"}
RECOVERY_CASE = "INFRA-S06-07.fresh-after-refusal"
TRUST_CASES = {
    "INFRA-S06-01.missing-authority": ("authority", None),
    "INFRA-S06-01.missing-base": ("base", None),
    "INFRA-S06-01.moving-authority": ("authority", "HEAD"),
    "INFRA-S06-01.moving-base": ("base", "HEAD"),
}
CASES = {
    "INFRA-S01-01": (None, None),
    "INFRA-S01-04.valid": (None, None),
    "INFRA-S01-04.refused": ("src/unscoped.py", "WDQ-COVERAGE"),
    RECOVERY_CASE: (None, None),
    "INFRA-S02-09.outside-scope": ("src/unscoped.py", "WDQ-COVERAGE"),
    "INFRA-S02-10.category": (POLICY, "WDQ-POLICY"),
    "INFRA-S02-10.scope": (POLICY, "WDQ-POLICY"),
    "INFRA-S02-10.enrollment-sessions": (POLICY, "WDQ-POLICY"),
    "INFRA-S06-02.cli-gate": ("scripts/check_workflow_delivery.py", "WDQ-POLICY"),
    "INFRA-S06-02.selector-gate": ("scripts/workflow_delivery_selector.py", "WDQ-POLICY"),
    "INFRA-S06-11.selection": ("requested-environment.json", "WDQ-ENVIRONMENT"),
    "INFRA-S06-11.blob": ("selected-environment.json", "WDQ-ENVIRONMENT"),
    "INFRA-S06-11.authority-membership": (POLICY, "WDQ-POLICY"),
    "INFRA-S04-08.candidate-only-receipt": ("docs/reviews/owner.md", "WDQ-RECEIPT"),
}
CASES.update({key: (None, "INVOCATION-ERROR") for key in sorted(CLI_ERROR_CASES)})
CASES.update({key: ("requested-environment.json", "WDQ-ENVIRONMENT") for key in FIXTURE_VARIANTS})
CASES.update(HISTORY_CASES)
CASES[CURRENT_HISTORY_CASE] = (None, None)
CASES.update({key: (path, code) for key, (path, code, _) in BASE_CASES.items()})
CASES.update({key: (path, code) for key, (path, code, _) in REVIEW_PHASE_CASES.items()})
CASES.update({key: (path, code) for key, (path, code, _) in SPECIFICATION_INPUT_CASES.items()})
CASES.update({key: (path, code) for key, (path, code, _) in COVERAGE_INPUT_CASES.items()})
CASES.update({key: (path, code) for key, (path, code, _) in READINESS_CASES.items()})
CASES.update({key: (path, code) for key, (path, code, _) in REVIEW_INPUT_CASES.items()})
CASES.update({key: REVIEW_INPUT_CASES[REPORT_FIXTURE][:2] for key in sorted(REPORT_CASES)})
CASES[REPORT_ONLY_CASE] = ("requested-environment.json", "WDQ-ENVIRONMENT")
CASES.update(SNAPSHOT_CASES)
CASES[INTERRUPTION_CASE] = (None, None)
CASES[MIXED_VARIANT] = (None, None)
# This prepared-input case names the diagnostic environment context, not a
# candidate file to mutate. The exact selected bytes live in its retained bundle.
CASES[SWAPPED_VARIANT] = ("environment", "WDQ-ENVIRONMENT")
CASES[PRODUCT_HEADLESS_VARIANT] = ("environment", "WDQ-ENVIRONMENT")
CASES.update({key: ("trust" if value is None else field + "_ref", "WDQ-TRUST")
              for key, (field, value) in TRUST_CASES.items()})
CASES.update({key: (path, "WDQ-ENVIRONMENT") for key, (path, _) in HEADLESS_VARIANTS.items()})


def expected_detail(case_id):
    if case_id in TRUST_CASES:
        return ("owner-selected --authority-ref and trusted --base-ref required; no HEAD fallback"
                if TRUST_CASES[case_id][1] is None else
                "trusted refs must pin full commit IDs, not moving candidate names")
    if case_id in ("INFRA-S01-04.valid", RECOVERY_CASE):
        return None
    if case_id == "INFRA-S01-04.refused":
        return "production change has no promoted scope"
    if case_id == REPORT_ONLY_CASE:
        return "owner-selected environment authority required"
    if case_id in REPORT_CASES:
        return REVIEW_INPUT_CASES[REPORT_FIXTURE][2]
    if case_id in CLI_ERROR_CASES:
        return None
    if case_id == CURRENT_HISTORY_CASE:
        return None
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
    if case_id in (SWAPPED_VARIANT, PRODUCT_HEADLESS_VARIANT):
        return "headless environment cannot substitute for product native evidence"
    if case_id in FIXTURE_VARIANTS:
        return FIXTURE_VARIANTS[case_id]
    if case_id.startswith("INFRA-S02-10."):
        return "policy change requires owner-controlled promotion"
    if case_id.startswith("INFRA-S06-02."):
        return "gate/wiring change requires owner-controlled promotion"
    return {"INFRA-S01-01": None,
            "INFRA-S04-08.candidate-only-receipt": "candidate receipt differs from promoted authority",
            "INFRA-S06-11.authority-membership": "policy change requires owner-controlled promotion",
            "INFRA-S02-09.outside-scope": "production change has no promoted scope",
            "INFRA-S06-11.selection": "environment selection change requires owner promotion",
            "INFRA-S06-11.blob": "artifact hash mismatch"}[case_id]


def changed_bytes(case_id, raw):
    target, _ = CASES[case_id]
    if case_id == "INFRA-S04-08.candidate-only-receipt":
        return raw + b"\nCandidate-only synthetic receipt amendment.\n"
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
    elif case_id == "INFRA-S06-11.authority-membership":
        value["enrolled"].append(dict(value["enrolled"][0], frontier_key="NEXT"))
    else:
        raise ValueError("no implemented mutation for " + case_id)
    return canonical_json(value)


def text_environment(environment):
    value = {key: data for key, data in environment.items() if key not in ("DISPLAY", "WAYLAND_DISPLAY")}
    value.update(TERM="dumb", NO_COLOR="1", CLICOLOR="0", FORCE_COLOR="0")
    return value


def prepare_trust_input(root, case_id):
    """Change only nonce-owned fixture trust, never repair the live checkout."""
    from workflow_delivery_capture_state import git
    field, value = TRUST_CASES[case_id]
    key = "datum.workflowDelivery" + field.title() + "Ref"
    before = git(root, "config", "--local", "--get", key).decode().strip()
    if value is None:
        git(root, "config", "--local", "--unset-all", key)
    else:
        git(root, "config", "--local", "--replace-all", key, value)
    return {"case_id": case_id, "local_trust_key": key, "before": before, "after": value}
