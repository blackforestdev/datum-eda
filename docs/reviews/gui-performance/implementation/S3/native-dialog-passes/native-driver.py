"""Bounded native timestamp plumbing proof; not a CPU/GPU budget trial.

Runs production launch adapters with disposable fixture state. No synthetic
desktop input, resize loop, source mutation, or acceptance threshold adjustment.
"""
import argparse
import hashlib
import json
import os
from pathlib import Path
import shutil
import subprocess
import tempfile
import time


def sha(path):
    return hashlib.sha256(path.read_bytes()).hexdigest()


def main():
    parser = argparse.ArgumentParser()
    parser.add_argument("--binary", type=Path, required=True)
    parser.add_argument("--board", type=Path, required=True)
    parser.add_argument("--output", type=Path, required=True)
    args = parser.parse_args()
    binary = args.binary.resolve()
    report = {"scope": "native launch/pass measurement plumbing only; no resource, temporal, lifecycle or overhead acceptance", "binary_sha256": sha(binary), "fixture_sha256": sha(args.board), "runs": []}
    for backend in ["wayland", "x11"]:
        for enabled in [True]:
            for host, flag in [("global", "--open-global-preferences"), ("project", "--open-project-preferences"), ("new", "--open-new-project")]:
                case = Path(tempfile.mkdtemp(prefix=f"datum-s0-{backend}-{host}-"))
                board = case / "board.kicad_pcb"
                shutil.copyfile(args.board, board)
                env = os.environ.copy()
                for key in list(env):
                    if key.startswith("DATUM_DIAGNOSTIC_") or key.startswith("DATUM_GPU_DIAGNOSTIC_"):
                        env.pop(key)
                if backend == "x11":
                    env.pop("WAYLAND_DISPLAY", None)
                env.update(WINIT_UNIX_BACKEND=backend, DATUM_GPU_MEASUREMENTS=str(int(enabled)), DATUM_GUI_LOG=str(case / "diagnostic.log"), XDG_CONFIG_HOME=str(case / "config"), XDG_CACHE_HOME=str(case / "cache"), TMPDIR=str(case))
                command = [str(binary), "--board", str(board), "--window-size", "1280x800"]
                if flag:
                    command.append(flag)
                row = {"backend_requested": backend, "host": host, "measurements_enabled": enabled, "command": command, "temporary_fixture": str(case)}
                with (case / "stdout.log").open("w") as stream:
                    process = subprocess.Popen(command, env=env, stdout=stream, stderr=stream)
                    try:
                        deadline = time.monotonic() + 10
                        while time.monotonic() < deadline:
                            log = (case / "diagnostic.log").read_text() if (case / "diagnostic.log").exists() else ""
                            samples = [json.loads(line.removeprefix("gpu_measurement ")) for line in log.splitlines() if line.startswith("gpu_measurement ")]
                            identities = [line for line in log.splitlines() if "surface identity" in line]
                            expected = 1 if host == "main" else 2
                            if len(identities) == expected and (not enabled or len({sample["host"] for sample in samples}) == expected):
                                time.sleep(2.1)
                                break
                            if process.poll() is not None:
                                break
                            time.sleep(0.05)
                        log = (case / "diagnostic.log").read_text() if (case / "diagnostic.log").exists() else ""
                        samples = [json.loads(line.removeprefix("gpu_measurement ")) for line in log.splitlines() if line.startswith("gpu_measurement ")]
                        row.update(samples=samples, identities=identities, diagnostic=log, stderr=(case / "stdout.log").read_text(), unexpected_exit=process.poll())
                        row["passed"] = all(f"window_backend={backend}" in identity for identity in identities) and process.poll() is None and len(identities) == expected and (len({s["host"] for s in samples}) == expected if enabled else not samples)
                        row["passed"] &= not any(marker in log + row["stderr"] for marker in ["gpu_measurement_incomplete", "gpu_measurement_log_failed", "GPU measurement incomplete"])
                        if enabled:
                            row["passed"] &= all(s["timestamp_period_ns"] > 0 and len(s["raw_ticks"]) == 2 * len(s["passes_ns"]) for s in samples)
                        dialog_samples = [sample for sample in samples if sample["host"] == 2]
                        row["dialog_pass_names"] = [[pair[0] for pair in sample["passes_ns"]] for sample in dialog_samples]
                        row["passed"] &= bool(dialog_samples) and all(names == ["dialog"] for names in row["dialog_pass_names"])
                        report["runs"].append(row)
                        args.output.write_text(json.dumps(report, indent=2) + "\n")
                        print(backend, enabled, host, "PASS" if row["passed"] else "FAIL", flush=True)
                        if not row["passed"]:
                            raise SystemExit("Native measurement check failed; no retry or scope expansion")
                    finally:
                        if process.poll() is None:
                            process.terminate()
                            try:
                                process.wait(timeout=5)
                            except subprocess.TimeoutExpired:
                                process.kill()
                                process.wait()
    assert sha(binary) == report["binary_sha256"]


if __name__ == "__main__":
    main()
