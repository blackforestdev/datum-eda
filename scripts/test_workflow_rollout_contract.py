"""Prepared infrastructure contract consistency, never delivery or activation proof."""

import ast
from copy import deepcopy
from pathlib import Path
import re
import unittest

from workflow_delivery_authority import authority_manifest
from workflow_delivery_contract import validate_contract, validate_handler_files
from workflow_delivery_io import DeliveryInputError, parse_json


ROOT = Path(__file__).resolve().parents[1]
CONTRACT = "specs/workflow_delivery/rollout.contract.json"
SPEC = "specs/WORKFLOW_DELIVERY_INFRASTRUCTURE_CONTRACT.md"
MECHANISM = "docs/decisions/PRODUCT_MECHANICS_042_BROAD_WORKFLOW_DELIVERY_ENFORCEMENT.md"
WORKSPACE = "specs/workflow_delivery/workspace-inputs.json"


class RepositoryView:
    """Read current preparation artifacts without pretending they are promoted."""

    def __init__(self, overrides=None):
        self.overrides = overrides or {}

    def read(self, path):
        return self.overrides[path] if path in self.overrides else (ROOT / path).read_bytes()

    def json(self, path):
        return parse_json(self.read(path), path)


class RolloutContractTest(unittest.TestCase):
    def setUp(self):
        self.tree = RepositoryView()
        self.contract = self.tree.json(CONTRACT)

    def test_contract_identity_and_existing_schema(self):
        result = validate_contract(self.contract, CONTRACT,
            frontier_key="WORKFLOW-DELIVERY-IMPLEMENTATION",
            issue_id="dat-wdq-rollout-implementation-ffy")
        self.assertEqual("infrastructure", result["category"])
        self.assertTrue(all(s["method"] == "infrastructure" for s in result["scenarios"]))

    def test_mapping_resolves_the_prepared_contract(self):
        proposal = self.tree.json("docs/reviews/workflow-delivery-rollout/rollout-mapping.json")
        self.assertEqual(CONTRACT, proposal["delivery"]["contract_path"])
        for key in ("frontier_key", "issue_id", "category"):
            self.assertEqual(self.contract[key], proposal[key])

    def test_handlers_name_real_entry_functions_not_synthetic_product_verbs(self):
        validate_handler_files(self.tree, self.contract)
        self.assertEqual({"validator", "selector"}, {c["id"] for c in self.contract["consumers"]})
        for consumer in self.contract["consumers"]:
            handler = consumer["handler_ref"]
            module = ast.parse(self.tree.read(handler["path"]))
            self.assertIn(handler["symbol"],
                          [n.name for n in module.body if isinstance(n, ast.FunctionDef)])
            self.assertEqual(Path(handler["path"]).stem + "." + handler["symbol"],
                             consumer["dispatch_key"])

    def test_cli_snapshots_owner_hook_and_real_selector_are_covered(self):
        consumers = {c["id"]: c for c in self.contract["consumers"]}
        self.assertEqual({"staged_cli", "candidate_cli", "owner_hook"},
                         set(consumers["validator"]["entry_surfaces"]))
        self.assertEqual(["project_status"], consumers["selector"]["entry_surfaces"])
        self.assertEqual({f"INFRA-S0{i}" for i in range(1, 7)},
                         {s["id"] for s in self.contract["scenarios"]})
        for scenario in self.contract["scenarios"]:
            self.assertEqual(set(consumers), set(scenario["consumer_ids"]))
            self.assertEqual("required", scenario["dimensions"]["normal"]["disposition"])

    def test_authority_is_complete_but_not_its_own_progress_or_receipts(self):
        members = {p["path"] for p in authority_manifest(self.tree, self.contract, ready=False)}
        self.assertEqual({SPEC, CONTRACT, MECHANISM, WORKSPACE,
            "research/process-quality/WORKFLOW_INFRASTRUCTURE_CONTRACT_BASIS.md"}, members)
        self.assertNotIn(self.contract["proof_path"], members)
        self.assertNotIn(self.contract["review_path"], members)
        self.assertNotIn("specs/WORKFLOW_DELIVERY_EXECUTION_PLAN.md", members)

    def test_changed_mechanism_or_contract_requires_fresh_route_review(self):
        for path in (MECHANISM, CONTRACT, SPEC, WORKSPACE):
            with self.subTest(path=path):
                tree = RepositoryView({path: self.tree.read(path) + b"\n"})
                with self.assertRaises(DeliveryInputError) as raised:
                    authority_manifest(tree, self.contract, ready=False)
                self.assertEqual("WDQ-AUTHORITY", raised.exception.code)

    def test_workspace_policy_is_an_explicit_validated_proof_input(self):
        from workflow_delivery_workspace import workspace_policy_shape
        policy = self.tree.json(WORKSPACE)
        workspace_policy_shape(policy)
        self.assertIn(WORKSPACE, self.contract["input_roots"])
        self.assertEqual(24, len(policy["local_files"]))
        owners = [row["path"] for row in policy["local_files"]
                  if row["category"] == "owner_local"]
        self.assertEqual(["scripts/run_gui_doa2526.sh"], owners)

    def test_operational_progress_does_not_rewrite_proof_authority(self):
        path = "specs/WORKFLOW_DELIVERY_EXECUTION_PLAN.md"
        tree = RepositoryView({path: self.tree.read(path) + b"\nOperational progress.\n"})
        self.assertEqual(authority_manifest(self.tree, self.contract, ready=False),
                         authority_manifest(tree, self.contract, ready=False))

    def test_future_result_records_do_not_supply_their_own_authority(self):
        tree = RepositoryView({self.contract["proof_path"]: b"fixture-proof-placeholder",
                               self.contract["review_path"]: b"fixture-review-placeholder"})
        self.assertEqual(authority_manifest(self.tree, self.contract, ready=False),
                         authority_manifest(tree, self.contract, ready=False))

    def test_adding_own_review_to_authority_is_refused(self):
        contract = deepcopy(self.contract)
        contract["review_path"] = SPEC
        with self.assertRaises(DeliveryInputError) as raised:
            authority_manifest(self.tree, contract, ready=False)
        self.assertEqual("WDQ-AUTHORITY", raised.exception.code)

    def test_input_inventory_is_explicit_and_foundations_do_not_fake_cad_proof(self):
        self.assertTrue(all(p.endswith((".py", ".sh", ".yml", ".json", ".rs", "/pre-commit"))
                            for p in self.contract["input_roots"]))
        self.assertNotIn("scripts", self.contract["input_roots"])
        self.assertNotIn("crates", self.contract["input_roots"])
        for key in ("units_precision", "numeric_entry", "selection_identity",
                    "grid_snap", "library_connectivity"):
            self.assertEqual("not_applicable", self.contract["foundation_answers"][key]["disposition"])
        for key in ("undo_cancel", "persistence_recovery", "settings_scope"):
            self.assertEqual("required", self.contract["foundation_answers"][key]["disposition"])

    def case_rows(self):
        source = self.tree.read(SPEC).decode()
        return [tuple(part.strip() for part in line.strip("|").split("|"))
                for line in source.splitlines() if line.startswith("| INFRA-S")]

    def test_hook_read_dependencies_are_in_the_input_inventory(self):
        exemptions = self.tree.json("specs/rustfmt_exemption_manifest.json")["exemptions"]
        required = set(exemptions) | {"scripts/check_file_lane_ownership.py",
            "scripts/check_rustfmt.py", "specs/rustfmt_exemption_manifest.json"}
        self.assertTrue(required <= set(self.contract["input_roots"]))
        self.assertTrue(all((ROOT / path).is_file() for path in required))

    def test_implemented_workflow_python_and_local_imports_are_explicit_inputs(self):
        import ast
        required = {p.relative_to(ROOT).as_posix() for pattern in
                    ("workflow_delivery_*.py", "test_workflow_delivery_*.py", "test_workflow_rollout_*.py")
                    for p in (ROOT / "scripts").glob(pattern)}
        queue = list(required | {p for p in self.contract["input_roots"] if p.endswith(".py")})
        visited = set()
        while queue:
            path = queue.pop()
            if path in visited:
                continue
            visited.add(path)
            for node in ast.walk(ast.parse((ROOT / path).read_bytes(), filename=path)):
                modules = ([a.name for a in node.names] if isinstance(node, ast.Import) else
                           [node.module] if isinstance(node, ast.ImportFrom) and node.module else [])
                for module in modules:
                    local = "scripts/" + module.split(".")[0] + ".py"
                    if (ROOT / local).is_file():
                        required.add(local)
                        queue.append(local)
        self.assertEqual([], sorted(required - set(self.contract["input_roots"])),
                         "Reconcile new inputs explicitly; this test must not update authority")

    def test_prepared_hook_and_selected_environment_bytes_are_inputs(self):
        import hashlib
        from workflow_delivery_bootstrap import HOOK_SOURCE
        selection_path = "specs/workflow_delivery/rollout.environments.json"
        required = {HOOK_SOURCE, selection_path}
        selection = self.tree.json(selection_path)
        for row in selection["environments"]:
            blob = row["environment"]
            required.add(blob["path"])
            self.assertEqual(blob["sha256"], hashlib.sha256(self.tree.read(blob["path"])).hexdigest())
        self.assertEqual([], sorted(required - set(self.contract["input_roots"])))

    def test_case_inventory_has_unique_ids_and_known_scenarios_and_surfaces(self):
        rows = self.case_rows()
        self.assertTrue(rows)
        self.assertTrue(all(len(row) == 4 for row in rows))
        self.assertEqual(len(rows), len({row[0] for row in rows}))
        scenarios = {s["id"] for s in self.contract["scenarios"]}
        self.assertEqual(scenarios, {row[0].rsplit("-", 1)[0] for row in rows})
        for identity, condition, outcome, surfaces in rows:
            self.assertRegex(identity, r"^INFRA-S0[1-6]-[0-9]{2}$")
            self.assertTrue(condition and outcome)
            names = surfaces.split()
            self.assertTrue(names and set(names) <= {"C", "R", "S", "H"})
            self.assertEqual(len(names), len(set(names)))
        declared = re.findall(r"^\| (INFRA-S\S+) \|", self.tree.read(SPEC).decode(), re.M)
        self.assertEqual(declared, [row[0] for row in rows])

    def test_case_inventory_distinguishes_history_from_worktree_snapshots(self):
        rows = {row[0]: row for row in self.case_rows()}
        for identity in ("INFRA-S05-03", "INFRA-S05-04", "INFRA-S05-07", "INFRA-S05-08"):
            self.assertEqual("R", rows[identity][3])
        for identity in ("INFRA-S05-01", "INFRA-S05-02", "INFRA-S05-05"):
            self.assertEqual({"C", "S", "H"}, set(rows[identity][3].split()))

    def test_every_matrix_surface_is_declared_by_its_scenario_consumers(self):
        surfaces = {"C": "staged_cli", "R": "candidate_cli",
                    "S": "project_status", "H": "owner_hook"}
        consumers = {c["id"]: set(c["entry_surfaces"])
                     for c in self.contract["consumers"]}
        scenarios = {s["id"]: set().union(*(consumers[c] for c in s["consumer_ids"]))
                     for s in self.contract["scenarios"]}
        for identity, _, _, required in self.case_rows():
            with self.subTest(case=identity):
                declared = scenarios[identity.rsplit("-", 1)[0]]
                self.assertTrue({surfaces[s] for s in required.split()} <= declared)


if __name__ == "__main__":
    unittest.main()
