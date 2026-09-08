"""Observe the current controlled pipes environment, not select or approve it."""

import os
from pathlib import Path
import platform
import shutil
import subprocess
import sys
import time

from workflow_delivery_headless import headless_shape
from workflow_delivery_observe_python import identity, runtime_files


def observe_pipes_environment(*, reproduction_commands):
    """Caller supplies an explicit reproduction recipe, never fake observations.

    The recipe is caller-described, not certified as executed by this function.
    Only interpreter, Git and current stdio/platform facts are observed here.
    Actual gate invocations, subprocess inputs and independent replay remain due.
    """
    if not (sys.flags.isolated and sys.flags.no_site and sys.flags.dont_write_bytecode):
        raise ValueError("environment observation requires Python -I -S -B")
    if os.isatty(0) or os.isatty(1):
        raise ValueError("pipes observer cannot describe TTY or mixed stdio as pipes")
    if (type(reproduction_commands) is not list or not reproduction_commands
            or not all(type(value) is str and value.strip() for value in reproduction_commands)):
        raise ValueError("explicit nonempty reproduction recipe required")
    located = shutil.which("git")
    if located is None:
        raise ValueError("Git executable unavailable")
    git_path = Path(located).resolve(strict=True)
    before = identity(git_path)
    command = [str(git_path), "--version"]
    version = subprocess.run(command, env={"PATH": os.defpath, "LC_ALL": "C",
        "GIT_CONFIG_NOSYSTEM": "1", "GIT_CONFIG_GLOBAL": os.devnull},
        capture_output=True, check=True, timeout=10)
    if identity(git_path) != before:
        raise ValueError("Git executable changed during observation")
    interpreter = identity(sys.executable)
    version_text = version.stdout.decode("utf-8").strip()
    environment = {"schema_version": 2, "kind": "headless", "os": platform.platform(),
        "toolchain": sys.version, "transport": "pipes", "terminal_size": None,
        "input_method": "Process arguments and standard streams",
        "tools": [{"name": "interpreter", "path": interpreter["path"],
                   "sha256": interpreter["sha256"], "version": sys.version},
                  {"name": "git", "path": before["path"], "sha256": before["sha256"],
                   "version": version_text}], "reproduction_commands": reproduction_commands[:]}
    headless_shape(environment)
    return {"schema_version": 1, "kind": "datum.workflow-delivery.environment-observation",
            "observed_at_ns": time.time_ns(), "pid": os.getpid(), "environment": environment,
            "git_version_invocation": {"argv": command, "returncode": version.returncode,
                "stdout_hex": version.stdout.hex(), "stderr_hex": version.stderr.hex()},
            "runtime": runtime_files(), "recipe_source": "explicit caller description, not an execution assertion",
            "selection_authorized": False, "input_closure_complete": False,
            "readiness_asserted": False, "acceptance_asserted": False}
