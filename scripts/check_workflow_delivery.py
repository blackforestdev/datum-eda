#!/usr/bin/env python3
"""Read-only PM041 delivery diagnostics; bootstrap defaults to report-only."""

import argparse
import json
from pathlib import Path
import sys

from workflow_delivery_authority import authority_sha256
from workflow_delivery_checkpoints import validate_delivery
from workflow_delivery_contract import load_contract, validate_handler_files
from workflow_delivery_frontier import validate_frontier
from workflow_delivery_io import DeliveryInputError
from workflow_delivery_shapes import require
from workflow_delivery_tree import Tree
from workflow_delivery_trust import FRONTIER_PATH, POLICY_PATH, Trust, policy_shape


def run(args):
    tree = Tree(args.root, staged=args.staged, revision=args.candidate_ref)
    trust = None
    if args.enforce:
        trust = Trust(tree, args.authority_ref, args.base_ref)
    require(not args.enforce or args.contract is None, "contract",
            "enforcement uses trusted enrollment, not an ad-hoc contract", "WDQ-TRUST")
    environment = tree.json(args.environment_path) if args.environment_path else None
    if args.contract:
        contract = load_contract(tree, args.contract)
        validate_handler_files(tree, contract)
        authority_sha256(tree, contract, ready=True)
        return ["contract structure and readiness references (report only)"]
    manifest = validate_frontier(tree) if args.enforce else tree.json(FRONTIER_PATH)
    if trust:
        enrolled = trust.enrolled
    elif POLICY_PATH in tree.entries:
        enrolled = {r["frontier_key"]: r for r in policy_shape(tree.json(POLICY_PATH))["enrolled"]}
    else:
        return ["no enrolled delivery obligations; no readiness or acceptance asserted"]
    items = {i["key"]: i for i in manifest["frontier"]}
    checked = []
    for key in enrolled:
        require(key in items, key, "enrolled Frontier item absent", "WDQ-POLICY")
        require("delivery" in items[key].get("completion", {}), key,
                "enrolled completion.delivery missing", "WDQ-CONTRACT")
        phase = validate_delivery(tree, items[key], trust=trust, environment=environment)
        checked.append(f"{key}: {phase}")
    return checked


def main(argv=None):
    parser = argparse.ArgumentParser(description=__doc__)
    mode = parser.add_mutually_exclusive_group()
    mode.add_argument("--report-only", action="store_true")
    mode.add_argument("--enforce", action="store_true")
    view = parser.add_mutually_exclusive_group()
    view.add_argument("--staged", action="store_true")
    view.add_argument("--candidate-ref")
    parser.add_argument("--root", type=Path, default=Path(__file__).resolve().parents[1])
    parser.add_argument("--authority-ref")
    parser.add_argument("--base-ref")
    parser.add_argument("--environment-path", help="explicit requested environment; exact match only")
    parser.add_argument("--contract", help="report-only readiness inspection before enrollment")
    args = parser.parse_args(argv)
    report = {"mode": "enforce" if args.enforce else "report-only", "findings": [],
              "checks": [], "readiness_asserted": False, "acceptance_asserted": False}
    status = 0
    try:
        report["checks"] = run(args)
    except DeliveryInputError as error:
        lane = {"WDQ-TRUST": "project owner / controlled runner", "WDQ-POLICY": "project owner",
                "WDQ-RECEIPT": "project owner", "WDQ-REVIEW": "independent reviewer",
                "WDQ-CONSUMER": "production consumer owner", "WDQ-INDEX": "committing lane",
                "WDQ-RESULT": "proof producer", "WDQ-AUTHORITY": "owning evidence route"}
        report["findings"].append({"code": error.code, "path": error.path,
                                   "key": getattr(error, "key", None),
                                   "step": getattr(error, "step", None),
                                   "scenario": getattr(error, "scenario", None),
                                   "detail": error.detail,
                                   "owning_lane": lane.get(error.code, "delivery contract/evidence owner")})
        status = 2 if error.code == "WDQ-TRUST" else 1
    except (KeyError, TypeError, ValueError, OSError, UnicodeError) as error:
        report["findings"].append({"code": "WDQ-CONTRACT", "path": "candidate",
                                   "detail": str(error), "owning_lane": "delivery contract/evidence owner"})
        status = 1
    print(json.dumps(report, sort_keys=True, ensure_ascii=False))
    # Report-only findings never acquire blocking/acceptance semantics. Invocation
    # errors remain errors so a caller cannot mistake broken trust for a verdict.
    return status if args.enforce or status == 2 else 0


if __name__ == "__main__":
    sys.exit(main())
