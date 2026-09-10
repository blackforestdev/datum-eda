"""Capture real-roadmap or current-inventory cases with the recorded lease."""

import argparse
import hashlib
import json
from pathlib import Path
import shlex
import subprocess
import sys
import time

sys.path.insert(0, str(Path(__file__).resolve().parent))
from run_index_repair_batch import PIN, OLD, save


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("batch", choices=("real-roadmap", "current-inventory"), nargs="?", default="real-roadmap")
    batch = parser.parse_args().batch
    root = Path(__file__).resolve().parents[5]
    runtime = root / ".git/datum-wdq/proposals/index-preservation-repair-20260910"
    store = root / ".git/datum-wdq/proposals/index-repair-evidence-20260910"
    preparation = json.loads((store / "roadmap-preparation.json").read_bytes())
    assert preparation["runtime_source"] == PIN
    fixture = preparation["fixture_commit"]
    packet = Path(preparation["packet"])
    assert hashlib.sha256((packet / "source.bundle").read_bytes()).hexdigest() == preparation["snapshot"]["bundle_sha256"]
    record = Path(__file__).with_name("renewed-real-roadmap-observation.json")
    raw = record.read_bytes()
    original = shlex.split(json.loads(raw)["command"])
    if batch == "current-inventory":
        record = root / ".git/datum-wdq/proposals/wdq-full-candidate-inspection-20260909/independent-recheck-workspace-20260909/inventory-complete-command.json"
        raw = record.read_bytes()
        original = json.loads(raw)["argv"]
    assert original[:5] == ["python3", "-I", "-S", "-B", "-c"] and len(original) == 6
    changes = {
        OLD: PIN,
        'packet=support_locations(runtime,pin)["proposals"]/"renewed-real-roadmap-0e5b8064"':
            "fixture_ref=" + repr(fixture) + "\npacket=Path(" + repr(str(packet)) + ")",
        'store=packet.parent/"renewed-real-roadmap-captures-0e5b8064"':
            "store=Path(" + repr(str(store / "real-roadmap")) + ")",
        'git(root,"checkout","--quiet","--detach",pin)':
            'git(root,"checkout","--quiet","--detach",fixture_ref)',
        '["--candidate-ref",pin]': '["--candidate-ref",fixture_ref]',
        '"One exact real-roadmap fixture with all production roots and .beads captured; required owner-local script; clean, ignored undeclared source amid verified optimization caches, and restoration. Not all workspace groups or independent replay."':
            '"Recorded current-lease fixture with identical runtime inputs/authority; all production roots and .beads captured. Clean, ignored undeclared source amid verified caches, and restoration. Not full proof, independent replay or acceptance."',
    }
    if batch == "current-inventory":
        del changes['store=packet.parent/"renewed-real-roadmap-captures-0e5b8064"']
        del changes['["--candidate-ref",pin]']
        del changes['"One exact real-roadmap fixture with all production roots and .beads captured; required owner-local script; clean, ignored undeclared source amid verified optimization caches, and restoration. Not all workspace groups or independent replay."']
        lines = original[5].splitlines()
        old_store = next(line for line in lines if line.startswith("store=Path("))
        changes[old_store] = "store=Path(" + repr(str(store / batch)) + ")"
        # Current claim and its history are already in the verified snapshot.
        start = original[5].index('\ntransaction="21b3330f')
        end = original[5].index('\npolicy=json.loads(', start)
        changes[original[5][start:end]] = "\n"
        changes["independently observed current workspace paths"] = "freshly observed current workspace paths"
    code = original[5]
    for before, after in changes.items():
        assert code.count(before) == 1, before
        code = code.replace(before, after)
    command = original[:5] + [code]
    compile(code, "recorded-real-roadmap-recipe", "exec")
    git = lambda *args: subprocess.check_output(["git", "--no-optional-locks", *args], cwd=runtime)
    assert git("rev-parse", "HEAD").decode().strip() == PIN and not git("status", "--porcelain")
    assert not (store / batch).exists()
    save(store / (batch + "-command.json"), {"command": command, "cwd": str(runtime),
        "source_record": str(record), "source_record_sha256": hashlib.sha256(raw).hexdigest(), "changes": changes,
        "fixture_commit": fixture, "preparation": str(store / "roadmap-preparation.json")})
    out_path, err_path = store / (batch + "-stdout.bin"), store / (batch + "-stderr.bin")
    with out_path.open("xb") as out, err_path.open("xb") as err:
        start = time.time_ns()
        process = subprocess.Popen(command, cwd=runtime, stdout=out, stderr=err)
        save(store / (batch + "-started.json"), {"pid": process.pid, "started_ns": start})
        print(json.dumps({"pid": process.pid}), flush=True)
        status = process.wait()
    save(store / (batch + "-process.json"), {"pid": process.pid, "started_ns": start,
        "ended_ns": time.time_ns(), "exit_code": status,
        "stdout_sha256": hashlib.sha256(out_path.read_bytes()).hexdigest(),
        "stderr_sha256": hashlib.sha256(err_path.read_bytes()).hexdigest()})
    assert git("rev-parse", "HEAD").decode().strip() == PIN and not git("status", "--porcelain")
    print(json.dumps({"exit_code": status}), flush=True)
    raise SystemExit(status)


if __name__ == "__main__":
    main()
