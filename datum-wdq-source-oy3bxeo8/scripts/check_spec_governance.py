#!/usr/bin/env python3
"""Enforce that every specification steering Datum is classified in the roadmap.

Implements CLAUDE.md -> "Specification Governance (controlling)" in its lean
posture: coverage + classification only. Behavioral enforcement lives in the
proof gates (scripts/run_migration_proof_gates.sh) and the write-fence gates
(private-writer / daemon-write-parity / MCP taxonomy), not in per-doc ledger
prose.

Obligations checked here:

1. Coverage: every .md under the manifest's enforced_globs, plus every
   tracked_doc and authority_doc, plus any doc whose header self-declares
   active, MUST have an entry in specs/spec_governance_manifest.json. An
   unclassified spec fails (orphan). Every entries key must exist on disk.

2. Classification: every entry carries a valid class
   (governed / doctrine / pending / historical). A doc classified
   'historical' must not self-declare active in its header.
"""

from __future__ import annotations

import glob
import json
import pathlib
import re
import sys

# A doc that declares itself active in its header steers development and MUST be
# classified. Matches "Status: active", "> **Status**: Active", etc.
ACTIVE_HEADER = re.compile(r"status.{0,12}active", re.IGNORECASE)
ACTIVE_SWEEP_GLOBS = ["docs/**/*.md", "*.md"]

ROOT = pathlib.Path(__file__).resolve().parents[1]
MANIFEST = ROOT / "specs/spec_governance_manifest.json"

VALID_CLASSES = {"governed", "doctrine", "pending", "historical"}


def main() -> int:
    failures: list[str] = []

    manifest = json.loads(MANIFEST.read_text(encoding="utf-8"))
    entries: dict[str, dict] = manifest["entries"]
    enforced_globs: list[str] = manifest["enforced_globs"]
    tracked_docs: list[str] = manifest["tracked_docs"]

    # 1. Every spec file on disk under the enforced surface must be classified.
    #    Surface = hard globs (specs/, docs/contracts/, docs/decisions/) + the
    #    explicit tracked_docs + named authority_docs + any doc that declares
    #    itself active in its header (auto-discovered, so new active docs cannot
    #    slip in unclassified).
    disk_files: set[str] = set(tracked_docs) | set(manifest.get("authority_docs", []))
    for pattern in enforced_globs:
        for path in glob.glob(str(ROOT / pattern)):
            disk_files.add(str(pathlib.Path(path).relative_to(ROOT)))
    active_sweep: set[str] = set()
    for pattern in ACTIVE_SWEEP_GLOBS:
        for path in glob.glob(str(ROOT / pattern), recursive=True):
            p = pathlib.Path(path)
            try:
                header = "".join(p.read_text(encoding="utf-8").splitlines(keepends=True)[:8])
            except (OSError, UnicodeDecodeError):
                continue
            if ACTIVE_HEADER.search(header):
                active_sweep.add(str(p.relative_to(ROOT)))
    disk_files |= active_sweep
    for rel in sorted(disk_files):
        if rel not in entries:
            hint = " (declares itself active in its header)" if rel in active_sweep else ""
            failures.append(
                f"unclassified spec (orphan): {rel}{hint} — add an entry to "
                f"specs/spec_governance_manifest.json (governed/doctrine/pending/historical)"
            )

    # 2. No stale manifest entries.
    for rel in sorted(entries):
        if not (ROOT / rel).exists():
            failures.append(f"manifest references missing file: {rel}")

    # 3. Classification validity.
    for rel, entry in sorted(entries.items()):
        if not (ROOT / rel).exists():
            continue
        cls = entry.get("class")
        if cls not in VALID_CLASSES:
            failures.append(f"{rel}: invalid class {cls!r} (expected {sorted(VALID_CLASSES)})")
            continue

        if cls == "historical":
            # A historical doc must not contradict its status by self-declaring active.
            try:
                header = "".join((ROOT / rel).read_text(encoding="utf-8").splitlines(keepends=True)[:8])
                if ACTIVE_HEADER.search(header):
                    failures.append(
                        f"{rel}: classified 'historical' but its header still self-declares active — "
                        f"reconcile the doc's Status line with its governed status"
                    )
            except (OSError, UnicodeDecodeError):
                pass

    if failures:
        print("Spec governance gate failed:", file=sys.stderr)
        for failure in failures:
            print(f"- {failure}", file=sys.stderr)
        return 1

    counts: dict[str, int] = {c: 0 for c in VALID_CLASSES}
    for entry in entries.values():
        counts[entry.get("class", "?")] = counts.get(entry.get("class", "?"), 0) + 1
    print(
        f"Spec governance gate passed ({len(entries)} specs classified: "
        f"{counts['governed']} governed, {counts['doctrine']} doctrine, "
        f"{counts['pending']} pending, {counts['historical']} historical)."
    )
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
