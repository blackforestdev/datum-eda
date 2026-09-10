"""Prepare exact local support and a read-only preflight request, not activation."""

import json
from pathlib import Path
import subprocess
import sys

ROOT = Path("/home/bfadmin/Documents/datum-eda")
STORE = ROOT / ".git/datum-wdq/proposals/terminal-integration-1644"
PIN = "1644e2129f4ff83cec1cdf0e6e13534c3b258e8e"
BASE = "d45d1a3e30d6af4ab752dd19a7dbe45157df50d4"
RUNTIME = ROOT / ".git/datum-wdq/proposals/terminal-owner-20260910"


def main():
    assert sys.flags.isolated and sys.flags.no_site and sys.flags.dont_write_bytecode
    def git(*args):
        return subprocess.check_output(["git", "--no-replace-objects", "--no-optional-locks", *args], cwd=ROOT)
    assert git("rev-parse", "HEAD").decode().strip() == BASE and not git("status", "--porcelain")
    bootstrap = RUNTIME / "scripts/workflow_delivery_source_only.py"
    raw = bootstrap.read_bytes()
    assert raw == git("show", PIN + ":scripts/workflow_delivery_source_only.py")
    namespace = {"__name__": "_terminal_preflight_bootstrap"}
    exec(compile(raw, str(bootstrap), "exec"), namespace)
    namespace["install"](RUNTIME / "scripts")
    sys.path.insert(0, str(RUNTIME / "scripts"))
    from workflow_delivery_capture_command import save
    from workflow_delivery_capture_state import protected_state
    from workflow_delivery_support_bundle import prepare_bundle, verify_bundle
    from workflow_delivery_support_store import proposed_local_trust, support_locations
    from workflow_delivery_publication_delta import publication_delta
    from workflow_delivery_io import canonical_json, sha256
    inspection = json.loads((STORE / "candidate-inspection.json").read_bytes())
    assert inspection["candidate"] == PIN and inspection["base"] == BASE
    delta = publication_delta(ROOT, base=BASE, candidate=PIN)
    assert sha256(canonical_json(delta)) == inspection["publication_review"]["delta_sha256"]
    before = protected_state(ROOT, [])
    bundle = prepare_bundle(ROOT, PIN)
    assert verify_bundle(ROOT, PIN) == bundle
    assert protected_state(ROOT, []) == before
    environment = "specs/workflow_delivery/rollout.environments.json"
    policy = json.loads(git("show", PIN + ":specs/workflow_delivery_policy.json"))
    request = {"schema_version": 1, "kind": "datum.workflow-delivery.preflight-request",
        "base": BASE, "candidate": PIN, "authority": PIN,
        "input_roots": policy["coverage"]["production_roots"] + [".beads"],
        "prior_local_trust": before["local_trust"],
        "proposed_local_trust": proposed_local_trust(ROOT, authority=PIN, base=BASE, environment_path=environment),
        "publication_review": inspection["publication_review"], "environment_path": environment}
    save(STORE / "preflight-request.json", request)
    save(STORE / "support-preparation.json", {"candidate": PIN, "bundle": bundle,
        "request_sha256": sha256((STORE / "preflight-request.json").read_bytes()),
        "protected_state_unchanged": True, "activation_performed": False})
    locations = support_locations(ROOT, PIN)
    print(json.dumps({"request_sha256": sha256((STORE / "preflight-request.json").read_bytes()),
        "preflight_cli": str(locations["scripts"] / "workflow_delivery_preflight_cli.py"),
        "activation_performed": False}))


if __name__ == "__main__":
    main()
