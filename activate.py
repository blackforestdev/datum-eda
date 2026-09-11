"""Activate the exact independently reviewed installed-preparation recovery."""

import argparse
import hashlib
from pathlib import Path
import subprocess
import sys


ROOT = Path("/home/bfadmin/Documents/datum-eda")
STORE = ROOT / ".git/datum-wdq"
PREPARATION = STORE / "proposals/wdq-installed-recovery-activation-20260911"
BASE = "0f232e1026497d4eb404a503ee7c7e3fb7bb6561"
CANDIDATE = "19e948ba913595ec3aeffb60660167e419642491"
REQUEST_SHA256 = "6d4a57b32778c51f922ea5e1b9cf27ba85e28900ce3bf7456c835d63e46579b9"
FAILED_LOG_SHA256 = "e8f5931f586d2a38ace7c4bf5f812938e85df3107259b38802df926fcb4bbbd2"
RESPONSE = STORE / "owner-responses/installed-recovery-6d4a57b3.json"
OUTPUT = STORE / "logs/installed-recovery-19e948ba-activation.json"


def git(*arguments):
    return subprocess.check_output(
        ["git", "--no-replace-objects", "--no-optional-locks", *arguments], cwd=ROOT)


def digest(path):
    return hashlib.sha256(path.read_bytes()).hexdigest()


def main(argv=None):
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--activate", action="store_true", required=True)
    parser.add_argument("--owner-response-sha256", required=True)
    args = parser.parse_args(argv)
    request = PREPARATION / "activation-request.json"
    failed = STORE / "logs/preparation-bootstrap-0f232e10-activation.json"
    assert digest(request) == REQUEST_SHA256
    assert digest(failed) == FAILED_LOG_SHA256
    assert git("rev-parse", "HEAD").decode().strip() == BASE
    assert not git("status", "--porcelain=v1", "--untracked-files=all")
    assert RESPONSE.is_file() and not RESPONSE.is_symlink()
    assert digest(RESPONSE) == args.owner_response_sha256
    assert not OUTPUT.exists()
    runtime = STORE / "trusted" / CANDIDATE / "scripts/workflow_delivery_preflight_cli.py"
    command = [
        sys.executable, "-I", "-S", "-B", str(runtime), "--activate",
        "--root", str(ROOT), "--request", str(request),
        "--request-sha256", REQUEST_SHA256, "--output", str(OUTPUT),
        "--owner-response", str(RESPONSE),
        "--owner-response-sha256", args.owner_response_sha256,
    ]
    return subprocess.run(command, cwd=ROOT, check=False).returncode


if __name__ == "__main__":
    raise SystemExit(main())
