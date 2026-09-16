"""Bind evidence to the actual committed and current product input closure."""

from pathlib import Path
import os
import subprocess

from .inputs import array, closed, digest, parse, read_file, relative, require, revision, sha, text, unique


ROOTS = ("Cargo.toml", "Cargo.lock", ".cargo", "crates", "mcp-server", "scripts",
         "CLAUDE.md", "AGENTS.md", "specs/evidence_traceability_manifest.json")
ROUTE = "workspace-documentation-and-revision"


def git(root, *arguments):
    require(not any(k.startswith("GIT_") and k not in {"GIT_OPTIONAL_LOCKS", "GIT_TERMINAL_PROMPT", "GIT_PAGER"}
                    for k in os.environ), "Git environment overrides are not allowed during proof")
    process = subprocess.run(["git", "--no-pager", "--no-replace-objects", "--no-optional-locks", *arguments],
                             cwd=root, capture_output=True, check=False)
    require(process.returncode == 0, "cannot inspect candidate Git identity: " +
            process.stderr.decode("utf-8", "replace").strip())
    return process.stdout


def required_paths(root, candidate):
    manifest = parse(git(root, "show", candidate + ":specs/evidence_traceability_manifest.json"),
                     "candidate traceability manifest")
    routes = [r for r in manifest["routes"] if r["id"] == ROUTE]
    require(len(routes) == 1, "Preferences owning route missing or duplicated")
    roots = (*ROOTS, *routes[0]["sources"], *routes[0]["consumers"])
    files = git(root, "ls-tree", "-r", "--name-only", "-z", candidate).decode().split("\0")
    matches = lambda p: any(p == r or p.startswith(r + "/") for r in roots)
    return roots, sorted(p for p in files if p and matches(p))


def validate_candidate(root, candidate, input_reference, bundle, expected_revision):
    closed(candidate, "revision tree cargo_lock_sha256 toolchains build_commands binaries", "candidate")
    selected = revision(candidate["revision"])
    require(selected == revision(expected_revision), "evidence is for a different requested candidate")
    actual_tree = git(root, "rev-parse", selected + "^{tree}").decode().strip()
    require(candidate["tree"] == actual_tree, "candidate tree differs")
    require(candidate["cargo_lock_sha256"] == digest(git(root, "show", selected + ":Cargo.lock")),
            "candidate dependency lock differs")
    closed(candidate["toolchains"], "rustc cargo python", "toolchains")
    for name, value in candidate["toolchains"].items():
        text(value, name)
    from .gates import release_build
    from .observations import command_observation
    builds = array(candidate["build_commands"], "build commands")
    require(len(builds) == 1, "one complete release build observation required")
    build = builds[0]
    closed(build, "command candidate_revision input_manifest_sha256 exit_code log", "release build")
    require(build["command"] == release_build() and build["candidate_revision"] == selected and
            build["input_manifest_sha256"] == input_reference["sha256"], "release build identity differs")
    command_observation(build, bundle)
    binaries = candidate["binaries"]
    require(type(binaries) is dict and set(binaries) == {"cli", "gui", "daemon", "engine", "gui-render", "mcp"},
            "exact CLI, GUI, daemon, engine-test, renderer-test and MCP interpreter identities required")
    for name, reference in binaries.items():
        text(name, "binary identity")
        bundle.read(reference)
    entries = array(bundle.json(input_reference), "input manifest")
    paths = []
    roots, required = required_paths(root, selected)
    modes = dict(line.split(" ", 1)[::-1] for line in
                 git(root, "ls-tree", "-r", "--format=%(objectmode) %(path)", selected).decode().splitlines())
    for entry in entries:
        closed(entry, "path sha256", "input manifest entry")
        name = str(relative(entry["path"]))
        paths.append(name)
        expected = sha(entry["sha256"], name)
        require(digest(git(root, "show", selected + ":" + name)) == expected,
                "candidate input digest differs: " + name)
        require(digest(read_file(root, name)) == expected, "current proof input changed: " + name)
        require(modes.get(name) in {"100644", "100755"}, "unsupported proof input type: " + name)
        require(bool((Path(root) / name).stat().st_mode & 0o111) == (modes[name] == "100755"),
                "current proof input executable mode changed: " + name)
    require(paths == required, "input manifest must cover the complete sorted product/route closure")
    current = git(root, "ls-files", "--cached", "--others", "--exclude-standard", "-z", "--", *roots)
    require(sorted(set(p for p in current.decode().split("\0") if p)) == required,
            "new or removed proof input requires recapture")
    return {role: ref["sha256"] for role, ref in binaries.items()}


def command_line(value):
    for argument in array(value, "command argv"):
        text(argument, "command argument")
    return value


def validate_environments(values):
    result = {}
    fields = ("id os architecture kernel cpu memory_mib filesystem mount_options storage_device "
              "gpu driver display_backend compositor_or_server scale_factor window_size "
              "locale network_state power_mode background_load clock tool_versions")
    for environment in array(values, "environments"):
        closed(environment, fields, "environment")
        identifier = text(environment["id"], "environment id")
        require(identifier not in result, "duplicate environment")
        require(environment["os"] == "Linux" and environment["architecture"] == "x86_64",
                "environment outside declared platform")
        from .inputs import integer
        integer(environment["memory_mib"], "memory MiB", 1)
        for name in ("kernel", "cpu", "filesystem", "mount_options", "storage_device", "locale",
                     "network_state", "power_mode", "background_load", "clock"):
            text(environment[name], name)
            require(environment[name].lower() not in {"unknown", "unavailable"},
                    "required environment fact unavailable: " + name)
        backend = environment["display_backend"]
        if backend is None:
            require(all(environment[n] is None for n in
                        ("gpu", "driver", "compositor_or_server", "scale_factor", "window_size")),
                    "headless environment must not invent native facts")
            coordinate = None
        else:
            from .inventory import BACKENDS, SCALES, WINDOWS
            require(backend in BACKENDS and type(environment["scale_factor"]) in (int, float)
                    and environment["scale_factor"] in SCALES, "unsupported GUI backend/scale")
            size = environment["window_size"]
            require(type(size) is list and all(type(n) is int for n in size)
                    and tuple(size) in WINDOWS, "actual logical window dimensions required")
            for name in ("gpu", "driver", "compositor_or_server"):
                text(environment[name], name)
            coordinate = (backend, environment["scale_factor"], tuple(size))
        versions = environment["tool_versions"]
        require(type(versions) is dict and bool(versions), "observed tool versions required")
        for name, version in versions.items():
            text(name, "tool name")
            text(version, "tool version")
        result[identifier] = coordinate
    # Do not let a second, faster environment silently replace a failing trial.
    unique(list(result.values()), "environment coverage coordinates")
    return result
