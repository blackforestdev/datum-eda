"""Resolve proposed Git-common support locations without creating or trusting them.

Path validation is not byte verification, immutability, promotion or activation.
Existing external support remains owned by its installed legacy hook.
"""

from pathlib import Path

from workflow_delivery_capture_state import git
from workflow_delivery_shapes import commit_id, require
from workflow_delivery_bootstrap import pinned_commit
from workflow_delivery_io import normalized_path


def support_locations(root, authority):
    root = Path(root).resolve(strict=True)
    require(git(root, "rev-parse", "--show-toplevel").decode().strip() == str(root),
            "support", "exact worktree root required", "WDQ-TRUST")
    commit_id(authority, "authority")
    resolved = git(root, "rev-parse", "--verify", authority + "^{commit}").decode().strip()
    require(resolved == authority, "authority", "exact commit object required", "WDQ-TRUST")
    common = Path(git(root, "rev-parse", "--path-format=absolute", "--git-common-dir").decode().strip()).resolve(strict=True)
    locations = {"common": common, "store": common / "datum-wdq",
                 "proposals": common / "datum-wdq/proposals",
                 "logs": common / "datum-wdq/logs",
                 "trusted": common / "datum-wdq/trusted" / authority}
    locations["scripts"] = locations["trusted"] / "scripts"
    locations["hooks"] = locations["trusted"] / "owner-hooks"
    for name, path in locations.items():
        if name == "common":
            continue
        require(path.resolve() == path, str(path),
                "support paths cannot traverse symlink redirections", "WDQ-TRUST")
        require(not path.exists() or path.is_dir(), str(path),
                "support directory path is occupied by a file", "WDQ-TRUST")
    locations["runner"] = locations["scripts"] / "check_workflow_delivery.py"
    locations["hook"] = locations["hooks"] / "pre-commit"
    return locations


def validate_support_entry(root, authority, path, *, entry):
    require(entry in ("runner", "hook"), "support", "known support entry required", "WDQ-TRUST")
    expected = support_locations(root, authority)[entry]
    path = Path(path)
    require(path.is_absolute() and path == expected, str(path),
            "entry must use the exact owner-pinned Git-common location", "WDQ-TRUST")
    require(path.is_file() and not path.is_symlink() and path.resolve() == expected,
            str(path), "existing nonredirected support entry required", "WDQ-TRUST")
    return path


def proposed_local_trust(root, *, authority, base, environment_path):
    """Describe exact replacement values without selecting or installing them.

    The containing request must be reviewed and owner-selected separately.
    Location resolution does not establish environment validity or bundle bytes.
    """
    pinned_commit(root, base)
    normalized_path(environment_path)
    require(".git" not in Path(environment_path).parts, "environment",
            "environment must be repository data, not Git metadata", "WDQ-TRUST")
    locations = support_locations(root, authority)
    return {"core.hooksPath": [str(locations["hooks"])],
            "datum.workflowDeliveryAuthorityRef": [authority],
            "datum.workflowDeliveryBaseRef": [base],
            "datum.workflowDeliveryEnvironmentPath": [environment_path],
            "datum.workflowDeliveryRunnerPath": [str(locations["runner"])]}
