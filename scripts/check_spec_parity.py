#!/usr/bin/env python3
"""Check compact spec inventories against implemented code surfaces."""

from __future__ import annotations

import argparse
import hashlib
import json
from pathlib import Path
import re
import sys


ROOT = Path(__file__).resolve().parents[1]
MANIFEST = ROOT / "specs/spec_parity_manifest.json"
PARITY_DOC = ROOT / "specs/SPEC_PARITY.md"


def read_text(path: Path) -> str:
    return path.read_text(encoding="utf-8")


def digest_items(items: list[str]) -> str:
    payload = "\n".join(sorted(items)) + "\n"
    return hashlib.sha256(payload.encode("utf-8")).hexdigest()


def parse_daemon_methods() -> set[str]:
    text = read_text(ROOT / "crates/engine-daemon/src/dispatch.rs")
    return set(re.findall(r'^\s*"([a-z_]+)"\s*=>', text, flags=re.M))


def parse_tool_catalog_methods() -> set[str]:
    text = read_text(ROOT / "mcp-server/tools_catalog_data.py")
    return set(re.findall(r'"name":\s*"([a-z_]+)"', text))


def parse_cli_bridge_methods(tool_methods: set[str]) -> set[str]:
    public_methods: set[str] = set()
    for path in sorted((ROOT / "mcp-server").glob("server_runtime*.py")):
        public_methods.update(re.findall(r"^\s{4}def ([a-z_]+)\(", read_text(path), flags=re.M))
    return public_methods & tool_methods


def mcp_runtime_methods() -> list[str]:
    tool_methods = parse_tool_catalog_methods()
    runtime_methods = parse_daemon_methods() | parse_cli_bridge_methods(tool_methods)
    return sorted(runtime_methods)


def enum_body(text: str, enum_name: str) -> str:
    match = re.search(rf"\benum\s+{re.escape(enum_name)}\s*\{{", text)
    if not match:
        raise ValueError(f"unable to find enum {enum_name}")
    start = match.end()
    depth = 1
    for index in range(start, len(text)):
        char = text[index]
        if char == "{":
            depth += 1
        elif char == "}":
            depth -= 1
            if depth == 0:
                return text[start:index]
    raise ValueError(f"unterminated enum {enum_name}")


def rust_enum_variants(path: Path, enum_name: str) -> list[str]:
    body = enum_body(read_text(path), enum_name)
    variants = re.findall(r"^\s{4}([A-Z][A-Za-z0-9]+)(?:\s*\(|\s*,|\s*$)", body, flags=re.M)
    return sorted(set(variants))


def file_glob(pattern: str) -> list[str]:
    return sorted(path.name for path in ROOT.glob(pattern))


def cargo_workspace_members(path: Path) -> list[str]:
    text = read_text(path)
    members_match = re.search(r"^\s*members\s*=\s*\[(.*?)\]", text, flags=re.S | re.M)
    if not members_match:
        raise ValueError(f"no workspace.members in {path}")
    return sorted(re.findall(r'"crates/([\w-]+)"', members_match.group(1)))


def daemon_dispatch_methods() -> list[str]:
    return sorted(parse_daemon_methods())


def engine_api_pub_fns() -> list[str]:
    api_root = ROOT / "crates/engine/src/api"
    names: set[str] = set()
    for path in api_root.rglob("*.rs"):
        if "tests" in path.parts or path.name.endswith("_tests.rs"):
            continue
        text = read_text(path)
        for match in re.finditer(r"^\s*pub fn ([a-z_][a-z0-9_]*)", text, flags=re.M):
            names.add(match.group(1))
    return sorted(names)


def inventory_items(spec: dict[str, str]) -> list[str]:
    kind = spec["kind"]
    if kind == "mcp_runtime_methods":
        return mcp_runtime_methods()
    if kind == "rust_enum_variants":
        return rust_enum_variants(ROOT / spec["path"], spec["enum"])
    if kind == "file_glob":
        return file_glob(spec["glob"])
    if kind == "cargo_workspace_members":
        return cargo_workspace_members(ROOT / spec["path"])
    if kind == "daemon_dispatch_methods":
        return daemon_dispatch_methods()
    if kind == "engine_api_pub_fns":
        return engine_api_pub_fns()
    raise ValueError(f"unknown inventory kind: {kind}")


def load_manifest() -> dict:
    return json.loads(read_text(MANIFEST))


def expected_rows() -> dict[str, tuple[str, int, str]]:
    rows: dict[str, tuple[str, int, str]] = {}
    for spec in load_manifest()["inventories"]:
        items = inventory_items(spec)
        rows[spec["id"]] = (spec["owner_spec"], len(items), digest_items(items))
    return rows


def parse_doc_rows() -> dict[str, tuple[str, int, str]]:
    rows: dict[str, tuple[str, int, str]] = {}
    pattern = re.compile(
        r"^\|\s*`([^`]+)`\s*\|\s*`([^`]+)`\s*\|\s*(\d+)\s*\|\s*`([0-9a-f]+|pending)`\s*\|$",
        flags=re.M,
    )
    for match in pattern.finditer(read_text(PARITY_DOC)):
        rows[match.group(1)] = (match.group(2), int(match.group(3)), match.group(4))
    return rows


def update_doc() -> None:
    text = read_text(PARITY_DOC)
    rows = expected_rows()
    for inventory_id, (owner_spec, count, digest) in rows.items():
        replacement = f"| `{inventory_id}` | `{owner_spec}` | {count} | `{digest}` |"
        pattern = re.compile(
            rf"^\|\s*`{re.escape(inventory_id)}`\s*\|[^\n]*$",
            flags=re.M,
        )
        text, replacements = pattern.subn(replacement, text)
        if replacements != 1:
            raise RuntimeError(f"unable to update row for {inventory_id}")
    PARITY_DOC.write_text(text, encoding="utf-8")


def print_inventory() -> None:
    for spec in load_manifest()["inventories"]:
        items = inventory_items(spec)
        print(f"[{spec['id']}] count={len(items)} sha256={digest_items(items)}")
        for item in items:
            print(f"  {item}")


def check() -> int:
    expected = expected_rows()
    actual = parse_doc_rows()
    failures: list[str] = []

    for inventory_id, expected_row in expected.items():
        actual_row = actual.get(inventory_id)
        if actual_row is None:
            failures.append(f"missing SPEC_PARITY row for {inventory_id}")
            continue
        if actual_row != expected_row:
            failures.append(
                f"{inventory_id}: doc row {actual_row} != code inventory {expected_row}"
            )

    extra = sorted(set(actual) - set(expected))
    for inventory_id in extra:
        failures.append(f"stale SPEC_PARITY row for {inventory_id}")

    if failures:
        print("Spec parity check failed:", file=sys.stderr)
        for failure in failures:
            print(f"  - {failure}", file=sys.stderr)
        print(
            "Run `python3 scripts/check_spec_parity.py --update` after updating the owning spec.",
            file=sys.stderr,
        )
        return 1

    print(f"Spec parity check passed ({len(expected)} inventories).")
    return 0


def main() -> int:
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--update", action="store_true", help="refresh specs/SPEC_PARITY.md")
    parser.add_argument("--print", action="store_true", help="print inventory item names")
    args = parser.parse_args()

    if args.print:
        print_inventory()
        return 0
    if args.update:
        update_doc()
        return 0
    return check()


if __name__ == "__main__":
    raise SystemExit(main())
