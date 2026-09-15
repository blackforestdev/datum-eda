#!/usr/bin/env python3
"""Enforce Datum's repository-wide source-health policy.

The policy is deliberately dependency-free so the same program runs in a clean
checkout and in CI.  The working-tree scan includes tracked and untracked
(non-ignored) files.  With ``--base-ref`` it additionally enforces the
downward-only character of policy and legacy ceilings and requires touched
legacy monoliths to become smaller.
"""

from __future__ import annotations

import argparse
import ast
import json
import re
import subprocess
import sys
from dataclasses import dataclass
from pathlib import Path, PurePosixPath
from typing import Any


EXTENSIONS = {
    ".rs", ".py", ".sh", ".bash", ".js", ".jsx", ".ts", ".tsx",
    ".html", ".css", ".c", ".h", ".cc", ".cpp", ".cxx", ".hh", ".hpp",
}
DEFAULT_THRESHOLDS = {
    "production": 700,
    "test": 700,
    "inline_test": 350,
    "shell": 400,
    "web": 700,
}
INCLUDE_RE = re.compile(r'\binclude!\s*\(\s*"([^"\n]+)"\s*\)')
CFG_TEST_RE = re.compile(r"^\s*#\s*\[\s*cfg\s*\(\s*test\s*\)\s*\]", re.MULTILINE)
TEST_NAMES = re.compile(r"(^|[_.-])(test|tests|spec)([_.-]|$)", re.IGNORECASE)


class PolicyError(ValueError):
    pass


@dataclass(frozen=True)
class Measurement:
    physical: int
    production: int
    inline_test: int
    logical: int
    category: str


def run_git(root: Path, args: list[str], *, text: bool = True) -> str | bytes:
    proc = subprocess.run(
        ["git", *args], cwd=root, stdout=subprocess.PIPE, stderr=subprocess.PIPE,
        text=text, check=False,
    )
    if proc.returncode:
        detail = proc.stderr.strip() if text else proc.stderr.decode(errors="replace").strip()
        raise PolicyError(f"git {' '.join(args)} failed: {detail}")
    return proc.stdout


def count_lines(data: bytes) -> int:
    """Count logical text lines, including a final unterminated line."""
    return 0 if not data else data.count(b"\n") + (not data.endswith(b"\n"))


def canonical_rel(value: str) -> str:
    if not value or "\\" in value:
        raise PolicyError(f"non-canonical path: {value!r}")
    path = PurePosixPath(value)
    if path.is_absolute() or ".." in path.parts or str(path) != value:
        raise PolicyError(f"non-canonical path: {value!r}")
    return value


def require_int(obj: dict[str, Any], key: str, minimum: int = 0) -> int:
    value = obj.get(key)
    if isinstance(value, bool) or not isinstance(value, int) or value < minimum:
        raise PolicyError(f"{key} must be an integer >= {minimum}")
    return value


def load_policy_bytes(data: bytes, source: str) -> dict[str, Any]:
    try:
        policy = json.loads(data.decode("utf-8"))
    except (UnicodeDecodeError, json.JSONDecodeError) as exc:
        raise PolicyError(f"invalid policy {source}: {exc}") from exc
    if not isinstance(policy, dict):
        raise PolicyError("policy root must be an object")
    allowed = {"schema_version", "policy_decision", "baseline_commit", "description", "entries", "exclusions", "exceptions"}
    unknown = sorted(set(policy) - allowed)
    if unknown:
        raise PolicyError(f"unknown policy keys: {', '.join(unknown)}")
    if policy.get("schema_version") != 2:
        raise PolicyError("schema_version must be 2")
    if policy.get("policy_decision") != "PRODUCT_MECHANICS_022":
        raise PolicyError("policy_decision must be PRODUCT_MECHANICS_022")
    if not isinstance(policy.get("baseline_commit"), str) or not policy["baseline_commit"].strip():
        raise PolicyError("baseline_commit must be a non-empty git revision")
    exclusions = policy.get("exclusions")
    if not isinstance(exclusions, dict) or set(exclusions) != {"generated", "vendored", "data_assets"}:
        raise PolicyError("exclusions must contain generated, vendored, and data_assets")
    excluded: list[str] = []
    for category, detail in exclusions.items():
        if not isinstance(detail, dict) or set(detail) != {"paths", "requirements"}:
            raise PolicyError(f"exclusions.{category} must contain paths and requirements")
        if not isinstance(detail["paths"], list) or any(not isinstance(x, str) for x in detail["paths"]):
            raise PolicyError(f"exclusions.{category}.paths must be a list")
        if not isinstance(detail["requirements"], str) or not detail["requirements"].strip():
            raise PolicyError(f"exclusions.{category}.requirements must be non-empty")
        excluded.extend(detail["paths"])
    for prefix in excluded:
        canonical_rel(prefix.rstrip("/"))
        if not prefix.endswith("/"):
            raise PolicyError(f"excluded path must end in '/': {prefix}")
    entries = policy.get("entries")
    if not isinstance(entries, dict):
        raise PolicyError("entries must be an object")
    allowed_entry = {"kind", "limits", "status", "note", "owner", "trigger_commit", "target_lines"}
    for rel, entry in entries.items():
        canonical_rel(rel)
        if not isinstance(entry, dict):
            raise PolicyError(f"entries[{rel}] must be an object")
        extra = sorted(set(entry) - allowed_entry)
        if extra:
            raise PolicyError(f"entries[{rel}] unknown keys: {', '.join(extra)}")
        if entry.get("kind") not in {"rust-production", "rust-test", "python-production", "python-test", "shell", "web", "native-production", "native-test"}:
            raise PolicyError(f"entries[{rel}].kind is invalid")
        limits = entry.get("limits")
        if not isinstance(limits, dict) or not limits:
            raise PolicyError(f"entries[{rel}].limits must be a non-empty object")
        if set(limits) - {"file_lines", "pre_test_lines", "inline_test_lines", "expanded_lines"}:
            raise PolicyError(f"entries[{rel}].limits has unknown metrics")
        for metric in limits:
            require_int(limits, metric, 1)
        target = require_int(entry, "target_lines", 1)
        if target > 700:
            raise PolicyError(f"entries[{rel}].target_lines must be <= 700")
        if entry.get("status") != "decomposition-required":
            raise PolicyError(f"entries[{rel}] status must be decomposition-required")
        for key in ("note", "owner", "trigger_commit"):
            if not isinstance(entry.get(key), str) or not entry[key].strip():
                raise PolicyError(f"entries[{rel}].{key} must be a non-empty string")
    exceptions = policy.get("exceptions", [])
    if not isinstance(exceptions, list) or exceptions:
        raise PolicyError("schema v2 does not permit standing exceptions")
    return policy


def load_policy(path: Path) -> dict[str, Any]:
    try:
        return load_policy_bytes(path.read_bytes(), str(path))
    except OSError as exc:
        raise PolicyError(f"cannot read policy {path}: {exc}") from exc


def repository_contract_failures(root: Path) -> list[str]:
    """Verify the active doctrine remains classified, owned, and CI-wired."""
    decision = root / "docs/decisions/PRODUCT_MECHANICS_022_SOURCE_HEALTH_GOVERNANCE.md"
    if not decision.exists():
        # Hermetic fixture repositories exercise the engine without Datum's
        # governance-document surface.
        return []
    failures: list[str] = []
    required = (
        "docs/decisions/PRODUCT_MECHANICS_022_SOURCE_HEALTH_GOVERNANCE.md",
        "docs/SOURCE_HEALTH_POLICY.md",
        "scripts/check_source_health.py",
        "scripts/test_source_health_governance.py",
        "scripts/check_source_module_size.py",
        "specs/source_module_size_manifest.json",
        "scripts/run_drift_gates.sh",
        ".github/workflows/alignment.yml",
        ".github/CODEOWNERS",
        "specs/spec_governance_manifest.json",
    )
    for rel in required:
        if not (root / rel).is_file():
            failures.append(f"protected source-health surface missing: {rel}")
    drift_path = root / "scripts/run_drift_gates.sh"
    if drift_path.is_file():
        drift = drift_path.read_text(encoding="utf-8")
        for command in (
            "python3 scripts/test_source_health_governance.py",
            "python3 scripts/check_source_health.py",
        ):
            if command not in drift:
                failures.append(f"required source-health gate is not wired: {command}")
    governance_path = root / "specs/spec_governance_manifest.json"
    if governance_path.is_file():
        governance = json.loads(governance_path.read_text(encoding="utf-8"))
        decision_entry = governance.get("entries", {}).get(
            "docs/decisions/PRODUCT_MECHANICS_022_SOURCE_HEALTH_GOVERNANCE.md", {}
        )
        if decision_entry.get("class") != "doctrine" or not decision_entry.get("controlling"):
            failures.append("decision 022 must remain controlling doctrine")
    codeowners_path = root / ".github/CODEOWNERS"
    if codeowners_path.is_file():
        codeowners = codeowners_path.read_text(encoding="utf-8")
        for rel in required:
            if f"/{rel} " not in codeowners:
                failures.append(f"protected source-health surface lacks CODEOWNERS: {rel}")
    return failures


def assignment_literal(data: bytes, name: str) -> Any | None:
    try:
        tree = ast.parse(data.decode("utf-8"))
    except (UnicodeDecodeError, SyntaxError):
        return None
    for node in tree.body:
        if isinstance(node, ast.Assign):
            if any(isinstance(target, ast.Name) and target.id == name for target in node.targets):
                try:
                    return ast.literal_eval(node.value)
                except (ValueError, TypeError):
                    return None
    return None


def discover_sources(root: Path, policy: dict[str, Any]) -> list[str]:
    # -c: cached/tracked, -o: untracked, --exclude-standard: honor ignore rules.
    raw = run_git(root, ["ls-files", "-z", "-co", "--exclude-standard"], text=False)
    assert isinstance(raw, bytes)
    extensions = EXTENSIONS
    policy_excludes = [path for detail in policy["exclusions"].values() for path in detail["paths"]]
    # Git already omits ignored files. Authored source never disappears merely
    # because it sits in a conventionally named directory; decision 022
    # requires every additional exclusion to be explicit in the ledger.
    excludes = tuple(policy_excludes)
    found: set[str] = set()
    for item in raw.split(b"\0"):
        if not item:
            continue
        rel = item.decode("utf-8", errors="surrogateescape")
        path = root / rel
        if not path.is_file() or path.is_symlink() or rel.startswith(excludes):
            continue
        if PurePosixPath(rel).suffix.lower() in extensions:
            found.add(rel)
    return sorted(found)


def is_test_path(rel: str) -> bool:
    path = PurePosixPath(rel)
    return "tests" in {p.lower() for p in path.parts} or bool(TEST_NAMES.search(path.name))


def split_rust(data: bytes) -> tuple[int, int]:
    text = data.decode("utf-8", errors="replace")
    match = CFG_TEST_RE.search(text)
    if not match:
        return count_lines(data), 0
    before = text[: match.start()].encode("utf-8")
    # The attribute belongs to the boundary, not to either authored body.
    boundary_end = text.find("\n", match.end())
    tail = text[boundary_end + 1 :].encode("utf-8") if boundary_end >= 0 else b""
    return count_lines(before), count_lines(tail)


def include_targets(root: Path, rel: str, data: bytes) -> list[str]:
    text = data.decode("utf-8", errors="replace")
    targets: list[str] = []
    for raw in INCLUDE_RE.findall(text):
        target = (PurePosixPath(rel).parent / raw)
        normalized = target.as_posix()
        canonical_rel(normalized)
        if not (root / normalized).is_file():
            raise PolicyError(f"{rel}: literal include target does not exist: {normalized}")
        targets.append(normalized)
    # Dynamic include paths conceal logical size and need an architectural fix.
    if "include!(" in text and len(targets) != text.count("include!("):
        raise PolicyError(f"{rel}: dynamic or unsupported include! expression")
    return targets


def logical_lines(root: Path, rel: str, stack: tuple[str, ...] = ()) -> int:
    if rel in stack:
        raise PolicyError(f"include! cycle: {' -> '.join((*stack, rel))}")
    data = (root / rel).read_bytes()
    total = count_lines(data)
    for target in include_targets(root, rel, data):
        total += logical_lines(root, target, (*stack, rel))
    return total


def measure(root: Path, rel: str) -> Measurement:
    data = (root / rel).read_bytes()
    suffix = PurePosixPath(rel).suffix.lower()
    physical = count_lines(data)
    production, inline = split_rust(data) if suffix == ".rs" and not is_test_path(rel) else (physical, 0)
    if suffix == ".sh":
        category = "shell"
    elif suffix in {".html", ".css"}:
        category = "web"
    elif is_test_path(rel):
        category = "test"
    else:
        category = "production"
    # expanded_lines is meaningful only for an include root. Ordinary Rust
    # modules are governed by pre-test/file metrics and must not have their
    # inline tests accidentally folded back into the production metric.
    logical = logical_lines(root, rel) if suffix == ".rs" and "include!(" in data.decode("utf-8", errors="replace") else 0
    return Measurement(physical, production, inline, logical, category)


def blob(root: Path, ref: str, rel: str) -> bytes | None:
    proc = subprocess.run(
        ["git", "show", f"{ref}:{rel}"], cwd=root, stdout=subprocess.PIPE,
        stderr=subprocess.DEVNULL, check=False,
    )
    return proc.stdout if proc.returncode == 0 else None


def metric_values(metric: Measurement) -> dict[str, int]:
    return {
        "file_lines": metric.physical,
        "pre_test_lines": metric.production,
        "inline_test_lines": metric.inline_test,
        "expanded_lines": metric.logical,
    }


def logical_lines_at_ref(root: Path, ref: str, rel: str, stack: tuple[str, ...] = ()) -> int:
    if rel in stack:
        raise PolicyError(f"include! cycle at {ref}: {' -> '.join((*stack, rel))}")
    data = blob(root, ref, rel)
    if data is None:
        raise PolicyError(f"{ref}: missing included source {rel}")
    total = count_lines(data)
    text = data.decode("utf-8", errors="replace")
    for raw in INCLUDE_RE.findall(text):
        target = (PurePosixPath(rel).parent / raw).as_posix()
        canonical_rel(target)
        total += logical_lines_at_ref(root, ref, target, (*stack, rel))
    return total


def diff_numstat(root: Path, base: str) -> dict[str, tuple[int, int]]:
    raw = run_git(root, ["diff", "--numstat", base, "--"])
    assert isinstance(raw, str)
    result: dict[str, tuple[int, int]] = {}
    for line in raw.splitlines():
        parts = line.split("\t")
        if len(parts) != 3 or parts[0] == "-" or parts[1] == "-":
            continue
        result[parts[2]] = (int(parts[0]), int(parts[1]))
    return result


def structural_extraction_evidence(
    root: Path, base: str, legacy_rel: str, sources: set[str]
) -> bool:
    """Recognize a real sibling module receiving implementation in this change."""
    stats = diff_numstat(root, base)
    raw = run_git(
        root, ["ls-files", "-z", "--others", "--exclude-standard"], text=False
    )
    assert isinstance(raw, bytes)
    untracked = {
        item.decode("utf-8", errors="surrogateescape")
        for item in raw.split(b"\0")
        if item
    }
    legacy = PurePosixPath(legacy_rel)
    legacy_text = (root / legacy_rel).read_text(encoding="utf-8", errors="replace")
    for candidate in sorted(sources - {legacy_rel}):
        path = PurePosixPath(candidate)
        if path.suffix.lower() != legacy.suffix.lower():
            continue
        if path.parent != legacy.parent and path.parent.parent != legacy.parent:
            continue
        added = stats.get(candidate, (0, 0))[0]
        if candidate in untracked:
            added = count_lines((root / candidate).read_bytes())
        if added <= 0:
            continue
        candidate_text = (root / candidate).read_text(
            encoding="utf-8", errors="replace"
        )
        if path.suffix.lower() == ".rs":
            if "include!(" in candidate_text:
                continue
            stem = path.stem if path.name != "mod.rs" else path.parent.name
            if not re.search(
                rf"\b(?:pub\s+)?mod\s+{re.escape(stem)}\s*;", legacy_text
            ):
                continue
        elif path.suffix.lower() == ".py":
            module = path.stem
            if not re.search(
                rf"\b(?:from\s+\S+\s+import\s+.*\b{re.escape(module)}\b|"
                rf"import\s+.*\b{re.escape(module)}\b)",
                legacy_text,
            ):
                continue
        return True
    return False


def base_checks(root: Path, base: str, manifest_rel: str, policy: dict[str, Any], live: dict[str, Measurement]) -> list[str]:
    run_git(root, ["rev-parse", "--verify", f"{base}^{{commit}}"])
    old_bytes = blob(root, base, manifest_rel)
    if old_bytes is None:
        raise PolicyError(f"base {base} has no {manifest_rel}")
    failures: list[str] = []
    try:
        raw_old = json.loads(old_bytes.decode("utf-8"))
    except (UnicodeDecodeError, json.JSONDecodeError) as exc:
        raise PolicyError(f"invalid base policy: {exc}") from exc
    # A schema-v1 base is the one sanctioned migration/bootstrap. Thereafter
    # all policy evolution is checked against schema v2.
    if not isinstance(raw_old, dict) or raw_old.get("schema_version") != 2:
        return failures
    old = load_policy_bytes(old_bytes, f"{base}:{manifest_rel}")
    old_decision = blob(
        root, base,
        "docs/decisions/PRODUCT_MECHANICS_022_SOURCE_HEALTH_GOVERNANCE.md",
    )
    decision_path = root / "docs/decisions/PRODUCT_MECHANICS_022_SOURCE_HEALTH_GOVERNANCE.md"
    if old_decision is not None and decision_path.read_bytes() != old_decision:
        failures.append("policy weakening: decision 022 is immutable; add a later amendment")
    old_checker = blob(root, base, "scripts/check_source_health.py")
    if old_checker is not None:
        old_thresholds = assignment_literal(old_checker, "DEFAULT_THRESHOLDS")
        old_extensions = assignment_literal(old_checker, "EXTENSIONS")
        if isinstance(old_thresholds, dict):
            for category, old_limit in old_thresholds.items():
                if DEFAULT_THRESHOLDS.get(category, old_limit + 1) > old_limit:
                    failures.append(f"policy weakening: {category} threshold increased")
        if isinstance(old_extensions, set) and not old_extensions <= EXTENSIONS:
            failures.append("policy weakening: authored source extensions were removed")
    if policy["baseline_commit"] != old["baseline_commit"]:
        failures.append("policy weakening: baseline_commit is immutable")
    if policy["exclusions"] != old["exclusions"]:
        failures.append("policy weakening: exclusions changed")
    old_entries = old["entries"]
    new_entries = policy["entries"]
    for rel in sorted(set(new_entries) - set(old_entries)):
        failures.append(f"policy weakening: new legacy entry is forbidden: {rel}")
    for rel, old_entry in old_entries.items():
        if rel not in new_entries:
            continue
        for name, old_limit in old_entry["limits"].items():
            new_limit = new_entries[rel]["limits"].get(name)
            if new_limit is None or new_limit > old_limit:
                failures.append(f"policy weakening: {rel} limit {name} removed or raised")
        if new_entries[rel]["target_lines"] > old_entry["target_lines"]:
            failures.append(f"policy weakening: {rel} target raised")
    changed_raw = run_git(root, ["diff", "--name-only", "-z", base, "--"], text=False)
    assert isinstance(changed_raw, bytes)
    changed = {p.decode("utf-8", errors="surrogateescape") for p in changed_raw.split(b"\0") if p}
    source_set = set(live)
    for rel in sorted(set(new_entries) & changed):
        before = blob(root, base, rel)
        if before is None or rel not in live:
            failures.append(f"touched legacy module cannot be compared to base: {rel}")
        else:
            prior_file = count_lines(before)
            if live[rel].physical >= prior_file:
                failures.append(f"touched legacy module must shrink: {rel} is {live[rel].physical}, base is {prior_file}")
            elif not structural_extraction_evidence(root, base, rel, source_set):
                failures.append(
                    f"touched legacy module lacks real extraction evidence: {rel}"
                )
            if "expanded_lines" in new_entries[rel]["limits"]:
                old_expanded = logical_lines_at_ref(root, base, rel)
                if live[rel].logical >= old_expanded:
                    failures.append(f"include expansion must shrink, not move text: {rel} is {live[rel].logical}, base is {old_expanded}")
            values = metric_values(live[rel])
            for name, old_limit in old_entries[rel]["limits"].items():
                if values[name] < old_limit and new_entries[rel]["limits"].get(name) != values[name]:
                    failures.append(
                        f"legacy limit must ratchet exactly: {rel} {name} is "
                        f"{values[name]}, proposed {new_entries[rel]['limits'].get(name)}"
                    )
    return failures


def evaluate(root: Path, manifest: Path, base_ref: str | None) -> tuple[list[str], dict[str, Measurement]]:
    policy = load_policy(manifest)
    sources = discover_sources(root, policy)
    live: dict[str, Measurement] = {}
    failures: list[str] = repository_contract_failures(root)
    for rel in sources:
        try:
            live[rel] = measure(root, rel)
        except (OSError, PolicyError) as exc:
            failures.append(str(exc))
    entries = policy["entries"]
    thresholds = DEFAULT_THRESHOLDS
    for rel, metric in sorted(live.items()):
        limit = thresholds[metric.category]
        assessed = metric.production if metric.category == "production" else metric.physical
        # Logical Rust size guards against include!-only file splitting.
        peak = max(assessed, metric.logical)
        entry = entries.get(rel)
        if entry:
            for name, ceiling in entry["limits"].items():
                actual = metric_values(metric)[name]
                if actual > ceiling:
                    failures.append(f"legacy ceiling exceeded: {rel} {name}={actual} ceiling={ceiling}")
            if peak <= limit and metric.inline_test <= thresholds["inline_test"]:
                failures.append(f"legacy entry must be removed after burn-down: {rel}")
        elif peak > limit:
            failures.append(f"oversized {metric.category} source: {rel} is {peak} lines (limit {limit})")
        if not entry and metric.inline_test > thresholds["inline_test"]:
            failures.append(f"oversized inline test tail: {rel} is {metric.inline_test} lines (limit {thresholds['inline_test']})")
    for rel in sorted(entries):
        if rel not in live:
            failures.append(f"stale legacy entry: {rel}")
    if base_ref:
        manifest_rel = manifest.resolve().relative_to(root.resolve()).as_posix()
        failures.extend(base_checks(root, base_ref, manifest_rel, policy, live))
    return sorted(set(failures)), live


def main(argv: list[str] | None = None) -> int:
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--root", type=Path, default=Path(__file__).resolve().parents[1])
    parser.add_argument("--manifest", type=Path)
    parser.add_argument("--base-ref", help="git commit/ref used for immutable policy and touched-file checks")
    parser.add_argument("--json", action="store_true", help="emit stable machine-readable output")
    args = parser.parse_args(argv)
    root = args.root.resolve()
    manifest = args.manifest or root / "specs/source_module_size_manifest.json"
    try:
        failures, live = evaluate(root, manifest, args.base_ref)
    except (OSError, PolicyError, ValueError) as exc:
        failures, live = [str(exc)], {}
    result = {"ok": not failures, "files_checked": len(live), "failures": failures}
    if args.json:
        print(json.dumps(result, sort_keys=True, separators=(",", ":")))
    elif failures:
        print("Source health governance FAILED:", file=sys.stderr)
        for failure in failures:
            print(f"- {failure}", file=sys.stderr)
    else:
        print(f"Source health governance passed ({len(live)} source files checked).")
    return int(bool(failures))


if __name__ == "__main__":
    raise SystemExit(main())
