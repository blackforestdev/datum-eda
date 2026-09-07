#!/usr/bin/env python3
"""Prepare an isolated G06 proposal; never promote trust or update live main.

The retained refs/datum/workflow-delivery-candidates/<oid> is an object-retention
reference ONLY. It is never a default authority input or a feature branch.
"""

import argparse
import json
from pathlib import Path
import shutil
import subprocess

from workflow_delivery_authority import authority_sha256, resolve_ref
from workflow_delivery_contract import load_contract
from workflow_delivery_evidence_shapes import review_sha256, review_shape
from workflow_delivery_frontier import validate_frontier
from workflow_delivery_proof import packet_sha256, validate_proof
from workflow_delivery_prepare_handoff import promotion_markdown
from workflow_delivery_prepare_review import check_review
from workflow_delivery_review import receipt_section
from workflow_delivery_tree import Tree


KEY = "WORKFLOW-DELIVERY-GATE-PILOT"
ISSUE = "dat-workflow-gate-pilot-b3s"
HANDOFF = "docs/reviews/workflow-delivery-pilot/handoff.md"
POLICY = "specs/workflow_delivery_policy.json"
ENVIRONMENT = "docs/reviews/workflow-delivery-pilot/proof-artifacts/environment.json"
ACTIVATED = "<!-- EVIDENCE:WORKFLOW-DELIVERY-GATE-PILOT:WDQ-G06-ACTIVATED -->"
OWNED = [POLICY, HANDOFF, "specs/active_frontier.json", "specs/PROGRESS.md",
         "specs/spec_governance_manifest.json", ".beads/issues.jsonl"]


def run(root, *args):
    return subprocess.run(args, cwd=root, check=True, capture_output=True, text=True).stdout.strip()


def save(root, path, value):
    (root / path).write_text(json.dumps(value, indent=2, ensure_ascii=False) + "\n", encoding="utf-8")


def live_state(root):
    """Compare semantic index/config state without printing private configuration."""
    return tuple(run(root, "git", *args) for args in (
        ("rev-parse", "HEAD"), ("status", "--porcelain"),
        ("ls-files", "--stage", "-z"), ("config", "--local", "--null", "--list")))


def require_unchanged(root, initial):
    if live_state(root) != initial:
        raise ValueError("live HEAD/index/worktree/config changed during preparation; do not promote")


def replace_item(raw, item):
    """Keep every other item's original bytes, including its formatting."""
    position = raw.index("[", raw.index('"frontier"')) + 1
    decoder = json.JSONDecoder()
    while True:
        while raw[position].isspace() or raw[position] == ",":
            position += 1
        if raw[position] == "]":
            raise ValueError("pilot item missing")
        current, end = decoder.raw_decode(raw, position)
        if current["key"] == item["key"]:
            formatted = json.dumps(item, indent=2, ensure_ascii=False).replace("\n", "\n    ")
            return raw[:position] + formatted + raw[end:]
        position = end


def completion_proposal(manifest, base):
    """Transform only the enrolled item in an isolated proposal, not live state."""
    item = next(i for i in manifest["frontier"] if i["key"] == KEY)
    steps = {s["id"]: s for s in item["completion"]["steps"]}
    preparing = (steps["WDQ-G06P"]["status"] == "in_progress"
                 and item["authorization"] == "execution" and bool(item.get("claim")))
    prepared = (steps["WDQ-G06P"]["status"] == "complete"
                and item["authorization"] == "owner_decision" and not item.get("claim"))
    if not (preparing or prepared) or steps["WDQ-G06"]["status"] != "pending":
        raise ValueError("requires authorized preparation or its released owner handoff; G06 must be pending")
    if preparing:
        steps["WDQ-G06P"]["status"] = "complete"
        steps["WDQ-G06P"].setdefault("completion_evidence", []).append(
            {"kind": "document", "path": HANDOFF, "marker": ACTIVATED})
    item.update(state="landed", authorization="none", landing_commit=base)
    item.pop("claim", None)
    item["completion"]["canonical_next_step_id"] = None
    item["completion"]["delivery"] = {"contract_path": "specs/workflow_delivery/pilot.contract.json",
        "checkpoints": {"ready": "WDQ-G01", "activate": "WDQ-G04", "verify": "WDQ-G05",
                        "accept": "WDQ-G06"}}
    steps["WDQ-G06"]["status"] = "complete"
    steps["WDQ-G06"]["completion_evidence"].append(
        {"kind": "document", "path": HANDOFF, "marker": ACTIVATED})
    # PM025 derives post-completion unblocks from still-open blocked issues.
    # Closing this issue removes just its predecessor's now-satisfied entry.
    for predecessor in manifest["frontier"]:
        if predecessor["key"] == "WORKFLOW-DELIVERY-QUALITY":
            unblocks = predecessor["completion"]["post_completion"]["unblocks_issue_ids"]
            unblocks.remove(ISSUE)
            predecessor["unblocks"].remove(ISSUE)
    return manifest


def prepare(root, output):
    root, output = root.resolve(), output.resolve()
    if output.exists() or output.is_relative_to(root):
        raise ValueError("output must be a new directory outside the live repository")
    base = run(root, "git", "rev-parse", "HEAD")
    if run(root, "git", "status", "--porcelain"):
        raise ValueError("commit coordinated work first; preparation requires a clean worktree")
    initial = live_state(root)
    tree = Tree(root, revision=base)
    # Validate the actual source lease and authorization before any output writes.
    validate_frontier(tree)
    check_review(tree, "specs/workflow_delivery/pilot.contract.json", ENVIRONMENT)
    contract = load_contract(tree, "specs/workflow_delivery/pilot.contract.json")
    proof = validate_proof(tree, contract)
    review = review_shape(tree.json(contract["review_path"]), contract["review_path"])
    if packet_sha256(contract, proof, authority_sha256(tree, contract)) != review["packet_sha256"]:
        raise ValueError("accepted packet is stale")
    if not review["owner_receipt"]:
        raise ValueError("record the exact owner acceptance before preparation")
    ref = review["owner_receipt"]
    receipt = receipt_section(resolve_ref(tree, ref), ref["marker"], ref["path"])
    exact = f"ACCEPT {KEY}/WDQ-G06 {review['packet_sha256']} {review_sha256(review)}"
    if review["disposition"] != "approve" or receipt.splitlines().count(exact) != 1:
        raise ValueError("the exact current review must be approved by the recorded owner receipt")
    if Tree(root).manifest(contract["input_roots"]) != tree.json(proof["input_manifest"]["path"]):
        raise ValueError("live input closure is dirty, including ignored generated files")
    proposed = completion_proposal(tree.json("specs/active_frontier.json"), base)
    if not shutil.which("br"):
        raise ValueError("br is required; never substitute direct JSONL edits")
    output.mkdir(mode=0o700)
    candidate = output / "candidate"
    run(root, "git", "clone", "--shared", "--no-hardlinks", "--no-checkout", str(root), str(candidate))
    run(candidate, "git", "checkout", "--detach", base)
    for key in ("user.name", "user.email"):
        run(candidate, "git", "config", "--local", key, run(root, "git", "config", "--get", key))
    frontier_path = candidate / "specs/active_frontier.json"
    item = next(i for i in proposed["frontier"] if i["key"] == KEY)
    frontier_path.write_text(replace_item(frontier_path.read_text(encoding="utf-8"), item), encoding="utf-8")
    predecessor = next(i for i in proposed["frontier"] if i["key"] == "WORKFLOW-DELIVERY-QUALITY")
    frontier_path.write_text(replace_item(frontier_path.read_text(encoding="utf-8"), predecessor), encoding="utf-8")
    save(candidate, POLICY, tree.json("docs/reviews/workflow-delivery-pilot/activation-policy.candidate.json"))
    governance_path = candidate / "specs/spec_governance_manifest.json"
    governance = governance_path.read_text(encoding="utf-8")
    if POLICY in tree.json("specs/spec_governance_manifest.json")["entries"]:
        raise ValueError("policy already classified; do not overwrite an activation")
    governance = governance.replace('"tracked_docs": [\n', '"tracked_docs": [\n    "' + POLICY + '",\n', 1)
    entry = ('    "' + POLICY + '": {"class": "governed", '
             '"progress_anchor": "WORKFLOW-DELIVERY-GATE-PILOT:TRACKING", '
             '"notes": "Exact owner-local activation proposal; active only after external promotion."},\n')
    governance = governance.replace('"entries": {\n', '"entries": {\n' + entry, 1)
    governance_path.write_text(governance, encoding="utf-8")
    with (candidate / HANDOFF).open("a") as stream:
        stream.write("\n" + ACTIVATED + "\n## Conditional activation proposal\n\n"
            "This detached candidate proposes final enrollment and completion. It is NOT live\n"
            "Preparation completion here is conditional on successful artifact generation;\n"
            "live G06P remains unchanged until that success is separately recorded on main.\n"
            "while retained outside main. The owner must select this exact commit externally,\n"
            "run its trusted validator successfully, and only then publish it to main and\n"
            "install the owner-local blocking hook. The already recorded ACCEPT and both\n"
            "DEFER responses remain unchanged. Preferences stays at GP-CM04, unclaimed;\n"
            "no GP-CM05 approval, successor selection or additional enrollment is granted.\n"
            f"Preparation comparison base: {base}. No agent-selected promotion occurred.\n")
    run(candidate, "br", "close", ISSUE, "--actor", "codex", "--reason",
        f"Proposed activation of accepted proof on {base}; not live until owner promotion and verification.")
    run(candidate, "br", "sync", "--flush-only", "--actor", "codex")
    # Generate the projection only. The public selector deliberately refuses
    # unpromoted acceptance; structural validation below is not enforcement.
    run(candidate, "python3", "-c", "import sys; from pathlib import Path; "
        "sys.path.insert(0, 'scripts'); from project_status import render_status; "
        "render_status(Path.cwd(), True)")
    run(candidate, "python3", "scripts/check_spec_governance.py")
    run(candidate, "python3", "scripts/check_evidence_traceability.py")
    run(candidate, "git", "add", "--", *OWNED)
    run(candidate, "git", "diff", "--cached", "--check")
    changed = set(run(candidate, "git", "diff", "--cached", "--name-only").splitlines())
    if changed != set(OWNED):
        raise ValueError("candidate must change exactly the six approved proposal paths")
    run(candidate, "git", "commit", "-m", "chore(workflow): propose exact owner-local pilot activation",
        "-m", "Problem:\nAccepted pilot requires a separately owner-promoted activation transaction.\n\n"
        "Change:\nPropose only the approved policy, checkpoint mapping and final completion. This detached "
        "commit is not live authority or activation until the owner selects, verifies and publishes it.\n\n"
        "Proof:\nProducer and independent native evidence are retained with unchanged accepted hashes. "
        "Preparation verifies source freshness, governing evidence and candidate structure; no trusted "
        "enforcement run or external promotion is claimed by the preparing agent.\n\n"
        "Roadmap:\nWDQ-G06 proposal for dat-workflow-gate-pilot-b3s under PM041/PM025. Proposed closure "
        "becomes live only after owner-controlled successful verification/promotion. Both explicitly deferred "
        "Console issues stay open; Preferences GP-CM04 and canonical selection remain unchanged. "
        "No production, dependency, licensing or prototype change.")
    revision = run(candidate, "git", "rev-parse", "HEAD")
    # This validates PM025 structure without pretending the owner selected trust.
    validate_frontier(Tree(candidate, revision=revision))
    hooks = output / "owner-hooks"
    hooks.mkdir(mode=0o700)
    shutil.copyfile(candidate / "docs/reviews/workflow-delivery-pilot/owner-pre-commit.sh", hooks / "pre-commit")
    (hooks / "pre-commit").chmod(0o700)
    require_unchanged(root, initial)
    if Tree(root).manifest(contract["input_roots"]) != tree.json(proof["input_manifest"]["path"]):
        raise ValueError("live input closure changed during preparation; do not promote")
    run(root, "git", "fetch", "--no-tags", "--no-write-fetch-head", str(candidate), revision)
    run(root, "git", "update-ref", "refs/datum/workflow-delivery-candidates/" + revision, revision)
    require_unchanged(root, initial)
    result = {"candidate": revision, "base": base, "runner": str(candidate / "scripts/check_workflow_delivery.py"),
              "hooks": str(hooks), "environment": ENVIRONMENT, "packet_sha256": review["packet_sha256"],
              "review_sha256": review_sha256(review), "promotion_performed": False}
    save(output, "preparation.json", result)
    (output / "promotion.md").write_text(promotion_markdown(root, result), encoding="utf-8")
    return result


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--root", type=Path, default=Path(__file__).resolve().parents[1])
    mode = parser.add_mutually_exclusive_group(required=True)
    mode.add_argument("--output", type=Path)
    mode.add_argument("--check-review-only", action="store_true",
                      help="read committed review accounting without requiring or promoting owner trust")
    args = parser.parse_args()
    try:
        if args.check_review_only:
            tree = Tree(args.root, revision=run(args.root, "git", "rev-parse", "HEAD"))
            result = check_review(tree, "specs/workflow_delivery/pilot.contract.json", ENVIRONMENT)
        else:
            result = prepare(args.root, args.output)
        print(json.dumps(result, indent=2))
    except (ValueError, subprocess.CalledProcessError) as error:
        detail = (error.stderr or error.stdout or "") if isinstance(error, subprocess.CalledProcessError) else ""
        parser.exit(1, str(error) + "\n" + detail +
                    "\nNo owner promotion performed; preserve the diagnostic directory.\n")


if __name__ == "__main__":
    main()
