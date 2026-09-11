"""Read-only publication preconditions; never activate or claim owner approval.

An eventual activation transaction must revalidate these observations under its
own coordination boundary. Successful preparation is not a lock or permission.
"""

import os
from copy import deepcopy
from pathlib import Path

from workflow_delivery_bootstrap import git
from workflow_delivery_capture_state import TRUST_KEYS, protected_state
from workflow_delivery_io import canonical_json, sha256
from workflow_delivery_publication_delta import publication_delta, tree_entries, verify_publication_review

REDIRECTS = ("GIT_DIR", "GIT_WORK_TREE", "GIT_COMMON_DIR", "GIT_INDEX_FILE", "GIT_NAMESPACE",
             "GIT_OBJECT_DIRECTORY", "GIT_ALTERNATE_OBJECT_DIRECTORIES", "GIT_CONFIG",
             "GIT_CONFIG_COUNT", "GIT_CONFIG_PARAMETERS")

ROLLOUT = "WORKFLOW-DELIVERY-IMPLEMENTATION"
S5A = "UVT-S5A-BUILD"

S5A_PREPARATION_SCOPE = {
    "frontier_key": S5A,
    "step_ids": ["S5A-C01"],
    "paths": [
        "crates/test-harness/Cargo.toml",
        "crates/test-harness/testdata/selection/s5a_v1",
        "crates/test-harness/tests/s5a_preparation.rs",
    ],
    "boundary_ref": {
        "path": "docs/decisions/PRODUCT_MECHANICS_042_BROAD_WORKFLOW_DELIVERY_ENFORCEMENT.md",
        "marker": "<!-- WDQ-042-PREPARATION -->",
    },
    "approval_ref": {
        "path": "docs/reviews/workflow-delivery-rollout/s5a/preparation-owner-20260910.json",
        "marker": "\"resolves_request\": \"Authorize S5A fixture and dispatch-evidence preparation under 6ece73ed.\"",
    },
}
PREPARATION_REVIEW = (
    "docs/reviews/workflow-delivery-rollout/infrastructure/"
    "preparation-bootstrap/independent-review.json"
)
PREPARATION_REPLAY_COMMANDS = {
    "targeted": ["python3", "-B", "-m", "unittest",
                 "test_workflow_delivery_activation_preflight.PreparationMigrationTest",
                 "test_workflow_delivery_source_scopes",
                 "test_workflow_delivery_category_boundaries.PreparationBoundaryTest"],
    "workflow-suite": ["python3", "-B", "-m", "unittest", "discover",
                       "-s", "scripts", "-p", "test_workflow_delivery*.py"],
    "traceability": ["python3", "-B", "scripts/check_evidence_traceability.py"],
    "governance": ["python3", "-B", "scripts/check_spec_governance.py"],
    "source-health": ["python3", "-B", "scripts/check_source_health.py"],
}

RECOVERY_REVIEW = (
    "docs/reviews/workflow-delivery-rollout/infrastructure/"
    "preparation-bootstrap/installed-recovery/independent-review.json"
)
RECOVERY_REPLAY_COMMANDS = {
    "activation-focused": ["python3", "-B", "-m", "unittest", "discover",
                           "-s", "scripts", "-p", "test_workflow_delivery_activation*.py"],
    "workflow-suite": PREPARATION_REPLAY_COMMANDS["workflow-suite"],
    "traceability": PREPARATION_REPLAY_COMMANDS["traceability"],
    "governance": PREPARATION_REPLAY_COMMANDS["governance"],
    "source-health": PREPARATION_REPLAY_COMMANDS["source-health"],
}
RECOVERY_IMPLEMENTATION_PATHS = {
    "scripts/test_workflow_delivery_activation_preflight.py",
    "scripts/test_workflow_delivery_activation_state.py",
    "scripts/workflow_delivery_activation.py",
    "scripts/workflow_delivery_activation_preflight.py",
    "scripts/workflow_delivery_activation_state.py",
    "scripts/workflow_delivery_checkpoints.py",
}


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


def preparation_scope_migration(manifest, baseline, policy, prior_policy):
    """Recognize only the reviewed first S5A preparation-scope amendment."""
    from workflow_delivery_coverage import index_items
    from workflow_delivery_io import canonical_json
    from workflow_delivery_trust import policy_shape

    policy_shape(prior_policy)
    policy_shape(policy)
    old_coverage = prior_policy.get("coverage", {})
    new_coverage = policy.get("coverage", {})
    if (prior_policy.get("schema_version") != 2
            or "preparation_scopes" in old_coverage
            or "preparation_scopes" not in new_coverage):
        return False
    require(canonical_json(manifest) == canonical_json(baseline),
            "preparation-scope promotion cannot change Frontier state")
    current = index_items(manifest)
    rollout = current.get(ROLLOUT, {})
    step = next((s for s in rollout.get("completion", {}).get("steps", [])
                 if s.get("id") == "WDQ-I05"), {})
    require(rollout.get("state") == "in_progress"
            and rollout.get("authorization") == "execution"
            and rollout.get("completion", {}).get("canonical_next_step_id") == "WDQ-I05"
            and step.get("kind") == "execution" and step.get("status") == "in_progress"
            and bool(rollout.get("claim")),
            "preparation-scope promotion requires the unchanged live WDQ-I05 boundary")
    s5a = current.get(S5A, {})
    s5a_step = next((s for s in s5a.get("completion", {}).get("steps", [])
                    if s.get("id") == "S5A-C01"), {})
    require(s5a.get("state") == "specified" and s5a.get("authorization") == "planning"
            and not s5a.get("claim")
            and s5a.get("completion", {}).get("canonical_next_step_id") == "S5A-C01"
            and s5a_step.get("kind") == "planning" and s5a_step.get("status") == "pending",
            "preparation-scope promotion must preserve the released S5A planning boundary")
    require(new_coverage["preparation_scopes"] == [S5A_PREPARATION_SCOPE],
            "preparation-scope promotion must add only the exact approved S5A scope")
    normalized = deepcopy(policy)
    normalized["coverage"].pop("preparation_scopes")
    old_scopes = old_coverage["source_scopes"]
    new_scopes = normalized["coverage"]["source_scopes"]
    require(len(old_scopes) == len(new_scopes) == 1
            and old_scopes[0]["frontier_key"] == new_scopes[0]["frontier_key"] == ROLLOUT
            and old_scopes[0]["step_ids"] == ["WDQ-I03", "WDQ-REVIEW"]
            and new_scopes[0]["step_ids"] == ["WDQ-I03", "WDQ-REVIEW", "WDQ-I05"],
            "preparation-scope promotion may only extend the rollout scope to WDQ-I05")
    normalized_scope = deepcopy(new_scopes[0])
    normalized_scope["step_ids"] = ["WDQ-I03", "WDQ-REVIEW"]
    require(normalized_scope == old_scopes[0],
            "preparation-scope promotion changed the existing rollout scope")
    normalized["coverage"]["source_scopes"] = deepcopy(old_scopes)
    require(canonical_json(normalized) == canonical_json(prior_policy),
            "preparation-scope promotion changed unrelated owner policy")
    return True


def preparation_upgrade_boundary(manifest, baseline, coverage, changed_paths):
    """Bound the first preparation amendment to the unchanged live I05 claim."""
    from workflow_delivery_coverage import index_items
    from workflow_delivery_coverage_shapes import coverage_shape
    from workflow_delivery_source_scopes import contains

    coverage_shape(coverage)
    require(manifest == baseline, "preparation upgrade cannot change Frontier state")
    item = index_items(manifest)[ROLLOUT]
    claim = item["claim"]
    scopes = coverage["source_scopes"]
    require(len(scopes) == 1 and scopes[0]["frontier_key"] == ROLLOUT
            and scopes[0]["step_ids"] == ["WDQ-I03", "WDQ-REVIEW", "WDQ-I05"],
            "preparation upgrade requires the exact extended rollout scope")
    checked = []
    for path in changed_paths:
        require(not path.startswith("docs/gui/prototypes/"),
                "preparation upgrade touches the protected prototype lane")
        if contains(coverage["production_roots"], path):
            require(path in scopes[0]["paths"],
                    "preparation upgrade production path lacks exact reviewed scope: " + path)
            require(contains(claim["scope"], path),
                    "preparation upgrade production path lies outside the live I05 claim: " + path)
            checked.append(path)
    return sorted(set(checked))


def validate_preparation_upgrade_review(tree, *, base):
    """Validate retained independent replay for the exact producer commit."""
    from workflow_delivery_io import sha256
    from workflow_delivery_publication_delta import publication_delta

    review = tree.json(PREPARATION_REVIEW)
    require(type(review) is dict and set(review) == {
        "schema_version", "kind", "base_commit", "producer_commit", "producer_session",
        "reviewer_session", "independent_of", "disposition", "implementation_paths",
        "replay_checks", "findings", "activation_asserted"},
        "closed preparation-upgrade review required")
    require(review["schema_version"] == 1
            and review["kind"] == "datum.workflow-delivery.preparation-upgrade-review",
            "preparation-upgrade review version 1 required")
    producer = review["producer_commit"]
    require(review["base_commit"] == base and tree.resolve(producer) == producer,
            "review must pin the exact activation base and producer commit")
    require(tree.git("merge-base", base, producer).decode().strip() == base
            and tree.git("merge-base", producer, tree.revision).decode().strip() == producer,
            "reviewed producer must be between activation base and candidate")
    delta = publication_delta(tree.root, base=base, candidate=producer)
    require(review["implementation_paths"] == delta["touched_paths"],
            "review must enumerate the exact producer history")
    require(review["producer_session"] == "codex-wdq-i05-preparation-repair-20260911"
            and type(review["reviewer_session"]) is str and review["reviewer_session"]
            and review["reviewer_session"] != review["producer_session"]
            and review["independent_of"] == [review["producer_session"]],
            "reviewer must be explicitly independent of the producer session")
    require(review["disposition"] == "approve" and review["findings"] == [],
            "preparation upgrade requires approved review with no undisposed findings")
    require(review["activation_asserted"] is False,
            "independent review cannot assert activation")
    require(type(review["replay_checks"]) is list
            and [row.get("id") for row in review["replay_checks"]]
                == list(PREPARATION_REPLAY_COMMANDS),
            "review must retain every exact replay check in order")
    retained_paths = {PREPARATION_REVIEW}
    review_root = PREPARATION_REVIEW.rsplit("/", 1)[0] + "/"
    for row in review["replay_checks"]:
        require(type(row) is dict and set(row) == {
            "id", "command", "returncode", "stdout", "stderr"},
            "closed replay-check record required")
        require(row["command"] == PREPARATION_REPLAY_COMMANDS[row["id"]]
                and row["returncode"] == 0,
                "independent replay command or outcome differs")
        for stream in ("stdout", "stderr"):
            blob = row[stream]
            require(type(blob) is dict and set(blob) == {"path", "sha256"},
                    "replay output Blob required")
            raw = tree.read(blob["path"], committed=True)
            require(sha256(raw) == blob["sha256"],
                    "replay output hash differs: " + blob["path"])
            require(blob["path"].startswith(review_root),
                    "replay output must stay inside the bounded review directory")
            require(blob["path"] not in retained_paths,
                    "replay output paths must be unique")
            retained_paths.add(blob["path"])
    review_delta = publication_delta(tree.root, base=producer, candidate=tree.revision)
    require(review_delta["touched_paths"] == sorted(retained_paths),
            "candidate changed bytes outside the exact independent-review artifacts")
    return review


def installed_preparation_recovery(manifest, baseline, policy, prior_policy, changed_paths):
    """Recognize only the exact I05 repair of the partially installed gate."""
    from workflow_delivery_coverage import index_items
    from workflow_delivery_io import canonical_json
    from workflow_delivery_source_scopes import contains
    from workflow_delivery_trust import policy_shape

    policy_shape(policy)
    policy_shape(prior_policy)
    require(canonical_json(policy) == canonical_json(prior_policy),
            "installed recovery cannot change owner policy")
    current, old = index_items(manifest), index_items(baseline)
    require(set(current) == set(old) and ROLLOUT in current,
            "installed recovery must preserve every Frontier identity")
    for key in current:
        if key != ROLLOUT:
            require(current[key] == old[key],
                    "installed recovery changes another Frontier lane: " + key)
    item, prior = deepcopy(current[ROLLOUT]), deepcopy(old[ROLLOUT])
    require(item.get("state") == prior.get("state") == "in_progress"
            and item.get("authorization") == prior.get("authorization") == "execution"
            and item.get("completion", {}).get("canonical_next_step_id")
                == prior.get("completion", {}).get("canonical_next_step_id") == "WDQ-I05",
            "installed recovery requires the unchanged live I05 lifecycle")
    claim, old_claim = item.get("claim"), prior.get("claim")
    require(type(claim) is dict and type(old_claim) is dict,
            "installed recovery requires the existing synchronized I05 claim")
    scope, old_scope = set(claim["scope"]), set(old_claim["scope"])
    require(scope - old_scope == RECOVERY_IMPLEMENTATION_PATHS - old_scope
            and old_scope <= scope,
            "installed recovery may add only its exact implementation paths to the I05 claim")
    claim["scope"], old_claim["scope"] = sorted(scope), sorted(scope)
    require(item == prior, "installed recovery changed I05 state outside its path scope")
    rollout_scope = next(row for row in policy["coverage"]["source_scopes"]
                         if row["frontier_key"] == ROLLOUT)
    production = []
    for path in changed_paths:
        require(not path.startswith("docs/gui/prototypes/"),
                "installed recovery touches the protected prototype lane")
        if contains(policy["coverage"]["production_roots"], path):
            require(path in RECOVERY_IMPLEMENTATION_PATHS
                    and path in rollout_scope["paths"] and contains(claim["scope"], path),
                    "installed recovery production path is outside its exact reviewed scope: " + path)
            production.append(path)
    require(set(production) == RECOVERY_IMPLEMENTATION_PATHS,
            "installed recovery must contain its complete exact implementation set")
    return sorted(production)


def validate_installed_recovery_review(tree, *, base):
    """Validate independent replay retained after the exact recovery producer."""
    review = tree.json(RECOVERY_REVIEW)
    require(type(review) is dict and set(review) == {
        "schema_version", "kind", "base_commit", "producer_commit", "producer_session",
        "reviewer_session", "independent_of", "disposition", "implementation_paths",
        "failed_activation_log_sha256", "replay_checks", "findings", "activation_asserted"},
        "closed installed-recovery review required")
    require(review["schema_version"] == 1
            and review["kind"] == "datum.workflow-delivery.installed-recovery-review",
            "installed-recovery review version 1 required")
    producer = review["producer_commit"]
    require(review["base_commit"] == base and tree.resolve(producer) == producer,
            "recovery review must pin the exact partial-install base and producer")
    require(tree.git("merge-base", base, producer).decode().strip() == base
            and tree.git("merge-base", producer, tree.revision).decode().strip() == producer,
            "reviewed recovery producer must be between activation base and candidate")
    delta = publication_delta(tree.root, base=base, candidate=producer)
    require(review["implementation_paths"] == delta["touched_paths"],
            "recovery review must enumerate the exact producer history")
    require(review["producer_session"] == "codex-wdq-i05-preparation-repair-20260911"
            and type(review["reviewer_session"]) is str and review["reviewer_session"]
            and review["reviewer_session"] != review["producer_session"]
            and review["independent_of"] == [review["producer_session"]],
            "recovery reviewer must be explicitly independent of the producer")
    require(review["disposition"] == "approve" and review["findings"] == []
            and review["activation_asserted"] is False,
            "installed recovery requires approved review without activation assertion")
    require(type(review["failed_activation_log_sha256"]) is str
            and len(review["failed_activation_log_sha256"]) == 64
            and all(c in "0123456789abcdef" for c in review["failed_activation_log_sha256"]),
            "recovery review must pin the failed activation log digest")
    require(type(review["replay_checks"]) is list
            and [row.get("id") for row in review["replay_checks"]]
                == list(RECOVERY_REPLAY_COMMANDS),
            "recovery review must retain every exact replay check in order")
    retained = {RECOVERY_REVIEW}
    root = RECOVERY_REVIEW.rsplit("/", 1)[0] + "/"
    for row in review["replay_checks"]:
        require(type(row) is dict and set(row) == {
            "id", "command", "returncode", "stdout", "stderr"},
            "closed recovery replay-check record required")
        require(row["command"] == RECOVERY_REPLAY_COMMANDS[row["id"]]
                and row["returncode"] == 0,
                "recovery replay command or outcome differs")
        for stream in ("stdout", "stderr"):
            blob = row[stream]
            require(type(blob) is dict and set(blob) == {"path", "sha256"},
                    "recovery replay output Blob required")
            raw = tree.read(blob["path"], committed=True)
            require(blob["path"].startswith(root) and sha256(raw) == blob["sha256"]
                    and blob["path"] not in retained,
                    "recovery replay output path or hash differs: " + blob["path"])
            retained.add(blob["path"])
    review_delta = publication_delta(tree.root, base=producer, candidate=tree.revision)
    require(review_delta["touched_paths"] == sorted(retained),
            "recovery candidate changed bytes outside independent-review artifacts")
    return review


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
    prior_policy = trust.base.json(POLICY_PATH)
    recovery = (canonical_json(trust.policy) == canonical_json(prior_policy)
                and canonical_json(manifest) != canonical_json(baseline))
    preparation_migration = False if recovery else preparation_scope_migration(
        manifest, baseline, trust.policy, prior_policy)
    migration = False if preparation_migration else initial_mapping_migration(
        manifest, baseline, trust.policy, prior_policy) if not recovery else False
    if recovery:
        paths = installed_preparation_recovery(
            manifest, baseline, trust.policy, prior_policy, delta["touched_paths"])
        validate_installed_recovery_review(tree, base=base)
    elif preparation_migration:
        paths = preparation_upgrade_boundary(
            manifest, baseline, trust.policy["coverage"], delta["touched_paths"])
    else:
        paths = promotion_boundary(manifest, baseline, trust.policy["coverage"],
                                   delta["touched_paths"], legacy_migration=migration)
    if preparation_migration:
        validate_preparation_upgrade_review(tree, base=base)
    require(ROLLOUT in trust.enrolled, "rollout must be enrolled before publication")
    environments = load_environments(tree, environment_path, trust.policy, authority=trust.authority)
    if preparation_migration or recovery:
        from workflow_delivery_coverage_runtime import validate_coverage_structure
        items, readiness = validate_coverage_structure(tree, manifest, trust)
        for key in readiness:
            if key != ROLLOUT and key not in trust.enrolled:
                validate_delivery(tree, items[key], phase="ready",
                                  environment=environments.get(key))
    else:
        items = validate_coverage_state(tree, manifest, trust, environments=environments)
    checked = []
    for key in trust.enrolled:
        require(key in items, "enrolled publication item missing: " + key)
        repair_structure = preparation_migration and key == ROLLOUT
        phase = validate_delivery(tree, items[key], trust=None if repair_structure else trust,
            phase="structure" if repair_structure else "review" if key == ROLLOUT else None,
            environment=environments.get(key))
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
