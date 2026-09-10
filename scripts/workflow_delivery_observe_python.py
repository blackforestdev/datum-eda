#!/usr/bin/env python3
"""Observe one real Python invocation; emit raw events, never acceptance verdicts.

Invoke with Python -I -S -B. Output is exclusive append-only JSONL. A missing
finish event means interrupted/incomplete capture, never a successful run.
This observes this interpreter only; subprocess and native proof remain separate.
"""

import argparse
import hashlib
import json
import marshal
import os
from pathlib import Path
import runpy
import sys
import time
import uuid


def identity(path):
    path = Path(path).resolve(strict=True)
    with path.open("rb") as stream:
        digest = hashlib.file_digest(stream, "sha256").hexdigest()
    return {"path": str(path), "sha256": digest, "size": path.stat().st_size}


def runtime_files():
    """Observe current modules/maps; do not claim unloaded or child inputs."""
    modules = []
    for name, module in sorted(tuple(sys.modules.items())):
        path = getattr(module, "__file__", None)
        if path:
            try:
                record = {"module": name, **identity(path)}
                cached = getattr(module, "__cached__", None)
                if cached:
                    # -B prevents writes, not reads. Preserve the candidate cache
                    # identity without asserting that this cache was executed.
                    record["cache_candidate"] = (identity(cached) if Path(cached).is_file()
                        else {"path": str(cached), "present": False})
                modules.append(record)
            except (OSError, ValueError) as error:
                modules.append({"module": name, "path": str(path), "error": str(error)})
    mapped = {}
    errors = []
    try:
        for line in Path("/proc/self/maps").read_text().splitlines():
            fields = line.split(maxsplit=5)
            if len(fields) == 6 and fields[5].startswith("/"):
                path = fields[5]
                if path not in mapped:
                    try:
                        mapped[path] = identity(path)
                    except (OSError, ValueError) as error:
                        mapped[path] = {"path": path, "error": str(error)}
    except OSError as error:
        errors.append(str(error))
    return {"modules": modules, "mapped_files": list(mapped.values()), "errors": errors,
            "scope": "current interpreter at observation time; not complete child/tool closure"}


def main(argv=None):
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--script", type=Path, required=True)
    parser.add_argument("--source-root", type=Path, required=True)
    parser.add_argument("--output", type=Path, required=True)
    parser.add_argument("--function", action="append", required=True)
    parser.add_argument("--no-script-path", action="store_true",
                        help="target bootstrap verifies its modules before adding their directory")
    parser.add_argument("arguments", nargs=argparse.REMAINDER)
    args = parser.parse_args(argv)
    if not (sys.flags.isolated and sys.flags.no_site and sys.flags.dont_write_bytecode):
        parser.error("controlled observation requires interpreter flags -I -S -B")
    root = args.source_root.resolve(strict=True)
    script = args.script.resolve(strict=True)
    if not root.is_dir() or not script.is_file() or not script.is_relative_to(root):
        parser.error("script must be an existing file inside the explicit source root")
    arguments = args.arguments[1:] if args.arguments[:1] == ["--"] else args.arguments
    requested = set(args.function)
    invocation = str(uuid.uuid4())
    script_before = identity(script)
    # Exclusive creation refuses overwrite, including a pre-existing symlink.
    with args.output.open("x", encoding="utf-8") as stream:
        def emit(kind, **fields):
            row = {"event": kind, "invocation_id": invocation, "event_id": str(uuid.uuid4()),
                   "pid": os.getpid(), "time_ns": time.time_ns(),
                   "monotonic_ns": time.monotonic_ns(), **fields}
            stream.write(json.dumps(row, sort_keys=True) + "\n")
            stream.flush()

        emit("start", argv=[str(script), *arguments], cwd=str(Path.cwd()),
             stdio_tty={name: stream.isatty() for name, stream in
                        (("stdin", sys.stdin), ("stdout", sys.stdout), ("stderr", sys.stderr))},
             stdin_target=os.readlink("/proc/self/fd/0"),
             source_root=str(root), script=script_before, interpreter=identity(sys.executable),
             python_version=sys.version, observer=identity(__file__),
             flags={key: getattr(sys.flags, key) for key in
                    ("isolated", "no_site", "ignore_environment", "dont_write_bytecode")},
             requested_functions=sorted(requested), runtime=runtime_files())

        def profile(frame, event, arg):
            if event != "call" or frame.f_code.co_name not in requested:
                return
            source = Path(frame.f_code.co_filename).resolve()
            if source.is_relative_to(root) and source.is_file():
                emit("call", function=frame.f_code.co_qualname,
                     first_line=frame.f_code.co_firstlineno, source=identity(source),
                     code_sha256=hashlib.sha256(marshal.dumps(frame.f_code)).hexdigest())

        old_argv, old_path, old_profile = sys.argv, sys.path[:], sys.getprofile()
        outcome = {"kind": "returned", "exit_code": 0}
        try:
            sys.argv = [str(script), *arguments]
            if not args.no_script_path:
                sys.path.insert(0, str(script.parent))
            sys.setprofile(profile)
            runpy.run_path(str(script), run_name="__main__")
        except SystemExit as error:
            outcome = {"kind": "system_exit", "exit_code":
                       error.code if isinstance(error.code, int) else 0 if error.code is None else 1}
            raise
        except BaseException as error:
            outcome = {"kind": "exception", "exception_type": type(error).__name__}
            raise
        finally:
            intact = sys.getprofile() is profile
            sys.setprofile(old_profile)
            sys.argv, sys.path = old_argv, old_path
            emit("finish", outcome=outcome, profiler_still_installed=intact,
                 script_after=identity(script), runtime=runtime_files(),
                 acceptance_asserted=False, child_processes_observed=False)


if __name__ == "__main__":
    main()
