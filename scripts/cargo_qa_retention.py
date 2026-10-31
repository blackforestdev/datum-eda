"""Explicit retirement of released, rebuildable QA debug outputs only.

Source snapshots and evidence are never removal candidates. This is not a
scheduler: release annotation plus an explicit invocation are both required.
"""
from contextlib import ExitStack
import fnmatch
import json
import os
from pathlib import Path, PurePosixPath
import stat
import subprocess

import cargo_qa_companions as q
import run_cargo_guarded as guard

BINARIES = {"datum-gui", "datum-eda", "eda-engine-daemon"}


class Failure(guard.ResourcePolicyError):
    def __init__(self, message, receipt):
        super().__init__(message)
        self.receipt = receipt


def refuse(message):
    raise guard.ResourcePolicyError(f"released QA build refused: {message}")


class Bound:
    """Hold the parent chain and leaf; writable ELF admission prevents exec."""
    def __init__(self, root, relative, stack, *, executable=False):
        self.root, self.relative = root, relative
        self.parts = PurePosixPath(relative).parts
        self.parent, self.chain = q.directory(root, self.parts[:-1], stack)
        self.name = self.parts[-1]
        before = os.stat(self.name, dir_fd=self.parent, follow_symlinks=False)
        if not stat.S_ISREG(before.st_mode) or before.st_uid != os.getuid():
            refuse("selected input is not an owned regular file")
        self.fd = os.open(self.name, q.WRITE if executable else q.READ, dir_fd=self.parent)
        stack.callback(os.close, self.fd)
        if q.info(os.fstat(self.fd)) != q.info(before):
            refuse("inode changed during admission")
        self.expected = q.snapshot(self.fd)
        if executable:
            h = os.pread(self.fd, 18, 0)
            if (not before.st_mode & 0o111 or len(h) != 18 or h[:4] != b"\x7fELF"
                    or h[4] not in (1, 2) or h[5] not in (1, 2)
                    or int.from_bytes(h[16:18], "little" if h[5] == 1 else "big") not in (2, 3)):
                refuse("selected binary is not an executable ELF")
        elif relative.endswith(".d") and before.st_mode & 0o111:
            refuse("dependency metadata must not be executable")
        self.verify(content=False)

    def verify(self, *, named=True, content=True):
        with ExitStack() as check:
            _, chain = q.directory(self.root, self.parts[:-1], check)
            if chain != self.chain:
                refuse("root or parent identity changed")
        if content:
            actual, expected = q.snapshot(self.fd), self.expected
        else:
            actual = q.info(os.fstat(self.fd))
            actual["xattrs_sha256"] = q.security(self.fd)
            if q.info(os.fstat(self.fd)) != {key: actual[key] for key in q.FIELDS}:
                refuse("held metadata changed during inspection")
            expected = {key: self.expected[key] for key in (*q.FIELDS, "xattrs_sha256")}
        if actual != expected:
            refuse("held input changed")
        if named and q.info(os.stat(self.name, dir_fd=self.parent, follow_symlinks=False)) != q.info(os.fstat(self.fd)):
            refuse("named input changed")

    def post_unlink(self):
        # Hash before accepting ctime: a concurrent writer must not be hidden
        # behind the timestamp change legitimately caused by unlink.
        actual = q.snapshot(self.fd)
        expected = dict(self.expected, nlink=self.expected["nlink"] - 1,
                        ctime_ns=actual["ctime_ns"])
        if actual != expected:
            refuse("unexpected bytes or metadata after unlink")
        return actual

    def accept_unlink(self, verified):
        expected = dict(self.expected, nlink=self.expected["nlink"] - 1,
                        ctime_ns=verified["ctime_ns"])
        if verified != expected:
            refuse("inconsistent shared-inode unlink result")
        self.expected = verified.copy()
        self.verify(named=False, content=False)

    def json(self):
        return json.loads(os.pread(self.fd, self.expected["size"], 0))


def read_json(root, path):
    with ExitStack() as stack:
        value = Bound(root, path, stack)
        result = value.json()
        value.verify()
        return result


def ownership(root, issue, bundles):
    # Read current semantic state, not a frozen whole-tracker digest: unrelated
    # peers may export their rows while a released output is being retired.
    with ExitStack() as stack:
        tracker = Bound(root, ".beads/issues.jsonl", stack)
        rows = [json.loads(row) for row in os.pread(tracker.fd, tracker.expected["size"], 0).splitlines() if row.strip()]
        tracker.verify()
    if not all(isinstance(row, dict) for row in rows):
        refuse("tracker rows must be objects")
    matches = [row for row in rows if row.get("id") == issue]
    if len(matches) != 1 or matches[0].get("status") != "closed":
        refuse("release issue must exist uniquely and be closed")
    frontier = read_json(root, "specs/active_frontier.json")
    if not isinstance(frontier, dict) or not isinstance(frontier.get("frontier"), list):
        refuse("unknown Frontier layout")
    for row in frontier["frontier"]:
        if not isinstance(row, dict):
            refuse("Frontier rows must be objects")
        claim = row.get("claim")
        if claim is not None and not isinstance(claim, dict):
            refuse("Frontier claims must be objects")
        if claim is None:
            continue
        if row.get("issue_id") == issue or claim.get("issue_id") == issue:
            refuse("release issue still has a claim")
        scopes = claim.get("scope")
        if not isinstance(scopes, list) or not all(isinstance(p, str) for p in scopes):
            refuse("uninspectable claim scope")
        for scope in scopes:
            p = Path(scope)
            if p.is_absolute():
                try:
                    scope = str(p.relative_to(root))
                except ValueError:
                    continue
            scope = scope.rstrip("/")
            for bundle in bundles:
                selected = bundle + "/debug"
                if (scope in ("", ".") or selected == scope or selected.startswith(scope + "/")
                        or scope.startswith(selected + "/") or fnmatch.fnmatchcase(selected, scope)
                        or any(fnmatch.fnmatchcase(selected + "/" + name, scope)
                               for name in BINARIES | {n + ".d" for n in BINARIES})):
                    refuse("selected output still has a Frontier claim")


def retire(root, relatives, lock_path, active_check, users_check, *, apply=False):
    root = Path(root).resolve()
    receipt = {"operation": "released_qa_build", "apply": apply, "status": "preflight",
               "completed": [], "selected": [], "retained": [], "observed_absent": [],
               "runtime_proof": "held ELF write descriptors; no unrelated-app idle assertion"}
    bound = []
    try:
        bundles = []
        for relative in relatives:
            parts = PurePosixPath(relative).parts
            if (len(parts) != 3 or parts[:2] != ("target", "qa")
                    or relative != str(PurePosixPath(relative)) or parts[2] in (".", "..")):
                refuse("expected exact target/qa/<bundle> relative path")
            if relative in bundles:
                refuse("duplicate bundle selection")
            bundles.append(relative)
        if not bundles:
            refuse("no selected bundle")
        with guard.acquire_lock(lock_path, 0), ExitStack() as stack:
            def idle():
                if active_check():
                    refuse("Cargo/rustc must be idle under the shared lock")

            idle()
            records, directories = [], []
            for bundle in bundles:
                metadata = Bound(root, bundle + "/build-info.json", stack)
                info = metadata.json()
                if not isinstance(info, dict):
                    refuse("build-info must be an object")
                release = info.get("build_retention", {})
                if not isinstance(release, dict):
                    refuse("build_retention must be an object")
                issue = release.get("issue_id")
                if release.get("state") != "released" or not isinstance(issue, str) or not issue:
                    refuse("build-info requires a positive released issue annotation")
                ownership(root, issue, [bundle])
                records.append((metadata, issue, bundle))
                debug, chain = q.directory(root, PurePosixPath(bundle + "/debug").parts, stack)
                names = sorted(os.listdir(debug))
                directories.append((bundle, debug, chain, set(names)))
                allowed = BINARIES | {name + ".d" for name in BINARIES}
                if set(names) - allowed:
                    refuse("unknown debug output; preserve it and classify before retirement")
                files, binaries = info.get("files_sha256"), info.get("binary_sha256")
                if any(v is not None and not isinstance(v, dict) for v in (files, binaries)):
                    refuse("declared hash maps must be objects")
                files, binaries = files or {}, binaries or {}
                for name in names:
                    relative = bundle + "/debug/" + name
                    expected = files.get("debug/" + name)
                    if name.endswith(".d") and expected is None:
                        # Unpinned dependency metadata is retained, never inferred
                        # disposable from a neighboring binary's hash.
                        value = Bound(root, relative, stack)
                        receipt["retained"].append(relative)
                        continue
                    value = Bound(root, relative, stack, executable=name in BINARIES)
                    if expected is None and name in BINARIES:
                        expected = binaries.get(name)
                    if not isinstance(expected, str) or expected != value.expected["sha256"]:
                        refuse("debug output lacks a matching declared SHA256")
                    bound.append(value)
            paths = [value.relative for value in bound]
            receipt["selected"] = paths
            tracked = subprocess.run(["git", "-c", "core.fsmonitor=false", "-c", "core.hooksPath=/dev/null",
                "ls-files", "-z", "--", *bundles], cwd=root, check=True, capture_output=True).stdout
            if tracked:
                refuse("selected QA bundle contains tracked files")
            if users_check([root / p for p in paths]):
                refuse("selected output is mapped by an active process")

            def validate():
                idle()
                for metadata, issue, bundle in records:
                    metadata.verify()
                    ownership(root, issue, [bundle])
                for bundle, debug, expected_chain, names in directories:
                    with ExitStack() as check:
                        _, chain = q.directory(root, PurePosixPath(bundle + "/debug").parts, check)
                        if chain != expected_chain:
                            refuse("debug directory identity changed")
                    removed = {PurePosixPath(p).name for p in receipt["completed"]
                               if str(PurePosixPath(p).parent) == bundle + "/debug"}
                    if set(os.listdir(debug)) != names - removed:
                        refuse("debug directory membership changed")
                for value in bound:
                    if value.relative not in receipt["completed"]:
                        value.verify(content=False)

            validate()  # All bundles must pass before the first unlink.
            if apply:
                for value in bound:
                    validate()
                    value.verify()
                    os.unlink(value.name, dir_fd=value.parent)
                    receipt["completed"].append(value.relative)
                    verified = value.post_unlink()
                    # All held names sharing this inode use the verified result.
                    for alias in bound:
                        if (alias.expected["device"], alias.expected["inode"]) == (value.expected["device"], value.expected["inode"]):
                            alias.accept_unlink(verified)
                validate()
                receipt["status"] = "complete_source_and_evidence_retained"
            else:
                receipt["status"] = "validated_dry_run"
    except (OSError, ValueError, TypeError, KeyError, subprocess.SubprocessError, guard.ResourcePolicyError) as exc:
        receipt["status"] = "partial_failure" if receipt["completed"] else "refused"
        receipt["reason"] = str(exc)
        raise Failure(str(exc), receipt) from exc
    finally:
        # No claim that a merely attempted unlink succeeded. Observe each name,
        # including syscall-success/receipt-interruption cases, without mutation.
        for value in bound:
            try:
                (root / value.relative).lstat()
            except FileNotFoundError:
                receipt["observed_absent"].append(value.relative)
            except OSError:
                receipt.setdefault("observation_unknown", []).append(value.relative)
    return receipt
