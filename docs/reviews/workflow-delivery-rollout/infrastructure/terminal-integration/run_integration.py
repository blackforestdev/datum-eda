"""Run retained real-roadmap recipes against exact claim-free I04 evidence."""

import argparse
import hashlib
import json
from pathlib import Path
import shlex
import subprocess
import sys
import time

ROOT = Path("/home/bfadmin/Documents/datum-eda")
PROPOSALS = ROOT / ".git/datum-wdq/proposals"
RUNTIME = PROPOSALS / "terminal-owner-20260910"
STORE = PROPOSALS / "terminal-integration-5407"
PACKET = PROPOSALS / "terminal-real-roadmap-5407"
PIN = "5407bc87d8b941e362bd26705a59d72da1a60176"
OBSERVED = "9924678c39fe5d176e58ebfd8e25ab6feb213f35"
HELPERS = ROOT / "docs/reviews/workflow-delivery-rollout/infrastructure/workspace-renewal"


def save(path, value):
    with path.open("x") as stream:
        json.dump(value, stream, indent=2)
        stream.write("\n")


def main():
    global PIN, STORE, PACKET
    assert sys.flags.isolated and sys.flags.no_site and sys.flags.dont_write_bytecode
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("batch", choices=("real-roadmap", "current-inventory"))
    parser.add_argument("--candidate", default=PIN)
    parser.add_argument("--store-name", default=STORE.name)
    parser.add_argument("--packet-name", default=PACKET.name)
    args = parser.parse_args()
    PIN = args.candidate
    assert len(PIN) == 40 and all(c in "0123456789abcdef" for c in PIN)
    for name in (args.store_name, args.packet_name):
        assert Path(name).name == name and name.startswith("terminal-")
    STORE, PACKET = PROPOSALS / args.store_name, PROPOSALS / args.packet_name
    assert STORE.is_dir() and STORE.resolve() == STORE and PACKET.resolve() == PACKET
    snapshot = json.loads((PACKET / "result.json").read_bytes())
    assert snapshot["source_commit"] == PIN and len(snapshot["frontier_keys"]) == 56
    assert hashlib.sha256((PACKET / "source.bundle").read_bytes()).hexdigest() == snapshot["bundle_sha256"]
    record = HELPERS / "renewed-real-roadmap-observation.json"
    if args.batch == "current-inventory":
        record = PROPOSALS / "wdq-full-candidate-inspection-20260909/independent-recheck-workspace-20260909/inventory-complete-command.json"
    raw = record.read_bytes()
    source = json.loads(raw)
    command = source["argv"] if args.batch == "current-inventory" else shlex.split(source["command"])
    assert command[:5] == ["python3", "-I", "-S", "-B", "-c"] and len(command) == 6
    code = command[5]
    changes = {}
    def replace(old, new):
        nonlocal code
        assert code.count(old) == 1, old
        changes[old] = new
        code = code.replace(old, new)
    replace('pin="0e5b8064a9faca407fac43bd2b362ba4d7200327"', 'pin=' + repr(PIN))
    replace('packet=support_locations(runtime,pin)["proposals"]/"renewed-real-roadmap-0e5b8064"',
            'packet=Path(' + repr(str(PACKET)) + ')')
    store_line = next(line for line in code.splitlines() if line.startswith("store="))
    replace(store_line, "store=Path(" + repr(str(STORE / args.batch)) + ")")
    if args.batch == "current-inventory":
        start, end = code.index('\ntransaction="21b3330f'), code.index('\npolicy=json.loads(')
        replace(code[start:end], "\nfixture_ref=pin\n")
        replace("independently observed current workspace paths", "freshly observed current workspace paths")
    replace('assert git(runtime,"rev-parse","HEAD").decode().strip()==pin and not git(runtime,"status","--porcelain")',
            'assert git(runtime,"rev-parse","HEAD").decode().strip()==' + repr(OBSERVED) + ' and not git(runtime,"status","--porcelain")')
    helper = (HELPERS / "retain_real_prestate.py").read_text()
    needle = "from workflow_delivery_capture_command import capture_command,save\n"
    injection = '\n_retention={}\nexec(compile(' + repr(helper) + ',"retain_real_prestate.py","exec"),_retention)\n'
    injection += '''_capture_command=capture_command
def capture_command(root,output,command,environment,**kwargs):
 metadata=_retention["retain"](root,output,command,packet=packet,nonce=kwargs["fixture_nonce"])
 result=_capture_command(root,output,command,environment,**kwargs)
 save(Path(output)/"prestate-archive.json",metadata)
 return result
'''
    replace(needle, needle + injection)
    compile(code, "terminal-real-integration-recipe", "exec")
    command = command[:5] + [code]
    assert not (STORE / args.batch).exists()
    save(STORE / (args.batch + "-command.json"), {"command": command, "cwd": str(RUNTIME),
        "source_record": str(record), "source_record_sha256": hashlib.sha256(raw).hexdigest(),
        "changes": changes, "candidate": PIN, "runtime": OBSERVED,
        "scope": "Actual claim-free I04 roadmap and complete evidence, not reopened execution or installation."})
    out, err = STORE / (args.batch + "-stdout.bin"), STORE / (args.batch + "-stderr.bin")
    with out.open("xb") as stdout, err.open("xb") as stderr:
        started = time.time_ns()
        process = subprocess.Popen(command, cwd=RUNTIME, stdout=stdout, stderr=stderr)
        save(STORE / (args.batch + "-started.json"), {"pid": process.pid, "started_ns": started})
        print(json.dumps({"batch": args.batch, "pid": process.pid}), flush=True)
        status = process.wait()
    save(STORE / (args.batch + "-process.json"), {"pid": process.pid, "started_ns": started,
        "finished_ns": time.time_ns(), "exit_code": status,
        "stdout_sha256": hashlib.sha256(out.read_bytes()).hexdigest(),
        "stderr_sha256": hashlib.sha256(err.read_bytes()).hexdigest()})
    print(json.dumps({"batch": args.batch, "exit_code": status}), flush=True)
    raise SystemExit(status)


if __name__ == "__main__":
    main()
