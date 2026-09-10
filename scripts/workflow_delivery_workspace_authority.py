"""Load an optional workspace policy from an explicit immutable authority tree.

No file is created here. Candidate-only additions, changes, redirects and removal
refuse; an absent policy leaves the legacy strict enumeration behavior intact.
"""

import os
from pathlib import Path
import sys

from workflow_delivery_io import parse_json
from workflow_delivery_shapes import require
from workflow_delivery_workspace import WorkspaceInputs, workspace_policy_shape


WORKSPACE_POLICY_PATH = "specs/workflow_delivery/workspace-inputs.json"


def source_only_import():
    """Observe this import's loader and its matching installed source finder.

    This is a supported fresh-process startup observation, not authentication
    against arbitrary Python code already executing inside the process.
    Trust separately authenticates the running implementation source bytes.
    """
    loader_type = type(globals().get("__loader__"))
    directory = str(Path(__file__).resolve().parent)
    for finder in sys.meta_path:
        method = getattr(type(finder), "find_spec", None)
        namespace = getattr(method, "__globals__", {})
        if (namespace.get("SourceOnlyLoader") is loader_type
                and namespace.get("RepositorySourceFinder") is type(finder)
                and getattr(finder, "root", None) == directory):
            return True
    return False


def load_workspace(candidate, authority, *, policy_schema):
    require(authority.revision is not None and candidate.root == authority.root,
            WORKSPACE_POLICY_PATH, "explicit same-repository authority commit required", "WDQ-TRUST")
    approved_entry = authority.entries.get(WORKSPACE_POLICY_PATH)
    proposed_entry = candidate.entries.get(WORKSPACE_POLICY_PATH)
    live_present = not (candidate.staged or candidate.revision) and os.path.lexists(
        candidate.root / WORKSPACE_POLICY_PATH)
    if approved_entry is None:
        require(proposed_entry is None and not live_present, WORKSPACE_POLICY_PATH,
                "candidate-only workspace policy requires owner promotion", "WDQ-POLICY")
        return None
    require(policy_schema == 2, WORKSPACE_POLICY_PATH,
            "workspace policy requires broad schema-2 authority", "WDQ-POLICY")
    require(approved_entry[0] == "100644" and proposed_entry is not None
            and proposed_entry[0] == "100644", WORKSPACE_POLICY_PATH,
            "workspace policy must remain tracked non-executable regular data", "WDQ-POLICY")
    if not (candidate.staged or candidate.revision):
        path = candidate.root / WORKSPACE_POLICY_PATH
        require(path.is_file() and path.resolve(strict=True) == path
                and not path.stat().st_mode & 0o111, WORKSPACE_POLICY_PATH,
                "workspace policy worktree redirect or executable mode refused", "WDQ-POLICY")
    raw = authority.read(WORKSPACE_POLICY_PATH, committed=True)
    require(candidate.read(WORKSPACE_POLICY_PATH, committed=True) == raw,
            WORKSPACE_POLICY_PATH, "workspace policy change requires owner promotion", "WDQ-POLICY")
    policy = workspace_policy_shape(parse_json(raw, WORKSPACE_POLICY_PATH))
    isolated = source_only_import()
    require(isolated or (policy["python_caches"] == "none" and not any(
        row["category"] == "legacy_python_cache" for row in policy["local_files"])),
        WORKSPACE_POLICY_PATH, "cache policy requires verified source-only startup", "WDQ-TRUST")
    tracked = set(candidate.entries) | set(authority.entries)
    return WorkspaceInputs(candidate.root, policy, tracked, cache_isolated=isolated)


def promotion_workspace(root, *, candidate, authority):
    """Read prospective pins after caller authenticates its prepared bundle.

    Pre-publication worktree does not yet contain the proposed policy. Read its
    bytes from the exact candidate instead; this supplies no activation approval.
    """
    from workflow_delivery_tree import Tree
    from workflow_delivery_trust import POLICY_PATH, policy_shape

    proposed = Tree(root, revision=candidate)
    approved = Tree(root, revision=authority)
    schema = policy_shape(approved.json(POLICY_PATH))["schema_version"]
    return load_workspace(proposed, approved, policy_schema=schema)
