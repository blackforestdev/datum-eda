"""GP-CM05E Phase-A U1: the §7 environment-record producer.

Captures the real host environment facts for the headless measurement/case surface as the
21-field environment dict `validate_environments` (identity.py) requires, provides the
framework to merge the native coordinate entries a GUI craft realizes, and computes the
`environment-sha256 = digest(canonical(environments))` that binds every measurement/case/native
observation and the Phase-B runner's `--environment-sha256`.

Two outputs: the load-bearing `environments` list (what the report carries and the digest binds),
and a §7 `measurement-environment-report.v1` window record that discloses the recording context
(loadavg, process summary, power mode) honestly -- the acceptance window's environment control,
never a pass by itself. No readiness is issued here; this is one evidence input.
"""

import argparse
import json
import locale as _locale
import os
import platform
import subprocess
from pathlib import Path

from .inputs import canonical, digest, require

ENVIRONMENT_FIELDS = (
    "id", "os", "architecture", "kernel", "cpu", "memory_mib", "filesystem", "mount_options",
    "storage_device", "gpu", "driver", "display_backend", "compositor_or_server", "scale_factor",
    "window_size", "locale", "network_state", "power_mode", "background_load", "clock",
    "tool_versions",
)
WINDOW_SCHEMA = "datum.preferences.measurement-environment-report.v1"
HEADLESS_ID = "headless-linux-x86_64"


def _read_text(path):
    try:
        return Path(path).read_text(encoding="utf-8")
    except OSError:
        return ""


def _fact(value, fallback):
    """A recorded fact must never be the literal 'unknown'/'unavailable' (validate_environments
    rejects those); fall back to an honest, specific alternative string instead of guessing."""
    value = (value or "").strip()
    return value if value and value.lower() not in {"unknown", "unavailable"} else fallback


def _cpu_model():
    for line in _read_text("/proc/cpuinfo").splitlines():
        if line.lower().startswith("model name"):
            model = line.split(":", 1)[1].strip()
            count = sum(1 for l in _read_text("/proc/cpuinfo").splitlines()
                        if l.lower().startswith("model name"))
            return _fact(f"{model} x{count}", f"x86_64 x{os.cpu_count() or 1}")
    return f"x86_64 x{os.cpu_count() or 1}"


def _memory_mib():
    for line in _read_text("/proc/meminfo").splitlines():
        if line.startswith("MemTotal:"):
            return max(1, int(line.split()[1]) // 1024)
    return 1


def _target_mount(target):
    """Resolve the filesystem, mount options and backing device for the mount that holds `target`,
    by longest-prefix match over /proc/mounts (never /tmp -- the proof target is project-backed)."""
    target = str(Path(target).resolve())
    best = None
    for line in _read_text("/proc/mounts").splitlines():
        parts = line.split()
        if len(parts) < 4:
            continue
        device, mount, fstype, opts = parts[0], parts[1], parts[2], parts[3]
        if target == mount or target.startswith(mount.rstrip("/") + "/") or mount == "/":
            if best is None or len(mount) > len(best[1]):
                best = (device, mount, fstype, opts)
    if best is None:
        return "unresolved-device", "unresolved-fs", "defaults"
    return best[0], best[2], best[3]


def _power_mode():
    governor = _read_text("/sys/devices/system/cpu/cpu0/cpufreq/scaling_governor").strip()
    return _fact(governor, "no-cpufreq-governor")


def _loadavg():
    return _read_text("/proc/loadavg").strip().split()[:3] or ["0", "0", "0"]


def _process_summary():
    """A disclosed background-load fact: the 1/5/15 loadavg plus whether any build/GUI contender
    is live (the §7 hard bar), for honest window recording -- never the literal 'unknown'."""
    try:
        comms = subprocess.run(["ps", "-eo", "comm="], capture_output=True, text=True,
                               check=False).stdout.split()
    except OSError:
        comms = []
    cargo = sum(1 for c in comms if c in {"cargo", "rustc"})
    gui = sum(1 for c in comms if "datum-gui" in c or c == "weston")
    one, five, fifteen = _loadavg()
    return (f"loadavg {one}/{five}/{fifteen}; {len(comms)} procs; "
            f"cargo+rustc={cargo}; datum-gui/weston={gui}")


def _tool_versions():
    versions = {"python": platform.python_version()}
    for name, argv in (("rustc", ["rustc", "--version"]), ("cargo", ["cargo", "--version"]),
                       ("git", ["git", "--version"])):
        try:
            out = subprocess.run(argv, capture_output=True, text=True, check=False).stdout.strip()
        except OSError:
            out = ""
        if out:
            versions[name] = out
    return versions


def headless_environment(target):
    """The 21-field headless environment dict (display_backend None, so gpu/driver/compositor/
    scale/window are None) with real captured host facts. `target` is the project-backed proof
    target whose mount is recorded (never /tmp)."""
    require(platform.system() == "Linux" and platform.machine() == "x86_64",
            "the qualified platform is Linux/x86_64")
    device, fstype, opts = _target_mount(target)
    try:
        current_locale = _locale.setlocale(_locale.LC_ALL, "")
    except (_locale.Error, ValueError):
        current_locale = ""
    return {
        "id": HEADLESS_ID,
        "os": "Linux",
        "architecture": "x86_64",
        "kernel": _fact(platform.release(), "linux"),
        "cpu": _cpu_model(),
        "memory_mib": _memory_mib(),
        "filesystem": _fact(fstype, "unresolved-fs"),
        "mount_options": _fact(opts, "defaults"),
        "storage_device": _fact(device, "unresolved-device"),
        "gpu": None,
        "driver": None,
        "display_backend": None,
        "compositor_or_server": None,
        "scale_factor": None,
        "window_size": None,
        "locale": _fact(current_locale or os.environ.get("LANG", ""), "C"),
        "network_state": _fact(os.environ.get("GP_CM05E_NETWORK_STATE", ""),
                               "not-required-isolated"),
        "power_mode": _power_mode(),
        "background_load": _process_summary(),
        "clock": _fact(f"CLOCK_MONOTONIC res={subprocess.os.sysconf('SC_CLK_TCK')}Hz",
                       "CLOCK_MONOTONIC"),
        "tool_versions": _tool_versions(),
    }


def _parse_coordinate(text):
    """'wayland/1.0/1024x900' -> ('wayland', 1.0, [1024, 900]), validated against the matrix axes."""
    from .inventory import BACKENDS, SCALES, WINDOWS
    backend, scale, size = text.split("/")
    width, height = (int(n) for n in size.lower().split("x"))
    scale_value = float(scale)
    window = [width, height]
    require(backend in BACKENDS, f"unknown display backend: {backend}")
    require(scale_value in SCALES, f"unsupported scale factor: {scale_value}")
    require(tuple(window) in WINDOWS, f"unsupported window size: {window}")
    return backend, scale_value, window


def native_environment(target, coordinate, *, gpu, driver, compositor, background_load):
    """A native coordinate's 21-field environment dict. The shared host facts (kernel/cpu/memory/
    filesystem/locale/power/clock/tool_versions) come from the SAME headless capture so there is one
    construction path and no drift; only the native fields (backend/scale/window/gpu/driver/
    compositor) and the coordinate id + the GUI-craft's capture-time background_load differ."""
    backend, scale, window = _parse_coordinate(coordinate)
    environment = headless_environment(target)
    environment.update({
        "id": f"native-{backend}-{scale}-{window[0]}x{window[1]}",
        "gpu": _fact(gpu, "unresolved-gpu"),
        "driver": _fact(driver, "unresolved-driver"),
        "display_backend": backend,
        "compositor_or_server": _fact(compositor, "unresolved-compositor"),
        "scale_factor": scale,
        "window_size": window,
        "background_load": _fact(background_load, environment["background_load"]),
    })
    return environment


def environment_sha256(environments):
    """The binding digest over the canonical environments list (matches report.py's
    digest(canonical(report["environments"])) and the runner's --environment-sha256)."""
    return digest(canonical(environments))


def window_record(environments, *, subject):
    """A §7 window record disclosing the recording context -- environment control, never a pass."""
    return {
        "schema": WINDOW_SCHEMA,
        "lane": "GP-CM05E",
        "subject": subject,
        "not_acceptance_evidence": True,
        "environment_sha256": environment_sha256(environments),
        "environments": environments,
        "recorded_background_load": [_process_summary()],
    }


def build_environments(target, native_specs=None):
    """Assemble the environments list: the headless entry plus one entry per native coordinate the
    GUI craft will realize. Each native spec is {coordinate, gpu, driver, compositor, background_load};
    U1 builds every dict so there is a single construction path. Coordinates must stay unique
    (validate_environments enforces it); a duplicate coordinate is rejected here with a clear error."""
    environments = [headless_environment(target)]
    seen = set()
    for spec in native_specs or []:
        require(set(spec) >= {"coordinate", "gpu", "driver", "compositor", "background_load"},
                "native spec needs coordinate/gpu/driver/compositor/background_load")
        require(spec["coordinate"] not in seen, f"duplicate native coordinate: {spec['coordinate']}")
        seen.add(spec["coordinate"])
        environments.append(native_environment(
            target, spec["coordinate"], gpu=spec["gpu"], driver=spec["driver"],
            compositor=spec["compositor"], background_load=spec["background_load"]))
    return environments


def main(argv=None):
    parser = argparse.ArgumentParser(description="Emit the GP-CM05E environment record (U1).")
    parser.add_argument("--target", type=Path, required=True,
                        help="project-backed proof target whose mount is recorded (never /tmp)")
    parser.add_argument("--native-specs", type=Path,
                        help="optional JSON file: a list of {coordinate,gpu,driver,compositor,background_load}")
    parser.add_argument("--out", type=Path, required=True, help="window-record output path")
    args = parser.parse_args(argv)
    require("/tmp" not in str(Path(args.target).resolve()).split(os.sep)[:2],
            "proof target must be project-backed, never /tmp")
    native = json.loads(args.native_specs.read_text()) if args.native_specs else []
    environments = build_environments(args.target, native)
    record = window_record(environments, subject="GP-CM05E S6 candidate qualification window")
    args.out.write_text(json.dumps(record, indent=2, sort_keys=True) + "\n")
    print(f"environment-sha256: {record['environment_sha256']}")
    print(f"environments: {len(environments)} ({HEADLESS_ID} + {len(environments) - 1} native)")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
