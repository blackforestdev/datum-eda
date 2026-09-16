"""Exact coverage and budget regressions for the acceptance instructions."""
from copy import deepcopy
import json
from pathlib import Path
import unittest

from global_preferences_acceptance.inputs import EvidenceError, parse
from global_preferences_acceptance.inventory import BUDGETS
from global_preferences_acceptance.matrix import case_inventory, validate_matrix
from global_preferences_acceptance.measurements import measurement_inventory

MATRIX = json.loads((Path(__file__).resolve().parents[1] /
                    'specs/global_preferences_production_acceptance_matrix.json').read_text())


class MatrixTests(unittest.TestCase):
    def test_complete_inventory(self):
        validate_matrix(MATRIX)
        self.assertEqual(len(case_inventory()), 389)
        self.assertEqual(len(measurement_inventory()), 90)

    def test_each_required_case_variant_assertion_and_surface(self):
        for index, case in enumerate(MATRIX['durable_corpus']):
            changed = deepcopy(MATRIX)
            changed['durable_corpus'].pop(index)
            with self.subTest(case=case['id']), self.assertRaises(EvidenceError):
                validate_matrix(changed)
            for variant_index, variant in enumerate(case['variants']):
                for field in ('surfaces', 'assertions'):
                    for member in range(len(variant[field])):
                        changed = deepcopy(MATRIX)
                        changed['durable_corpus'][index]['variants'][variant_index][field].pop(member)
                        with self.subTest(case=case['id'], variant=variant['id'], field=field), self.assertRaises(EvidenceError):
                            validate_matrix(changed)
                changed = deepcopy(MATRIX)
                changed['durable_corpus'][index]['variants'].pop(variant_index)
                with self.assertRaises(EvidenceError):
                    validate_matrix(changed)

    def test_each_budget_and_limit_cannot_be_weakened(self):
        for index, budget in enumerate(MATRIX['candidate_resource_budgets']):
            changed = deepcopy(MATRIX)
            changed['candidate_resource_budgets'].pop(index)
            with self.assertRaises(EvidenceError):
                validate_matrix(changed)
            for limit in BUDGETS[budget['id']]:
                for value in (True, 1e50, budget['limit'][limit] + 1):
                    changed = deepcopy(MATRIX)
                    changed['candidate_resource_budgets'][index]['limit'][limit] = value
                    with self.subTest(budget=budget['id'], limit=limit, value=value), self.assertRaises(EvidenceError):
                        validate_matrix(changed)

    def test_closed_shape_platform_and_markers(self):
        for field, value in [('platform', []), ('requirement_markers', None), ('execution_step', 'GP-CM05'),
                             ('durable_corpus', [None]), ('candidate_resource_budgets', [{}])]:
            changed = deepcopy(MATRIX)
            changed[field] = value
            with self.subTest(field=field), self.assertRaises(EvidenceError):
                validate_matrix(changed)
        for field in MATRIX:
            changed = deepcopy(MATRIX)
            del changed[field]
            with self.assertRaises(EvidenceError):
                validate_matrix(changed)

    def test_json_is_strict(self):
        for raw in (b'{"a":1,"a":2}', b'{"a":NaN}', b'[Infinity]', b'{', b'"\\ud800"'):
            with self.subTest(raw=raw), self.assertRaises(EvidenceError):
                parse(raw, 'test')


if __name__ == '__main__':
    unittest.main()
