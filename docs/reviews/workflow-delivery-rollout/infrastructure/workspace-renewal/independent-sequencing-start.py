"""Retain independent construction and exact-fixture lease diagnostic execution."""
import hashlib
import json
from pathlib import Path
import subprocess
import sys
import time

ROOT = Path(__file__).resolve().parents[5]
STORE = ROOT / ".git/datum-wdq/proposals/independent-sequencing-20260910"
RUNTIME = ROOT / ".git/datum-wdq/proposals/sequencing-repair-20260910"
PIN = "1d48f249dc7071fc3718b345a4ab16b366af43be"

def save(path, value):
    with path.open("x") as stream:
        json.dump(value, stream, indent=2)
        stream.write("\n")

def run(name, command):
    save(STORE / (name + "-command.json"), {"argv": command, "cwd": str(RUNTIME)})
    with (STORE / (name + "-stdout.bin")).open("xb") as out, (STORE / (name + "-stderr.bin")).open("xb") as err:
        started = time.time_ns()
        process = subprocess.Popen(command, cwd=RUNTIME, stdout=out, stderr=err)
        save(STORE / (name + "-started.json"), {"pid": process.pid, "started_ns": started})
        print(json.dumps({"name": name, "pid": process.pid}), flush=True)
        code = process.wait()
    save(STORE / (name + "-process.json"), {"pid": process.pid, "started_ns": started,
        "ended_ns": time.time_ns(), "exit_code": code,
        **{key + "_sha256": hashlib.sha256((STORE / (name + "-" + key + ".bin")).read_bytes()).hexdigest()
           for key in ("stdout", "stderr")}})
    return code

def main():
    STORE.mkdir(exist_ok=False)
    command = ["python3", "-I", "-S", "-B", str(Path(__file__).with_name("freeze_sequencing_source.py"))]
    assert run("construction", command) == 0
    actual = json.loads((STORE / "construction-stdout.bin").read_bytes())
    expected = json.loads(Path(__file__).with_name("sequencing-source-freeze.json").read_bytes())
    fields = ["source_commit", "input_manifest", "input_manifest_sha256", "input_count",
              "modules_compiled_in_memory", "authority_sha256", "interpreter", "toolchain",
              "source_only_bootstrap_sha256", "script_sha256"]
    assert all(actual[key] == expected[key] for key in fields)
    save(STORE / "construction-verification.json", {"equal_fields": fields, "source": PIN,
         "reviewer_session": "wdq-independent-review-20260906", "independent_recheck_complete": False})
    old = ROOT / ".git/datum-wdq/proposals/wdq-full-candidate-inspection-20260909/independent-recheck-index-repair-s02-coverage-20260910/command.json"
    command = json.loads(old.read_bytes())["argv"]
    code = command[-1]
    code = code.replace("base/'independent-recheck-index-repair-s02-coverage-20260910'", "root/'.git/datum-wdq/proposals/independent-sequencing-20260910/ownership-expiry-diagnostic'")
    code = code.replace("index-preservation-repair-20260910", "sequencing-repair-20260910").replace("index-repair-evidence-20260910", "sequencing-evidence-20260910")
    code = code.replace("s02-coverage", "s02-ownership").replace("for row in inventory['observations']:", "for row in inventory['observations'][:2]:")
    code = code.replace("assert len(results)==110", "assert len(results)==2").replace("4e11d60b6f0ec50aa391c68ed39a0df138adf8cf", PIN)
    (STORE / "ownership-expiry-diagnostic").mkdir()
    # Existing collector source was read before reuse. These are exact copies;
    # no lease, clock or original fixture bytes are changed.
    save(STORE / "diagnostic-provenance.json", {"prior_command": str(old),
        "prior_command_sha256": hashlib.sha256(old.read_bytes()).hexdigest(),
        "scope": "First two ownership records only; diagnose already observed expired exact fixture lease."})
    status = run("ownership-expiry", command[:-1] + [code])
    print(json.dumps({"diagnostic_exit_code": status}), flush=True)

if __name__ == "__main__":
    main()
