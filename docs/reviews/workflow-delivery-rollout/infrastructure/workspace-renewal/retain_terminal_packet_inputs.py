"""Retain compact producer inputs in Git; never publish main or install trust."""

import hashlib
import json
import os
from pathlib import Path
import subprocess
import sys
import tarfile
import tempfile


def main():
    assert sys.flags.isolated and sys.flags.no_site and sys.flags.dont_write_bytecode
    root = Path(__file__).resolve().parents[5]
    proposals = root / ".git/datum-wdq/proposals"
    store = proposals / "terminal-rollout-evidence-20260910"
    assessment = proposals / "terminal-final-assessment-01"
    destination = proposals / "terminal-packet-retention-01"
    assert not destination.exists()
    assembled = json.loads((assessment / "observations/dispatch-observations.json").read_bytes())
    assert len(assembled["observations"]) == 1400
    files = set()
    def add(path):
        path = Path(path)
        assert path.is_file() and not path.is_symlink() and path.resolve() == path
        assert path.is_relative_to(proposals)
        files.add(path)
    for row in assembled["observations"]:
        capture = Path(row["capture_path"])
        for path in capture.iterdir():
            assert path.is_file(), path
            add(path)
        prestate = capture / "prestate-archive.json"
        if prestate.exists():
            record = json.loads(prestate.read_bytes())
            path = Path(record["path"])
            assert hashlib.sha256(path.read_bytes()).hexdigest() == record["sha256"]
            add(path)
        sibling = capture.parent / "assessment.json"
        if sibling.is_file():
            add(sibling)
    # Retain deterministic input packets, not duplicate prepared working repos.
    # Never descend into an embedded Git repository while finding recipe packets.
    for current, directories, names in os.walk(store):
        if ".git" in directories or ".git" in names:
            directories[:] = []
            continue
        current = Path(current)
        if "recipe.json" in names and "fixture.bundle" in names:
            for name in ("recipe.json", "fixture.bundle", "prepared-state.json"):
                add(current / name)
        for name in names:
            if name.endswith(".json") and (current == store or current.parent == store or current.name == "standard-series"):
                add(current / name)
            elif current == store and name.endswith(".bin"):
                add(current / name)
    for path in assessment.rglob("*"):
        if path.is_file():
            add(path)
    inventory = [{"path": p.relative_to(proposals).as_posix(), "size": p.stat().st_size,
                  "sha256": hashlib.sha256(p.read_bytes()).hexdigest()} for p in sorted(files)]
    destination.mkdir()
    archive = destination / "raw.tar.xz"
    with tarfile.open(archive, "w:xz") as output:
        for row in inventory:
            output.add(proposals / row["path"], arcname=row["path"], recursive=False)
    with tarfile.open(archive, "r:xz") as retained:
        assert retained.getnames() == [row["path"] for row in inventory]
        for row in inventory:
            member = retained.getmember(row["path"])
            assert member.isfile() and member.size == row["size"]
            with retained.extractfile(member) as stream:
                assert hashlib.sha256(stream.read()).hexdigest() == row["sha256"]
    prefix = "docs/reviews/workflow-delivery-rollout/infrastructure/terminal-renewal/"
    metadata = {"observed_source": assembled["observed_source_commit"], "assessment_source": assembled["source_commit"],
        "observations": 1400, "inventory": inventory,
        "archive": {"path": prefix + "raw.tar.xz", "sha256": hashlib.sha256(archive.read_bytes()).hexdigest()},
        "terminal_supplement_ref": "refs/datum-wdq/evidence/terminal-owner-79fe4cd84174b8b31c224fe74346c23ba7ee48d5",
        "scope": "Producer raw observations, deterministic packets and workspace prestates; terminal supplement bundles/failed attempts remain in its separate immutable ref. Not independent completion or installed proof."}
    metadata_raw = (json.dumps(metadata, indent=2) + "\n").encode()
    def git(*args, data=None, env=None):
        return subprocess.check_output(["git", "--no-replace-objects", "--no-optional-locks", *args],
                                       cwd=root, input=data, env=env)
    base = git("rev-parse", "HEAD").decode().strip()
    assert not git("status", "--porcelain"), "commit owned driver first"
    with tempfile.TemporaryDirectory(prefix="terminal-input-index-", dir=proposals) as directory:
        env = dict(os.environ, GIT_INDEX_FILE=str(Path(directory) / "index"))
        git("read-tree", "--empty", env=env)
        for path, raw in ((prefix + "raw.tar.xz", archive.read_bytes()), (prefix + "inventory.json", metadata_raw)):
            oid = git("hash-object", "-w", "--stdin", data=raw).decode().strip()
            git("update-index", "--add", "--cacheinfo", "100644", oid, path, env=env)
        tree = git("write-tree", env=env).decode().strip()
    message = ("test(workflow): retain compact terminal renewal observations\n\n"
        "Problem: Exact packet inputs need durable custody without duplicating working fixtures in main.\n"
        "Change: Archive 1400 producer observations, deterministic packets and exact workspace prestates; preserve historical source identities.\n"
        "Proof: Every tar member size and SHA-256 was read back and matched the inventory; no independent completion asserted.\n"
        "Roadmap: WDQ-I04, dat-wdq-rollout-implementation-ffy; evidence retention only, no publication, product, dependency or licensing change.\n")
    commit = git("commit-tree", tree, data=message.encode()).decode().strip()
    ref = "refs/datum-wdq/evidence/terminal-renewal-" + commit
    git("update-ref", ref, commit, "0" * 40)
    assert git("rev-parse", "HEAD").decode().strip() == base and not git("status", "--porcelain")
    result = {"commit": commit, "ref": ref, "files": len(files), "archive_bytes": archive.stat().st_size,
              "archive_sha256": metadata["archive"]["sha256"], "activation_performed": False}
    with (destination / "retention.json").open("x") as stream:
        json.dump(result, stream, indent=2)
    print(json.dumps(result))


if __name__ == "__main__":
    main()
