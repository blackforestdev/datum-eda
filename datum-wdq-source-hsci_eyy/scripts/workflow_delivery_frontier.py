"""Reuse PM025 validation against isolated candidate metadata, including index bytes.

Only the private temporary metadata view is written. Candidate files, Git index,
Git configuration and authority records are never modified. Git commit lookups
use the existing object database through a temporary gitdir pointer.
"""

from pathlib import Path
import tempfile

from workflow_delivery_io import normalized_path
from workflow_delivery_shapes import require


def _reference_paths(value):
    if type(value) is dict:
        if type(value.get("path")) is str:
            yield value["path"]
        for child in value.values():
            yield from _reference_paths(child)
    elif type(value) is list:
        for child in value:
            yield from _reference_paths(child)


def validate_frontier(tree):
    from project_status import validate, render_status
    manifest = tree.json("specs/active_frontier.json")
    require(type(manifest) is dict and type(manifest.get("frontier")) is list,
            "specs/active_frontier.json", "Frontier object/array required", "WDQ-TRANSITION")
    paths = {"specs/active_frontier.json", "specs/spec_governance_manifest.json",
             ".beads/issues.jsonl", "specs/PROGRESS.md"}
    if type(manifest.get("policy_decision")) is str:
        paths.add(manifest["policy_decision"])
    paths.update(_reference_paths(manifest))
    for item in manifest["frontier"]:
        if type(item) is dict and type(item.get("governing_docs")) is list:
            paths.update(p for p in item["governing_docs"] if type(p) is str)
    gitdir = tree.git("rev-parse", "--absolute-git-dir").decode().strip()
    with tempfile.TemporaryDirectory(prefix="datum-wdq-frontier-") as directory:
        root = Path(directory)
        for path in sorted(paths):
            normalized_path(path)
            require(".git" not in Path(path).parts, path, "Git metadata cannot be candidate evidence",
                    "WDQ-ARTIFACT")
            target = root / path
            target.parent.mkdir(parents=True, exist_ok=True)
            target.write_bytes(tree.read(path))
        (root / ".git").write_text("gitdir: " + gitdir + "\n", encoding="utf-8")
        # No public CLI switch disables delivery. This call prevents recursion:
        # the caller immediately applies the separately trusted delivery checks.
        failures, state = validate(root, delivery_checks=False)
        require(not failures, "specs/active_frontier.json", "; ".join(failures), "WDQ-TRANSITION")
        current, _ = render_status(root, False)
        require(current, "specs/PROGRESS.md", "candidate Frontier projection is stale", "WDQ-TRANSITION")
    return state["manifest"]
