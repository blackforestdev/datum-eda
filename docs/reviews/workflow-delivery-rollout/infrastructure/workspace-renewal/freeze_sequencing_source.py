"""Read-only construction receipt for the exact sequencing-repair source."""

import hashlib
import json
from pathlib import Path
import subprocess
import sys
import time


PIN = "1d48f249dc7071fc3718b345a4ab16b366af43be"


def main():
    if not (sys.flags.isolated and sys.flags.no_site and sys.flags.dont_write_bytecode):
        raise ValueError("source freeze requires Python -I -S -B")
    root = Path(__file__).resolve().parents[5]
    runtime = root / ".git/datum-wdq/proposals/sequencing-repair-20260910"
    def git(*args):
        return subprocess.check_output(["git", "--no-replace-objects", "--no-optional-locks", *args], cwd=runtime)
    started = time.time_ns()
    assert git("rev-parse", "HEAD").decode().strip() == PIN
    assert not git("status", "--porcelain")
    bootstrap = runtime / "scripts/workflow_delivery_source_only.py"
    raw = bootstrap.read_bytes()
    assert raw == git("show", PIN + ":scripts/workflow_delivery_source_only.py")
    namespace = {"__name__": "_sequencing_source_only"}
    exec(compile(raw, str(bootstrap), "exec"), namespace)
    namespace["install"](runtime / "scripts")
    sys.path.insert(0, str(runtime / "scripts"))
    from workflow_delivery_tree import Tree
    from workflow_delivery_io import canonical_json
    from workflow_delivery_authority import authority_sha256
    tree = Tree(runtime, revision=PIN)
    contract = tree.json("specs/workflow_delivery/rollout.contract.json")
    manifest = tree.manifest(contract["input_roots"])
    assert len(manifest) == 164
    assert manifest == Tree(runtime).manifest(contract["input_roots"])
    modules = [row for row in manifest if row["path"].endswith(".py")]
    for row in modules:
        compile(tree.read(row["path"]), row["path"], "exec")
    executable = Path(sys.executable).resolve()
    assert git("rev-parse", "HEAD").decode().strip() == PIN
    assert not git("status", "--porcelain")
    print(json.dumps({
        "schema_version": 1, "step": "WDQ-COMPAT", "source_commit": PIN,
        "retained_ref": "refs/datum-wdq/candidates/sequencing-review-inspector-20260910",
        "command": ["python3", "-I", "-S", "-B", str(Path(__file__).resolve())],
        "started_ns": started, "finished_ns": time.time_ns(),
        "script_sha256": hashlib.sha256(Path(__file__).read_bytes()).hexdigest(),
        "source_only_bootstrap_sha256": hashlib.sha256(raw).hexdigest(),
        "input_manifest_sha256": hashlib.sha256(canonical_json(manifest)).hexdigest(),
        "input_count": len(manifest), "modules_compiled_in_memory": len(modules),
        "authority_sha256": authority_sha256(tree, contract),
        "input_manifest": manifest, "toolchain": sys.version,
        "interpreter": {"path": str(executable), "sha256": hashlib.sha256(executable.read_bytes()).hexdigest()},
        "source_clean": True, "build_artifacts_written": False,
        "activation_performed": False, "repair_complete": False,
        "scope": "Exact source construction only; no renewed scenario observations, real-roadmap proof, independent replay or activation."
    }, indent=2))


if __name__ == "__main__":
    main()
