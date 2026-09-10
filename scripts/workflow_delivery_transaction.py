"""Capture changed production paths from an explicit transaction baseline.

The caller owns baseline selection and trust. This module never substitutes
HEAD for a missing baseline, chooses a task, or grants mutation permission.
"""

import os
from pathlib import Path
import tempfile

from workflow_delivery_coverage_shapes import paths
from workflow_delivery_shapes import require
from workflow_delivery_source_scopes import contains
from workflow_delivery_tree import Tree


def committed_production_paths(tree, roots, base_ref):
    # Promotion inspects a commit sequence, not only its final net tree diff:
    # add-then-delete and edit-then-revert still represent proposed mutations.
    common = tree.git("merge-base", base_ref, tree.revision).decode().strip()
    require(common == base_ref, "transaction",
            "candidate must descend from the explicit transaction base", "WDQ-COVERAGE")
    revisions = tree.git("rev-list", base_ref + ".." + tree.revision).decode().splitlines()
    names = set()
    for revision in revisions:
        changed = tree.git("--literal-pathspecs", "diff-tree", "--root", "-m", "-r",
                           "--no-commit-id", "--no-renames", "--name-only", "-z",
                           revision, "--", *roots)
        names.update(raw.decode("utf-8") for raw in changed.split(b"\0") if raw)
    require(all(contains(roots, path) for path in names), "transaction",
            "Git returned a path outside literal production roots", "WDQ-COVERAGE")
    return names


def worktree_diff(tree, roots, base_ref):
    # Git diff may refresh stat-cache bytes even with --no-optional-locks.
    # Keep normal content comparison, but allow refresh only in our copy.
    index = Path(tree.index_path) if tree.index_path else Path(tree.git(
        "rev-parse", "--path-format=absolute", "--git-path", "index").decode().strip())
    data = index.read_bytes() if index.exists() else None
    descriptor, name = tempfile.mkstemp(prefix=".datum-wdq-index-", dir=index.parent)
    private = Path(name)
    try:
        with os.fdopen(descriptor, "wb") as stream:
            if data is not None:
                stream.write(data)
        if data is None:
            private.unlink()
        # Same directory resolves existing split-index links; disable splitting
        # for this command so refresh cannot write shared-index state.
        return tree.git("-c", "core.splitIndex=false", "--literal-pathspecs", "diff",
                        "--no-ext-diff", "--no-textconv", "--no-renames", "--name-only",
                        "-z", base_ref, "--", *roots, index_path=name)
    finally:
        private.unlink(missing_ok=True)
        Path(name + ".lock").unlink(missing_ok=True)


def changed_production_paths(tree, roots, *, base_ref, workspace=None):
    paths(roots, "production_roots")
    tree.has_commit(base_ref)
    base = Tree(tree.root, revision=base_ref)
    if tree.staged or tree.revision:
        # Use captured index entries, not a fresh live-index diff. Repaired
        # worktree bytes or a later index change cannot replace this snapshot.
        names = set(base.entries) | set(tree.entries)
        changed = {path for path in names if contains(roots, path)
                   and base.entries.get(path) != tree.entries.get(path)}
        if tree.revision:
            changed.update(committed_production_paths(tree, roots, base_ref))
        return sorted(changed)
    changed = worktree_diff(tree, roots, base_ref)
    # Ignored generated files can still be real build inputs. Deliberately do
    # not use --exclude-standard; the reviewed roots bound this enumeration.
    new = tree.git("--literal-pathspecs", "ls-files", "--others", "-z", "--", *roots,
                   index_path=tree.index_path)
    names = {raw.decode("utf-8") for raw in (changed + new).split(b"\0") if raw}
    require(all(contains(roots, path) for path in names), "transaction",
            "Git returned a path outside literal production roots", "WDQ-COVERAGE")
    if workspace is not None:
        names = workspace.input_paths(names, root=tree.root,
                                      tracked_paths=set(base.entries) | set(tree.entries),
                                      explicit_paths=roots)
    return sorted(names)
