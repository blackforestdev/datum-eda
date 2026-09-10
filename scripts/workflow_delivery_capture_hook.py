"""Retain real owner-hook process observations on restored synthetic fixtures.

This exercises the hook's pinned observer path. Complete contract-handler
accounting and child/tool closure still remain separate I03 obligations.
"""

from pathlib import Path

from workflow_delivery_capture_assertions import capture_envelope, observer_calls
from workflow_delivery_capture_assertions import evaluate_text_output, recovery_link
from workflow_delivery_capture_command import capture_command, save
from workflow_delivery_capture_fixture import restore_fixture
from workflow_delivery_capture_mutations import CASES, CLI_ERROR_CASES, changed_bytes, expected_detail
from workflow_delivery_capture_mutations import TEXT_CASES, RECOVERY_CASE, text_environment
from workflow_delivery_capture_mutations import TRUST_CASES, prepare_trust_input
from workflow_delivery_capture_history import BASE_CASES, HISTORY_CASES
from workflow_delivery_capture_history import CURRENT_HISTORY_CASE, prepare_current_history
from workflow_delivery_capture_snapshots import SNAPSHOT_CASES, prepare_snapshots, snapshot_expectation
from workflow_delivery_capture_state import git, file_state
from workflow_delivery_io import parse_json, sha256
from workflow_delivery_shapes import require
from workflow_delivery_environment_cases import case_uses_prepared_input
from workflow_delivery_support_bundle import prepare_bundle, verify_bundle
from workflow_delivery_support_store import support_locations
from workflow_delivery_interruption_case import INTERRUPTION_CASE, evaluate_interruption
from workflow_delivery_review_input import PROOF_INPUT_CASES, REPORT_CASES, proof_expectation
from workflow_delivery_review_phase_input import REVIEW_PHASE_CASES, review_phase_expectation
from workflow_delivery_coverage_input import BOUNDARY_CASES, mutate_external_boundary


HOOK_TRUST_CASES = {
    "INFRA-S06-02.missing-runner-selection": "missing runner",
    "INFRA-S06-02.missing-runner-file": "runtime module set differs; caches and unpinned imports are forbidden",
    "INFRA-S06-02.changed-runner": "runner bytes differ from pinned Git authority: scripts/check_workflow_delivery.py",
    "INFRA-S06-02.changed-bootstrap": "changed pinned scripts/workflow_delivery_bootstrap.py",
}


def evaluate_hook(output, *, case_id, hook):
    output = Path(output)
    result, invocation = capture_envelope(output, case_id, "H")
    require(invocation["argv"] == [str(hook)], "hook", "actual owner-hook entry required")
    if case_id in HOOK_TRUST_CASES:
        message = HOOK_TRUST_CASES[case_id]
        require(result["returncode"] == 2 and (output / "stdout.bin").read_bytes() == b""
                and (output / "stderr.bin").read_bytes() == ("WDQ-TRUST: " + message + "\n").encode(),
                "hook", "exact selected-support refusal required")
        trace = output / "calls.jsonl"
        before_bootstrap = case_id.endswith(("missing-runner-selection", "changed-bootstrap"))
        if before_bootstrap:
            require(not trace.exists(), "hook", "pre-bootstrap refusal cannot claim Python dispatch")
        else:
            trust = parse_json((output / "before.json").read_bytes(), "before.json")["local_trust"]
            script = Path(hook).parent.parent / "scripts/workflow_delivery_bootstrap.py"
            arguments = ["--root", invocation["cwd"], "--authority-ref", trust["datum.workflowDeliveryAuthorityRef"][0],
                         "--base-ref", trust["datum.workflowDeliveryBaseRef"][0], "--environment-path",
                         trust["datum.workflowDeliveryEnvironmentPath"][0], "--runner",
                         trust["datum.workflowDeliveryRunnerPath"][0]]
            events = observer_calls(output, result, invocation, script, arguments)
            require(not any(row["event"] == "call" and row["source"]["path"] ==
                            trust["datum.workflowDeliveryRunnerPath"][0] for row in events),
                    "hook", "untrusted runner was executed")
        return {"case_id": case_id, "surface": "H", "invocation_id": result["invocation_id"],
                "observation_matches_expected": True, "validator_entrypoint_observed": False,
                "refusal_boundary": "owner-hook-before-bootstrap" if before_bootstrap else "bootstrap-support-verification",
                "result_sha256": sha256((output / "result.json").read_bytes()),
                "handler_trace_complete": False, "child_tool_closure_complete": False,
                "complete_matrix": False, "acceptance_asserted": False}
    if case_id in TRUST_CASES:
        field, value = TRUST_CASES[case_id]
        message = "missing " + field if value is None else field + " must be a full commit ID"
        require(result["returncode"] == 2 and (output / "stdout.bin").read_bytes() == b""
                and (output / "stderr.bin").read_bytes() == ("WDQ-TRUST: " + message + "\n").encode(),
                "hook", "exact pre-bootstrap trust refusal required")
        require(not (output / "calls.jsonl").exists(), "hook",
                "pre-bootstrap refusal must not claim validator observations")
        return {"case_id": case_id, "surface": "H", "invocation_id": result["invocation_id"],
                "observation_matches_expected": True, "validator_entrypoint_observed": False,
                "refusal_boundary": "owner-hook-before-bootstrap", "expected_error": message,
                "result_sha256": sha256((output / "result.json").read_bytes()),
                "handler_trace_complete": False, "child_tool_closure_complete": False,
                "complete_matrix": False, "acceptance_asserted": False}
    root = invocation["cwd"]
    trust = parse_json((output / "before.json").read_bytes(), "before.json")["local_trust"]
    scripts = Path(hook).parent.parent / "scripts"
    arguments = ["--root", root, "--authority-ref", trust["datum.workflowDeliveryAuthorityRef"][0],
                 "--base-ref", trust["datum.workflowDeliveryBaseRef"][0],
                 "--environment-path", trust["datum.workflowDeliveryEnvironmentPath"][0],
                 "--runner", trust["datum.workflowDeliveryRunnerPath"][0]]
    require(invocation["environment"].get("DATUM_WDQ_OBSERVE_PATH") == str(output / "calls.jsonl"),
            "hook", "exact observation output required")
    events = observer_calls(output, result, invocation, scripts / "workflow_delivery_bootstrap.py", arguments)
    runner = scripts / "check_workflow_delivery.py"
    require(any(row["event"] == "call" and row["function"] == "main"
                and row["source"]["path"] == str(runner)
                and row["source"]["sha256"] == sha256(runner.read_bytes()) for row in events),
            "hook", "actual pinned validator entrypoint was not observed")
    target, code = CASES[case_id]
    if case_id in SNAPSHOT_CASES:
        target, code, detail = snapshot_expectation(case_id, "H")
    else:
        detail = expected_detail(case_id)
    if case_id in PROOF_INPUT_CASES:
        target, code, detail = proof_expectation(case_id, "H")
    if case_id in REVIEW_PHASE_CASES:
        target, code, detail = review_phase_expectation(case_id, "H")
    expected_status = 0 if code is None else 2 if code == "WDQ-TRUST" else 1
    require(result["returncode"] == expected_status, "hook", "unexpected hook status")
    lines = (output / "stdout.bin").read_bytes().splitlines()
    require(len(lines) == 3 and lines[0].startswith(b"visual-truth file-lane gate passed")
            and lines[1].startswith(b"rustfmt gate passed"),
            "hook", "both earlier gates must precede enforcement")
    require((output / "stderr.bin").read_bytes() == b"", "hook", "unexpected hook stderr")
    report = parse_json(lines[2], "hook report")
    require(report["mode"] == "enforce" and report["acceptance_asserted"] is False
            and report["readiness_asserted"] is False, "hook", "enforcement diagnostic required")
    findings = report["findings"]
    require([row["code"] for row in findings] == ([] if code is None else [code]),
            "hook", "precise refusal required")
    if code is not None:
        require(findings[0]["path"] == target and detail in findings[0]["detail"],
                "hook", "wrong diagnostic input or cause")
    return {"case_id": case_id, "surface": "H", "invocation_id": result["invocation_id"],
            "observation_matches_expected": True, "result_sha256": sha256((output / "result.json").read_bytes()),
            "handler_trace_complete": False, "child_tool_closure_complete": False,
            "validator_entrypoint_observed": True,
            "complete_matrix": False, "acceptance_asserted": False}


def run_hook_case(packet, destination, environment, *, case_id):
    if case_id not in CASES.keys() | HOOK_TRUST_CASES.keys() or case_id in (HISTORY_CASES | BASE_CASES) or case_id in (CLI_ERROR_CASES | REPORT_CASES):
        raise ValueError("explicit implemented hook case required")
    if case_id in SNAPSHOT_CASES:
        snapshot_expectation(case_id, "H")
    destination = Path(destination).resolve()
    destination.mkdir(exist_ok=False)
    if case_id == RECOVERY_CASE:
        run_hook_case(packet, destination / "prior-refusal", environment,
                      case_id="INFRA-S02-09.outside-scope")
    root, output = destination / "fixture", destination / "capture"
    recipe = restore_fixture(packet, root)
    prepared_input = case_uses_prepared_input(case_id, recipe)
    authority = recipe["head"]
    bundle = prepare_bundle(root, authority)
    locations = support_locations(root, authority)
    # Only this nonce-owned restored fixture receives local trust selection.
    git(root, "config", "--local", "core.hooksPath", str(locations["hooks"]))
    git(root, "config", "--local", "datum.workflowDeliveryRunnerPath", str(locations["runner"]))
    save(destination / "support-before.json", bundle)
    target, _ = CASES.get(case_id, (None, None))
    mutation = {"case_id": case_id, "path": target, "before_sha256": None, "after_hex": None,
                "prepared_fixture_variant": recipe.get("fixture_variant")}
    if case_id in HOOK_TRUST_CASES:
        if case_id.endswith("missing-runner-selection"):
            git(root, "config", "--local", "--unset-all", "datum.workflowDeliveryRunnerPath")
        else:
            damaged = (locations["scripts"] / "workflow_delivery_bootstrap.py"
                       if case_id.endswith("changed-bootstrap") else locations["runner"])
            mutation["support_path"] = str(damaged)
            mutation["before_sha256"] = sha256(damaged.read_bytes())
            if case_id.endswith("missing-runner-file"):
                damaged.unlink()
            else:
                damaged.chmod(0o644)
                damaged.write_bytes(b'print("UNTRUSTED SUPPORT EXECUTED")\n')
                damaged.chmod(0o444)
        # Intentionally invalid fixture support is an INPUT; compare its actual
        # state across the refusal instead of asking verify_bundle to bless it.
    elif case_id in TRUST_CASES:
        mutation = prepare_trust_input(root, case_id)
    elif case_id in BOUNDARY_CASES:
        mutation = mutate_external_boundary(root, case_id)
    elif case_id in SNAPSHOT_CASES:
        mutation = prepare_snapshots(root, case_id)
    elif case_id == CURRENT_HISTORY_CASE:
        mutation = prepare_current_history(root, authority, destination / "history.bundle")
    elif target and not prepared_input:
        path = root / target
        raw = path.read_bytes() if path.exists() else None
        mutation["before_sha256"] = sha256(raw) if raw is not None else None
        changed = changed_bytes(case_id, raw)
        path.write_bytes(changed)
        mutation["after_hex"] = changed.hex()
        git(root, "add", "--", target)
    save(destination / "mutation.json", mutation)
    support_paths = {row["path"]: Path(locations["hook"]).parent.parent / row["path"]
                     for row in bundle["files"]}
    support_state = {name: file_state(path) for name, path in support_paths.items()}
    save(destination / "support-input-state.json", support_state)
    observed_environment = dict(text_environment(environment) if case_id in TEXT_CASES else environment,
                                DATUM_WDQ_OBSERVE_PATH=str(output / "calls.jsonl"))
    trigger = ({"trace": "calls.jsonl", "source": str(locations["runner"]), "function": "main"}
               if case_id == INTERRUPTION_CASE else None)
    capture_command(root, output, [str(locations["hook"])], observed_environment,
                    fixture_nonce=recipe["fixture_nonce"], case_id=case_id, surface="H",
                    untracked_roots=recipe["untracked_roots"], timeout=30, interrupt_on_call=trigger)
    after_state = {name: file_state(path) for name, path in support_paths.items()}
    save(destination / "support-output-state.json", after_state)
    require(support_state == after_state, "support", "hook support changed during capture")
    if case_id not in HOOK_TRUST_CASES:
        after = verify_bundle(root, authority)
        save(destination / "support-after.json", after)
        require(bundle == after, "support", "verified hook support changed")
    assessment = (evaluate_interruption(output, case_id=case_id, surface="H", trigger=trigger)
                  if trigger is not None else evaluate_hook(output, case_id=case_id, hook=locations["hook"]))
    assessment["support_manifest_sha256"] = sha256((destination / "support-before.json").read_bytes())
    assessment["support_input_sha256"] = sha256((destination / "support-input-state.json").read_bytes())
    assessment["support_output_sha256"] = sha256((destination / "support-output-state.json").read_bytes())
    if case_id in TEXT_CASES:
        assessment["text_evidence"] = evaluate_text_output(output, surface="H")
    if case_id == RECOVERY_CASE:
        assessment["recovery"] = recovery_link(destination / "prior-refusal", output,
                                              case_id=case_id, surface="H")
    save(destination / "assessment.json", assessment)
    return assessment
