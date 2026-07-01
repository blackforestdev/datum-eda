#!/usr/bin/env python3
"""Enforce that every specification steering Datum is woven into the roadmap.

Implements CLAUDE.md -> "Specification Governance (controlling)": a spec that
exists in the repo but is neither tracked in specs/PROGRESS.md nor locked by a
drift gate is an "orphaned spec" (governance defect). This gate makes orphans
impossible to introduce silently:

- Every .md file under the manifest's enforced_globs, plus every tracked_doc,
  MUST have an entry in specs/spec_governance_manifest.json. An unclassified
  spec file fails the gate.
- 'governed' entries must be tracked: a progress_anchor string present in
  PROGRESS.md, or a 'gate' that actually runs in run_drift_gates.sh, or the
  single-source tracker itself.
- 'pending' entries (known orphans) must carry a remediation note, so they are
  visible and owned rather than silently untracked.
- 'historical' entries must carry a reason. 'doctrine' entries must be numbered
  decision records.

The gate is green against the current repo by construction; it only fails on new
or reclassified drift.
"""

from __future__ import annotations

import glob
import json
import pathlib
import re
import sys

ROOT = pathlib.Path(__file__).resolve().parents[1]
MANIFEST = ROOT / "specs/spec_governance_manifest.json"
PROGRESS = ROOT / "specs/PROGRESS.md"
DRIFT_RUNNER = ROOT / "scripts/run_drift_gates.sh"

VALID_CLASSES = {"governed", "doctrine", "pending", "historical"}


def real_gates() -> set[str]:
    """Gate script basenames actually invoked by the drift-gate runner."""
    text = DRIFT_RUNNER.read_text(encoding="utf-8")
    return set(re.findall(r"scripts/(check_[a-z_]+)\.py", text))


def main() -> int:
    failures: list[str] = []

    manifest = json.loads(MANIFEST.read_text(encoding="utf-8"))
    entries: dict[str, dict] = manifest["entries"]
    enforced_globs: list[str] = manifest["enforced_globs"]
    tracked_docs: list[str] = manifest["tracked_docs"]
    progress_text = PROGRESS.read_text(encoding="utf-8")
    gates = real_gates()

    # 1. Every spec file on disk under the enforced surface must be classified.
    disk_files: set[str] = set(tracked_docs)
    for pattern in enforced_globs:
        for path in glob.glob(str(ROOT / pattern)):
            disk_files.add(str(pathlib.Path(path).relative_to(ROOT)))

    for rel in sorted(disk_files):
        if rel not in entries:
            failures.append(
                f"unclassified spec (orphan): {rel} — add an entry to "
                f"specs/spec_governance_manifest.json (governed/doctrine/pending/historical)"
            )

    # 2. No stale manifest entries pointing at missing files.
    for rel in sorted(entries):
        if not (ROOT / rel).exists():
            failures.append(f"manifest references missing file: {rel}")

    # 3. Per-class obligations.
    for rel, entry in sorted(entries.items()):
        if not (ROOT / rel).exists():
            continue  # already reported in (2)
        cls = entry.get("class")
        if cls not in VALID_CLASSES:
            failures.append(f"{rel}: invalid class {cls!r} (expected one of {sorted(VALID_CLASSES)})")
            continue

        if cls == "governed":
            tracked = False
            if entry.get("self_tracker"):
                tracked = True
            anchor = entry.get("progress_anchor")
            if anchor:
                if anchor in progress_text:
                    tracked = True
                else:
                    failures.append(
                        f"{rel}: governed entry declares progress_anchor {anchor!r} "
                        f"but it is absent from specs/PROGRESS.md"
                    )
            gate = entry.get("gate")
            if gate:
                if gate in gates:
                    tracked = True
                else:
                    failures.append(
                        f"{rel}: governed entry declares gate {gate!r} which is not "
                        f"invoked by scripts/run_drift_gates.sh"
                    )
            if not tracked:
                failures.append(
                    f"{rel}: governed spec is not tracked — needs a progress_anchor "
                    f"present in PROGRESS.md, a real drift gate, or self_tracker"
                )
        elif cls == "doctrine":
            if not rel.startswith("docs/decisions/"):
                failures.append(
                    f"{rel}: class 'doctrine' is reserved for numbered decision records "
                    f"under docs/decisions/"
                )
        elif cls == "pending":
            if not entry.get("remediation"):
                failures.append(
                    f"{rel}: class 'pending' (known orphan) must carry a non-empty "
                    f"'remediation' note so it stays visible and owned"
                )
        elif cls == "historical":
            if not entry.get("reason"):
                failures.append(
                    f"{rel}: class 'historical' must carry a non-empty 'reason'"
                )

    if failures:
        print("Spec governance gate failed:", file=sys.stderr)
        for failure in failures:
            print(f"- {failure}", file=sys.stderr)
        return 1

    counts = {c: 0 for c in VALID_CLASSES}
    for entry in entries.values():
        counts[entry.get("class", "?")] = counts.get(entry.get("class", "?"), 0) + 1
    print(
        "Spec governance gate passed "
        f"({len(entries)} specs classified: "
        f"{counts['governed']} governed, {counts['doctrine']} doctrine, "
        f"{counts['pending']} pending, {counts['historical']} historical)."
    )
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
