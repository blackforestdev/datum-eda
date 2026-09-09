"""Real staged/candidate/selector refusal and recovery on synthetic inputs."""

import os
import json
from pathlib import Path
import tempfile
import unittest

from workflow_delivery_capture_cases import CASES, CLI_ERROR_CASES, run_case
from workflow_delivery_capture_assertions import evaluate_capture
from workflow_delivery_capture_mutations import TEXT_CASES, RECOVERY_CASE
from workflow_delivery_capture_mutations import TRUST_CASES
from workflow_delivery_capture_hook import run_hook_case
from workflow_delivery_capture_fixture import prepare_fixture
from workflow_delivery_io import canonical_json, sha256
from workflow_delivery_environment_cases import PREPARED_VARIANTS
from workflow_delivery_capture_history import BASE_CASES, CURRENT_HISTORY_CASE, HISTORY_CASES
from workflow_delivery_capture_snapshots import SNAPSHOT_CASES
from workflow_delivery_interruption_case import INTERRUPTION_CASE
from workflow_delivery_review_input import REPORT_CASES, REPORT_FIXTURE, REPORT_ONLY_CASE


class CaptureCasesTest(unittest.TestCase):
    def test_candidate_cannot_expand_trusted_environment_membership(self):
        with tempfile.TemporaryDirectory() as temporary:
            store = Path(temporary)
            packet = store / "packet"
            recipe = prepare_fixture(store / "baseline", packet)
            environment = {"PATH": os.defpath, "HOME": str(store), "XDG_CONFIG_HOME": str(store),
                           "GIT_CONFIG_NOSYSTEM": "1", "GIT_CONFIG_GLOBAL": os.devnull, "LANG": "C"}
            runtime = Path(__file__).resolve().parents[1]
            identities = set()
            for surface in ("C", "R", "S-check", "S-details", "H"):
                with self.subTest(surface=surface):
                    for phase, case in (("refusal", "INFRA-S06-11.authority-membership"),
                                        ("recovery", "INFRA-S01-01")):
                        output = store / (surface + phase)
                        result = (run_hook_case(packet, output, environment, case_id=case)
                                  if surface == "H" else run_case(packet, output, runtime, environment,
                                                                 case_id=case, surface=surface))
                        self.assertTrue(result["observation_matches_expected"])
                        self.assertFalse(result["acceptance_asserted"])
                        self.assertFalse(result["complete_matrix"])
                        self.assertNotIn(result["invocation_id"], identities)
                        identities.add(result["invocation_id"])
                        before = json.loads((output / "capture/before.json").read_bytes())
                        self.assertEqual([recipe["head"]], before["local_trust"]["datum.workflowDeliveryAuthorityRef"])
                        policy = json.loads((output / "fixture/specs/workflow_delivery_policy.json").read_bytes())
                        self.assertEqual(["TASK", "NEXT"] if phase == "refusal" else ["TASK"],
                                         [row["frontier_key"] for row in policy["enrolled"]])
            self.assertEqual(10, len(identities))

    def test_explicit_cases_refuse_for_intended_reason_and_fresh_baseline_recovers(self):
        with tempfile.TemporaryDirectory() as temporary:
            store = Path(temporary)
            packet = store / "packet"
            prepare_fixture(store / "baseline", packet)
            environment = {"PATH": os.defpath, "HOME": str(store), "XDG_CONFIG_HOME": str(store),
                "GIT_CONFIG_NOSYSTEM": "1", "GIT_CONFIG_GLOBAL": os.devnull,
                "PYTHONDONTWRITEBYTECODE": "1", "LANG": "C"}
            runtime = Path(__file__).resolve().parents[1]
            invocations = set()
            ordinary_cases = {key: value for key, value in CASES.items()
                              if key not in PREPARED_VARIANTS | HISTORY_CASES.keys() | BASE_CASES.keys() | SNAPSHOT_CASES.keys() | CLI_ERROR_CASES | REPORT_CASES | TRUST_CASES.keys() | {INTERRUPTION_CASE, CURRENT_HISTORY_CASE}}
            for case_id in ordinary_cases:
                for surface in ("C", "R", "S-check", "S-details"):
                    with self.subTest(case_id=case_id, surface=surface):
                        output = store / (case_id + "-" + surface)
                        result = run_case(packet, output, runtime, environment,
                                          case_id=case_id, surface=surface)
                        self.assertTrue(result["observation_matches_expected"])
                        self.assertFalse(result["complete_matrix"])
                        self.assertNotIn(result["invocation_id"], invocations)
                        invocations.add(result["invocation_id"])
                        if case_id in TEXT_CASES:
                            self.assertTrue(result["text_evidence"]["noninteractive_descriptors_observed"])
                            self.assertTrue(result["text_evidence"]["no_color_escapes"])
                            self.assertFalse(result["text_evidence"]["gui_accessibility_asserted"])
                        if case_id == RECOVERY_CASE:
                            self.assertFalse(result["recovery"]["authority_refreshed"])
                            self.assertNotEqual(result["invocation_id"], result["recovery"]["prior_invocation_id"])
                            self.assertTrue((output / result["recovery"]["prior_result_path"]).is_file())
                        script = runtime / "scripts" / ("project_status.py" if surface.startswith("S-")
                                                        else "check_workflow_delivery.py")
                        if CASES[case_id][1] is not None:
                            with self.assertRaisesRegex(ValueError, "precise diagnostic"):
                                evaluate_capture(output / "capture", case_id=case_id,
                                    surface="S" if surface.startswith("S-") else surface,
                                    expected_code="WDQ-WRONG-FAILURE", script=script)
                            for extra, message in (({"expected_path": "wrong-input.json"}, "input path"),
                                                   ({"expected_detail": "wrong cause"}, "diagnostic cause")):
                                with self.assertRaisesRegex(ValueError, message):
                                    evaluate_capture(output / "capture", case_id=case_id,
                                        surface="S" if surface.startswith("S-") else surface,
                                        expected_code=CASES[case_id][1], script=script, **extra)
                        if CASES[case_id][1] is not None:
                            recovery = run_case(packet, store / (output.name + "-recovery"), runtime,
                                environment, case_id="INFRA-S01-01", surface=surface)
                            self.assertTrue(recovery["observation_matches_expected"])
                            self.assertNotIn(recovery["invocation_id"], invocations)
                            invocations.add(recovery["invocation_id"])
            self.assertEqual(4 * sum(1 + (code is not None) for _, code in ordinary_cases.values()), len(invocations))

    def test_missing_and_moving_trust_refs_refuse_separately_on_every_surface(self):
        with tempfile.TemporaryDirectory() as temporary:
            store = Path(temporary)
            packet = store / "packet"
            recipe = prepare_fixture(store / "baseline", packet)
            environment = {"PATH": os.defpath, "HOME": str(store), "XDG_CONFIG_HOME": str(store),
                           "GIT_CONFIG_NOSYSTEM": "1", "GIT_CONFIG_GLOBAL": os.devnull, "LANG": "C"}
            runtime = Path(__file__).resolve().parents[1]
            identities = set()
            for case, (field, value) in TRUST_CASES.items():
                for surface in ("C", "R", "S-check", "S-details", "H"):
                    with self.subTest(case=case, surface=surface):
                        for phase, selected in (("refusal", case), ("recovery", "INFRA-S01-01")):
                            output = store / (case + surface + phase)
                            result = (run_hook_case(packet, output, environment, case_id=selected)
                                      if surface == "H" else run_case(packet, output, runtime, environment,
                                                                     case_id=selected, surface=surface))
                            self.assertTrue(result["observation_matches_expected"])
                            self.assertFalse(result["acceptance_asserted"])
                            self.assertFalse(result["complete_matrix"])
                            self.assertNotIn(result["invocation_id"], identities)
                            identities.add(result["invocation_id"])
                            before = json.loads((output / "capture/before.json").read_bytes())
                            key = "datum.workflowDelivery" + field.title() + "Ref"
                            expected = value if phase == "refusal" else recipe["head"]
                            self.assertEqual([] if expected is None else [expected], before["local_trust"][key])
                            other = "Base" if field == "authority" else "Authority"
                            self.assertEqual([recipe["head"]], before["local_trust"]["datum.workflowDelivery" + other + "Ref"])
                            if surface == "H" and phase == "refusal":
                                self.assertFalse(result["validator_entrypoint_observed"])
                                self.assertEqual("owner-hook-before-bootstrap", result["refusal_boundary"])
            self.assertEqual(40, len(identities))

    def test_invocation_errors_have_no_success_report_and_recover(self):
        with tempfile.TemporaryDirectory() as temporary:
            store = Path(temporary)
            packet = store / "packet"
            prepare_fixture(store / "baseline", packet)
            environment = {"PATH": os.defpath, "HOME": str(store), "XDG_CONFIG_HOME": str(store),
                           "GIT_CONFIG_NOSYSTEM": "1", "GIT_CONFIG_GLOBAL": os.devnull, "LANG": "C"}
            runtime = Path(__file__).resolve().parents[1]
            identities = set()
            for case in sorted(CLI_ERROR_CASES):
                for surface in ("C", "R"):
                    destination = store / (case + surface)
                    result = run_case(packet, destination, runtime, environment, case_id=case, surface=surface)
                    self.assertEqual("invocation_error", result["kind"])
                    self.assertEqual(b"", (destination / "capture/stdout.bin").read_bytes())
                    recovery = run_case(packet, store / (case + surface + "recovery"), runtime,
                                        environment, case_id="INFRA-S01-01", surface=surface)
                    for observed in (result, recovery):
                        self.assertTrue(observed["observation_matches_expected"])
                        self.assertFalse(observed["complete_matrix"])
                        self.assertFalse(observed["acceptance_asserted"])
                        self.assertNotIn(observed["invocation_id"], identities)
                        identities.add(observed["invocation_id"])
                for surface in ("S-check", "S-details"):
                    with self.assertRaisesRegex(ValueError, "CLI surface"):
                        run_case(None, None, None, {}, case_id=case, surface=surface)
            self.assertEqual(8, len(identities))

    def test_same_input_report_only_is_not_enforcement(self):
        with tempfile.TemporaryDirectory() as temporary:
            store = Path(temporary)
            packet, baseline = store / "packet", store / "baseline-packet"
            recipe = prepare_fixture(store / "source", packet, fixture_variant=REPORT_FIXTURE)
            prepare_fixture(store / "baseline", baseline)
            environment = {"PATH": os.defpath, "HOME": str(store), "XDG_CONFIG_HOME": str(store),
                           "GIT_CONFIG_NOSYSTEM": "1", "GIT_CONFIG_GLOBAL": os.devnull, "LANG": "C"}
            runtime = Path(__file__).resolve().parents[1]
            identities, diagnostics = set(), {}
            for case in sorted(REPORT_CASES):
                for surface in ("C", "R"):
                    destination = store / (case + surface)
                    result = run_case(packet, destination, runtime, environment, case_id=case, surface=surface)
                    self.assertEqual(case == REPORT_ONLY_CASE, result["report_only"])
                    before = json.loads((destination / "capture/before.json").read_bytes())
                    self.assertEqual(recipe["head"], before["head"])
                    report = json.loads((destination / "capture/stdout.bin").read_bytes())
                    self.assertEqual([], report["checks"])
                    finding = report["findings"][0]
                    diagnostics.setdefault(case, []).append((finding["code"], finding["path"], finding["detail"]))
                    if case == REPORT_ONLY_CASE:
                        with self.assertRaises(ValueError):
                            evaluate_capture(destination / "capture", case_id=case, surface=surface,
                                expected_code="WDQ-ENVIRONMENT", script=runtime / "scripts/check_workflow_delivery.py")
                    recovery = run_case(baseline, store / (case + surface + "recovery"), runtime,
                                        environment, case_id="INFRA-S01-01", surface=surface)
                    for observed in (result, recovery):
                        self.assertTrue(observed["observation_matches_expected"])
                        self.assertFalse(observed["complete_matrix"])
                        self.assertFalse(observed["acceptance_asserted"])
                        self.assertNotIn(observed["invocation_id"], identities)
                        identities.add(observed["invocation_id"])
            self.assertEqual(8, len(identities))
            # Identical retained input, deliberately different trust semantics:
            # report-only stops before schema-2 environment authority resolution;
            # enforcement reaches the invalid observed result. Neither grants permission.
            for case, observed in diagnostics.items():
                expected = ("WDQ-ENVIRONMENT", "requested-environment.json",
                            "owner-selected environment authority required") if case == REPORT_ONLY_CASE else (
                            "WDQ-RESULT", "evidence/original-events.json", "event and observed result disagree")
                self.assertEqual([expected, expected], observed)

    def test_capture_tampering_and_false_unchanged_summary_are_rejected(self):
        with tempfile.TemporaryDirectory() as temporary:
            store = Path(temporary)
            packet = store / "packet"
            prepare_fixture(store / "baseline", packet)
            environment = {"PATH": os.defpath, "HOME": str(store), "XDG_CONFIG_HOME": str(store),
                "GIT_CONFIG_NOSYSTEM": "1", "GIT_CONFIG_GLOBAL": os.devnull, "LANG": "C"}
            runtime = Path(__file__).resolve().parents[1]
            output = store / "case"
            run_case(packet, output, runtime, environment, case_id="INFRA-S01-01", surface="C")
            capture = output / "capture"
            def evaluate():
                return evaluate_capture(capture, case_id="INFRA-S01-01", surface="C",
                    expected_code=None, script=runtime / "scripts/check_workflow_delivery.py")
            original = (capture / "stdout.bin").read_bytes()
            (capture / "stdout.bin").write_bytes(original + b"\n")
            with self.assertRaisesRegex(ValueError, "hash mismatch"):
                evaluate()
            (capture / "stdout.bin").write_bytes(original)
            result_path = capture / "result.json"
            result = json.loads(result_path.read_text())
            result["kind"] = "timeout"
            result_path.write_bytes(canonical_json(result))
            with self.assertRaisesRegex(ValueError, "not a refusal"):
                evaluate()
            result["kind"] = "exited"
            after_path = capture / "after.json"
            after = json.loads(after_path.read_text())
            after["files"]["src/read.py"]["sha256"] = "0" * 64
            after_path.write_bytes(canonical_json(after))
            for artifact in result["artifacts"]:
                if artifact["path"] == "after.json":
                    artifact["sha256"] = sha256(after_path.read_bytes())
            result_path.write_bytes(canonical_json(result))
            with self.assertRaisesRegex(ValueError, "protected state changed"):
                evaluate()


if __name__ == "__main__":
    unittest.main()
