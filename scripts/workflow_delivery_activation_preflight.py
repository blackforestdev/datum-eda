"""Read-only publication preconditions; never activate or claim owner approval.

An eventual activation transaction must revalidate these observations under its
own coordination boundary. Successful preparation is not a lock or permission.
"""

import os
from pathlib import Path

from workflow_delivery_bootstrap import git
from workflow_delivery_capture_state import TRUST_KEYS, protected_state
from workflow_delivery_io import canonical_json, sha256
from workflow_delivery_publication_delta import publication_delta, tree_entries, verify_publication_review

REDIRECTS = ("GIT_DIR", "GIT_WORK_TREE", "GIT_COMMON_DIR", "GIT_INDEX_FILE", "GIT_NAMESPACE",
             "GIT_OBJECT_DIRECTORY", "GIT_ALTERNATE_OBJECT_DIRECTORIES", "GIT_CONFIG",
             "GIT_CONFIG_COUNT", "GIT_CONFIG_PARAMETERS")

ROLLOUT = "WORKFLOW-DELIVERY-IMPLEMENTATION"


def initial_mapping_migration(manifest, baseline, policy, prior_policy):
    """Recognize only the initial legacy-to-rollout mapping transition."""
    from workflow_delivery_coverage import index_items
    from workflow_delivery_trust import policy_shape

    policy_shape(prior_policy)
    policy_shape(policy)
    if prior_policy["schema_version"] != 1:
        return False
    current, old = index_items(manifest), index_items(baseline)
    require(ROLLOUT in current and ROLLOUT in old, "migration requires existing rollout identity")
    require("delivery" not in old[ROLLOUT].get("completion", {}),
            "legacy publication baseline must have no rollout mapping")
    prior = {row["frontier_key"]: row for row in prior_policy["enrolled"]}
    proposed = {row["frontier_key"]: row for row in policy["enrolled"]}
    require(ROLLOUT not in prior and set(proposed) == set(prior) | {ROLLOUT}
            and all(proposed[key] == row for key, row in prior.items()),
            "initial migration must preserve prior enrollments and add only rollout")
    require(policy["schema_version"] == 2
            and policy["legacy_baseline"] == prior_policy["legacy_baseline"],
            "initial migration requires schema-2 policy with unchanged legacy baseline")
    require(current[ROLLOUT].get("completion", {}).get("delivery") == {
        "schema_version": 2, "contract_path": "specs/workflow_delivery/rollout.contract.json",
        "checkpoints": {"ready": "WDQ-READY", "activate": None, "verify": "WDQ-COMPAT",
                        "review": "WDQ-RECHECK", "accept": None}},
        "initial migration requires the exact renewed rollout mapping")
    return True


def promotion_boundary(manifest, baseline, coverage, changed_paths, *, legacy_migration=False):
    """Check bounded rollout publication, not an ordinary execution permission."""
    from workflow_delivery_coverage import index_items
    from workflow_delivery_coverage_shapes import coverage_shape
    coverage_shape(coverage)
    current, old = index_items(manifest), index_items(baseline)
    require(set(current) == set(old), "promotion cannot add or remove Frontier identities")
    require(ROLLOUT in current, "promotion requires the actual rollout owner boundary")
    for key in current:
        if key != ROLLOUT:
            require(current[key] == old[key], "promotion changes another Frontier lane: " + key)
    for is_baseline, item in ((True, old[ROLLOUT]), (False, current[ROLLOUT])):
        completion = item.get("completion", {})
        steps = {s["id"]: s for s in completion.get("steps", [])}
        require(item.get("issue_id") == "dat-wdq-rollout-implementation-ffy"
                and item.get("authorization") == "owner_decision" and not item.get("claim")
                and completion.get("canonical_next_step_id") == "WDQ-I04",
                "publication base and candidate must select claim-free WDQ-I04")
        owner = steps.get("WDQ-I04", {})
        require(owner.get("kind") == "owner_decision" and owner.get("status") == "pending",
                "WDQ-I04 must await exact owner activation, not claim prior completion")
        for sid in ("WDQ-TOOLS", "WDQ-READY", "WDQ-I03", "WDQ-REVIEW",
                    "WDQ-COMPAT", "WDQ-RECHECK"):
            require(steps.get(sid, {}).get("status") == "complete",
                    "publication requires completed " + sid)
        require(owner.get("depends_on") == ["WDQ-RECHECK"]
                and steps["WDQ-RECHECK"].get("depends_on") == ["WDQ-COMPAT"]
                and steps["WDQ-COMPAT"].get("depends_on") == ["WDQ-REVIEW"],
                "publication requires the exact renewal dependency chain")
        delivery = completion.get("delivery", {})
        if is_baseline and legacy_migration and "delivery" not in completion:
            continue  # Only the separately validated initial migration permits absence.
        require(delivery.get("schema_version") == 2 and delivery.get("checkpoints") == {
            "ready": "WDQ-READY", "activate": None, "verify": "WDQ-COMPAT",
            "review": "WDQ-RECHECK", "accept": None},
            "publication requires the renewed verification and independent-review mapping")
    return publication_scope(coverage, changed_paths)


def publication_scope(coverage, changed_paths):
    """Exact bounded path inventory shared by review and final inspection."""
    from workflow_delivery_coverage_shapes import coverage_shape
    from workflow_delivery_source_scopes import contains

    coverage_shape(coverage)
    scopes = coverage["source_scopes"]
    require(len(scopes) == 1 and scopes[0]["frontier_key"] == ROLLOUT
            and scopes[0]["step_ids"] == ["WDQ-I03", "WDQ-REVIEW"],
            "initial rollout publication requires its sole bounded infrastructure scope")
    checked = []
    for path in changed_paths:
        require(not path.startswith("docs/gui/prototypes/"), "publication touches the protected prototype lane")
        if contains(coverage["production_roots"], path):
            require(path in scopes[0]["paths"], "publication production path lacks exact reviewed scope: " + path)
            checked.append(path)
    return sorted(set(checked))


def inspect_promotion_candidate(root, *, base, candidate, authority, environment_path, publication_review):
    """Inspect full delivery evidence without installing trust or granting approval.

    The caller separately checks its exact owner response and clean live state.
    This path never supplies a synthetic execution claim or an empty delta to
    the ordinary source gate. Only the bounded rollout publication is supported.
    """
    from workflow_delivery_checkpoints import validate_delivery
    from workflow_delivery_coverage_runtime import validate_coverage_state
    from workflow_delivery_environments import load_environments
    from workflow_delivery_frontier import validate_frontier
    from workflow_delivery_trust import Trust, FRONTIER_PATH, POLICY_PATH
    from workflow_delivery_tree import Tree

    require(authority == candidate, "initial promotion authority must be the exact complete candidate")
    delta = publication_delta(root, base=base, candidate=candidate)
    verify_publication_review(delta, publication_review)
    tree = Tree(root, revision=candidate)
    manifest = validate_frontier(tree)
    trust = Trust(tree, authority, base)
    require(trust.policy["schema_version"] == 2, "broad promotion requires schema-2 coverage")
    baseline = trust.base.json(FRONTIER_PATH)
    migration = initial_mapping_migration(manifest, baseline, trust.policy,
                                          trust.base.json(POLICY_PATH))
    paths = promotion_boundary(manifest, baseline, trust.policy["coverage"],
                               delta["touched_paths"], legacy_migration=migration)
    require(ROLLOUT in trust.enrolled, "rollout must be enrolled before publication")
    environments = load_environments(tree, environment_path, trust.policy, authority=trust.authority)
    items = validate_coverage_state(tree, manifest, trust, environments=environments)
    checked = []
    for key in trust.enrolled:
        require(key in items, "enrolled publication item missing: " + key)
        phase = validate_delivery(tree, items[key], trust=trust,
            phase="review" if key == ROLLOUT else None, environment=environments.get(key))
        checked.append(f"{key}: {phase}")
    return {"checks": checked, "production_paths": paths,
            "publication_authorized": False, "activation_asserted": False,
            "scope": "read-only exact candidate inspection; owner approval and coordinated activation remain separate"}


def require(condition, message):
    if not condition:
        raise ValueError("WDQ-PREFLIGHT: " + message)


def clean_inputs(root, snapshot, base, *, workspace=None, explicit_paths=()):
    entries = tree_entries(root, base)
    for path, entry in entries.items():
        state = snapshot["files"].get(path, {"kind": "missing"})
        mode = entry["mode"]
        require(mode in ("100644", "100755", "120000"), "unsupported tracked mode: " + path)
        if mode == "120000":
            require(state["kind"] == "symlink", "tracked symlink changed: " + path)
            actual = sha256(os.fsencode(state["target"]))
        else:
            require(state["kind"] == "file", "tracked file absent or redirected: " + path)
            require(bool(state["mode"] & 0o111) == (mode == "100755"), "tracked executable mode changed: " + path)
            actual = state["sha256"]
        require(actual == sha256(git(root, "cat-file", "blob", entry["oid"])),
                "tracked bytes differ from base: " + path)
    inputs = set(snapshot["files"])
    if workspace is not None:
        inputs = workspace.input_paths(inputs, root=root, tracked_paths=entries,
                                       explicit_paths=explicit_paths, captured_files=snapshot["files"])
    for path in inputs:
        state = snapshot["files"][path]
        require(path in entries or state["kind"] == "directory",
                "untracked, ignored or missing reviewed input: " + path)


def preflight(root, *, base, candidate, input_roots, expected_local_trust, publication_review,
              workspace=None):
    require(not any(name in os.environ for name in REDIRECTS), "repository/index/config overrides are not activation inputs")
    require(type(input_roots) is list and bool(input_roots) and len(set(input_roots)) == len(input_roots),
            "explicit unique reviewed input roots required")
    require(type(expected_local_trust) is dict and set(expected_local_trust) == set(TRUST_KEYS)
            and all(type(values) is list and all(type(value) is str for value in values)
                    for values in expected_local_trust.values()), "exact named prior local trust required")
    root = Path(root).resolve(strict=True)
    before = protected_state(root, input_roots)
    require(before["symbolic_head"] == "refs/heads/main",
            "publication requires an attached main checkout; no branch switch performed")
    require(before["head"] == base, "live HEAD differs from pinned publication base")
    require(before["local_trust"] == expected_local_trust, "local trust differs from reviewed prior state")
    require(git(root, "status", "--porcelain=v1", "--untracked-files=all") == b"",
            "dirty index/worktree or untracked files; no publication performed")
    staged = git(root, "diff", "--cached", "--raw", "--no-ext-diff", "--no-textconv", base, "--")
    require(staged == b"", "index differs from pinned publication base")
    clean_inputs(root, before, base, workspace=workspace, explicit_paths=input_roots)
    delta = publication_delta(root, base=base, candidate=candidate)
    reviewed = verify_publication_review(delta, publication_review)
    after = protected_state(root, input_roots)
    require(canonical_json(before) == canonical_json(after), "protected state changed during preflight")
    require(git(root, "status", "--porcelain=v1", "--untracked-files=all") == b"",
            "worktree changed during preflight")
    return {"schema_version": 1, "kind": "datum.workflow-delivery.activation-preflight",
            "base": base, "candidate": candidate, "protected_state": before,
            "publication_delta": delta, "publication_authorized": False, "activation_asserted": False,
            "publication_review": reviewed,
            "scope": "read-only preconditions; owner receipt, independent evidence and activation coordination still required"}
