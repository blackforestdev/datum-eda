#!/usr/bin/env python3
"""Enforce research/prototype-to-spec traceability and review freshness.

Every Markdown artifact under research/ and every HTML visual source under
docs/gui/prototypes/ must belong to exactly one evidence route.  A route names
its governed consumers and records one digest over both sides.  Changing either
an evidence source or a consuming specification therefore forces an explicit
route review in the same change.
"""

from __future__ import annotations

import argparse
import hashlib
import json
import pathlib
import sys


ROOT = pathlib.Path(__file__).resolve().parents[1]
MANIFEST = ROOT / "specs/evidence_traceability_manifest.json"
GOVERNANCE = ROOT / "specs/spec_governance_manifest.json"
VALID_STATUSES = {"active", "in_progress", "integrated", "historical"}


def digest(paths: list[str]) -> str:
    value = hashlib.sha256()
    for rel in sorted(paths):
        value.update(rel.encode("utf-8"))
        value.update(b"\0")
        value.update((ROOT / rel).read_bytes())
        value.update(b"\0")
    return value.hexdigest()


def discovered_sources() -> set[str]:
    research = {
        str(path.relative_to(ROOT))
        for path in (ROOT / "research").rglob("*.md")
        if path.is_file()
    }
    prototypes = {
        str(path.relative_to(ROOT))
        for path in (ROOT / "docs/gui/prototypes").glob("*.html")
        if path.is_file()
    }
    return research | prototypes


def main() -> int:
    parser = argparse.ArgumentParser()
    parser.add_argument(
        "--print-digests",
        action="store_true",
        help="print current route digests without accepting them",
    )
    args = parser.parse_args()

    manifest = json.loads(MANIFEST.read_text(encoding="utf-8"))
    governance = json.loads(GOVERNANCE.read_text(encoding="utf-8"))
    governed = set(governance["entries"])
    routes = manifest.get("routes", [])
    failures: list[str] = []
    owners: dict[str, list[str]] = {}
    route_ids: set[str] = set()

    for route in routes:
        route_id = route.get("id", "")
        if not route_id or route_id in route_ids:
            failures.append(f"invalid or duplicate route id: {route_id!r}")
            continue
        route_ids.add(route_id)

        sources = route.get("sources", [])
        consumers = route.get("consumers", [])
        if not sources:
            failures.append(f"{route_id}: route has no evidence sources")
        if not consumers:
            failures.append(f"{route_id}: route has no governed consumers")
        if route.get("status") not in VALID_STATUSES:
            failures.append(
                f"{route_id}: invalid status {route.get('status')!r}; "
                f"expected {sorted(VALID_STATUSES)}"
            )
        if not str(route.get("review_note", "")).strip():
            failures.append(f"{route_id}: review_note must explain the integration boundary")

        for rel in sources:
            owners.setdefault(rel, []).append(route_id)
            if not (ROOT / rel).is_file():
                failures.append(f"{route_id}: missing evidence source: {rel}")
        for rel in consumers:
            if not (ROOT / rel).is_file():
                failures.append(f"{route_id}: missing consumer: {rel}")
            elif rel not in governed:
                failures.append(
                    f"{route_id}: consumer is not classified by spec governance: {rel}"
                )

        existing = [rel for rel in sources + consumers if (ROOT / rel).is_file()]
        current = digest(existing) if len(existing) == len(sources) + len(consumers) else ""
        if args.print_digests:
            print(f"{route_id} {current}")
        elif route.get("reviewed_digest") != current:
            failures.append(
                f"{route_id}: evidence/spec review is stale; set reviewed_digest to {current} "
                "only after reviewing every listed source against every listed consumer"
            )

    discovered = discovered_sources()
    listed = set(owners)
    for rel in sorted(discovered - listed):
        failures.append(f"untracked research/prototype evidence: {rel}")
    for rel in sorted(listed - discovered):
        failures.append(f"manifest source is outside the discovered evidence surface: {rel}")
    for rel, route_owners in sorted(owners.items()):
        if len(route_owners) != 1:
            failures.append(
                f"{rel}: must have exactly one owning evidence route; found {route_owners}"
            )

    if args.print_digests:
        return 1 if failures else 0
    if failures:
        print("Evidence traceability gate failed:", file=sys.stderr)
        for failure in failures:
            print(f"- {failure}", file=sys.stderr)
        return 1

    print(
        f"Evidence traceability gate passed ({len(routes)} routes, "
        f"{len(discovered)} research/prototype artifacts)."
    )
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
