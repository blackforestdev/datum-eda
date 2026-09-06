#!/usr/bin/env python3
"""Bind actual native observations and explicit producer findings into G04 proof.

This is not an independent review, acceptance receipt or enforcement activation.
Visual assertions must be supplied after inspecting the exact native captures;
the packager cannot infer them from machine checks or expected contract text.
"""

import argparse
from copy import deepcopy
import json
from pathlib import Path
import platform

from workflow_delivery_authority import authority_sha256
from workflow_delivery_contract import contract_sha256
from workflow_delivery_evidence_shapes import proof_shape
from workflow_delivery_io import sha256
from workflow_delivery_pilot_observations import evaluate, read, records
from workflow_delivery_tree import Tree

ROOT = Path(__file__).resolve().parents[1]
CONTRACT = "specs/workflow_delivery/pilot.contract.json"


def blob(path):
    return {"path": path.resolve().relative_to(ROOT).as_posix(),
            "sha256": sha256(path.read_bytes())}


def write(path, value):
    with path.open("x") as stream:
        stream.write(json.dumps(value, indent=2, ensure_ascii=False) + "\n")
    return blob(path)


def merge_registry(trace):
    entries = {}
    for record in trace:
        if record["event"] != "state":
            continue
        for entry in record["value"]["registry"]:
            key = entry["dispatch_key"]
            if key not in entries:
                entries[key] = deepcopy(entry)
                continue
            existing = entries[key]
            if any(existing[k] != entry[k] for k in ("handler_ref", "entry_surfaces")):
                raise ValueError("production registry mapping changed during observations: " + key)
            for context, state in entry["contexts"].items():
                if context in existing["contexts"] and existing["contexts"][context] != state:
                    raise ValueError("context identity aliases different availability: " + context)
                existing["contexts"][context] = deepcopy(state)
    if not entries:
        raise ValueError("production registry observations absent")
    return [entries[key] for key in sorted(entries)]


def observed_dispatches(trace, roots):
    # Zero is established from unchanged journal/design bytes for the whole
    # interaction, not guessed from availability or copied from the contract.
    for root in roots:
        diff = read(root / "source-diff.json")
        if diff["changed_or_removed"] or any(
            p != ".datum/gui-terminal-context.json" and not p.startswith(
                (".datum/terminal-contexts/", ".datum/tool-sessions/"))
            for p in diff["new_paths"]
        ):
            raise ValueError("source/journal preservation not established: " + str(root))
    return [dict(record["value"], mutation_count=0)
            for record in trace if record["event"] == "dispatch"]


def validate_assessment(assessment, contract):
    expected = {s["id"]: s for s in contract["scenarios"]}
    if set(assessment) != set(expected):
        raise ValueError("explicit producer findings required for all five scenarios")
    for sid, result in assessment.items():
        if set(result) != {"actual_visible", "actual_state", "assertions", "defects"}:
            raise ValueError("producer assessment fields differ: " + sid)
        dimensions = {k for k, v in expected[sid]["dimensions"].items()
                      if v["disposition"] == "required"}
        if {a["dimension"] for a in result["assertions"]} != dimensions:
            raise ValueError("explicit findings required for each applicable dimension: " + sid)
        if any(a["outcome"] != "pass" for a in result["assertions"]):
            raise ValueError("producer finding is not passing: " + sid)


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--native-root", type=Path, required=True)
    parser.add_argument("--build-root", type=Path, required=True)
    parser.add_argument("--assessment", type=Path, required=True)
    parser.add_argument("--output", type=Path, required=True, help="new packaging directory")
    parser.add_argument("--proof", type=Path, required=True, help="new final proof path")
    parser.add_argument("--producer-session", required=True)
    args = parser.parse_args()
    contract = read(ROOT / CONTRACT)
    assessment = read(args.assessment)
    validate_assessment(assessment, contract)
    source = read(args.build_root / "source.json")
    historical = Tree(ROOT, revision=source["source_commit"])
    authority = authority_sha256(historical, contract)
    if authority_sha256(Tree(ROOT), contract) != authority:
        raise ValueError("authority changed since the identified source closure")
    receipts = {key: read(args.build_root / (key + "-receipt.json")) for key in ("gui", "cli")}
    scenarios = contract["scenarios"]
    observations = [evaluate(args.native_root / s["id"], receipts) for s in scenarios]
    if not all(r["machine_checks_pass"] for r in observations):
        raise ValueError("native machine checks are not all passing")
    args.output.mkdir(parents=True, exist_ok=False)
    observation_blob = write(args.output / "machine-observations.json", observations)
    receipt = receipts["gui"]
    commands = ["python3 scripts/workflow_delivery_pilot_capture.py --gui target/debug/datum-gui "
                "--cli target/debug/datum-eda --output <new-native-root>/" + sid + " --scenario " + sid
                for sid in [s["id"] for s in scenarios] + ["PILOT-S03-pointer-regression"]]
    environment = write(args.output / "environment.json", {
        "os": platform.freedesktop_os_release()["PRETTY_NAME"],
        "backend": "X11: private headless Weston/pixman plus rootful Xwayland; private session/AT-SPI buses",
        "toolchain": receipt["toolchain"], "scale": 1.0, "window_size": [1280, 768],
        "input_method": "xdotool XTest pointer/wheel/key input; normal X11 WM_DELETE_WINDOW close",
        "reproduction_commands": commands,
    })
    results = []
    for scenario in scenarios:
        sid = scenario["id"]
        roots = [args.native_root / sid]
        if sid == "PILOT-S03":
            roots.append(args.native_root / "PILOT-S03-pointer-regression")
        trace = [record for root in roots for record in records(root)]
        result = dict(deepcopy(assessment[sid]), scenario_id=sid, outcome="pass")
        registry = write(args.output / (sid + "-registry.json"), {
            "schema_version": 1, "binary_sha256": receipt["binary_sha256"],
            "entries": merge_registry(trace),
        })
        event = write(args.output / (sid + "-events.json"), {
            "schema_version": 1, "scenario_id": sid, "producer_session": args.producer_session,
            "method": scenario["method"], "binary_sha256": receipt["binary_sha256"],
            "authority_sha256": authority, "inputs": scenario["inputs"],
            "dispatches": observed_dispatches(trace, roots),
            "actual_visible": result["actual_visible"], "actual_state": result["actual_state"],
        })
        raw = [path for root in roots for path in sorted(root.iterdir()) if path.is_file()]
        captures = [blob(p) for p in raw if p.suffix == ".png"]
        state = [blob(p) for p in raw if p.suffix != ".png"] + [
            blob(args.assessment), observation_blob, blob(args.build_root / "cli-receipt.json"),
            blob(args.build_root / "source.json"), blob(args.build_root / "build.log")]
        index = write(args.output / (sid + "-artifacts.json"), {
            "kind": "datum.workflow-delivery.artifacts/v1", "scenario_id": sid,
            "events": [event], "captures": captures, "state": state, "registry": registry,
        })
        result["artifacts"] = [index, event, registry] + captures + state
        results.append(result)
    proof = {
        "schema_version": 1, "contract_sha256": contract_sha256(contract),
        "producer_session": args.producer_session, "source_commit": source["source_commit"],
        "input_manifest": blob(args.build_root / "input-manifest.json"),
        "build": {"command": receipt["build_command"], "receipt": blob(args.build_root / "gui-receipt.json"),
                  "toolchain": receipt["toolchain"], "source_clean": source["source_clean"]},
        "fixture": blob(ROOT / "research/process-quality/evidence/wdq-c01-current/native-fixture.tar.gz"),
        "environment": environment, "results": results,
    }
    proof_shape(proof, str(args.proof))
    write(args.proof, proof)
    print("Producer proof packaged; staged/revision validation and independent replay still required.")


if __name__ == "__main__":
    main()
