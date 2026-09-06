#!/usr/bin/env python3
"""Build identified pilot binaries and record actual receipts; no acceptance."""

import argparse
import json
from pathlib import Path
import subprocess
import tarfile
import tempfile

from workflow_delivery_io import sha256
from workflow_delivery_pilot_capture import digest, files
from workflow_delivery_tree import Tree

ROOT = Path(__file__).resolve().parents[1]
CONTRACT = "specs/workflow_delivery/pilot.contract.json"

def snapshot_inputs(snapshot, roots):
    result = []
    for root in roots:
        path = snapshot / root
        members = {root: digest(path)} if path.is_file() else {
            root + "/" + name: sha for name, sha in files(path).items()}
        result.extend({"path": name, "sha256": sha} for name, sha in members.items())
    return sorted(result, key=lambda item: item["path"])


def write(path, value):
    with path.open("x") as stream:
        stream.write(json.dumps(value, indent=2) + "\n")


def build_command(snapshot, target):
    return ["python3", str(snapshot / "scripts/run_cargo_guarded.py"),
            "--target-dir", str(target), "--workload", "proof", "--",
            "cargo", "build", "--offline", "--locked",
            "--manifest-path", str(snapshot / "Cargo.toml"),
            "-p", "datum-gui-app", "-p", "datum-eda-cli", "--bins"]


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--output", type=Path, required=True)
    parser.add_argument("--source-snapshot", type=Path)
    args = parser.parse_args()
    out = args.output.resolve()
    out.mkdir(parents=True, exist_ok=False)
    tree = Tree(ROOT)
    source = tree.resolve("HEAD")
    roots = tree.json(CONTRACT)["input_roots"]
    inputs = Tree(ROOT, revision=source).manifest(roots)
    snapshot = args.source_snapshot.resolve() if args.source_snapshot else Path(
        tempfile.mkdtemp(prefix="datum-wdq-source-", dir=ROOT.parent))
    if not args.source_snapshot:
        with subprocess.Popen(["git", "archive", "--format=tar", source], cwd=ROOT,
                              stdout=subprocess.PIPE) as archive:
            with tarfile.open(fileobj=archive.stdout, mode="r|") as source_tar:
                source_tar.extractall(snapshot, filter="data")
        if archive.returncode:
            raise RuntimeError("Git archive failed")
    if snapshot_inputs(snapshot, roots) != inputs:
        raise RuntimeError("source archive does not match committed input closure")
    command = build_command(snapshot, ROOT / "target")
    write(out / "input-manifest.json", inputs)
    toolchain = subprocess.check_output(["rustc", "-Vv"], text=True).strip()
    with (out / "build.log").open("x") as log:
        result = subprocess.run(command, cwd=snapshot, stdout=log, stderr=subprocess.STDOUT)
    if result.returncode:
        raise RuntimeError(f"guarded build failed: {result.returncode}; see {out / 'build.log'}")
    if snapshot_inputs(snapshot, roots) != inputs:
        raise RuntimeError("source inputs changed during build")
    for name, binary in [("gui", "datum-gui"), ("cli", "datum-eda")]:
        write(out / (name + "-receipt.json"), {
            "binary_sha256": digest(ROOT / "target/debug" / binary),
            "build_command": command, "toolchain": toolchain,
            "input_manifest_sha256": sha256((out / "input-manifest.json").read_bytes()),
            "exit_code": result.returncode,
        })
    write(out / "source.json", {"source_commit": source, "source_clean": True,
                               "source_snapshot": str(snapshot)})
    print(f"Identified GUI/CLI build recorded at {out}", flush=True)


if __name__ == "__main__":
    main()
