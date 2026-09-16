#!/usr/bin/env python3
"""Run every standing gate serially and validate the retained complete V2 corpus."""
import argparse
import json
from pathlib import Path

from global_preferences_acceptance.capture import Capture
from global_preferences_acceptance.gates import commands
from global_preferences_acceptance.identity import validate_candidate
from global_preferences_acceptance.inputs import Bundle, EvidenceError, canonical, parse, require
from global_preferences_acceptance.matrix import validate_matrix
from global_preferences_acceptance.report import validate_report

ROOT = Path(__file__).resolve().parents[1]


def main(argv=None):
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument('--report', type=Path, required=True, help='retained case, native, measurement and review evidence')
    parser.add_argument('--bundle-root', type=Path, required=True)
    parser.add_argument('--candidate', required=True)
    parser.add_argument('--environment-sha256', required=True)
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
        validate_candidate(ROOT, report['candidate'], report['input_manifest'], bundle, args.candidate)
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
