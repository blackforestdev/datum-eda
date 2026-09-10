"""Run a fixed sequence of fresh proof batches, stopping at the first failure."""

import hashlib
import json
import os
from pathlib import Path
import subprocess
import sys
import time

HERE = Path(__file__).resolve().parent
BATCHES = ("s01", "s03", "s04", "s05-snapshots", "s06-selection", "s06-headless", "s06-trust")


def main():
    assert sys.flags.isolated and sys.flags.no_site and sys.flags.dont_write_bytecode
    sys.path.insert(0, str(HERE))
    from renew_terminal_rollout import PIN, ROOT, STORE
    assert len(sys.argv) == 1
    destination = STORE / "standard-series"
    assert not destination.exists()
    assert all(not (STORE / batch).exists() for batch in BATCHES)
    assert json.loads((STORE / "s02-coverage-process.json").read_bytes())["exit_code"] == 0
    assert json.loads((STORE / "source-freeze.json").read_bytes())["source_commit"] == PIN
    destination.mkdir()
    def save(name, value):
        with (destination / name).open("x") as stream:
            json.dump(value, stream, indent=2)
            stream.write("\n")
    driver = HERE / "renew_terminal_rollout.py"
    save("request.json", {"source": PIN, "batches": BATCHES,
        "driver_sha256": hashlib.sha256(driver.read_bytes()).hexdigest(),
        "series_sha256": hashlib.sha256(Path(__file__).read_bytes()).hexdigest(),
        "pid": os.getpid(), "scope": "Serial fresh synthetic recipe renewal; no real-roadmap, independent review or activation assertion."})
    completed = []
    for batch in BATCHES:
        command = [sys.executable, "-I", "-S", "-B", str(driver), batch]
        out, err = destination / (batch + "-stdout.bin"), destination / (batch + "-stderr.bin")
        with out.open("xb") as stdout, err.open("xb") as stderr:
            started = time.time_ns()
            process = subprocess.Popen(command, cwd=ROOT, stdout=stdout, stderr=stderr)
            save(batch + "-started.json", {"command": command, "pid": process.pid, "started_ns": started})
            print(json.dumps({"batch": batch, "pid": process.pid}), flush=True)
            result = process.wait()
        save(batch + "-process.json", {"pid": process.pid, "started_ns": started,
            "ended_ns": time.time_ns(), "exit_code": result,
            "stdout_sha256": hashlib.sha256(out.read_bytes()).hexdigest(),
            "stderr_sha256": hashlib.sha256(err.read_bytes()).hexdigest()})
        print(json.dumps({"batch": batch, "exit_code": result}), flush=True)
        if result:
            raise SystemExit(result)
        completed.append(batch)
    save("complete.json", {"source": PIN, "completed": completed, "activation_performed": False})


if __name__ == "__main__":
    main()
