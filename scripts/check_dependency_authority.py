#!/usr/bin/env python3
"""Enforce decision 029's dependency and Datum-owned terminal boundary."""

from __future__ import annotations

import argparse
import json
import subprocess
import sys
from pathlib import Path


def cargo_external_dependencies(root: Path) -> set[str]:
    result = subprocess.run(
        ["cargo", "metadata", "--no-deps", "--format-version", "1"],
        cwd=root,
        check=True,
        capture_output=True,
        text=True,
    )
    metadata = json.loads(result.stdout)
    members = {package["name"] for package in metadata["packages"]}
    return {
        dependency["name"]
        for package in metadata["packages"]
        for dependency in package["dependencies"]
        if dependency["name"] not in members
    }


def check(root: Path) -> list[str]:
    failures: list[str] = []
    policy_path = root / "specs/third_party_dependency_policy.json"
    policy = json.loads(policy_path.read_text(encoding="utf-8"))
    allowed = set(policy["inherited_direct_external_dependencies"])
    actual = cargo_external_dependencies(root)
    additions = sorted(actual - allowed)
    removals = sorted(allowed - actual)
    if additions:
        failures.append(
            "unratified direct external dependencies: " + ", ".join(additions)
        )
    if removals:
        failures.append(
            "dependency baseline contains absent entries; ratchet it in the same change: "
            + ", ".join(removals)
        )

    forbidden = tuple(policy["forbidden_terminal_dependencies"])
    source_roots = [root / "Cargo.toml", root / "Cargo.lock", root / "crates"]
    for source_root in source_roots:
        paths = [source_root] if source_root.is_file() else source_root.rglob("*")
        for path in paths:
            if not path.is_file() or path.suffix not in {".rs", ".toml", ".lock"}:
                continue
            content = path.read_text(encoding="utf-8", errors="replace")
            for token in forbidden:
                if token in content:
                    failures.append(
                        f"forbidden terminal dependency token {token!r} in {path.relative_to(root)}"
                    )

    for path in (
        root / "third_party/libghostty-vt",
        root / "scripts/build_libghostty_vt.py",
    ):
        if path.exists():
            failures.append(f"forbidden terminal dependency path exists: {path.relative_to(root)}")
    return failures


def main() -> int:
    parser = argparse.ArgumentParser()
    parser.add_argument("--root", type=Path, default=Path(__file__).resolve().parents[1])
    args = parser.parse_args()
    failures = check(args.root.resolve())
    if failures:
        for failure in failures:
            print(f"FAIL: {failure}")
        return 1
    print("dependency authority: PASS")
    return 0


if __name__ == "__main__":
    sys.exit(main())
