"""Prepare the exact reviewed PM042 activation request; never install it."""

import hashlib
import json
from pathlib import Path
import subprocess
import sys


ROOT = Path("/home/bfadmin/Documents/datum-eda")
STORE = Path(__file__).resolve().parent
BASE = "354ef94969b878fe8ad978833880755e750ff7e8"
CANDIDATE = "0f232e1026497d4eb404a503ee7c7e3fb7bb6561"
REVIEW_SHA256 = "a3e88fc6ee5cf321d8969f44738e4e8edfcb41bb08dc03280799b439bba49124"
ENVIRONMENT = "specs/workflow_delivery/rollout.environments.json"
INPUT_ROOTS = [
    ".cargo", ".github", "Cargo.toml", "Cargo.lock", "crates", "mcp-server",
    "scripts", "docs/gui/icon_set.json", "docs/gui/menu_model.csv",
    "docs/gui/menu_model.json", ".beads",
]


def main():
    assert sys.flags.isolated and sys.flags.no_site and sys.flags.dont_write_bytecode
    git = lambda *args: subprocess.check_output(
        ["git", "--no-replace-objects", "--no-optional-locks", *args], cwd=ROOT)
    assert git("rev-parse", "HEAD").decode().strip() == BASE
    assert not git("status", "--porcelain=v1", "--untracked-files=all")

    runtime = ROOT / ".git/datum-wdq/trusted" / CANDIDATE / "scripts"
    bootstrap = runtime / "workflow_delivery_source_only.py"
    raw = bootstrap.read_bytes()
    assert raw == git("show", CANDIDATE + ":scripts/workflow_delivery_source_only.py")
    namespace = {"__name__": "_datum_activation_request_bootstrap"}
    exec(compile(raw, str(bootstrap), "exec"), namespace)
    namespace["install"](runtime)
    sys.path.insert(0, str(runtime))

    from workflow_delivery_activation import COORDINATION
    from workflow_delivery_activation_preflight import (
        inspect_promotion_candidate, validate_preparation_upgrade_review,
    )
    from workflow_delivery_capture_state import protected_state
    from workflow_delivery_io import canonical_json, sha256
    from workflow_delivery_publication_delta import publication_delta
    from workflow_delivery_support_bundle import verify_bundle
    from workflow_delivery_support_store import proposed_local_trust
    from workflow_delivery_tree import Tree

    verify_bundle(ROOT, CANDIDATE)
    review_path = (
        "docs/reviews/workflow-delivery-rollout/infrastructure/"
        "preparation-bootstrap/independent-review.json"
    )
    review_raw = git("show", CANDIDATE + ":" + review_path)
    assert hashlib.sha256(review_raw).hexdigest() == REVIEW_SHA256
    retained = validate_preparation_upgrade_review(Tree(ROOT, revision=CANDIDATE), base=BASE)
    assert retained["disposition"] == "approve" and retained["producer_commit"] == (
        "0863f13baa4d6d49a6463be6f2a0646a02d2f6d7"
    )
    delta = publication_delta(ROOT, base=BASE, candidate=CANDIDATE)
    publication_review = {
        "delta_sha256": sha256(canonical_json(delta)),
        "paths": delta["touched_paths"],
    }
    inspection = inspect_promotion_candidate(
        ROOT, base=BASE, candidate=CANDIDATE, authority=CANDIDATE,
        environment_path=ENVIRONMENT, publication_review=publication_review,
    )
    assert not inspection["activation_asserted"] and inspection["checks"] == [
        "WORKFLOW-DELIVERY-GATE-PILOT: accept",
        "WORKFLOW-DELIVERY-IMPLEMENTATION: structure",
    ]
    prior = protected_state(ROOT, INPUT_ROOTS)["local_trust"]
    proposed = proposed_local_trust(
        ROOT, authority=CANDIDATE, base=BASE, environment_path=ENVIRONMENT)
    request = {
        "schema_version": 1,
        "kind": "datum.workflow-delivery.activation-request",
        "base": BASE,
        "candidate": CANDIDATE,
        "authority": CANDIDATE,
        "input_roots": INPUT_ROOTS,
        "prior_local_trust": prior,
        "publication_review": publication_review,
        "proposed_local_trust": proposed,
        "environment_path": ENVIRONMENT,
        "coordination": COORDINATION,
    }
    request_raw = canonical_json(request)
    request_path = STORE / "activation-request.json"
    request_path.write_bytes(request_raw)
    request_sha256 = sha256(request_raw)
    packet = {
        "schema_version": 1,
        "kind": "datum.workflow-delivery.preparation-bootstrap-activation-packet",
        "base": BASE,
        "candidate": CANDIDATE,
        "producer": retained["producer_commit"],
        "independent_review_sha256": REVIEW_SHA256,
        "publication_delta_sha256": publication_review["delta_sha256"],
        "request_sha256": request_sha256,
        "installation_performed": False,
        "owner_response_created": False,
    }
    (STORE / "packet.json").write_bytes(canonical_json(packet))
    print(json.dumps(packet, sort_keys=True))


if __name__ == "__main__":
    main()
