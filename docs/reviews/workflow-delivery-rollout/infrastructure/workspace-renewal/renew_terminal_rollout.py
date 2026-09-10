"""Renew actual rollout observations for the frozen terminal-owner repair."""

import argparse
import hashlib
import json
from pathlib import Path
import subprocess
import sys
import time

HERE = Path(__file__).resolve().parent
ROOT = HERE.parents[4]
PIN = "9924678c39fe5d176e58ebfd8e25ab6feb213f35"
RUNTIME_NAME = "terminal-owner-20260910"
STORE_NAME = "terminal-rollout-evidence-20260910"
RUNTIME = ROOT / ".git/datum-wdq/proposals" / RUNTIME_NAME
STORE = ROOT / ".git/datum-wdq/proposals" / STORE_NAME


def freeze():
    """Compile committed Python inputs in memory, without a product checkout."""
    def git(*args):
        return subprocess.check_output(["git", "--no-replace-objects", "--no-optional-locks", *args], cwd=RUNTIME)
    assert git("rev-parse", "HEAD").decode().strip() == PIN
    assert not git("status", "--porcelain")
    assert not STORE.exists()
    started = time.time_ns()
    bootstrap = RUNTIME / "scripts/workflow_delivery_source_only.py"
    raw = bootstrap.read_bytes()
    assert raw == git("show", PIN + ":scripts/workflow_delivery_source_only.py")
    namespace = {"__name__": "_terminal_rollout_source_only"}
    exec(compile(raw, str(bootstrap), "exec"), namespace)
    namespace["install"](RUNTIME / "scripts")
    sys.path.insert(0, str(RUNTIME / "scripts"))
    from workflow_delivery_tree import Tree
    from workflow_delivery_io import canonical_json
    from workflow_delivery_authority import authority_sha256
    tree = Tree(RUNTIME, revision=PIN)
    contract = tree.json("specs/workflow_delivery/rollout.contract.json")
    manifest = tree.manifest(contract["input_roots"])
    assert len(manifest) == 164
    modules = [row for row in manifest if row["path"].endswith(".py")]
    for row in modules:
        source = tree.read(row["path"])
        assert (RUNTIME / row["path"]).read_bytes() == source
        compile(source, row["path"], "exec")
    executable = Path(sys.executable).resolve()
    receipt = {"schema_version": 1, "step": "WDQ-I04", "source_commit": PIN,
        "retained_ref": "refs/datum-wdq/candidates/terminal-owner-source-" + PIN,
        "command": ["python3", "-I", "-S", "-B", str(Path(__file__).resolve()), "freeze"],
        "started_ns": started, "finished_ns": time.time_ns(),
        "script_sha256": hashlib.sha256(Path(__file__).read_bytes()).hexdigest(),
        "source_only_bootstrap_sha256": hashlib.sha256(raw).hexdigest(),
        "input_manifest_sha256": hashlib.sha256(canonical_json(manifest)).hexdigest(),
        "input_count": len(manifest), "modules_compiled_in_memory": len(modules),
        "authority_sha256": authority_sha256(tree, contract),
        "input_manifest": manifest, "toolchain": sys.version,
        "interpreter": {"path": str(executable), "sha256": hashlib.sha256(executable.read_bytes()).hexdigest()},
        "source_clean": True, "build_artifacts_written": False, "activation_performed": False,
        "scope": "Committed input construction and checked-out Python equality; not full checkout equality, proof or acceptance."}
    assert not git("status", "--porcelain")
    STORE.mkdir()
    with (STORE / "source-freeze.json").open("x") as stream:
        json.dump(receipt, stream, indent=2)
        stream.write("\n")
    print(json.dumps({key: receipt[key] for key in
        ("source_commit", "input_count", "modules_compiled_in_memory", "input_manifest_sha256", "authority_sha256")}))


def main():
    assert sys.flags.isolated and sys.flags.no_site and sys.flags.dont_write_bytecode
    sys.path.insert(0, str(HERE))
    import run_index_repair_batch as batches
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("action", choices=["freeze", "s01", *sorted(batches.BATCHES)])
    args = parser.parse_args()
    if args.action == "freeze":
        freeze()
    elif args.action == "s01":
        receipt = json.loads((STORE / "source-freeze.json").read_bytes())
        assert receipt["source_commit"] == PIN
        bootstrap = RUNTIME / "scripts/workflow_delivery_source_only.py"
        raw = bootstrap.read_bytes()
        assert hashlib.sha256(raw).hexdigest() == receipt["source_only_bootstrap_sha256"]
        namespace = {"__name__": "_terminal_rollout_source_only"}
        exec(compile(raw, str(bootstrap), "exec"), namespace)
        namespace["install"](RUNTIME / "scripts")
        sys.path.insert(0, str(RUNTIME / "scripts"))
        from workflow_delivery_capture_command import save
        from workflow_delivery_capture_state import git
        from workflow_delivery_capture_fixture import prepare_fixture
        from workflow_delivery_capture_cases import run_case
        from workflow_delivery_capture_hook import run_hook_case
        from workflow_delivery_review_input import REPORT_CASES, REPORT_FIXTURE
        from renew_index_repair_evidence import run_s01
        run_s01(ROOT, RUNTIME, STORE, save, prepare_fixture, run_case, run_hook_case,
                REPORT_CASES, REPORT_FIXTURE, git, resume=False, pin=PIN)
    else:
        receipt = json.loads((STORE / "source-freeze.json").read_bytes())
        assert receipt["source_commit"] == PIN
        batches.main(pin=PIN, runtime_name=RUNTIME_NAME, store_name=STORE_NAME)


if __name__ == "__main__":
    main()
