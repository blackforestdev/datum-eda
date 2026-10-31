#!/usr/bin/env python3
"""Run every standing gate serially and validate the retained complete V2 corpus."""
import argparse
import json
from pathlib import Path

from global_preferences_acceptance.capture import Capture
from global_preferences_acceptance.gates import commands
from global_preferences_acceptance.identity import validate_candidate, validate_environments, validate_live_inputs
from global_preferences_acceptance.inputs import Bundle, EvidenceError, canonical, digest, parse, require
from global_preferences_acceptance import inventory
from global_preferences_acceptance.matrix import validate_matrix
from global_preferences_acceptance.observations import validate_cases, validate_native
from global_preferences_acceptance.report import validate_measurements, validate_report
from project_feature_delivery import _ordinary_record

ROOT = Path(__file__).resolve().parents[1]


def validate_entry(report, matrix, bundle, ordinary_evidence, candidate, environment_sha256):
    """Refuse an unproducible campaign before spending on its whole-workspace gates."""
    dispositions = (*inventory.SURFACE_DISPOSITIONS.values(),
                    *inventory.NATIVE_DISPOSITIONS.values(),
                    *inventory.MEASUREMENT_DISPOSITIONS.values())
    require(not any(d.get('qualification_followup') or d.get('mode') in {'scope_exit', 'env_blocked'}
                    for d in dispositions),
            'This bar cannot be met as written: unresolved coverage dispositions; '
            'PM bar review must reduce the authorized bar or escalate to the owner before qualification')
    frontier = parse((ROOT / 'specs/active_frontier.json').read_bytes(), 'frontier')
    tasks = [item for item in frontier['frontier'] if item['key'] == 'GLOBAL-PREFERENCES-COMPLETION']
    require(len(tasks) == 1, 'ordinary workflow contract missing or ambiguous')
    failures = _ordinary_record(ROOT, str(ordinary_evidence), tasks[0]['feature_delivery'])
    require(not failures, '; '.join(failures))
    ordinary = parse((ROOT / ordinary_evidence).read_bytes(), 'ordinary application evidence')
    require(ordinary['source_commit'] == candidate, 'ordinary workflow evidence is for a different candidate')
    require(report['schema'] == 'datum-global-preferences-production-evidence-v2', 'unknown report schema')
    require(digest(canonical(report['environments'])) == environment_sha256,
            'observed environments differ from the selected reference environment record')
    require(report['matrix_sha256'] == digest(canonical(matrix)), 'report matrix digest differs')
    binaries = validate_candidate(ROOT, report['candidate'], report['input_manifest'], bundle, candidate)
    environments = validate_environments(report['environments'])
    validate_cases(report, bundle, environments, binaries)
    validate_native(report, bundle, environments, binaries)
    validate_measurements(report, bundle, environments, binaries)


def main(argv=None):
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument('--report', type=Path, required=True, help='retained case, native, measurement and review evidence')
    parser.add_argument('--bundle-root', type=Path, required=True)
    parser.add_argument('--candidate', required=True)
    parser.add_argument('--environment-sha256', required=True)
    parser.add_argument('--ordinary-evidence', type=Path, required=True,
                        help='repository-relative ordinary_application record for the candidate workflow and safety checks')
    parser.add_argument('--output', type=Path, required=True, help='new report; existing evidence is never overwritten')
    parser.add_argument('--timeout', type=int, default=3600, help='per-command timeout; timeout is failure')
    args = parser.parse_args(argv)
    if args.timeout <= 0:
        parser.error('--timeout must be positive')
    try:
        require(not args.output.exists(), 'output already exists; preserve previous proof')
        report = parse(args.report.read_bytes(), 'report')
        require(report['origin']['kind'] == 'product-execution', 'synthetic fixtures are not executable production proof')
        matrix = parse((ROOT / 'specs/global_preferences_production_acceptance_matrix.json').read_bytes(), 'matrix')
        validate_matrix(matrix)
        bundle = Bundle(args.bundle_root, report['artifacts'])
        validate_entry(report, matrix, bundle, args.ordinary_evidence, args.candidate, args.environment_sha256)
        validate_live_inputs(ROOT, args.candidate)
        capture = Capture(args.bundle_root, report['artifacts'])
        report['gate_results'] = []
        failed = False
        for gate, argv_list in commands(ROOT).items():
            row = {'id': gate, 'candidate_revision': args.candidate,
                   'input_manifest_sha256': report['input_manifest']['sha256'], 'commands': []}
            report['gate_results'].append(row)
            for command in argv_list:
                print('Running ' + gate + ': ' + ' '.join(command), flush=True)
                observation = capture.command(command, cwd=ROOT, candidate=args.candidate,
                                              inputs=report['input_manifest']['sha256'], timeout=args.timeout)
                row['commands'].append(observation)
                if observation['exit_code'] != 0:
                    failed = True
                    break
            if failed:
                break
        report['artifacts'] = list(capture.inventory.values())
        with args.output.open('xb') as stream:
            stream.write(canonical(report) + b'\n')
        validate_live_inputs(ROOT, args.candidate)
        require(not failed, 'mandatory gate failed; incomplete report and raw failure logs retained')
        result = validate_report(report, matrix, root=ROOT, bundle_root=args.bundle_root, review_only=True,
                                 expected_revision=args.candidate, expected_environment_sha256=args.environment_sha256)
        print(json.dumps(result, sort_keys=True))
        return 0
    except (EvidenceError, OSError, KeyError, TypeError) as error:
        print(json.dumps({'state': 'incomplete', 'production_accepted': False, 'error': str(error)}))
        return 1


if __name__ == '__main__':
    raise SystemExit(main())
