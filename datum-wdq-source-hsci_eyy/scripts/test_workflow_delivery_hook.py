#!/usr/bin/env python3
"""Hook sequencing in a private repository; no visual-lane capability is used."""

from pathlib import Path
import subprocess
import unittest

from workflow_delivery_test_support import Fixture


class HookTests(unittest.TestCase):
    def test_both_blocking_gates_precede_nonblocking_delivery_report(self):
        hook = (Path(__file__).parent / "git-hooks/pre-commit").read_bytes()
        for failing, expected in ((None, ["lane", "format", "delivery"]),
                                  ("lane", ["lane"]), ("format", ["lane", "format"])):
            with self.subTest(failing=failing):
                fixture = Fixture()
                self.addCleanup(fixture.close)
                fixture.write("scripts/git-hooks/pre-commit", hook)
                for name, path in (("lane", "check_file_lane_ownership.py"),
                                   ("format", "check_rustfmt.py"),
                                   ("delivery", "check_workflow_delivery.py")):
                    source = (
                        "from pathlib import Path\nimport sys\n"
                        "with Path('calls.log').open('a') as output:\n"
                        f"    output.write({name!r} + '\\n')\n"
                        "assert '--staged' in sys.argv\n"
                        + ("assert '--report-only' in sys.argv\n" if name == "delivery" else "")
                        + f"raise SystemExit({1 if name == failing else 0})\n")
                    fixture.write("scripts/" + path, source.encode())
                result = subprocess.run(["bash", "scripts/git-hooks/pre-commit"], cwd=fixture.root,
                                        capture_output=True, check=False)
                self.assertEqual(result.returncode, 1 if failing else 0, result.stderr.decode())
                self.assertEqual((fixture.root / "calls.log").read_text().splitlines(), expected)


if __name__ == "__main__":
    unittest.main()
