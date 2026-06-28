#!/usr/bin/env python3
"""Cross-language drift gate for the daemon non-journaled write fence.

The public AI-write bypass (decision 004, Private Mutation Ban) is closed by a
hand-maintained Python allowlist, `NON_JOURNALED_DAEMON_WRITE_METHODS` in
`mcp-server/tools_catalog_data.py`, which mirrors the non-journaled write arms
in the unfenced Rust daemon dispatcher `crates/engine-daemon/src/dispatch.rs`.
Nothing keeps the two in sync, so a new bypassing daemon write arm could be
added without the catalog hiding it from the public tools list.

This gate reconstructs the daemon's non-journaled write surface from source and
fails on any disagreement with the Python allowlist:

  * a daemon write arm missing from the allowlist  -> an unfenced public bypass
  * an allowlist entry with no matching daemon arm  -> a stale fence entry

Daemon write arms are derived structurally, not hand-listed: the authoritative
universe of legacy non-journaled mutators is every `pub fn` declared under
`crates/engine/src/api/write_ops/` (excluding the journaled undo/redo module),
and a daemon write arm is any dispatch arm that calls one of those mutators as
`engine.<mutator>(`.
"""

from __future__ import annotations

from pathlib import Path
import re
import sys


ROOT = Path(__file__).resolve().parents[1]
DISPATCH = ROOT / "crates/engine-daemon/src/dispatch.rs"
WRITE_OPS_DIR = ROOT / "crates/engine/src/api/write_ops"
CATALOG = ROOT / "mcp-server/tools_catalog_data.py"

def read_text(path: Path) -> str:
    return path.read_text(encoding="utf-8")


def write_ops_mutators() -> set[str]:
    """Every legacy non-journaled mutator pub fn under api/write_ops/*.rs.

    The journaled undo/redo module is excluded; it is a session journal control
    surface, not a direct board mutator that bypasses the journal.
    """
    mutators: set[str] = set()
    for path in sorted(WRITE_OPS_DIR.glob("*.rs")):
        if path.stem == "undo_redo":
            continue
        for match in re.finditer(r"^\s*pub fn ([a-z_][a-z0-9_]*)", read_text(path), flags=re.M):
            mutators.add(match.group(1))
    return mutators


def daemon_write_arms() -> set[str]:
    """Dispatch arms that call a non-journaled write_ops mutator as engine.<m>(."""
    dispatch_text = read_text(DISPATCH)
    dispatch_arms = set(re.findall(r'^\s*"([a-z_]+)"\s*=>', dispatch_text, flags=re.M))
    arms: set[str] = set()
    for mutator in write_ops_mutators():
        if mutator not in dispatch_arms:
            continue
        if re.search(r"\bengine\." + re.escape(mutator) + r"\s*\(", dispatch_text):
            arms.add(mutator)
    return arms


def python_allowlist() -> set[str]:
    text = read_text(CATALOG)
    match = re.search(
        r"NON_JOURNALED_DAEMON_WRITE_METHODS:\s*frozenset\[str\]\s*=\s*frozenset\(\{(.*?)\}\)",
        text,
        flags=re.S,
    )
    if not match:
        raise RuntimeError(
            "unable to locate NON_JOURNALED_DAEMON_WRITE_METHODS in "
            f"{CATALOG.relative_to(ROOT)}"
        )
    return set(re.findall(r'"([a-z_]+)"', match.group(1)))


def check() -> int:
    daemon_arms = daemon_write_arms()
    allowlist = python_allowlist()
    mutators = write_ops_mutators()

    expected_fenced = daemon_arms
    failures: list[str] = []

    for arm in sorted(expected_fenced - allowlist):
        failures.append(
            f"daemon write arm `{arm}` is not in the Python fence allowlist "
            "NON_JOURNALED_DAEMON_WRITE_METHODS: an unfenced public write bypass."
        )
    for arm in sorted(allowlist - expected_fenced):
        if arm not in mutators:
            failures.append(
                f"allowlist entry `{arm}` is not a known api/write_ops mutator: "
                "stale or fabricated fence entry."
            )
        else:
            failures.append(
                f"allowlist entry `{arm}` has no matching non-journaled daemon "
                "dispatch arm: stale fence entry."
            )

    if failures:
        print("Daemon write-parity drift gate failed:", file=sys.stderr)
        for failure in failures:
            print(f"  - {failure}", file=sys.stderr)
        print(
            "Keep NON_JOURNALED_DAEMON_WRITE_METHODS "
            f"({CATALOG.relative_to(ROOT)}) in lockstep with the non-journaled "
            f"write arms in {DISPATCH.relative_to(ROOT)}; the journaled datum.pcb.* "
            "family is the only public board-write surface.",
            file=sys.stderr,
        )
        return 1

    print(
        "Daemon write-parity drift gate passed "
        f"({len(allowlist)} fenced arms)."
    )
    return 0


if __name__ == "__main__":
    raise SystemExit(check())
