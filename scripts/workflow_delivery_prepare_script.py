"""Package the exact reviewed shell block as a logged owner-run Python script."""

from workflow_delivery_prepare_handoff import promotion_markdown


def activation_script(root, result):
    # Keep one command source: the runnable artifact contains the very same
    # guarded commands shown in the owner handoff, not a second mutation path.
    block = promotion_markdown(root, result).split("```bash\n")[2].split("```")[0]
    return '''#!/usr/bin/env python3
"""Owner-only activation. Run with --activate after reviewing promotion.md."""
import os
from pathlib import Path
import subprocess
import sys
import tempfile

COMMANDS = ''' + repr(block) + '''
IDENTITY = ''' + repr("Candidate: " + result["candidate"] + "; Base: " + result["base"]) + '''


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
        log.write(IDENTITY + "\\n")
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
            log.write(message + "\\n")
            print(message, flush=True)
            code = 1
        summary = f"WDQ finished with exit code {code}. Log: {name}"
        log.write(summary + "\\n")
        print(summary, flush=True)
    return code


if __name__ == "__main__":
    raise SystemExit(main())
'''
