"""Freeze the exact inspected real roadmap without modifying live main."""

import argparse
import json
from pathlib import Path
import subprocess
import sys

ROOT = Path("/home/bfadmin/Documents/datum-eda")
BASE = "5e1144dfaf5f4c358c4ec850c48e9b2f0c0eee41"
CANDIDATE = "5407bc87d8b941e362bd26705a59d72da1a60176"
SOURCE = "0abb325d9f06e7c8e4ea7b36e3a7ea2d8cd6f45c"
RUNTIME = ROOT / ".git/datum-wdq/proposals/terminal-owner-20260910"


def main():
    global BASE, CANDIDATE
    assert sys.flags.isolated and sys.flags.no_site and sys.flags.dont_write_bytecode
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--base", default=BASE)
    parser.add_argument("--candidate", default=CANDIDATE)
    parser.add_argument("--packet-name", default="terminal-real-roadmap-5407")
    parser.add_argument("--output-name", default="terminal-integration-5407")
    args = parser.parse_args()
    BASE, CANDIDATE = args.base, args.candidate
    assert all(len(pin) == 40 and all(c in "0123456789abcdef" for c in pin) for pin in (BASE, CANDIDATE))
    for name in (args.packet_name, args.output_name):
        assert Path(name).name == name and name.startswith("terminal-")
    def git(*args):
        return subprocess.check_output(["git", "--no-replace-objects", "--no-optional-locks", *args], cwd=ROOT)
    assert git("rev-parse", "HEAD").decode().strip() == BASE
    assert not git("status", "--porcelain")
    bootstrap = RUNTIME / "scripts/workflow_delivery_source_only.py"
    raw = bootstrap.read_bytes()
    assert raw == git("show", CANDIDATE + ":scripts/workflow_delivery_source_only.py")
    namespace = {"__name__": "_terminal_integration_bootstrap"}
    exec(compile(raw, str(bootstrap), "exec"), namespace)
    namespace["install"](RUNTIME / "scripts")
    sys.path.insert(0, str(RUNTIME / "scripts"))
    from workflow_delivery_roadmap_snapshot import frontier_commits, prepare_roadmap_snapshot
    from workflow_delivery_capture_command import save
    from workflow_delivery_tree import Tree
    from workflow_delivery_authority import authority_sha256
    source, candidate = Tree(ROOT, revision=SOURCE), Tree(ROOT, revision=CANDIDATE)
    contract = candidate.json("specs/workflow_delivery/rollout.contract.json")
    assert source.manifest(contract["input_roots"]) == candidate.manifest(contract["input_roots"])
    assert authority_sha256(source, contract) == authority_sha256(candidate, contract)
    manifest = candidate.json("specs/active_frontier.json")
    keys = [item["key"] for item in manifest["frontier"]]
    assert len(keys) == 56
    pins = frontier_commits(manifest) | {
        SOURCE, "d88b2ee082effa4566e6d69b265f01df98d69390",
        "79fe4cd84174b8b31c224fe74346c23ba7ee48d5",
        "57879644be22eaae3570528473a5005640500d4a",
    }
    def export(pin):
        ref = "refs/datum/workflow-delivery-candidates/terminal-integration-" + pin
        existing = subprocess.run(["git", "rev-parse", "--verify", ref], cwd=ROOT, capture_output=True)
        if existing.returncode == 0:
            assert existing.stdout.decode().strip() == pin
        else:
            git("update-ref", ref, pin, "0" * 40)
        return ref
    evidence = {export(pin): pin for pin in sorted(pins - {CANDIDATE})}
    output = ROOT / ".git/datum-wdq/proposals" / args.output_name
    output.mkdir(exist_ok=True)
    assert not (output / "snapshot-result.json").exists()
    result = prepare_roadmap_snapshot(ROOT, ROOT / ".git/datum-wdq/proposals" / args.packet_name,
        source_ref=export(CANDIDATE), source_commit=CANDIDATE, frontier_keys=keys, evidence_refs=evidence)
    save(output / "snapshot-result.json", result)
    print(json.dumps({"candidate": CANDIDATE, "frontier_items": len(keys),
        "bundle_sha256": result["bundle_sha256"], "source_unchanged": result["source_unchanged"],
        "activation_performed": False}), flush=True)


if __name__ == "__main__":
    main()
