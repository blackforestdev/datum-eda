"""Interrupt only this capture's live child after its actual named call event."""

import json
import os
from pathlib import Path
import signal
import subprocess
import time

from workflow_delivery_io import sha256


def validate_trigger(value):
    if (type(value) is not dict or set(value) != {"trace", "source", "function"}
            or any(type(v) is not str or not v for v in value.values())
            or Path(value["trace"]).name != value["trace"]
            or value["trace"] in (".", "..")
            or not Path(value["source"]).is_absolute()):
        raise ValueError("explicit local trace filename and absolute call source required")


def interrupt_after_call(process, output, trigger, timeout):
    """Return None for normal exit; a missing trigger remains a timeout, not proof."""
    deadline = time.monotonic() + timeout
    trace = output / trigger["trace"]
    while process.poll() is None:
        if trace.exists():
            if trace.is_symlink() or not trace.is_file():
                raise ValueError("regular owned observer trace required")
            raw = trace.read_bytes()
            # The writer flushes complete JSON lines; ignore only a partial tail.
            for line in raw.splitlines(keepends=True):
                if not line.endswith(b"\n"):
                    continue
                row = json.loads(line)
                if (row.get("event") != "call" or row.get("pid") != process.pid
                        or row.get("function") != trigger["function"]
                        or row.get("source", {}).get("path") != trigger["source"]):
                    continue
                if process.poll() is not None:
                    return None
                observed = time.time_ns()
                try:
                    os.killpg(process.pid, signal.SIGTERM)
                except ProcessLookupError:
                    process.wait(timeout=5)
                    return None  # Exited before delivery: no invented interruption.
                record = {"pid": process.pid, "signal": "SIGTERM", "live_observed_ns": observed,
                          "signal_sent_ns": time.time_ns(), "event": row,
                          "event_line_sha256": sha256(line), "event_line_hex": line.hex(),
                          "trace": trigger["trace"], "escalated": False,
                          "acceptance_asserted": False}
                try:
                    process.wait(timeout=5)
                except subprocess.TimeoutExpired:
                    if process.poll() is None:
                        try:
                            os.killpg(process.pid, signal.SIGKILL)
                            record["escalated"] = True
                        except ProcessLookupError:
                            pass
                    process.wait(timeout=5)
                record["returncode"] = process.returncode
                record["finished_ns"] = time.time_ns()
                return record
        if time.monotonic() >= deadline:
            raise subprocess.TimeoutExpired(process.args, timeout)
        time.sleep(0.001)
    return None
