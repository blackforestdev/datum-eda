"""Retain and validate an evidence-only candidate ref; do not publish main."""

import hashlib
import json
import os
from pathlib import Path
import subprocess
import sys
import tempfile


ROOT = Path(__file__).resolve().parents[5]
PIN = "0e5b8064a9faca407fac43bd2b362ba4d7200327"
PREFIX = "docs/reviews/workflow-delivery-rollout/infrastructure/workspace-renewal/"
REF = "refs/datum-wdq/candidates/compat-typed-packet-20260909"


def git(*args, data=None, env=None):
    return subprocess.check_output(["git", *args], cwd=ROOT, input=data, env=env)


def main():
    original_head = git("rev-parse", "HEAD").decode().strip()
    original_status = git("status", "--porcelain")
    runtime = ROOT / ".git/datum-wdq/proposals/wdq-final-owner-disposed-producer-20260909"
    retained = ROOT / ".git/datum-wdq/proposals/wdq-full-candidate-inspection-20260909"
    overlay = retained / "typed-packet-selected-environment-0e5b8064"
    assert subprocess.check_output(["git", "rev-parse", "HEAD"], cwd=runtime).decode().strip() == PIN
    sys.path.insert(0, str(runtime / "scripts"))
    from workflow_delivery_authority import authority_sha256
    from workflow_delivery_native import validate_correlations, validate_environment
    from workflow_delivery_proof import packet_sha256, validate_proof
    from workflow_delivery_tree import Tree

    # A private index changes no worktree/index, branch, installed hook or trust.
    with tempfile.TemporaryDirectory(prefix="typed-index-", dir=retained) as temp:
        env = dict(os.environ, GIT_INDEX_FILE=str(Path(temp) / "index"))
        git("read-tree", PIN, env=env)
        entries = git("ls-tree", "-rz", original_head, "--", PREFIX).split(b"\0")
        for entry in filter(None, entries):
            meta, path = entry.split(b"\t", 1)
            mode, kind, oid = meta.split()
            assert kind == b"blob"
            git("update-index", "--add", "--cacheinfo",
                mode.decode(), oid.decode(), path.decode(), env=env)
        for path in sorted(overlay.rglob("*")):
            if not path.is_file():
                continue
            relative = path.relative_to(overlay).as_posix()
            assert relative == "docs/reviews/workflow-delivery-rollout/infrastructure/proof.json" or relative.startswith(PREFIX + "typed/")
            oid = git("hash-object", "-w", "--stdin", data=path.read_bytes()).decode().strip()
            git("update-index", "--add", "--cacheinfo", "100644", oid, relative, env=env)
        tree_id = git("write-tree", env=env).decode().strip()
    message = """test(workflow): assemble committed compatibility evidence candidate

Problem: Prospective overlay validation did not establish committed-tree proof.
Change: Bind renewed main-committed evidence and generated typed proof to the
unchanged 0e5b8064 runtime through an isolated candidate ref, not main publication.
Proof: Runtime inputs and authority must remain identical; the producer performs
proof, selected-environment and correlation validation against this exact commit.
Roadmap: WDQ-COMPAT, dat-wdq-rollout-implementation-ffy and
dat-wdq-workspace-inputs-zmt advance without closure. Independent WDQ-RECHECK,
owner activation and product cohorts remain required. No dependency/license,
product-lane, installed-hook or local-trust changes.
"""
    candidate = git("commit-tree", tree_id, "-p", PIN, data=message.encode()).decode().strip()
    git("update-ref", REF, candidate, "0" * 40)
    base, tree = Tree(ROOT, revision=PIN), Tree(ROOT, revision=candidate)
    contract = tree.json("specs/workflow_delivery/rollout.contract.json")
    assert tree.manifest(contract["input_roots"]) == base.manifest(contract["input_roots"])
    authority = authority_sha256(tree, contract)
    assert authority == authority_sha256(base, contract)
    proof = tree.json(contract["proof_path"])
    selection = tree.json("specs/workflow_delivery/rollout.environments.json")
    selected = [row["environment"] for row in selection["environments"]
                if row["frontier_key"] == "WORKFLOW-DELIVERY-IMPLEMENTATION"]
    assert selected == [proof["environment"]]
    environment = tree.json(selected[0]["path"])
    assert hashlib.sha256(tree.read(selected[0]["path"])).hexdigest() == selected[0]["sha256"]
    validate_proof(tree, contract)
    validate_environment(tree, proof, environment, contract=contract)
    events = validate_correlations(tree, contract, proof)
    packet = packet_sha256(contract, proof, authority)
    assert packet == "678daa96208fb7a372962c383832a099f90ccc1bd63d76c091fee9b4d3fb84f9"
    assert git("rev-parse", "HEAD").decode().strip() == original_head
    assert git("status", "--porcelain") == original_status
    print(json.dumps({"schema_version": 1, "candidate": candidate, "candidate_ref": REF,
        "tree": tree_id, "evidence_source_commit": original_head, "runtime_source_commit": PIN,
        "packet_sha256": packet, "authority_sha256": authority,
        "input_manifest_unchanged": True, "correlated_event_blobs": len(events),
        "committed_tree_proof_validation": "pass", "selected_environment_validation": "pass",
        "correlation_validation": "pass", "main_head_and_worktree_unchanged": True,
        "full_entrypoint_verification": False, "independent_replay_complete": False,
        "publication_verified": False, "activation_performed": False}))


if __name__ == "__main__":
    main()
