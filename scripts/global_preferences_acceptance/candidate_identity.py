"""GP-CM05E Phase-A U2: the §11 candidate-identity builder.

Builds the `candidate` object and the `input_manifest` that validate_candidate (identity.py)
requires, bound to the pinned revision: revision/tree/cargo_lock_sha256, the observed
toolchains, the ONE guarded release-build observation (gates.release_build()), the six role
binaries (cli/gui/daemon release exes, engine/gui-render test bins, mcp interpreter) retained in
the bundle, and the sorted product/route input-manifest closure (ROOTS + the owning ETM route's
sources/consumers, each entry's sha256 == the committed bytes).

Split by testability: the deterministic identity + manifest closure (revision_identity,
input_manifest) are exercised now against identity.required_paths at HEAD; the release build and
binary retention are the capture-time orchestration (build_candidate), fully validated by
validate_candidate at the real capture on the re-pinned, clean, orchestrator-inclusive candidate.
No readiness is issued here; this is one evidence input.
"""

import json
import subprocess
from pathlib import Path

from .identity import git, required_paths, validate_live_inputs
from .inputs import canonical, digest, require

ROOT = Path(__file__).resolve().parents[2]
ROLES = ("cli", "gui", "daemon", "engine", "gui-render", "mcp")
# release-profile distributed exes built by gates.release_build().
RELEASE_BINARY_PATHS = {
    "cli": "target/release/datum-eda",
    "gui": "target/release/datum-gui",
    "daemon": "target/release/eda-engine-daemon",
}
# The engine role is the feature-ON acceptance driver [[bin]] -- gp_acceptance_driver, the
# injection-harness binary that ACTUALLY executes the engine-surface cases (contract section 11:
# "engine ... are the executed test binaries" + the surface-disposition injection harness;
# crates/engine/src/bin/gp_acceptance_driver.rs: "the engine surface the production acceptance corpus
# pins as candidate.binaries[engine]"; cb67dad4 feature-gating; the recorded PM engine-role
# disposition). It is built separately (gates.acceptance_driver_build) like the test bins and
# retained by its bytes; it is NOT a candidate build_commands entry.
ACCEPTANCE_DRIVER_PATH = "target/release/gp_acceptance_driver"
# test-bin crates whose executed test binary is a role identity (located via cargo metadata JSON).
TEST_BINARY_CRATES = {"gui-render": "datum-gui-render"}


def _tool_version(argv):
    try:
        out = subprocess.run(argv, capture_output=True, text=True, check=False).stdout.strip()
    except OSError:
        out = ""
    require(bool(out), f"required toolchain version unavailable: {argv[0]}")
    return out


def revision_identity(root, revision):
    """The deterministic candidate identity bound to `revision` (no build): revision, tree,
    Cargo.lock digest and the observed rustc/cargo/python versions."""
    selected = git(root, "rev-parse", "--verify", revision + "^{commit}").decode().strip()
    tree = git(root, "rev-parse", selected + "^{tree}").decode().strip()
    cargo_lock_sha256 = digest(git(root, "show", selected + ":Cargo.lock"))
    toolchains = {
        "rustc": _tool_version(["rustc", "--version"]),
        "cargo": _tool_version(["cargo", "--version"]),
        "python": _tool_version(["python3", "--version"]),
    }
    return {"revision": selected, "tree": tree, "cargo_lock_sha256": cargo_lock_sha256,
            "toolchains": toolchains}


def input_manifest(root, capture, revision):
    """Retain the sorted product/route input-manifest closure (ROOTS + the owning ETM route's
    sources/consumers at `revision`); each entry's sha256 is the COMMITTED blob digest. Returns the
    bundle reference whose sha256 is the input_manifest_sha256 that binds every observation."""
    selected = git(root, "rev-parse", "--verify", revision + "^{commit}").decode().strip()
    _roots, required = required_paths(root, selected)
    entries = [{"path": path, "sha256": digest(git(root, "show", selected + ":" + path))}
               for path in required]
    return capture.put(entries)


def release_build_observation(capture, root, revision, input_manifest_sha256):
    """Run the ONE guarded release build (gates.release_build()) and retain its command-observation;
    the returned entry matches validate_candidate's build-command shape. Exit code is preserved as
    evidence (a failed build is reported, never relabelled)."""
    from .gates import release_build
    selected = git(root, "rev-parse", "--verify", revision + "^{commit}").decode().strip()
    return capture.command(release_build(), cwd=root, candidate=selected,
                           inputs=input_manifest_sha256)


def _test_binary_paths(root):
    """Locate the executed engine / gui-render test binaries via `cargo test --no-run` JSON; their
    exact executables are the candidate's engine/gui-render role identities."""
    argv = ["python3", "scripts/run_cargo_guarded.py", "--workload", "proof", "--",
            "cargo", "test", "--no-run", "--locked", "--offline", "--message-format=json",
            "-p", "datum-gui-render"]
    result = subprocess.run(argv, cwd=root, capture_output=True, text=True, check=False)
    require(result.returncode == 0, "cargo test --no-run failed locating test binaries")
    located = {}
    for line in result.stdout.splitlines():
        try:
            message = json.loads(line)
        except json.JSONDecodeError:
            continue
        target = message.get("target") or {}
        name = target.get("name")
        executable = message.get("executable")
        # Cargo reports the lib test target's name in crate-identifier form (hyphens -> underscores),
        # e.g. package `eda-engine` -> target name `eda_engine`; match that normalized lib artifact.
        if not executable or "lib" not in (target.get("kind") or []):
            continue
        for role, crate in TEST_BINARY_CRATES.items():
            if name == crate.replace("-", "_"):
                located[role] = executable
    require(set(located) == set(TEST_BINARY_CRATES),
            f"missing test binaries: {set(TEST_BINARY_CRATES) - set(located)}")
    return located


def _build_acceptance_driver(root):
    """Build the feature-ON engine acceptance driver (gp_acceptance_driver) outside the recorded
    release build, as the test bins are built outside it. A failed build is reported, never ignored."""
    from .gates import acceptance_driver_build
    result = subprocess.run(acceptance_driver_build(), cwd=root, capture_output=True, text=True,
                            check=False)
    require(result.returncode == 0,
            f"engine acceptance driver build failed (exit {result.returncode}): "
            f"{result.stderr[-800:]}")


def retain_binaries(capture, root):
    """Retain the six role binaries in the bundle and return {role: {path, sha256}}. The first three
    release exes come from gates.release_build(); the engine role is the feature-ON acceptance driver
    (gp_acceptance_driver, built separately -- the injection-harness binary that executes the engine
    cases, pinned as candidate.binaries["engine"]); gui-render is the located test bin; mcp is the
    Python interpreter that runs the candidate MCP source (itself in the input closure)."""
    import sys
    binaries = {}
    for role, relative in RELEASE_BINARY_PATHS.items():
        path = Path(root) / relative
        require(path.is_file(), f"release binary missing (build first): {relative}")
        binaries[role] = capture.put(path.read_bytes())
    _build_acceptance_driver(root)
    driver = Path(root) / ACCEPTANCE_DRIVER_PATH
    require(driver.is_file(), f"engine acceptance driver missing (build first): {ACCEPTANCE_DRIVER_PATH}")
    binaries["engine"] = capture.put(driver.read_bytes())
    for role, executable in _test_binary_paths(root).items():
        binaries[role] = capture.put(Path(executable).read_bytes())
    interpreter = Path(sys.executable).resolve()
    require(interpreter.is_file(), "python interpreter path not a file")
    binaries["mcp"] = capture.put(interpreter.read_bytes())
    require(set(binaries) == set(ROLES), "exactly the six role binaries are required")
    return binaries


def build_candidate(root, capture, revision):
    """Capture-time orchestration: assemble the full candidate object. Runs the guarded release
    build, locates+retains the six role binaries, computes the input manifest, and returns
    (candidate, input_manifest_ref). Fully validated by validate_candidate at the real capture."""
    selected = git(root, "rev-parse", "--verify", revision + "^{commit}").decode().strip()
    validate_live_inputs(root, selected)
    manifest_ref = input_manifest(root, capture, selected)
    build = release_build_observation(capture, root, selected, manifest_ref["sha256"])
    binaries = retain_binaries(capture, root)
    validate_live_inputs(root, selected)
    identity = revision_identity(root, selected)
    candidate = {**identity, "build_commands": [build], "binaries": binaries}
    return candidate, manifest_ref


def input_manifest_sha256(root, revision):
    """The input_manifest_sha256 without retaining (digest over the canonical closure entries) --
    used to pre-bind observations before the bundle is assembled."""
    selected = git(root, "rev-parse", "--verify", revision + "^{commit}").decode().strip()
    _roots, required = required_paths(root, selected)
    entries = [{"path": path, "sha256": digest(git(root, "show", selected + ":" + path))}
               for path in required]
    return digest(canonical(entries))
