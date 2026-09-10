"""Validate exact committed producer artifacts without publishing main."""

import argparse
import json
import os
from pathlib import Path
import subprocess
import sys
import tempfile


def main():
    assert sys.flags.isolated and sys.flags.no_site and sys.flags.dont_write_bytecode
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--packet-name", default="typed-packet-01")
    args = parser.parse_args()
    assert Path(args.packet_name).name == args.packet_name and args.packet_name.startswith("typed-packet-")
    here = Path(__file__).resolve().parent
    root = here.parents[4]
    sys.path.insert(0, str(here))
    from assess_terminal_inputs import FINAL
    proposals = root / ".git/datum-wdq/proposals"
    runtime = proposals / "terminal-owner-20260910"
    assessment = proposals / "terminal-final-assessment-01"
    overlay = assessment / args.packet_name
    receipt_path = assessment / ("committed-" + args.packet_name + ".json")
    assert overlay.is_dir() and overlay.resolve() == overlay and not receipt_path.exists()
    def git(*args, data=None, env=None):
        return subprocess.check_output(["git", "--no-replace-objects", "--no-optional-locks", *args],
                                       cwd=root, input=data, env=env)
    main_head = git("rev-parse", "HEAD").decode().strip()
    assert not git("status", "--porcelain"), "commit owned tooling first"
    bootstrap = runtime / "scripts/workflow_delivery_source_only.py"
    raw = bootstrap.read_bytes()
    assert raw == git("show", FINAL + ":scripts/workflow_delivery_source_only.py")
    namespace = {"__name__": "_terminal_commit_source_only"}
    exec(compile(raw, str(bootstrap), "exec"), namespace)
    namespace["install"](runtime / "scripts")
    sys.path.insert(0, str(runtime / "scripts"))
    from workflow_delivery_tree import Tree
    from workflow_delivery_io import canonical_json, sha256
    from workflow_delivery_authority import authority_sha256
    from workflow_delivery_proof import packet_sha256, validate_proof
    from workflow_delivery_native import validate_correlations, validate_environment
    prefix = "docs/reviews/workflow-delivery-rollout/infrastructure/"
    verification_path = prefix + "terminal-renewal/typed/verification.json"
    expected = json.loads((overlay / verification_path).read_bytes())
    assert expected["source"] == FINAL and expected["activation_performed"] is False
    inventory = []
    with tempfile.TemporaryDirectory(prefix="terminal-typed-index-", dir=proposals) as directory:
        env = dict(os.environ, GIT_INDEX_FILE=str(Path(directory) / "index"))
        git("read-tree", FINAL, env=env)
        for path in sorted(overlay.rglob("*")):
            assert not path.is_symlink()
            if path.is_dir():
                continue
            relative = path.relative_to(overlay).as_posix()
            assert (relative == prefix + "proof.json" or relative.startswith(prefix + "terminal-renewal/")
                    or relative == prefix + "terminal-owner/inventory.json"), relative
            raw = path.read_bytes()
            inventory.append({"path": relative, "size": len(raw), "sha256": sha256(raw)})
            oid = git("hash-object", "-w", "--stdin", data=raw).decode().strip()
            git("update-index", "--add", "--cacheinfo", "100644", oid, relative, env=env)
        tree_id = git("write-tree", env=env).decode().strip()
    message = ("test(workflow): retain exact terminal assessment packet\n\n"
        "Problem: Prospective overlay checks do not establish committed-tree evidence integrity.\n"
        "Change: Overlay only exact producer packet artifacts on normative source0abb; preserve source and tracker.\n"
        "Proof: Committed input/authority, proof, environment, correlations and inventory checks follow; no independent completion asserted.\n"
        "Roadmap: WDQ-I04, dat-wdq-rollout-implementation-ffy; no main publication, product, dependency or licensing changes.\n")
    candidate = git("commit-tree", tree_id, "-p", FINAL, data=message.encode()).decode().strip()
    ref = "refs/datum-wdq/candidates/terminal-typed-" + candidate
    git("update-ref", ref, candidate, "0" * 40)
    base, tree = Tree(root, revision=FINAL), Tree(root, revision=candidate)
    contract = tree.json("specs/workflow_delivery/rollout.contract.json")
    assert tree.manifest(contract["input_roots"]) == base.manifest(contract["input_roots"])
    authority = authority_sha256(tree, contract)
    assert authority == authority_sha256(base, contract) == expected["authority_sha256"]
    proof = validate_proof(tree, contract)
    environment = tree.json(proof["environment"]["path"])
    validate_environment(tree, proof, environment, contract=contract)
    events = validate_correlations(tree, contract, proof)
    assert packet_sha256(contract, proof, authority) == expected["packet_sha256"]
    for row in inventory:
        raw = tree.read(row["path"])
        assert len(raw) == row["size"] and sha256(raw) == row["sha256"]
    assert git("rev-parse", "HEAD").decode().strip() == main_head and not git("status", "--porcelain")
    result = {"candidate": candidate, "ref": ref, "source": FINAL, "packet_sha256": expected["packet_sha256"],
        "authority_sha256": authority, "inventory": inventory, "event_blobs": len(events),
        "committed_proof_validation": "pass", "independent_review_complete": False,
        "strict_promotion_complete": False, "activation_performed": False}
    with receipt_path.open("xb") as stream:
        stream.write(canonical_json(result))
    print(json.dumps({key: value for key, value in result.items() if key != "inventory"}))


if __name__ == "__main__":
    main()
