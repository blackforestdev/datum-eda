"""Shared source pattern matching for canonical writer boundary checks."""
from pathlib import Path


def matching_lines(path: Path, patterns: list[str]) -> list[tuple[int, str, str]]:
    matches: list[tuple[int, str, str]] = []
    for index, line in enumerate(path.read_text().splitlines(), start=1):
        stripped = line.strip()
        for pattern in patterns:
            if pattern in stripped:
                matches.append((index, pattern, stripped))
    return matches


def count_pattern(path: Path, pattern: str) -> int:
    return path.read_text().count(pattern)


def require_patterns(
    failures: list[str], relative: Path, text: str, patterns: list[str], label: str
) -> None:
    for pattern in patterns:
        if pattern not in text:
            failures.append(f"{relative}: missing expected {label} pattern `{pattern}`")
