"""Tiny hermetic infrastructure records for validator tests, never native proof."""

from copy import deepcopy
import hashlib
from pathlib import Path
import subprocess
import tempfile

from workflow_delivery_contract import contract_sha256
from workflow_delivery_io import canonical_json, sha256
from workflow_delivery_shapes import DIMENSIONS, FOUNDATIONS
from workflow_delivery_tree import Tree


class Fixture:
    def __init__(self):
        self.temp = tempfile.TemporaryDirectory()
        self.root = Path(self.temp.name)
        self.git("init", "-q")
        self.git("config", "user.name", "Fixture")
        self.git("config", "user.email", "fixture@example.invalid")
        self.ref = {"path": "docs/authority.md", "marker": "<!-- RULE -->"}
        answer = {"disposition": "not_applicable", "reason": "Read-only fixture scope",
                  "authority_refs": [self.ref]}
        dimensions = {key: deepcopy(answer) for key in DIMENSIONS}
        dimensions["normal"]["disposition"] = "required"
        self.contract = {
            "schema_version": 1, "id": "contract", "frontier_key": "TASK",
            "issue_id": "dat-test", "category": "infrastructure",
            "consuming_workflow": "Validation fixture", "intent": "Read one file",
            "scope": "Read-only", "exclusions": "No product acceptance",
            "authority_refs": [self.ref], "route_ids": ["route"],
            "foundation_answers": {key: deepcopy(answer) for key in FOUNDATIONS},
            "open_decisions": [], "input_roots": ["src"],
            "consumers": [{"id": "reader", "entry_surfaces": ["api"],
                           "dispatch_key": "read", "handler_ref": {
                               "path": "src/read.py", "symbol": "read.read"},
                           "scope": "Fixture", "timing": "Immediate",
                           "persistence": "No mutation", "unavailable_reason": "Missing file",
                           "scenario_ids": ["S01"]}],
            "scenarios": [{"id": "S01", "consumer_ids": ["reader"],
                           "requirement_refs": [self.ref], "preconditions": "File exists",
                           "inputs": ["Read file"], "expected_visible": "value",
                           "expected_state": "unchanged", "dimensions": dimensions,
                           "method": "infrastructure"}],
            "proof_path": "docs/reviews/proof.json", "review_path": "docs/reviews/review.json",
        }
        self.write("src/read.py", b"def read(path):\n    return path.read_bytes()\n")
        self.write("docs/authority.md", b"<!-- RULE -->\nRead-only fixture authority.\n")
        self.write("research/source.md", b"Fixture-only requirement, not native EDA evidence.\n")
        self.write(".beads/issues.jsonl", b'{"id":"dat-test","status":"open"}\n')
        self.save("specs/spec_governance_manifest.json", {
            "entries": {self.ref["path"]: {"class": "governed"}}})
        route = {"id": "route", "sources": ["research/source.md"],
                 "consumers": [self.ref["path"]]}
        digest = hashlib.sha256()
        for path in sorted(route["sources"] + route["consumers"]):
            digest.update(path.encode() + b"\0" + (self.root / path).read_bytes() + b"\0")
        route["reviewed_digest"] = digest.hexdigest()
        self.save("specs/evidence_traceability_manifest.json", {"routes": [route]})
        self.save("contract.json", self.contract)
        self.stage()
        self.git("commit", "-qm", "fixture baseline")
        self.head = self.git("rev-parse", "HEAD").decode().strip()

    def close(self):
        self.temp.cleanup()

    def git(self, *args):
        return subprocess.run(["git", *args], cwd=self.root, check=True,
                              capture_output=True).stdout

    def write(self, path, raw):
        target = self.root / path
        target.parent.mkdir(parents=True, exist_ok=True)
        target.write_bytes(raw)

    def save(self, path, value):
        self.write(path, canonical_json(value))

    def stage(self):
        paths = sorted(p.relative_to(self.root).as_posix() for p in self.root.rglob("*")
                       if p.is_file() and ".git" not in p.relative_to(self.root).parts)
        self.git("add", "--", *paths)

    def blob(self, path, raw):
        self.write(path, raw)
        return {"path": path, "sha256": sha256(raw)}

    def proof(self):
        manifest = Tree(self.root).manifest(self.contract["input_roots"])
        inputs = self.blob("evidence/inputs.json", canonical_json(manifest))
        command = ["python3", "fixture_reader.py"]
        receipt = self.blob("evidence/build.json", canonical_json({
            "binary_sha256": sha256(b"fixture interpreter"), "build_command": command,
            "toolchain": "fixture-python", "input_manifest_sha256": inputs["sha256"],
            "exit_code": 0}))
        result = {
            "schema_version": 1, "contract_sha256": contract_sha256(self.contract),
            "producer_session": "original", "source_commit": self.head,
            "input_manifest": inputs,
            "build": {"command": command, "receipt": receipt, "toolchain": "fixture-python",
                      "source_clean": True},
            "fixture": self.blob("evidence/fixture.txt", b"value\n"),
            "environment": self.blob("evidence/environment.txt", b"test fixture environment\n"),
            "results": [{"scenario_id": "S01", "outcome": "pass", "actual_visible": "value",
                         "actual_state": "unchanged", "assertions": [{"dimension": "normal",
                         "expected": "value", "observed": "value", "outcome": "pass"}],
                         "artifacts": [self.blob("evidence/events.txt", b"fixture read => value\n")],
                         "defects": []}],
        }
        self.save(self.contract["proof_path"], result)
        self.stage()
        return result

    def snapshot(self):
        return {p.relative_to(self.root).as_posix(): p.read_bytes()
                for p in self.root.rglob("*") if p.is_file()
                and ".git" not in p.relative_to(self.root).parts}
