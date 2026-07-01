#!/usr/bin/env python3
"""Enforce that every specification steering Datum is woven into the roadmap AND
carries a verified enforcement declaration.

Implements CLAUDE.md -> "Specification Governance (controlling)". Two obligations:

1. Coverage: every .md under the manifest's enforced_globs, plus every
   tracked_doc, MUST be classified in specs/spec_governance_manifest.json
   (governed / doctrine / pending / historical). An unclassified spec fails.

2. Enforcement honesty: every non-historical entry MUST carry an `enforcement`
   block declaring how it is actually enforced -- a non-empty `enforced_by`
   list of VERIFIED references, and/or an explicit `gap` string naming what is
   NOT enforced. This gate verifies each enforced_by reference actually exists,
   so a spec cannot claim enforcement it does not have, and cannot sit
   un-enforced without the gap being visible.

Reference forms accepted in `enforced_by` (each is verified):
  - "check_NAME"        gate script scripts/check_NAME.py wired in run_drift_gates.sh
  - "ci:check_NAME"     gate script present + wired in .github/workflows/alignment.yml
  - "proof_gate:LABEL"  LABEL present in scripts/run_migration_proof_gates.sh
  - "spec_parity:INV"   inventory INV present in the spec-parity manifest/table
  - "tests:PATH"        path PATH exists in the repo

The gate is green against the current repo by construction; it fails only on new
drift or on an unverifiable / fabricated enforcement claim.
"""

from __future__ import annotations

import glob
import json
import pathlib
import sys

ROOT = pathlib.Path(__file__).resolve().parents[1]
MANIFEST = ROOT / "specs/spec_governance_manifest.json"
PROGRESS = ROOT / "specs/PROGRESS.md"
DRIFT_RUNNER = ROOT / "scripts/run_drift_gates.sh"
CI_WORKFLOW = ROOT / ".github/workflows/alignment.yml"
PROOF_GATES = ROOT / "scripts/run_migration_proof_gates.sh"
PARITY_MANIFEST = ROOT / "specs/spec_parity_manifest.json"
PARITY_TABLE = ROOT / "specs/SPEC_PARITY.md"

VALID_CLASSES = {"governed", "doctrine", "pending", "historical"}


def read(path: pathlib.Path) -> str:
    return path.read_text(encoding="utf-8") if path.exists() else ""


def main() -> int:
    failures: list[str] = []

    manifest = json.loads(MANIFEST.read_text(encoding="utf-8"))
    entries: dict[str, dict] = manifest["entries"]
    enforced_globs: list[str] = manifest["enforced_globs"]
    tracked_docs: list[str] = manifest["tracked_docs"]

    progress_text = read(PROGRESS)
    drift_text = read(DRIFT_RUNNER)
    ci_text = read(CI_WORKFLOW)
    proof_text = read(PROOF_GATES)
    parity_text = read(PARITY_MANIFEST) + read(PARITY_TABLE)

    def verify_ref(ref: str) -> bool:
        if ref.startswith("ci:check_"):
            name = ref[len("ci:"):]
            return (ROOT / f"scripts/{name}.py").exists() and f"{name}.py" in ci_text
        if ref.startswith("check_"):
            return (ROOT / f"scripts/{ref}.py").exists() and f"{ref}.py" in drift_text
        if ref.startswith("proof_gate:"):
            return ref.split(":", 1)[1] in proof_text
        if ref.startswith("spec_parity:"):
            return ref.split(":", 1)[1] in parity_text
        if ref.startswith("tests:"):
            return (ROOT / ref.split(":", 1)[1]).exists()
        return False

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

    # 2. No stale manifest entries.
    for rel in sorted(entries):
        if not (ROOT / rel).exists():
            failures.append(f"manifest references missing file: {rel}")

    # 3. Per-entry obligations.
    for rel, entry in sorted(entries.items()):
        if not (ROOT / rel).exists():
            continue
        cls = entry.get("class")
        if cls not in VALID_CLASSES:
            failures.append(f"{rel}: invalid class {cls!r} (expected {sorted(VALID_CLASSES)})")
            continue

        # 3a. Class-specific tracking obligations.
        if cls == "governed":
            tracked = bool(entry.get("self_tracker"))
            anchor = entry.get("progress_anchor")
            if anchor and anchor in progress_text:
                tracked = True
            if not tracked and not entry.get("enforcement", {}).get("enforced_by"):
                failures.append(
                    f"{rel}: governed spec is not tracked — needs a progress_anchor in "
                    f"PROGRESS.md, self_tracker, or a verified enforced_by reference"
                )
            if anchor and anchor not in progress_text:
                failures.append(
                    f"{rel}: governed entry declares progress_anchor {anchor!r} absent from PROGRESS.md"
                )
        elif cls == "doctrine":
            if not rel.startswith("docs/decisions/"):
                failures.append(f"{rel}: class 'doctrine' is reserved for docs/decisions/ records")
        elif cls == "pending":
            if not entry.get("remediation"):
                failures.append(f"{rel}: class 'pending' must carry a non-empty 'remediation' note")
        elif cls == "historical":
            if not entry.get("reason"):
                failures.append(f"{rel}: class 'historical' must carry a non-empty 'reason'")
            continue  # historical is exempt from the enforcement declaration

        # 3b. Enforcement declaration (required for governed/doctrine/pending).
        enforcement = entry.get("enforcement")
        if not isinstance(enforcement, dict):
            failures.append(
                f"{rel}: missing 'enforcement' block — declare enforced_by (verified refs) and/or a gap"
            )
            continue
        enforced_by = enforcement.get("enforced_by", [])
        gap = enforcement.get("gap", "")
        if not enforced_by and not gap:
            failures.append(
                f"{rel}: 'enforcement' must declare a non-empty enforced_by or an explicit gap"
            )
        for ref in enforced_by:
            if not verify_ref(ref):
                failures.append(
                    f"{rel}: enforced_by reference {ref!r} could not be verified "
                    f"(unknown form or the named gate/test/inventory does not exist)"
                )

    if failures:
        print("Spec governance gate failed:", file=sys.stderr)
        for failure in failures:
            print(f"- {failure}", file=sys.stderr)
        return 1

    counts: dict[str, int] = {c: 0 for c in VALID_CLASSES}
    gaps = 0
    for entry in entries.values():
        counts[entry.get("class", "?")] = counts.get(entry.get("class", "?"), 0) + 1
        if entry.get("enforcement", {}).get("gap"):
            gaps += 1
    print(
        f"Spec governance gate passed ({len(entries)} specs classified: "
        f"{counts['governed']} governed, {counts['doctrine']} doctrine, "
        f"{counts['pending']} pending, {counts['historical']} historical; "
        f"{gaps} carry a declared enforcement gap)."
    )
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
