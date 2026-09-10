"""Run reviewed capture recipes with recorded source, store and setup changes."""

import argparse
import hashlib
import json
from pathlib import Path
import shlex
import subprocess
import time


PIN = "4e11d60b6f0ec50aa391c68ed39a0df138adf8cf"
OLD = "0e5b8064a9faca407fac43bd2b362ba4d7200327"
BATCHES = {
    "baseline": ("renewed-baseline-observation.json", "command"),
    "s02-coverage": ("renewed-s02-coverage-observation.json", "command"),
    "s02-ownership": ("renewed-s02-ownership-observation.json", "initial_command"),
    "s03": ("renewed-s03-observation.json", "command"),
    "s04": ("renewed-s04-observation.json", "command"),
    "s05-snapshots": ("renewed-s05-snapshots-observation.json", "command"),
    "s06-selection": ("renewed-s06-selection-observation.json", "command"),
    "s06-headless": ("renewed-s06-headless-observation.json", "command"),
    "s06-trust": ("renewed-s06-trust-observation.json", "command"),
    "workspace-runtime": ("renewed-workspace-runtime-observation.json", "command"),
    "workspace-policy-legacy": ("renewed-workspace-policy-legacy-observation.json", "command"),
    "workspace-proof-input": ("renewed-workspace-proof-input-observation.json", "command"),
}


def save(path, value):
    with path.open("x") as stream:
        json.dump(value, stream, indent=2)
        stream.write("\n")


def main(*, pin=PIN, runtime_name="index-preservation-repair-20260910",
         store_name="index-repair-evidence-20260910"):
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("batch", choices=sorted(BATCHES))
    args = parser.parse_args()
    root = Path(__file__).resolve().parents[5]
    assert len(pin) == 40 and all(c in "0123456789abcdef" for c in pin)
    for name in (runtime_name, store_name):
        assert name and name not in (".", "..") and Path(name).name == name
    runtime = root / ".git/datum-wdq/proposals" / runtime_name
    store = root / ".git/datum-wdq/proposals" / store_name
    assert runtime.resolve() == runtime and store.resolve() == store and store.is_dir()
    record_name, field = BATCHES[args.batch]
    record = Path(__file__).with_name(record_name)
    original_bytes = record.read_bytes()
    original = shlex.split(json.loads(original_bytes)[field])
    assert len(original) == 6 and original[:5] == ["python3", "-I", "-S", "-B", "-c"]
    old_suffix = args.batch + ("-entrypoints" if args.batch.startswith("workspace-") else "")
    old_store = ".git/datum-wdq/proposals/wdq-full-candidate-inspection-20260909/renewed-" + old_suffix + "-0e5b8064"
    new_store = str(store.relative_to(root) / args.batch)
    code = original[5]
    assert code.count(OLD) == 1 and code.count(old_store) == 1
    recipe_changes = {}
    if args.batch == "s03":
        # The retained successful recipe resumed setup. This batch starts fresh.
        recipe_changes = {
            "assert store.is_dir()": "store.mkdir(exist_ok=False)",
            '  assert (packet/"recipe.json").is_file()\n  assert json.loads((packet/"recipe.json").read_text()).get("fixture_variant")==recipe.get("fixture_variant")':
                '  prepare_fixture(store/("prepared-fixture-"+str(index)),packet,fixture_variant=recipe.get("fixture_variant"))',
        }
        for before, after in recipe_changes.items():
            assert code.count(before) == 1
            code = code.replace(before, after)
    command = original[:5] + [code.replace(OLD, pin).replace(old_store, new_store)]
    git = lambda *words: subprocess.check_output(["git", "--no-optional-locks", *words], cwd=runtime)
    assert git("rev-parse", "HEAD").decode().strip() == pin
    assert not git("status", "--porcelain") and not (store / args.batch).exists()
    save(store / (args.batch + "-command.json"), {"command": command, "cwd": str(runtime),
        "source_record": str(record), "source_record_sha256": hashlib.sha256(original_bytes).hexdigest(),
        "substitutions": {OLD: pin, old_store: new_store}, "fresh_setup_changes": recipe_changes})
    out_path, err_path = (store / (args.batch + "-" + name + ".bin") for name in ("stdout", "stderr"))
    environment = None
    if args.batch == "s02-ownership":
        # Select the already recorded real formatter, not an isolated-HOME rustup proxy.
        import os
        formatter = json.loads(original_bytes)["formatter"]
        assert hashlib.sha256(Path(formatter["path"]).read_bytes()).hexdigest() == formatter["sha256"]
        environment = dict(os.environ, PATH=str(Path(formatter["path"]).parent) + ":/bin:/usr/bin")
        save(store / (args.batch + "-environment.json"), {"PATH": environment["PATH"], "formatter": formatter})
    with out_path.open("xb") as out, err_path.open("xb") as err:
        start = time.time_ns()
        process = subprocess.Popen(command, cwd=runtime, env=environment, stdout=out, stderr=err)
        save(store / (args.batch + "-started.json"), {"pid": process.pid, "started_ns": start})
        print(json.dumps({"batch": args.batch, "pid": process.pid}), flush=True)
        status = process.wait()
    save(store / (args.batch + "-process.json"), {"pid": process.pid, "started_ns": start,
        "ended_ns": time.time_ns(), "exit_code": status,
        "stdout_sha256": hashlib.sha256(out_path.read_bytes()).hexdigest(),
        "stderr_sha256": hashlib.sha256(err_path.read_bytes()).hexdigest()})
    assert git("rev-parse", "HEAD").decode().strip() == pin and not git("status", "--porcelain")
    print(json.dumps({"batch": args.batch, "exit_code": status}), flush=True)
    raise SystemExit(status)


if __name__ == "__main__":
    main()
