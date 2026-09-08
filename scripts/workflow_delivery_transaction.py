"""Capture changed production paths from an explicit transaction baseline.

The caller owns baseline selection and trust. This module never substitutes
HEAD for a missing baseline, chooses a task, or grants mutation permission.
"""

from workflow_delivery_coverage_shapes import paths
from workflow_delivery_shapes import require
from workflow_delivery_source_scopes import contains
from workflow_delivery_tree import Tree


def changed_production_paths(tree, roots, *, base_ref):
    paths(roots, "production_roots")
    tree.has_commit(base_ref)
    base = Tree(tree.root, revision=base_ref)
    if tree.staged or tree.revision:
        # Use captured index entries, not a fresh live-index diff. Repaired
        # worktree bytes or a later index change cannot replace this snapshot.
        names = set(base.entries) | set(tree.entries)
        return sorted(path for path in names if contains(roots, path)
                      and base.entries.get(path) != tree.entries.get(path))
    changed = tree.git("--literal-pathspecs", "diff", "--no-ext-diff",
                       "--no-textconv", "--no-renames", "--name-only", "-z",
                       base_ref, "--", *roots)
    # Ignored generated files can still be real build inputs. Deliberately do
    # not use --exclude-standard; the reviewed roots bound this enumeration.
    new = tree.git("--literal-pathspecs", "ls-files", "--others", "-z", "--", *roots)
    names = {raw.decode("utf-8") for raw in (changed + new).split(b"\0") if raw}
    require(all(contains(roots, path) for path in names), "transaction",
            "Git returned a path outside literal production roots", "WDQ-COVERAGE")
    return sorted(names)
