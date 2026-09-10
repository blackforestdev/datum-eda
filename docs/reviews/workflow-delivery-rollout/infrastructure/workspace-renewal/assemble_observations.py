"""Assemble retained renewal observations; never execute, approve or install them."""

import argparse
from collections import Counter
import hashlib
import json
from pathlib import Path
import subprocess


PIN = "0e5b8064a9faca407fac43bd2b362ba4d7200327"
BATCHES = {
    "s01": 36, "s02-coverage": 110, "s02-ownership": 190, "s03": 185,
    "s04": 175, "s05-snapshots": 39, "s06-selection": 60,
    "s06-headless": 95, "s06-trust": 223, "baseline": 5,
}
SURFACES = {"C": "staged_cli", "R": "candidate_cli", "H": "owner_hook",
            "S-check": "project_status", "S-details": "project_status"}


def read(path):
    return json.loads(path.read_bytes())


def digest(path):
    return hashlib.sha256(path.read_bytes()).hexdigest()


def identity(row):
    return tuple(row[key] for key in ("case_id", "captured_case_id", "surface", "phase"))


def observe(runtime, capture, row):
    result = read(capture / "result.json")
    for artifact in result["artifacts"]:
        target = capture / artifact["path"]
        assert target.resolve().is_relative_to(capture.resolve()), target
        assert digest(target) == artifact["sha256"], target
    assert not result["changed_files"] and not result["changed_state_fields"]
    assert (capture / "before.json").read_bytes() == (capture / "after.json").read_bytes()
    selector = row["surface"].startswith("S-")
    source = "scripts/workflow_delivery_selector.py" if selector else "scripts/check_workflow_delivery.py"
    symbol = "selector_failures" if selector else "main"
    trace = capture / "calls.jsonl"
    calls = []
    if trace.exists():
        events = [json.loads(line) for line in trace.read_bytes().splitlines() if line.strip()]
        calls = [event for event in events if event.get("event") == "call"
                 and event.get("function") == symbol
                 and Path(event["source"]["path"]).name == Path(source).name]
        assert all(event["source"]["sha256"] == digest(runtime / source) for event in calls)
    if not calls:
        assert row["surface"] == "H" and result["returncode"] != 0
        assert (capture / "stderr.bin").read_bytes().strip()
    assert len(calls) <= 1, capture
    return {
        **{key: row[key] for key in ("case_id", "captured_case_id", "surface", "phase")},
        "capture_path": str(capture), "invocation_id": result["invocation_id"],
        "result_sha256": digest(capture / "result.json"),
        "trace_sha256": digest(trace) if trace.exists() else None,
        "consumer": "selector" if selector else "validator",
        "handler_ref": {"path": source, "symbol": symbol},
        "handler_sha256": digest(runtime / source), "handler_invoked": bool(calls),
        "observed_call_count": len(calls), "result_kind": result["kind"],
        "returncode": result["returncode"], "protected_state_unchanged": True,
        "unavailable_reason": None if calls else (capture / "stderr.bin").read_text().strip(),
    }


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--root", type=Path, required=True)
    parser.add_argument("--runtime", type=Path, required=True)
    parser.add_argument("--output", type=Path, required=True)
    parser.add_argument("--index-repair", action="store_true")
    args = parser.parse_args()
    pin = "4e11d60b6f0ec50aa391c68ed39a0df138adf8cf" if args.index_repair else PIN
    root, runtime, output = (p.resolve() for p in (args.root, args.runtime, args.output))
    git = lambda *words: subprocess.check_output(["git", "--no-optional-locks", *words], cwd=runtime)
    assert git("rev-parse", "HEAD").decode().strip() == pin
    assert not git("status", "--porcelain")
    proposals = root / ".git/datum-wdq/proposals"
    retained = proposals / "wdq-full-candidate-inspection-20260909"
    repaired = proposals / "index-repair-evidence-20260910"
    assert output.is_relative_to(retained) and not output.exists()
    original = read(proposals / "wdq-i03-9cab72da-retained-package/dispatch-observations-complete.json")
    rows = []
    for name, count in BATCHES.items():
        batch = retained / ("renewed-" + name + "-0e5b8064")
        if args.index_repair:
            batch = repaired / name
        inventory_name = "observations-complete.json" if args.index_repair and name == "s01" else "observations.json"
        inventory = read(batch / inventory_name)
        assert inventory["candidate"] == pin and len(inventory["observations"]) == count
        for row in inventory["observations"]:
            capture = batch / row["directory"] / "capture"
            assessment = read(capture.parent / "assessment.json")
            assert assessment == row["assessment"] and assessment["observation_matches_expected"]
            assert assessment["result_sha256"] == digest(capture / "result.json")
            rows.append(observe(runtime, capture, row))
    real = runtime / ".git/datum-wdq/proposals/renewed-real-roadmap-captures-0e5b8064"
    if args.index_repair:
        real = repaired / "owner-reopened-reconciled/real-roadmap"
    inventory = read(real / "observations.json")
    assert inventory["candidate"] == pin
    real_binding = None
    if args.index_repair:
        preparation = read(repaired / "owner-reopened-roadmap-preparation-reconciled.json")
        fixture = preparation["fixture_commit"]
        assert all(inventory[key] == fixture for key in ("fixture_commit", "authority_ref", "base_ref"))
        assert inventory["owner_renewal_transaction"] == preparation["transaction"]
        real_binding = {"fixture_commit": fixture, "authority_ref": fixture, "base_ref": fixture,
                        "owner_renewal_transaction": preparation["transaction"],
                        "snapshot_bundle_sha256": inventory["snapshot_bundle_sha256"],
                        "scope": "Owner-authorized prospective reopened lifecycle; not installed-main proof."}
    for original_row in inventory["invocations"]:
        if original_row["case"] != "clean":
            continue
        surface = {"S": "S-check", "D": "S-details"}.get(original_row["surface"], original_row["surface"])
        row = dict(case_id="INFRA-S04-11", captured_case_id="INFRA-S04-11",
                   surface=surface, phase="primary")
        captured = observe(runtime, real / original_row["capture_directory"], row)
        assert captured["returncode"] == 0 and captured["invocation_id"] == original_row["invocation_id"]
        rows.append(captured)
    assert len(rows) == 1123
    assert Counter(map(identity, rows)) == Counter(map(identity, original["observations"]))
    original_count = len(rows)
    extra = [(retained / ("renewed-workspace-" + name + "-entrypoints-0e5b8064"),
              "renewal-result.json", None) for name in ("runtime", "policy-legacy", "proof-input")]
    extra.extend([(real, "observations.json", "clean"),
        (runtime / ".git/datum-wdq/proposals/renewed-real-roadmap-inventory-0e5b8064",
         "observations.json", None)])
    if args.index_repair:
        extra = [(repaired / ("workspace-" + name), "renewal-result.json", None)
                 for name in ("runtime", "policy-legacy", "proof-input")]
        extra.extend([(real, "observations.json", "clean"),
                      (repaired / "owner-reopened-reconciled/current-inventory", "observations.json", None)])
    for batch, inventory_name, excluded in extra:
        inventory = read(batch / inventory_name)
        assert inventory["candidate"] == pin
        if args.index_repair and "fixture_commit" in inventory:
            assert all(inventory[key] == real_binding[key]
                       for key in ("fixture_commit", "authority_ref", "base_ref", "owner_renewal_transaction"))
        for saved in inventory["invocations"]:
            if saved["case"] == excluded:
                continue
            capture = batch / saved["capture_directory"]
            invocation = read(capture / "invocation.json")
            surface = {"S": "S-check", "D": "S-details"}.get(saved["surface"], saved["surface"])
            row = dict(case_id="INFRA-S05-WORKSPACE-" + saved["case"],
                       captured_case_id=invocation["case_id"], surface=surface, phase="compatibility")
            captured = observe(runtime, capture, row)
            assert captured["invocation_id"] == saved["invocation_id"]
            assert captured["returncode"] == saved["returncode"]
            assert (captured["returncode"] != 0) == saved["expected_refusal"]
            rows.append(captured)
    assert len(rows) - original_count == 191
    ids = [row["invocation_id"] for row in rows]
    assert len(set(ids)) == len(ids)
    assert not set(ids) & {row["invocation_id"] for row in original["observations"]}
    registry = {}
    dispatches = {}
    for row in rows:
        consumer = row["consumer"]
        entry = registry.setdefault(consumer, {"dispatch_key": Path(row["handler_ref"]["path"]).stem
            + "." + row["handler_ref"]["symbol"], "handler_ref": row["handler_ref"],
            "entry_surfaces": ["project_status"] if consumer == "selector" else
            ["staged_cli", "candidate_cli", "owner_hook"], "contexts": {}})
        context = "invocation-" + row["invocation_id"]
        enabled = row["handler_invoked"]
        reason = "Exact pinned handler call observed." if enabled else row["unavailable_reason"]
        entry["contexts"][context] = {"enabled": enabled, "reason": reason}
        sid = row["case_id"][:9]
        dispatches.setdefault(sid, []).append({"dispatch_key": entry["dispatch_key"],
            "entry_surface": SURFACES[row["surface"]], "context": context,
            "eligible": enabled, "enabled": enabled, "invoked": enabled,
            "handler_ref": entry["handler_ref"], "unavailable_reason": None if enabled else reason,
            "mutation_count": 0})
    output.mkdir()
    products = {"dispatch-observations.json": {"schema_version": 1, "source_commit": pin,
        "observations": rows, "original_inventory_exactly_covered": True,
        "original_inventory_observations": original_count, "additional_workspace_observations": 191,
        "additional_workspace_evidence_included": True, "independent_replay_complete": False,
        "real_roadmap_binding": real_binding,
        "acceptance_asserted": False}, "registry-entries.json": list(registry.values()),
        "scenario-dispatches.json": dispatches}
    for name, value in products.items():
        (output / name).write_text(json.dumps(value, indent=2) + "\n")
    assert git("rev-parse", "HEAD").decode().strip() == pin and not git("status", "--porcelain")
    print(json.dumps({"observations": len(rows), "scenarios": {key: len(value) for key, value
        in dispatches.items()}, "output": str(output), "complete_proof": False}))


if __name__ == "__main__":
    main()
