"""Exercise repaired path detection against the exact retained failure fixture."""

import json
from pathlib import Path
import shutil
import sys


def main():
    root = Path(__file__).resolve().parents[5]
    retained = root / ".git/datum-wdq/proposals"
    runtime = retained / "index-preservation-repair-20260910"
    store = retained / "wdq-full-candidate-inspection-20260909"
    source = store / "renewed-s01-0e5b8064/INFRA-S01-04.refused-S-check-primary"
    destination = runtime / ".git/index-repair-original-fixture"
    assert not destination.exists()
    original = json.loads((source / "capture/before.json").read_bytes())
    shutil.copytree(source / "fixture", destination, symlinks=True)
    sys.path.insert(0, str(runtime / "scripts"))
    from workflow_delivery_capture_state import protected_state
    from workflow_delivery_transaction import changed_production_paths
    from workflow_delivery_tree import Tree
    before = protected_state(destination, original["untracked_roots"])
    for key in ("files", "head", "symbolic_head", "index", "index_entries_hex", "refs", "git_info_exclude"):
        assert before[key] == original[key], key
    paths = changed_production_paths(Tree(destination), ["src"], base_ref=original["head"])
    after = protected_state(destination, original["untracked_roots"])
    assert paths == ["src/unscoped.py"], paths
    assert before == after
    assert not list((destination / ".git").glob(".datum-wdq-index-*"))
    print(json.dumps({"source_fixture": str(source), "diagnostic_fixture": str(destination),
        "changed_paths": paths, "protected_state_unchanged": True,
        "index_sha256": before["index"]["sha256"], "temporary_indexes_remaining": 0,
        "scope": "Direct repaired-function execution on exact copied failure inputs, not full selector replay or fresh typed proof."}))


if __name__ == "__main__":
    main()
