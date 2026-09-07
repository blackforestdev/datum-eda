#!/usr/bin/env python3
"""Measure the bounded GP-CM05 release paths without retaining user state."""

from __future__ import annotations

import argparse
import errno
import json
import os
from pathlib import Path
import pty
import shutil
import statistics
import tempfile
import time
import uuid


ROOT = Path(__file__).resolve().parents[1]


def percentile(values: list[float], fraction: float) -> float:
    ordered = sorted(values)
    return ordered[max(0, int(len(ordered) * fraction) - 1)]


def summary(samples: list[tuple[float, int]]) -> dict[str, float | int]:
    elapsed = [sample[0] for sample in samples]
    return {
        "samples": len(samples),
        "p50_ms": round(statistics.median(elapsed), 3),
        "p95_ms": round(percentile(elapsed, 0.95), 3),
        "max_ms": round(max(elapsed), 3),
        "max_rss_kib": max(sample[1] for sample in samples),
    }


def run(binary: Path, arguments: list[str], environment: dict[str, str]) -> tuple[float, int]:
    started = time.perf_counter()
    pid = os.fork()
    if pid == 0:
        null = os.open(os.devnull, os.O_WRONLY)
        os.dup2(null, 1)
        os.dup2(null, 2)
        os.execve(str(binary), [str(binary), *arguments], environment)
    _, status, usage = os.wait4(pid, 0)
    if status != 0:
        raise RuntimeError(f"release command failed with wait status {status}: {arguments!r}")
    return ((time.perf_counter() - started) * 1000, usage.ru_maxrss)


def confirmed_set(
    binary: Path,
    environment: dict[str, str],
    expected: dict[str, object],
    index: int,
) -> tuple[float, int, dict[str, object]]:
    request_id = uuid.uuid5(uuid.NAMESPACE_URL, f"datum:gp-cm05:write:{index}")
    value = "true" if index % 2 == 0 else "false"
    arguments = [
        "--format",
        "json",
        "preferences",
        "set",
        "datum.accessibility.reduced_motion",
        "--value-json",
        value,
        "--expected",
        json.dumps(expected, separators=(",", ":")),
        "--reason",
        "GP-CM05 release measurement",
        "--request-id",
        str(request_id),
    ]
    started = time.perf_counter()
    pid, descriptor = pty.fork()
    if pid == 0:
        os.execve(str(binary), [str(binary), *arguments], environment)
    output = bytearray()
    confirmed = False
    while True:
        try:
            chunk = os.read(descriptor, 4096)
        except OSError as error:
            if error.errno == errno.EIO:
                break
            raise
        if not chunk:
            break
        output.extend(chunk)
        if not confirmed and b"Type APPLY to confirm:" in output:
            os.write(descriptor, b"APPLY\n")
            confirmed = True
    _, status, usage = os.wait4(pid, 0)
    os.close(descriptor)
    if status != 0 or not confirmed:
        raise RuntimeError(output.decode(errors="replace"))
    rendered = output.decode().replace("\r", "")
    start = rendered.find('{\n  "ok"')
    if start < 0:
        raise RuntimeError("confirmed mutation did not return a JSON product envelope")
    response = json.loads(rendered[start:])
    return ((time.perf_counter() - started) * 1000, usage.ru_maxrss, response)


def measure(binary: Path, count: int) -> dict[str, object]:
    temporary = Path(tempfile.mkdtemp(prefix="datum-gp-cm05-release-", dir="/tmp"))
    try:
        configuration = temporary / "config"
        configuration.mkdir()
        environment = os.environ.copy()
        environment["XDG_CONFIG_HOME"] = str(configuration)

        run(binary, ["--format", "json", "preferences", "describe"], environment)
        query = [
            run(binary, ["--format", "json", "preferences", "describe"], environment)
            for _ in range(count)
        ]

        factory: list[tuple[float, int]] = []
        global_no_file: list[tuple[float, int]] = []
        validation: list[tuple[float, int]] = []
        project_sizes: list[int] = []
        for index in range(count):
            mode = "factory" if index < count // 2 else "global"
            destination = temporary / f"project-{mode}-{index}"
            measured = run(
                binary,
                [
                    "project",
                    "new",
                    "--name",
                    f"Acceptance {index}",
                    "--project-id",
                    str(uuid.uuid5(uuid.NAMESPACE_URL, f"datum:gp-cm05:project:{index}")),
                    "--request-id",
                    str(uuid.uuid5(uuid.NAMESPACE_URL, f"datum:gp-cm05:request:{index}")),
                    "--units-source",
                    mode,
                    "--json",
                    str(destination),
                ],
                environment,
            )
            (factory if mode == "factory" else global_no_file).append(measured)
            validation.append(run(binary, ["project", "validate", str(destination)], environment))
            project_sizes.append(
                sum(path.stat().st_size for path in destination.rglob("*") if path.is_file())
            )
        repository_after_reads = (configuration / "datum/preferences").exists()

        expected: dict[str, object] = {"kind": "missing"}
        mutations: list[tuple[float, int]] = []
        for index in range(count):
            elapsed, rss, response = confirmed_set(binary, environment, expected, index)
            mutations.append((elapsed, rss))
            expected = {"kind": "generation", "generation": response["context"]["generation"]}
        repository = configuration / "datum/preferences"
        repository_files = [path for path in repository.rglob("*") if path.is_file()]

        return {
            "schema": "datum-global-preferences-release-measurement-v1",
            "binary": str(binary),
            "samples_per_path": count,
            "host": {
                "kernel": os.uname().release,
                "architecture": os.uname().machine,
                "cpu": next(
                    (
                        line.split(":", 1)[1].strip()
                        for line in Path("/proc/cpuinfo").read_text(encoding="utf-8").splitlines()
                        if line.startswith("model name")
                    ),
                    "unknown",
                ),
            },
            "measurements": {
                "release_query": summary(query),
                "project_genesis_factory": summary(factory),
                "project_genesis_global_no_file": summary(global_no_file),
                "project_validation": summary(validation),
                "durable_mutation": summary(mutations),
                "native_project_bytes": {"minimum": min(project_sizes), "maximum": max(project_sizes)},
                "preference_repository": {
                    "created_by_read_or_factory_paths": repository_after_reads,
                    "files_after_mutations": len(repository_files),
                    "bytes_after_mutations": sum(path.stat().st_size for path in repository_files),
                    "generations": count,
                },
            },
        }
    finally:
        shutil.rmtree(temporary)


def main() -> int:
    parser = argparse.ArgumentParser()
    parser.add_argument("--binary", type=Path, default=ROOT / "target/release/datum-eda")
    parser.add_argument("--samples", type=int, default=20)
    arguments = parser.parse_args()
    binary = arguments.binary.resolve()
    if not binary.is_file() or not os.access(binary, os.X_OK):
        parser.error(f"release binary is not executable: {binary}")
    if arguments.samples < 10 or arguments.samples % 2:
        parser.error("--samples must be an even integer of at least 10")
    print(json.dumps(measure(binary, arguments.samples), indent=2))
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
