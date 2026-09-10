"""Prepare a bounded descendant of real main; never publish or install trust."""

from copy import deepcopy
import hashlib
import json
import os
from pathlib import Path
import subprocess
import sys
import tempfile

sys.path.insert(0, str(Path(__file__).resolve().parent))
from freeze_sequencing_source import PIN


EVIDENCE = "docs/reviews/workflow-delivery-rollout/infrastructure/"
PRODUCER = "06bafeb9979dce91ab710bfaaff7f8f9ca3e72f2"
PACKET = "fa0b802e11d5dc266f7f3e78bde8a1ac91ac6c1ae4165df8858b56b94c158fee"


def reviewed_payloads(git, overlay, review_digest, inventory_digest):
    """Import only the pinned producer and separately hashed review evidence."""
    from workflow_delivery_io import canonical_json, sha256
    overlay = Path(overlay)
    assert overlay.is_absolute() and overlay.is_dir() and not overlay.is_symlink()
    payloads = {}
    producer_prefix = EVIDENCE + "workspace-renewal/paired-real-typed/"
    entries = git("ls-tree", "-rz", PRODUCER, "--", EVIDENCE + "proof.json", producer_prefix)
    for entry in filter(None, entries.split(b"\0")):
        meta, raw_path = entry.split(b"\t", 1)
        mode, kind, oid = meta.split()
        path = raw_path.decode()
        assert mode == b"100644" and kind == b"blob"
        assert path == EVIDENCE + "proof.json" or path.startswith(producer_prefix)
        payloads[path] = git("cat-file", "blob", oid.decode())
    assert EVIDENCE + "proof.json" in payloads
    review_prefix = EVIDENCE + "workspace-renewal/independent-sequencing-typed/"
    review_files = {}
    for path in sorted(overlay.rglob("*")):
        assert not path.is_symlink(), str(path)
        if path.is_dir():
            continue
        assert path.is_file(), str(path)
        relative = path.relative_to(overlay).as_posix()
        assert relative == EVIDENCE + "review.json" or relative.startswith(review_prefix), relative
        review_files[relative] = path.read_bytes()
    assert sha256(review_files[EVIDENCE + "review.json"]) == review_digest
    inventory = [{"path": path, "sha256": sha256(raw), "size": len(raw)}
                 for path, raw in sorted(review_files.items())]
    assert sha256(canonical_json(inventory)) == inventory_digest
    assert not payloads.keys() & review_files.keys()
    payloads.update(review_files)
    return payloads, inventory


def main(*, expected_step="WDQ-COMPAT", review_overlay=None,
         review_digest=None, review_inventory_digest=None):
    if not (sys.flags.isolated and sys.flags.no_site and sys.flags.dont_write_bytecode):
        raise ValueError("candidate preparation requires Python -I -S -B")
    if expected_step not in ("WDQ-COMPAT", "WDQ-RECHECK"):
        raise ValueError("candidate preparation requires an authorized review execution step")
    evidence_args = (review_overlay, review_digest, review_inventory_digest)
    if any(value is not None for value in evidence_args):
        if expected_step != "WDQ-RECHECK" or not all(evidence_args):
            raise ValueError("review evidence requires RECHECK and all three exact overlay pins")
    root = Path(__file__).resolve().parents[5]
    runtime = root / ".git/datum-wdq/proposals/sequencing-repair-20260910"
    store = root / ".git/datum-wdq/proposals/sequencing-evidence-20260910"
    def git(*args, data=None, env=None):
        return subprocess.check_output(["git", "--no-replace-objects", "--no-optional-locks", *args],
                                       cwd=root, input=data, env=env)
    base = git("rev-parse", "HEAD").decode().strip()
    assert git("symbolic-ref", "HEAD").strip() == b"refs/heads/main"
    assert not git("status", "--porcelain")
    bootstrap = runtime / "scripts/workflow_delivery_source_only.py"
    raw = bootstrap.read_bytes()
    assert raw == git("show", PIN + ":scripts/workflow_delivery_source_only.py")
    namespace = {"__name__": "_sequencing_source_only"}
    exec(compile(raw, str(bootstrap), "exec"), namespace)
    namespace["install"](runtime / "scripts")
    sys.path.insert(0, str(runtime / "scripts"))
    from project_status import render_block, replace_block
    from workflow_delivery_tree import Tree
    from workflow_delivery_io import canonical_json, sha256
    from workflow_delivery_authority import authority_sha256
    from workflow_delivery_frontier import validate_frontier
    from workflow_delivery_review_inspection import inspect_review_candidate
    from workflow_delivery_publication_delta import publication_delta
    source, live = Tree(root, revision=PIN), Tree(root, revision=base)
    contract = source.json("specs/workflow_delivery/rollout.contract.json")
    policy = source.json("specs/workflow_delivery_policy.json")
    scoped = set(policy["coverage"]["source_scopes"][0]["paths"])
    # Read dependencies never become write permission. Current product bytes
    # must already match; otherwise this preparation stops without adapting them.
    explicit = {
        "specs/workflow_delivery/rollout.contract.json", "specs/workflow_delivery_policy.json",
        "specs/workflow_delivery/rollout.environments.json", "specs/workflow_delivery/workspace-inputs.json",
        "docs/reviews/workflow-delivery-rollout/infrastructure/prepared-hook-pipes-environment.json",
        "specs/WORKFLOW_DELIVERY_INFRASTRUCTURE_CONTRACT.md", "specs/WORKFLOW_DELIVERY_EXECUTION_PLAN.md",
        "specs/WORKFLOW_DELIVERY_COVERAGE_PROPOSAL.md",
        "docs/reviews/workflow-delivery-rollout/coverage-proposal.json",
        "docs/decisions/PRODUCT_MECHANICS_042_BROAD_WORKFLOW_DELIVERY_ENFORCEMENT.md",
        "research/process-quality/WORKFLOW_INFRASTRUCTURE_CONTRACT_BASIS.md",
    }
    for path in set(contract["input_roots"]) - scoped - explicit:
        assert live.read(path) == source.read(path), path
    payloads = {path: source.read(path) for path in scoped | explicit}
    review_inventory = None
    if review_overlay is not None:
        evidence_payloads, review_inventory = reviewed_payloads(
            git, review_overlay, review_digest, review_inventory_digest)
        assert not payloads.keys() & evidence_payloads.keys()
        payloads.update(evidence_payloads)
    frontier = live.json("specs/active_frontier.json")
    key = "WORKFLOW-DELIVERY-IMPLEMENTATION"
    item = next(i for i in frontier["frontier"] if i["key"] == key)
    assert item["completion"]["canonical_next_step_id"] == expected_step
    assert "delivery" not in item["completion"]
    source_item = next(i for i in source.json("specs/active_frontier.json")["frontier"] if i["key"] == key)
    item["completion"]["delivery"] = deepcopy(source_item["completion"]["delivery"])
    payloads["specs/active_frontier.json"] = canonical_json(frontier)
    payloads["specs/PROGRESS.md"] = replace_block(live.read("specs/PROGRESS.md").decode(), render_block(frontier)).encode()
    trace_path = "specs/evidence_traceability_manifest.json"
    trace = live.json(trace_path)
    routes = {r["id"]: r for r in source.json(trace_path)["routes"]}
    trace["routes"] = [routes[r["id"]] if r["id"] in
        ("workflow-delivery-infrastructure", "workflow-delivery-rollout") else r for r in trace["routes"]]
    payloads[trace_path] = canonical_json(trace)
    governance_path = "specs/spec_governance_manifest.json"
    governance = live.json(governance_path)
    for path, entry in source.json(governance_path)["entries"].items():
        if path in payloads:
            governance["entries"][path] = entry
    payloads[governance_path] = canonical_json(governance)
    with tempfile.TemporaryDirectory(prefix="sequencing-index-", dir=store) as directory:
        env = dict(os.environ, GIT_INDEX_FILE=str(Path(directory) / "index"))
        git("read-tree", base, env=env)
        for path, raw in sorted(payloads.items()):
            oid = git("hash-object", "-w", "--stdin", data=raw).decode().strip()
            mode = ("100644" if path in (evidence_payloads if review_overlay is not None else {})
                    else git("ls-tree", PIN, "--", path).decode().split()[0])
            git("update-index", "--add", "--cacheinfo", mode, oid, path, env=env)
        tree_id = git("write-tree", env=env).decode().strip()
    message = ("test(workflow): prepare exact live-base review candidate\n\n"
        "Problem: Renewed inspection requires a descendant of actual main, not invented lifecycle state.\n"
        "Change: Overlay reviewed workflow source and contracts only; preserve main claims, tracker and other lanes.\n"
        "Proof: Exact input/authority equality, Frontier validation and read-only review inspection follow.\n"
        f"Roadmap: {expected_step}, dat-wdq-review-publication-cycle-a1y; preparation only, no activation, dependency or licensing change.\n")
    candidate = git("commit-tree", tree_id, "-p", base, data=message.encode()).decode().strip()
    ref = "refs/datum-wdq/candidates/sequencing-live-review-" + candidate
    git("update-ref", ref, candidate, "0" * 40)
    prepared = Tree(root, revision=candidate)
    assert source.manifest(contract["input_roots"]) == prepared.manifest(contract["input_roots"])
    assert authority_sha256(source, contract) == authority_sha256(prepared, contract)
    assert prepared.read(".beads/issues.jsonl") == live.read(".beads/issues.jsonl")
    if review_overlay is not None:
        from workflow_delivery_proof import packet_sha256, validate_proof
        from workflow_delivery_native import validate_correlations, validate_environment
        proof = validate_proof(prepared, contract)
        assert packet_sha256(contract, proof, authority_sha256(prepared, contract)) == PACKET
        assert hashlib.sha256(prepared.read(contract["review_path"])).hexdigest() == review_digest
        for path, raw in evidence_payloads.items():
            assert prepared.read(path) == raw, path
        environment = prepared.json(proof["environment"]["path"])
        validate_environment(prepared, proof, environment, contract=contract)
        validate_correlations(prepared, contract, proof)
    validate_frontier(prepared)
    delta = publication_delta(root, base=base, candidate=candidate)
    review = {"delta_sha256": sha256(canonical_json(delta)), "paths": delta["touched_paths"]}
    result = inspect_review_candidate(root, base=base, candidate=candidate, authority=candidate,
        environment_path="specs/workflow_delivery/rollout.environments.json", publication_review=review)
    if review_overlay is not None:
        from workflow_delivery_review import validate_independent_review
        from workflow_delivery_trust import Trust
        validate_independent_review(prepared, contract, proof,
            authority_sha256(prepared, contract), Trust(prepared, candidate, base), item, environment)
    assert git("rev-parse", "HEAD").decode().strip() == base and not git("status", "--porcelain")
    print(json.dumps({"base": base, "candidate": candidate, "ref": ref,
        "publication_review": review, "inspection": result,
        "reviewed_evidence": None if review_overlay is None else {
            "producer_candidate": PRODUCER, "packet_sha256": PACKET,
            "review_sha256": review_digest, "review_inventory_sha256": review_inventory_digest,
            "review_inventory": review_inventory,
            "independent_review_validation": "pass",
            "scope": "Exact evidence import and producer/review validation; independent final-delta review still required."},
        "activation_performed": False, "scope": "Preparation and library inspection only; supported CLI/preflight capture still required."}, indent=2))


if __name__ == "__main__":
    main()
