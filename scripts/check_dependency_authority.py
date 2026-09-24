#!/usr/bin/env python3
"""Enforce decision 029's dependency and Datum-owned terminal boundary."""

from __future__ import annotations

import argparse
import hashlib
import json
import subprocess
import sys
import tomllib
from pathlib import Path


def check_source_patches(root: Path, policy: dict) -> list[str]:
    """Keep the PM046 exception closed over package, path, version and notices."""
    failures: list[str] = []
    approved = policy.get("approved_source_patches", {})
    manifest = tomllib.loads((root / "Cargo.toml").read_text())
    for registry, patches in manifest.get("patch", {}).items():
        for name, patch in patches.items():
            rule = approved.get(name)
            if registry != "crates-io" or rule is None or patch != {"path": rule["path"]}:
                failures.append(f"unratified Cargo patch: {registry}/{name}")
    if manifest.get("replace"):
        failures.append("unratified Cargo replacement table")
    source_root = root / "third_party"
    if source_root.exists():
        allowed_paths = {rule["path"] for rule in approved.values()}
        for path in source_root.iterdir():
            if str(path.relative_to(root)) not in allowed_paths:
                failures.append(f"unratified third-party source path: {path.relative_to(root)}")
    for name, rule in approved.items():
        if not (root / rule["decision"]).is_file():
            failures.append(f"missing numbered patch authority for {name}")
        path = root / rule["path"]
        if not path.exists():
            continue  # Authority may precede source introduction.
        try:
            package = tomllib.loads((path / "Cargo.toml").read_text())["package"]
            if package["name"] != name or package["version"] != rule["version"]:
                failures.append(f"unratified patched package identity: {name}")
            if rule["license"] != "MIT" or "MIT" not in package["license"].split(" OR "):
                failures.append(f"patched package lacks the approved MIT option: {name}")
            actual = hashlib.sha256((path / "LICENSE-MIT").read_bytes()).hexdigest()
            if actual != rule["license_sha256"]:
                failures.append(f"modified or missing approved MIT notice: {name}")
        except (OSError, KeyError, tomllib.TOMLDecodeError) as error:
            failures.append(f"invalid approved source package {name}: {error}")
    return failures


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
    failures.extend(check_source_patches(root, policy))
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
