"""External Linux DRM client-memory snapshots; no sums or atomic-peak claims."""
import argparse
import json
import os
from pathlib import Path
import time

MAX_FDS = 4096
MAX_TEXT = 65536
MEMORY = ("total", "shared", "resident", "purgeable", "active", "memory")


def parse(text):
    raw = {}
    for line in text.splitlines():
        if not line.startswith("drm-"):
            continue
        key, separator, value = line.partition(":")
        if not separator or key in raw or any(c.isspace() for c in key):
            raise ValueError("malformed or duplicate DRM key")
        raw[key] = value.strip()
    if not raw:
        return None
    if not raw.get("drm-driver"):
        raise ValueError("DRM fields without driver identity")
    client = raw.get("drm-client-id")
    if client is not None and (not client.isascii() or not client.isdecimal()):
        raise ValueError("invalid DRM client identity")
    memory = {}
    for key, value in raw.items():
        # Utilization total-cycles shares the memory total prefix, but is not bytes.
        if key.startswith("drm-total-cycles-"):
            continue
        kind = next((k for k in MEMORY if key.startswith("drm-" + k + "-")), None)
        if kind is None:
            continue
        region = key[len(kind) + 5:]
        fields = value.split()
        if (not region or len(fields) not in (1, 2)
                or not fields[0].isascii() or not fields[0].isdecimal()):
            raise ValueError("invalid DRM memory counter: " + key)
        unit = fields[1] if len(fields) == 2 else "bytes"
        if unit not in ("bytes", "KiB", "MiB"):
            raise ValueError("unsupported DRM memory unit: " + unit)
        # Deprecated drm-memory is retained separately, never added to resident.
        memory[key] = int(fields[0]) * {"bytes": 1, "KiB": 1024, "MiB": 1048576}[unit]
    return {"driver": raw["drm-driver"], "pdev": raw.get("drm-pdev"),
            "client_id": int(client) if client is not None else None,
            "memory_bytes": memory, "raw_drm": raw}


def read_at(directory, name):
    fd = os.open(name, os.O_RDONLY | os.O_CLOEXEC, dir_fd=directory)
    with os.fdopen(fd, "rb") as stream:
        data = stream.read(MAX_TEXT + 1)
    if len(data) > MAX_TEXT:
        raise ValueError("proc record exceeds bounded read")
    return data.decode("utf-8")


def starttime(text):
    # comm may contain spaces and closing parentheses; fields begin after its last one.
    end = text.rfind(") ")
    if end < 0:
        raise ValueError("malformed process stat")
    fields = text[end + 2:].split()
    return int(fields[19])  # field22, relative to field3(state)


def collect(pids, proc_root=Path("/proc")):
    result = {"started_ns": time.monotonic_ns(), "processes": [], "clients": [],
              "unidentified": [], "errors": [], "atomic_snapshot": False,
              "semantics": "First fd sample per driver/pdev/client identity; aliases retained, never summed. Memory classes and client totals overlap. Missing counters are unknown, not zero. Only listed fd names and supplied PIDs are observed; new/short-lived descriptors and omitted descendants are unknown. PID/client lifecycle epochs and instantaneous peaks are not established. A missing client does not prove zero retained memory."}
    clients = {}
    for pid in sorted(set(pids)):
        proc = fds = None
        process = {"pid": pid}
        result["processes"].append(process)
        try:
            # Pin this proc directory so PID reuse cannot redirect later reads.
            proc = os.open(proc_root / str(pid), os.O_RDONLY | os.O_DIRECTORY | os.O_CLOEXEC)
            process["starttime_ticks"] = starttime(read_at(proc, "stat"))
            fds = os.open("fdinfo", os.O_RDONLY | os.O_DIRECTORY | os.O_CLOEXEC, dir_fd=proc)
            names = sorted(os.listdir(fds), key=int)
            if len(names) > MAX_FDS:
                raise ValueError("fd count exceeds collector bound")
            process["enumerated_fds"] = len(names)
            for name in names:
                began = time.monotonic_ns()
                text = None
                try:
                    text = read_at(fds, name)
                    item = parse(text)
                    if item is None:
                        continue
                    sample = {"pid": pid, "starttime_ticks": process["starttime_ticks"],
                              "fd": int(name), "started_ns": began,
                              "finished_ns": time.monotonic_ns(), **item}
                    if item["client_id"] is None:
                        result["unidentified"].append(sample)
                        continue
                    key = (item["driver"], item["pdev"], item["client_id"])
                    if key not in clients:
                        clients[key] = {"first_sample": sample, "other_fd_observations": []}
                    else:
                        clients[key]["other_fd_observations"].append(sample)
                except (OSError, ValueError, UnicodeError) as error:
                    result["errors"].append({"pid": pid, "fd": name, "error": str(error),
                                             "raw_text": text})
            if starttime(read_at(proc, "stat")) != process["starttime_ticks"]:
                raise ValueError("process identity changed during observation")
            process["identity_verified_after_read"] = True
        except (OSError, ValueError, UnicodeError, IndexError) as error:
            result["errors"].append({"pid": pid, "error": str(error)})
        finally:
            if fds is not None:
                os.close(fds)
            if proc is not None:
                os.close(proc)
    result["clients"] = list(clients.values())
    result["finished_ns"] = time.monotonic_ns()
    result["complete_enumeration"] = not result["errors"] and not result["unidentified"]
    # Some memory fields per identified client, not complete residency coverage.
    result["memory_counters_available"] = bool(clients) and all(
        c["first_sample"]["memory_bytes"] for c in clients.values())
    return result


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--pid", type=int, action="append", required=True)
    parser.add_argument("--output", type=Path, required=True)
    args = parser.parse_args()
    if any(pid <= 0 for pid in args.pid):
        parser.error("PIDs must be positive")
    result = collect(args.pid)
    with args.output.open("x") as stream:
        json.dump(result, stream, indent=2)
        stream.write("\n")
    return 0 if result["complete_enumeration"] else 1


if __name__ == "__main__":
    raise SystemExit(main())
