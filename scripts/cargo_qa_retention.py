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
import uuid

import cargo_qa_companions as q
import run_cargo_guarded as guard

BINARIES = {"datum-gui", "datum-eda", "eda-engine-daemon"}
PROOF_BINARIES = BINARIES | {"datum-gui-test"}


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


def proof_lifecycle(root, bundle, lock_path, active_check, users_check, **options):
    """Every refused lifecycle action returns an honest, machine-readable result."""
    try:
        return _proof_lifecycle(root, bundle, lock_path, active_check, users_check, **options)
    except Failure:
        raise
    except (OSError, ValueError, TypeError, KeyError, guard.ResourcePolicyError) as error:
        receipt = {"operation": "proof_lifecycle", "status": "refused", "bundle": bundle,
                   "reason": str(error), "completed": [], "resolution_recorded": False}
        try:
            parts = PurePosixPath(bundle).parts
            if len(parts) != 3 or parts[:2] != ("target", "qa") or ".." in parts or str(PurePosixPath(bundle)) != bundle:
                raise guard.ResourcePolicyError("invalid proof scope")
            recorded = read_json(Path(root).resolve(), bundle + "/build-info.json")["build_retention"]
            receipt["resolution_recorded"] = (recorded.get("state") == "released"
                and (recorded.get("owner"), recorded.get("session")) == (options.get("owner"), options.get("session"))
                and recorded.get("resolution") == {"result": options.get("result"), "provenance": options.get("provenance")})
        except (OSError, ValueError, TypeError, KeyError, guard.ResourcePolicyError):
            pass
        raise Failure(str(error), receipt) from error


def _proof_lifecycle(root, bundle, lock_path, active_check, users_check, *, owner,
                    session, issue=None, reason=None, pending=None, result=None, provenance=None):
    """Record a proof's owner; resolving it retires its outputs in this command."""
    root = Path(root).resolve()
    parts = PurePosixPath(bundle).parts
    if (len(parts) != 3 or parts[:2] != ("target", "qa")
            or str(PurePosixPath(bundle)) != bundle or ".." in parts):
        refuse("proof output must be an exact target/qa/BUNDLE")
    if not all(isinstance(v, str) and v.strip() for v in (owner, session)):
        refuse("proof requires an owner and session")
    resolving = result is not None
    required = (result, provenance) if resolving else (issue, reason, pending)
    if not all(isinstance(v, str) and v.strip() for v in required):
        refuse("retain requires task/reason/pending proof; resolve requires result/provenance")
    relative = bundle + "/build-info.json"
    with guard.acquire_lock(lock_path, 0), ExitStack() as stack:
        if active_check():
            refuse("Cargo/rustc must be idle before changing proof retention")
        tracked = subprocess.run(["git", "ls-files", "-z", "--", bundle],
                                 cwd=root, check=True, capture_output=True).stdout
        if tracked:
            refuse("proof lifecycle cannot rewrite tracked inputs")
        metadata = Bound(root, relative, stack)
        info = metadata.json()
        if not isinstance(info, dict) or not isinstance(info.get("build_retention", {}), dict):
            refuse("invalid proof build metadata")
        retention = info.get("build_retention", {})
        if retention and (retention.get("owner"), retention.get("session")) != (owner, session):
            refuse("proof belongs to a different owner/session or uses legacy retention")
        if retention.get("owner_hold"):
            refuse("proof has an explicit owner hold")
        if resolving:
            if retention.get("state") not in ("retained", "released"):
                refuse("only a registered proof can be resolved")
            rows = [row for row in read_json_lines(root, ".beads/issues.jsonl")
                    if row.get("id") == retention.get("issue_id")]
            if len(rows) != 1 or rows[0].get("assignee") != owner:
                refuse("proof task is no longer assigned to its recorded owner")
            if retention.get("state") == "released" and retention.get("resolution") != {
                    "result": result, "provenance": provenance}:
                refuse("resolved proof outcome is immutable")
            retention = dict(retention, state="released", resolution={
                "result": result, "provenance": provenance})
        else:
            if retention.get("state") == "released":
                refuse("resolved proof cannot be retained again")
            if retention and retention.get("issue_id") != issue:
                refuse("proof ownership cannot be transferred to another task")
            tracker = read_json_lines(root, ".beads/issues.jsonl")
            rows = [r for r in tracker if r.get("id") == issue]
            if len(rows) != 1 or rows[0].get("assignee") != owner:
                refuse("proof task must be uniquely assigned to its recorded owner")
            # Authenticate runnable inputs before recording a cleanup obligation.
            files = info.get("files_sha256", {})
            binaries = info.get("binary_sha256", {})
            if not isinstance(files, dict) or not isinstance(binaries, dict):
                refuse("invalid proof hash maps")
            present = False
            for name in PROOF_BINARIES:
                path = root / bundle / "debug" / name
                if not path.exists():
                    continue
                value = Bound(root, bundle + "/debug/" + name, stack, executable=True)
                if value.expected["sha256"] != files.get("debug/" + name, binaries.get(name)):
                    refuse("proof binary lacks a matching source receipt hash")
                present = True
            if not present:
                refuse("proof registration requires at least one authenticated binary")
            retention = {"state": "retained", "issue_id": issue, "owner": owner,
                         "session": session, "reason": reason, "pending_proof": pending}
        updated = dict(info, build_retention=retention)
        # Atomic replacement keeps outcome durable before attempting retirement.
        metadata.verify()
        temporary = ".build-retention-" + uuid.uuid4().hex
        fd = os.open(temporary, os.O_WRONLY | os.O_CREAT | os.O_EXCL | os.O_NOFOLLOW,
                     0o600, dir_fd=metadata.parent)
        try:
            with os.fdopen(fd, "w") as destination:
                json.dump(updated, destination, indent=2)
                destination.write("\n")
                destination.flush()
                os.fsync(destination.fileno())
            metadata.verify()
            os.replace(temporary, metadata.name, src_dir_fd=metadata.parent, dst_dir_fd=metadata.parent)
            os.fsync(metadata.parent)
        finally:
            try:
                os.unlink(temporary, dir_fd=metadata.parent)
            except FileNotFoundError:
                pass
    if not resolving:
        return {"status": "proof_retained", "bundle": bundle, "build_retention": retention}
    try:
        receipt = retire(root, [bundle], lock_path, active_check, users_check, apply=True,
                         retain_metadata=True, registered_proof=True)
    except Failure as error:
        receipt = error.receipt
        receipt["proof_resolution"] = retention["resolution"]
        receipt["cleanup_residue"] = [bundle + "/debug/" + name
            for name in sorted(os.listdir(root / bundle / "debug")) if not name.endswith(".d")]
        record_cleanup(root, bundle, receipt)
        raise
    receipt["proof_resolution"] = retention["resolution"]
    receipt["cleanup_residue"] = []
    record_cleanup(root, bundle, receipt)
    return receipt


def record_cleanup(root, bundle, receipt):
    """Persist the observed cleanup result beside the retained proof receipt."""
    with ExitStack() as stack:
        parent, _ = q.directory(root, PurePosixPath(bundle).parts, stack)
        # Each attempt remains immutable, including successful unlinks before a
        # later refusal. A retry must never erase the earlier retirement record.
        attempt = "proof-cleanup-" + uuid.uuid4().hex + ".json"
        receipt["attempt_record"] = attempt
        name = ".proof-cleanup-" + uuid.uuid4().hex
        fd = os.open(name, os.O_WRONLY | os.O_CREAT | os.O_EXCL | os.O_NOFOLLOW,
                     0o600, dir_fd=parent)
        try:
            with os.fdopen(fd, "w") as output:
                json.dump(receipt, output, indent=2)
                output.write("\n")
                output.flush()
                os.fsync(output.fileno())
            os.replace(name, attempt, src_dir_fd=parent, dst_dir_fd=parent)
            os.fsync(parent)
            # The stable summary is a pointer to this immutable attempt.
            summary = ".proof-summary-" + uuid.uuid4().hex
            try:
                destination = os.open(summary, os.O_WRONLY | os.O_CREAT | os.O_EXCL | os.O_NOFOLLOW,
                                      0o600, dir_fd=parent)
                with os.fdopen(destination, "w") as output:
                    json.dump(receipt, output, indent=2)
                    output.write("\n")
                    output.flush()
                    os.fsync(output.fileno())
                os.replace(summary, "proof-cleanup.json", src_dir_fd=parent, dst_dir_fd=parent)
            finally:
                try:
                    os.unlink(summary, dir_fd=parent)
                except FileNotFoundError:
                    pass
            os.fsync(parent)
        finally:
            try:
                os.unlink(name, dir_fd=parent)
            except FileNotFoundError:
                pass


def read_json_lines(root, path):
    with ExitStack() as stack:
        value = Bound(root, path, stack)
        rows = [json.loads(row) for row in os.pread(value.fd, value.expected["size"], 0).splitlines() if row.strip()]
        value.verify()
    if not all(isinstance(row, dict) for row in rows):
        refuse("invalid task records")
    return rows


def ownership(root, issue, bundles, *, owner=None):
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
    if owner is not None and matches[0].get("assignee") != owner:
        refuse("proof task was transferred to a different owner")
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
                               for name in PROOF_BINARIES | {n + ".d" for n in PROOF_BINARIES})):
                    refuse("selected output still has a Frontier claim")


def retire(root, relatives, lock_path, active_check, users_check, *, apply=False,
           retain_metadata=False, registered_proof=False):
    root = Path(root).resolve()
    receipt = {"operation": "released_qa_build", "apply": apply, "status": "preflight",
               "completed": [], "selected": [], "retained": [], "observed_absent": [],
               "runtime_proof": "held ELF write descriptors; no unrelated-app idle assertion"}
    bound = []
    roles = PROOF_BINARIES if retain_metadata else BINARIES
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
                if release.get("owner_hold"):
                    refuse("selected proof has an explicit owner hold")
                if registered_proof and (not retain_metadata or not all(
                        isinstance(release.get(key), str) and release[key].strip()
                        for key in ("owner", "session", "reason", "pending_proof"))
                        or not isinstance(release.get("resolution"), dict)
                        or not all(isinstance(release["resolution"].get(key), str)
                                   and release["resolution"][key].strip()
                                   for key in ("result", "provenance"))):
                    refuse("registered proof lacks authenticated retention and resolution")
                issue = release.get("issue_id")
                if release.get("state") != "released" or not isinstance(issue, str) or not issue:
                    refuse("build-info requires a positive released issue annotation")
                ownership(root, issue, [bundle], owner=release.get("owner") if retain_metadata else None)
                records.append((metadata, issue, bundle))
                debug, chain = q.directory(root, PurePosixPath(bundle + "/debug").parts, stack)
                names = sorted(os.listdir(debug))
                directories.append((bundle, debug, chain, set(names)))
                allowed = roles | {name + ".d" for name in roles}
                if set(names) - allowed:
                    refuse("unknown debug output; preserve it and classify before retirement")
                files, binaries = info.get("files_sha256"), info.get("binary_sha256")
                if any(v is not None and not isinstance(v, dict) for v in (files, binaries)):
                    refuse("declared hash maps must be objects")
                files, binaries = files or {}, binaries or {}
                for name in names:
                    relative = bundle + "/debug/" + name
                    expected = files.get("debug/" + name)
                    if name.endswith(".d") and retain_metadata:
                        value = Bound(root, relative, stack)
                        if expected is not None and expected != value.expected["sha256"]:
                            refuse("retained dependency metadata hash changed")
                        receipt["retained"].append(relative)
                        continue
                    if name.endswith(".d") and expected is None:
                        # Unpinned dependency metadata is retained, never inferred
                        # disposable from a neighboring binary's hash.
                        value = Bound(root, relative, stack)
                        receipt["retained"].append(relative)
                        continue
                    value = Bound(root, relative, stack, executable=name in roles)
                    if expected is None and name in roles:
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
            # Registered lifecycle outputs are positively bound with writable
            # ELF descriptors above: the kernel refuses an executing image.
            # Legacy retirement retains its existing process/mapping checks.
            if not registered_proof and users_check([root / p for p in paths]):
                refuse("selected output is mapped by an active process")

            def validate():
                idle()
                for metadata, issue, bundle in records:
                    metadata.verify()
                    ownership(root, issue, [bundle], owner=metadata.json()["build_retention"].get("owner") if retain_metadata else None)
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
