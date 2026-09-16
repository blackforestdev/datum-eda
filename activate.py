"""Run the exact reviewed WDQ lease repair after the owner's exact response."""

import argparse
import hashlib
from pathlib import Path
import subprocess
import sys


ROOT = Path("/home/bfadmin/Documents/datum-eda")
STORE = ROOT / ".git/datum-wdq"
PACKET = STORE / "proposals/lease-renewal-activation-20260915"
REQUEST_SHA256 = "681d51e12c7cd0d5e2a363091136e35dd774b82b4d885aad742bd02311f489be"
CANDIDATE = "61f7b2db1730f0dc7911b5a05f623ad6bf2d1000"
BASE = "19e948ba913595ec3aeffb60660167e419642491"


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--activate", action="store_true", required=True)
    parser.add_argument("--owner-response-sha256", required=True)
    args = parser.parse_args()
    request = PACKET / "activation-request.json"
    response = STORE / "owner-responses/lease-renewal-681d51e1.json"
    output = STORE / "logs/lease-renewal-61f7b2db-activation.json"
    try:
        if hashlib.sha256(request.read_bytes()).hexdigest() != REQUEST_SHA256:
            raise ValueError("The activation request changed; no activation attempted.")
        if response.is_symlink() or not response.is_file():
            raise ValueError("The exact owner response has not been recorded.")
        if hashlib.sha256(response.read_bytes()).hexdigest() != args.owner_response_sha256:
            raise ValueError("The owner response differs from its selected digest.")
        if output.exists():
            raise ValueError("An activation log already exists; inspect it before retrying.")
        head = subprocess.check_output(["git", "rev-parse", "HEAD"], cwd=ROOT).decode().strip()
        dirty = subprocess.check_output(["git", "status", "--porcelain=v1", "--untracked-files=all"], cwd=ROOT)
        if head != BASE or dirty:
            raise ValueError("Main moved or has uncommitted work; no activation attempted.")
        runtime = STORE / "trusted" / CANDIDATE / "scripts/workflow_delivery_preflight_cli.py"
        print(f"WDQ: validating and activating reviewed lease repair {CANDIDATE}.", flush=True)
        print(f"WDQ: durable activation log: {output}", flush=True)
        result = subprocess.run([
            sys.executable, "-I", "-S", "-B", str(runtime), "--activate",
            "--root", str(ROOT), "--request", str(request),
            "--request-sha256", REQUEST_SHA256, "--output", str(output),
            "--owner-response", str(response),
            "--owner-response-sha256", args.owner_response_sha256,
        ], cwd=ROOT, check=False)
        print(f"WDQ: activation finished with exit code {result.returncode}.", flush=True)
        return result.returncode
    except (OSError, ValueError, subprocess.CalledProcessError) as error:
        print(f"WDQ: {error}", file=sys.stderr, flush=True)
        return 1


if __name__ == "__main__":
    raise SystemExit(main())
