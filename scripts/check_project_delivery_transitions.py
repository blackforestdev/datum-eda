#!/usr/bin/env python3
"""Check committed/staged delivery transitions without reading worktree resolutions."""
from __future__ import annotations

import argparse
import json
from pathlib import Path
import subprocess

from project_delivery_gaps import transition_errors, validate_delivery_gaps
from project_feature_delivery import validate_feature_delivery

FRONTIER = "specs/active_frontier.json"
TRACKER = ".beads/issues.jsonl"


def read_evidence(root: Path, revision: str, path: str) -> str:
    """Read a regular evidence blob from the candidate, never the working tree."""
    spec = f":{path}" if revision == ":" else f"{revision}:{path}"
    command = (["git", "ls-files", "--stage", "--", path] if revision == ":"
               else ["git", "ls-tree", revision, "--", path])
    try:
        entry = subprocess.check_output(command, cwd=root, text=True, stderr=subprocess.DEVNULL)
        if not entry.startswith(("100644 ", "100755 ")):
            raise OSError(f"ordinary evidence is not a regular candidate file: {path}")
        return subprocess.check_output(["git", "show", spec], cwd=root, text=True,
                                       stderr=subprocess.DEVNULL)
    except subprocess.CalledProcessError as error:
        raise OSError(f"ordinary evidence is unavailable in candidate: {path}") from error


def load_snapshot(root: Path, revision: str) -> tuple[dict, dict]:
    def read(path):
        spec = f":{path}" if revision == ":" else f"{revision}:{path}"
        return subprocess.check_output(["git", "show", spec], cwd=root, text=True)
    manifest = json.loads(read(FRONTIER))
    issues = {}
    for line in read(TRACKER).splitlines():
        if not line.strip():
            continue
        issue = json.loads(line)
        if issue["id"] in issues:
            raise ValueError(f"duplicate tracker identity: {issue['id']}")
        issues[issue["id"]] = issue
    return manifest, issues


def check(root: Path, base: str, candidate: str) -> list[str]:
    before, old_issues = load_snapshot(root, base)
    after, issues = load_snapshot(root, candidate)
    failures = validate_delivery_gaps(after, issues)
    for item in after.get("frontier", []):
        issue = issues.get(item["issue_id"], {})
        failures.extend(validate_feature_delivery(
            root, item, issue, read_evidence=lambda path: read_evidence(root, candidate, path)))
        declared = set(item.get("dependencies", []))
        actual = {edge.get("depends_on_id") for edge in issue.get("dependencies", [])
                  if isinstance(edge, dict) and edge.get("type") == "blocks"}
        if declared != actual:
            failures.append(f"{item['key']}: staged Frontier/tracker prerequisite mismatch")
    return sorted(set(failures + transition_errors(before, after, old_issues, issues)))


def main(argv=None) -> int:
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--root", type=Path, default=Path(__file__).resolve().parents[1])
    parser.add_argument("--staged", action="store_true")
    parser.add_argument("--base", default="HEAD^")
    args = parser.parse_args(argv)
    root = args.root.resolve()
    try:
        if args.staged:
            names = subprocess.check_output(
                ["git", "diff", "--cached", "--name-only", "-z"], cwd=root, text=True).split("\0")
            manifest, _ = load_snapshot(root, ":")
            evidence = {Path(path).as_posix() for item in manifest.get("frontier", [])
                        if isinstance(contract := item.get("feature_delivery"), dict)
                        for path in contract.get("ordinary_evidence", []) if isinstance(path, str)}
            if not ({FRONTIER, TRACKER} | evidence).intersection(names):
                print("Delivery transition check passed (no staged delivery state or evidence).")
                return 0
        failures = check(root, "HEAD" if args.staged else args.base, ":" if args.staged else "HEAD")
    except (OSError, ValueError, KeyError, TypeError, subprocess.CalledProcessError) as error:
        print(f"Delivery transition check failed: {error}")
        return 1
    if failures:
        print("Delivery transition check failed:\n- " + "\n- ".join(failures))
        return 1
    print("Delivery transition check passed.")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
