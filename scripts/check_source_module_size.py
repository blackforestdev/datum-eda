#!/usr/bin/env python3
"""Source-module size governance gate (governance-triggered decomposition flag).

Datum's decomposition doctrine is **governance-triggered and organic**, never
scheduled or proactive (docs/DECOMPOSITION_PLAN.md / the Decomposition
Philosophy). This gate is the *governance trigger*: it walks tracked source
modules, and any module over the threshold FAILS the gate **unless** it is
registered in ``specs/source_module_size_manifest.json`` as an oversized module
with status ``decomposition-pending`` and an accurate recorded line count.

Consequences:

- **New oversized modules fail immediately** — you cannot silently add another
  monolith; you must either keep it under threshold or make its size a visible,
  budgeted governance decision by registering it.
- **Known-oversized modules are FLAGGED, not silently tolerated** — every
  registered module is printed as a decomposition-pending flag on each run.
- **The ledger cannot rot** — if a registered module's live line count drifts
  materially from its recorded count, the gate fails until the manifest is
  updated (so the recorded budget stays honest).
- **Shrink-back is surfaced** — a registered module that drops back under
  threshold is reported as a de-list candidate (warning, not failure).
- **Registered monoliths cannot grow without bound** — each entry carries a
  FROZEN ``ceiling_lines`` (its registered baseline + headroom). A flagged module
  may grow only up to its ceiling; past it the gate FAILS. The ceiling does not
  move: the only sanctioned way back under it is to DECOMPOSE (which shrinks the
  file), never to raise the ceiling. This is the perpetuation guard — the ledger
  stays honest AND a known monolith has a hard cap on further growth.

This gate does NOT decompose anything. Registration here is the flag that a
future governance-triggered decomposition change should act on; the split itself
is deliberately deferred — but the ceiling bounds how far a flagged module may
drift before decomposition is forced.
"""

from __future__ import annotations

import json
import math
import subprocess
import sys
from pathlib import Path

ROOT = Path(__file__).resolve().parents[1]
MANIFEST = ROOT / "specs/source_module_size_manifest.json"


def is_tracked_source(rel: str) -> bool:
    """crates/**/src/**.rs, top-level scripts/*.py, mcp-server/**.py."""
    if rel.startswith("crates/") and "/src/" in rel and rel.endswith(".rs"):
        return True
    if rel.startswith("scripts/") and rel.endswith(".py") and rel.count("/") == 1:
        return True
    if rel.startswith("mcp-server/") and rel.endswith(".py"):
        return True
    return False


def tracked_sources() -> list[str]:
    out = subprocess.check_output(
        ["git", "ls-files", "crates", "scripts", "mcp-server"],
        cwd=ROOT,
        text=True,
    )
    return sorted(p for p in out.splitlines() if is_tracked_source(p))


def line_count(rel: str) -> int:
    # Count newline bytes (matches `wc -l`), stable across encodings.
    return (ROOT / rel).read_bytes().count(b"\n")


def material_drift(recorded: int, floor: int, pct: float) -> int:
    return max(floor, math.ceil(pct * recorded))


def main() -> int:
    manifest = json.loads(MANIFEST.read_text(encoding="utf-8"))
    threshold: int = manifest["threshold_lines"]
    floor: int = manifest.get("drift_floor_lines", 40)
    pct: float = manifest.get("drift_pct", 0.03)
    headroom_pct: float = manifest.get("growth_headroom_pct", 0.15)
    headroom_floor: int = manifest.get("growth_headroom_floor", 250)
    valid_status: set[str] = set(manifest.get("valid_status", ["decomposition-pending"]))
    entries: dict[str, dict] = manifest["entries"]

    failures: list[str] = []
    warnings: list[str] = []
    flags: list[str] = []

    live = {rel: line_count(rel) for rel in tracked_sources()}
    oversized = {rel: n for rel, n in live.items() if n > threshold}

    # 1. Every oversized module must be registered, with an honest recorded count.
    for rel, count in sorted(oversized.items()):
        entry = entries.get(rel)
        if entry is None:
            suggested_ceiling = count + max(headroom_floor, math.ceil(headroom_pct * count))
            failures.append(
                f"NEW oversized module: {rel} is {count} lines (> {threshold}) and is "
                f"not registered. Either keep it under {threshold} lines, or register it "
                f"in {MANIFEST.relative_to(ROOT)} as a decomposition-pending module with an "
                f"accurate recorded_lines, a frozen ceiling_lines (suggested "
                f"{suggested_ceiling} = baseline + headroom), and a note "
                f"(governance-triggered decomposition)."
            )
            continue
        status = entry.get("status")
        if status not in valid_status:
            failures.append(
                f"{rel}: invalid status {status!r} (expected one of {sorted(valid_status)})."
            )
        recorded = entry.get("recorded_lines")
        if not isinstance(recorded, int):
            failures.append(f"{rel}: manifest entry missing integer recorded_lines.")
            continue
        ceiling = entry.get("ceiling_lines")
        if not isinstance(ceiling, int):
            failures.append(
                f"{rel}: manifest entry missing integer ceiling_lines — the FROZEN "
                f"decomposition-headroom cap (baseline + headroom = the larger of "
                f"{headroom_floor} lines or {int(headroom_pct * 100)}% of the baseline)."
            )
            continue
        if ceiling < recorded:
            failures.append(
                f"{rel}: ceiling_lines {ceiling} < recorded_lines {recorded}; the ceiling "
                f"must be >= the recorded baseline."
            )
        entry_ok = True
        tol = material_drift(recorded, floor, pct)
        if abs(count - recorded) > tol:
            failures.append(
                f"LEDGER DRIFT: {rel} is now {count} lines but the manifest records "
                f"{recorded} (drift {count - recorded:+d} > tolerance ±{tol}). Update "
                f"recorded_lines in {MANIFEST.relative_to(ROOT)} so the budget stays honest "
                f"(a large shrink usually means a decomposition landed — de-list instead)."
            )
            entry_ok = False
        if count > ceiling:
            failures.append(
                f"OVER HEADROOM: {rel} is {count} lines, past its frozen decomposition "
                f"ceiling {ceiling} (baseline {recorded}). A decomposition-pending module may "
                f"grow only up to its ceiling; the ONLY sanctioned way back under it is to "
                f"DECOMPOSE (which shrinks the file), never to raise the ceiling. Split it now."
            )
            entry_ok = False
        if entry_ok:
            flags.append(
                f"FLAG decomposition-pending: {rel} — {count} lines "
                f"(recorded {recorded}, ceiling {ceiling}, threshold {threshold})"
            )

    # 2. No stale / shrunk registrations.
    for rel, entry in sorted(entries.items()):
        if rel not in live:
            failures.append(
                f"STALE registration: {rel} is registered oversized but is not a tracked "
                f"source module (moved/deleted?). Remove it from {MANIFEST.relative_to(ROOT)}."
            )
            continue
        if live[rel] <= threshold:
            warnings.append(
                f"DE-LIST CANDIDATE: {rel} is now {live[rel]} lines (<= {threshold}); it "
                f"dropped back under threshold — remove it from the manifest."
            )

    for flag in flags:
        print(f"SOURCE-SIZE {flag}")
    for warning in warnings:
        print(f"SOURCE-SIZE WARN {warning}")

    if failures:
        print("Source-module size gate FAILED:", file=sys.stderr)
        for failure in failures:
            print(f"- {failure}", file=sys.stderr)
        return 1

    print(
        f"Source-module size gate passed: {len(oversized)} oversized module(s) flagged "
        f"decomposition-pending, {len(warnings)} de-list candidate(s); "
        f"no new oversized modules, no ledger drift, none over its decomposition ceiling "
        f"(threshold {threshold} lines)."
    )
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
