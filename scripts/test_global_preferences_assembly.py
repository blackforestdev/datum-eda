"""Validator-grounded tests for the GP-CM05E Phase-A U3 assembly core.

Covers the Q1 shared-identity injection into producer.build_family (consume-never-compute,
surface-binary pinning), the frozen-baton seams the audit flagged (seam 1: 21-field environments
verbatim + the frozen.json-only selection aid, ONE binding; seam 2: recipe-in-manifest), the
engine/CLI merge refusals (duplicate row, identity mismatch), and the placeholder lifecycle.

Everything here is a SYNTHETIC identity exercise of binding/merge mechanics (origin shapes,
mocked recipe runs) -- never product proof. The partially assembled report is EXPLICITLY
asserted to stay incomplete/non-ready under the governed validators; no test weakens a
validator, and no path issues readiness.
"""
import json
import os
import tempfile
import unittest
from pathlib import Path
from unittest.mock import patch

from global_preferences_acceptance import assemble, gui_producer, producer
from global_preferences_acceptance import report as report_module
from global_preferences_acceptance.capture import Capture
from global_preferences_acceptance.environment_record import build_environments
from global_preferences_acceptance.identity import validate_environments
from global_preferences_acceptance.inputs import Bundle, EvidenceError, canonical, digest, parse

REAL_ROOT = Path(__file__).resolve().parents[1]
NATIVE_SPEC = {"coordinate": "wayland/1.0/1024x900",
               "gpu": "Intel(R) HD Graphics P630 (KBL GT2)", "driver": "Mesa 25.0.7",
               "compositor": "owned nested Weston",
               "background_load": "quiescent -- floor-quiet, one capture process"}
PAIRS = [(("surface-parity-negative", "active-inventory"), "cli"),
         (("managed-offline", "inactive"), "engine")]


def fake_command(capture, argv, *, cwd, candidate, inputs, timeout=3600):
    """Class-level Capture.command replacement: a synthetic recipe run whose observations affirm
    the frozen fixture's expected assertions. Binding/merge mechanics only; never product proof."""
    frozen = json.loads(argv[3])
    stdout = capture.put({"schema": "datum.preferences.case-observations.v2",
                          "executed_tests": 1,
                          "observations": {aid: True for aid in frozen["expected_assertions"]}})
    record = {"schema": "datum.preferences.command-observation.v2", "command": argv,
              "exit_code": 0, "candidate_revision": candidate, "input_manifest_sha256": inputs,
              "executed_tests": 1, "stdout": stdout, "stderr": capture.put(b"")}
    return {name: record[name] for name in ("command", "exit_code", "candidate_revision",
                                            "input_manifest_sha256")} | {"log": capture.put(record)}


class AssemblyTests(unittest.TestCase):
    def setUp(self):
        # These assembly fixtures deliberately use synthetic, non-Git identities.
        guard = patch.object(producer, "validate_live_inputs")
        self.live_guard = guard.start()
        self.addCleanup(guard.stop)
        temporary = tempfile.TemporaryDirectory()
        self.addCleanup(temporary.cleanup)
        self.root = Path(temporary.name)
        self.bundle_root = self.root / "bundle"
        self.bundle_root.mkdir()
        shared = Capture(self.bundle_root)
        recipes = []
        for relative in (producer.RECIPE_RELPATH, producer.ENGINE_RECIPE_RELPATH):
            path = self.root / relative
            path.parent.mkdir(parents=True, exist_ok=True)
            path.write_bytes(b"# Synthetic assembly recipe; never product proof.\n")
            recipes.append({"path": relative, "sha256": digest(path.read_bytes())})
        self.manifest = shared.put(recipes)
        self.binaries = {}
        for role in assemble.CANDIDATE_ROLES:
            path = self.root / "bins" / role
            path.parent.mkdir(exist_ok=True)
            path.write_bytes(b"synthetic role binary: " + role.encode())
            self.binaries[role] = path
        frozen_binaries = {role: {"path": "command-capture/" + digest(path.read_bytes()),
                                  "sha256": digest(path.read_bytes())}
                           for role, path in self.binaries.items()}
        self.candidate = {"revision": "a" * 40, "tree": "b" * 40, "cargo_lock_sha256": "c" * 64,
                          "toolchains": {"rustc": "rustc", "cargo": "cargo", "python": "python3"},
                          "build_commands": [{"synthetic": True}], "binaries": frozen_binaries}
        self.environments = build_environments(self.root, [NATIVE_SPEC])
        self.identity = assemble.freeze_identity(self.candidate, self.environments,
                                                 self.manifest, "d" * 64)

    def patched(self):
        return (patch.object(producer, "ROOT", self.root), patch.dict(os.environ),
                patch.object(Capture, "command", fake_command))

    def build_injected_family(self, family, surface):
        patches = self.patched()
        with patches[0], patches[1], patches[2]:
            return producer.build_family(
                self.bundle_root, family=family, surface=surface,
                cli_binary=self.binaries["cli"], engine_binary=self.binaries["engine"],
                identity=self.identity)

    def test_changed_capture_inputs_refuse_before_recipe(self):
        self.live_guard.side_effect = EvidenceError("current proof input changed")
        with patch.object(Capture, "command") as command:
            with self.assertRaisesRegex(EvidenceError, "current proof input changed"):
                with patch.object(producer, "ROOT", self.root):
                    producer.build_family(self.bundle_root, family=PAIRS[0][0], surface="cli",
                                          cli_binary=self.binaries["cli"], identity=self.identity)
            command.assert_not_called()

    def test_changes_during_recipe_refuse_completed_row(self):
        self.live_guard.side_effect = [None, EvidenceError("current proof input changed")]
        with self.assertRaisesRegex(EvidenceError, "current proof input changed"):
            self.build_injected_family(*PAIRS[0])

    def assemble_pairs(self, pairs=PAIRS):
        patches = self.patched()
        with patches[0], patches[1], patches[2]:
            return assemble.assemble_cases(self.bundle_root, self.identity,
                                           cli_binary=self.binaries["cli"],
                                           engine_binary=self.binaries["engine"], pairs=pairs)

    def test_freeze_identity_carries_environments_verbatim_with_selection_aid(self):
        identity = self.identity
        # Seam 1: the 21-field array is verbatim (no injected keys) and binds report.py:32's
        # environment_sha256; the governed validator accepts it unchanged.
        self.assertEqual(canonical(identity["environments"]), canonical(self.environments))
        self.assertTrue(all("coordinate" not in entry for entry in identity["environments"]))
        self.assertEqual(identity["environment_sha256"],
                         digest(canonical(identity["environments"])))
        validate_environments(identity["environments"])
        # The selection aid lives ONLY in the frozen baton and lands on ids present in the
        # 21-field array (ONE binding, no fork), matching the deterministic derivation.
        self.assertEqual(len(identity["selection"]), 1)
        selected = identity["selection"][0]
        self.assertEqual(selected["id"], "native-wayland-1-1024x900")
        self.assertIn(selected["id"], {entry["id"] for entry in identity["environments"]})
        self.assertEqual(selected["coordinate"], ["wayland", 1.0, [1024, 900]])
        # F1 compatibility: the aid's coordinate is exactly the shape gui_producer normalises.
        self.assertEqual(gui_producer._coordinate_tuple(selected["coordinate"]),
                         ("wayland", 1.0, (1024, 900)))
        # Round-trip through the baton file preserves the binding.
        frozen_path = self.root / "frozen.json"
        assemble.write_frozen(frozen_path, identity)
        self.assertEqual(canonical(assemble.load_frozen(frozen_path)), canonical(identity))

    def test_freeze_identity_refuses_nonderived_native_id(self):
        environments = json.loads(json.dumps(self.environments))
        environments[1]["id"] = "native-wayland-1.0-1024x900"
        with self.assertRaisesRegex(EvidenceError, "deterministic derivation"):
            assemble.freeze_identity(self.candidate, environments, self.manifest, "d" * 64)

    def test_injected_identity_binds_frozen_fields_on_rows(self):
        report, binaries, environments, keys = self.build_injected_family(PAIRS[0][0], "cli")
        self.assertEqual(keys, [("surface-parity-negative", "active-inventory", "default",
                                 "cli", None)])
        row = report["case_results"][0]
        # Consume-never-compute: every bound identity field is the frozen one.
        self.assertEqual(row["candidate_revision"], self.candidate["revision"])
        self.assertEqual(row["input_manifest_sha256"], self.manifest["sha256"])
        self.assertEqual(row["matrix_sha256"], "d" * 64)
        self.assertEqual(row["binary_sha256"], self.candidate["binaries"]["cli"]["sha256"])
        self.assertEqual(row["environment_id"], "headless-linux-x86_64")
        self.assertIn(row["environment_id"], {entry["id"] for entry in self.environments})
        self.assertEqual(canonical(report["candidate"]), canonical(self.candidate))
        self.assertEqual(canonical(report["environments"]), canonical(self.environments))
        self.assertEqual(report["input_manifest"], self.manifest)
        # The governed per-case validators accept the injected row as produced.
        bundle = Bundle(self.bundle_root, report["artifacts"])
        producer.validate_case_row(bundle, row, keys[0], report, binaries, environments)

    def test_injected_identity_refuses_substituted_surface_binary(self):
        patches = self.patched()
        with patches[0], patches[1], patches[2]:
            with self.assertRaisesRegex(EvidenceError, "no substitution"):
                producer.build_family(self.bundle_root, family=PAIRS[0][0], surface="cli",
                                      cli_binary=self.binaries["gui"], identity=self.identity)

    def test_assembled_report_merges_both_surfaces_under_one_identity(self):
        report = self.assemble_pairs()
        self.assertEqual({row["surface"] for row in report["case_results"]}, {"cli", "engine"})
        self.assertEqual(len(report["case_results"]), 2)
        role_shas = {role: ref["sha256"] for role, ref in self.candidate["binaries"].items()}
        for row in report["case_results"]:
            self.assertEqual(row["candidate_revision"], self.candidate["revision"])
            self.assertEqual(row["input_manifest_sha256"], self.manifest["sha256"])
            self.assertEqual(row["matrix_sha256"], "d" * 64)
            self.assertEqual(row["binary_sha256"], role_shas[row["surface"]])
            self.assertIn(row["environment_id"], {e["id"] for e in self.environments})
        # The merged report is exactly the closed 13-key V2 shape with the frozen identity
        # carried verbatim; environments still bind the frozen environment_sha256.
        self.assertEqual(set(report), set(
            "schema origin candidate matrix_sha256 input_manifest environments artifacts "
            "case_results measurements gate_results native_observations independent_review "
            "release_notes".split()))
        self.assertEqual(report["schema"], assemble.REPORT_SCHEMA)
        self.assertEqual(report["origin"]["kind"], "product-execution")
        self.assertEqual(canonical(report["environments"]), canonical(self.environments))
        self.assertEqual(digest(canonical(report["environments"])),
                         self.identity["environment_sha256"])
        self.assertEqual(report["gate_results"], [])
        self.assertEqual(report["measurements"], [])
        self.assertEqual(report["native_observations"], [])
        # Replay the governed per-case validators against the merged rows.
        bundle = Bundle(self.bundle_root, report["artifacts"])
        environments = validate_environments(report["environments"])
        for row in report["case_results"]:
            key = (row["case_id"], row["variant_id"], row["subcase_id"], row["surface"], None)
            producer.validate_case_row(bundle, row, key, report,
                                       {row["surface"]: row["binary_sha256"]}, environments)

    def test_unwired_family_refuses_loudly(self):
        # An inventory (family, surface) pair without a producer handler must REFUSE -- a loud
        # ValueError, never a silent skip -- and the refusal fires before any case command
        # executes (a partial selection can never masquerade as produced coverage).
        def forbidden(capture, argv, **kwargs):
            raise AssertionError("no case may execute for an unwired family")
        unwired = (("clean-install", "missing-store"), "cli")
        self.assertIn(unwired, assemble.cli_engine_pairs())
        with patch.object(producer, "ROOT", self.root), patch.dict(os.environ), \
                patch.object(Capture, "command", forbidden):
            with self.assertRaisesRegex(ValueError, "case family not implemented"):
                assemble.assemble_cases(self.bundle_root, self.identity,
                                        cli_binary=self.binaries["cli"],
                                        engine_binary=self.binaries["engine"],
                                        pairs=[unwired])

    def test_duplicate_row_refused(self):
        family_report, *_ = self.build_injected_family(PAIRS[0][0], "cli")
        with self.assertRaisesRegex(EvidenceError, "duplicate case row"):
            assemble.merge_case_results(self.identity, [family_report, family_report])

    def test_identity_mismatch_refused(self):
        family_report, *_ = self.build_injected_family(PAIRS[0][0], "cli")
        tampered = json.loads(json.dumps(family_report))
        tampered["matrix_sha256"] = "f" * 64
        with self.assertRaisesRegex(EvidenceError, "sub-report matrix_sha256 differs"):
            assemble.merge_case_results(self.identity, [tampered])
        for field, value, message in (
                ("candidate_revision", "e" * 40, "row identity differs"),
                ("environment_id", "unfrozen-environment", "not an id in the frozen"),
                ("binary_sha256", self.candidate["binaries"]["gui"]["sha256"],
                 "not the frozen candidate binary")):
            with self.subTest(field=field):
                tampered = json.loads(json.dumps(family_report))
                tampered["case_results"][0][field] = value
                with self.assertRaisesRegex(EvidenceError, message):
                    assemble.merge_case_results(self.identity, [tampered])

    def test_recipe_outside_frozen_manifest_refused(self):
        family_report, *_ = self.build_injected_family(PAIRS[0][0], "cli")
        row = family_report["case_results"][0]
        capture = Capture(self.bundle_root, inventory=family_report["artifacts"])
        fixture = Bundle(self.bundle_root, family_report["artifacts"]).json(row["fixture"])
        fixture["recipe"] = {"path": "scripts/uncommitted_recipe.py", "sha256": "0" * 64}
        foreign = {**row, "fixture": capture.put(fixture)}
        bundle = Bundle(self.bundle_root, list(capture.inventory.values()))
        with self.assertRaisesRegex(EvidenceError, "not in the frozen input-manifest closure"):
            assemble.verify_recipe_membership(bundle, self.manifest, [foreign])

    def test_partial_assembled_report_stays_non_ready(self):
        report = self.assemble_pairs()
        bundle = Bundle(self.bundle_root, report["artifacts"])
        # The explicit placeholders can NEVER satisfy their governed validators.
        pending_review = bundle.json(report["independent_review"])
        self.assertEqual(pending_review["schema"], assemble.PENDING_REVIEW_SCHEMA)
        with self.assertRaises(EvidenceError):
            report_module.validate_review(pending_review, report, bundle)
        pending_release = bundle.json(report["release_notes"])
        self.assertEqual(pending_release["schema"], assemble.PENDING_RELEASE_SCHEMA)
        with self.assertRaises(EvidenceError):
            report_module.validate_release(pending_release, report, bundle)
        # The governed full validator refuses the partial report outright (review_only included):
        # a partially assembled corpus is INCOMPLETE and never reaches any readiness state.
        matrix = parse((REAL_ROOT / producer.MATRIX_RELPATH).read_bytes(), "matrix")
        with self.assertRaises(EvidenceError):
            report_module.validate_report(
                report, matrix, root=REAL_ROOT, bundle_root=self.bundle_root,
                expected_revision=self.candidate["revision"],
                expected_environment_sha256=self.identity["environment_sha256"],
                review_only=True)


if __name__ == "__main__":
    unittest.main()
