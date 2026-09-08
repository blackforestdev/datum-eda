"""Deliberately divergent fixture snapshots, never repair of observed inputs."""

from workflow_delivery_capture_state import git
from workflow_delivery_io import canonical_json, parse_json, sha256


POLICY = "specs/workflow_delivery_policy.json"
SNAPSHOT_CASES = {
    "INFRA-S05-01.staged-bad": (POLICY, "WDQ-POLICY"),
    "INFRA-S05-02.worktree-bad": (POLICY, "WDQ-POLICY"),
    "INFRA-S05-03.candidate-isolation": (POLICY, "WDQ-POLICY"),
}


def snapshot_expectation(case_id, surface):
    if case_id not in SNAPSHOT_CASES:
        raise ValueError("explicit snapshot case required")
    if case_id == "INFRA-S05-03.candidate-isolation":
        if surface != "R":
            raise ValueError("candidate isolation requires surface R")
        return None, None, None
    if surface not in ("C", "H", "S", "S-check", "S-details"):
        raise ValueError("index/worktree divergence requires C, H or S")
    bad = (surface in ("C", "H")) == (case_id == "INFRA-S05-01.staged-bad")
    return ((POLICY, "WDQ-POLICY", "policy change requires owner-controlled promotion")
            if bad else (None, None, None))


def prepare_snapshots(root, case_id):
    if case_id not in SNAPSHOT_CASES:
        raise ValueError("explicit snapshot case required")
    if not (root / ".git/datum-wdq-capture-owner").is_file() or git(root, "status", "--porcelain"):
        raise ValueError("clean nonce-owned snapshot fixture required")
    path = root / POLICY
    original = path.read_bytes()
    bad_policy = parse_json(original, POLICY)
    bad_policy["coverage"]["rows"][0]["category"] = "product"
    bad_index = canonical_json(bad_policy)
    if case_id == "INFRA-S05-03.candidate-isolation":
        bad_policy["coverage"]["production_roots"].append("synthetic-unapproved-root")
        worktree = canonical_json(bad_policy)
        staged = bad_index
    elif case_id == "INFRA-S05-01.staged-bad":
        staged, worktree = bad_index, original
    else:
        staged, worktree = original, bad_index
    path.write_bytes(staged)
    git(root, "add", "--", POLICY)
    path.write_bytes(worktree)
    return {"case_id": case_id, "path": POLICY,
            "head": git(root, "rev-parse", "HEAD").decode().strip(),
            "head_bytes_sha256": sha256(original), "index_bytes_hex": staged.hex(),
            "worktree_bytes_hex": worktree.hex(),
            "index_blob": git(root, "rev-parse", ":" + POLICY).decode().strip(),
            "acceptance_asserted": False}
