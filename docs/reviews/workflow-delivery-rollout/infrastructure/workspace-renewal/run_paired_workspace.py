"""Repeat producer workspace recipes while retaining each exact pre-state."""

import hashlib
import json
from pathlib import Path
import subprocess
import sys
import time

ROOT = Path("/home/bfadmin/Documents/datum-eda")
PIN = "1d48f249dc7071fc3718b345a4ab16b366af43be"
RUNTIME = ROOT / ".git/datum-wdq/proposals/sequencing-repair-20260910"
STORE = ROOT / ".git/datum-wdq/proposals/sequencing-paired-evidence-20260910"
ORIGINAL = ROOT / ".git/datum-wdq/proposals/sequencing-evidence-20260910"


def save(path, value):
    with path.open("x") as stream:
        json.dump(value, stream, indent=2)
        stream.write("\n")


def main(*, pin=PIN, runtime=RUNTIME, store=STORE):
    assert sys.flags.isolated and sys.flags.no_site and sys.flags.dont_write_bytecode
    batch = sys.argv[1]
    assert batch in ("workspace-runtime", "workspace-policy-legacy", "workspace-proof-input")
    capture_name = sys.argv[2] if len(sys.argv) > 2 else batch
    assert capture_name == batch or (capture_name.startswith(batch+"-") and Path(capture_name).name == capture_name)
    raw = (ORIGINAL / (batch + "-command.json")).read_bytes()
    prior = json.loads(raw)
    command = prior["command"]
    assert command[:5] == ["python3", "-I", "-S", "-B", "-c"] and len(command) == 6
    old_store = str((ORIGINAL / batch).relative_to(ROOT))
    new_store = str((store / capture_name).relative_to(ROOT))
    assert command[5].count(old_store) == 1
    code = command[5].replace(old_store, new_store)
    if pin != PIN:
        assert len(pin) == 40 and all(c in "0123456789abcdef" for c in pin)
        assert code.count(PIN) == 1
        code = code.replace(PIN, pin)
    injection = '''
import hashlib
_capture_command = capture_command
def capture_command(root, output, command, environment, **kwargs):
 root, output = Path(root), Path(output)
 marker = root/".git/datum-wdq-capture-owner"
 assert marker.is_file() and not marker.is_symlink()
 assert marker.read_text().strip() == kwargs["fixture_nonce"]
 assert root.resolve() != runtime.resolve()
 archive = output.with_name(output.name+"-prestate.tar.xz")
 assert not archive.exists() and not output.exists()
 subprocess.run(["tar","-cJf",str(archive),"-C",str(root),"."],check=True)
 result = _capture_command(root,output,command,environment,**kwargs)
 save(output/"prestate-archive.json",{"path":str(archive),"sha256":hashlib.sha256(archive.read_bytes()).hexdigest(),"scope":"Exact owned fixture before invocation, including raw Git index and non-Git input bytes."})
 return result
'''
    needle = "from workflow_delivery_capture_command import capture_command,save\n"
    assert code.count(needle) == 1
    code = code.replace(needle, needle + injection)
    command = command[:5] + [code]
    compile(code, "paired-workspace-recipe", "exec")
    git = lambda *args: subprocess.check_output(["git", "--no-optional-locks", *args], cwd=runtime)
    assert git("rev-parse", "HEAD").decode().strip() == pin and not git("status", "--porcelain")
    assert store.resolve() == store and store.is_relative_to(ROOT / ".git/datum-wdq/proposals")
    assert not (store / capture_name).exists()
    save(store / (capture_name+"-command.json"), {"command":command,"cwd":str(runtime),
        "source_record":str(ORIGINAL/(batch+"-command.json")),
        "source_record_sha256":hashlib.sha256(raw).hexdigest(),
        "source_substitution": {PIN: pin},
        "scope":"New identified producer fixtures with per-invocation exact pre-state retention; historical inputs are untouched."})
    out, err = store/(capture_name+"-stdout.bin"), store/(capture_name+"-stderr.bin")
    with out.open("xb") as stdout, err.open("xb") as stderr:
        started = time.time_ns()
        process = subprocess.Popen(command,cwd=runtime,stdout=stdout,stderr=stderr)
        save(store/(capture_name+"-started.json"),{"pid":process.pid,"started_ns":started})
        print(json.dumps({"batch":batch,"pid":process.pid}),flush=True)
        status = process.wait()
    save(store/(capture_name+"-process.json"),{"pid":process.pid,"started_ns":started,
        "ended_ns":time.time_ns(),"exit_code":status,
        "stdout_sha256":hashlib.sha256(out.read_bytes()).hexdigest(),
        "stderr_sha256":hashlib.sha256(err.read_bytes()).hexdigest()})
    assert git("rev-parse", "HEAD").decode().strip() == pin and not git("status", "--porcelain")
    print(json.dumps({"batch":batch,"exit_code":status}),flush=True)
    raise SystemExit(status)


if __name__ == "__main__":
    main()
