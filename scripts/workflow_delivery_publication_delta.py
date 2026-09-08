"""Describe exact publication history without granting permission to publish.

This is a preparation input for owner promotion, not ordinary source-scope
authorization, independent review, an owner receipt, or an activation command.
"""

from workflow_delivery_bootstrap import git, pinned_commit
from workflow_delivery_io import canonical_json, normalized_path, parse_json, sha256
from workflow_delivery_shapes import closed, digest, require


def tree_entries(root, revision):
    entries = {}
    for row in git(root, "ls-tree", "-rz", revision).split(b"\0"):
        if row:
            metadata, path = row.split(b"\t", 1)
            mode, kind, oid = metadata.decode().split()
            entries[path.decode()] = {"mode": mode, "kind": kind, "oid": oid}
    return entries


def publication_delta(root, *, base, candidate):
    """Retain all parent edges, including merge sides and net-zero mutations."""
    pinned_commit(root, base)
    pinned_commit(root, candidate)
    if base == candidate:
        raise ValueError("publication base must differ from candidate; empty comparison is not publication proof")
    if git(root, "merge-base", base, candidate).decode().strip() != base:
        raise ValueError("publication candidate must descend from the exact base")
    revisions = git(root, "rev-list", "--reverse", "--topo-order", base + ".." + candidate).decode().splitlines()
    trees, records, touched = {}, [], set()
    def entries(revision):
        if revision not in trees:
            trees[revision] = tree_entries(root, revision)
        return trees[revision]
    for revision in revisions:
        raw = git(root, "cat-file", "commit", revision)
        header = raw.split(b"\n\n", 1)[0].splitlines()
        parents = [line[7:].decode() for line in header if line.startswith(b"parent ")]
        current = entries(revision)
        edges = []
        for parent in parents or [None]:
            before = entries(parent) if parent else {}
            changes = [{"path": path, "before": before.get(path), "after": current.get(path)}
                       for path in sorted(set(before) | set(current)) if before.get(path) != current.get(path)]
            touched.update(row["path"] for row in changes)
            edges.append({"parent": parent, "changes": changes})
        records.append({"commit": revision, "commit_sha256": sha256(raw),
                        "tree": git(root, "rev-parse", revision + "^{tree}").decode().strip(),
                        "parents": parents, "edges": edges})
    original, final = entries(base), entries(candidate)
    return {"schema_version": 1, "kind": "datum.workflow-delivery.publication-delta",
            "base": base, "candidate": candidate, "commits": records,
            "touched_paths": sorted(touched),
            "net_changes": [{"path": path, "before": original.get(path), "after": final.get(path)}
                            for path in sorted(set(original) | set(final))
                            if original.get(path) != final.get(path)],
            "publication_authorized": False, "activation_asserted": False,
            "scope": "original Git commit history and tree entries; not permission, review or acceptance"}


def verify_publication_delta(root, value, *, base, candidate):
    """Rebuild from externally supplied exact pins; never trust artifact pins."""
    expected = publication_delta(root, base=base, candidate=candidate)
    if canonical_json(value) != canonical_json(expected):
        raise ValueError("publication delta differs from exact pinned Git history")
    return expected


def verify_publication_review(delta, review):
    """Bind a caller-pinned review inventory, not an owner receipt or permission.

    The caller must supply a delta rebuilt from its explicit Git pins. Exact
    file names cover every parent edge, not only net changes or directory roots.
    This does not waive ordinary execution claims or prove independent review.
    """
    closed(review, "delta_sha256 paths", "publication review")
    digest(review["delta_sha256"], "publication review")
    paths = review["paths"]
    if type(paths) is not list or any(type(path) is not str for path in paths):
        raise ValueError("publication review requires an explicit file-path list")
    for path in paths:
        normalized_path(path)
    if paths != sorted(set(paths)):
        raise ValueError("publication review paths must be sorted and unique")
    if paths != delta["touched_paths"]:
        raise ValueError("publication review must enumerate every touched file exactly; no prefix grants")
    if review["delta_sha256"] != sha256(canonical_json(delta)):
        raise ValueError("publication review digest differs from exact pinned Git history")
    return {"delta_sha256": review["delta_sha256"], "paths": list(paths)}


def inspect_activation_response(raw, *, response_sha256, request_sha256):
    """Compare separately selected response bytes; never authenticate their author.

    Uses WDQ-I04's recorded response vocabulary with the exact request digest as
    packet identity. Candidate content cannot select this external response hash.
    Real owner selection, independent evidence and activation remain separate.
    """
    digest(response_sha256, "owner response digest")
    digest(request_sha256, "activation request digest")
    require(type(raw) is bytes and sha256(raw) == response_sha256, "owner response",
            "response bytes differ from externally selected digest", "WDQ-RECEIPT")
    value = parse_json(raw, "owner response")
    closed(value, "schema_version kind response source recorded_at", "owner response", "WDQ-RECEIPT")
    require(type(value["schema_version"]) is int and value["schema_version"] == 1
            and value["kind"] == "datum.workflow-delivery.activation-response", "owner response",
            "activation response version 1 required", "WDQ-RECEIPT")
    expected = "WORKFLOW-DELIVERY-IMPLEMENTATION: approve ACTIVATE — " + request_sha256
    require(value["response"] == expected, "owner response",
            "response does not approve this exact activation request", "WDQ-RECEIPT")
    require(all(type(value[key]) is str and value[key].strip() for key in ("source", "recorded_at")),
            "owner response", "response source and recorded date required", "WDQ-RECEIPT")
    return {"response_sha256": response_sha256, "request_sha256": request_sha256,
            "response_matches_request": True, "owner_identity_verified": False,
            "publication_authorized": False, "activation_asserted": False,
            "scope": "Exact separately selected response bytes only; not independent proof or activation authority."}
