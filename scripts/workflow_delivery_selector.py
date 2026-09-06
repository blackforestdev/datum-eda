"""Read-only PM025 adapter; delivery artifacts never choose or reorder work."""

from pathlib import Path
import subprocess

from workflow_delivery_checkpoints import validate_delivery
from workflow_delivery_io import DeliveryInputError
from workflow_delivery_shapes import require
from workflow_delivery_tree import Tree
from workflow_delivery_trust import POLICY_PATH, Trust, policy_shape


def _external(root, name):
    result = subprocess.run(["git", "config", "--local", "--get", "datum." + name],
                            cwd=root, capture_output=True, text=True, check=False)
    return result.stdout.strip() if result.returncode == 0 else None


def selector_failures(root, manifest):
    authority = _external(root, "workflowDeliveryAuthorityRef")
    base = _external(root, "workflowDeliveryBaseRef")
    has_policy = (Path(root) / POLICY_PATH).is_file()
    items = [i for i in manifest.get("frontier", []) if type(i) is dict]
    declared = [i for i in items if "delivery" in i.get("completion", {})]
    if not has_policy and not declared and not authority and not base:
        return []
    try:
        require(manifest.get("schema_version") == 6, "schema_version",
                "delivery requires Frontier schema 6", "WDQ-TRANSITION")
        tree = Tree(root)
        enrolled = {}
        if has_policy:
            policy = policy_shape(tree.json(POLICY_PATH))
            enrolled = {r["frontier_key"]: r for r in policy["enrolled"]}
        require(set(enrolled) <= {i["key"] for i in declared}, POLICY_PATH,
                "enrolled delivery declaration removed", "WDQ-POLICY")
        env_path = _external(root, "workflowDeliveryEnvironmentPath")
        trust = Trust(tree, authority, base) if authority or base else None
        if trust:
            require(set(trust.enrolled) <= {i["key"] for i in declared}, POLICY_PATH,
                    "trusted enrollment cannot disappear from candidate", "WDQ-POLICY")
            enrolled = trust.enrolled
        environment = tree.json(env_path) if env_path else None
        for item in declared:
            validate_delivery(tree, item, phase=None if item["key"] in enrolled else "structure",
                              trust=trust if item["key"] in enrolled else None,
                              environment=environment)
        return []
    except DeliveryInputError as error:
        return [str(error)]
    except (KeyError, TypeError, ValueError, OSError) as error:
        return [f"WDQ-CONTRACT: delivery declaration: {error}"]
