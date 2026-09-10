"""Retain real terminal-owner case observations; never activate live Datum.

All embedded product/owner records are synthetic inputs. Actual observed output
comes from the frozen validator, selector and authenticated owner-hook processes.
"""

import argparse
from copy import deepcopy
import json
import os
from pathlib import Path
import subprocess
import sys
from unittest.mock import patch
import uuid

HERE = Path(__file__).resolve().parent
ROOT = HERE.parents[4]
PIN = "9924678c39fe5d176e58ebfd8e25ab6feb213f35"
RUNTIME = ROOT / ".git/datum-wdq/proposals/terminal-owner-20260910"
VARIANTS = {
    "pending": None,
    "closeout": None,
    "source": "no promoted scope",
    "selected-requirement": "selected requirement/authority changed",
    "other-step": "work crossed the promoted boundary",
    "unfinished-predecessor": "landed item requires every completion step to be complete",
    "missing-evidence": "completion evidence",
    "document-only": "review/decision evidence",
    "tracker-open": "closed",
    "missing-landing": "landing_commit",
    "invalid-landing": "landing commit",
    "execution": "authorization",
}


def mutate(fixture, variant):
    h, f, item = fixture.h, fixture.h.f, fixture.item
    if variant == "unfinished-predecessor":
        extra = deepcopy(item["completion"]["steps"][0])
        extra.update(id="NEXT-EXTRA", kind="planning", status="pending",
                     completion_evidence=[], depends_on=[],
                     requirement_refs=[{"path": item["governing_docs"][0], "marker": "NEXT-EXTRA"}])
        item["completion"]["steps"].insert(1, extra)
        doc = item["governing_docs"][0]
        f.write(doc, (f.root / doc).read_bytes() + b"\n<!-- REQ:NEXT:NEXT-EXTRA -->\n")
        h.issue["acceptance_criteria"] = "\n".join(
            step["id"] + ": " + step["action"] for step in item["completion"]["steps"])
        h.promote()
        fixture.landing = h.h.authority
    if variant != "pending":
        fixture.close()
    if variant == "source":
        f.write("external/owned/input.py", b"# Unapproved synthetic execution.\n")
    elif variant in ("selected-requirement", "other-step"):
        item["completion"]["steps"][-1 if variant == "selected-requirement" else 0]["action"] = "Unauthorized expansion"
    elif variant == "missing-evidence":
        item["completion"]["steps"][-1]["completion_evidence"] = []
    elif variant == "document-only":
        item["completion"]["steps"][-1]["completion_evidence"][0]["kind"] = "document"
    elif variant == "tracker-open":
        h.issue["status"] = "open"
    elif variant in ("missing-landing", "invalid-landing"):
        item["landing_commit"] = None if variant == "missing-landing" else "0" * 40
    elif variant == "execution":
        item.update(state="ready", authorization="execution")
        item.pop("landing_commit", None)
        h.issue["status"] = "open"
    h.save()


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--destination", type=Path, required=True)
    parser.add_argument("--variant", choices=VARIANTS, action="append")
    args = parser.parse_args()
    if not (sys.flags.isolated and sys.flags.no_site and sys.flags.dont_write_bytecode):
        raise ValueError("run with Python -I -S -B")
    store = args.destination
    parent = ROOT / ".git/datum-wdq/proposals"
    assert store.is_absolute() and store.parent == parent and store.resolve() == store
    assert not store.exists() and not store.is_symlink()
    git_source = lambda *a: subprocess.check_output(
        ["git", "--no-replace-objects", "--no-optional-locks", "-C", str(RUNTIME), *a])
    assert git_source("rev-parse", "HEAD").decode().strip() == PIN
    assert not git_source("status", "--porcelain")
    sys.path.insert(0, str(RUNTIME / "scripts"))
    import test_workflow_delivery_category_boundaries as support
    from workflow_delivery_capture_command import capture_command, save
    from workflow_delivery_capture_state import protected_state
    from workflow_delivery_support_bundle import prepare_bundle, verify_bundle
    from workflow_delivery_support_store import support_locations
    from workflow_delivery_tree import Tree
    from workflow_delivery_authority import authority_sha256
    from workflow_delivery_io import sha256, canonical_json
    from workflow_delivery_test_support import Fixture
    sys.path.insert(0, str(HERE))
    from test_terminal_owner_categories import TerminalOwnerTest
    source = Tree(RUNTIME, revision=PIN)
    contract = source.json("specs/workflow_delivery/rollout.contract.json")
    manifest = source.manifest(contract["input_roots"])
    store.mkdir()
    save(store / "source.json", {"source": PIN, "authority_sha256": authority_sha256(source, contract),
         "input_manifest_sha256": sha256(canonical_json(manifest)), "input_manifest": manifest,
         "capture_script_sha256": sha256(Path(__file__).read_bytes()),
         "fixture_recipe_sha256": sha256((HERE / "test_terminal_owner_categories.py").read_bytes()),
         "variants": args.variant or list(VARIANTS), "activation_performed": False})
    results = []
    environment = {"PATH": os.defpath, "HOME": str(store), "XDG_CONFIG_HOME": str(store),
                   "GIT_CONFIG_NOSYSTEM": "1", "GIT_CONFIG_GLOBAL": os.devnull, "LANG": "C"}
    for variant in args.variant or VARIANTS:
        job = store / variant
        job.mkdir()
        fixture = TerminalOwnerTest()
        try:
            # Constructor substitution chooses an owned retained INPUT location.
            # No validator, clock, evidence or process result is patched.
            with patch.object(support.support, "Fixture", lambda: Fixture(job / "fixture")):
                fixture.setUp()
            mutate(fixture, variant)
            h, f = fixture.h, fixture.h.f
            authority = h.h.authority
            f.git("commit", "--allow-empty", "-qm", "test(workflow): freeze terminal fixture\n\n"
                  + variant + "; synthetic input, not actual product acceptance.")
            candidate = f.git("rev-parse", "HEAD").decode().strip()
            f.git("update-ref", "refs/datum-fixture/candidate", candidate)
            f.git("reset", "--soft", authority)
            nonce = str(uuid.uuid4())
            f.write(".git/datum-wdq-capture-owner", (nonce + "\n").encode())
            bundle = prepare_bundle(f.root, authority)
            locations = support_locations(f.root, authority)
            f.git("config", "--local", "core.hooksPath", str(locations["hooks"]))
            f.git("config", "--local", "datum.workflowDeliveryRunnerPath", str(locations["runner"]))
            f.git("bundle", "create", str(job / "fixture.bundle"), "--all")
            save(job / "fixture.json", {"authority": authority, "candidate": candidate,
                 "nonce": nonce, "bundle_sha256": sha256((job / "fixture.bundle").read_bytes()),
                 "state": protected_state(f.root, ["src", "external", "scripts", ".beads", "specs"]),
                 "synthetic_input_only": True})
            for surface in ("C", "R", "S-check", "S-details", "H"):
                output = job / surface
                observed_environment = dict(environment)
                if surface == "H":
                    command = [str(locations["hook"])]
                    observed_environment["DATUM_WDQ_OBSERVE_PATH"] = str(output / "calls.jsonl")
                else:
                    selector = surface.startswith("S-")
                    script = "project_status.py" if selector else "check_workflow_delivery.py"
                    words = ["--root", str(f.root)]
                    if selector:
                        words += ["--json", surface[2:]] + (["NEXT"] if surface == "S-details" else [])
                    else:
                        words += ["--enforce", "--authority-ref", authority, "--base-ref", authority,
                                  "--environment-path", "requested-environment.json"]
                        words += ["--candidate-ref", candidate] if surface == "R" else ["--staged"]
                    command = [sys.executable, "-I", "-S", "-B",
                        str(RUNTIME / "scripts/workflow_delivery_observe_python.py"),
                        "--script", str(RUNTIME / "scripts" / script), "--source-root", str(RUNTIME),
                        "--output", str(output / "calls.jsonl"), "--function", "main",
                        "--function", "selector_failures", "--", *words]
                case_id = "INFRA-S02-05.terminal-" + variant
                result = capture_command(f.root, output, command, observed_environment,
                    fixture_nonce=nonce, case_id=case_id, surface="S" if surface.startswith("S-") else surface,
                    untracked_roots=["src", "external", "scripts", ".beads", "specs"], timeout=30)
                stdout, stderr = (output / "stdout.bin").read_text(), (output / "stderr.bin").read_text()
                expected = 0 if VARIANTS[variant] is None else 1
                assessment = {"case": case_id, "surface": surface, "returncode": result["returncode"],
                              "expected_status": expected, "expected_reason": VARIANTS[variant],
                              "result_sha256": sha256((output / "result.json").read_bytes())}
                save(job / (surface + "-assessment.json"), assessment)
                print(json.dumps(assessment), flush=True)
                assert result["returncode"] == expected and not result["changed_state_fields"], assessment
                if VARIANTS[variant]:
                    assert VARIANTS[variant] in stdout + stderr, (assessment, stdout, stderr)
                assert bundle == verify_bundle(f.root, authority)
                results.append(assessment)
        finally:
            fixture.doCleanups()  # Explicit-root fixtures stay retained.
    save(store / "complete.json", {"source": PIN, "observations": results,
         "scope": "Retained terminal-owner observations only; complete renewed rollout proof/replay and installation remain due.",
         "activation_performed": False})


if __name__ == "__main__":
    main()
