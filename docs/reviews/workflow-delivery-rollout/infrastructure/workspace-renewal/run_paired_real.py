"""Run retained real-roadmap recipes with bounded exact pre-state archives."""

import hashlib
import json
from pathlib import Path
import subprocess
import sys
import time

ROOT = Path("/home/bfadmin/Documents/datum-eda")
PROPOSALS = ROOT / ".git/datum-wdq/proposals"
RUNTIME = PROPOSALS / "sequencing-repair-20260910"
ORIGINAL = PROPOSALS / "sequencing-evidence-20260910/owner-reopened"
STORE = PROPOSALS / "sequencing-paired-evidence-20260910/owner-reopened"
BUNDLE = PROPOSALS / "sequencing-roadmap-377323b2/source.bundle"
BUNDLE_SHA = "8b9f667fe2d8f406edf2e790060b7329393c2acc939723c69f6954ee4f072f6b"
PIN = "1d48f249dc7071fc3718b345a4ab16b366af43be"


def save(path, value):
    with path.open("x") as stream:
        json.dump(value, stream, indent=2)
        stream.write("\n")


def main():
    assert sys.flags.isolated and sys.flags.no_site and sys.flags.dont_write_bytecode
    batch = sys.argv[1]
    assert batch in ("real-roadmap", "current-inventory") and len(sys.argv) == 2
    assert hashlib.sha256(BUNDLE.read_bytes()).hexdigest() == BUNDLE_SHA
    raw = (ORIGINAL / (batch + "-command.json")).read_bytes()
    prior = json.loads(raw)
    command = prior["command"]
    assert command[:5] == ["python3", "-I", "-S", "-B", "-c"] and len(command) == 6
    old, new = str(ORIGINAL / batch), str(STORE / batch)
    assert command[5].count(old) == 1
    code = command[5].replace(old, new)
    helper = Path(__file__).with_name("retain_real_prestate.py").read_text()
    injection = f'''
_retention_namespace = {{}}
exec(compile({helper!r}, "retain_real_prestate.py", "exec"), _retention_namespace)
_capture_command = capture_command
def capture_command(root, output, command, environment, **kwargs):
 metadata = _retention_namespace["retain"](root, output, command, packet=packet, nonce=kwargs["fixture_nonce"])
 metadata["source_bundle_sha256"] = {BUNDLE_SHA!r}
 result = _capture_command(root, output, command, environment, **kwargs)
 save(Path(output)/"prestate-archive.json", metadata)
 return result
'''
    needle = "from workflow_delivery_capture_command import capture_command,save\n"
    assert code.count(needle) == 1
    code = code.replace(needle, needle + injection)
    command = command[:5] + [code]
    compile(code, "paired-real-recipe", "exec")
    git = lambda *args: subprocess.check_output(["git", "--no-optional-locks", *args], cwd=RUNTIME)
    assert git("rev-parse", "HEAD").decode().strip() == PIN and not git("status", "--porcelain")
    STORE.mkdir(exist_ok=True)
    assert not (STORE / batch).exists()
    save(STORE / (batch + "-command.json"), {"command": command, "cwd": str(RUNTIME),
        "source_record": str(ORIGINAL / (batch + "-command.json")),
        "source_record_sha256": hashlib.sha256(raw).hexdigest(),
        "scope": "New paired fixture; only capture location and pre-state retention change. Historical captures and installed trust remain untouched."})
    out, err = STORE / (batch + "-stdout.bin"), STORE / (batch + "-stderr.bin")
    with out.open("xb") as stdout, err.open("xb") as stderr:
        started = time.time_ns()
        process = subprocess.Popen(command, cwd=RUNTIME, stdout=stdout, stderr=stderr)
        save(STORE / (batch + "-started.json"), {"pid": process.pid, "started_ns": started})
        print(json.dumps({"batch": batch, "pid": process.pid}), flush=True)
        status = process.wait()
    save(STORE / (batch + "-process.json"), {"pid": process.pid, "started_ns": started,
        "ended_ns": time.time_ns(), "exit_code": status,
        "stdout_sha256": hashlib.sha256(out.read_bytes()).hexdigest(),
        "stderr_sha256": hashlib.sha256(err.read_bytes()).hexdigest()})
    assert git("rev-parse", "HEAD").decode().strip() == PIN and not git("status", "--porcelain")
    print(json.dumps({"batch": batch, "exit_code": status}), flush=True)
    raise SystemExit(status)


if __name__ == "__main__":
    main()
