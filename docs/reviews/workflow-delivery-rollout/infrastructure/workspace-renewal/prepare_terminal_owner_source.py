"""Freeze the bounded terminal-owner repair; never publish or reuse old proof."""

import hashlib
import json
import os
from pathlib import Path
import subprocess
import sys
import tempfile


SOURCE = "1d48f249dc7071fc3718b345a4ab16b366af43be"
LANE = "docs/reviews/workflow-delivery-rollout/infrastructure/workspace-renewal/"
PM = "docs/decisions/PRODUCT_MECHANICS_042_BROAD_WORKFLOW_DELIVERY_ENFORCEMENT.md"
COVERAGE = "specs/WORKFLOW_DELIVERY_COVERAGE_PROPOSAL.md"
INFRA = "specs/WORKFLOW_DELIVERY_INFRASTRUCTURE_CONTRACT.md"
CONTRACT = "specs/workflow_delivery/rollout.contract.json"


def section(raw, start, end):
    text = raw.decode()
    if text.count(start) != 1 or text.count(end) != 1:
        raise ValueError("amendment anchors must be unique")
    left, right = text.index(start), text.index(end)
    if left >= right:
        raise ValueError("amendment anchors are reversed")
    return text[left:right]


def replace_section(old, current, start, end):
    return old.decode().replace(section(old, start, end),
                                section(current, start, end), 1).encode()


def main():
    if not (sys.flags.isolated and sys.flags.no_site and sys.flags.dont_write_bytecode):
        raise ValueError("run with Python -I -S -B")
    root = Path(__file__).resolve().parents[5]

    def git(*args, data=None, env=None):
        return subprocess.check_output(
            ["git", "--no-replace-objects", "--no-optional-locks", *args],
            cwd=root, input=data, env=env)

    def read(ref, path):
        return git("show", ref + ":" + path)

    base = git("rev-parse", "HEAD").decode().strip()
    assert git("symbolic-ref", "HEAD").strip() == b"refs/heads/main"
    assert not git("status", "--porcelain"), "commit owned preparation first"
    payloads = {
        "scripts/workflow_delivery_categories.py": read(base, LANE + "terminal_owner_categories.py"),
        PM: replace_section(read(SOURCE, PM), read(base, PM),
                            "- `external_lane`:", "Only external rows carry"),
        COVERAGE: replace_section(read(SOURCE, COVERAGE), read(base, COVERAGE),
                                 "<!-- WDQ-COVERAGE:EXTERNAL -->",
                                 "<!-- WDQ-COVERAGE:SOURCE-PERMISSIONS -->"),
    }
    block = section(read(base, INFRA), "<!-- WDQ-INFRA-TERMINAL-OWNER -->",
                    "<!-- WDQ-INFRA-S03 -->")
    source_infra = read(SOURCE, INFRA).decode()
    assert "WDQ-INFRA-TERMINAL-OWNER" not in source_infra
    payloads[INFRA] = source_infra.replace("<!-- WDQ-INFRA-S03 -->",
                                          block + "<!-- WDQ-INFRA-S03 -->", 1).encode()
    contract = json.loads(read(SOURCE, CONTRACT))
    current = json.loads(read(base, CONTRACT))
    amendment = next(s for s in current["scenarios"] if s["id"] == "INFRA-S02")["inputs"]
    target = next(s for s in contract["scenarios"] if s["id"] == "INFRA-S02")
    assert target["inputs"] == [value for value in amendment if "WDQ-INFRA-TERMINAL-OWNER" not in value]
    target["inputs"] = amendment
    payloads[CONTRACT] = (json.dumps(contract, indent=2) + "\n").encode()
    trace_path = "specs/evidence_traceability_manifest.json"
    trace = json.loads(read(SOURCE, trace_path))
    for route in trace["routes"]:
        if route["id"] not in ("workflow-delivery-rollout", "workflow-delivery-infrastructure"):
            continue
        digest = hashlib.sha256()
        for path in sorted(route["sources"] + route["consumers"]):
            raw = payloads[path] if path in payloads else read(SOURCE, path)
            digest.update(path.encode() + b"\0" + raw + b"\0")
        route["reviewed_digest"] = digest.hexdigest()
    payloads[trace_path] = (json.dumps(trace, indent=2) + "\n").encode()
    compile(payloads["scripts/workflow_delivery_categories.py"], "workflow_delivery_categories.py", "exec")
    common = Path(git("rev-parse", "--path-format=absolute", "--git-common-dir").decode().strip())
    store = common / "datum-wdq/proposals"
    assert store.is_dir() and not store.is_symlink()
    with tempfile.TemporaryDirectory(prefix="terminal-owner-index-", dir=store) as directory:
        env = dict(os.environ, GIT_INDEX_FILE=str(Path(directory) / "index"))
        git("read-tree", SOURCE, env=env)
        for path, raw in sorted(payloads.items()):
            oid = git("hash-object", "-w", "--stdin", data=raw).decode().strip()
            mode = git("ls-tree", SOURCE, "--", path).decode().split()[0]
            assert mode == "100644", path
            git("update-index", "--cacheinfo", mode, oid, path, env=env)
        tree = git("write-tree", env=env).decode().strip()
    message = ("fix(workflow): freeze terminal owner closeout source\n\n"
               "Problem: External coverage froze legitimate terminal owner closeout.\n"
               "Change: Apply the committed bounded repair and reconciled authority only.\n"
               "Proof: Compile repaired source; preserve other source, scopes and old evidence identities.\n"
               "Roadmap: WDQ-I04, dat-wdq-rollout-implementation-ffy, conditional PM042; "
               "source preparation only, not renewed proof or activation. No dependency/licensing change.\n")
    candidate = git("commit-tree", tree, "-p", SOURCE, data=message.encode()).decode().strip()
    retained = "refs/datum-wdq/candidates/terminal-owner-source-" + candidate
    git("update-ref", retained, candidate, "0" * 40)
    assert git("rev-parse", "HEAD").decode().strip() == base
    assert not git("status", "--porcelain")
    print(json.dumps({"source": candidate, "parent": SOURCE, "retained_ref": retained,
                      "prepared_from_main": base,
                      "changes": [{"path": p, "sha256": hashlib.sha256(raw).hexdigest()}
                                  for p, raw in sorted(payloads.items())],
                      "renewed_proof_asserted": False, "activation_performed": False,
                      "scope": "Frozen source only; old proof does not certify these changed inputs."}, indent=2))


if __name__ == "__main__":
    main()
