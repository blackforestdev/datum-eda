"""End-to-end fail-closed evidence validation; no fixture is product proof."""
from copy import deepcopy
import json
import os
from pathlib import Path
import subprocess
import sys
import tempfile
import unittest
from unittest.mock import patch

from global_preferences_acceptance.inputs import Bundle, EvidenceError, canonical, digest
from global_preferences_acceptance.identity import validate_live_inputs
from global_preferences_acceptance.report import validate_report
from global_preferences_acceptance.test_support import (
    SyntheticBundle, lift_dispositions, restore_dispositions)
from test_global_preferences_production_matrix import MATRIX


class EvidenceTests(unittest.TestCase):
    @classmethod
    def setUpClass(cls):
        # Mechanics suite: exercise validate_report against the full NOMINAL (pre-disposition / pin-2)
        # corpus so the §9 native-replay branch and the complete-coverage paths keep coverage. The
        # current-pin dispositions (scale-2 env-block, native scope-exit) and their non-ready
        # consequence are asserted separately in test_global_preferences_coverage_dispositions.
        cls._dispositions = lift_dispositions()
        cls.temporary = tempfile.TemporaryDirectory()
        cls.fixture = SyntheticBundle(cls.temporary.name, MATRIX)
        cls.environment_sha = digest(canonical(cls.fixture.report['environments']))

    @classmethod
    def tearDownClass(cls):
        cls.temporary.cleanup()
        restore_dispositions(cls._dispositions)

    def check(self, report, **kwargs):
        return validate_report(report, MATRIX, root=self.fixture.root, bundle_root=self.fixture.directory,
                               expected_revision=self.fixture.revision,
                               expected_environment_sha256=self.environment_sha, test_only=True, **kwargs)

    def test_complete_synthetic_report_and_production_exclusion(self):
        result = self.check(self.fixture.report)
        self.assertEqual(result['state'], 'synthetic_validated')
        self.assertFalse(result['production_accepted'])
        with self.assertRaisesRegex(EvidenceError, 'synthetic'):
            validate_report(self.fixture.report, MATRIX, root=self.fixture.root, bundle_root=self.fixture.directory,
                            expected_revision=self.fixture.revision, expected_environment_sha256=self.environment_sha)

    def test_missing_duplicate_and_unknown_coverage(self):
        for field in ('case_results', 'measurements', 'native_observations', 'gate_results'):
            for change in ('missing', 'duplicate', 'unknown'):
                report = deepcopy(self.fixture.report)
                if change == 'missing':
                    report[field].pop()
                elif change == 'duplicate':
                    report[field].append(deepcopy(report[field][0]))
                else:
                    id_field = {'case_results': 'case_id', 'measurements': 'budget_id',
                                'native_observations': 'scenario_id', 'gate_results': 'id'}[field]
                    report[field][0][id_field] = 'unknown'
                with self.subTest(field=field, change=change), self.assertRaises(EvidenceError):
                    self.check(report)

    def test_matching_noop_command_log_cannot_replace_fixture_recipe(self):
        report = deepcopy(self.fixture.report)
        bundle = self.fixture.refresh()
        row = report['case_results'][0]
        log = bundle.json(row['log'])
        row['command'] = log['command'] = ['true']
        row['log'] = self.fixture.put(log)
        report['artifacts'] = list(self.fixture.artifacts.values())
        from global_preferences_acceptance.identity import validate_environments
        from global_preferences_acceptance.observations import validate_cases
        with self.assertRaises(EvidenceError):
            validate_cases(report, Bundle(self.fixture.directory, report['artifacts']),
                           validate_environments(report['environments']),
                           {k: v['sha256'] for k, v in report['candidate']['binaries'].items()})

    def test_case_recipe_parameters_and_interpreter_are_not_substitutable(self):
        from global_preferences_acceptance.identity import validate_environments
        from global_preferences_acceptance.observations import validate_cases
        bundle = self.fixture.refresh()
        original = self.fixture.report['case_results'][0]['command']
        fixture_argument = json.loads(original[3])
        fixture_argument['parameters']['coverage'][1] = 'other-variant'
        substitutions = [
            ['python3', 'scripts/other_recipe.py', *original[2:]],
            [*original[:3], json.dumps(fixture_argument)],
            original[:2],
            [*original, '--skip-case'],
            ['sh', '-c', 'true'],
        ]
        for command in substitutions:
            report = deepcopy(self.fixture.report)
            row = report['case_results'][0]
            log = bundle.json(row['log'])
            row['command'] = log['command'] = command
            row['log'] = self.fixture.put(log)
            report['artifacts'] = list(self.fixture.artifacts.values())
            with self.subTest(command=command[:3]), self.assertRaisesRegex(EvidenceError, 'frozen fixture recipe and parameters'):
                validate_cases(report, Bundle(self.fixture.directory, report['artifacts']),
                               validate_environments(report['environments']),
                               {k: v['sha256'] for k, v in report['candidate']['binaries'].items()})

    def test_identity_refusals(self):
        for field in ('candidate_revision', 'input_manifest_sha256', 'matrix_sha256', 'binary_sha256'):
            report = deepcopy(self.fixture.report)
            report['case_results'][0][field] = '0' * (40 if field == 'candidate_revision' else 64)
            with self.subTest(field=field), self.assertRaises(EvidenceError):
                self.check(report)
        for field, value in [('executed_tests', 0), ('exit_code', 1), ('assertions', []),
                             ('assertions', [None]), ('assertions', [{'id': []}])]:
            report = deepcopy(self.fixture.report)
            report['case_results'][0][field] = value
            with self.subTest(field=field), self.assertRaises(EvidenceError):
                self.check(report)

    def test_missing_review_native_capture_and_substituted_gate(self):
        for field in ('independent_review', 'release_notes'):
            report = deepcopy(self.fixture.report)
            report[field] = {'path': 'missing', 'sha256': '0' * 64}
            with self.assertRaises(EvidenceError):
                self.check(report)
        report = deepcopy(self.fixture.report)
        report['gate_results'][0]['commands'][0]['command'] = ['true']
        with self.assertRaisesRegex(EvidenceError, 'substituted'):
            self.check(report)
        report = deepcopy(self.fixture.report)
        report['native_observations'][1]['capture'] = report['native_observations'][0]['capture']
        with self.assertRaisesRegex(EvidenceError, 'capture execution identity'):
            self.check(report)

    def test_changed_live_input_and_new_untracked_input(self):
        # Historical proof remains about its original candidate. New execution must
        # still refuse the same changed worktree, rather than relabeling it as that pin.
        path = self.fixture.root / 'scripts/fixture.py'
        original = path.read_bytes()
        try:
            path.write_bytes(b'changed input')
            self.assertEqual(self.check(self.fixture.report)['candidate'], self.fixture.revision)
            with self.assertRaisesRegex(EvidenceError, 'current proof input changed'):
                validate_live_inputs(self.fixture.root, self.fixture.revision)
        finally:
            path.write_bytes(original)
        extra = self.fixture.root / 'scripts/unreviewed.py'
        try:
            extra.write_bytes(b'unreviewed')
            self.assertEqual(self.check(self.fixture.report)['candidate'], self.fixture.revision)
            with self.assertRaisesRegex(EvidenceError, 'new or removed proof input'):
                validate_live_inputs(self.fixture.root, self.fixture.revision)
        finally:
            extra.unlink()
        mode = path.stat().st_mode
        try:
            path.chmod(0o755)
            self.assertEqual(self.check(self.fixture.report)['candidate'], self.fixture.revision)
            with self.assertRaisesRegex(EvidenceError, 'executable mode'):
                validate_live_inputs(self.fixture.root, self.fixture.revision)
        finally:
            path.chmod(mode)

    def test_original_matrix_and_candidate_remain_bound_after_live_edits(self):
        path = self.fixture.root / 'specs/global_preferences_production_acceptance_matrix.json'
        original = path.read_bytes()
        try:
            path.write_bytes(b'{"unrelated_current_matrix":true}')
            result = self.check(self.fixture.report)
            self.assertEqual(result['candidate'], self.fixture.revision)
            self.assertFalse(result['production_accepted'])
            with self.assertRaisesRegex(EvidenceError, 'current proof input changed'):
                validate_live_inputs(self.fixture.root, self.fixture.revision)
            report = deepcopy(self.fixture.report)
            report['candidate']['revision'] = '0' * 40
            with self.assertRaisesRegex(EvidenceError, 'different requested candidate'):
                self.check(report)
        finally:
            path.write_bytes(original)

    def test_manifest_cannot_be_rewritten_to_match_changed_worktree(self):
        report = deepcopy(self.fixture.report)
        entries = self.fixture.refresh().json(report['input_manifest'])
        entries[0]['sha256'] = '0' * 64
        # A content-addressed rewrite cannot turn altered inputs into the recorded Git pin.
        report['input_manifest'] = self.fixture.put(entries)
        build = report['candidate']['build_commands'][0]
        build['input_manifest_sha256'] = report['input_manifest']['sha256']
        log = self.fixture.refresh().json(build['log'])
        log['input_manifest_sha256'] = report['input_manifest']['sha256']
        build['log'] = self.fixture.put(log)
        report['artifacts'] = list(self.fixture.artifacts.values())
        with self.assertRaisesRegex(EvidenceError, 'candidate input digest differs'):
            self.check(report)

    def test_build_refuses_changes_before_and_during_capture(self):
        from global_preferences_acceptance import candidate_identity as ci
        from global_preferences_acceptance.capture import Capture
        path = self.fixture.root / 'scripts/fixture.py'
        original = path.read_bytes()
        capture = Capture(self.fixture.directory)
        try:
            path.write_bytes(b'changed before build')
            with patch.object(ci, 'release_build_observation') as build:
                with self.assertRaisesRegex(EvidenceError, 'current proof input changed'):
                    ci.build_candidate(self.fixture.root, capture, self.fixture.revision)
                build.assert_not_called()
            path.write_bytes(original)

            def concurrent_edit(*args):
                path.write_bytes(b'changed during build')
                return self.fixture.report['candidate']['build_commands'][0]

            with patch.object(ci, 'release_build_observation', side_effect=concurrent_edit), \
                 patch.object(ci, 'retain_binaries', return_value=self.fixture.report['candidate']['binaries']):
                with self.assertRaisesRegex(EvidenceError, 'current proof input changed'):
                    ci.build_candidate(self.fixture.root, capture, self.fixture.revision)
        finally:
            path.write_bytes(original)

    def test_stale_matrix_environment_and_historical_report(self):
        for field, value in [('matrix_sha256', '0' * 64), ('environments', []),
                             ('schema', 'datum-global-preferences-production-evidence-v1')]:
            report = deepcopy(self.fixture.report)
            report[field] = value
            with self.subTest(field=field), self.assertRaises(EvidenceError):
                self.check(report)

    def test_review_binds_exact_material_and_refuses_malformed_replay(self):
        bundle = self.fixture.refresh()
        report = deepcopy(self.fixture.report)
        release = bundle.json(report['release_notes'])
        release['clean_user_launch'] = self.fixture.put(b'different launch capture')
        report['release_notes'] = self.fixture.put(release)
        report['artifacts'] = list(self.fixture.artifacts.values())
        with self.assertRaisesRegex(EvidenceError, 'selects different evidence'):
            self.check(report)
        report = deepcopy(self.fixture.report)
        review = bundle.json(report['independent_review'])
        log = bundle.json(review['replay'][0]['log'])
        log['stdout'] = self.fixture.put([])
        review['replay'][0]['log'] = self.fixture.put(log)
        report['independent_review'] = self.fixture.put(review)
        report['artifacts'] = list(self.fixture.artifacts.values())
        with self.assertRaisesRegex(EvidenceError, 'replay verdict'):
            self.check(report)

    def test_each_omitted_case_and_gate_is_rejected(self):
        from global_preferences_acceptance.observations import validate_cases, validate_gates
        report = deepcopy(self.fixture.report)
        for index in range(len(report['case_results'])):
            changed = {**report, 'case_results': report['case_results'][:index] + report['case_results'][index+1:]}
            with self.assertRaisesRegex(EvidenceError, 'count differs'):
                validate_cases(changed, None, {}, {})
        for index in range(len(report['gate_results'])):
            changed = {**report, 'gate_results': report['gate_results'][:index] + report['gate_results'][index+1:]}
            with self.assertRaisesRegex(EvidenceError, 'count differs'):
                validate_gates(changed, None, self.fixture.root)

    def test_artifact_digest_symlink_and_traversal(self):
        with tempfile.TemporaryDirectory() as directory:
            root = Path(directory)
            (root / 'file').write_bytes(b'actual')
            with self.assertRaises(EvidenceError):
                Bundle(root, [{'path': 'file', 'sha256': digest(b'other')}])
            (root / 'link').symlink_to(root / 'file')
            for name in ('../file', '/file', './file', '.', 'link'):
                with self.subTest(name=name), self.assertRaises((EvidenceError, OSError)):
                    Bundle(root, [{'path': name, 'sha256': digest(b'actual')}])

    def test_cli_historical_and_malformed_never_claim_ready(self):
        script = Path(__file__).parent / 'check_global_preferences_production_evidence.py'
        result = subprocess.run([sys.executable, str(script)], capture_output=True, text=True)
        self.assertEqual(result.returncode, 0, result.stderr)
        self.assertEqual(json.loads(result.stdout)['state'], 'historical_incomplete')
        result = subprocess.run([sys.executable, str(script), '--ready'], capture_output=True, text=True)
        self.assertNotEqual(result.returncode, 0)
        self.assertIn('--report', result.stderr)
        with tempfile.NamedTemporaryFile() as malformed:
            malformed.write(b'{"schema":NaN}')
            malformed.flush()
            result = subprocess.run([sys.executable, str(script), '--ready', '--report', malformed.name,
                                     '--bundle-root', self.temporary.name, '--candidate', self.fixture.revision,
                                     '--environment-sha256', self.environment_sha], capture_output=True, text=True)
            self.assertEqual(result.returncode, 1)
            self.assertEqual(json.loads(result.stdout)['state'], 'incomplete')
            self.assertNotIn('Traceback', result.stderr)


if __name__ == '__main__':
    unittest.main()
