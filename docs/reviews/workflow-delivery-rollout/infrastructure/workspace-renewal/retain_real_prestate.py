"""Retain mutable real-fixture state beside its already retained source bundle."""

import hashlib
import os
from pathlib import Path
import re
import stat
import subprocess
import tarfile


def retain(root, output, command, *, packet, nonce):
    root, output, packet = map(Path, (root, output, packet))
    assert root.name == "fixture" and root.parent == output.parent
    marker = root / ".git/datum-wdq-capture-owner"
    assert not marker.is_symlink() and marker.read_text().strip() == nonce
    from workflow_delivery_capture_state import git
    index = root / ".git/index"
    original_index = index.read_bytes()
    # These commands disable optional locks; raw index equality below is an
    # additional check that retention did not refresh or repair it.
    git(root, "diff-files", "--quiet", "--no-ext-diff", "--")
    git(root, "diff-index", "--cached", "--quiet", "HEAD", "--")
    entries = git(root, "ls-files", "--stage", "-z")
    tracked, objects = set(), set()
    for entry in filter(None, entries.split(b"\0")):
        meta, path = entry.split(b"\t", 1)
        mode, oid, stage = meta.split()
        assert stage == b"0"
        actual = root / os.fsdecode(path)
        info = actual.lstat()
        if mode == b"120000":
            assert stat.S_ISLNK(info.st_mode), path
            payload = os.fsencode(os.readlink(actual))
        else:
            assert mode in (b"100644", b"100755") and stat.S_ISREG(info.st_mode), path
            assert bool(info.st_mode & 0o111) == (mode == b"100755"), path
            payload = actual.read_bytes()
        blob = b"blob " + str(len(payload)).encode() + b"\0" + payload
        assert hashlib.sha1(blob).hexdigest() == oid.decode(), path
        tracked.add(os.fsdecode(path))
        objects.add(oid.decode())
    head = git(root, "rev-parse", "HEAD").decode().strip()
    objects.add(head)
    refs = git(root, "for-each-ref", "--format=%(objectname)").decode().splitlines()
    objects.update(refs)
    for arg in command:
        if re.fullmatch(r"[0-9a-f]{40}", arg):
            objects.add(arg)
    # This restored repository was made solely from the pinned bundle. No
    # network fetch or fallback object source is permitted during verification.
    restored = packet / "snapshot"
    assert not (restored / ".git/objects/info/alternates").exists()
    result = subprocess.run(["git", "--no-replace-objects", "--no-optional-locks",
        "cat-file", "--batch-check"], cwd=restored,
        input=("\n".join(sorted(objects))+"\n").encode(),
        capture_output=True, check=True,
        env={"PATH":os.defpath,"LC_ALL":"C","GIT_CONFIG_NOSYSTEM":"1",
             "GIT_CONFIG_GLOBAL":os.devnull,"GIT_GRAFT_FILE":os.devnull})
    assert len(result.stdout.splitlines()) == len(objects)
    assert [line.split()[0].decode() for line in result.stdout.splitlines()] == sorted(objects)
    assert all(len(line.split()) == 3 and not line.endswith(b" missing")
               for line in result.stdout.splitlines())
    archive = output.with_name(output.name + "-prestate.tar.xz")
    assert not archive.exists()
    modes = {}
    def metadata_only(info):
        name = info.name.removeprefix("./")
        return None if name == ".git/objects" or name.startswith(".git/objects/") else info
    with tarfile.open(archive, "x:xz", dereference=False) as tar:
        tar.add(root / ".git", arcname=".git", filter=metadata_only)
        for directory, dirs, files in os.walk(root, followlinks=False):
            if Path(directory) == root:
                dirs.remove(".git")
            for name in dirs + files:
                path = Path(directory) / name
                relative = path.relative_to(root).as_posix()
                info = path.lstat()
                modes[relative] = stat.S_IMODE(info.st_mode)
                if relative not in tracked or stat.S_ISDIR(info.st_mode):
                    tar.add(path, arcname=relative, recursive=False)
    assert index.read_bytes() == original_index
    return {"archive":str(archive),"archive_sha256":hashlib.sha256(archive.read_bytes()).hexdigest(),
        "source_bundle":str(packet/"source.bundle"),"head":head,
        "bundle_object_checks":len(objects),
        "object_check_sha256":hashlib.sha256(result.stdout).hexdigest(),
        "tracked_bytes_and_index_match_head":True,"filesystem_modes":modes,
        "original_root":str(root),"fixture_nonce":nonce,
        "scope":"Restore exact bundle HEAD at the original owned pathname, overlay metadata and untracked payloads, restore listed modes without following symlinks, then require full captured before-state equality. Git objects and tracked regular-file bytes are not duplicated."}
