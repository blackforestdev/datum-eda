"""Replay the existing synchronized claim transaction into an isolated candidate."""

import json
import os
from pathlib import Path
import subprocess
import tempfile


ROOT = Path(__file__).resolve().parents[5]
BASE = "e31e7a5c0c1140825d206e4fc1eed33006395f27"
RENEWAL = "21b3330f6192cfcaea071e6cab05dbf4785164ca"
REF = "refs/datum-wdq/candidates/compat-current-claim-20260909"


def git(*args, data=None, env=None):
    return subprocess.check_output(["git", *args], cwd=ROOT, input=data, env=env)


def main():
    head, status = git("rev-parse", "HEAD"), git("status", "--porcelain")
    paths = git("diff-tree", "--no-commit-id", "--name-only", "-r", RENEWAL).decode().splitlines()
    assert sorted(paths) == [".beads/issues.jsonl", "specs/active_frontier.json"]
    patch = git("diff", RENEWAL + "^", RENEWAL, "--", *paths)
    retained = ROOT / ".git/datum-wdq/proposals/wdq-full-candidate-inspection-20260909"
    with tempfile.TemporaryDirectory(prefix="claim-index-", dir=retained) as temp:
        env = dict(os.environ, GIT_INDEX_FILE=str(Path(temp) / "index"))
        git("read-tree", BASE, env=env)
        git("apply", "--cached", "--check", data=patch, env=env)
        git("apply", "--cached", data=patch, env=env)
        tree = git("write-tree", env=env).decode().strip()
    message = """chore(workflow): import synchronized compatibility claim into candidate

Problem: The evidence candidate retained the expired historical WDQ-COMPAT lease.
Change: Replay only the exact main-committed 21b3330f Frontier/Beads transaction
through a private index. No fabricated tracker export or renewed authorization.
Proof: Exact patch application checked before import; only two governance paths
change. Full candidate entrypoint verification remains a separate recorded check.
Roadmap: WDQ-COMPAT and dat-wdq-rollout-implementation-ffy remain in progress;
dat-wdq-workspace-inputs-zmt remains open. No runtime, dependency/license,
product lane, installed trust, hook, activation or acceptance changes.
"""
    candidate = git("commit-tree", tree, "-p", BASE, data=message.encode()).decode().strip()
    git("update-ref", REF, candidate, "0" * 40)
    assert git("rev-parse", "HEAD") == head and git("status", "--porcelain") == status
    print(json.dumps({"schema_version": 1, "base_candidate": BASE,
        "imported_main_transaction": RENEWAL, "candidate": candidate,
        "candidate_ref": REF, "tree": tree, "changed_paths": paths,
        "exact_patch_applied": True, "main_unchanged": True,
        "activation_performed": False}))


if __name__ == "__main__":
    main()
