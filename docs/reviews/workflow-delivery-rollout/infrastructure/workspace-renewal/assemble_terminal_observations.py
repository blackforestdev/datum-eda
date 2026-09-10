"""Assess retained executions without relabeling them as new-source executions."""

from collections import Counter
import json
from pathlib import Path
import sys


def main():
    assert sys.flags.isolated and sys.flags.no_site and sys.flags.dont_write_bytecode
    here = Path(__file__).resolve().parent
    root = here.parents[4]
    proposals = root / ".git/datum-wdq/proposals"
    store = proposals / "terminal-rollout-evidence-20260910"
    runtime = proposals / "terminal-owner-20260910"
    destination = proposals / "terminal-final-assessment-01/observations"
    assert not destination.exists()
    sys.path.insert(0, str(here))
    from assemble_observations import BATCHES, SURFACES, digest, observe, read
    from replay_terminal_owner import JOBS
    from assess_terminal_inputs import FINAL, OBSERVED
    construction = read(destination.parent / "construction.json")
    assert construction["source_commit"] == FINAL and construction["observed_source_commit"] == OBSERVED
    assert not construction["changed_inputs"] and construction["runtime_bytes_equal"]
    rows = []
    def append(capture, row):
        result = observe(runtime, capture, row)
        result["observed_source_commit"] = OBSERVED
        result["assessment_source_commit"] = FINAL
        result["assessment_kind"] = "historical-execution-applicability"
        rows.append(result)
    for name, count in BATCHES.items():
        batch = store / name
        inventory = read(batch / ("observations-complete.json" if name == "s01" else "observations.json"))
        assert inventory["candidate"] == OBSERVED and len(inventory["observations"]) == count
        for row in inventory["observations"]:
            capture = batch / row["directory"] / "capture"
            assessment = read(capture.parent / "assessment.json")
            assert assessment == row["assessment"] and assessment["observation_matches_expected"]
            assert assessment["result_sha256"] == digest(capture / "result.json")
            append(capture, row)
    assert len(rows) == 1118
    for name, count in (("runtime", 52), ("policy-legacy", 75), ("proof-input", 45)):
        batch = store / ("workspace-" + name)
        inventory = read(batch / "renewal-result.json")
        assert inventory["candidate"] == OBSERVED and len(inventory["invocations"]) == count
        for saved in inventory["invocations"]:
            capture = batch / saved["capture_directory"]
            invocation = read(capture / "invocation.json")
            surface = {"S": "S-check", "D": "S-details"}.get(saved["surface"], saved["surface"])
            append(capture, {"case_id": "INFRA-S05-WORKSPACE-" + saved["case"],
                "captured_case_id": invocation["case_id"], "surface": surface, "phase": "compatibility"})
            assert rows[-1]["invocation_id"] == saved["invocation_id"]
            assert rows[-1]["returncode"] == saved["returncode"]
            assert (rows[-1]["returncode"] != 0) == saved["expected_refusal"]
            archive = read(capture / "prestate-archive.json")
            assert digest(Path(archive["path"])) == archive["sha256"]
    assert len(rows) == 1290
    for batch, variants in JOBS.items():
        for variant in variants:
            packet = proposals / ("terminal-owner-capture-" + batch) / variant
            for surface in ("C", "R", "S-check", "S-details", "H"):
                capture = packet / surface
                invocation = read(capture / "invocation.json")
                assessment_path = packet / (surface + "-assessment.json")
                if not assessment_path.exists():
                    assessment_path = capture / "assessment.json"
                assessment = read(assessment_path)
                assert assessment["returncode"] == (0 if variant in ("pending", "closeout") else 1)
                append(capture, {"case_id": invocation["case_id"], "captured_case_id": invocation["case_id"],
                    "surface": surface, "phase": "primary"})
                assert rows[-1]["returncode"] == assessment["returncode"]
    recovery = proposals / "terminal-owner-recovery-01"
    restored = read(recovery / "complete.json")["observations"]
    assert len(restored) == 50
    for saved in restored:
        capture = recovery / saved["variant"] / saved["surface"]
        invocation = read(capture / "invocation.json")
        append(capture, {"case_id": saved["case"], "captured_case_id": invocation["case_id"],
            "surface": saved["surface"], "phase": "recovery"})
        assert rows[-1]["returncode"] == 0 and rows[-1]["result_sha256"] == saved["result_sha256"]
    assert len(rows) == 1400
    assert len({row["invocation_id"] for row in rows}) == len(rows)
    registry, dispatches = {}, {}
    for row in rows:
        entry = registry.setdefault(row["consumer"], {
            "dispatch_key": Path(row["handler_ref"]["path"]).stem + "." + row["handler_ref"]["symbol"],
            "handler_ref": row["handler_ref"], "entry_surfaces": ["project_status"] if row["consumer"] == "selector"
            else ["staged_cli", "candidate_cli", "owner_hook"], "contexts": {}})
        context = "invocation-" + row["invocation_id"]
        enabled = row["handler_invoked"]
        entry["contexts"][context] = {"enabled": enabled,
            "reason": "Historical exact handler call observed; current source bytes equal." if enabled else row["unavailable_reason"]}
        dispatches.setdefault(row["case_id"][:9], []).append({"dispatch_key": entry["dispatch_key"],
            "entry_surface": SURFACES[row["surface"]], "context": context,
            "eligible": enabled, "enabled": enabled, "invoked": enabled,
            "handler_ref": entry["handler_ref"], "unavailable_reason": None if enabled else row["unavailable_reason"],
            "mutation_count": 0})
    result = {"source_commit": FINAL, "observed_source_commit": OBSERVED, "observations": rows,
        "construction_sha256": digest(destination.parent / "construction.json"),
        "scope": "Current assessment of immutable historical producer executions; no new execution or independent replay asserted.",
        "pending": ["Independent applicability/replay", "Full I04 real-roadmap clean/mixed/restored/inventory integration",
                    "Strict promotion and installed verification", "Three actual product adoption cohorts and final acceptance"],
        "excluded": ["Superseded historical real-roadmap executions", "Two failed terminal fixture setups", "Independent attempt01 serialization failure"],
        "activation_performed": False}
    destination.mkdir()
    for name, value in (("dispatch-observations.json", result), ("registry-entries.json", list(registry.values())),
                        ("scenario-dispatches.json", dispatches)):
        with (destination / name).open("x") as stream:
            json.dump(value, stream, indent=2)
            stream.write("\n")
    print(json.dumps({"observations": len(rows), "scenarios": dict(Counter(r["case_id"][:9] for r in rows)),
                      "destination": str(destination), "independent_replay_asserted": False}))


if __name__ == "__main__":
    main()
