"""Prepare a current-lease real-roadmap fixture without changing runtime or main."""

import argparse
import hashlib
import json
import os
from pathlib import Path
import subprocess
import sys
import tempfile

sys.path.insert(0, str(Path(__file__).resolve().parent))
from run_index_repair_batch import PIN, save


TRANSACTION = "7c36cbb6"
KEY = "WORKFLOW-DELIVERY-IMPLEMENTATION"
OWNED = {"dat-wdq-rollout-implementation-ffy", "dat-wdq-workspace-inputs-zmt", "dat-wdq-index-refresh-ogw", "dat-wdq-proof-renewal-cycle-qgb"}


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--owner-renewal-transaction")
    parser.add_argument("--reconciled", action="store_true")
    args = parser.parse_args()
    renewal = args.owner_renewal_transaction is not None
    root = Path(__file__).resolve().parents[5]
    runtime = root / ".git/datum-wdq/proposals/index-preservation-repair-20260910"
    store = root / ".git/datum-wdq/proposals/index-repair-evidence-20260910"

    def git(*args, cwd=runtime, data=None, env=None):
        return subprocess.check_output(["git", "--no-optional-locks", *args], cwd=cwd, input=data, env=env)

    assert git("rev-parse", "HEAD").decode().strip() == PIN and not git("status", "--porcelain")
    # Evidence artifacts may be added concurrently; protect tracked state here.
    main_state = lambda: (git("rev-parse", "HEAD", cwd=root),
        git("status", "--porcelain", "--untracked-files=no", cwd=root),
        git("diff", "--binary", "HEAD", cwd=root))
    before_main = main_state()
    transaction = git("rev-parse", args.owner_renewal_transaction or TRANSACTION, cwd=root).decode().strip()
    path = "specs/active_frontier.json"
    original = json.loads(git("show", PIN + ":" + path))
    frontier = json.loads(json.dumps(original))
    recorded = json.loads(git("show", transaction + ":" + path, cwd=root))
    item = lambda value: next(row for row in value["frontier"] if row["key"] == KEY)
    assert item(frontier)["authorization"] == item(recorded)["authorization"] == "execution"
    assert item(frontier)["completion"]["canonical_next_step_id"] == "WDQ-RECHECK"
    item(frontier)["claim"] = item(recorded)["claim"]
    additional = {}
    if renewal:
        authority_path = "docs/reviews/workflow-delivery-rollout/infrastructure/workspace-renewal/proof-renewal-authorization.json"
        authority_bytes = git("show", transaction + ":" + authority_path, cwd=root)
        authorization = json.loads(authority_bytes)
        assert authorization["owner_response_verbatim"] == "approved"
        assert authorization["renewal"]["runtime_source_commit"] == PIN
        assert authorization["selected_step"] == "WDQ-COMPAT" and not authorization["activation_authorized"]
        current, approved = item(frontier), item(recorded)
        assert approved["completion"]["canonical_next_step_id"] == "WDQ-COMPAT"
        current["summary"] = approved["summary"]
        assert authority_path in approved["governing_docs"] and authority_path not in current["governing_docs"]
        current["governing_docs"].append(authority_path)
        current["completion"]["canonical_next_step_id"] = "WDQ-COMPAT"
        for step in current["completion"]["steps"]:
            if step["id"] in ("WDQ-COMPAT", "WDQ-RECHECK"):
                source = next(s for s in approved["completion"]["steps"] if s["id"] == step["id"])
                step.update(status=source["status"], completion_evidence=source["completion_evidence"])
        # Preserve candidate-only schema/mapping; copy only the new authorization ledger entry.
        governance_path = "specs/spec_governance_manifest.json"
        governance = json.loads(git("show", PIN + ":" + governance_path))
        approved_governance = json.loads(git("show", transaction + ":" + governance_path, cwd=root))
        governance["entries"][authority_path] = approved_governance["entries"][authority_path]
        additional = {authority_path: authority_bytes,
                      governance_path: (json.dumps(governance, indent=2) + "\n").encode()}
        # Make the recorded claim's actual history available to the isolated export.
        git("fetch", "--quiet", "--no-tags", str(root), transaction + ":refs/datum-wdq/owner-renewal/" + transaction)
    assert [row for row in frontier["frontier"] if row["key"] != KEY] == [row for row in original["frontier"] if row["key"] != KEY]
    beads_path = ".beads/issues.jsonl"
    old_beads = git("show", PIN + ":" + beads_path)
    new_beads = git("show", transaction + ":" + beads_path, cwd=root)
    unrelated = lambda raw: {json.loads(line)["id"]: line for line in raw.splitlines() if json.loads(line)["id"] not in OWNED}
    assert unrelated(old_beads) == unrelated(new_beads)
    sys.path.insert(0, str(runtime / "scripts"))
    from project_status import render_block, replace_block
    from workflow_delivery_tree import Tree
    from workflow_delivery_authority import authority_sha256
    from workflow_delivery_roadmap_snapshot import prepare_roadmap_snapshot, frontier_commits
    from workflow_delivery_support_store import support_locations

    progress = git("show", PIN + ":specs/PROGRESS.md").decode()
    payloads = {path: (json.dumps(frontier, indent=2) + "\n").encode(), beads_path: new_beads,
                "specs/PROGRESS.md": replace_block(progress, render_block(frontier)).encode()}
    payloads.update(additional)
    with tempfile.TemporaryDirectory(prefix="roadmap-index-", dir=store) as directory:
        env = dict(os.environ, GIT_INDEX_FILE=str(Path(directory) / "index"))
        git("read-tree", PIN, env=env)
        for name, raw in payloads.items():
            oid = git("hash-object", "-w", "--stdin", data=raw).decode().strip()
            git("update-index", "--add", "--cacheinfo", "100644", oid, name, env=env)
        tree = git("write-tree", env=env).decode().strip()
    message = ("test(workflow): retain authorized real-roadmap renewal fixture\n\n"
        "Problem: Historical lease expires before renewed real-roadmap replay.\n"
        "Change: Copy the recorded WDQ claim and exact canonical Beads export from " + transaction + ".\n"
        + ("Apply the explicit owner COMPAT reopening and its authority record; preserve candidate-only delivery mapping.\n" if renewal else "") +
        "Regenerate the matching projection; preserve all other Frontier rows and non-owned tracker bytes.\n"
        "Proof: Source manifest and authority equality are verified separately before snapshot export.\n"
        "Roadmap: WDQ-COMPAT/WDQ-RECHECK, dat-wdq-rollout-implementation-ffy and dat-wdq-index-refresh-ogw;\n"
        "fixture only, no new authorization, live trust, product, dependency or licensing changes.\n")
    candidate = git("commit-tree", tree, "-p", PIN, data=message.encode()).decode().strip()
    base, adapted = Tree(runtime, revision=PIN), Tree(runtime, revision=candidate)
    contract = base.json("specs/workflow_delivery/rollout.contract.json")
    assert base.manifest(contract["input_roots"]) == adapted.manifest(contract["input_roots"])
    assert authority_sha256(base, contract) == authority_sha256(adapted, contract)
    ref = "refs/datum/workflow-delivery-candidates/" + candidate
    git("update-ref", ref, candidate, "0" * 40)
    evidence = {}
    for commit in sorted(frontier_commits(frontier)):
        probe = subprocess.run(["git", "merge-base", "--is-ancestor", commit, candidate], cwd=runtime)
        assert probe.returncode in (0, 1)
        if probe.returncode:
            name = "refs/datum/workflow-delivery-candidates/" + commit
            existing = subprocess.run(["git", "rev-parse", "--verify", name], cwd=runtime, capture_output=True, text=True)
            if existing.returncode == 0:
                assert existing.stdout.strip() == commit
            else:
                git("update-ref", name, commit, "0" * 40)
            evidence[name] = commit
    suffix = "owner-reopened-roadmap-20260910" if renewal else "index-repair-current-roadmap-20260910"
    if args.reconciled:
        assert renewal
        suffix += "-reconciled"
    packet = support_locations(runtime, candidate)["proposals"] / suffix
    result = prepare_roadmap_snapshot(runtime, packet, source_ref=ref, source_commit=candidate,
        frontier_keys=[row["key"] for row in frontier["frontier"]], evidence_refs=evidence)
    result_name = "owner-reopened-roadmap-preparation.json" if renewal else "roadmap-preparation.json"
    if args.reconciled:
        result_name = "owner-reopened-roadmap-preparation-reconciled.json"
    save(store / result_name, {"runtime_source": PIN, "fixture_commit": candidate,
        "transaction": transaction, "packet": str(packet), "snapshot": result,
        "beads_sha256": hashlib.sha256(new_beads).hexdigest(), "changed_paths": sorted(payloads),
        "authority_inputs_unchanged": True, "other_frontier_and_tracker_items_unchanged": True,
        "scope": "Recorded owner reopening, claim and canonical export copied into isolated fixture; no clock spoofing or installed authority change." if renewal else "Recorded claim and canonical export copied into isolated fixture; no clock spoofing or new owner authority.",
        "activation_performed": False, "gate_captures_performed": False})
    assert before_main == main_state()
    assert git("rev-parse", "HEAD").decode().strip() == PIN and not git("status", "--porcelain")
    print(json.dumps({"fixture": candidate, "packet": str(packet), "gate_captures_performed": False}))


if __name__ == "__main__":
    main()
