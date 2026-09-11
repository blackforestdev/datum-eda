"""Bounded owner-invoked publication; never automatic repair or roadmap acceptance.

The CLI authenticates its prepared runtime and exact external response first.
The owner must pause other writers: this cooperative lock cannot lock editors.
"""

import fcntl
import os
from pathlib import Path
import stat
import signal
import subprocess
import sys
import time

from workflow_delivery_activation_preflight import preflight, inspect_promotion_candidate, require
from workflow_delivery_activation_state import inspect_activation_state
from workflow_delivery_bootstrap import git
from workflow_delivery_capture_state import protected_state
from workflow_delivery_support_bundle import verify_bundle
from workflow_delivery_support_store import support_locations, proposed_local_trust
from workflow_delivery_workspace_authority import promotion_workspace

COORDINATION = "All writers to this checkout and its Git-common trust configuration are paused for this exact activation."


class ActivationSignals:
    """Defer handled signals until the owned child is reaped; keep the lock held."""

    def __init__(self, record):
        self.record = record
        self.signum = None
        self.previous = {}
        self.lock_fd = None

    def __enter__(self):
        for number in (signal.SIGINT, signal.SIGTERM):
            self.previous[number] = signal.signal(number, self.request_stop)
        return self

    def request_stop(self, number, frame):
        self.signum = self.signum or number  # No I/O or exceptions in the handler.

    def check(self):
        if self.signum is not None:
            raise KeyboardInterrupt(f"activation interrupted by signal {self.signum}; owned child has finished")

    def __exit__(self, kind, value, traceback):
        for number, handler in self.previous.items():
            signal.signal(number, handler)
        if kind is None:
            self.check()

    def run(self, root, command, *, timeout=120):
        self.check()
        env = {"PATH": os.defpath, "LC_ALL": "C", "GIT_CONFIG_NOSYSTEM": "1",
               "GIT_CONFIG_GLOBAL": os.devnull, "GIT_GRAFT_FILE": os.devnull}
        with subprocess.Popen(command, cwd=root, env=env, stdout=subprocess.PIPE,
                              stderr=subprocess.PIPE, start_new_session=True,
                              pass_fds=() if self.lock_fd is None else (self.lock_fd,)) as child:
            deadline = time.monotonic() + timeout
            reported = False
            try:
                self.record({"stage": "child-started", "pid": child.pid, "command": command})
                while True:
                    if self.signum is not None and not reported:
                        self.record({"stage": "interrupt-waiting-for-child", "pid": child.pid,
                                     "signal": self.signum})
                        reported = True
                    remaining = deadline - time.monotonic()
                    if remaining <= 0:
                        raise subprocess.TimeoutExpired(command, timeout)
                    try:
                        stdout, stderr = child.communicate(timeout=min(0.2, remaining))
                        break
                    except subprocess.TimeoutExpired:
                        continue
            except BaseException:
                # Terminate only this invocation's separately created process
                # group. Never identify children by executable/process name.
                try:
                    os.killpg(child.pid, signal.SIGKILL)
                except ProcessLookupError:
                    pass
                child.communicate()
                self.record({"stage": "child-stopped-after-error", "pid": child.pid,
                             "exit_code": child.returncode})
                raise
            self.record({"stage": "child-finished", "pid": child.pid, "exit_code": child.returncode,
                         "stdout": stdout.decode("utf-8", "replace"), "stderr": stderr.decode("utf-8", "replace")})
            self.check()
            return subprocess.CompletedProcess(command, child.returncode, stdout, stderr)

    def write_git(self, root, *arguments):
        result = self.run(root, ["git", "--no-replace-objects", "--no-optional-locks", *arguments])
        require(result.returncode == 0, "activation Git command failed: " + result.stderr.decode("utf-8", "replace"))
        return result.stdout


def activate(root, request, *, record):
    with ActivationSignals(record) as signals:
        return _activate(root, request, record=record, signals=signals)


def _activate(root, request, *, record, signals):
    """Publish only after reinspection; retain and report any partial transaction.

    record must durably append each event before returning. A failure never
    triggers reset, hook bypass, cleanup, force update or an automatic retry.
    """
    require(request["coordination"] == COORDINATION, "explicit writer-pause acknowledgement required")
    locations = support_locations(root, request["authority"])
    root = Path(root).resolve(strict=True)
    require(git(root, "config", "--type=bool", "--default=false", "--get", "extensions.worktreeConfig").strip() == b"false",
            "worktree-specific configuration requires separate reviewed activation support")
    lock_path = locations["store"] / "activation.lock"
    descriptor = os.open(lock_path, os.O_RDWR | os.O_CREAT | os.O_NOFOLLOW, 0o600)
    try:
        require(stat.S_ISREG(os.fstat(descriptor).st_mode), "activation lock must be a regular file")
        try:
            fcntl.flock(descriptor, fcntl.LOCK_EX | fcntl.LOCK_NB)
        except BlockingIOError as error:
            raise ValueError("another activation holds the Git-common lock") from error
        require(lock_path.stat().st_ino == os.fstat(descriptor).st_ino,
                "activation lock changed during acquisition")
        signals.lock_fd = descriptor  # The child retains the lock if the parent dies.
        prior = request["prior_local_trust"]
        proposed = proposed_local_trust(root, authority=request["authority"],
            base=request["base"], environment_path=request["environment_path"])
        require(proposed == request["proposed_local_trust"] and proposed != prior,
                "exact distinct replacement trust map required")
        bundle = verify_bundle(root, request["authority"])
        workspace = promotion_workspace(
            root, candidate=request["candidate"], authority=request["authority"])
        options = dict(base=request["base"], candidate=request["candidate"],
            input_roots=request["input_roots"], expected_local_trust=prior,
            publication_review=request["publication_review"], workspace=workspace)
        before = preflight(root, **options)
        evidence = inspect_promotion_candidate(root, base=request["base"], candidate=request["candidate"],
            authority=request["authority"], environment_path=request["environment_path"],
            publication_review=request["publication_review"])
        from workflow_delivery_tree import Tree
        from workflow_delivery_trust import Trust, FRONTIER_PATH
        from workflow_delivery_review import validate_defect_dispositions
        from workflow_delivery_activation_preflight import ROLLOUT
        tree = Tree(root, revision=request["candidate"])
        trust = Trust(tree, request["authority"], request["base"])
        item = next(i for i in tree.json(FRONTIER_PATH)["frontier"] if i["key"] == ROLLOUT)
        contract = tree.json(item["completion"]["delivery"]["contract_path"])
        review_path = contract["review_path"]
        validate_defect_dispositions(tree, tree.json(review_path), review_path, trust)
        require(preflight(root, **options) == before, "protected state changed during activation validation")
        require(verify_bundle(root, request["authority"]) == bundle, "support changed before publication")
        record({"stage": "validated", "inspection": evidence})
        # This log precedes the first mutation. Even abrupt death leaves an
        # explicit potentially-partial transaction, never a false success.
        record({"stage": "publication-starting", "base": request["base"], "candidate": request["candidate"]})
        require(git(root, "rev-parse", "HEAD").decode().strip() == request["base"],
                "HEAD moved before publication")
        signals.write_git(root, "merge", "--ff-only", request["candidate"])
        record({"stage": "published", "head": git(root, "rev-parse", "HEAD").decode().strip()})
        expected = dict(prior)
        keys = [key for key in proposed if key != "core.hooksPath"] + ["core.hooksPath"]
        for key in keys:
            state = protected_state(root, request["input_roots"])
            require(state["head"] == request["candidate"] and state["local_trust"] == expected,
                    "HEAD or local trust changed during activation")
            require(not git(root, "status", "--porcelain=v1", "--untracked-files=all"),
                    "worktree changed during activation")
            record({"stage": "config-starting", "key": key, "values": proposed[key]})
            signals.write_git(root, "config", "--local", "--replace-all", key, proposed[key][0])
            expected[key] = proposed[key]
            record({"stage": "config-written", "key": key})
        require(verify_bundle(root, request["authority"]) == bundle, "installed support changed")
        for key, values in proposed.items():
            require(git(root, "config", "--get-all", key).decode().splitlines() == values,
                    "effective Git configuration differs from installed local trust: " + key)
        state_options = dict(base=request["base"], candidate=request["candidate"],
            input_roots=request["input_roots"], prior_trust=prior, proposed_trust=proposed,
            workspace=workspace)
        state = inspect_activation_state(root, **state_options)
        require(state["state"] == "candidate_and_configuration_match_unverified",
                "publication/configuration do not match the reviewed candidate")
        commands = [[str(locations["hook"])],
                    [sys.executable, "-I", "-S", "-B", "-c",
                     "import sys; sys.path.insert(0, sys.argv[1]); from project_status import main; raise SystemExit(main(sys.argv[2:]))",
                     str(locations["scripts"]),
                     "--root", str(root), "check"]]
        for command in commands:
            record({"stage": "verification-starting", "command": command})
            result = signals.run(root, command)
            record({"stage": "verification-result", "command": command, "exit_code": result.returncode,
                    "stdout": result.stdout.decode("utf-8", "replace"),
                    "stderr": result.stderr.decode("utf-8", "replace")})
            require(result.returncode == 0, "installed verification failed; activation remains partial")
        require(inspect_activation_state(root, **state_options) == state,
                "protected state changed during installed verification")
        require(verify_bundle(root, request["authority"]) == bundle, "support changed during installed verification")
        signals.check()
        record({"stage": "verified", "activation_asserted": True, "roadmap_acceptance_asserted": False})
        return {"activation_asserted": True, "roadmap_acceptance_asserted": False,
                "head": request["candidate"], "local_trust": proposed}
    finally:
        os.close(descriptor)
