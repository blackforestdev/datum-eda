#!/usr/bin/env python3
"""Compute complete V2 readiness; historical inspection never implies readiness."""
import argparse
import json
from pathlib import Path

from global_preferences_acceptance.inputs import EvidenceError, parse
from global_preferences_acceptance.report import validate_acceptance, validate_report

ROOT = Path(__file__).resolve().parents[1]
MATRIX = ROOT / 'specs/global_preferences_production_acceptance_matrix.json'
EVIDENCE = ROOT / 'specs/evidence/preferences/gp-cm05-candidate-20260907.json'


def main(argv=None):
    parser = argparse.ArgumentParser(description=__doc__)
    mode = parser.add_mutually_exclusive_group()
    mode.add_argument('--ready', action='store_true')
    mode.add_argument('--review', action='store_true', help='complete pre-review validation; never readiness')
    mode.add_argument('--acceptance', action='store_true')
    mode.add_argument('--test-only', action='store_true', help='validate synthetic regression evidence; never readiness')
    parser.add_argument('--report', type=Path)
    parser.add_argument('--matrix', type=Path, default=MATRIX)
    parser.add_argument('--bundle-root', type=Path)
    parser.add_argument('--candidate')
    parser.add_argument('--environment-sha256')
    parser.add_argument('--owner-receipt', type=Path)
    parser.add_argument('--owner-receipt-sha256')
    args = parser.parse_args(argv)
    try:
        strict = args.ready or args.acceptance or args.test_only or args.review
        if not strict:
            if any((args.report, args.bundle_root, args.candidate, args.environment_sha256,
                    args.owner_receipt, args.owner_receipt_sha256)):
                parser.error('select --ready, --acceptance or --test-only explicitly')
            historical = parse(EVIDENCE.read_bytes(), 'historical report')
            print(json.dumps({'state': 'historical_incomplete', 'production_accepted': False,
                              'source': str(EVIDENCE.relative_to(ROOT)),
                              'recorded_schema': historical.get('schema'),
                              'reason': 'V1 aggregate results cannot establish V2 readiness'}))
            return 0
        if not all((args.report, args.bundle_root, args.candidate, args.environment_sha256)):
            parser.error('V2 requires --report, --bundle-root, --candidate and --environment-sha256')
        report = parse(args.report.read_bytes(), 'report')
        matrix = parse(args.matrix.read_bytes(), 'matrix')
        options = dict(root=ROOT, bundle_root=args.bundle_root, expected_revision=args.candidate,
                       expected_environment_sha256=args.environment_sha256)
        if args.acceptance:
            if not args.owner_receipt or not args.owner_receipt_sha256:
                parser.error('--acceptance requires the externally selected receipt and SHA-256')
            result = validate_acceptance(args.owner_receipt.read_bytes(),
                                         expected_receipt_sha256=args.owner_receipt_sha256,
                                         report=report, matrix=matrix, **options)
        else:
            if args.owner_receipt or args.owner_receipt_sha256:
                parser.error('owner receipts apply only to --acceptance')
            result = validate_report(report, matrix, test_only=args.test_only, review_only=args.review, **options)
        print(json.dumps(result, sort_keys=True))
        return 0
    except (EvidenceError, OSError, UnicodeError) as error:
        print(json.dumps({'state': 'incomplete', 'production_accepted': False, 'error': str(error)}))
        return 1


if __name__ == '__main__':
    raise SystemExit(main())
