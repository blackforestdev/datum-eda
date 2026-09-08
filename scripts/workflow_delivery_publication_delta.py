"""Describe exact publication history without granting permission to publish.

This is a preparation input for owner promotion, not ordinary source-scope
authorization, independent review, an owner receipt, or an activation command.
"""

from workflow_delivery_bootstrap import git, pinned_commit
from workflow_delivery_io import canonical_json, sha256


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
