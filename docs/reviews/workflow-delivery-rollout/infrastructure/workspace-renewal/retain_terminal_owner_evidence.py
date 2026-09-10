"""Retain completed supplemental captures in Git without publishing main."""

import hashlib
import json
import os
from pathlib import Path
import subprocess
import sys
import tempfile

HERE = Path(__file__).resolve().parent
ROOT = HERE.parents[4]
STORE = ROOT / ".git/datum-wdq/proposals"
PREFIX = "docs/reviews/workflow-delivery-rollout/infrastructure/terminal-owner/"


def main():
    if not (sys.flags.isolated and sys.flags.no_site and sys.flags.dont_write_bytecode):
        raise ValueError("run with Python -I -S -B")
    def git(*args, data=None, env=None):
        return subprocess.check_output(["git", "--no-replace-objects", "--no-optional-locks", *args],
                                       cwd=ROOT, input=data, env=env)
    base = git("rev-parse", "HEAD").decode().strip()
    assert not git("status", "--porcelain"), "commit owned tooling first"
    directories = ["terminal-owner-capture-" + n for n in ("01", "02", "03", "04")]
    directories += ["terminal-owner-recovery-01", "terminal-owner-independent-01",
                    "terminal-owner-independent-02", "terminal-owner-independent-recovery-01"]
    for name, count in (("terminal-owner-recovery-01", 50),
                        ("terminal-owner-independent-02", 60),
                        ("terminal-owner-independent-recovery-01", 50)):
        assert len(json.loads((STORE / name / "complete.json").read_bytes())["observations"]) == count
    payloads = {}
    for name in directories:
        directory = STORE / name
        assert directory.is_dir() and directory.resolve() == directory
        for current, dirs, files in os.walk(directory):
            dirs[:] = [d for d in dirs if d != "fixture"]
            for filename in files:
                path = Path(current) / filename
                assert path.is_file() and not path.is_symlink(), path
                payloads[PREFIX + path.relative_to(STORE).as_posix()] = path.read_bytes()
    # Preserve the exact previously executed producer-driver versions, including
    # failed setups. Their recorded SHA-256 must match before reconstruction is
    # accepted; this does not rewrite any original observation or hash.
    current = (HERE / "capture_terminal_owner.py").read_text()
    second = current.replace('        item.pop("landing_commit", None)\n', '')
    first = second.replace('import json\n', 'import hashlib\nimport json\n')
    first = first.replace('"unfinished-predecessor": "landed item requires every completion step to be complete"',
                          '"unfinished-predecessor": "pending"')
    first = first.replace('        h.issue["acceptance_criteria"] = "\\n".join(\n'
                          '            step["id"] + ": " + step["action"] for step in item["completion"]["steps"])',
                          '        h.issue["acceptance_criteria"] += "\\nNEXT-EXTRA: Additional pending fixture obligation"')
    first = first.replace('save(job / (surface + "-assessment.json"), assessment)',
                          'save(output / "assessment.json", assessment)')
    for name, raw in zip(directories[:4], (first.encode(), first.encode(), second.encode(), current.encode())):
        expected = json.loads((STORE / name / "source.json").read_bytes())["capture_script_sha256"]
        assert hashlib.sha256(raw).hexdigest() == expected
        payloads[PREFIX + "drivers/" + expected + ".py"] = raw
    replay = (HERE / "replay_terminal_owner.py").read_bytes()
    old_replay = git("cat-file", "blob", "58b68c48f4387c4ff000e1ed97336f07ceddb5f5")
    failed_replay = replay.replace(b'"equal_fields": list(fields)', b'"equal_fields": fields')
    for name, raw in (("terminal-owner-recovery-01", old_replay),
                      ("terminal-owner-independent-01", failed_replay),
                      ("terminal-owner-independent-02", replay),
                      ("terminal-owner-independent-recovery-01", replay)):
        expected = json.loads((STORE / name / "request.json").read_bytes())["script_sha256"]
        assert hashlib.sha256(raw).hexdigest() == expected
        payloads[PREFIX + "drivers/" + expected + ".py"] = raw
    recipe = (HERE / "test_terminal_owner_categories.py").read_bytes()
    for name in directories[:4]:
        assert hashlib.sha256(recipe).hexdigest() == json.loads((STORE / name / "source.json").read_bytes())["fixture_recipe_sha256"]
    payloads[PREFIX + "drivers/fixture-recipe.py"] = recipe
    inventory = [{"path": path, "size": len(raw), "sha256": hashlib.sha256(raw).hexdigest()}
                 for path, raw in sorted(payloads.items())]
    manifest = {"source": "9924678c39fe5d176e58ebfd8e25ab6feb213f35", "prepared_from_main": base,
                "producer_cases": 60, "producer_restored": 50,
                "independent_cases": 60, "independent_restored": 50,
                "excluded_failed_cases": ["terminal-owner-capture-02/unfinished-predecessor",
                                           "terminal-owner-capture-03/execution"],
                "failed_replay_setup": "terminal-owner-independent-01",
                "inventory": inventory, "activation_performed": False,
                "scope": "Supplemental terminal-owner evidence only; not the complete renewed rollout proof or installed validation."}
    payloads[PREFIX + "inventory.json"] = (json.dumps(manifest, indent=2) + "\n").encode()
    with tempfile.TemporaryDirectory(prefix="terminal-retention-index-", dir=STORE) as directory:
        env = dict(os.environ, GIT_INDEX_FILE=str(Path(directory) / "index"))
        git("read-tree", "--empty", env=env)
        for path, raw in sorted(payloads.items()):
            oid = git("hash-object", "-w", "--stdin", data=raw).decode().strip()
            git("update-index", "--add", "--cacheinfo", "100644", oid, path, env=env)
        tree = git("write-tree", env=env).decode().strip()
    message = ("test(workflow): retain terminal closeout capture and replay\n\n"
        "Problem: Ephemeral regression output cannot establish retained case evidence.\n"
        "Change: Archive raw producer/reviewer observations, fixture bundles, exact drivers and all failed attempts.\n"
        "Proof:60 producer and60 independent cases plus50 restored successes each; source9924678c, exact raw hashes retained.\n"
        "Roadmap: WDQ-I04, dat-wdq-rollout-implementation-ffy; supplemental evidence only, not full proof or activation. "
        "No product ownership, source permission, dependency or licensing change.\n")
    commit = git("commit-tree", tree, data=message.encode()).decode().strip()
    ref = "refs/datum-wdq/evidence/terminal-owner-" + commit
    git("update-ref", ref, commit, "0" * 40)
    assert git("rev-parse", "HEAD").decode().strip() == base and not git("status", "--porcelain")
    print(json.dumps({"commit": commit, "ref": ref, "files": len(payloads),
                      "bytes": sum(map(len, payloads.values())), "activation_performed": False}))


if __name__ == "__main__":
    main()
