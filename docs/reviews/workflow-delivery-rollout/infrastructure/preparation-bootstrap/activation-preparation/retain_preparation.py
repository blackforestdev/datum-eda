"""Commit exact pre-activation artifacts without moving main or candidate refs."""

import hashlib
import json
import os
from pathlib import Path
import subprocess
import tempfile


ROOT = Path("/home/bfadmin/Documents/datum-eda")
SOURCE = Path(__file__).resolve().parent
BASE = "354ef94969b878fe8ad978833880755e750ff7e8"
PREFIX = (
    "docs/reviews/workflow-delivery-rollout/infrastructure/"
    "preparation-bootstrap/activation-preparation/"
)


def git(*arguments, data=None, env=None):
    return subprocess.check_output(
        ["git", "--no-replace-objects", "--no-optional-locks", *arguments],
        cwd=ROOT, input=data, env=env,
    )


def main():
    assert git("rev-parse", "HEAD").decode().strip() == BASE
    assert not git("status", "--porcelain=v1", "--untracked-files=all")
    names = [
        "activate.py", "activation-request.json", "packet.json", "prepare.py",
        "retain_preparation.py",
    ]
    files = {name: (SOURCE / name).read_bytes() for name in names}
    inventory = [
        {"path": name, "bytes": len(raw), "sha256": hashlib.sha256(raw).hexdigest()}
        for name, raw in sorted(files.items())
    ]
    files["inventory.json"] = (
        json.dumps(inventory, indent=2, sort_keys=True) + "\n"
    ).encode()
    with tempfile.TemporaryDirectory(prefix="retain-", dir=SOURCE) as temporary:
        environment = dict(os.environ, GIT_INDEX_FILE=str(Path(temporary) / "index"))
        git("read-tree", BASE, env=environment)
        for name, raw in sorted(files.items()):
            oid = git("hash-object", "-w", "--stdin", data=raw).decode().strip()
            git("update-index", "--add", "--cacheinfo", "100644", oid, PREFIX + name,
                env=environment)
        tree = git("write-tree", env=environment).decode().strip()
    message = """test(workflow): retain reviewed preparation activation packet

Problem:
The reviewed PM042 preparation-bootstrap candidate requires an exact owner response before installation. Its activation request, wrapper and preparation basis must remain recoverable without moving main or altering the independently reviewed candidate.

Change:
Retain the exact request, packet, preparation and activation scripts plus a hashed inventory on a dedicated evidence ref rooted at the activation base.

Proof:
The request digest is 453f04ca94d5dfe9e3b86d2bf8b56c7acf597ba7572b15d182ef58a80182937b. Main is clean at the exact base, candidate 0f232e1026497d4eb404a503ee7c7e3fb7bb6561 remains unchanged, and no installation or local-trust mutation is performed.

Roadmap:
WORKFLOW-DELIVERY-IMPLEMENTATION/WDQ-I05, PM042, dat-wdq-preparation-bootstrap-rid. Owner activation and all product cohort evidence remain pending.
"""
    commit = git("commit-tree", tree, "-p", BASE, data=message.encode()).decode().strip()
    ref = "refs/datum-wdq/evidence/preparation-bootstrap-activation-" + commit
    git("update-ref", ref, commit, "0" * 40)
    for name, raw in files.items():
        assert git("show", commit + ":" + PREFIX + name) == raw
    print(json.dumps({"commit": commit, "ref": ref, "files": len(files)}, sort_keys=True))


if __name__ == "__main__":
    main()
