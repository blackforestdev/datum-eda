"""Packaging refusal tests, never production or independent-replay evidence."""

from copy import deepcopy
import json
from pathlib import Path
import tempfile
import unittest

from workflow_delivery_pilot_package import merge_registry, observed_dispatches, validate_assessment


def state(enabled=True):
    return {"event": "state", "value": {"registry": [{
        "dispatch_key": "view.fit", "handler_ref": {"path": "runtime.rs", "symbol": "fit"},
        "entry_surfaces": ["pointer"],
        "contexts": {"board": {"enabled": enabled, "reason": "Available" if enabled else "Absent"}},
    }]}}


class PackageTests(unittest.TestCase):
    def test_registry_is_merged_from_actual_snapshots_without_changing_inputs(self):
        first, second = state(), state(False)
        second["value"]["registry"][0]["contexts"]["schematic"] = second["value"]["registry"][0]["contexts"].pop("board")
        trace = [first, second]
        before = deepcopy(trace)
        result = merge_registry(trace)
        self.assertEqual(set(result[0]["contexts"]), {"board", "schematic"})
        self.assertEqual(trace, before)

    def test_conflicting_registry_is_refused_not_last_writer_wins(self):
        with self.assertRaises(ValueError):
            merge_registry([state(), state(False)])
        changed = state()
        changed["value"]["registry"][0]["handler_ref"] = None
        with self.assertRaises(ValueError):
            merge_registry([state(), changed])
        with self.assertRaises(ValueError):
            merge_registry([])

    def test_zero_mutation_requires_source_and_new_path_preservation(self):
        with tempfile.TemporaryDirectory() as directory:
            root = Path(directory)
            path = root / "source-diff.json"
            trace = [{"event": "dispatch", "value": {"invoked": False}}]
            path.write_text(json.dumps({"changed_or_removed": [], "new_paths": {}}))
            self.assertEqual(observed_dispatches(trace, [root]), [{"invoked": False, "mutation_count": 0}])
            for diff in [{"changed_or_removed": ["journal.jsonl"], "new_paths": {}},
                         {"changed_or_removed": [], "new_paths": {"board.json": "new"}}]:
                path.write_text(json.dumps(diff))
                with self.assertRaises(ValueError):
                    observed_dispatches(trace, [root])

    def test_missing_or_failed_producer_findings_cannot_be_packaged(self):
        contract = {"scenarios": [{"id": "S01", "dimensions": {"normal": {"disposition": "required"}}}]}
        result = {"actual_visible": "observed", "actual_state": "observed", "defects": [],
                  "assertions": [{"dimension": "normal", "outcome": "pass"}]}
        validate_assessment({"S01": result}, contract)
        with self.assertRaises(ValueError):
            validate_assessment({}, contract)
        for assertions in [[], [{"dimension": "normal", "outcome": "fail"}]]:
            with self.assertRaises(ValueError):
                validate_assessment({"S01": dict(result, assertions=assertions)}, contract)


if __name__ == "__main__":
    unittest.main()
