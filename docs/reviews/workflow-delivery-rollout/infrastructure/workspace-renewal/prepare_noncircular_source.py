"""Freeze the normative-only renewal amendment without changing runtime bytes."""

import hashlib
import json
import os
from pathlib import Path
import subprocess
import sys
import tempfile

SOURCE = "9924678c39fe5d176e58ebfd8e25ab6feb213f35"
PM = "docs/decisions/PRODUCT_MECHANICS_042_BROAD_WORKFLOW_DELIVERY_ENFORCEMENT.md"
INFRA = "specs/WORKFLOW_DELIVERY_INFRASTRUCTURE_CONTRACT.md"
PLAN = "specs/WORKFLOW_DELIVERY_EXECUTION_PLAN.md"
CONTRACT = "specs/workflow_delivery/rollout.contract.json"


def main():
    assert sys.flags.isolated and sys.flags.no_site and sys.flags.dont_write_bytecode
    root = Path(__file__).resolve().parents[5]
    def git(*args, data=None, env=None):
        return subprocess.check_output(["git", "--no-replace-objects", "--no-optional-locks", *args],
                                       cwd=root, input=data, env=env)
    def read(ref, path):
        return git("show", ref + ":" + path)
    def block(raw, start, end):
        text = raw.decode()
        assert text.count(start) == text.count(end) == 1
        left, right = text.index(start), text.index(end)
        assert left < right
        return text[left:right]
    base = git("rev-parse", "HEAD").decode().strip()
    assert git("symbolic-ref", "HEAD").strip() == b"refs/heads/main"
    assert not git("status", "--porcelain")
    marker = "## Ratification evidence boundary"
    old, current = read(SOURCE, PM).decode(), read(base, PM).decode()
    assert old.count(marker) == current.count(marker) == 1
    payloads = {PM: (old.split(marker)[0] + marker + current.split(marker)[1]).encode()}
    old, current = read(SOURCE, INFRA).decode(), read(base, INFRA).decode()
    start, end = "<!-- WDQ-INFRA-NONCIRCULAR-RENEWAL -->", "<!-- WDQ-INFRA-NONCIRCULAR-RENEWAL-END -->"
    assert start not in old
    addition = block(current.encode(), start, end) + end + "\n\n"
    insertion = "WDQ-TOOLS must turn this reviewed procedure"
    assert old.count(insertion) == 1
    old = old.replace(insertion, addition + insertion, 1)
    rows = lambda text: [line for line in text.splitlines() if line.startswith("| INFRA-S04-11 |")]
    assert len(rows(old)) == len(rows(current)) == 1
    payloads[INFRA] = old.replace(rows(old)[0], rows(current)[0], 1).encode()
    start, end = "<!-- REQ:WORKFLOW-DELIVERY-IMPLEMENTATION:WDQ-I04 -->", "<!-- REQ:WORKFLOW-DELIVERY-IMPLEMENTATION:WDQ-I05 -->"
    old = read(SOURCE, PLAN)
    payloads[PLAN] = old.decode().replace(block(old, start, end), block(read(base, PLAN), start, end), 1).encode()
    contract, current = json.loads(read(SOURCE, CONTRACT)), json.loads(read(base, CONTRACT))
    target = next(s for s in contract["scenarios"] if s["id"] == "INFRA-S04")
    inputs = next(s for s in current["scenarios"] if s["id"] == "INFRA-S04")["inputs"]
    assert target["inputs"] == [s for s in inputs if "WDQ-INFRA-NONCIRCULAR-RENEWAL" not in s]
    target["inputs"] = inputs
    payloads[CONTRACT] = (json.dumps(contract, indent=2) + "\n").encode()
    trace_path = "specs/evidence_traceability_manifest.json"
    trace = json.loads(read(SOURCE, trace_path))
    for route in trace["routes"]:
        if route["id"] not in ("workflow-delivery-infrastructure", "workflow-delivery-rollout"):
            continue
        digest = hashlib.sha256()
        for path in sorted(route["sources"] + route["consumers"]):
            raw = payloads[path] if path in payloads else read(SOURCE, path)
            digest.update(path.encode() + b"\0" + raw + b"\0")
        route["reviewed_digest"] = digest.hexdigest()
    payloads[trace_path] = (json.dumps(trace, indent=2) + "\n").encode()
    store = root / ".git/datum-wdq/proposals"
    with tempfile.TemporaryDirectory(prefix="noncircular-index-", dir=store) as directory:
        env = dict(os.environ, GIT_INDEX_FILE=str(Path(directory) / "index"))
        git("read-tree", SOURCE, env=env)
        for path, raw in sorted(payloads.items()):
            oid = git("hash-object", "-w", "--stdin", data=raw).decode().strip()
            git("update-index", "--cacheinfo", "100644", oid, path, env=env)
        tree = git("write-tree", env=env).decode().strip()
    changed = git("diff-tree", "--no-commit-id", "--name-only", "-r", SOURCE, tree).decode().splitlines()
    assert set(changed) == set(payloads)
    assert all(not path.endswith(".py") for path in changed)
    message = ("docs(workflow): freeze noncircular renewal authority\n\n"
        "Problem: Real-roadmap integration cannot be its own reviewed packet prerequisite.\n"
        "Change: Overlay five reviewed normative/route files only; preserve all runtime, scopes and historical evidence.\n"
        "Proof: Exact changed-path set excludes every runtime module; fresh final construction and applicability assessment remain required.\n"
        "Roadmap: WDQ-I04, dat-wdq-rollout-implementation-ffy; preparation only, no activation, product, dependency or licensing change.\n")
    candidate = git("commit-tree", tree, "-p", SOURCE, data=message.encode()).decode().strip()
    ref = "refs/datum-wdq/candidates/noncircular-source-" + candidate
    git("update-ref", ref, candidate, "0" * 40)
    assert git("rev-parse", "HEAD").decode().strip() == base and not git("status", "--porcelain")
    print(json.dumps({"source": candidate, "parent": SOURCE, "ref": ref,
        "prepared_from_main": base, "changed_paths": changed, "runtime_bytes_changed": False,
        "proof_reuse_asserted": False, "activation_performed": False}))


if __name__ == "__main__":
    main()
