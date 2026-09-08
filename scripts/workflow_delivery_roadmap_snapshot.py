"""Freeze committed roadmap input without rewriting its rows or installing trust.

This is snapshot preparation, not real-roadmap gate proof or final input closure.
The explicit retained source ref is only an export handle; its exact commit pin
is checked before and after. Never export all local refs or host configuration.
"""

from pathlib import Path
import re

from workflow_delivery_bootstrap import pinned_commit
from workflow_delivery_capture_command import save
from workflow_delivery_capture_state import git, protected_state
from workflow_delivery_io import canonical_json, parse_json, sha256
from workflow_delivery_support_store import support_locations


def frontier_commits(value):
    """Named PM025 commit references, not arbitrary hash-looking prose."""
    found = set()
    if isinstance(value, dict):
        for key, member in value.items():
            if key in ("revision", "landing_commit", "head") and isinstance(member, str) and re.fullmatch(r"[a-f0-9]{40}|[a-f0-9]{64}", member):
                found.add(member)
            found.update(frontier_commits(member))
    elif isinstance(value, list):
        for member in value:
            found.update(frontier_commits(member))
    return found


def prepare_roadmap_snapshot(root, destination, *, source_ref, source_commit, frontier_keys, evidence_refs=None):
    root = Path(root).resolve(strict=True)
    pinned_commit(root, source_commit)
    locations = support_locations(root, source_commit)
    destination = Path(destination)
    if (not destination.is_absolute() or destination.parent != locations["proposals"]
            or destination.resolve() != destination or destination.exists()):
        raise ValueError("fresh direct child of resolved Git-common proposals required")
    if not isinstance(source_ref, str) or not source_ref.startswith("refs/datum/workflow-delivery-candidates/"):
        raise ValueError("explicit retained candidate export ref required")
    if (type(frontier_keys) is not list or not frontier_keys
            or not all(type(key) is str and key for key in frontier_keys)
            or len(set(frontier_keys)) != len(frontier_keys)):
        raise ValueError("explicit unique expected Frontier identities required")
    if git(root, "rev-parse", "--verify", source_ref).decode().strip() != source_commit:
        raise ValueError("export ref differs from exact source commit")
    exports = {source_ref: source_commit}
    if evidence_refs is not None:
        if type(evidence_refs) is not dict:
            raise ValueError("explicit evidence ref-to-commit map required")
        for name, pin in evidence_refs.items():
            if not isinstance(name, str) or not name.startswith("refs/datum/workflow-delivery-candidates/") or name == source_ref:
                raise ValueError("distinct retained evidence export ref required")
            pinned_commit(root, pin)
            if git(root, "rev-parse", "--verify", name).decode().strip() != pin:
                raise ValueError("evidence export ref differs from exact commit")
            exports[name] = pin
    if git(root, "status", "--porcelain=v1", "--untracked-files=all"):
        raise ValueError("source checkout must be clean before snapshot preparation")
    raw = git(root, "show", source_commit + ":specs/active_frontier.json")
    manifest = parse_json(raw, "source Frontier")
    actual = [item["key"] for item in manifest["frontier"]]
    if len(actual) != len(frontier_keys) or set(actual) != set(frontier_keys):
        raise ValueError("source Frontier differs from complete expected identities")
    before = protected_state(root, [])
    locations["proposals"].mkdir(parents=True, exist_ok=True)
    destination.mkdir()
    save(destination / "request.json", {"source_ref": source_ref, "source_commit": source_commit,
         "frontier_keys": frontier_keys, "evidence_refs": evidence_refs or {}, "activation_asserted": False})
    save(destination / "source-before.json", before)
    try:
        bundle, restored = destination / "source.bundle", destination / "snapshot"
        git(root, "bundle", "create", str(bundle), *sorted(exports))
        git(root, "bundle", "verify", str(bundle))
        heads = git(root, "bundle", "list-heads", str(bundle)).decode().splitlines()
        if sorted(heads) != sorted(pin + " " + name for name, pin in exports.items()):
            raise ValueError("bundle must advertise only the exact reviewed export refs")
        restored.mkdir()
        git(restored, "init", "-q")
        git(restored, "fetch", "--quiet", "--no-tags", str(bundle), *sorted(exports))
        git(restored, "checkout", "--quiet", "--detach", source_commit)
        for pin in sorted(frontier_commits(manifest)):
            try:
                pinned_commit(restored, pin)
            except ValueError as error:
                raise ValueError("referenced Frontier commit absent from restored history: " + pin) from error
        tree = git(root, "rev-parse", source_commit + "^{tree}").decode().strip()
        if git(restored, "rev-parse", "HEAD^{tree}").decode().strip() != tree:
            raise ValueError("restored tree differs from exact source")
        git(restored, "diff", "--exit-code", source_commit, "--")
        state = protected_state(restored, [])
        if state["head"] != source_commit or any(state["local_trust"].values()):
            raise ValueError("restored snapshot identity or local trust differs")
        if (restored / "specs/active_frontier.json").read_bytes() != raw:
            raise ValueError("restored roadmap bytes differ")
        after = protected_state(root, [])
        save(destination / "source-after.json", after)
        if canonical_json(before) != canonical_json(after):
            raise ValueError("source state changed during snapshot preparation")
        for name, pin in exports.items():
            if git(root, "rev-parse", "--verify", name).decode().strip() != pin:
                raise ValueError("export ref moved during snapshot preparation")
        save(destination / "restored-state.json", state)
        result = {"schema_version": 1, "kind": "datum.workflow-delivery.roadmap-snapshot",
                  "source_commit": source_commit, "tree": tree, "frontier_keys": actual,
                  "evidence_refs": evidence_refs or {}, "resolved_frontier_commits": sorted(frontier_commits(manifest)),
                  "bundle_sha256": sha256(bundle.read_bytes()), "source_unchanged": True,
                  "restored_trust_installed": False, "readiness_asserted": False,
                  "activation_asserted": False, "acceptance_asserted": False,
                  "scope": "Exact committed repository and history input; no gate run, synthetic row replacement, environment selection, lease renewal or input-closure claim."}
        save(destination / "result.json", result)
        return result
    except Exception as error:
        save(destination / "failure.json", {"type": type(error).__name__, "detail": str(error),
                                             "activation_asserted": False})
        raise
