#!/usr/bin/env python3
"""Retain raw standalone CLI observations; this partial collector never issues readiness."""
import argparse
import json
import os
from pathlib import Path
import tempfile
import uuid

from global_preferences_acceptance.capture import Capture
from global_preferences_acceptance.inputs import EvidenceError, canonical, digest, parse, require
from global_preferences_acceptance.measurements import percentile
from global_preferences_acceptance.process import execute

ROOT = Path(__file__).resolve().parents[1]
KEY = 'datum.accessibility.reduced_motion'


def isolated_environment(configuration):
    environment = {k: v for k, v in os.environ.items() if k not in
                   {'DATUM_ENGINE_SOCKET', 'EDA_ENGINE_SOCKET', 'XDG_CONFIG_HOME', 'DATUM_GUI_PREFERENCES_PATH'}}
    environment['XDG_CONFIG_HOME'] = str(configuration)
    return environment


def mutation(action, expected, identity, *, key=KEY, value=True):
    command = ['--format', 'json', 'preferences', action, key, '--expected', json.dumps(expected),
               '--request-id', str(uuid.uuid5(uuid.NAMESPACE_URL, identity)), '--reason', 'GP-CM05E release proof']
    if action == 'set':
        command += ['--value-json', json.dumps(value)]
    return command


def observe(binary, command, environment, capture, *, confirmation=False, timeout=15):
    value = execute(binary, command, environment, timeout=timeout, confirmation=confirmation)
    stdout = value.pop('stdout')
    stderr = value.pop('stderr')
    value['stdout'] = capture.put(stdout)
    value['stderr'] = capture.put(stderr)
    value['binary_sha256'] = digest(binary.read_bytes())
    reference = capture.put(value)
    require(value['exit_code'] == 0 and (not confirmation or value['confirmed']),
            'command failed or timed out; raw failure retained at ' + reference['path'])
    response = None
    if confirmation:
        rendered = stdout.decode().replace('\r', '')
        start = rendered.find('{\n  "ok"')
        require(start >= 0, 'TTY command has no product envelope; raw output retained')
        response = parse(rendered[start:].encode(), 'TTY product response')
        require(type(response) is dict and response.get('ok') is True, 'mutation refused; raw response retained')
    return value, reference, response


def collect(binary, root, capture, count, timeout):
    """Three trials, isolated durable roots, all retained warm-ups and observations."""
    results = []
    for trial in range(3):
        with tempfile.TemporaryDirectory(prefix=f'trial-{trial}-', dir=root) as temporary:
            base = Path(temporary)
            configuration = base / 'config'
            configuration.mkdir()
            environment = isolated_environment(configuration)
            expected = {'kind': 'missing'}
            queries = {'describe': [], 'list': [], 'get': [KEY], 'search': ['motion'],
                       'explain': [KEY], 'preview-project-units-seed': ['--source', 'global']}
            for state in ('defaults', 'populated'):
                if state == 'populated':
                    _, _, response = observe(binary, mutation('set', expected, f'{trial}/populate'),
                                              environment, capture, confirmation=True, timeout=timeout)
                    expected = {'kind': 'generation', 'generation': response['context']['generation']}
                for verb, arguments in queries.items():
                    observations = []
                    for index in range(count + 10):
                        value, reference, _ = observe(binary, ['--format', 'json', 'preferences', verb, *arguments],
                                                       environment, capture, timeout=timeout)
                        response = parse(capture.root.joinpath(value['stdout']['path']).read_bytes(), 'query response')
                        require(type(response) is dict and response.get('ok') is True, 'query returned a failed product envelope')
                        observations.append({'index': index, 'warmup': index < 10, 'raw': reference,
                                             'elapsed_ns': value['elapsed_ns'], 'rss_kib': value['rss_kib']})
                    results.append({'budget_id': 'release-query', 'variant_id': verb + '/' + state,
                                    'trial': trial, 'observations': observations})
                    if state == 'defaults':
                        require(not (configuration / 'datum/preferences').exists(), 'query created a repository')
            for action in ('set', 'reset'):
                observations = []
                for index in range(count + 10):
                    # Reset needs a real override; setup is separate and retained.
                    if action == 'reset':
                        _, _, response = observe(binary, mutation('set', expected, f'{trial}/reset-setup/{index}'),
                                                  environment, capture, confirmation=True, timeout=timeout)
                        expected = {'kind': 'generation', 'generation': response['context']['generation']}
                    value, reference, response = observe(binary, mutation(action, expected, f'{trial}/{action}/{index}'),
                                                          environment, capture, confirmation=True, timeout=timeout)
                    expected = {'kind': 'generation', 'generation': response['context']['generation']}
                    observations.append({'index': index, 'warmup': index < 10, 'raw': reference,
                                         'elapsed_ns': value['elapsed_ns'], 'rss_kib': value['rss_kib']})
                results.append({'budget_id': 'durable-mutation', 'variant_id': action + '/standalone',
                                'trial': trial, 'observations': observations})
            results.extend(collect_projects(binary, base, environment, capture, count, timeout, trial))
    return results


def collect_projects(binary, base, environment, capture, count, timeout, trial):
    """Preserve the legacy real-Project proof, now separated by seed mode and trial."""
    results = []
    for mode in ('factory', 'global-missing', 'global-populated'):
        env = {**environment, 'XDG_CONFIG_HOME': str(base / ('config-' + mode))}
        Path(env['XDG_CONFIG_HOME']).mkdir()
        if mode == 'global-populated':
            observe(binary, mutation('set', {'kind': 'missing'}, f'{trial}/{mode}/setup',
                                     key='datum.units.system', value='imperial'), env,
                    capture, confirmation=True, timeout=timeout)
        genesis, validation, storage = [], [], []
        for index in range(count + 10):
            destination = base / f'{mode}-{index}'
            identity = f'datum:gp-cm05e:{trial}:{mode}:{index}'
            command = ['project', 'new', '--name', f'Acceptance {index}', '--project-id',
                       str(uuid.uuid5(uuid.NAMESPACE_URL, identity + '/project')), '--request-id',
                       str(uuid.uuid5(uuid.NAMESPACE_URL, identity + '/request')), '--units-source',
                       'factory' if mode == 'factory' else 'global', '--json', str(destination)]
            for arguments, observations in [(command, genesis), (['project', 'validate', str(destination)], validation)]:
                value, reference, _ = observe(binary, arguments, env, capture, timeout=timeout)
                observations.append({'index': index, 'warmup': index < 10, 'raw': reference,
                                     'elapsed_ns': value['elapsed_ns'], 'rss_kib': value['rss_kib']})
            files = []
            for path in sorted(destination.rglob('*')):
                require(not path.is_symlink(), 'published Project contains an unexpected symlink')
                if path.is_file():
                    raw = path.read_bytes()
                    reference = capture.put(raw)
                    files.append({'path': str(path.relative_to(destination)), 'bytes': len(raw), 'artifact': reference})
            storage.append({'index': index, 'warmup': index < 10, 'project_bytes': sum(f['bytes'] for f in files),
                            'manifest': capture.put({'project': str(destination), 'files': files})})
        for budget, observations in [('project-genesis', genesis), ('project-validation', validation),
                                     ('storage-growth', storage)]:
            results.append({'budget_id': budget, 'variant_id': mode, 'trial': trial, 'observations': observations})
        if mode != 'global-populated':
            require(not (Path(env['XDG_CONFIG_HOME']) / 'datum/preferences').exists(),
                    'factory/missing-store genesis created a Preferences repository')
    return results


def main(argv=None):
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument('--binary', type=Path, default=ROOT / 'target/release/datum-eda')
    parser.add_argument('--samples', type=int, default=100)
    parser.add_argument('--timeout', type=float, default=15)
    parser.add_argument('--output', type=Path, required=True, help='new directory retaining every raw command and failure')
    parser.add_argument('--fixture-root', type=Path, default=ROOT / 'target/preferences-measurement-fixtures')
    args = parser.parse_args(argv)
    if args.samples < 1 or args.timeout <= 0:
        parser.error('sample count and timeout must be positive')
    binary = args.binary.resolve()
    try:
        require(binary.is_file() and os.access(binary, os.X_OK), 'release binary must be executable')
        root = args.fixture_root.resolve()
        require(root.is_relative_to(ROOT) and not root.is_relative_to(Path('/tmp')),
                'durable fixtures belong on project-backed storage, not /tmp')
        root.mkdir(parents=True, exist_ok=True)
        args.output.mkdir(parents=True, exist_ok=False)
        capture = Capture(args.output)
        report = {'schema': 'datum.preferences.partial-release-capture.v2', 'state': 'partial_diagnostic',
                  'production_accepted': False, 'binary_sha256': digest(binary.read_bytes()),
                  'samples_per_trial': args.samples, 'warmups_per_trial': 10, 'trials': 3,
                  'missing': ['reachable-daemon mutation', 'continuous generation storage capture',
                              'native feedback/window/lifecycle', 'complete candidate proof and independent review']}
        try:
            report['measurements'] = collect(binary, root, capture, args.samples, args.timeout)
            for row in report['measurements']:
                if row['budget_id'] == 'storage-growth':
                    continue
                measured = [o['elapsed_ns'] for o in row['observations'] if not o['warmup']]
                row['derived_p95_ns'] = percentile(measured)
                row['derived_max_ns'] = max(measured)
        except (EvidenceError, OSError, ValueError) as error:
            report['error'] = str(error)
        report['artifacts'] = list(capture.inventory.values())
        (args.output / 'capture.json').write_bytes(canonical(report) + b'\n')
        print(json.dumps({'state': report['state'], 'production_accepted': False,
                          'capture': str(args.output / 'capture.json'), 'error': report.get('error')}))
        return 1 if 'error' in report else 0
    except (EvidenceError, OSError) as error:
        print(json.dumps({'state': 'incomplete', 'production_accepted': False, 'error': str(error)}))
        return 1


if __name__ == '__main__':
    raise SystemExit(main())
