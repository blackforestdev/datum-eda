"""Build a prospective evidence overlay; never publish, activate or accept it."""

import argparse
from collections import Counter
import hashlib
import json
from pathlib import Path
import platform
import subprocess
import sys


PIN = "0e5b8064a9faca407fac43bd2b362ba4d7200327"
INFRA = "docs/reviews/workflow-delivery-rollout/infrastructure/"
WORKSPACE = INFRA + "workspace-renewal/"
TYPED = WORKSPACE + "typed/"


def read(path):
    return json.loads(path.read_bytes())


def sha(path):
    return hashlib.sha256(path.read_bytes()).hexdigest()


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    for name in ("root", "runtime", "observations", "output"):
        parser.add_argument("--" + name, type=Path, required=True)
    variants = parser.add_mutually_exclusive_group()
    variants.add_argument("--index-repair", action="store_true")
    variants.add_argument("--sequencing-repair", action="store_true")
    args = parser.parse_args()
    pin = "4e11d60b6f0ec50aa391c68ed39a0df138adf8cf" if args.index_repair else PIN
    typed = WORKSPACE + "index-repair-typed/" if args.index_repair else TYPED
    if args.sequencing_repair:
        pin = "1d48f249dc7071fc3718b345a4ab16b366af43be"
        typed = WORKSPACE + "sequencing-typed/"
    root, runtime, observations, output = (getattr(args, key).resolve()
        for key in ("root", "runtime", "observations", "output"))
    retained = root / ".git/datum-wdq/proposals/wdq-full-candidate-inspection-20260909"
    repaired = root / ".git/datum-wdq/proposals/index-repair-evidence-20260910"
    if args.sequencing_repair:
        repaired = root / ".git/datum-wdq/proposals/sequencing-evidence-20260910"
    assert output.is_relative_to(repaired if args.sequencing_repair else retained) and not output.exists()
    git = lambda *words: subprocess.check_output(["git", "--no-optional-locks", *words], cwd=runtime)
    assert git("rev-parse", "HEAD").decode().strip() == pin and not git("status", "--porcelain")
    sys.path.insert(0, str(runtime / "scripts"))
    from workflow_delivery_authority import authority_sha256
    from workflow_delivery_contract import contract_sha256
    from workflow_delivery_io import canonical_json, parse_json
    from workflow_delivery_native import validate_correlations, validate_environment
    from workflow_delivery_proof import packet_sha256, validate_proof
    from workflow_delivery_tree import Tree

    baseline = read(repaired / "owner-reopened-roadmap-preparation-reconciled.json")["fixture_commit"] if args.index_repair else pin
    if args.sequencing_repair:
        baseline = read(repaired / "owner-reopened-roadmap-preparation.json")["fixture_commit"]
    base = Tree(runtime, revision=baseline)
    contract = base.json("specs/workflow_delivery/rollout.contract.json")
    assert base.manifest(contract["input_roots"]) == Tree(runtime, revision=pin).manifest(contract["input_roots"])
    assert contract["proof_path"] == INFRA + "proof.json"
    assembled = read(observations / "dispatch-observations.json")
    assert assembled["source_commit"] == pin and len(assembled["observations"]) == 1314
    output.mkdir()

    class Overlay:
        """Prospective artifact validation, not evidence of Git publication."""
        staged = False

        def __getattr__(self, name):
            return getattr(base, name)

        def read(self, path, **kwargs):
            if (output / path).is_file():
                return (output / path).read_bytes()
            if path.startswith(WORKSPACE) and (root / path).is_file():
                return (root / path).read_bytes()
            return base.read(path, **kwargs)

        def json(self, path, **kwargs):
            return parse_json(self.read(path, **kwargs), path)

    tree = Overlay()

    def blob(path):
        return {"path": path, "sha256": hashlib.sha256(tree.read(path)).hexdigest()}

    def write(path, value):
        target = output / path
        target.parent.mkdir(parents=True, exist_ok=True)
        with target.open("x") as stream:
            json.dump(value, stream, indent=2)
            stream.write("\n")
        return blob(path)

    authority = authority_sha256(tree, contract)
    receipt_path = WORKSPACE + "typed-build-receipt.json"
    if args.index_repair:
        receipt_path = typed + "build-receipt.json"
        write(receipt_path, read(repaired / "build-receipt.json"))
        write(typed + "build-process.json", read(repaired / "build-process.json"))
    if args.sequencing_repair:
        frozen = tree.json(WORKSPACE + "sequencing-source-freeze.json")
        assert frozen["source_commit"] == pin and frozen["source_clean"]
        assert frozen["authority_sha256"] == authority
        assert frozen["input_manifest"] == base.manifest(contract["input_roots"])
        assert sha(Path(frozen["interpreter"]["path"])) == frozen["interpreter"]["sha256"]
        receipt_path = typed + "build-receipt.json"
        write(receipt_path, {"binary_sha256": frozen["interpreter"]["sha256"],
            "build_command": frozen["command"], "exit_code": 0,
            "input_manifest_sha256": frozen["input_manifest_sha256"],
            "toolchain": frozen["toolchain"]})
    receipt = tree.json(receipt_path)
    binary = receipt["binary_sha256"]
    registry = write(typed + "registry.json", {"schema_version": 1, "binary_sha256": binary,
        "entries": read(observations / "registry-entries.json")})
    dispatches = read(observations / "scenario-dispatches.json")
    regression = tree.json(WORKSPACE + "renewed-workspace-regression.json")
    tools = []
    for name, value in regression["tools_after"].items():
        if name == "shell":
            version = subprocess.check_output(["dpkg-query", "-W", "-f=${Package} ${Version}", "dash"], text=True)
        else:
            version = value["version_stdout"].strip()
        assert sha(Path(value["resolved_path"])) == value["sha256"]
        tools.append({"name": "interpreter" if name == "python" else name,
            "path": value["resolved_path"], "sha256": value["sha256"], "version": version})
    bash = Path("/usr/bin/bash")
    tools.append({"name": "bash", "path": str(bash), "sha256": sha(bash),
        "version": subprocess.check_output([str(bash), "--version"], text=True).strip()})
    tool_observation = write(typed + "observed-tools.json", {"schema_version": 1,
        "os": platform.platform(), "tools": tools,
        "scope": "Supplemental observed tool identities, not replacement environment authority.",
        "regression_receipt": blob(WORKSPACE + "renewed-workspace-regression.json")})
    selection = base.json("specs/workflow_delivery/rollout.environments.json")
    selected = [row["environment"] for row in selection["environments"]
                if row["frontier_key"] == "WORKFLOW-DELIVERY-IMPLEMENTATION"]
    assert len(selected) == 1
    environment_blob = selected[0]
    assert blob(environment_blob["path"]) == environment_blob
    environment = base.json(environment_blob["path"])
    manifest = read(repaired / "input-manifest.json") if args.index_repair else tree.json(WORKSPACE + "renewed-input-manifest.json")
    if args.sequencing_repair:
        manifest = frozen["input_manifest"]
    assert manifest == base.manifest(contract["input_roots"])
    manifest_path = typed + "input-manifest.json"
    (output / manifest_path).write_bytes(canonical_json(manifest))
    assert blob(manifest_path)["sha256"] == receipt["input_manifest_sha256"]
    archives = sorted((root / WORKSPACE).glob("renewed-*-captures.tar.xz"))
    archives += [root / WORKSPACE / "full-workspace-inventory-captures.tar.xz",
                 root / WORKSPACE / "typed-observations.tar.xz"]
    if args.index_repair:
        archives = sorted((root / WORKSPACE).glob("index-repair-*.tar.xz"))
        archives += [root / WORKSPACE / name for name in (
            "owner-reopened-preparation.tar.xz", "owner-reopened-real-captures.tar.xz",
            "index-repair-typed-observations.tar.xz")]
        archives = sorted(set(archives))
    if args.sequencing_repair:
        archives = sorted((root / WORKSPACE).glob("sequencing-*.tar.xz"))
    archive_blobs = [blob(path.relative_to(root).as_posix()) for path in archives]
    fixture = write(typed + "fixtures.json", {"schema_version": 1, "source_commit": pin,
        "capture_archives": archive_blobs, "raw_capture_paths": sorted({str(Path(row["capture_path"]).parent)
            for row in assembled["observations"]}), "real_roadmap_snapshot":
        read(repaired / "owner-reopened-roadmap-preparation.json") if args.sequencing_repair else
        read(repaired / "owner-reopened-roadmap-preparation-reconciled.json") if args.index_repair else tree.json(WORKSPACE + "renewed-roadmap-snapshot.json"),
        "real_roadmap_binding": assembled.get("real_roadmap_binding"),
        "limitation": "Real-roadmap archives omit duplicated full checkouts; exact retained source bundle and raw fixtures remain at the recorded local paths. No standalone portable replay claim."})
    common = [blob(WORKSPACE + name) for name in ("typed-build-observation.json",
        "typed-assembly-independent-inspection.json", "renewed-workspace-regression.json",
        "renewed-workspace-independent-inspection.json", "shell-package-observation.json",
        "assemble_observations.py", "build_typed_packet.py", "typed-packet-attempts.json",
        "renewed-s03-setup-attempts.json", "renewed-s02-ownership-observation.json",
        "ownership-formatter-observation.json", "renewed-s01-observation.json",
        "full-workspace-inventory-observation.json")]
    common += [blob(path.relative_to(root).as_posix())
               for path in sorted((root / WORKSPACE).glob("renewed-*-observation.json"))]
    if args.index_repair:
        common = [blob(path.relative_to(root).as_posix())
                  for path in sorted((root / WORKSPACE).glob("index-repair-*.json"))]
        common += [blob(WORKSPACE + name) for name in (
            "proof-renewal-authorization.json", "owner-reopened-preparation.json",
            "owner-reopened-real-captures.json", "assemble_observations.py", "build_typed_packet.py")]
        common += [blob(typed + "build-process.json")]
    if args.sequencing_repair:
        common = [blob(path.relative_to(root).as_posix())
                  for path in sorted((root / WORKSPACE).glob("sequencing-*.json"))]
        common += [blob(WORKSPACE + name) for name in
                   ("assemble_observations.py", "build_typed_packet.py", "freeze_sequencing_source.py")]
        review_record = tree.json(WORKSPACE + "sequencing-live-review-observation.json")
        log_path = root / ".git/datum-wdq/logs/sequencing-live-review-377323b2.json"
        assert sha(log_path) == review_record["verified_observation"]["log_sha256"]
        inspection = read(log_path)
        assert inspection["ok"] and not any(inspection[key] for key in
            ("evidence_validated", "publication_authorized", "activation_asserted"))
        assert inspection["review_inspection"]["candidate"] == baseline
        assert inspection["review_inspection"]["selected_step"] == "WDQ-COMPAT"
        common.append(write(typed + "producer-review-inspection.json", inspection))
    common = list({item["path"]: item for item in common}.values())
    common.append(tool_observation)
    limits = write(typed + "limits.json", {"schema_version": 1,
        "producer_scope": "Current-source producer observations; no independent execution, publication inspection, activation or product acceptance.",
        "runtime_scope": "Bounded repository, fixture and named-tool accounting, not exhaustive descendant/shared-library tracing. Existing tracing flags are not upgraded.",
        "pending": ["WDQ-RECHECK independently repeats entrypoints and exact pre-review inspection"
                    if args.sequencing_repair else
                    "WDQ-RECHECK independently repeats entrypoints and exact publication inspection",
                    "WDQ-I04 owner ratification/activation", "WDQ-I05 three real cohorts", "WDQ-I06 owner acceptance"]})
    results = []
    session = "wdq-index-repair-producer-20260910" if args.index_repair else "wdq-compat-producer-20260909"
    if args.sequencing_repair:
        session = "wdq-sequencing-repair-producer-20260910"
    for scenario in contract["scenarios"]:
        sid = scenario["id"]
        rows = [row for row in assembled["observations"] if row["case_id"].startswith(sid)]
        counts = Counter(row["phase"] for row in rows)
        exits = Counter(str(row["returncode"]) for row in rows)
        visible = f"Observed {len(rows)} invocations: phases {dict(counts)}, return codes {dict(exits)}. Case-specific expected outcomes passed; interruptions and uninvoked hooks remain explicit."
        state = f"All {len(rows)} recorded protected before/after states matched. Producer evidence only; independent repetition/publication remains under WDQ-RECHECK. No activation or product acceptance."
        if args.sequencing_repair:
            state = f"All {len(rows)} recorded protected before/after states matched. Independent repetition and pre-review inspection remain under WDQ-RECHECK; final full-evidence inspection belongs to I04. No activation or product acceptance."
            if sid == "INFRA-S05":
                visible += " Separately recorded producer pre-review source/history inspection passed for candidate " + baseline + "; its exact log is included, without claiming final publication authorization."
        observation = write(typed + sid + "-observations.json", {"schema_version": 1,
            "scenario_id": sid, "source_commit": pin, "observations": rows})
        event = write(typed + sid + "-events.json", {"schema_version": 1,
            "scenario_id": sid, "producer_session": session, "method": scenario["method"],
            "binary_sha256": binary, "authority_sha256": authority, "inputs": scenario["inputs"],
            "dispatches": dispatches[sid], "actual_visible": visible, "actual_state": state})
        state_blobs = [observation, limits, fixture, environment_blob, *common, *archive_blobs]
        roles = write(typed + sid + "-roles.json", {"kind": "datum.workflow-delivery.artifacts/v1",
            "scenario_id": sid, "events": [event], "captures": [], "state": state_blobs,
            "registry": registry})
        evidence = {"normal": visible, "invalid": visible, "scope": state,
            "failure_recovery": f"Original recovery invocations: {counts['recovery']}; additional compatibility restorations retain their named cases. All expected recovery outcomes passed.",
            "accessibility": "Original text-only diagnostic and usage cases passed their existing capture assessments; no native GUI accessibility claim.",
            "cancel": "Five real interrupted invocations and hashed interruption receipts are retained with unchanged protected state.",
            "save_reopen": "Fresh-process recovery after interruption/refusal is retained; no native Save or design reopen claim."}
        assertions = [{"dimension": key, "expected": value["reason"], "observed": evidence[key],
            "outcome": "pass"} for key, value in scenario["dimensions"].items()
            if value["disposition"] == "required"]
        defects = ["dat-wdq-workspace-inputs-zmt"] if sid == "INFRA-S05" else []
        if args.index_repair or args.sequencing_repair:
            defects.append("dat-wdq-index-refresh-ogw")
            if sid in ("INFRA-S04", "INFRA-S05"):
                defects.append("dat-wdq-proof-renewal-cycle-qgb")
        if args.sequencing_repair and sid == "INFRA-S05":
            defects.append("dat-wdq-review-publication-cycle-a1y")
        results.append({"scenario_id": sid, "outcome": "pass", "actual_visible": visible,
            "actual_state": state, "assertions": assertions, "artifacts": [roles, event, registry, *state_blobs],
            "defects": defects})
    proof = {"schema_version": 1, "contract_sha256": contract_sha256(contract),
        "producer_session": session, "source_commit": pin,
        "input_manifest": blob(manifest_path),
        "build": {"command": receipt["build_command"], "receipt": blob(receipt_path),
                  "toolchain": receipt["toolchain"], "source_clean": True},
        "fixture": fixture, "environment": environment_blob, "results": results}
    write(contract["proof_path"], proof)
    validate_proof(tree, contract)
    validate_environment(tree, proof, environment, contract=contract)
    events = validate_correlations(tree, contract, proof)
    assert git("rev-parse", "HEAD").decode().strip() == pin and not git("status", "--porcelain")
    result = {"schema_version": 1, "packet_sha256": packet_sha256(contract, proof, authority),
        "authority_sha256": authority, "correlated_event_blobs": len(events), "observations": 1314,
        "validation": "Prospective overlay: proof freshness/shape, headless environment and artifact-role/dispatch correlations passed.",
        "committed_candidate_verified": False, "independent_replay_complete": False, "activation_performed": False}
    write(typed + "verification.json", result)
    print(json.dumps(result))


if __name__ == "__main__":
    main()
