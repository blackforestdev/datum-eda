#!/usr/bin/env python3
"""Guard the GP-CM03 one-service and private-writer boundary."""

from __future__ import annotations

from pathlib import Path
import re
import sys


ROOT = Path(__file__).resolve().parents[1]
ADAPTER_ROOTS = (
    Path("crates/cli/src"),
    Path("crates/engine-daemon/src"),
    Path("crates/gui-app/src"),
    Path("mcp-server"),
)
BANNED_ADAPTER_AUTHORITY = (
    "PreferenceRepository",
    "GlobalPreferencesService",
    "resolve_preference(",
    "DATUM_GUI_PREFERENCES_PATH",
    "XDG_CONFIG_HOME",
)

ALLOWED_PRODUCT_SERVICE_FILES = {
    Path("crates/cli/src/commands/preferences.rs"),
    Path("crates/cli/src/commands/project/genesis_product.rs"),
    Path("crates/engine-daemon/src/preferences_state.rs"),
    Path("crates/gui-app/src/windows/global_preferences_runtime.rs"),
}

PREFERENCE_WRITE_FILES = {
    Path("crates/engine/src/preferences/repository/io.rs"),
    Path("crates/engine/src/preferences/repository/recovery.rs"),
}


def production_files(root: Path) -> list[Path]:
    files: list[Path] = []
    for adapter_root in ADAPTER_ROOTS:
        base = root / adapter_root
        for suffix in ("*.rs", "*.py"):
            for path in base.rglob(suffix):
                relative = path.relative_to(root)
                if (
                    "tests" in relative.parts
                    or "main_tests" in relative.parts
                    or path.name.endswith("_tests.rs")
                    or path.name == "tests.rs"
                    or path.name.startswith("test_")
                ):
                    continue
                files.append(path)
    return sorted(set(files))


def adapter_authority(root: Path) -> tuple[set[Path], list[str]]:
    """Inspect adapter files conservatively, without skipping later production."""
    failures: list[str] = []
    product_users: set[Path] = set()
    for path in production_files(root):
        relative = path.relative_to(root)
        text = path.read_text(encoding="utf-8")
        for marker in BANNED_ADAPTER_AUTHORITY:
            if marker in text:
                failures.append(f"{relative}: private preference authority `{marker}`")
        if "GlobalPreferencesProductService" in text:
            product_users.add(relative)
    return product_users, failures


def check() -> int:
    product_users, failures = adapter_authority(ROOT)

    if product_users != ALLOWED_PRODUCT_SERVICE_FILES:
        failures.append(
            "GlobalPreferencesProductService adapter inventory changed: "
            f"expected {sorted(map(str, ALLOWED_PRODUCT_SERVICE_FILES))}, "
            f"observed {sorted(map(str, product_users))}"
        )

    preferences_root = ROOT / "crates/engine/src/preferences"
    write_pattern = re.compile(
        r"(?:std::fs::|fs::)(?:write|rename|create_dir|create_dir_all|remove_file)\s*\("
    )
    observed_writers: set[Path] = set()
    for path in preferences_root.rglob("*.rs"):
        relative = path.relative_to(ROOT)
        if (
            "tests" in relative.parts
            or path.name == "tests.rs"
            or path.name.endswith("_tests.rs")
        ):
            continue
        # Files with inline tests are checked only before the test module.
        text = path.read_text(encoding="utf-8").split("#[cfg(test)]", 1)[0]
        if write_pattern.search(text):
            observed_writers.add(relative)

    unexpected = observed_writers - PREFERENCE_WRITE_FILES - {
        Path("crates/engine/src/preferences/project_genesis.rs")
    }
    for path in sorted(unexpected):
        failures.append(f"{path}: preference filesystem mutation outside repository writer")
    missing = PREFERENCE_WRITE_FILES - observed_writers
    for path in sorted(missing):
        failures.append(f"{path}: expected repository writer markers disappeared")

    preference_tools = (
        ROOT / "mcp-server/tools_catalog_preferences.py"
    ).read_text(encoding="utf-8")
    for forbidden in ('_tool("datum.preferences.set"', '_tool("datum.preferences.reset"'):
        if forbidden in preference_tools:
            failures.append(f"public MCP direct mutation appeared: {forbidden}")

    project_args = (ROOT / "crates/cli/src/args/project_genesis.rs").read_text(
        encoding="utf-8"
    )
    if "preferences_path" in project_args or "config_root" in project_args:
        failures.append("Project genesis CLI gained caller-selected preference storage")

    if failures:
        print("Global Preferences boundary gate failed:", file=sys.stderr)
        for failure in failures:
            print(f"  - {failure}", file=sys.stderr)
        return 1

    print(
        "Global Preferences boundary gate passed "
        f"({len(product_users)} product-service adapters; "
        f"{len(observed_writers)} engine mutation files)."
    )
    return 0


if __name__ == "__main__":
    raise SystemExit(check())
