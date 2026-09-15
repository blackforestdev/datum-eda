#!/usr/bin/env python3
"""Guard resolver authority by limiting raw native-project root loads."""

from pathlib import Path
import sys

ROOT = Path(__file__).resolve().parents[1]
CLI_SRC = ROOT / "crates" / "cli" / "src"
PATTERN = "load_native_project(root)?"

ALLOWLIST = {}


def rel(path: Path) -> Path:
    return path.relative_to(ROOT)


def main() -> int:
    failures: list[str] = []
    observed: dict[Path, list[int]] = {}

    for path in sorted(CLI_SRC.rglob("*.rs")):
        text = path.read_text()
        lines = [
            index
            for index, line in enumerate(text.splitlines(), start=1)
            if PATTERN in line
        ]
        if not lines:
            continue
        relative = rel(path)
        observed[relative] = lines
        if relative not in ALLOWLIST:
            failures.append(
                f"{relative}: raw {PATTERN} is not allowlisted (lines {lines})"
            )

    for path, policy in ALLOWLIST.items():
        full_path = ROOT / path
        text = full_path.read_text() if full_path.exists() else ""
        lines = observed.get(path, [])
        expected_count = policy["count"]
        if len(lines) != expected_count:
            failures.append(
                f"{path}: expected {expected_count} allowlisted {PATTERN} call(s), found {len(lines)}"
            )
        for needle in policy["required"]:
            if needle not in text:
                failures.append(
                    f"{path}: allowlist rationale no longer matches file; missing marker {needle!r}"
                )

    if failures:
        print("Resolver raw-load guard failed:", file=sys.stderr)
        for failure in failures:
            print(f"  - {failure}", file=sys.stderr)
        print("\nAllowed exceptions:", file=sys.stderr)
        for path, policy in ALLOWLIST.items():
            print(f"  - {path}: {policy['reason']}", file=sys.stderr)
        return 1

    total = sum(len(lines) for lines in observed.values())
    print(f"Resolver raw-load guard passed ({total} allowlisted call sites).")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
