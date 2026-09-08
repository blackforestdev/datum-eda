"""Initial real-process case recipes; explicitly not the complete INFRA matrix."""

from pathlib import Path
import sys

from workflow_delivery_capture_assertions import evaluate_capture
from workflow_delivery_capture_command import capture_command, save
from workflow_delivery_capture_fixture import restore_fixture
from workflow_delivery_capture_state import git
from workflow_delivery_io import canonical_json, parse_json, sha256

CASES = {
    "INFRA-S01-01": (None, None),
    "INFRA-S02-09.outside-scope": ("src/unscoped.py", "WDQ-COVERAGE"),
    "INFRA-S06-11.selection": ("requested-environment.json", "WDQ-ENVIRONMENT"),
    "INFRA-S06-11.blob": ("selected-environment.json", "WDQ-ENVIRONMENT"),
}


def run_case(packet, destination, runtime_root, environment, *, case_id, surface):
    if case_id not in CASES or surface not in ("C", "R", "S-check", "S-details"):
        raise ValueError("only explicitly implemented case/surface combinations may run")
    destination, runtime_root = Path(destination).resolve(), Path(runtime_root).resolve(strict=True)
    destination.mkdir(exist_ok=False)
    root, output = destination / "fixture", destination / "capture"
    recipe = restore_fixture(packet, root)
    target, expected = CASES[case_id]
    mutation = {"case_id": case_id, "path": target, "before_sha256": None, "after_hex": None}
    if target:
        path = root / target
        raw = path.read_bytes() if path.exists() else None
        mutation["before_sha256"] = sha256(raw) if raw is not None else None
        if target == "src/unscoped.py":
            changed = b"# Synthetic unpermitted implementation input.\n"
        else:
            value = parse_json(raw, target)
            if target == "requested-environment.json":
                value["environments"] = []
            else:
                value["toolchain"] = "synthetic candidate-only toolchain"
            changed = canonical_json(value)
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
        arguments += ["--enforce", "--authority-ref", recipe["head"], "--base-ref", recipe["head"],
                      "--environment-path", "requested-environment.json"]
        arguments += ["--candidate-ref", candidate] if surface == "R" else ["--staged"]
    command = [sys.executable, "-I", "-S", "-B",
        str(runtime_root / "scripts/workflow_delivery_observe_python.py"),
        "--script", str(script), "--source-root", str(runtime_root),
        "--output", str(output / "calls.jsonl"), "--function", "main",
        "--function", "selector_failures", "--", *arguments]
    capture_command(root, output, command, environment, fixture_nonce=recipe["fixture_nonce"],
                    case_id=case_id, surface="S" if selector else surface,
                    untracked_roots=recipe["untracked_roots"], timeout=30)
    try:
        assessment = evaluate_capture(output, case_id=case_id, surface="S" if selector else surface,
                                      expected_code=expected, script=script)
    except Exception as error:
        save(destination / "assessment-error.json", {"type": type(error).__name__, "detail": str(error),
                                                   "acceptance_asserted": False})
        raise
    assessment.update(selector_operation=surface if selector else None, complete_matrix=False)
    save(destination / "assessment.json", assessment)
    return assessment
