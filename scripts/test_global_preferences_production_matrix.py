"""Exact coverage and budget regressions for the acceptance instructions."""
from copy import deepcopy
import json
from pathlib import Path
import unittest

from global_preferences_acceptance.gates import commands
from global_preferences_acceptance.inputs import EvidenceError, parse
from global_preferences_acceptance.inventory import BUDGETS, GATES
from global_preferences_acceptance.matrix import case_inventory, validate_matrix
from global_preferences_acceptance.measurements import (
    measurement_env_blocked_variants, measurement_inventory, measurement_matrix)
from global_preferences_acceptance.observations import (
    native_inventory, native_matrix, native_scope_exited)

MATRIX = json.loads((Path(__file__).resolve().parents[1] /
                    'specs/global_preferences_production_acceptance_matrix.json').read_text())


class MatrixTests(unittest.TestCase):
    def test_complete_inventory(self):
        validate_matrix(MATRIX)
        # 321 since the surface-disposition co-move (1cae5aa9): 389 minus the 68
        # dispositioned rows (engine-by-contract absorptions + scope exits).
        self.assertEqual(len(case_inventory()), 321)
        # Measurement coordinate disposition (scale-2 env-blocked): 90 full matrix minus the 32
        # scale-2 variants = 58 strict required. Native structural scope-exit: 48 full matrix minus
        # the whole corpus = 0 strict required. The full-matrix coordinate space is unchanged.
        self.assertEqual(len(measurement_matrix()), 90)
        self.assertEqual(len(measurement_env_blocked_variants()), 32)
        self.assertEqual(len(measurement_inventory()), 58)
        self.assertEqual(len(native_matrix()), 48)
        self.assertEqual(len(native_scope_exited()), 48)
        self.assertEqual(len(native_inventory()), 0)

    def test_gate_catalog_matches_commands(self):
        # Catalog-drift regression: a gate added to gates.commands() without extending the
        # explicit GATES inventory makes validate_gates reject a complete report ("unknown or
        # duplicate gate" -- the cb67dad4 feature-ON gates drifted exactly this way).
        self.assertEqual(len(GATES), len(set(GATES)))
        self.assertEqual(set(GATES), set(commands(Path(__file__).resolve().parents[1])))

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
