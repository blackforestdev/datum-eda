#!/usr/bin/env python3
"""Owner-only activation. Run with --activate after reviewing promotion.md."""
import os
from pathlib import Path
import subprocess
import sys
import tempfile

COMMANDS = '(\nset -euo pipefail\ntrap \'echo "WDQ activation stopped at line $LINENO (exit $?). No later commands ran; report this output." >&2\' ERR\ncd /home/bfadmin/Documents/datum-eda\necho \'WDQ: checking the pinned base and clean worktree; a mismatch means this block is obsolete.\'\ntest "$(git rev-parse HEAD)" = 76152fc7dab4ba0431da991f58d6a5ebc1838763\ntest -z "$(git status --porcelain)"\necho \'WDQ: validating the exact candidate before publication.\'\npython3 /home/bfadmin/Documents/datum-wdq-owner-activation-20260907/candidate/scripts/check_workflow_delivery.py --root /home/bfadmin/Documents/datum-eda --enforce --authority-ref 4ccac933e9ca2b049ea68daf50de16fafe375fd8 --base-ref 76152fc7dab4ba0431da991f58d6a5ebc1838763 --environment-path docs/reviews/workflow-delivery-pilot/proof-artifacts/environment.json --candidate-ref 4ccac933e9ca2b049ea68daf50de16fafe375fd8\ntest "$(git rev-parse HEAD)" = 76152fc7dab4ba0431da991f58d6a5ebc1838763\ntest -z "$(git status --porcelain)"\necho \'WDQ: publishing the verified candidate and installing owner-selected local trust.\'\ngit merge --ff-only 4ccac933e9ca2b049ea68daf50de16fafe375fd8\ngit config --local datum.workflowDeliveryAuthorityRef 4ccac933e9ca2b049ea68daf50de16fafe375fd8\ngit config --local datum.workflowDeliveryBaseRef 76152fc7dab4ba0431da991f58d6a5ebc1838763\ngit config --local datum.workflowDeliveryEnvironmentPath docs/reviews/workflow-delivery-pilot/proof-artifacts/environment.json\ngit config --local datum.workflowDeliveryRunnerPath /home/bfadmin/Documents/datum-wdq-owner-activation-20260907/candidate/scripts/check_workflow_delivery.py\ngit config --local core.hooksPath /home/bfadmin/Documents/datum-wdq-owner-activation-20260907/owner-hooks\npython3 /home/bfadmin/Documents/datum-wdq-owner-activation-20260907/candidate/scripts/check_workflow_delivery.py --root /home/bfadmin/Documents/datum-eda --enforce --authority-ref 4ccac933e9ca2b049ea68daf50de16fafe375fd8 --base-ref 76152fc7dab4ba0431da991f58d6a5ebc1838763 --environment-path docs/reviews/workflow-delivery-pilot/proof-artifacts/environment.json --staged\n/home/bfadmin/Documents/datum-wdq-owner-activation-20260907/owner-hooks/pre-commit\npython3 scripts/project_status.py check\necho \'WDQ: owner activation and blocking verification completed successfully.\'\n)\n'
IDENTITY = 'Candidate: 4ccac933e9ca2b049ea68daf50de16fafe375fd8; Base: 76152fc7dab4ba0431da991f58d6a5ebc1838763'


def main():
    if sys.argv[1:] != ["--activate"]:
        print("No activation performed. Review promotion.md, then run:", flush=True)
        print(f"python3 {Path(__file__).resolve()} --activate", flush=True)
        return 0 if sys.argv[1:] in ([], ["--help"]) else 2
    folder = Path(__file__).resolve().parent
    descriptor, name = tempfile.mkstemp(prefix="activation-", suffix=".log", dir=folder)
    print(f"WDQ activation log: {name}", flush=True)
    with os.fdopen(descriptor, "w", encoding="utf-8") as log:
        print(IDENTITY, flush=True)
        log.write(IDENTITY + "\n")
        log.flush()
        try:
            process = subprocess.Popen(["bash", "-c", COMMANDS], stdout=subprocess.PIPE,
                                       stderr=subprocess.STDOUT, text=True, bufsize=1)
            for line in process.stdout:
                log.write(line)
                log.flush()
                print(line, end="", flush=True)
            code = process.wait()
        except OSError as error:
            message = f"WDQ runner error: {error}"
            log.write(message + "\n")
            print(message, flush=True)
            code = 1
        summary = f"WDQ finished with exit code {code}. Log: {name}"
        log.write(summary + "\n")
        print(summary, flush=True)
    return code


if __name__ == "__main__":
    raise SystemExit(main())
