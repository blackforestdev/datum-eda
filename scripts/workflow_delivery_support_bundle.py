"""Prepare/verify content-addressed runner bytes; never install local trust.

This runner-only bundle is not a complete activation packet or an owner hook.
Existing and partial destinations are retained and refused, never overwritten.
"""

from pathlib import Path
import stat

from workflow_delivery_io import canonical_json, sha256
from workflow_delivery_shapes import require
from workflow_delivery_support_store import support_locations
from workflow_delivery_tree import Tree
from workflow_delivery_trust import _gate_paths


def expected_bundle(root, authority):
    tree = Tree(root, revision=authority)
    sources = {}
    records = []
    for path in sorted(_gate_paths(tree)):
        if not path.startswith("scripts/") or not path.endswith(".py"):
            continue
        require(path == "scripts/" + Path(path).name, path,
                "flat runtime module path required", "WDQ-TRUST")
        require(path in tree.entries, path, "authority lacks required runtime module", "WDQ-TRUST")
        require(tree.entries[path][0] in ("100644", "100755"), path,
                "runtime source must be a regular Git blob", "WDQ-TRUST")
        raw = tree.read(path, committed=True)
        sources[path] = raw
        records.append({"path": path, "source_blob": tree.entries[path][1], "sha256": sha256(raw)})
    require("scripts/check_workflow_delivery.py" in sources, "bundle",
            "authority lacks runner entrypoint", "WDQ-TRUST")
    manifest = {"schema_version": 1, "kind": "datum.workflow-delivery.runner-bundle",
                "authority": authority, "files": records, "activation_asserted": False,
                "scope": "runner Python bytes only; not hook installation or complete toolchain"}
    return manifest, sources


def verify_bundle(root, authority):
    locations = support_locations(root, authority)
    directory = locations["trusted"]
    expected, sources = expected_bundle(root, authority)
    require(directory.is_dir(), str(directory), "runner bundle absent", "WDQ-TRUST")
    require({path.name for path in directory.iterdir()} == {"scripts", "manifest.json"},
            str(directory), "runner bundle incomplete or has unexpected entries", "WDQ-TRUST")
    actual_files = {path.name for path in locations["scripts"].iterdir()}
    require(actual_files == {Path(path).name for path in sources}, "scripts",
            "runtime module set differs; caches and unpinned imports are forbidden", "WDQ-TRUST")
    payloads = {**sources, "manifest.json": canonical_json(expected)}
    for name, raw in payloads.items():
        path = directory / name
        require(path.is_file() and not path.is_symlink(), str(path),
                "regular nonredirected bundle file required", "WDQ-TRUST")
        require(stat.S_IMODE(path.stat().st_mode) == 0o444, str(path),
                "runner bundle files must remain read-only", "WDQ-TRUST")
        require(path.read_bytes() == raw, str(path),
                "runner bytes differ from pinned Git authority", "WDQ-TRUST")
    return expected


def prepare_bundle(root, authority):
    locations = support_locations(root, authority)
    manifest, sources = expected_bundle(root, authority)
    # The full authority ID names a write-once destination. Never infer that an
    # existing partial directory is owned by this invocation or safe to repair.
    locations["trusted"].mkdir(parents=True, exist_ok=False)
    locations["scripts"].mkdir()
    for name, raw in {**sources, "manifest.json": canonical_json(manifest)}.items():
        path = locations["trusted"] / name
        with path.open("xb") as stream:
            stream.write(raw)
        path.chmod(0o444)
    return verify_bundle(root, authority)
