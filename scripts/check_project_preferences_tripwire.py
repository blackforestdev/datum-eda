#!/usr/bin/env python3
"""Tripwire: Project Preferences may be drawn, but not built without a specification.

The Project Preferences surface is drawn in
``docs/gui/prototypes/project-preferences-revision-gate.html`` with its category
column derived in ``docs/gui/prototypes/project-preferences-category-study.html``.
Neither is a specification, and deliberately so: the owner's direction is that a
single unspecified menu surface must not halt development.

The risk that creates is that the gap becomes an easter-egg hunt — a fact buried
in prototype review notes that a future agent only finds by already looking at
those files. This gate removes that risk without demanding the spec early. It
stays silent while Project Preferences is only a drawing, and fails the moment
any code begins implementing it, at which point ``specs/PROJECT_PREFERENCES_SPEC.md``
must exist and be classified in ``specs/spec_governance_manifest.json``.

The governance manifest cannot itself hold the placeholder: check_spec_governance
refuses an entry whose file does not exist, so a pending row is impossible for an
unwritten spec. Hence a gate rather than a manifest entry.
"""

from __future__ import annotations

import json
import re
import sys
from pathlib import Path

ROOT = Path(__file__).resolve().parents[1]
SPEC = Path("specs/PROJECT_PREFERENCES_SPEC.md")
MANIFEST = Path("specs/spec_governance_manifest.json")
STUDY = Path("docs/gui/prototypes/project-preferences-category-study.html")

# Implementation signals. Prose, prototypes, and this gate itself are not code.
CODE_ROOTS = ("crates", "mcp-server")
SIGNAL = re.compile(r"project[_-]?preferences", re.IGNORECASE)
SKIP_SUFFIXES = {".lock", ".png", ".json"}


def implementation_sites() -> list[str]:
    """Return code paths that reference a Project Preferences surface."""
    hits: list[str] = []
    for root in CODE_ROOTS:
        base = ROOT / root
        if not base.is_dir():
            continue
        for path in sorted(base.rglob("*")):
            if not path.is_file() or path.suffix in SKIP_SUFFIXES:
                continue
            if "target" in path.parts:
                continue
            try:
                text = path.read_text(encoding="utf-8")
            except (UnicodeDecodeError, OSError):
                continue
            if SIGNAL.search(text):
                hits.append(str(path.relative_to(ROOT)))
    return hits


def main() -> int:
    failures: list[str] = []
    sites = implementation_sites()

    if sites:
        if not (ROOT / SPEC).is_file():
            failures.append(
                f"Project Preferences implementation has begun in "
                f"{', '.join(sites[:5])}"
                + (f" (+{len(sites) - 5} more)" if len(sites) > 5 else "")
                + f" but {SPEC} does not exist. The surface may be drawn without a "
                f"specification; it may not be built without one. Draft the spec from "
                f"{STUDY}, classify it in {MANIFEST}, and place it in the Active Frontier."
            )
        else:
            manifest = json.loads((ROOT / MANIFEST).read_text(encoding="utf-8"))
            if str(SPEC) not in manifest.get("entries", {}):
                failures.append(
                    f"{SPEC} exists but is not classified in {MANIFEST}."
                )

    if not (ROOT / STUDY).is_file():
        failures.append(
            f"{STUDY} is missing; it is the derivation this tripwire points a future "
            f"author at. Restore it or update this gate."
        )

    if failures:
        print("Project Preferences tripwire failed:", file=sys.stderr)
        for failure in failures:
            print(f"- {failure}", file=sys.stderr)
        return 1

    state = "armed (no implementation yet)" if not sites else "satisfied"
    print(f"Project Preferences tripwire {state}: drawing without a spec is allowed.")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
