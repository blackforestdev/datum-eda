#!/usr/bin/env python3
"""Run and report the bounded GP-CM03 cross-surface proof corpus."""

from __future__ import annotations

from pathlib import Path
import subprocess
import sys


ROOT = Path(__file__).resolve().parents[1]
GUARD = [sys.executable, str(ROOT / "scripts/run_cargo_guarded.py"), "--workload", "proof", "--"]

STATE_FIXTURES = (
    ("missing", "crates/engine/src/preferences/service.rs", "reset_against_missing_repository_is_a_side_effect_free_noop", "assert!(!repository_root.exists())"),
    ("ready", "crates/engine/src/preferences/product_tests.rs", "mutation_idempotency_survives_restart_and_replays_the_original_generation", "expect(\"repository reopens\")"),
    ("stale", "crates/engine/src/preferences/service.rs", "stale_generation_preserves_draft_and_refreshes_truth", "PreferenceServiceRefusalKind::StaleGeneration"),
    ("competing_writer", "crates/engine/src/preferences/service.rs", "writer_conflict_preserves_draft_and_last_valid_truth", "PreferenceServiceRefusalKind::WriterConflict"),
    ("unreadable", "crates/engine/src/preferences/service.rs", "corrupt_head_stays_exact_and_disables_all_rows", "PreferenceServiceStatus::PreservedUnreadable"),
    ("migration_required", "crates/engine/src/preferences/repository/tests.rs", "migration_plan_is_pure_moves_one_alias_and_writes_a_pre_migration_backup", "RepositoryStatus::MigrationRequired"),
    ("v1_receipt", "crates/engine/src/api/native_write/project_units_receipt_tests.rs", "v1_units_receipt_projects_remain_readable_without_receipt_rewrite", "assert_eq!(model.project.project_units_seed_receipt, Some(exact_v1))"),
    ("global_seed", "crates/engine/src/preferences/project_genesis_lifecycle_tests.rs", "global_seed_is_pinned_once_and_never_follows_later_edits", "ProjectUnitsReceiptSourceV2::Global"),
    ("explicit_factory", "crates/engine/src/preferences/project_genesis.rs", "factory_genesis_publishes_one_complete_eight_item_project_without_preferences", "ACTIVE_UNITS_KEYS"),
)

FAULT_POINTS = (
    "staging_prepared",
    "project_built",
    "project_validated",
    "staging_synced",
    "project_published",
)


def verify_fixture_inventory() -> None:
    failures: list[str] = []
    for state, relative, test_name, marker in STATE_FIXTURES:
        text = (ROOT / relative).read_text(encoding="utf-8")
        if f"fn {test_name}(" not in text:
            failures.append(f"{state}: missing test {relative}::{test_name}")
        if marker not in text:
            failures.append(f"{state}: missing authority assertion {marker!r}")
    lifecycle = (ROOT / "crates/engine/src/preferences/project_genesis_lifecycle_tests.rs").read_text(
        encoding="utf-8"
    )
    for point in FAULT_POINTS:
        rust_name = "".join(part.title() for part in point.split("_"))
        if f"GenesisCheckpoint::{rust_name}" not in lifecycle:
            failures.append(f"fault point not covered: {point}")
    if failures:
        raise RuntimeError("\n".join(failures))


def run(command: list[str]) -> None:
    subprocess.run(command, cwd=ROOT, check=True)


def main() -> int:
    verify_fixture_inventory()
    for checker in (
        "check_global_preferences_boundary.py",
        "check_spec_parity.py",
        "check_resolver_raw_loads.py",
        "check_daemon_write_parity.py",
        "check_mcp_public_taxonomy.py",
        "check_dependency_authority.py",
        "check_cargo_resource_policy.py",
    ):
        run([sys.executable, str(ROOT / "scripts" / checker)])

    for package, test_filter in (
        ("eda-engine", "preferences::"),
        ("datum-eda-cli", "preferences"),
        ("datum-eda-cli", "project_new"),
        ("eda-engine-daemon", "preferences"),
        ("datum-gui-app", "global_preferences"),
    ):
        run([*GUARD, "cargo", "test", "-p", package, test_filter])
    run(
        [
            sys.executable,
            "-m",
            "unittest",
            "discover",
            "-s",
            "mcp-server",
            "-p",
            "test_preferences_product.py",
        ]
    )

    print("GP-CM03 bounded proof report: PASS")
    print("Durable states: " + ", ".join(state for state, *_ in STATE_FIXTURES))
    print("Genesis fault injection: " + ", ".join(FAULT_POINTS))
    print("Atomic authority: every pre-publication fault leaves the destination absent; ")
    print("post-publication loss replays one resolver-valid Project; no partial preference generation or Project becomes authoritative.")
    print("Boundary: 11 active Global settings, eight Project Units seeds, no live following, no private adapter writer, no direct MCP Set/Reset.")
    print("Disposition: GP-CM03 implementation proof only; no GP-CM04 or production-acceptance claim.")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
