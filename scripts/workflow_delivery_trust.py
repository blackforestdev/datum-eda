"""Owner-selected authority/base checks; never promote candidate authority."""

from pathlib import Path

from workflow_delivery_authority import resolve_ref
from workflow_delivery_bootstrap import HOOK_SOURCE, runtime_paths
from workflow_delivery_contract import canonical_contract, validate_contract
from workflow_delivery_coverage_shapes import coverage_shape
from workflow_delivery_io import DeliveryInputError, canonical_json
from workflow_delivery_shapes import (
    array, closed, commit_id, ids, identifier, ref, require, unique,
)
from workflow_delivery_tree import Tree
from workflow_delivery_workspace_authority import load_workspace


POLICY_PATH = "specs/workflow_delivery_policy.json"
FRONTIER_PATH = "specs/active_frontier.json"


def policy_shape(value):
    try:
        return _policy_shape(value)
    except DeliveryInputError as error:
        raise DeliveryInputError("WDQ-POLICY", error.path, error.detail) from error


def _policy_shape(value):
    require(type(value) is dict, POLICY_PATH, "policy object required", "WDQ-POLICY")
    schema = value.get("schema_version")
    require(type(schema) is int and schema in (1, 2), POLICY_PATH,
            "integer policy schema 1 or 2 required", "WDQ-POLICY")
    fields = "schema_version decision_ref enrolled legacy_baseline"
    closed(value, fields + (" coverage" if schema == 2 else ""), POLICY_PATH, "WDQ-POLICY")
    if schema == 2:
        coverage_shape(value["coverage"])
    ref(value["decision_ref"], POLICY_PATH)
    commit_id(value["legacy_baseline"], POLICY_PATH)
    array(value["enrolled"], POLICY_PATH)
    for row in value["enrolled"]:
        closed(row, "frontier_key implementation_sessions activation_ref", POLICY_PATH, "WDQ-POLICY")
        identifier(row["frontier_key"], POLICY_PATH)
        ids(row["implementation_sessions"], POLICY_PATH)
        ref(row["activation_ref"], POLICY_PATH)
    unique([r["frontier_key"] for r in value["enrolled"]], POLICY_PATH, "WDQ-POLICY")
    return value


def _gate_paths(tree):
    fixed = {HOOK_SOURCE, "scripts/git-hooks/pre-commit", "scripts/run_drift_gates.sh",
             ".github/workflows/alignment.yml"}
    return fixed | runtime_paths(tree.entries)


class Trust:
    def __init__(self, candidate, authority_ref, base_ref):
        require(bool(authority_ref) and bool(base_ref), "trust",
                "owner-selected --authority-ref and trusted --base-ref required; no HEAD fallback",
                "WDQ-TRUST")
        try:
            commit_id(authority_ref, "authority_ref")
            commit_id(base_ref, "base_ref")
        except DeliveryInputError as error:
            raise DeliveryInputError("WDQ-TRUST", error.path,
                                     "trusted refs must pin full commit IDs, not moving candidate names") from error
        self.authority = Tree(candidate.root, revision=authority_ref)
        self.base = Tree(candidate.root, revision=base_ref)
        try:
            self.policy = policy_shape(self.authority.json(POLICY_PATH))
        except DeliveryInputError as error:
            raise DeliveryInputError("WDQ-TRUST", POLICY_PATH,
                                     "selected authority has no valid promoted policy") from error
        self.authority.has_commit(self.policy["legacy_baseline"])
        resolve_ref(self.authority, self.policy["decision_ref"])
        try:
            proposed = policy_shape(candidate.json(POLICY_PATH))
        except DeliveryInputError as error:
            raise DeliveryInputError("WDQ-POLICY", POLICY_PATH,
                                     "candidate removed or malformed enrolled policy") from error
        require(canonical_json(proposed) == canonical_json(self.policy), POLICY_PATH,
                "policy change requires owner-controlled promotion", "WDQ-POLICY")
        for row in self.policy["enrolled"]:
            resolve_ref(self.authority, row["activation_ref"])
        trusted_paths = _gate_paths(self.authority)
        require(_gate_paths(candidate) == trusted_paths, "gate",
                "gate implementation set changed without promotion", "WDQ-POLICY")
        for path in sorted(trusted_paths):
            approved = self.authority.read(path)
            require(candidate.read(path) == approved, path,
                    "gate/wiring change requires owner-controlled promotion", "WDQ-POLICY")
            if path.endswith(".py") and path.startswith("scripts/"):
                running = Path(__file__).resolve().parent / Path(path).name
                require(running.is_file() and running.read_bytes() == approved, path,
                        "execute the trusted revision's gate, not candidate code", "WDQ-TRUST")
        self.enrolled = {r["frontier_key"]: r for r in self.policy["enrolled"]}
        self.workspace = load_workspace(candidate, self.authority,
                                        policy_schema=self.policy["schema_version"])
        candidate.workspace = self.workspace

    def contract(self, candidate, item, contract):
        key = item["key"]
        require(key in self.enrolled, key, "key not enrolled by owner", "WDQ-POLICY")
        trusted_items = self.authority.json(FRONTIER_PATH)["frontier"]
        matches = [i for i in trusted_items if i["key"] == key]
        require(len(matches) == 1, key, "trusted Frontier identity absent/duplicate", "WDQ-POLICY")
        trusted = matches[0]
        require(item["issue_id"] == trusted["issue_id"] and
                item["completion"]["delivery"] == trusted["completion"]["delivery"], key,
                "delivery mapping change requires owner promotion", "WDQ-POLICY")
        path = trusted["completion"]["delivery"]["contract_path"]
        approved = validate_contract(self.authority.json(path), path,
                                     frontier_key=key, issue_id=item["issue_id"])
        require(canonical_contract(approved) == canonical_contract(contract), path,
                "contract/root/scenario change requires owner promotion", "WDQ-POLICY")
        return self.enrolled[key]

    def acceptance_present(self, item):
        key = item["key"]
        step_id = item["completion"]["delivery"]["checkpoints"]["accept"]
        trusted = next(i for i in self.authority.json(FRONTIER_PATH)["frontier"] if i["key"] == key)
        # The owner promotes the prepared acceptance transaction; a candidate-only
        # completed step is not sufficient for the ordinary selector either.
        require(any(s["id"] == step_id and s["kind"] == "owner_decision" and s["status"] == "complete"
                    for s in trusted["completion"]["steps"]), key,
                "completed acceptance absent from promoted authority", "WDQ-RECEIPT")
