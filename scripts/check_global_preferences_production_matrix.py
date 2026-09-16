#!/usr/bin/env python3
"""Validate the exact GP-CM05E coverage and unchanged resource limits."""
from pathlib import Path

from global_preferences_acceptance.inputs import EvidenceError, parse
from global_preferences_acceptance.matrix import case_inventory, validate_matrix
from global_preferences_acceptance.measurements import measurement_inventory

ROOT = Path(__file__).resolve().parents[1]
MATRIX = ROOT / 'specs/global_preferences_production_acceptance_matrix.json'


def failures(root=ROOT):
    try:
        validate_matrix(parse((root / MATRIX.relative_to(ROOT)).read_bytes(), 'matrix'))
        return []
    except (EvidenceError, OSError) as error:
        return [str(error)]


def main():
    problems = failures()
    for problem in problems:
        print('Global Preferences production matrix: ' + problem)
    if problems:
        return 1
    print(f'GP-CM05E matrix valid: {len(case_inventory())} case rows, 48 native scenarios, '
          f'{len(measurement_inventory())} measurement variants. Product proof remains separate.')
    return 0


if __name__ == '__main__':
    raise SystemExit(main())
