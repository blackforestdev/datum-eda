"""Build explicitly reassessed producer evidence; never publish or accept it."""

from collections import Counter
import json
from pathlib import Path
import subprocess
import sys


def main():
    assert sys.flags.isolated and sys.flags.no_site and sys.flags.dont_write_bytecode
    here = Path(__file__).resolve().parent
    root = here.parents[4]
    proposals = root / ".git/datum-wdq/proposals"
    assessment = proposals / "terminal-final-assessment-01"
    output = assessment / "typed-packet-01"
    assert not output.exists()
    runtime = proposals / "terminal-owner-20260910"
    sys.path.insert(0, str(here))
    from assess_terminal_inputs import FINAL, OBSERVED
    bootstrap = runtime / "scripts/workflow_delivery_source_only.py"
    def git(*args):
        return subprocess.check_output(["git", "--no-replace-objects", "--no-optional-locks", *args], cwd=root)
    raw = bootstrap.read_bytes()
    assert raw == git("show", FINAL + ":scripts/workflow_delivery_source_only.py")
    namespace = {"__name__": "_terminal_packet_source_only"}
    exec(compile(raw, str(bootstrap), "exec"), namespace)
    namespace["install"](runtime / "scripts")
    sys.path.insert(0, str(runtime / "scripts"))
    from workflow_delivery_tree import Tree
    from workflow_delivery_io import canonical_json, parse_json, sha256
    from workflow_delivery_authority import authority_sha256
    from workflow_delivery_contract import contract_sha256
    from workflow_delivery_proof import packet_sha256, validate_proof
    from workflow_delivery_native import validate_correlations, validate_environment
    base = Tree(root, revision=FINAL)
    contract = base.json("specs/workflow_delivery/rollout.contract.json")
    read = lambda path: json.loads(path.read_bytes())
    construction = read(assessment / "construction.json")
    assembled = read(assessment / "observations/dispatch-observations.json")
    assert construction["source_commit"] == assembled["source_commit"] == FINAL
    assert assembled["observed_source_commit"] == OBSERVED and len(assembled["observations"]) == 1400
    assert assembled["construction_sha256"] == sha256((assessment / "construction.json").read_bytes())
    assert construction["input_manifest"] == base.manifest(contract["input_roots"])
    archive_commit = read(proposals / "terminal-packet-retention-01/retention.json")["commit"]
    assert archive_commit == "57879644be22eaae3570528473a5005640500d4a"
    prefix = "docs/reviews/workflow-delivery-rollout/infrastructure/terminal-renewal/"
    typed = prefix + "typed/"
    payloads = {prefix + name: git("show", archive_commit + ":" + prefix + name)
                for name in ("raw.tar.xz", "inventory.json")}
    terminal_prefix = "docs/reviews/workflow-delivery-rollout/infrastructure/terminal-owner/"
    supplement_commit = "79fe4cd84174b8b31c224fe74346c23ba7ee48d5"
    # The supplement inventory binds exact original helpers, fixture bundles and
    # excluded failures. Its immutable Git ref remains required for restoration.
    payloads[terminal_prefix + "inventory.json"] = git("show", supplement_commit + ":" + terminal_prefix + "inventory.json")
    class Overlay:
        staged = False
        def __getattr__(self, name):
            return getattr(base, name)
        def read(self, path, **kwargs):
            return payloads[path] if path in payloads else base.read(path, **kwargs)
        def json(self, path, **kwargs):
            return parse_json(self.read(path, **kwargs), path)
    tree = Overlay()
    def blob(path):
        return {"path": path, "sha256": sha256(tree.read(path))}
    def write(path, value):
        assert path not in payloads
        payloads[path] = canonical_json(value)
        return blob(path)
    receipt = read(assessment / "build-receipt.json")
    binary = receipt["binary_sha256"]
    assert sha256(Path(construction["interpreter"]["path"]).read_bytes()) == binary
    receipt_blob = write(typed + "build-receipt.json", receipt)
    manifest = write(typed + "input-manifest.json", construction["input_manifest"])
    assert manifest["sha256"] == receipt["input_manifest_sha256"]
    construction_blob = write(typed + "construction.json", construction)
    observations_blob = write(typed + "observations.json", assembled)
    authority = authority_sha256(base, contract)
    assert authority == construction["authority_sha256"]
    selection = base.json("specs/workflow_delivery/rollout.environments.json")
    environments = [row["environment"] for row in selection["environments"]
                    if row["frontier_key"] == "WORKFLOW-DELIVERY-IMPLEMENTATION"]
    assert len(environments) == 1
    environment_blob = environments[0]
    assert blob(environment_blob["path"]) == environment_blob
    environment = base.json(environment_blob["path"])
    fixture = write(typed + "fixtures.json", {"observed_source": OBSERVED,
        "raw_archive": blob(prefix + "raw.tar.xz"), "raw_inventory": blob(prefix + "inventory.json"),
        "terminal_supplement_commit": supplement_commit, "terminal_supplement_inventory": blob(terminal_prefix + "inventory.json"),
        "scope": "Exact retained synthetic/workspace input packets; not an installed or full real-roadmap success claim."})
    limits = write(typed + "limits.json", {"assessment_source": FINAL, "observed_source": OBSERVED,
        "assessment_kind": "current applicability assessment of immutable historical execution",
        "pending": assembled["pending"], "excluded_attempts": assembled["excluded"],
        "original_raw_authority_preserved": True, "new_execution_asserted": False,
        "independent_replay_complete": False, "activation_performed": False})
    registry = write(typed + "registry.json", {"schema_version": 1, "binary_sha256": binary,
        "entries": read(assessment / "observations/registry-entries.json")})
    dispatches = read(assessment / "observations/scenario-dispatches.json")
    session = "codex-wdq-rollout-implementation-20260908"
    common = [fixture, limits, construction_blob, observations_blob, environment_blob,
              blob(prefix + "raw.tar.xz"), blob(prefix + "inventory.json"), blob(terminal_prefix + "inventory.json")]
    results = []
    for scenario in contract["scenarios"]:
        sid = scenario["id"]
        rows = [row for row in assembled["observations"] if row["case_id"].startswith(sid)]
        counts = Counter(row["phase"] for row in rows)
        visible = (f"Current applicability assessment of {len(rows)} immutable source992 executions; "
                   f"phases {dict(counts)}, exits {dict(Counter(str(r['returncode']) for r in rows))}. "
                   "Original raw timestamps, authority and invocation identities are retained. This is not a new execution.")
        state = ("Exact runtime/build-input equality and retained artifact/trace/protected-state checks passed. "
                 "Current authority binds this assessment, not the historical subprocesses. "
                 "Independent applicability/replay and mandatory I04 full-roadmap/installed checks remain pending.")
        event = write(typed + sid + "-events.json", {"schema_version": 1, "scenario_id": sid,
            "producer_session": session, "method": scenario["method"], "binary_sha256": binary,
            "authority_sha256": authority, "inputs": scenario["inputs"], "dispatches": dispatches[sid],
            "actual_visible": visible, "actual_state": state})
        roles = write(typed + sid + "-roles.json", {"kind": "datum.workflow-delivery.artifacts/v1",
            "scenario_id": sid, "events": [event], "captures": [], "state": common, "registry": registry})
        evidence = {"normal": visible, "invalid": visible, "scope": state,
            "failure_recovery": f"Retained recovery observations: {counts['recovery']}; named workspace restorations are additionally retained.",
            "accessibility": "Retained text-only diagnostic/usage outcomes passed; no GUI accessibility assertion.",
            "cancel": "Retained actual interruption and protected-state observations are included, not newly executed.",
            "save_reopen": "Retained fresh-process recovery is included; no native Save/reopen assertion."}
        assertions = [{"dimension": name, "expected": value["reason"], "observed": evidence[name], "outcome": "pass"}
                      for name, value in scenario["dimensions"].items() if value["disposition"] == "required"]
        results.append({"scenario_id": sid, "outcome": "pass", "actual_visible": visible, "actual_state": state,
            "assertions": assertions, "artifacts": [roles, event, registry, *common],
            "defects": ["dat-wdq-rollout-implementation-ffy"]})
    proof = {"schema_version": 1, "contract_sha256": contract_sha256(contract), "producer_session": session,
        "source_commit": FINAL, "input_manifest": manifest,
        "build": {"command": receipt["build_command"], "receipt": receipt_blob, "toolchain": receipt["toolchain"], "source_clean": True},
        "fixture": fixture, "environment": environment_blob, "results": results}
    write(contract["proof_path"], proof)
    validate_proof(tree, contract)
    validate_environment(tree, proof, environment, contract=contract)
    events = validate_correlations(tree, contract, proof)
    result = {"source": FINAL, "observed_source": OBSERVED, "packet_sha256": packet_sha256(contract, proof, authority),
        "authority_sha256": authority, "event_blobs": len(events), "observations": 1400,
        "scope": "Prospective producer assessment packet only; not independent completion, strict promotion or installation.",
        "activation_performed": False}
    write(typed + "verification.json", result)
    output.mkdir()
    for path, raw in payloads.items():
        target = output / path
        target.parent.mkdir(parents=True, exist_ok=True)
        with target.open("xb") as stream:
            stream.write(raw)
    print(json.dumps(result))


if __name__ == "__main__":
    main()
