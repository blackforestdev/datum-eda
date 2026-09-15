#!/usr/bin/env python3
"""Enforce the DTC-P21 integrated TerminalCore proof and measurement boundary."""

from __future__ import annotations

import sys
from pathlib import Path


ROOT = Path(__file__).resolve().parents[1]
CRATE = Path("crates/terminal-core")

REQUIRED_PROOFS = (
    "normative_corpus_matches_expected_state_and_every_chunk_partition",
    "seeded_generational_mutation_replays_and_shrinks_deterministically",
    "hostile_streams_exhaust_limits_then_reset_to_bounded_initial_state",
    "repeated_generations_remain_within_history_graphics_and_snapshot_resources",
    "DTC-R01-R03-ECMA48-001",
    "DTC-R02-DECSTATE-001",
    "DTC-R04-R07-UNICODE-HISTORY-001",
    "DTC-R06-METADATA-001",
    "DTC-R08-GRAPHICS-001",
    "PROOF_SEED",
    "minimize_replay",
    "report.consumed > 0",
    "core.render_snapshot()",
)

REQUIRED_PROBE = (
    '"datum-terminal-core-proof-v1"',
    "DEFAULT_PAYLOAD_BYTES: usize = 8 * 1024 * 1024",
    '"--release"',
    '"--offline"',
    '"dtc_p21_probe"',
    '"elapsed_ns"',
    '"mib_per_second"',
    '"errors"',
)

FORBIDDEN_PROOF_AUTHORITY = (
    "std::fs",
    "std::process",
    "std::net",
    "unsafe {",
    "unsafe fn",
    "include!",
    "wgpu",
    "winit",
    "glyphon",
    "gui_protocol",
    "DesignModel",
    "Operation",
)

FORBIDDEN_INVENTED_BUDGETS = (
    "assert!(mib_per_second",
    "mib_per_second >",
    "mib_per_second >=",
    "MIN_MIB_PER_SECOND",
)


def read(path: Path) -> str:
    return path.read_text(encoding="utf-8") if path.is_file() else ""


def check(root: Path) -> list[str]:
    failures: list[str] = []
    source = root / CRATE / "src"
    proof_path = source / "proof_tests.rs"
    proof = read(proof_path)
    if not proof:
        failures.append("DTC-P21 integrated proof corpus is missing: proof_tests.rs")
    for marker in REQUIRED_PROOFS:
        if marker not in proof:
            failures.append(f"DTC-P21 integrated proof lacks marker: {marker}")
    for marker in FORBIDDEN_PROOF_AUTHORITY:
        if marker in proof:
            failures.append(f"DTC-P21 core proof contains forbidden external authority: {marker}")

    library = read(source / "lib.rs")
    if "mod proof_tests;" not in library:
        failures.append("DTC-P21 proof corpus is not compiled by TerminalCore")

    manifest = read(root / CRATE / "Cargo.toml")
    dependency_tail = manifest.partition("[dependencies]")[2].strip()
    if dependency_tail:
        failures.append("DTC-P21 TerminalCore proof introduced a code dependency")

    probe = read(root / CRATE / "examples/dtc_p21_probe.rs")
    runner = read(root / "scripts/run_terminal_core_proof.py")
    combined_probe = probe + "\n" + runner
    for marker in REQUIRED_PROBE:
        if marker not in combined_probe:
            failures.append(f"DTC-P21 release measurement lacks marker: {marker}")
    for marker in FORBIDDEN_INVENTED_BUDGETS:
        if marker in combined_probe:
            failures.append(f"DTC-P21 invented an unratified performance budget: {marker}")

    drift = read(root / "scripts/run_drift_gates.sh")
    for command in (
        "python3 scripts/test_terminal_core_proof.py",
        "python3 scripts/check_terminal_core_proof.py",
        "python3 scripts/run_terminal_core_proof.py",
    ):
        if command not in drift:
            failures.append(f"DTC-P21 drift gate is missing: {command}")
    return failures


def main() -> int:
    failures = check(ROOT)
    if failures:
        print("Datum TerminalCore proof gate FAILED:", file=sys.stderr)
        for failure in failures:
            print(f"- {failure}", file=sys.stderr)
        return 1
    print("Datum TerminalCore proof gate passed.")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
