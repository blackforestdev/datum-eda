"""Replay the existing synchronized claim transaction into an isolated candidate."""

import argparse
import json
import os
from pathlib import Path
import subprocess
import sys
import tempfile


ROOT = Path(__file__).resolve().parents[5]
BASE = "e31e7a5c0c1140825d206e4fc1eed33006395f27"
RENEWAL = "21b3330f6192cfcaea071e6cab05dbf4785164ca"
REF = "refs/datum-wdq/candidates/compat-current-claim-20260909"


def git(*args, data=None, env=None):
    return subprocess.check_output(["git", *args], cwd=ROOT, input=data, env=env)


def closeout_view(args):
    """Transfer only the committed WDQ fields, preserving candidate mapping."""
    path = "specs/active_frontier.json"
    load = lambda ref: json.loads(git("show", ref + ":" + path))
    before, after, candidate = load(args.transaction + "^"), load(args.transaction), load(args.base)
    item = lambda manifest: next(row for row in manifest["frontier"]
                                if row["key"] == "WORKFLOW-DELIVERY-IMPLEMENTATION")
    old, new, target = item(before), item(after), item(candidate)
    for key in ("claim", "summary"):
        assert target[key] == old[key]
        target[key] = new[key]
    key = "canonical_next_step_id"
    assert target["completion"][key] == old["completion"][key]
    target["completion"][key] = new["completion"][key]
    for sid in ("WDQ-COMPAT", "WDQ-RECHECK"):
        prior = next(row for row in old["completion"]["steps"] if row["id"] == sid)
        revised = next(row for row in new["completion"]["steps"] if row["id"] == sid)
        index = next(i for i, row in enumerate(target["completion"]["steps"]) if row["id"] == sid)
        assert target["completion"]["steps"][index] == prior
        target["completion"]["steps"][index] = revised
    added = [p for p in new["governing_docs"] if p not in old["governing_docs"]]
    assert added == ["docs/reviews/workflow-delivery-rollout/infrastructure/workspace-renewal/producer-closeout.json"]
    assert all(p not in target["governing_docs"] for p in added)
    target["governing_docs"].extend(added)
    frozen = load(args.base)
    assert [r for r in candidate["frontier"] if r["key"] != target["key"]] == [r for r in frozen["frontier"] if r["key"] != target["key"]]
    assert target["completion"]["delivery"] == item(frozen)["completion"]["delivery"]
    runtime = ROOT / ".git/datum-wdq/proposals/wdq-final-owner-disposed-producer-20260909"
    sys.path.insert(0, str(runtime / "scripts"))
    from project_status import render_block, replace_block
    progress = git("show", args.base + ":specs/PROGRESS.md").decode()
    return {path: (json.dumps(candidate, indent=2) + "\n").encode(),
            "specs/PROGRESS.md": replace_block(progress, render_block(candidate)).encode()}


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--base", default=BASE)
    parser.add_argument("--transaction", default=RENEWAL)
    parser.add_argument("--ref", default=REF)
    parser.add_argument("--allowed-path", action="append")
    parser.add_argument("--closeout-view", action="store_true")
    args = parser.parse_args()
    assert args.ref.startswith("refs/datum-wdq/candidates/")
    head, status = git("rev-parse", "HEAD"), git("status", "--porcelain")
    paths = git("diff-tree", "--no-commit-id", "--name-only", "-r", args.transaction).decode().splitlines()
    allowed = args.allowed_path or [".beads/issues.jsonl", "specs/active_frontier.json"]
    assert sorted(paths) == sorted(allowed)
    views = closeout_view(args) if args.closeout_view else {}
    patch_paths = [p for p in paths if p not in views]
    patch = git("diff", args.transaction + "^", args.transaction, "--", *patch_paths)
    retained = ROOT / ".git/datum-wdq/proposals/wdq-full-candidate-inspection-20260909"
    with tempfile.TemporaryDirectory(prefix="claim-index-", dir=retained) as temp:
        env = dict(os.environ, GIT_INDEX_FILE=str(Path(temp) / "index"))
        git("read-tree", args.base, env=env)
        git("apply", "--cached", "--check", data=patch, env=env)
        git("apply", "--cached", data=patch, env=env)
        for path, data in views.items():
            oid = git("hash-object", "-w", "--stdin", data=data).decode().strip()
            git("update-index", "--cacheinfo", "100644", oid, path, env=env)
        tree = git("write-tree", env=env).decode().strip()
    message = f"""chore(workflow): import synchronized workflow transaction into candidate

Problem: The isolated evidence candidate needs current committed workflow state.
Change: Import the main-committed {args.transaction} transaction
through a private index. No fabricated tracker export or renewed authorization.
Closeout view mode: {args.closeout_view}; preserves candidate-only mapping and
governing-list differences, transfers exact changed WDQ fields and regenerates
the projection using the candidate renderer. All other files use exact patching.
Proof: Patch application and field preconditions checked; all {len(paths)} changed
paths match the caller's explicit allowlist. Full candidate entrypoint
verification remains a separate recorded check.
Roadmap: Import only the recorded WDQ lifecycle for dat-wdq-rollout-implementation-ffy;
dat-wdq-workspace-inputs-zmt remains open. No runtime, dependency/license,
product lane, installed trust, hook, activation or acceptance changes.
"""
    candidate = git("commit-tree", tree, "-p", args.base, data=message.encode()).decode().strip()
    git("update-ref", args.ref, candidate, "0" * 40)
    assert git("rev-parse", "HEAD") == head and git("status", "--porcelain") == status
    print(json.dumps({"schema_version": 1, "base_candidate": args.base,
        "imported_main_transaction": args.transaction, "candidate": candidate,
        "candidate_ref": args.ref, "tree": tree, "changed_paths": paths,
        "exact_patch_applied": not args.closeout_view,
        "closeout_fields_imported": args.closeout_view, "main_unchanged": True,
        "activation_performed": False}))


if __name__ == "__main__":
    main()
