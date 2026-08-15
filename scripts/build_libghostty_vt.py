#!/usr/bin/env python3
"""Validate and reproduce Datum's pinned libghostty-vt dependency build."""

from __future__ import annotations

import argparse
import hashlib
import json
import os
from pathlib import Path
import platform
import shutil
import subprocess
import sys
import tarfile
import tempfile
import urllib.request


ROOT_KEYS = {
    "schema_version",
    "name",
    "upstream_repository",
    "source",
    "license",
    "toolchain",
    "build",
    "features",
    "compatibility_constraints",
}
SOURCE_KEYS = {
    "commit",
    "commit_date",
    "project_version",
    "library_version",
    "archive_url",
    "archive_sha256",
    "build_manifest_sha256",
    "public_header_sha256",
}
LICENSE_KEYS = {"spdx", "upstream_path", "upstream_sha256", "retained_notice"}
TOOLCHAIN_KEYS = {"name", "version", "host", "archive_url", "archive_sha256"}
BUILD_KEYS = {
    "target",
    "optimize",
    "features",
    "args",
    "required_outputs",
    "runtime_dependencies_linux",
}
EXPECTED_FEATURES = [
    "snapshot",
    "formatter",
    "selection",
    "render_state",
    "input_encode",
    "color",
    "grid_introspection",
    "glyph_protocol",
    "kitty_graphics",
]


class PinError(RuntimeError):
    """The checked-in pin or a fetched input violates the closed contract."""


def sha256(path: Path) -> str:
    digest = hashlib.sha256()
    with path.open("rb") as stream:
        for chunk in iter(lambda: stream.read(1024 * 1024), b""):
            digest.update(chunk)
    return digest.hexdigest()


def require_keys(value: object, expected: set[str], name: str) -> dict[str, object]:
    if not isinstance(value, dict):
        raise PinError(f"{name} must be an object")
    actual = set(value)
    if actual != expected:
        raise PinError(f"{name} keys differ: missing={sorted(expected-actual)} extra={sorted(actual-expected)}")
    return value


def require_string(value: object, name: str) -> str:
    if not isinstance(value, str) or not value.strip():
        raise PinError(f"{name} must be a nonempty string")
    return value


def validate_pin(root: Path, data: object) -> dict[str, object]:
    pin = require_keys(data, ROOT_KEYS, "lock")
    if pin["schema_version"] != 1 or pin["name"] != "libghostty-vt":
        raise PinError("unsupported libghostty-vt lock identity or schema")
    source = require_keys(pin["source"], SOURCE_KEYS, "source")
    license_data = require_keys(pin["license"], LICENSE_KEYS, "license")
    toolchain = require_keys(pin["toolchain"], TOOLCHAIN_KEYS, "toolchain")
    build = require_keys(pin["build"], BUILD_KEYS, "build")
    for section_name, section in (
        ("source", source),
        ("license", license_data),
        ("toolchain", toolchain),
        ("build", build),
    ):
        for key, value in section.items():
            if key not in {"args", "required_outputs", "runtime_dependencies_linux"}:
                require_string(value, f"{section_name}.{key}")
    for digest_key in ("archive_sha256", "build_manifest_sha256", "public_header_sha256"):
        digest = require_string(source[digest_key], f"source.{digest_key}")
        if len(digest) != 64 or any(char not in "0123456789abcdef" for char in digest):
            raise PinError(f"source.{digest_key} must be lowercase SHA-256")
    tool_digest = require_string(toolchain["archive_sha256"], "toolchain.archive_sha256")
    if len(tool_digest) != 64 or any(char not in "0123456789abcdef" for char in tool_digest):
        raise PinError("toolchain.archive_sha256 must be lowercase SHA-256")
    commit = require_string(source["commit"], "source.commit")
    if len(commit) != 40 or any(char not in "0123456789abcdef" for char in commit):
        raise PinError("source.commit must be a full lowercase Git object ID")
    expected_args = [
        "build",
        "install",
        "-Demit-lib-vt=true",
        f"-Doptimize={build['optimize']}",
        f"-Dtarget={build['target']}",
        f"-Dlib-version-string={source['library_version']}",
        f"-Dvt-features={build['features']}",
    ]
    if build["args"] != expected_args:
        raise PinError("build.args does not match the closed build properties")
    if pin["features"] != EXPECTED_FEATURES or build["features"] != "all":
        raise PinError("Datum requires the exact complete libghostty-vt feature inventory")
    for field in ("required_outputs", "runtime_dependencies_linux"):
        values = build[field]
        if not isinstance(values, list) or not values or not all(isinstance(v, str) and v for v in values):
            raise PinError(f"build.{field} must be a nonempty string list")
    constraints = pin["compatibility_constraints"]
    if not isinstance(constraints, list) or len(constraints) < 5 or not all(isinstance(v, str) and v for v in constraints):
        raise PinError("compatibility_constraints must retain the complete recorded envelope")
    notice = root / require_string(license_data["retained_notice"], "license.retained_notice")
    if not notice.is_file() or sha256(notice) != license_data["upstream_sha256"]:
        raise PinError("retained MIT notice is missing or differs from the pinned upstream license")
    return pin


def load_pin(root: Path) -> dict[str, object]:
    lock_path = root / "third_party/libghostty-vt/lock.json"
    try:
        data = json.loads(lock_path.read_text(encoding="utf-8"))
    except (OSError, json.JSONDecodeError) as exc:
        raise PinError(f"cannot read {lock_path}: {exc}") from exc
    return validate_pin(root, data)


def fetch(url: str, expected: str, destination: Path) -> None:
    destination.parent.mkdir(parents=True, exist_ok=True)
    if destination.is_file() and sha256(destination) == expected:
        print(f"verified cached download: {destination.name}", flush=True)
        return
    temporary = destination.with_suffix(destination.suffix + ".partial")
    print(f"downloading {url}", flush=True)
    with urllib.request.urlopen(url, timeout=120) as response, temporary.open("wb") as output:
        shutil.copyfileobj(response, output)
    actual = sha256(temporary)
    if actual != expected:
        temporary.unlink(missing_ok=True)
        raise PinError(f"checksum mismatch for {url}: expected {expected}, got {actual}")
    temporary.replace(destination)
    print(f"verified download: {destination.name}", flush=True)


def extract_fresh(archive: Path, destination: Path, strip_first: bool) -> None:
    temporary = destination.with_name(destination.name + f".extracting-{os.getpid()}")
    if temporary.exists():
        shutil.rmtree(temporary)
    temporary.mkdir(parents=True)
    with tarfile.open(archive) as bundle:
        bundle.extractall(temporary, filter="data")
    children = list(temporary.iterdir())
    if strip_first:
        if len(children) != 1 or not children[0].is_dir():
            raise PinError(f"{archive} does not contain one expected top-level directory")
        if destination.exists():
            shutil.rmtree(destination)
        children[0].replace(destination)
        temporary.rmdir()
    else:
        if destination.exists():
            shutil.rmtree(destination)
        temporary.replace(destination)


def host_name() -> str:
    machine = platform.machine().lower()
    machine = {"amd64": "x86_64"}.get(machine, machine)
    system = platform.system().lower()
    return f"{machine}-{system}"


def run(command: list[str], cwd: Path, env: dict[str, str] | None = None) -> None:
    print("+", " ".join(command), flush=True)
    completed = subprocess.run(
        command,
        cwd=cwd,
        env=env,
        check=False,
        timeout=1800,
        stdout=subprocess.PIPE,
        stderr=subprocess.STDOUT,
        text=True,
    )
    if completed.stdout:
        print(completed.stdout, end="" if completed.stdout.endswith("\n") else "\n")
    if completed.returncode != 0:
        raise subprocess.CalledProcessError(completed.returncode, command)


def smoke_test(prefix: Path, cache: Path) -> None:
    source = cache / "abi-smoke.c"
    binary = cache / "abi-smoke"
    source.write_text(
        """#include <stdbool.h>
#include <stdio.h>
#include <ghostty/vt.h>

int main(void) {
    GhosttyString version = {0};
    bool simd = false;
    bool kitty = false;
    if (ghostty_build_info(GHOSTTY_BUILD_INFO_VERSION_STRING, &version) != GHOSTTY_SUCCESS ||
        ghostty_build_info(GHOSTTY_BUILD_INFO_SIMD, &simd) != GHOSTTY_SUCCESS ||
        ghostty_build_info(GHOSTTY_BUILD_INFO_KITTY_GRAPHICS, &kitty) != GHOSTTY_SUCCESS) {
        return 1;
    }
    printf("version=%.*s simd=%d kitty_graphics=%d\\n",
           (int)version.len, version.ptr, simd, kitty);
    return 0;
}
""",
        encoding="utf-8",
    )
    run(
        [
            os.environ.get("CC", "cc"),
            "-std=c11",
            "-Wall",
            "-Wextra",
            "-Werror",
            f"-I{prefix / 'include'}",
            str(source),
            f"-L{prefix / 'lib'}",
            f"-Wl,-rpath,{prefix / 'lib'}",
            "-lghostty-vt",
            "-o",
            str(binary),
        ],
        cache,
    )
    run([str(binary)], cache)


def build(root: Path, pin: dict[str, object], cache: Path, prefix: Path) -> None:
    source_data = pin["source"]
    toolchain = pin["toolchain"]
    build_data = pin["build"]
    assert isinstance(source_data, dict) and isinstance(toolchain, dict) and isinstance(build_data, dict)
    if host_name() != toolchain["host"]:
        raise PinError(f"pin is verified for {toolchain['host']}, not {host_name()}; add a pinned host entry and proof")
    cache.mkdir(parents=True, exist_ok=True)
    downloads = cache / "downloads"
    source_archive = downloads / f"ghostty-{source_data['commit']}.tar.gz"
    zig_archive = downloads / f"zig-{toolchain['host']}-{toolchain['version']}.tar.xz"
    fetch(str(source_data["archive_url"]), str(source_data["archive_sha256"]), source_archive)
    fetch(str(toolchain["archive_url"]), str(toolchain["archive_sha256"]), zig_archive)
    source_dir = cache / f"ghostty-{source_data['commit']}"
    zig_dir = cache / f"zig-{toolchain['host']}-{toolchain['version']}"
    # Never trust a mutable prior extraction. The checksum-verified archives
    # are cheap to unpack and remain the source/toolchain authority.
    extract_fresh(source_archive, source_dir, strip_first=True)
    extract_fresh(zig_archive, zig_dir, strip_first=True)
    if sha256(source_dir / "build.zig.zon") != source_data["build_manifest_sha256"]:
        raise PinError("extracted Ghostty build.zig.zon differs from the lock")
    if sha256(source_dir / "include/ghostty/vt.h") != source_data["public_header_sha256"]:
        raise PinError("extracted Ghostty public C header differs from the lock")
    if sha256(source_dir / "LICENSE") != pin["license"]["upstream_sha256"]:
        raise PinError("extracted Ghostty license differs from the retained notice")
    zig = zig_dir / "zig"
    env = dict(os.environ)
    env["ZIG_GLOBAL_CACHE_DIR"] = str(cache / "zig-global-cache")
    if prefix.exists():
        forbidden = {Path("/"), Path("/tmp"), Path.home().resolve(), root, root.parent}
        if prefix in forbidden or len(prefix.parts) < 3:
            raise PinError(f"refusing to replace unsafe install prefix: {prefix}")
        shutil.rmtree(prefix)
    # A single build job keeps the reproducibility probe usable on constrained
    # CI/agent workers; it does not alter the produced library configuration.
    command = [
        str(zig),
        *build_data["args"],
        "-j1",
        "--prefix",
        str(prefix),
        "--cache-dir",
        str(cache / "zig-cache"),
    ]
    run(command, source_dir, env)
    missing = [path for path in build_data["required_outputs"] if not (prefix / path).is_file()]
    if missing:
        raise PinError(f"build succeeded without required outputs: {', '.join(missing)}")
    smoke_test(prefix, cache)
    print(f"libghostty-vt pin verified and built at {prefix}")


def main(argv: list[str] | None = None) -> int:
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("command", choices=("check", "build"))
    parser.add_argument("--root", type=Path, default=Path(__file__).resolve().parents[1])
    parser.add_argument("--cache", type=Path)
    parser.add_argument("--prefix", type=Path)
    args = parser.parse_args(argv)
    root = args.root.resolve()
    try:
        pin = load_pin(root)
        if args.command == "check":
            print("libghostty-vt dependency pin passed")
        else:
            cache = (args.cache or root / "target/libghostty-vt/cache").resolve()
            prefix = (args.prefix or root / "target/libghostty-vt/install").resolve()
            build(root, pin, cache, prefix)
    except (PinError, OSError, subprocess.CalledProcessError) as exc:
        print(f"libghostty-vt dependency error: {exc}", file=sys.stderr)
        return 1
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
