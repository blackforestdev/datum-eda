"""Bounded owner-invoked publication; never automatic repair or roadmap acceptance.

The CLI authenticates its prepared runtime and exact external response first.
The owner must pause other writers: this cooperative lock cannot lock editors.
"""

import fcntl
import os
from pathlib import Path
import stat
import subprocess
import sys

from workflow_delivery_activation_preflight import preflight, inspect_promotion_candidate, require
from workflow_delivery_activation_state import inspect_activation_state
from workflow_delivery_bootstrap import git
from workflow_delivery_capture_state import protected_state
from workflow_delivery_support_bundle import verify_bundle
from workflow_delivery_support_store import support_locations, proposed_local_trust

COORDINATION = "All writers to this checkout and its Git-common trust configuration are paused for this exact activation."


def activate(root, request, *, record):
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
        prior = request["prior_local_trust"]
        proposed = proposed_local_trust(root, authority=request["authority"],
            base=request["base"], environment_path=request["environment_path"])
        require(proposed == request["proposed_local_trust"] and proposed != prior,
                "exact distinct replacement trust map required")
        bundle = verify_bundle(root, request["authority"])
        options = dict(base=request["base"], candidate=request["candidate"],
            input_roots=request["input_roots"], expected_local_trust=prior,
            publication_review=request["publication_review"])
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
        git(root, "merge", "--ff-only", request["candidate"])
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
            git(root, "config", "--local", "--replace-all", key, proposed[key][0])
            expected[key] = proposed[key]
            record({"stage": "config-written", "key": key})
        require(verify_bundle(root, request["authority"]) == bundle, "installed support changed")
        for key, values in proposed.items():
            require(git(root, "config", "--get-all", key).decode().splitlines() == values,
                    "effective Git configuration differs from installed local trust: " + key)
        state_options = dict(base=request["base"], candidate=request["candidate"],
            input_roots=request["input_roots"], prior_trust=prior, proposed_trust=proposed)
        state = inspect_activation_state(root, **state_options)
        require(state["state"] == "candidate_and_configuration_match_unverified",
                "publication/configuration do not match the reviewed candidate")
        commands = [[str(locations["hook"])],
                    [sys.executable, "-I", "-S", "-B", "-c",
                     "import sys; sys.path.insert(0, sys.argv[1]); from project_status import main; raise SystemExit(main(sys.argv[2:]))",
                     str(locations["scripts"]),
                     "--root", str(root), "check"]]
        env = {"PATH": os.defpath, "LC_ALL": "C", "GIT_CONFIG_NOSYSTEM": "1", "GIT_CONFIG_GLOBAL": os.devnull}
        for command in commands:
            record({"stage": "verification-starting", "command": command})
            result = subprocess.run(command, cwd=root, env=env, capture_output=True, timeout=120)
            record({"stage": "verification-result", "command": command, "exit_code": result.returncode,
                    "stdout": result.stdout.decode("utf-8", "replace"),
                    "stderr": result.stderr.decode("utf-8", "replace")})
            require(result.returncode == 0, "installed verification failed; activation remains partial")
        require(inspect_activation_state(root, **state_options) == state,
                "protected state changed during installed verification")
        require(verify_bundle(root, request["authority"]) == bundle, "support changed during installed verification")
        record({"stage": "verified", "activation_asserted": True, "roadmap_acceptance_asserted": False})
        return {"activation_asserted": True, "roadmap_acceptance_asserted": False,
                "head": request["candidate"], "local_trust": proposed}
    finally:
        os.close(descriptor)
