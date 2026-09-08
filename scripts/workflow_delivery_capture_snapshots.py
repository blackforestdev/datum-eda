"""Deliberately divergent fixture snapshots, never repair of observed inputs."""

from workflow_delivery_capture_state import git
from workflow_delivery_io import canonical_json, parse_json, sha256


POLICY = "specs/workflow_delivery_policy.json"
UNTRACKED_CASES = {"INFRA-S05-05.untracked", "INFRA-S05-05.ignored"}
SNAPSHOT_CASES = {
    "INFRA-S05-01.staged-bad": (POLICY, "WDQ-POLICY"),
    "INFRA-S05-02.worktree-bad": (POLICY, "WDQ-POLICY"),
    "INFRA-S05-03.candidate-isolation": (POLICY, "WDQ-POLICY"),
}
SNAPSHOT_CASES.update({key: ("src/unscoped.py", "WDQ-COVERAGE") for key in UNTRACKED_CASES})


def snapshot_expectation(case_id, surface):
    if case_id not in SNAPSHOT_CASES:
        raise ValueError("explicit snapshot case required")
    if case_id == "INFRA-S05-03.candidate-isolation":
        if surface != "R":
            raise ValueError("candidate isolation requires surface R")
        return None, None, None
    if surface not in ("C", "H", "S", "S-check", "S-details"):
        raise ValueError("index/worktree divergence requires C, H or S")
    if case_id in UNTRACKED_CASES:
        return (("src/unscoped.py", "WDQ-COVERAGE", "production change has no promoted scope")
                if surface.startswith("S") else (None, None, None))
    bad = (surface in ("C", "H")) == (case_id == "INFRA-S05-01.staged-bad")
    return ((POLICY, "WDQ-POLICY", "policy change requires owner-controlled promotion")
            if bad else (None, None, None))


def prepare_snapshots(root, case_id):
    if case_id not in SNAPSHOT_CASES:
        raise ValueError("explicit snapshot case required")
    if not (root / ".git/datum-wdq-capture-owner").is_file() or git(root, "status", "--porcelain"):
        raise ValueError("clean nonce-owned snapshot fixture required")
    if case_id in UNTRACKED_CASES:
        target = "src/unscoped.py"
        path = root / target
        if path.exists():
            raise ValueError("fresh untracked source path required")
        exclude = root / ".git/info/exclude"
        if exclude.is_symlink():
            raise ValueError("regular local exclusion input required")
        before = exclude.read_bytes() if exclude.exists() else b""
        if case_id.endswith(".ignored"):
            exclude.write_bytes(before + b"\n/src/unscoped.py\n")
        path.write_bytes(b"# Synthetic unstaged production input.\n")
        return {"case_id": case_id, "path": target, "worktree_bytes_hex": path.read_bytes().hex(),
                "exclude_before_hex": before.hex(),
                "exclude_after_hex": (exclude.read_bytes() if exclude.exists() else b"").hex(),
                "head": git(root, "rev-parse", "HEAD").decode().strip(), "acceptance_asserted": False}
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
