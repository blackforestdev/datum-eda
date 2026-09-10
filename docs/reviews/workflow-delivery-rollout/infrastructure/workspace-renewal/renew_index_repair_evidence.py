"""Renew construction and S01 captures for the pinned index-preservation repair."""

import json
import os
from pathlib import Path
import subprocess
import sys
import time


PIN = "4e11d60b6f0ec50aa391c68ed39a0df138adf8cf"
OLD = "0e5b8064a9faca407fac43bd2b362ba4d7200327"


def main():
    root = Path(__file__).resolve().parents[5]
    runtime = root / ".git/datum-wdq/proposals/index-preservation-repair-20260910"
    store = root / ".git/datum-wdq/proposals/index-repair-evidence-20260910"
    evidence = root / "docs/reviews/workflow-delivery-rollout/infrastructure/workspace-renewal"
    resume = sys.argv[1:] == ["--resume"]
    assert not sys.argv[1:] or resume
    assert store.exists() == resume
    sys.path.insert(0, str(runtime / "scripts"))
    from workflow_delivery_capture_command import save
    from workflow_delivery_capture_state import git
    from workflow_delivery_capture_fixture import prepare_fixture
    from workflow_delivery_capture_cases import run_case
    from workflow_delivery_capture_hook import run_hook_case
    from workflow_delivery_review_input import REPORT_CASES, REPORT_FIXTURE
    from workflow_delivery_tree import Tree
    from workflow_delivery_io import canonical_json, sha256
    from workflow_delivery_authority import authority_sha256
    assert git(runtime, "rev-parse", "HEAD").decode().strip() == PIN
    assert not git(runtime, "status", "--porcelain")
    if resume:
        construction = json.loads((store / "construction.json").read_bytes())
        assert construction["candidate"] == PIN
        tree = Tree(runtime, revision=PIN)
        contract = tree.json("specs/workflow_delivery/rollout.contract.json")
        manifest = tree.manifest(contract["input_roots"])
        assert sha256(canonical_json(manifest)) == construction["input_manifest_sha256"]
        return run_s01(root, runtime, store, save, prepare_fixture, run_case, run_hook_case,
                       REPORT_CASES, REPORT_FIXTURE, git, resume=True)
    store.mkdir()
    prior_receipt = json.loads((evidence / "typed-build-receipt.json").read_bytes())
    command = prior_receipt["build_command"][:]
    assert command[:5] == ["python3", "-I", "-S", "-B", "-c"]
    assert command[5].count(OLD) == 1
    command[5] = command[5].replace(OLD, PIN)
    started = time.time_ns()
    process = subprocess.Popen(command, cwd=runtime, stdout=subprocess.PIPE, stderr=subprocess.PIPE)
    out, err = process.communicate()
    save(store / "build-process.json", {"command": command, "cwd": str(runtime),
        "pid": process.pid, "started_ns": started, "ended_ns": time.time_ns(),
        "exit_code": process.returncode, "stdout": out.decode(), "stderr": err.decode()})
    assert process.returncode == 0
    observed = json.loads(out)
    tree = Tree(runtime, revision=PIN)
    contract = tree.json("specs/workflow_delivery/rollout.contract.json")
    manifest = tree.manifest(contract["input_roots"])
    (store / "input-manifest.json").write_bytes(canonical_json(manifest))
    assert sha256(canonical_json(manifest)) == observed["input_manifest_sha256"]
    save(store / "build-receipt.json", {"binary_sha256": observed["interpreter"]["sha256"],
        "build_command": command, "toolchain": observed["toolchain"],
        "input_manifest_sha256": observed["input_manifest_sha256"], "exit_code": 0})
    save(store / "construction.json", {"candidate": PIN, "modules": observed["modules_compiled"],
        "inputs": len(manifest), "authority_sha256": authority_sha256(tree, contract),
        "input_manifest_sha256": observed["input_manifest_sha256"], "activation_performed": False})
    run_s01(root, runtime, store, save, prepare_fixture, run_case, run_hook_case,
            REPORT_CASES, REPORT_FIXTURE, git, resume=False)


def run_s01(root, runtime, store, save, prepare_fixture, run_case, run_hook_case,
            report_cases, report_fixture, git, *, resume, pin=PIN):
    assert git(runtime, "rev-parse", "HEAD").decode().strip() == pin
    assert not git(runtime, "status", "--porcelain")
    batch = store / "s01"
    packet, report = batch / "packet", batch / "report-packet"
    if not resume:
        batch.mkdir()
        prepare_fixture(batch / "prepared-fixture", packet)
        prepare_fixture(batch / "report-prepared-fixture", report, fixture_variant=report_fixture)
    else:
        assert (packet / "recipe.json").is_file() and (report / "recipe.json").is_file()
    prior_path = root / ".git/datum-wdq/proposals/wdq-i03-9cab72da-s01/observations-final.json"
    prior = json.loads(prior_path.read_bytes())
    env = {"PATH": os.environ["PATH"], "HOME": str(batch), "XDG_CONFIG_HOME": str(batch),
           "GIT_CONFIG_NOSYSTEM": "1", "GIT_CONFIG_GLOBAL": os.devnull, "LANG": "C"}
    observations = []
    for row in prior["observations"]:
        destination = batch / row["directory"]
        case, surface = row["captured_case_id"], row["surface"]
        selected = report if case in report_cases else packet
        if resume and (destination / "assessment.json").is_file():
            assessment = json.loads((destination / "assessment.json").read_bytes())
        else:
            assessment = (run_hook_case(selected, destination, env, case_id=case) if surface == "H"
                else run_case(selected, destination, runtime, env, case_id=case, surface=surface))
        assert assessment["observation_matches_expected"]
        observations.append({"case_id": row["case_id"], "captured_case_id": case,
            "surface": surface, "phase": row["phase"], "directory": destination.name,
            "assessment": assessment})
        print(json.dumps({"completed": len(observations), "case": case, "surface": surface}), flush=True)
    assert len(observations) == 36
    save(batch / "observations-complete.json", {"candidate": pin, "observations": observations,
        "expected": 36, "complete": True, "activation_performed": False})
    assert git(runtime, "rev-parse", "HEAD").decode().strip() == pin
    assert not git(runtime, "status", "--porcelain")


if __name__ == "__main__":
    main()
