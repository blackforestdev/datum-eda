"""Construct final inputs and compare them with immutable observed-source inputs."""

import hashlib
import json
from pathlib import Path
import subprocess
import sys
import time

OBSERVED = "9924678c39fe5d176e58ebfd8e25ab6feb213f35"
FINAL = "0abb325d9f06e7c8e4ea7b36e3a7ea2d8cd6f45c"


def main():
    assert sys.flags.isolated and sys.flags.no_site and sys.flags.dont_write_bytecode
    root = Path(__file__).resolve().parents[5]
    runtime = root / ".git/datum-wdq/proposals/terminal-owner-20260910"
    destination = root / ".git/datum-wdq/proposals/terminal-final-assessment-01"
    assert not destination.exists()
    def git(*args):
        return subprocess.check_output(["git", "--no-replace-objects", "--no-optional-locks", *args], cwd=runtime)
    assert git("rev-parse", "HEAD").decode().strip() == OBSERVED
    assert not git("status", "--porcelain")
    started = time.time_ns()
    bootstrap = runtime / "scripts/workflow_delivery_source_only.py"
    raw = bootstrap.read_bytes()
    assert raw == git("show", FINAL + ":scripts/workflow_delivery_source_only.py")
    namespace = {"__name__": "_terminal_assessment_source_only"}
    exec(compile(raw, str(bootstrap), "exec"), namespace)
    namespace["install"](runtime / "scripts")
    sys.path.insert(0, str(runtime / "scripts"))
    from workflow_delivery_tree import Tree
    from workflow_delivery_io import canonical_json, sha256
    from workflow_delivery_authority import authority_sha256
    old, current = Tree(root, revision=OBSERVED), Tree(root, revision=FINAL)
    contract_path = "specs/workflow_delivery/rollout.contract.json"
    prior_contract, contract = old.json(contract_path), current.json(contract_path)
    assert contract["input_roots"] == prior_contract["input_roots"]
    before, after = old.manifest(contract["input_roots"]), current.manifest(contract["input_roots"])
    assert len(before) == len(after) == 164
    assert [row["path"] for row in before] == [row["path"] for row in after]
    changed = [{"path": a["path"], "before_sha256": b["sha256"], "after_sha256": a["sha256"]}
               for b, a in zip(before, after) if b != a]
    # Normative authority is deliberately separate from runtime build inputs.
    permitted = set()
    assert {row["path"] for row in changed} == permitted
    modules = [row for row in after if row["path"].endswith(".py")]
    for row in modules:
        source = current.read(row["path"])
        assert source == old.read(row["path"]) == (runtime / row["path"]).read_bytes()
        compile(source, row["path"], "exec")
    assert old.read("specs/workflow_delivery_policy.json") == current.read("specs/workflow_delivery_policy.json")
    # Only scenario placement is added to the machine contract; every pre-existing
    # runtime assertion, dimension, handler, permission and environment input stays.
    restored_contract = json.loads(json.dumps(contract))
    s04 = next(s for s in restored_contract["scenarios"] if s["id"] == "INFRA-S04")
    new_inputs = [s for s in s04["inputs"] if "WDQ-INFRA-NONCIRCULAR-RENEWAL" in s]
    assert len(new_inputs) == 1
    s04["inputs"].remove(new_inputs[0])
    assert restored_contract == prior_contract
    interpreter = Path(sys.executable).resolve()
    command = ["python3", "-I", "-S", "-B", str(Path(__file__).resolve())]
    receipt = {"binary_sha256": sha256(interpreter.read_bytes()), "build_command": command,
        "toolchain": sys.version, "input_manifest_sha256": sha256(canonical_json(after)), "exit_code": 0}
    result = {"source_commit": FINAL, "observed_source_commit": OBSERVED,
        "started_ns": started, "finished_ns": time.time_ns(), "command": command,
        "script_sha256": sha256(Path(__file__).read_bytes()), "input_manifest": after,
        "input_manifest_sha256": receipt["input_manifest_sha256"], "input_count": len(after),
        "modules_compiled_in_memory": len(modules), "authority_sha256": authority_sha256(current, contract),
        "observed_authority_sha256": authority_sha256(old, prior_contract),
        "interpreter": {"path": str(interpreter), "sha256": receipt["binary_sha256"]},
        "toolchain": sys.version, "changed_inputs": changed, "machine_contract_delta": new_inputs,
        "authority_changed": True,
        "runtime_bytes_equal": True, "policy_bytes_equal": True,
        "source_clean": True, "activation_performed": False,
        "scope": "Fresh final construction and source/contract comparison, not a new subprocess case run or independent replay.",
        "remaining_applicability": "Per-observation fixture, selected policy/environment, tools, trace identity, protected state and scenario applicability must still be assessed; this comparison alone does not authorize proof reuse."}
    assert not git("status", "--porcelain")
    destination.mkdir()
    for name, value in (("construction.json", result), ("build-receipt.json", receipt), ("input-manifest.json", after)):
        with (destination / name).open("xb") as stream:
            stream.write(canonical_json(value))
    print(json.dumps({"source": FINAL, "destination": str(destination), "inputs": len(after),
        "modules": len(modules), "changed_inputs": sorted(permitted), "proof_reuse_asserted": False}))


if __name__ == "__main__":
    main()
