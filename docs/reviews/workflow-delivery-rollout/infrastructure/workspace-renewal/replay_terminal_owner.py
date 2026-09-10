"""Replay or restore the exact retained terminal fixtures; never touch main."""

import argparse
import json
from pathlib import Path
import sys

HERE = Path(__file__).resolve().parent
ROOT = HERE.parents[4]
STORE = ROOT / ".git/datum-wdq/proposals"
RUNTIME = STORE / "terminal-owner-20260910"
JOBS = {"01": ("pending", "closeout"),
        "02": ("source", "selected-requirement", "other-step"),
        "03": ("unfinished-predecessor", "missing-evidence", "document-only",
               "tracker-open", "missing-landing", "invalid-landing"),
        "04": ("execution",)}


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--mode", choices=("replay", "recover"), required=True)
    parser.add_argument("--session", required=True)
    parser.add_argument("--destination", type=Path, required=True)
    args = parser.parse_args()
    if not (sys.flags.isolated and sys.flags.no_site and sys.flags.dont_write_bytecode):
        raise ValueError("run with Python -I -S -B")
    destination = args.destination
    assert destination.parent == STORE and destination.resolve() == destination and not destination.exists()
    sys.path.insert(0, str(RUNTIME / "scripts"))
    from workflow_delivery_capture_state import git, protected_state
    from workflow_delivery_capture_command import capture_command, save
    from workflow_delivery_capture_assertions import capture_envelope, observer_calls
    from workflow_delivery_io import sha256
    from workflow_delivery_support_bundle import verify_bundle
    sys.path.insert(0, str(HERE))
    from capture_terminal_owner import PIN, VARIANTS
    assert git(RUNTIME, "rev-parse", "HEAD").decode().strip() == PIN
    assert not git(RUNTIME, "status", "--porcelain")
    destination.mkdir()
    save(destination / "request.json", {"mode": args.mode, "session": args.session,
         "script_sha256": sha256(Path(__file__).read_bytes()), "source": PIN,
         "scope": "Controlled retained synthetic fixture inputs only; no installed Datum mutation."})
    results = []
    for batch, variants in JOBS.items():
        for variant in variants:
            if args.mode == "recover" and VARIANTS[variant] is None:
                continue
            packet = STORE / ("terminal-owner-capture-" + batch) / variant
            metadata = json.loads((packet / "fixture.json").read_bytes())
            root = packet / "fixture"
            assert root.resolve() == root and (root / ".git").is_dir()
            assert (root / ".git/datum-wdq-capture-owner").read_text().strip() == metadata["nonce"]
            assert sha256((packet / "fixture.bundle").read_bytes()) == metadata["bundle_sha256"]
            assert git(root, "rev-parse", "HEAD").decode().strip() == metadata["authority"]
            support_before = verify_bundle(root, metadata["authority"])
            desired = metadata["authority" if args.mode == "recover" else "candidate"]
            paths = set()
            for revision in (metadata["authority"], metadata["candidate"]):
                paths.update(filter(None, git(root, "ls-tree", "-rz", "--name-only", revision).decode().split("\0")))
            assert paths and all(not p.startswith("/") and ".." not in Path(p).parts
                                 and ".git" not in Path(p).parts for p in paths)
            output_root = destination / variant
            output_root.mkdir()
            input_roots = metadata["state"]["untracked_roots"]
            save(output_root / "before-input-restore.json", protected_state(root, input_roots))
            # Exact retained source paths in this nonce-owned synthetic fixture.
            # Old raw observations and the fixture bundle are outside this tree.
            git(root, "restore", "--source", desired, "--staged", "--worktree", "--", *sorted(paths))
            save(output_root / "input-restore.json", {"fixture": str(root), "source": desired,
                 "paths": sorted(paths), "state": protected_state(root, input_roots),
                 "original_fixture_metadata_sha256": sha256((packet / "fixture.json").read_bytes())})
            for surface in ("C", "R", "S-check", "S-details", "H"):
                original = json.loads((packet / surface / "invocation.json").read_bytes())
                prior_result = json.loads((packet / surface / "result.json").read_bytes())
                if args.mode == "replay":
                    original_state = json.loads((packet / surface / "before.json").read_bytes())
                    restored = protected_state(root, input_roots)
                    fields = ("root", "files", "index_entries_hex", "git_info_exclude", "head",
                              "symbolic_head", "refs", "local_trust", "untracked_roots")
                    for field in fields:
                        assert restored[field] == original_state[field], (variant, surface, field)
                    for field in ("kind", "mode"):
                        assert restored["index"][field] == original_state["index"][field]
                    save(output_root / (surface + "-restoration-check.json"), {
                        "equal_fields": list(fields), "original_index": original_state["index"],
                        "restored_index": restored["index"],
                        "scope": "Same logical fixture inputs; raw index stat-cache metadata observed afresh, following restore_fixture. No cross-run raw-byte identity asserted."})
                output = output_root / surface
                command, environment = original["argv"][:], original["environment"].copy()
                case = original["case_id"] + (".restored" if args.mode == "recover" else "")
                if surface == "H":
                    environment["DATUM_WDQ_OBSERVE_PATH"] = str(output / "calls.jsonl")
                else:
                    command[command.index("--output") + 1] = str(output / "calls.jsonl")
                    if surface == "R":
                        command[command.index("--candidate-ref") + 1] = desired
                result = capture_command(root, output, command, environment,
                    fixture_nonce=metadata["nonce"], case_id=case, surface=original["surface"],
                    untracked_roots=input_roots, timeout=30)
                result, invocation = capture_envelope(output, case, original["surface"])
                if surface == "H":
                    trust = json.loads((output / "before.json").read_bytes())["local_trust"]
                    script = Path(trust["datum.workflowDeliveryRunnerPath"][0]).with_name("workflow_delivery_bootstrap.py")
                    words = ["--root", str(root), "--authority-ref", metadata["authority"],
                             "--base-ref", metadata["authority"], "--environment-path",
                             trust["datum.workflowDeliveryEnvironmentPath"][0], "--runner",
                             trust["datum.workflowDeliveryRunnerPath"][0]]
                else:
                    script = Path(command[command.index("--script") + 1])
                    words = command[command.index("--") + 1:]
                events = observer_calls(output, result, invocation, script, words)
                expected = 0 if args.mode == "recover" else prior_result["returncode"]
                assert result["returncode"] == expected, (variant, surface, result)
                text = (output / "stdout.bin").read_text()
                if expected:
                    assert VARIANTS[variant] in text, (variant, surface, text)
                report = json.loads(text.splitlines()[-1] if surface == "H" else text)
                if surface.startswith("S-"):
                    assert report["ok"] is (expected == 0)
                else:
                    assert report["mode"] == "enforce"
                    assert report["readiness_asserted"] is False and report["acceptance_asserted"] is False
                    assert bool(report["findings"]) is (expected != 0)
                assert verify_bundle(root, metadata["authority"]) == support_before
                row = {"variant": variant, "surface": surface, "case": case,
                       "returncode": result["returncode"], "invocation": result["invocation_id"],
                       "observer_invocation": events[0]["invocation_id"],
                       "original_result_sha256": sha256((packet / surface / "result.json").read_bytes()),
                       "result_sha256": sha256((output / "result.json").read_bytes())}
                assert row["invocation"] != prior_result["invocation_id"]
                save(output_root / (surface + "-assessment.json"), row)
                results.append(row)
                print(json.dumps(row), flush=True)
    save(destination / "complete.json", {"source": PIN, "session": args.session,
         "mode": args.mode, "observations": results,
         "scope": "Retained supplemental terminal evidence, not the complete rollout packet or installation.",
         "activation_performed": False})


if __name__ == "__main__":
    main()
