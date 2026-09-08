"""Read-only publication preconditions; never activate or claim owner approval.

An eventual activation transaction must revalidate these observations under its
own coordination boundary. Successful preparation is not a lock or permission.
"""

import os
from pathlib import Path

from workflow_delivery_bootstrap import git
from workflow_delivery_capture_state import TRUST_KEYS, protected_state
from workflow_delivery_io import canonical_json, sha256
from workflow_delivery_publication_delta import publication_delta, tree_entries

REDIRECTS = ("GIT_DIR", "GIT_WORK_TREE", "GIT_COMMON_DIR", "GIT_INDEX_FILE", "GIT_NAMESPACE",
             "GIT_OBJECT_DIRECTORY", "GIT_ALTERNATE_OBJECT_DIRECTORIES", "GIT_CONFIG",
             "GIT_CONFIG_COUNT", "GIT_CONFIG_PARAMETERS")


def require(condition, message):
    if not condition:
        raise ValueError("WDQ-PREFLIGHT: " + message)


def clean_inputs(root, snapshot, base):
    entries = tree_entries(root, base)
    for path, entry in entries.items():
        state = snapshot["files"].get(path, {"kind": "missing"})
        mode = entry["mode"]
        require(mode in ("100644", "100755", "120000"), "unsupported tracked mode: " + path)
        if mode == "120000":
            require(state["kind"] == "symlink", "tracked symlink changed: " + path)
            actual = sha256(os.fsencode(state["target"]))
        else:
            require(state["kind"] == "file", "tracked file absent or redirected: " + path)
            require(bool(state["mode"] & 0o111) == (mode == "100755"), "tracked executable mode changed: " + path)
            actual = state["sha256"]
        require(actual == sha256(git(root, "cat-file", "blob", entry["oid"])),
                "tracked bytes differ from base: " + path)
    for path, state in snapshot["files"].items():
        require(path in entries or state["kind"] == "directory",
                "untracked, ignored or missing reviewed input: " + path)


def preflight(root, *, base, candidate, input_roots, expected_local_trust):
    require(not any(name in os.environ for name in REDIRECTS), "repository/index/config overrides are not activation inputs")
    require(type(input_roots) is list and bool(input_roots) and len(set(input_roots)) == len(input_roots),
            "explicit unique reviewed input roots required")
    require(type(expected_local_trust) is dict and set(expected_local_trust) == set(TRUST_KEYS)
            and all(type(values) is list and all(type(value) is str for value in values)
                    for values in expected_local_trust.values()), "exact named prior local trust required")
    root = Path(root).resolve(strict=True)
    before = protected_state(root, input_roots)
    require(before["symbolic_head"] == "refs/heads/main",
            "publication requires an attached main checkout; no branch switch performed")
    require(before["head"] == base, "live HEAD differs from pinned publication base")
    require(before["local_trust"] == expected_local_trust, "local trust differs from reviewed prior state")
    require(git(root, "status", "--porcelain=v1", "--untracked-files=all") == b"",
            "dirty index/worktree or untracked files; no publication performed")
    staged = git(root, "diff", "--cached", "--raw", "--no-ext-diff", "--no-textconv", base, "--")
    require(staged == b"", "index differs from pinned publication base")
    clean_inputs(root, before, base)
    delta = publication_delta(root, base=base, candidate=candidate)
    after = protected_state(root, input_roots)
    require(canonical_json(before) == canonical_json(after), "protected state changed during preflight")
    require(git(root, "status", "--porcelain=v1", "--untracked-files=all") == b"",
            "worktree changed during preflight")
    return {"schema_version": 1, "kind": "datum.workflow-delivery.activation-preflight",
            "base": base, "candidate": candidate, "protected_state": before,
            "publication_delta": delta, "publication_authorized": False, "activation_asserted": False,
            "scope": "read-only preconditions; owner receipt, independent evidence and activation coordination still required"}
