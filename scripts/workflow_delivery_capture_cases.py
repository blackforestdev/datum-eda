"""Initial real-process case recipes; explicitly not the complete INFRA matrix."""

from pathlib import Path
import sys

from workflow_delivery_capture_assertions import evaluate_capture, evaluate_invocation_error
from workflow_delivery_capture_assertions import evaluate_text_output, recovery_link
from workflow_delivery_capture_command import capture_command, save
from workflow_delivery_capture_fixture import restore_fixture
from workflow_delivery_capture_state import git
from workflow_delivery_capture_history import BASE_CASES, HISTORY_CASES, prepare_base_input, prepare_history
from workflow_delivery_capture_history import CURRENT_HISTORY_CASE, prepare_current_history
from workflow_delivery_capture_snapshots import SNAPSHOT_CASES, prepare_snapshots, snapshot_expectation
from workflow_delivery_capture_mutations import CASES, CLI_ERROR_CASES, changed_bytes, expected_detail
from workflow_delivery_capture_mutations import TEXT_CASES, RECOVERY_CASE, text_environment
from workflow_delivery_capture_mutations import TRUST_CASES, prepare_trust_input
from workflow_delivery_io import sha256
from workflow_delivery_interruption_case import INTERRUPTION_CASE, evaluate_interruption
from workflow_delivery_environment_cases import case_uses_prepared_input
from workflow_delivery_coverage_input import BOUNDARY_CASES, COVERAGE_INPUT_CASES, coverage_expectation, mutate_external_boundary
from workflow_delivery_review_input import PROOF_INPUT_CASES, REPORT_CASES, REPORT_ONLY_CASE, proof_expectation
from workflow_delivery_review_phase_input import REVIEW_PHASE_CASES, review_phase_expectation


def run_case(packet, destination, runtime_root, environment, *, case_id, surface):
    if case_id not in CASES or surface not in ("C", "R", "S-check", "S-details"):
        raise ValueError("only explicitly implemented case/surface combinations may run")
    if case_id in (HISTORY_CASES | BASE_CASES) and surface != "R":
        raise ValueError("history certification requires exact-candidate surface R")
    if case_id == CURRENT_HISTORY_CASE and surface == "R":
        raise ValueError("current transaction success is not history certification")
    if case_id in CLI_ERROR_CASES and surface not in ("C", "R"):
        raise ValueError("invocation-error captures require CLI surface C or R")
    if case_id in REPORT_CASES and surface not in ("C", "R"):
        raise ValueError("report-mode captures require CLI surface C or R")
    if case_id in SNAPSHOT_CASES:
        snapshot_expectation(case_id, surface)
    destination, runtime_root = Path(destination).resolve(), Path(runtime_root).resolve(strict=True)
    destination.mkdir(exist_ok=False)
    if case_id == RECOVERY_CASE:
        run_case(packet, destination / "prior-refusal", runtime_root, environment,
                 case_id="INFRA-S02-09.outside-scope", surface=surface)
    root, output = destination / "fixture", destination / "capture"
    recipe = restore_fixture(packet, root)
    prepared_input = case_uses_prepared_input(case_id, recipe)
    target, expected = CASES[case_id]
    detail = None if case_id in SNAPSHOT_CASES else expected_detail(case_id)
    if case_id == "INFRA-S06-11.authority-membership" and surface.startswith("S-"):
        # Selector enrollment/declaration consistency precedes pinned-policy
        # comparison. Retain that exact earlier refusal, not a CLI alias.
        detail = "enrolled delivery declaration removed"
    if case_id in COVERAGE_INPUT_CASES:
        target, expected, detail = coverage_expectation(case_id, surface)
    if case_id in PROOF_INPUT_CASES:
        target, expected, detail = proof_expectation(case_id, surface)
    if case_id in REVIEW_PHASE_CASES:
        target, expected, detail = review_phase_expectation(case_id, surface)
    mutation = {"case_id": case_id, "path": target, "before_sha256": None, "after_hex": None,
                "prepared_fixture_variant": recipe.get("fixture_variant")}
    if case_id in TRUST_CASES:
        mutation = prepare_trust_input(root, case_id)
    elif case_id in BOUNDARY_CASES:
        mutation = mutate_external_boundary(root, case_id)
        if surface == "R":
            git(root, "-c", "user.name=Fixture", "-c", "user.email=fixture@example.invalid",
                "commit", "-m", "test(workflow): retain synthetic external transaction\n\n" + case_id)
    elif case_id in SNAPSHOT_CASES:
        mutation = prepare_snapshots(root, case_id)
        target, expected, detail = snapshot_expectation(case_id, surface)
    elif case_id == CURRENT_HISTORY_CASE:
        mutation = prepare_current_history(root, recipe["head"], destination / "history.bundle")
    elif case_id in BASE_CASES:
        mutation = prepare_base_input(root, case_id, recipe["head"], destination / "history.bundle")
    elif case_id in HISTORY_CASES:
        mutation = prepare_history(root, case_id, recipe["head"], destination / "history.bundle")
    elif target and not prepared_input:
        path = root / target
        raw = path.read_bytes() if path.exists() else None
        mutation["before_sha256"] = sha256(raw) if raw is not None else None
        changed = changed_bytes(case_id, raw)
        path.write_bytes(changed)
        mutation["after_hex"] = changed.hex()
        git(root, "add", "--", target)
        if surface == "R":
            git(root, "-c", "user.name=Fixture", "-c", "user.email=fixture@example.invalid",
                "commit", "-m", "test(workflow): retain synthetic refusal input\n\n"
                "Prepare " + case_id + " for exact candidate-view regression.\n"
                "No real owner approval, native proof or production change.\n")
    save(destination / "mutation.json", mutation)
    candidate = git(root, "rev-parse", "HEAD").decode().strip()
    selector = surface.startswith("S-")
    script = runtime_root / "scripts" / ("project_status.py" if selector else "check_workflow_delivery.py")
    arguments = ["--root", str(root)]
    if selector:
        arguments += ["--json", surface.removeprefix("S-")]
        if surface == "S-details":
            arguments += ["NEXT"]
    else:
        arguments += ["--report-only" if case_id == REPORT_ONLY_CASE else "--enforce",
                      "--authority-ref", recipe["head"], "--base-ref", mutation.get("base_ref", recipe["head"]),
                      "--environment-path", "requested-environment.json"]
        arguments += ["--candidate-ref", candidate] if surface == "R" else ["--staged"]
    invocation_error = None
    if case_id in TRUST_CASES and not selector:
        field, value = TRUST_CASES[case_id]
        position = arguments.index("--" + field + "-ref")
        if value is None:
            del arguments[position:position + 2]
        else:
            arguments[position + 1] = value
    if case_id == "INFRA-S01-02.invalid-argument":
        arguments += ["--synthetic-invalid-option"]
        invocation_error = "unrecognized arguments: --synthetic-invalid-option"
    elif case_id == "INFRA-S01-02.conflicting-views":
        arguments += ["--candidate-ref", candidate] if surface == "C" else ["--staged"]
        invocation_error = ("argument --candidate-ref: not allowed with argument --staged" if surface == "C"
                            else "argument --staged: not allowed with argument --candidate-ref")
    command = [sys.executable, "-I", "-S", "-B",
        str(runtime_root / "scripts/workflow_delivery_observe_python.py"),
        "--script", str(script), "--source-root", str(runtime_root),
        "--output", str(output / "calls.jsonl"), "--function", "main",
        "--function", "selector_failures", "--", *arguments]
    trigger = ({"trace": "calls.jsonl", "source": str(runtime_root / "scripts/workflow_delivery_selector.py")
                if selector else str(script), "function": "selector_failures" if selector else "main"}
               if case_id == INTERRUPTION_CASE else None)
    observed_environment = text_environment(environment) if case_id in TEXT_CASES else environment
    capture_command(root, output, command, observed_environment, fixture_nonce=recipe["fixture_nonce"],
                    case_id=case_id, surface="S" if selector else surface,
                    untracked_roots=recipe["untracked_roots"], timeout=30, interrupt_on_call=trigger)
    try:
        if invocation_error is not None:
            assessment = evaluate_invocation_error(output, case_id=case_id, surface=surface,
                                                  script=script, expected_error=invocation_error)
        elif trigger is not None:
            assessment = evaluate_interruption(output, case_id=case_id,
                                               surface="S" if selector else surface, trigger=trigger)
        else:
            diagnostics = None
            if selector and case_id.startswith("INFRA-S02-08."):
                diagnostics = [{"code": "NEXT", "path": None, "detail": detail},
                    {"code": "WDQ-COVERAGE", "path": "external/owned/input.py",
                     "detail": "production change has no promoted scope"}]
            assessment = evaluate_capture(output, case_id=case_id, surface="S" if selector else surface,
                                      expected_code=expected, script=script, expected_path=target,
                                      expected_detail=detail, expected_diagnostics=diagnostics,
                                      expected_mode="report-only" if case_id == REPORT_ONLY_CASE else "enforce")
    except Exception as error:
        save(destination / "assessment-error.json", {"type": type(error).__name__, "detail": str(error),
                                                   "acceptance_asserted": False})
        raise
    if case_id in TEXT_CASES:
        assessment["text_evidence"] = evaluate_text_output(output, surface="S" if selector else surface)
    if case_id == RECOVERY_CASE:
        assessment["recovery"] = recovery_link(destination / "prior-refusal", output,
                                              case_id=case_id, surface="S" if selector else surface)
    assessment.update(selector_operation=surface if selector else None, complete_matrix=False)
    save(destination / "assessment.json", assessment)
    return assessment
