"""Exercise frozen repaired source in child processes, not patched entrypoints.

Synthetic development regression only; not a typed producer/replay packet.
"""

import json
from pathlib import Path
import subprocess
import sys


PIN = "9924678c39fe5d176e58ebfd8e25ab6feb213f35"
HERE = Path(__file__).resolve().parent
ROOT = HERE.parents[4]
RUNTIME = ROOT / ".git/datum-wdq/proposals/terminal-owner-20260910"


def main():
    if not (sys.flags.isolated and sys.flags.no_site and sys.flags.dont_write_bytecode):
        raise ValueError("run with Python -I -S -B")
    head = subprocess.check_output(["git", "--no-replace-objects", "-C", str(RUNTIME),
                                    "rev-parse", "HEAD"], text=True).strip()
    assert head == PIN
    sys.path.insert(0, str(RUNTIME / "scripts"))
    # Load the source-bound input recipe dependencies before importing the
    # historical regression harness. Only its fixture setup/close methods run;
    # its replacement-validator helpers are never invoked.
    import test_workflow_delivery_category_boundaries as support
    import workflow_delivery_categories as categories
    assert Path(categories.__file__).parent == RUNTIME / "scripts"
    sys.path.insert(0, str(HERE))
    from test_terminal_owner_categories import TerminalOwnerTest

    observations = []
    for variant in ("pending", "closeout", "source-refusal", "document-only-refusal"):
        fixture = TerminalOwnerTest()
        try:
            fixture.setUp()
            h, f = fixture.h, fixture.h.f
            if variant != "pending":
                fixture.close()
            if variant == "source-refusal":
                f.write("external/owned/input.py", b"unapproved execution\n")
                h.save()
            elif variant == "document-only-refusal":
                fixture.item["completion"]["steps"][-1]["completion_evidence"][0]["kind"] = "document"
                h.save()
            f.git("commit", "--allow-empty", "-qm",
                  "test(workflow): retain synthetic terminal input\n\n"
                  "Exercise " + variant + "; not actual product or owner evidence.")
            candidate = f.git("rev-parse", "HEAD").decode().strip()
            # Re-stage the exact same governed transaction for C, after retaining
            # an exact candidate for R. The base remains the promoted fixture.
            if variant != "pending":
                f.git("reset", "--soft", h.h.authority)
            for surface in ("C", "R", "S-check", "S-details"):
                selector = surface.startswith("S-")
                script = "project_status.py" if selector else "check_workflow_delivery.py"
                args = ["--root", str(f.root)]
                if selector:
                    args += ["--json", surface[2:]]
                    if surface == "S-details":
                        args += ["NEXT"]
                else:
                    args += ["--enforce", "--authority-ref", h.h.authority,
                             "--base-ref", h.h.authority,
                             "--environment-path", "requested-environment.json"]
                    args += ["--candidate-ref", candidate] if surface == "R" else ["--staged"]
                trace = f.root / ".git" / ("terminal-calls-" + surface + ".jsonl")
                command = [sys.executable, "-I", "-S", "-B",
                           str(RUNTIME / "scripts/workflow_delivery_observe_python.py"),
                           "--script", str(RUNTIME / "scripts" / script),
                           "--source-root", str(RUNTIME), "--output", str(trace),
                           "--function", "main", "--function", "selector_failures", "--", *args]
                before = f.snapshot()
                result = subprocess.run(command, capture_output=True, text=True, timeout=30)
                assert f.snapshot() == before, (variant, surface, "state changed")
                expected = 1 if variant.endswith("refusal") else 0
                observation = {"variant": variant, "surface": surface, "argv": command,
                               "returncode": result.returncode, "stdout": result.stdout,
                               "stderr": result.stderr, "fixture_file_bytes_unchanged": True}
                observations.append(observation)
                assert result.returncode == expected, observation
                reason = {"source-refusal": "no promoted scope",
                          "document-only-refusal": "review/decision evidence"}.get(variant)
                if reason:
                    assert reason in result.stdout + result.stderr, observation
                calls = [json.loads(line) for line in trace.read_text().splitlines()]
                assert any(row.get("event") == "call" and row.get("function") == "main"
                           for row in calls), observation
        finally:
            fixture.doCleanups()
    print(json.dumps({"source": PIN, "observations": observations,
                      "scope": "Synthetic child-process regression; compares non-Git file bytes, not complete protected Git state. Fixture traces are cleaned afterward. No hook, typed proof, independent replay or installation claim.",
                      "activation_performed": False}, indent=2))


if __name__ == "__main__":
    main()
