"""Prepare an exact activation request; do not publish, install or assert approval."""

import hashlib
import json
from pathlib import Path
import subprocess
import sys


def main():
    assert sys.flags.isolated and sys.flags.no_site and sys.flags.dont_write_bytecode
    root = Path("/home/bfadmin/Documents/datum-eda")
    store = Path(__file__).resolve().parent
    raw = (store / "preflight-request.json").read_bytes()
    assert hashlib.sha256(raw).hexdigest() == "daeac8a832a595503b7f99650d020cbe782b48a4b2e9a94a7d6db9321ac6b98b"
    request = json.loads(raw)
    pin = "1644e2129f4ff83cec1cdf0e6e13534c3b258e8e"
    assert request["candidate"] == request["authority"] == pin
    git = lambda *args: subprocess.check_output(["git", "--no-replace-objects", "--no-optional-locks", *args], cwd=root)
    assert git("rev-parse", "HEAD").decode().strip() == request["base"]
    assert not git("status", "--porcelain")
    reviewer = root / ".git/datum-wdq/proposals/independent-terminal-rollout-20260910"
    proof_raw = (reviewer / "integration-24-complete.json").read_bytes()
    assert hashlib.sha256(proof_raw).hexdigest() == "33e4b7567a2d0316892243dbadb03bb8ce0bf628ad61a8fb707c4b951b3e8dbf"
    proof = json.loads(proof_raw)
    assert proof["integration_candidate"] == pin and proof["counts"]["observations"] == 24
    assert proof["full_restored_prestate_and_within_run_state_equal"] and not proof["failed_replay_attempts"]
    final_raw = (reviewer / "final-candidate-inspection-02/result.json").read_bytes()
    assert hashlib.sha256(final_raw).hexdigest() == "8f8c42993f82612df39dba59fb2e3a17b7fe909ab2c10f3483a34a691aaf26ff"
    final = json.loads(final_raw)
    assert final["candidate"] == pin and final["protected_state_unchanged"]
    assert final["independent_review_and_scoped_disposition_validation"] == "pass"
    runtime = root / ".git/datum-wdq/trusted" / pin / "scripts"
    bootstrap = runtime / "workflow_delivery_source_only.py"
    raw = bootstrap.read_bytes()
    assert raw == git("show", pin + ":scripts/workflow_delivery_source_only.py")
    namespace = {"__name__": "_activation_request_bootstrap"}
    exec(compile(raw, str(bootstrap), "exec"), namespace)
    namespace["install"](runtime)
    sys.path.insert(0, str(runtime))
    from workflow_delivery_activation import COORDINATION
    from workflow_delivery_support_bundle import verify_bundle
    from workflow_delivery_capture_command import save
    verify_bundle(root, pin)
    request["kind"] = "datum.workflow-delivery.activation-request"
    request["coordination"] = COORDINATION
    path = store / "activation-request.json"
    save(path, request)
    digest = hashlib.sha256(path.read_bytes()).hexdigest()
    save(store / "activation-request-basis.json", {
        "candidate": pin, "base": request["base"], "request_sha256": digest,
        "independent_integration_sha256": hashlib.sha256(proof_raw).hexdigest(),
        "independent_final_inspection_sha256": hashlib.sha256(final_raw).hexdigest(),
        "coordination_clause": "Required condition for execution; this preparation does not assert a live writer pause.",
        "owner_response_created": False, "activation_performed": False,
        "scope": "Exact request preparation only. Standing authorization must be applied honestly, not represented as a newly typed owner response."})
    print(json.dumps({"candidate": pin, "request_sha256": digest, "activation_performed": False}))


if __name__ == "__main__":
    main()
