#!/usr/bin/env python3
"""Shared CLI invocation and store-fingerprint I/O for the GP-CM05E case recipes.

Extracted from ``preferences_case.py`` at the PM022 ceiling crossing that the
upgrade/migration-required family required: the recipe file was already over the
700-line limit as legacy debt, and this real ownership extraction ratchets it
back down. These are the low-level helpers every case scenario shares -- running
the real candidate CLI against an isolated config root, parsing typed envelopes,
driving a human APPLY under a pty, and disk-level store fingerprints -- with no
scenario logic. Imported directly by the recipe (no forwarding wrappers), and the
process/env/cleanup behaviour is preserved verbatim.
"""
import json
import os
import shutil
import subprocess
import sys
import tempfile
from pathlib import Path

# Reuse the governed foreground-TTY helper for the rare case family that must perform a human APPLY
# mutation (a write family's later edit); importing the sibling package keeps the pty logic in ONE
# audited place rather than duplicating pty.fork here, and ships with the recipe (reproducible replay).
sys.path.insert(0, str(Path(__file__).resolve().parents[2]))
from global_preferences_acceptance.process import execute as _execute_tty  # noqa: E402


def _require_binary(variable):
    value = os.environ.get(variable)
    if not value or not Path(value).is_file():
        raise SystemExit(f"{variable} must point at the candidate binary for this surface")
    return value


def _work_dir(prefix):
    """Return (work_root, cleanup). When the producer provides DATUM_EDA_WORK_ROOT the recipe runs
    there and LEAVES it so the producer can capture the real post-run state (mutating families); the
    path is carried out-of-band exactly like DATUM_EDA_CLI, so the committed fixture argv stays stable.
    Otherwise it uses a private temp dir it removes itself (read-only families need no post-state)."""
    provided = os.environ.get("DATUM_EDA_WORK_ROOT")
    if provided:
        work = Path(provided)
        work.mkdir(parents=True, exist_ok=True)
        return work, (lambda: None)
    work = Path(tempfile.mkdtemp(prefix=prefix))
    return work, (lambda: shutil.rmtree(work, ignore_errors=True))


def _run_cli(binary, args, *, work):
    """Run the real CLI against an isolated config root; capture typed JSON. stdin is /dev/null so a
    mutation with no foreground TTY refuses deterministically (human_presence_required) rather than
    blocking on input."""
    env = {
        **os.environ,
        "HOME": str(work),
        "XDG_CONFIG_HOME": str(work / "config"),
    }
    env.pop("DATUM_GUI_PREFERENCES_PATH", None)
    return subprocess.run(
        [binary, "--format", "json", *args],
        capture_output=True,
        text=True,
        stdin=subprocess.DEVNULL,
        env=env,
    )


def _run_and_parse(binary, work, argv):
    """Run one CLI invocation and parse its JSON stdout REGARDLESS of exit code (refusals exit 2 but
    still emit a typed error envelope); return (returncode, envelope-or-None)."""
    proc = _run_cli(binary, argv, work=work)
    try:
        return proc.returncode, json.loads(proc.stdout)
    except (json.JSONDecodeError, TypeError):
        return proc.returncode, None


def _run_apply(binary, work, argv):
    """Run a `preferences set/reset` under a pty, auto-submitting APPLY at the foreground-TTY prompt;
    return (committed: bool, envelope-or-None). Never retries."""
    env = {k: v for k, v in os.environ.items()
           if k not in ("DATUM_GUI_PREFERENCES_PATH", "DATUM_ENGINE_SOCKET", "EDA_ENGINE_SOCKET")}
    env["HOME"] = str(work)
    env["XDG_CONFIG_HOME"] = str(work / "config")
    run = _execute_tty(binary, ["--format", "json", *argv], env, timeout=30, confirmation=True)
    text = run["stdout"].decode("utf-8", "replace").replace("\r", "")
    start = text.find('{\n  "ok"')
    try:
        envelope = json.loads(text[start:]) if start >= 0 else None
    except json.JSONDecodeError:
        envelope = None
    committed = bool(run["exit_code"] == 0 and run["confirmed"] and not run["timed_out"]
                     and isinstance(envelope, dict) and envelope.get("ok") is True)
    return committed, envelope


def _no_project_written(work):
    """A read-only query writes no native Project anywhere under the work root."""
    return not any(path.name == "project.json" for path in work.rglob("*"))


def _query_context(binary, work, argv):
    """Run one read-only query verb; return its parsed envelope (or None) without writing a store."""
    proc = _run_cli(binary, ["preferences", *argv], work=work)
    if proc.returncode != 0:
        return None
    try:
        return json.loads(proc.stdout)
    except json.JSONDecodeError:
        return None


def _store_absent(work):
    return not (work / "config" / "datum" / "preferences").exists()


def _store_digest_map(repo):
    """Map every regular file under the repository to its sha256 -- a disk-level fingerprint used to
    prove the store bytes are unchanged (no silent repair) across a read."""
    import hashlib
    fingerprint = {}
    if repo.exists():
        for path in sorted(repo.rglob("*")):
            if path.is_file() and not path.is_symlink():
                fingerprint[path.relative_to(repo).as_posix()] = hashlib.sha256(path.read_bytes()).hexdigest()
    return fingerprint
