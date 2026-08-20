#!/usr/bin/env python3
"""Run the DTC-P21 deterministic corpus and release measurement offline."""

from __future__ import annotations

import json
import subprocess
from pathlib import Path


ROOT = Path(__file__).resolve().parents[1]


def main() -> int:
    subprocess.run(
        [
            "cargo",
            "test",
            "-p",
            "datum-terminal-core",
            "proof_tests",
            "--locked",
            "--offline",
        ],
        cwd=ROOT,
        check=True,
    )
    completed = subprocess.run(
        [
            "cargo",
            "run",
            "--release",
            "--quiet",
            "-p",
            "datum-terminal-core",
            "--example",
            "dtc_p21_probe",
            "--locked",
            "--offline",
        ],
        cwd=ROOT,
        check=True,
        capture_output=True,
        text=True,
    )
    measurement = json.loads(completed.stdout)
    assert measurement["schema"] == "datum-terminal-core-proof-v1"
    assert measurement["payload_bytes"] == 8 * 1024 * 1024
    assert measurement["actions"] > 0
    assert measurement["errors"] == 0
    assert measurement["elapsed_ns"] > 0
    assert measurement["mib_per_second"] > 0
    assert measurement["snapshot_rows"] >= measurement["history_rows"]
    print(json.dumps(measurement, sort_keys=True))
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
