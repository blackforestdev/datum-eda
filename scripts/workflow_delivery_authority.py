"""Resolve governed references and existing route digests without refreshing them."""

import hashlib

from workflow_delivery_io import canonical_json, sha256
from workflow_delivery_shapes import all_refs, require


def resolve_ref(tree, reference):
    path = reference["path"]
    entries = tree.json("specs/spec_governance_manifest.json")["entries"]
    require(path in entries, path, "reference must name governed evidence/authority",
            "WDQ-AUTHORITY")
    raw = tree.read(path)
    require(raw.count(reference["marker"].encode("utf-8")) == 1, path,
            f"marker must occur exactly once: {reference['marker']}", "WDQ-AUTHORITY")
    return raw


def authority_manifest(tree, contract, *, ready=True):
    manifest = tree.json("specs/evidence_traceability_manifest.json")
    routes = {r["id"]: r for r in manifest["routes"]}
    require(len(routes) == len(manifest["routes"]), "route_ids",
            "duplicate evidence route", "WDQ-AUTHORITY")
    selected = set(contract["route_ids"])
    references = list(all_refs(contract))
    paths = {r["path"] for r in references}
    for reference in references:
        resolve_ref(tree, reference)
        owners = {key for key, route in routes.items()
                  if reference["path"] in route["sources"] + route["consumers"]}
        require(owners <= selected, reference["path"],
                f"include owning routes {sorted(owners - selected)}", "WDQ-AUTHORITY")
    for key in sorted(selected):
        require(key in routes, key, "unknown owning route", "WDQ-AUTHORITY")
        route = routes[key]
        members = route["sources"] + route["consumers"]
        current = hashlib.sha256()
        for path in sorted(members):
            current.update(path.encode("utf-8") + b"\0" + tree.read(path) + b"\0")
        require(current.hexdigest() == route["reviewed_digest"], key,
                "owning route review is stale; return to owning lane", "WDQ-AUTHORITY")
        paths.update(members)
    require(not {contract["proof_path"], contract["review_path"]} & paths,
            "authority", "proof/review cannot belong to own authority closure", "WDQ-AUTHORITY")
    if ready:
        for decision in contract["open_decisions"]:
            require(decision["disposition_ref"] is not None, decision["id"],
                    "mandatory owner question unresolved", "WDQ-AUTHORITY")
    return [{"path": p, "sha256": sha256(tree.read(p))} for p in sorted(paths)]


def authority_sha256(tree, contract, *, ready=True):
    return sha256(canonical_json(authority_manifest(tree, contract, ready=ready)))
