"""Synthetic regression fixtures only: these can never receive a readiness verdict."""
from copy import deepcopy
import json
from pathlib import Path
import subprocess

from .gates import commands, release_build
from .inputs import Bundle, canonical, digest
from .inventory import NATIVE_ASSERTIONS, PRODUCT_BOUNDARY
from .matrix import case_inventory, gui_coordinates
from .measurements import measurement_inventory
from .observations import case_command, observation_context


class SyntheticBundle:
    def __init__(self, directory, matrix):
        self.root = Path(directory) / 'repo'
        self.root.mkdir()
        self.directory = Path(directory) / 'bundle'
        self.directory.mkdir()
        self.artifacts = {}
        files = {'Cargo.lock': b'# synthetic lock\n', 'Cargo.toml': b'# synthetic workspace\n',
                 'scripts/fixture.py': b'# Synthetic test recipe, not product evidence.\n',
                 'specs/global_preferences_production_acceptance_matrix.json': canonical(matrix),
                 'specs/evidence_traceability_manifest.json': canonical({'routes': [{
                     'id': 'workspace-documentation-and-revision', 'sources': [],
                     'consumers': ['specs/global_preferences_production_acceptance_matrix.json']}]})}
        for path, raw in files.items():
            target = self.root / path
            target.parent.mkdir(parents=True, exist_ok=True)
            target.write_bytes(raw)
        self.git('init', '-q')
        self.git('add', *files)
        self.git('-c', 'user.name=Regression Fixture', '-c', 'user.email=fixture@invalid',
                 '-c', 'core.hooksPath=/dev/null', 'commit', '-qm', 'Synthetic validation fixture')
        self.revision = self.git('rev-parse', 'HEAD').strip()
        self.input = self.put([{'path': p, 'sha256': digest(raw)} for p, raw in sorted(files.items())])
        self.recipe = {'path': 'scripts/fixture.py', 'sha256': digest(files['scripts/fixture.py'])}
        environments = [self.environment(None), *(self.environment(c) for c in gui_coordinates())]
        self.environment_ids = {None: 'headless', **{c: f'gui-{i}' for i, c in enumerate(gui_coordinates())}}
        for coordinate, identifier in self.environment_ids.items():
            environments[list(self.environment_ids).index(coordinate)]['id'] = identifier
        self.report = {'schema': 'datum-global-preferences-production-evidence-v2',
                       'origin': {'kind': 'synthetic-test', 'producer_session': 'synthetic-producer'},
                       'candidate': {'revision': self.revision, 'tree': self.git('rev-parse', 'HEAD^{tree}').strip(),
                                     'cargo_lock_sha256': digest(files['Cargo.lock']),
                                     'toolchains': {'rustc': 'test', 'cargo': 'test', 'python': 'test'},
                                     'build_commands': [self.command(release_build())],
                                     'binaries': {role: self.put(role.encode()) for role in
                                                  ('cli', 'gui', 'daemon', 'engine', 'gui-render', 'mcp')}},
                       'input_manifest': self.input, 'matrix_sha256': digest(canonical(matrix)),
                       'environments': environments, 'case_results': [], 'native_observations': [],
                       'measurements': [], 'gate_results': []}
        self.state = self.put({'schema': 'datum.preferences.state.v2', 'root': 'synthetic-root', 'files': []})
        for key, required in case_inventory().items():
            case, variant, subcase, surface, coordinate = key
            fixture = self.fixture(key, required)
            row = {**self.identity(surface, coordinate), 'case_id': case, 'variant_id': variant,
                   'subcase_id': subcase, 'surface': surface, 'fixture': fixture,
                   'executed_tests': 1, 'before': self.state, 'after': self.state}
            row['assertions'] = self.assertions(required, observation_context(row))
            row.update(self.command(case_command(json.loads((self.directory / fixture['path']).read_bytes())),
                                    {r: True for r in required}))
            self.report['case_results'].append(row)
        for scenario, required in NATIVE_ASSERTIONS.items():
            for coordinate in gui_coordinates():
                row = {**self.identity('gui', coordinate), 'scenario_id': scenario,
                       'fixture': self.fixture((scenario, coordinate), required),
                       'actions': ['synthetic input'], 'state_evidence': self.state}
                context = observation_context(row)
                row['assertions'] = self.assertions(required, context)
                for field, kind in [('capture', 'native'), ('accessibility_capture', 'at-spi')]:
                    row[field] = self.put({'schema': 'datum.preferences.native-capture.v2',
                                          'context': context, 'kind': kind, 'actions': row['actions'],
                                          'tool': 'synthetic-only', 'payload': self.put(b'synthetic capture')})
                self.report['native_observations'].append(row)
        for budget, variant, coordinate in sorted(measurement_inventory(), key=str):
            row = {**self.identity('gui' if coordinate else 'cli', coordinate),
                   'budget_id': budget, 'variant_id': variant, 'trials': []}
            for index in range(3):
                sample = self.sample(budget, variant)
                trial = {'index': index, 'warmup_count': 10,
                         'warmups': [{**deepcopy(sample), 'index': n, 'started_monotonic_ns': n + 2} for n in range(10)],
                         'baseline_rss_kib': 1024 if budget == 'window-lifecycle' else None,
                         'baseline_resources': {'owned_windows': [], 'writer_leases': []}
                         if budget == 'window-lifecycle' else None,
                         'cold_open': deepcopy(sample) if budget == 'native-window-open' else None,
                         'samples': [{**deepcopy(sample), 'index': n, 'started_monotonic_ns': n + 12} for n in range(100)]}
                for observation in ([trial['cold_open']] if trial['cold_open'] else []) + trial['warmups'] + trial['samples']:
                    observation['started_monotonic_ns'] += index * 1000
                if budget == 'storage-growth' and variant == 'generation':
                    self.storage_history(trial)
                row['trials'].append(trial)
            self.rebind_trials(row, capture_samples=True)
            self.report['measurements'].append(row)
        for gate, argv in commands(self.root).items():
            self.report['gate_results'].append({'id': gate, 'candidate_revision': self.revision,
                'input_manifest_sha256': self.input['sha256'], 'commands': [self.command(c) for c in argv]})
        self.report['release_notes'] = self.put({
            'schema': 'datum.preferences.release-handoff.v2', 'candidate_revision': self.revision,
            'input_manifest_sha256': self.input['sha256'], 'environment_ids': list(self.environment_ids.values()),
            'binaries': self.report['candidate']['binaries'], 'boundaries': PRODUCT_BOUNDARY,
            'support': {k: self.put(b'synthetic support') for k in ('settings_locations', 'diagnostics', 'errors',
                       'recovery', 'compatibility', 'storage', 'retained_history', 'known_exclusions')},
            'clean_user_launch': self.put(b'synthetic launch')})
        self.report['independent_review'] = self.put({
            'schema': 'datum.preferences.independent-review.v2', 'producer_session': 'synthetic-producer',
            'reviewer_session': 'synthetic-reviewer', 'candidate_revision': self.revision,
            'input_manifest_sha256': self.input['sha256'], 'matrix_sha256': self.report['matrix_sha256'],
            'review_subject_sha256': self.subject(), 'replay': self.replays(),
            'finding_dispositions': [], 'exclusions': PRODUCT_BOUNDARY['excluded']})
        self.refresh()

    def git(self, *args):
        return subprocess.check_output(['git', *args], cwd=self.root, stderr=subprocess.PIPE).decode()

    def put(self, value):
        raw = value if isinstance(value, bytes) else canonical(value)
        sha = digest(raw)
        path = f'objects/{sha}'
        target = self.directory / path
        target.parent.mkdir(exist_ok=True)
        target.write_bytes(raw)
        reference = {'path': path, 'sha256': sha}
        self.artifacts[path] = reference
        return reference

    def refresh(self):
        self.report['artifacts'] = list(self.artifacts.values())
        return Bundle(self.directory, self.report['artifacts'])

    def identity(self, surface, coordinate):
        return {'candidate_revision': self.revision, 'input_manifest_sha256': self.input['sha256'],
                'matrix_sha256': self.report['matrix_sha256'], 'environment_id': self.environment_ids[coordinate],
                'binary_sha256': self.report['candidate']['binaries'][surface]['sha256']}

    def fixture(self, key, required):
        return self.put({'schema': 'datum.preferences.fixture.v2', 'source_revision': self.revision,
                         'recipe': self.recipe, 'parameters': {'coverage': key}, 'initial_state': self.state,
                         'expected_assertions': {a: {'operator': 'equal', 'value': True} for a in required}})

    def assertions(self, required, context):
        return [{'id': a, 'expected': {'operator': 'equal', 'value': True}, 'observed': True,
                 'evidence': self.put({'schema': 'datum.preferences.assertion.v2', 'context': context,
                                       'id': a, 'observed': True})} for a in required]

    def command(self, argv, observations=None, raw_output=None):
        count = 1 if observations else 0
        stdout = self.put({'schema': 'datum.preferences.case-observations.v2', 'executed_tests': count,
                           'observations': observations}) if observations else self.put(b'synthetic gate output')
        if not observations and ('unittest' in argv or 'test' in argv):
            count = 1
            stdout = self.put(b'test result: ok. 1 passed; 0 failed;\n')
        if raw_output is not None:
            stdout = self.put(raw_output)
        raw = {'schema': 'datum.preferences.command-observation.v2', 'command': argv, 'exit_code': 0,
               'executed_tests': count, 'stdout': stdout, 'stderr': self.put(b''),
               'candidate_revision': self.revision, 'input_manifest_sha256': self.input['sha256']}
        return {k: raw[k] for k in ('command', 'exit_code', 'candidate_revision', 'input_manifest_sha256')} | {'log': self.put(raw)}

    def subject(self):
        from .report import review_subject
        return review_subject(self.report)

    def storage_history(self, trial):
        files = []
        before = self.state
        for index, sample in enumerate(trial['warmups'] + trial['samples']):
            for prefix in ('generations', 'receipts', 'request-index'):
                files.append({'path': f'{prefix}/{index:04}', 'kind': 'file', 'bytes': 1, 'sha256': digest(b'x')})
            after = self.put({'schema': 'datum.preferences.state.v2', 'root': 'synthetic-root',
                              'files': sorted(files, key=lambda f: f['path'])})
            sample.update(before=before, after=after)
            before = after

    def replays(self):
        rows = []
        evidence = self.command(['python3', 'scripts/check_global_preferences_production_evidence.py', '--review',
                                '--report', 'synthetic.json', '--bundle-root', 'synthetic', '--candidate', self.revision,
                                '--environment-sha256', digest(canonical(self.report['environments']))], raw_output={
                                    'state': 'reviewable', 'candidate': self.revision,
                                    'input_manifest_sha256': self.input['sha256'], 'matrix_sha256': self.report['matrix_sha256'],
                                    'review_subject_sha256': self.subject(), 'report_sha256': digest(canonical(self.report)),
                                    'production_accepted': False})
        rows.append({'category': 'evidence', 'coverage': ['complete-report'],
                     **{k: evidence[k] for k in ('command', 'exit_code', 'log')}})
        for category, family in [('durability', 'project-genesis-crash'), ('security', 'human-agent-authority')]:
            selected = next(r for r in self.report['case_results'] if r['case_id'] == family)
            rows.append({'category': category, 'coverage': [selected[k] for k in
                         ('case_id', 'variant_id', 'subcase_id', 'surface', 'environment_id')],
                         **{k: selected[k] for k in ('command', 'exit_code', 'log')}})
        selected = self.report['native_observations'][0]
        coverage = [selected['scenario_id'], selected['environment_id']]
        native = self.command(['python3', 'scripts/fixture.py', '--native-scenario', selected['scenario_id'],
                               '--environment', selected['environment_id']], raw_output={
                                  'schema': 'datum.preferences.native-replay.v2', 'candidate_revision': self.revision,
                                  'input_manifest_sha256': self.input['sha256'], 'matrix_sha256': self.report['matrix_sha256'],
                                  'coverage': coverage, 'capture': selected['capture'],
                                  'accessibility_capture': selected['accessibility_capture']})
        rows.append({'category': 'native', 'coverage': coverage, **{k: native[k] for k in ('command', 'exit_code', 'log')}})
        return rows

    def rebind_trials(self, row, *, capture_samples=False):
        for trial in row['trials']:
            if capture_samples:
                for phase in ('cold_open', 'warmups', 'samples'):
                    samples = ([trial[phase]] if trial[phase] else []) if phase == 'cold_open' else trial[phase]
                    for sample in samples:
                        sample['evidence'] = self.put({
                            'schema': 'datum.preferences.measurement-capture.v2',
                            'measurement': {k: v for k, v in row.items() if k != 'trials'},
                            'trial_index': trial['index'], 'phase': phase, 'exit_code': 0, 'timed_out': False,
                            'observation': {k: v for k, v in sample.items() if k != 'evidence'}})
            trial['log'] = self.put({k: v for k, v in trial.items() if k != 'log'} | {
                'schema': 'datum.preferences.measurement-trial.v2',
                'measurement': {k: v for k, v in row.items() if k != 'trials'}})

    def sample(self, budget, variant):
        common = {'index': 0, 'started_monotonic_ns': 1, 'outcome': 'success', 'evidence': self.put(b'synthetic sample capture')}
        if budget == 'window-lifecycle':
            return common | {'kind': 'lifecycle', 'post_close_rss_kib': 1024, 'orphan_windows': 0,
                             'orphan_writer_leases': 0, 'owned_windows': [], 'writer_leases': []}
        if budget == 'storage-growth':
            self.put(b'x')
            sizes = {name: 1 if (variant == 'generation' and name != 'project') or
                     (variant != 'generation' and name == 'project') else 0
                     for name in ('project', 'generation', 'receipt', 'request_index')}
            files = [{'path': path + '/new', 'kind': 'file', 'bytes': sizes[name], 'sha256': digest(b'x')}
                     for path, name in [('project', 'project'), ('generations', 'generation'),
                                        ('receipts', 'receipt'), ('request-index', 'request_index')]
                     if sizes[name]]
            after = self.put({'schema': 'datum.preferences.state.v2', 'root': 'synthetic-root',
                              'files': sorted(files, key=lambda f: f['path'])})
            return common | {'kind': 'storage', **{k + '_bytes': v for k, v in sizes.items()},
                             'fixed_overhead_bytes': 0, 'before': self.state, 'after': after}
        return common | {'kind': 'timed', 'elapsed_ns': 1, 'rss_kib': 1,
                         'daemon_rss_kib': 1 if budget == 'durable-mutation' and variant.endswith('/daemon') else None,
                         'launch_to_response_ns': 2 if budget == 'durable-mutation' else None,
                         'presented_elapsed_ns': 1 if budget in ('gui-feedback', 'native-window-open') else None}

    @staticmethod
    def environment(coordinate):
        value = {k: 'synthetic' for k in ('id', 'kernel', 'cpu', 'filesystem', 'mount_options',
                 'storage_device', 'locale', 'network_state', 'power_mode', 'background_load', 'clock')}
        value.update(os='Linux', architecture='x86_64', memory_mib=1024, tool_versions={'python': 'synthetic'})
        value.update(gpu='synthetic' if coordinate else None, driver='synthetic' if coordinate else None,
                     compositor_or_server='synthetic' if coordinate else None,
                     display_backend=coordinate[0] if coordinate else None,
                     scale_factor=coordinate[1] if coordinate else None,
                     window_size=list(coordinate[2]) if coordinate else None)
        return value
