"""Freeze committed roadmap input without rewriting its rows or installing trust.

This is snapshot preparation, not real-roadmap gate proof or final input closure.
The explicit retained source ref is only an export handle; its exact commit pin
is checked before and after. Never export all local refs or host configuration.
"""

from pathlib import Path
import re
import sys
import uuid

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


def capture_roadmap_snapshot(packet, destination, runtime, environment, *, surface):
    """Observe the unchanged real roadmap; no synthetic row or receipt replacement."""
    from workflow_delivery_capture_command import capture_command
    from workflow_delivery_capture_assertions import capture_envelope, observer_calls
    from workflow_delivery_support_bundle import prepare_bundle, verify_bundle

    if surface not in ("C", "R", "S-check", "S-details", "H"):
        raise ValueError("explicit roadmap capture surface required")
    packet, runtime = Path(packet).resolve(strict=True), Path(runtime).resolve(strict=True)
    request = parse_json((packet / "request.json").read_bytes(), "snapshot request")
    snapshot = parse_json((packet / "result.json").read_bytes(), "snapshot result")
    pin = snapshot["source_commit"]
    if (snapshot["kind"] != "datum.workflow-delivery.roadmap-snapshot"
            or request["source_commit"] != pin
            or sha256((packet / "source.bundle").read_bytes()) != snapshot["bundle_sha256"]):
        raise ValueError("exact retained roadmap snapshot required")
    if "WORKFLOW-DELIVERY-GATE-PILOT" not in snapshot["frontier_keys"]:
        raise ValueError("real accepted pilot must be present; synthetic TASK is not a substitute")
    destination = Path(destination).resolve()
    if destination.exists() or destination.is_relative_to(packet) or packet.is_relative_to(destination):
        raise ValueError("fresh capture destination outside retained snapshot required")
    destination.mkdir()
    root, output = destination / "fixture", destination / "capture"
    root.mkdir()
    git(root, "init", "-q")
    exports = [request["source_ref"], *sorted(request["evidence_refs"])]
    git(root, "fetch", "--quiet", "--no-tags", str(packet / "source.bundle"), *exports)
    git(root, "checkout", "--quiet", "--detach", pin)
    if git(root, "rev-parse", "HEAD^{tree}").decode().strip() != snapshot["tree"]:
        raise ValueError("restored roadmap tree differs")
    manifest = parse_json((root / "specs/active_frontier.json").read_bytes(), "restored Frontier")
    if [item["key"] for item in manifest["frontier"]] != snapshot["frontier_keys"]:
        raise ValueError("restored roadmap identities differ")
    nonce = str(uuid.uuid4())
    with (root / ".git/datum-wdq-capture-owner").open("x") as stream:
        stream.write(nonce + "\n")
    selection = "specs/workflow_delivery/rollout.environments.json"
    for key, value in (("AuthorityRef", pin), ("BaseRef", pin), ("EnvironmentPath", selection)):
        git(root, "config", "--local", "datum.workflowDelivery" + key, value)
    selector = surface.startswith("S-")
    actual_surface = "S" if selector else surface
    support = None
    if surface == "H":
        support = prepare_bundle(root, pin)
        locations = support_locations(root, pin)
        git(root, "config", "--local", "core.hooksPath", str(locations["hooks"]))
        git(root, "config", "--local", "datum.workflowDeliveryRunnerPath", str(locations["runner"]))
        command = [str(locations["hook"])]
        environment = dict(environment, DATUM_WDQ_OBSERVE_PATH=str(output / "calls.jsonl"))
    else:
        script = runtime / "scripts" / ("project_status.py" if selector else "check_workflow_delivery.py")
        arguments = ["--root", str(root)]
        if selector:
            arguments += ["--json", surface.removeprefix("S-")]
            if surface == "S-details":
                arguments += ["WORKFLOW-DELIVERY-IMPLEMENTATION"]
        else:
            arguments += ["--enforce", "--authority-ref", pin, "--base-ref", pin,
                          "--environment-path", selection]
            arguments += ["--candidate-ref", pin] if surface == "R" else ["--staged"]
        command = [sys.executable, "-I", "-S", "-B", str(runtime / "scripts/workflow_delivery_observe_python.py"),
                   "--script", str(script), "--source-root", str(runtime), "--output", str(output / "calls.jsonl"),
                   "--function", "main", "--function", "selector_failures", "--", *arguments]
    capture_command(root, output, command, environment, fixture_nonce=nonce,
                    case_id="INFRA-S04-11", surface=actual_surface, untracked_roots=[], timeout=180)
    result, invocation = capture_envelope(output, "INFRA-S04-11", actual_surface)
    if result["returncode"] != 0 or (output / "stderr.bin").read_bytes():
        raise ValueError("real roadmap success required; retain actual refusal for reconciliation")
    raw = (output / "stdout.bin").read_bytes()
    report = parse_json(raw.splitlines()[-1] if surface == "H" else raw, "roadmap output")
    if selector:
        if report["ok"] is not True:
            raise ValueError("real selector did not pass")
    elif report["findings"] or "WORKFLOW-DELIVERY-GATE-PILOT: accept" not in report["checks"]:
        raise ValueError("unchanged accepted pilot did not pass")
    if support is None:
        observer_calls(output, result, invocation, script, arguments)
    else:
        if verify_bundle(root, pin) != support:
            raise ValueError("prepared hook support changed")
        bootstrap = locations["runner"].parent / "workflow_delivery_bootstrap.py"
        arguments = ["--root", str(root), "--authority-ref", pin, "--base-ref", pin,
                     "--environment-path", selection, "--runner", str(locations["runner"])]
        events = observer_calls(output, result, invocation, bootstrap, arguments)
        if not any(row["event"] == "call" and row["function"] == "main"
                   and row["source"]["path"] == str(locations["runner"])
                   and row["source"]["sha256"] == sha256(locations["runner"].read_bytes())
                   for row in events):
            raise ValueError("actual pinned validator call missing")
    assessment = {"case_id": "INFRA-S04-11", "surface": actual_surface,
                  "selector_operation": surface if selector else None, "source_commit": pin,
                  "invocation_id": result["invocation_id"], "observation_matches_expected": True,
                  "result_sha256": sha256((output / "result.json").read_bytes()),
                  "complete_matrix": False, "input_closure_verified": False,
                  "publication_authorized": False, "acceptance_asserted": False}
    save(destination / "assessment.json", assessment)
    return assessment
