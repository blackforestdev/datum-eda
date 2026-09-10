"""Retain and validate an evidence-only candidate ref; do not publish main."""

import argparse
import hashlib
import json
import os
from pathlib import Path
import re
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
    parser = argparse.ArgumentParser(description=__doc__)
    variants = parser.add_mutually_exclusive_group()
    variants.add_argument("--index-repair", action="store_true")
    variants.add_argument("--sequencing-repair", action="store_true")
    parser.add_argument("--paired-real", action="store_true")
    parser.add_argument("--packet-sha256")
    args = parser.parse_args()
    if args.paired_real:
        if not args.sequencing_repair or not re.fullmatch(r"[0-9a-f]{64}", args.packet_sha256 or ""):
            parser.error("--paired-real requires --sequencing-repair and exact --packet-sha256")
    elif args.packet_sha256 is not None:
        parser.error("--packet-sha256 is restricted to --paired-real")
    pin = "4e11d60b6f0ec50aa391c68ed39a0df138adf8cf" if args.index_repair else PIN
    baseline = "ba318df013ca5c0263c489a9c77187dc4057a7be" if args.index_repair else pin
    ref = "refs/datum-wdq/candidates/index-repair-typed-packet-20260910" if args.index_repair else REF
    expected_packet = "02ce61ee28c450776a575330ce17aca93f2a8e4edc7e4c704cf299efd69cc8ce" if args.index_repair else "678daa96208fb7a372962c383832a099f90ccc1bd63d76c091fee9b4d3fb84f9"
    if args.sequencing_repair:
        pin = "1d48f249dc7071fc3718b345a4ab16b366af43be"
        baseline = "377323b29705ccccd6f8fa91f663d7d91c6d03c4"
        ref = "refs/datum-wdq/candidates/sequencing-typed-packet-20260910"
        expected_packet = "12afccdfb09c7aa8b60376b740eebc3385f5d5a15d5c205121341f43425eb626"
    if args.paired_real:
        expected_packet = args.packet_sha256
        ref = "refs/datum-wdq/candidates/sequencing-paired-real-" + expected_packet
    original_head = git("rev-parse", "HEAD").decode().strip()
    original_status = git("status", "--porcelain")
    runtime = ROOT / ".git/datum-wdq/proposals/wdq-final-owner-disposed-producer-20260909"
    retained = ROOT / ".git/datum-wdq/proposals/wdq-full-candidate-inspection-20260909"
    overlay = retained / "typed-packet-selected-environment-0e5b8064"
    if args.index_repair:
        runtime = ROOT / ".git/datum-wdq/proposals/index-preservation-repair-20260910"
        overlay = retained / "index-repair-typed-producer-20260910"
    if args.sequencing_repair:
        runtime = ROOT / ".git/datum-wdq/proposals/sequencing-repair-20260910"
        retained = ROOT / ".git/datum-wdq/proposals/sequencing-evidence-20260910"
        overlay = retained / "typed-packet-attempt-01"
    if args.paired_real:
        overlay = retained / "paired-real-typed-packet-attempt-01"
    assert subprocess.check_output(["git", "rev-parse", "HEAD"], cwd=runtime).decode().strip() == pin
    assert not subprocess.check_output(["git", "--no-optional-locks", "status", "--porcelain"], cwd=runtime)
    sys.path.insert(0, str(runtime / "scripts"))
    from workflow_delivery_authority import authority_sha256
    from workflow_delivery_native import validate_correlations, validate_environment
    from workflow_delivery_proof import packet_sha256, validate_proof
    from workflow_delivery_tree import Tree

    # A private index changes no worktree/index, branch, installed hook or trust.
    with tempfile.TemporaryDirectory(prefix="typed-index-", dir=retained) as temp:
        env = dict(os.environ, GIT_INDEX_FILE=str(Path(temp) / "index"))
        git("read-tree", baseline, env=env)
        if args.paired_real:
            # Preserve the newly discovered review finding without importing
            # unrelated live roadmap/tracker edits into the frozen baseline.
            issue_id = "dat-wdq-test-default-branch-apx"
            path = ".beads/issues.jsonl"
            current = git("show", original_head + ":" + path).splitlines()
            added = [row for row in current if json.loads(row)["id"] == issue_id]
            prior_raw = git("show", baseline + ":" + path)
            prior = prior_raw.splitlines()
            assert prior_raw == b"\n".join(prior) + b"\n"
            assert len(added) == 1 and json.loads(added[0])["status"] == "open"
            assert all(json.loads(row)["id"] != issue_id for row in prior)
            oid = git("hash-object", "-w", "--stdin", data=prior_raw + added[0] + b"\n").decode().strip()
            git("update-index", "--add", "--cacheinfo", "100644", oid, path, env=env)
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
            typed_prefix = "index-repair-typed/" if args.index_repair else "typed/"
            if args.sequencing_repair:
                typed_prefix = "sequencing-typed/"
            if args.paired_real:
                typed_prefix = "paired-real-typed/"
            assert relative == "docs/reviews/workflow-delivery-rollout/infrastructure/proof.json" or relative.startswith(PREFIX + typed_prefix)
            oid = git("hash-object", "-w", "--stdin", data=path.read_bytes()).decode().strip()
            git("update-index", "--add", "--cacheinfo", "100644", oid, relative, env=env)
        tree_id = git("write-tree", env=env).decode().strip()
    message = f"""test(workflow): assemble committed compatibility evidence candidate

Problem: Prospective overlay validation did not establish committed-tree proof.
Change: Bind renewed main-committed evidence and generated typed proof to the
unchanged {pin} runtime through an isolated candidate ref, not main publication.
Prospective baseline: {baseline}; evidence source: {original_head}.
Paired-real mode also carries the exact open dat-wdq-test-default-branch-apx
intake row from the evidence source, preserving every baseline tracker row.
Proof: Runtime inputs and authority must remain identical; the producer performs
proof, selected-environment and correlation validation against this exact commit.
Roadmap: {'WDQ-RECHECK' if args.paired_real else 'WDQ-COMPAT'}, dat-wdq-rollout-implementation-ffy and
dat-wdq-workspace-inputs-zmt advance without closure. Index-repair mode also
retains dat-wdq-index-refresh-ogw and dat-wdq-proof-renewal-cycle-qgb as open
proof findings. Historical typed artifacts remain unchanged. Independent WDQ-RECHECK,
owner activation and product cohorts remain required. No dependency/license,
product-lane, installed-hook or local-trust changes.
"""
    candidate = git("commit-tree", tree_id, "-p", baseline, data=message.encode()).decode().strip()
    git("update-ref", ref, candidate, "0" * 40)
    base, tree = Tree(ROOT, revision=pin), Tree(ROOT, revision=candidate)
    if args.paired_real:
        assert tree.read(".beads/issues.jsonl") == prior_raw + added[0] + b"\n"
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
    assert packet == expected_packet
    assert git("rev-parse", "HEAD").decode().strip() == original_head
    assert git("status", "--porcelain") == original_status
    print(json.dumps({"schema_version": 1, "candidate": candidate, "candidate_ref": ref,
        "tree": tree_id, "evidence_source_commit": original_head, "runtime_source_commit": pin,
        "prospective_baseline": baseline,
        "additional_review_intake": ["dat-wdq-test-default-branch-apx"] if args.paired_real else [],
        "packet_sha256": packet, "authority_sha256": authority,
        "input_manifest_unchanged": True, "correlated_event_blobs": len(events),
        "committed_tree_proof_validation": "pass", "selected_environment_validation": "pass",
        "correlation_validation": "pass", "main_head_and_worktree_unchanged": True,
        "full_entrypoint_verification": False, "independent_replay_complete": False,
        "publication_verified": False, "activation_performed": False}))


if __name__ == "__main__":
    main()
