"""Run the exact owner-approved PM042 preparation-bootstrap activation."""

import argparse
import hashlib
from pathlib import Path
import subprocess
import sys


ROOT = Path("/home/bfadmin/Documents/datum-eda")
STORE = ROOT / ".git/datum-wdq"
PREPARATION = STORE / "proposals/wdq-preparation-bootstrap-activation-20260911"
BASE = "354ef94969b878fe8ad978833880755e750ff7e8"
CANDIDATE = "0f232e1026497d4eb404a503ee7c7e3fb7bb6561"
REQUEST_SHA256 = "453f04ca94d5dfe9e3b86d2bf8b56c7acf597ba7572b15d182ef58a80182937b"
RESPONSE = STORE / "owner-responses/preparation-bootstrap-453f04ca.json"
OUTPUT = STORE / "logs/preparation-bootstrap-0f232e10-activation.json"


def git(*arguments):
    return subprocess.check_output(
        ["git", "--no-replace-objects", "--no-optional-locks", *arguments], cwd=ROOT)


def main(argv=None):
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--activate", action="store_true", required=True)
    parser.add_argument("--owner-response-sha256", required=True)
    args = parser.parse_args(argv)
    request = PREPARATION / "activation-request.json"
    assert hashlib.sha256(request.read_bytes()).hexdigest() == REQUEST_SHA256
    assert git("rev-parse", "HEAD").decode().strip() == BASE
    assert not git("status", "--porcelain=v1", "--untracked-files=all")
    assert RESPONSE.is_file() and not RESPONSE.is_symlink()
    assert hashlib.sha256(RESPONSE.read_bytes()).hexdigest() == args.owner_response_sha256
    assert not OUTPUT.exists()
    runtime = STORE / "trusted" / CANDIDATE / "scripts/workflow_delivery_preflight_cli.py"
    command = [
        sys.executable, "-I", "-S", "-B", str(runtime), "--activate",
        "--root", str(ROOT), "--request", str(request),
        "--request-sha256", REQUEST_SHA256, "--output", str(OUTPUT),
        "--owner-response", str(RESPONSE),
        "--owner-response-sha256", args.owner_response_sha256,
    ]
    result = subprocess.run(command, cwd=ROOT, check=False)
    return result.returncode


if __name__ == "__main__":
    raise SystemExit(main())
