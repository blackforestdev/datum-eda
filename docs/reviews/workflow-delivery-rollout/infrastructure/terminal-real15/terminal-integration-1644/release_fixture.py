"""Verify a terminal integration batch and retire its fixture recoverably."""

import argparse
import hashlib
import json
import os
from pathlib import Path
import sys


def main():
    assert sys.flags.isolated and sys.flags.no_site and sys.flags.dont_write_bytecode
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("batch", choices=("real-roadmap", "current-inventory"))
    args = parser.parse_args()
    store = Path(__file__).resolve().parent
    batch = store / args.batch
    read = lambda path: json.loads(path.read_bytes())
    digest = lambda path: hashlib.sha256(path.read_bytes()).hexdigest()
    process = read(store / (args.batch + "-process.json"))
    assert process["exit_code"] == 0
    assert digest(store / (args.batch + "-stdout.bin")) == process["stdout_sha256"]
    assert digest(store / (args.batch + "-stderr.bin")) == process["stderr_sha256"]
    record = read(batch / "observations.json")
    assert record["candidate"] == "1644e2129f4ff83cec1cdf0e6e13534c3b258e8e"
    rows = record["invocations"]
    assert len(rows) == (15 if args.batch == "real-roadmap" else 9)
    assert len({row["invocation_id"] for row in rows}) == len(rows)
    fixture, retired = batch / "fixture", batch / "fixture-retired-after-producer"
    assert fixture.is_dir() and not fixture.is_symlink() and fixture.resolve() == fixture
    assert not retired.exists()
    pids = [process["pid"]]
    artifact_count = 0
    for row in rows:
        capture = batch / row["capture_directory"]
        assert capture.parent == batch and capture.is_dir() and not capture.is_symlink()
        result, invocation = read(capture / "result.json"), read(capture / "invocation.json")
        assert result["kind"] == "exited" and result["returncode"] == row["returncode"]
        assert (result["returncode"] != 0) == row["expected_refusal"]
        assert result["invocation_id"] == row["invocation_id"] == invocation["invocation_id"]
        assert not result["changed_files"] and not result["changed_state_fields"]
        assert (capture / "before.json").read_bytes() == (capture / "after.json").read_bytes()
        for artifact in result["artifacts"]:
            path = capture / artifact["path"]
            assert path.parent == capture and digest(path) == artifact["sha256"]
            artifact_count += 1
        descriptor = read(capture / "prestate-archive.json")
        archive = Path(descriptor["archive"])
        assert archive.parent == batch and digest(archive) == descriptor["archive_sha256"]
        assert descriptor["original_root"] == str(fixture)
        assert descriptor["fixture_nonce"] == invocation["fixture_nonce"]
        assert (fixture / ".git/datum-wdq-capture-owner").read_text().strip() == invocation["fixture_nonce"]
        pids.append(result["pid"])
    for pid in pids:
        try:
            os.kill(pid, 0)
        except ProcessLookupError:
            continue
        raise AssertionError("Recorded producer process still exists: " + str(pid))
    fixture.rename(retired)
    assert not fixture.exists() and retired.is_dir()
    result = {"candidate": record["candidate"], "batch": args.batch, "observations": len(rows),
        "artifact_hashes_verified": artifact_count, "full_states_unchanged": True,
        "terminal_processes_verified_absent": pids, "released_path": str(fixture),
        "recoverable_retired_fixture": str(retired), "activation_performed": False,
        "scope": "Producer complete; original pathname released only for independently restored exact replay."}
    with (store / (args.batch + "-custody-release.json")).open("x") as stream:
        json.dump(result, stream, indent=2)
        stream.write("\n")
    print(json.dumps(result))


if __name__ == "__main__":
    main()
