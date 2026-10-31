"""Capture failures, enforce mandatory compilation and reject legacy subset modes."""
import json
import os
from pathlib import Path
import subprocess
import sys
import tempfile
import unittest
from unittest.mock import Mock, patch

from global_preferences_acceptance import producer
from global_preferences_acceptance.capture import Capture, executed_tests
from global_preferences_acceptance.gates import commands
from global_preferences_acceptance.inputs import Bundle, EvidenceError, digest
from global_preferences_acceptance.matrix import case_inventory


class RunnerTests(unittest.TestCase):
    def test_workspace_clippy_and_renderer_proof_are_mandatory(self):
        inventory = commands(Path(__file__).resolve().parents[1])
        self.assertIn('workspace-tests', inventory)
        self.assertIn('clippy', inventory)
        for name in ('workspace-tests', 'clippy'):
            argv = inventory[name][0]
            self.assertIn('scripts/run_cargo_guarded.py', argv)
            self.assertIn('--all-targets', argv)
            self.assertIn('--locked', argv)
            self.assertIn('--offline', argv)
        self.assertIn('-D', inventory['clippy'][0])
        # The workspace gates prove the SHIPPING configuration: they must NOT enable the
        # GP-CM05E acceptance feature (Option A would compile the fixtures into the
        # CLI/GUI/daemon build and stop the proof from testing what ships).
        for name in ('workspace-tests', 'clippy'):
            self.assertNotIn('--features', inventory[name][0])
            self.assertNotIn('gp-acceptance-support', inventory[name][0])
        # Dedicated feature-ON gates keep full test + clippy coverage of the acceptance
        # fixtures (zero coverage loss) without that unification, scoped to eda-engine.
        for name in ('acceptance-support-tests', 'acceptance-support-clippy'):
            self.assertIn(name, inventory)
            argv = inventory[name][0]
            self.assertIn('scripts/run_cargo_guarded.py', argv)
            self.assertIn('-p', argv)
            self.assertIn('eda-engine', argv)
            self.assertIn('--features', argv)
            self.assertIn('gp-acceptance-support', argv)
            self.assertIn('--all-targets', argv)
            self.assertIn('--locked', argv)
            self.assertIn('--offline', argv)
        self.assertIn('-D', inventory['acceptance-support-clippy'][0])

    def test_command_failure_retains_exact_outputs_and_status(self):
        with tempfile.TemporaryDirectory() as root:
            capture = Capture(root)
            row = capture.command([sys.executable, '-c', 'import sys; print("out"); print("err",file=sys.stderr); sys.exit(7)'],
                                  cwd=root, candidate='a' * 40, inputs='b' * 64)
            self.assertEqual(row['exit_code'], 7)
            bundle = Bundle(root, list(capture.inventory.values()))
            raw = bundle.json(row['log'])
            self.assertEqual(bundle.read(raw['stdout']), b'out\n')
            self.assertEqual(bundle.read(raw['stderr']), b'err\n')
            self.assertEqual(raw['executed_tests'], 0)

    def test_timeout_is_failure_and_retained(self):
        with tempfile.TemporaryDirectory() as root:
            capture = Capture(root)
            row = capture.command([sys.executable, '-c', 'import time; time.sleep(10)'], cwd=root,
                                  candidate='a' * 40, inputs='b' * 64, timeout=0.05)
            self.assertEqual(row['exit_code'], 124)
            bundle = Bundle(root, list(capture.inventory.values()))
            self.assertIn(b'timed out', bundle.read(bundle.json(row['log'])['stderr']))

    def test_test_discovery_does_not_count_as_execution(self):
        self.assertEqual(executed_tests(b'test_name: test\n100 tests listed\n'), 0)
        self.assertEqual(executed_tests(b'test result: ok. 0 passed;\n'), 0)
        self.assertEqual(executed_tests(b'test result: ok. 3 passed;\ntest result: ok. 2 passed;\n'), 5)
        self.assertEqual(executed_tests(b'Ran 11 tests in 0.1s\n\nOK\n'), 11)

    def test_optional_subset_flags_cannot_report_readiness(self):
        script = Path(__file__).parent / 'run_global_preferences_production_acceptance.py'
        for options in ([], ['--measure'], ['--full-workspace']):
            result = subprocess.run([sys.executable, str(script), *options], capture_output=True, text=True)
            self.assertNotEqual(result.returncode, 0)
            self.assertNotIn('PASS', result.stdout)
            self.assertNotIn('candidate_ready', result.stdout)


    def produce_binding(self, surface='cli', *, environments=None, recipe=None,
                manifest_mode=None, capture_after=None, setup=None, redirected=False,
                command_stdout=None, command_exit=0, command_stderr=b''):
        temporary = tempfile.TemporaryDirectory()
        self.addCleanup(temporary.cleanup)
        root = Path(temporary.name)
        bundle_root = root / 'bundle'
        bundle_root.mkdir()
        capture = self.last_capture = Capture(bundle_root)
        gui_recipe = 'scripts/gui_case.py'
        recipes = []
        for relative in (producer.RECIPE_RELPATH, gui_recipe):
            path = root / relative
            path.parent.mkdir(parents=True, exist_ok=True)
            path.write_bytes(b'# Synthetic binding recipe; never product proof.\n')
            recipes.append({'path': relative, 'sha256': digest(path.read_bytes())})
        selected_recipe = recipe or (gui_recipe if surface == 'gui' else producer.RECIPE_RELPATH)
        if manifest_mode == 'omit':
            recipes = [entry for entry in recipes if entry['path'] != selected_recipe]
        elif manifest_mode == 'wrong-digest':
            recipes = [{**entry, 'sha256': '0' * 64} if entry['path'] == selected_recipe else entry
                       for entry in recipes]
        if redirected:
            path = root / selected_recipe
            path.unlink()
            path.symlink_to(root / producer.RECIPE_RELPATH)
        manifest = capture.put(recipes)
        if manifest_mode == 'artifact-tampered':
            (capture.root / manifest['path']).write_bytes(b'[]')
        inventory = case_inventory()
        key = next(k for k in inventory
                   if k[:3] == ('clean-install', 'missing-store', 'default') and k[3] == surface)
        environments = environments if environments is not None else {
            **({'decoy': None} if surface == 'gui' else {}), 'selected': key[4]}
        binaries = {'cli': 'c' * 64, 'gui': 'd' * 64}
        candidate = {'revision': 'a' * 40}
        recorded = {}

        def command(argv, *, cwd, candidate, inputs):
            recorded.update(argv=argv, cwd=cwd,
                            binary=os.environ[f'DATUM_EDA_{surface.upper()}'])
            stdout = capture.put({'schema': 'datum.preferences.case-observations.v2',
                                  'executed_tests': 1,
                                  'observations': {name: True for name in inventory[key]}})
            if command_stdout is not None:
                stdout = capture.put(command_stdout)
            record = {'schema': 'datum.preferences.command-observation.v2', 'command': argv,
                      'exit_code': command_exit, 'candidate_revision': candidate,
                      'input_manifest_sha256': inputs, 'executed_tests': 1,
                      'stdout': stdout, 'stderr': capture.put(command_stderr)}
            self.last_command_record = record
            return {name: record[name] for name in ('command', 'exit_code', 'candidate_revision',
                                                   'input_manifest_sha256')} | {'log': capture.put(record)}

        capture.command = self.last_command = Mock(side_effect=command)
        self.last_work = bundle_root / '.work'
        options = {'recipe_relpath': selected_recipe} if surface == 'gui' or recipe else {}
        with patch.object(producer, 'ROOT', root), patch.dict(os.environ):
            row = producer.produce_case(
                capture, key=key, candidate=candidate, binaries=binaries,
                environments=environments, input_manifest=manifest, matrix_sha256='b' * 64,
                surface_binaries={surface: f'/synthetic/{surface}'}, capture_after=capture_after,
                bundle_root=bundle_root, setup=setup, **options)
        report = {'candidate': candidate, 'input_manifest': manifest, 'matrix_sha256': 'b' * 64}
        return capture, row, report, key, binaries, environments, recorded

    def validate_binding(self, result):
        capture, row, report, key, binaries, environments, _ = result
        producer.validate_case_row(Bundle(capture.root, list(capture.inventory.values())),
                                   row, key, report, binaries, environments)

    def test_invalid_case_output_reports_retained_failure_locations(self):
        for stdout in (b'', b'{malformed', b'\xff'):
            with self.subTest(stdout=stdout):
                with self.assertRaises(EvidenceError) as caught:
                    self.produce_binding(command_stdout=stdout, command_exit=7,
                                         command_stderr=b'private recipe traceback')
                capture = self.last_capture
                bundle = Bundle(capture.root, list(capture.inventory.values()))
                record = self.last_command_record
                message = str(caught.exception)
                self.assertIn('exit_code=7', message)
                for field in ('stdout', 'stderr'):
                    self.assertIn(str(capture.root / record[field]['path']), message)
                log = next(ref for ref in capture.inventory.values()
                           if bundle.read(ref) == producer.canonical(record))
                self.assertIn(str(capture.root / log['path']), message)
                self.assertEqual(bundle.read(record['stdout']), stdout)
                self.assertEqual(bundle.read(record['stderr']), b'private recipe traceback')
                self.assertNotIn('private recipe traceback', message)
                self.assertIsInstance(caught.exception.__cause__,
                                      (json.JSONDecodeError, UnicodeDecodeError))
                self.last_command.assert_called_once()

    def test_valid_case_output_preserves_nonzero_exit(self):
        result = self.produce_binding(command_exit=7)
        self.assertEqual(result[1]['exit_code'], 7)
        self.assertEqual(result[1]['executed_tests'], 1)
        self.last_command.assert_called_once()

    def test_default_cli_unchanged(self):
        result = self.produce_binding()
        self.validate_binding(result)
        capture, row, _, key, _, _, recorded = result
        frozen = Bundle(capture.root, list(capture.inventory.values())).json(row['fixture'])
        self.assertEqual(recorded['argv'][:3], ['python3', producer.RECIPE_RELPATH, '--fixture-json'])
        self.assertEqual(json.loads(recorded['argv'][3]), frozen)
        self.assertEqual(frozen['parameters']['coverage'], list(key))
        self.assertEqual(recorded['binary'], '/synthetic/cli')
        self.assertEqual(row['binary_sha256'], 'c' * 64)
        self.assertEqual(row['before'], row['after'])
        self.last_command.assert_called_once()

    def test_gui_role_recipe_digest_coordinate_with_decoy_environment(self):
        result = self.produce_binding('gui')
        self.validate_binding(result)
        capture, row, _, key, _, _, recorded = result
        frozen = Bundle(capture.root, list(capture.inventory.values())).json(row['fixture'])
        self.assertEqual(row['environment_id'], 'selected')
        self.assertEqual(row['binary_sha256'], 'd' * 64)
        self.assertEqual(recorded['binary'], '/synthetic/gui')
        self.assertEqual(recorded['argv'][:3], ['python3', 'scripts/gui_case.py', '--fixture-json'])
        self.assertEqual(json.loads(recorded['argv'][3]), frozen)
        self.assertEqual(frozen['parameters']['coverage'], json.loads(json.dumps(key)))
        self.assertEqual(frozen['recipe']['sha256'],
                         digest(b'# Synthetic binding recipe; never product proof.\n'))

    def test_missing_coordinate_fails_before_setup_work_tree_or_command(self):
        setup = Mock()
        with self.assertRaisesRegex(EvidenceError, 'matching environment'):
            self.produce_binding('gui', environments={'headless': None}, setup=setup)
        self.last_command.assert_not_called()
        setup.assert_not_called()
        self.assertFalse(self.last_work.exists())

    def test_unsafe_unbound_or_changed_recipe_fails_before_execution(self):
        for options in ({'recipe': '../escape.py'}, {'recipe': '/tmp/escape.py'},
                        {'manifest_mode': 'omit'}, {'manifest_mode': 'wrong-digest'}, {'manifest_mode': 'artifact-tampered'},
                        {'redirected': True}):
            with self.subTest(options=options):
                setup = Mock()
                with self.assertRaises((EvidenceError, OSError)):
                    self.produce_binding('gui', setup=setup, **options)
                self.last_command.assert_not_called()
                setup.assert_not_called()
                self.assertFalse(self.last_work.exists())

    def test_duplicate_matching_coordinate_fails_before_execution(self):
        with self.assertRaisesRegex(EvidenceError, 'exactly one matching environment'):
            self.produce_binding(environments={'first': None, 'second': None})
        self.last_command.assert_not_called()
        self.assertFalse(self.last_work.exists())

    def test_wrong_role_rejected_by_real_row_validator(self):
        result = self.produce_binding('gui')
        result[1]['binary_sha256'] = result[4]['cli']
        with self.assertRaisesRegex(EvidenceError, 'executable'):
            self.validate_binding(result)

    def test_changed_fixture_recipe_or_coverage_is_rejected(self):
        for field in ('recipe', 'coverage'):
            with self.subTest(field=field):
                result = self.produce_binding('gui')
                capture, row, *_ = result
                frozen = Bundle(capture.root, list(capture.inventory.values())).json(row['fixture'])
                if field == 'recipe':
                    frozen['recipe']['sha256'] = '0' * 64
                else:
                    frozen['parameters']['coverage'][3] = 'cli'
                row['fixture'] = capture.put(frozen)
                with self.assertRaises(EvidenceError):
                    self.validate_binding(result)

    def test_capture_after_result_is_retained(self):
        references = []
        def after(capture, key, work_root, before):
            self.assertTrue(work_root.is_dir())
            reference = capture.put({**before, 'root': before['root'] + '/after'})
            references.append(reference)
            return reference
        callback = Mock(side_effect=after)
        result = self.produce_binding(capture_after=callback)
        self.validate_binding(result)
        callback.assert_called_once()
        self.assertEqual(result[1]['after'], references[0])
        self.assertNotEqual(result[1]['before'], result[1]['after'])


if __name__ == '__main__':
    unittest.main()
