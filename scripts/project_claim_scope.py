"""Normalize claim paths and worktree identities before comparing ownership.

Bare scopes are worktree-relative files/directories; subsystem:<name> is an
opaque, case-sensitive reservation, compared globally by exact name. A subsystem
never substitutes for reserving paths it edits. Nonexistent future paths remain
valid; existing symlinks and parent segments resolve to their actual identities.
"""
from pathlib import Path
import re


def normalized_scopes(root: Path, claim: dict) -> list[tuple[str, str]]:
    worktree = Path(claim['worktree'])
    if not worktree.is_absolute():
        worktree = root / worktree
    worktree = worktree.resolve()
    result = []
    for scope in claim['scope']:
        if scope.startswith('subsystem:'):
            name = scope.removeprefix('subsystem:')
            if not re.fullmatch(r'[A-Za-z0-9][A-Za-z0-9_.-]*', name):
                raise ValueError('subsystem scope needs one explicit nonempty name')
            result.append(('subsystem', name))
        else:
            path = Path(scope)
            if path.is_absolute() or ':' in scope or not scope.strip():
                raise ValueError('path scopes must be relative paths, or subsystem:<name>')
            path = (worktree / path).resolve()
            if not path.is_relative_to(worktree):
                raise ValueError('path scope escapes its worktree')
            result.append(('path', str(path)))
    return result


def claim_scope_failures(root: Path, claims: list[tuple[str, dict]]) -> list[str]:
    failures = []
    normalized = []
    for key, claim in claims:
        if (not isinstance(claim.get('worktree'), str) or not claim['worktree'].strip()
                or not isinstance(claim.get('scope'), list)
                or any(not isinstance(s, str) for s in claim['scope'])):
            continue  # The base lease validator owns missing/malformed fields.
        try:
            normalized.append((key, normalized_scopes(root, claim)))
        except (ValueError, OSError, RuntimeError) as exc:
            failures.append(f'{key}: invalid claim scope identity: {exc}')
    for i, (left_key, left) in enumerate(normalized):
        for right_key, right in normalized[i + 1:]:
            overlap = []
            for lk, lv in left:
                for rk, rv in right:
                    if lk == rk and (lv == rv or lk == 'path' and
                                     (Path(lv).is_relative_to(rv) or Path(rv).is_relative_to(lv))):
                        overlap.append((lv, rv))
            if overlap:
                failures.append(f'overlapping active claims: {left_key} and {right_key} share {overlap}')
    return failures
