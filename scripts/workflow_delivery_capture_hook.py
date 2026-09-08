"""Retain real owner-hook process observations on restored synthetic fixtures.

This exercises the hook's pinned observer path. Complete contract-handler
accounting and child/tool closure still remain separate I03 obligations.
"""

from pathlib import Path

from workflow_delivery_capture_assertions import capture_envelope, observer_calls
from workflow_delivery_capture_command import capture_command, save
from workflow_delivery_capture_fixture import restore_fixture
from workflow_delivery_capture_mutations import CASES, changed_bytes, expected_detail
from workflow_delivery_capture_history import HISTORY_CASES
from workflow_delivery_capture_snapshots import SNAPSHOT_CASES, prepare_snapshots, snapshot_expectation
from workflow_delivery_capture_state import git
from workflow_delivery_io import parse_json, sha256
from workflow_delivery_shapes import require
from workflow_delivery_environment_cases import case_uses_prepared_input
from workflow_delivery_support_bundle import prepare_bundle, verify_bundle
from workflow_delivery_support_store import support_locations
from workflow_delivery_interruption_case import INTERRUPTION_CASE, evaluate_interruption


def evaluate_hook(output, *, case_id, hook):
    output = Path(output)
    result, invocation = capture_envelope(output, case_id, "H")
    require(invocation["argv"] == [str(hook)], "hook", "actual owner-hook entry required")
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
    if case_id not in CASES or case_id in HISTORY_CASES:
        raise ValueError("explicit implemented hook case required")
    if case_id in SNAPSHOT_CASES:
        snapshot_expectation(case_id, "H")
    destination = Path(destination).resolve()
    destination.mkdir(exist_ok=False)
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
    target, _ = CASES[case_id]
    mutation = {"case_id": case_id, "path": target, "before_sha256": None, "after_hex": None,
                "prepared_fixture_variant": recipe.get("fixture_variant")}
    if case_id in SNAPSHOT_CASES:
        mutation = prepare_snapshots(root, case_id)
    elif target and not prepared_input:
        path = root / target
        raw = path.read_bytes() if path.exists() else None
        mutation["before_sha256"] = sha256(raw) if raw is not None else None
        changed = changed_bytes(case_id, raw)
        path.write_bytes(changed)
        mutation["after_hex"] = changed.hex()
        git(root, "add", "--", target)
    save(destination / "mutation.json", mutation)
    observed_environment = dict(environment, DATUM_WDQ_OBSERVE_PATH=str(output / "calls.jsonl"))
    trigger = ({"trace": "calls.jsonl", "source": str(locations["runner"]), "function": "main"}
               if case_id == INTERRUPTION_CASE else None)
    capture_command(root, output, [str(locations["hook"])], observed_environment,
                    fixture_nonce=recipe["fixture_nonce"], case_id=case_id, surface="H",
                    untracked_roots=recipe["untracked_roots"], timeout=30, interrupt_on_call=trigger)
    after = verify_bundle(root, authority)
    save(destination / "support-after.json", after)
    require(bundle == after, "support", "hook support changed during capture")
    assessment = (evaluate_interruption(output, case_id=case_id, surface="H", trigger=trigger)
                  if trigger is not None else evaluate_hook(output, case_id=case_id, hook=locations["hook"]))
    assessment["support_manifest_sha256"] = sha256((destination / "support-before.json").read_bytes())
    save(destination / "assessment.json", assessment)
    return assessment
