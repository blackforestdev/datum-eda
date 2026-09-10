"""Build owned synthetic net-zero histories; never mutate live development refs."""

from workflow_delivery_capture_state import git
from workflow_delivery_publication_delta import publication_delta
from workflow_delivery_io import sha256


HISTORY_CASES = {
    "INFRA-S05-07.add-delete": ("src/unscoped.py", "WDQ-COVERAGE"),
    "INFRA-S05-07.change-revert": ("src/read.py", "WDQ-COVERAGE"),
    "INFRA-S05-08.merge-parent": ("src/unscoped.py", "WDQ-COVERAGE"),
}
BASE_CASES = {
    "INFRA-S05-04.missing-base": ("git", "WDQ-TRUST", "Needed a single revision"),
    "INFRA-S05-04.nonancestor-base": ("transaction", "WDQ-COVERAGE", "candidate must descend from the explicit transaction base"),
}
CURRENT_HISTORY_CASE = "INFRA-S05-09.empty-current-transaction"


def prepare_current_history(root, base, archive):
    result = prepare_history(root, "INFRA-S05-07.add-delete", base, archive)
    result.update(case_id=CURRENT_HISTORY_CASE, history_certified=False)
    return result


def prepare_base_input(root, case_id, base, archive):
    """Retain an explicit invalid base without changing the candidate checkout."""
    if case_id not in BASE_CASES or not (root / ".git/datum-wdq-capture-owner").is_file():
        raise ValueError("explicit owned base-input fixture required")
    if archive.exists() or archive.is_symlink() or archive.resolve().is_relative_to(root.resolve()):
        raise ValueError("fresh history archive outside fixture required")
    if git(root, "rev-parse", "HEAD").decode().strip() != base or git(root, "status", "--porcelain"):
        raise ValueError("base input requires clean exact fixture head")
    selected = "0" * len(base)
    if case_id.endswith(".nonancestor-base"):
        tree = git(root, "rev-parse", "HEAD^{tree}").decode().strip()
        selected = git(root, "-c", "user.name=Fixture", "-c", "user.email=fixture@example.invalid",
            "commit-tree", tree, "-p", base, "-m", "test(workflow): retain invalid future base\n\n"
            "Synthetic nonancestor INPUT; candidate remains its parent.").decode().strip()
        git(root, "update-ref", "refs/datum-capture/nonancestor-base", selected)
    git(root, "bundle", "create", str(archive), "--all")
    git(root, "bundle", "verify", str(archive))
    return {"case_id": case_id, "base_ref": selected, "candidate_ref": base,
            "history_bundle": {"path": archive.name, "sha256": sha256(archive.read_bytes())},
            "synthetic_input_only": True, "acceptance_asserted": False}


def prepare_history(root, case_id, base, archive):
    """Caller has restored a nonce-owned fixture; retain all intermediate objects."""
    if case_id not in HISTORY_CASES:
        raise ValueError("explicit implemented history case required")
    if archive.exists() or archive.is_symlink() or archive.resolve().is_relative_to(root.resolve()):
        raise ValueError("fresh history archive outside the observed fixture required")
    if not (root / ".git/datum-wdq-capture-owner").is_file():
        raise ValueError("owned restored fixture required")
    if git(root, "rev-parse", "HEAD").decode().strip() != base or git(root, "status", "--porcelain"):
        raise ValueError("history preparation requires the clean exact fixture base")
    target = HISTORY_CASES[case_id][0]
    path = root / target
    original = path.read_bytes() if path.exists() else None
    operations = []

    def commit(label, parents):
        tree = git(root, "write-tree").decode().strip()
        args = ["-c", "user.name=Fixture", "-c", "user.email=fixture@example.invalid",
                "commit-tree", tree]
        for parent in parents:
            args += ["-p", parent]
        args += ["-m", "test(workflow): synthetic " + label + "\n\n"
                 "Retain " + case_id + " hostile INPUT, not owner or product evidence."]
        result = git(root, *args).decode().strip()
        operations.append({"label": label, "commit": result, "tree": tree, "parents": parents})
        return result

    path.write_bytes((original or b"") + b"\n# Synthetic unpermitted history input.\n")
    git(root, "add", "--", target)
    changed = commit("intermediate source change", [base])
    if original is None:
        path.unlink()  # Only the exact file this invocation just created.
    else:
        path.write_bytes(original)
    git(root, "add", "--", target)
    restored = commit("net-zero final source", [changed])
    candidate = restored
    if case_id == "INFRA-S05-08.merge-parent":
        mainline = commit("unchanged first-parent history", [base])
        candidate = commit("merge retained hostile side history", [mainline, restored])
    git(root, "update-ref", "HEAD", candidate, base)
    delta = publication_delta(root, base=base, candidate=candidate)
    if delta["net_changes"] or delta["touched_paths"] != [target]:
        raise ValueError("fixture did not retain the exact intended net-zero history")
    git(root, "bundle", "create", str(archive), "--all")
    git(root, "bundle", "verify", str(archive))
    return {"case_id": case_id, "path": target, "operations": operations,
            "history_bundle": {"path": archive.name, "sha256": sha256(archive.read_bytes())},
            "publication_delta": delta, "acceptance_asserted": False}
