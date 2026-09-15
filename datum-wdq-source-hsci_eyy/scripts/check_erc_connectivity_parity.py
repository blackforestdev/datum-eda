#!/usr/bin/env python3
"""Lock ERC pin taxonomy and schematic connectivity spec/code parity."""

from __future__ import annotations

import re
import sys
from pathlib import Path

ROOT = Path(__file__).resolve().parents[1]

PIN_VARIANTS = [
    ("Input", "input"),
    ("Output", "output"),
    ("Bidirectional", "bidirectional"),
    ("Passive", "passive"),
    ("PowerIn", "power_in"),
    ("PowerOut", "power_out"),
    ("OpenCollector", "open_collector"),
    ("OpenEmitter", "open_emitter"),
    ("TriState", "tri_state"),
    ("NoConnect", "no_connect"),
]

ERC_CODES = [
    "output_to_output_conflict",
    "power_in_without_source",
    "noconnect_connected",
    "input_without_explicit_driver",
    "undriven_input_pin",
    "unconnected_component_pin",
    "unconnected_interface_port",
    "undriven_power_net",
    "undriven_named_net",
    "hierarchical_connectivity_mismatch",
]

CONNECTIVITY_DIAGNOSTICS = [
    "unsupported_bus_member_syntax",
    "dangling_component_pin",
    "dangling_interface_port",
    "anonymous_multi_pin_net",
    "missing_hierarchical_port_target",
    "multiply_mapped_hierarchical_port",
]

STALE_TAXONOMY_CLAIMS = [
    "narrows it to six variants",
    "Shipped (six variants)",
    "shipped six-variant taxonomy",
    "still six variants",
    "six-variant PinElectricalType",
    "TriState, and the pin-type",
]


def read(relative: str) -> str:
    return (ROOT / relative).read_text(encoding="utf-8")


def enum_body(source: str, enum_name: str) -> list[str]:
    match = re.search(rf"enum\s+{re.escape(enum_name)}\s*\{{(?P<body>.*?)\n\}}", source, re.S)
    if not match:
        raise AssertionError(f"missing enum {enum_name}")
    body = match.group("body")
    return re.findall(r"^\s*([A-Z][A-Za-z0-9_]*)\s*,", body, re.M)


def quoted_kind_values(source: str) -> set[str]:
    return set(re.findall(r'kind:\s*"([a-z0-9_]+)"\.into\(\)', source))


def fail(message: str, failures: list[str]) -> None:
    failures.append(message)


def main() -> int:
    failures: list[str] = []

    pool_pin = read("crates/engine/src/pool/pin.rs")
    schematic = read("crates/engine/src/schematic/mod.rs")
    erc_electrical = read("crates/engine/src/erc/electrical.rs")
    erc_mod = read("crates/engine/src/erc/mod.rs")
    connectivity_mod = read("crates/engine/src/connectivity/mod.rs")
    erc_spec = read("specs/ERC_SPEC.md")
    connectivity_spec = read("specs/SCHEMATIC_CONNECTIVITY_SPEC.md")
    erc_rationale = read("docs/ERC_SPEC.md")
    progress = read("specs/PROGRESS.md")

    expected_variants = [variant for variant, _snake in PIN_VARIANTS]
    try:
        actual_variants = enum_body(pool_pin, "LibraryPinElectricalType")
    except AssertionError as exc:
        fail(str(exc), failures)
        actual_variants = []
    if actual_variants != expected_variants:
        fail(
            "LibraryPinElectricalType variants drifted: "
            f"expected {expected_variants}, got {actual_variants}",
            failures,
        )

    if "pub type PinDirection = LibraryPinElectricalType;" not in pool_pin:
        fail("PinDirection must remain a compatibility alias to LibraryPinElectricalType", failures)
    if "pub use crate::pool::LibraryPinElectricalType as PinElectricalType;" not in schematic:
        fail("schematic::PinElectricalType must remain the pool-owned alias", failures)

    mapping = dict(
        re.findall(r'PinElectricalType::([A-Za-z0-9_]+)\s*=>\s*"([a-z0-9_]+)"', erc_electrical)
    )
    expected_mapping = dict(PIN_VARIANTS)
    if mapping != expected_mapping:
        fail(f"ERC canonical pin-name mapping drifted: expected {expected_mapping}, got {mapping}", failures)
    if 'PIN_ELECTRICAL_TAXONOMY_REVISION: &str = "LibraryPinElectricalType:v1"' not in erc_electrical:
        fail("ERC taxonomy revision must remain LibraryPinElectricalType:v1", failures)

    for path, text in {
        "specs/ERC_SPEC.md": erc_spec,
        "docs/ERC_SPEC.md": erc_rationale,
        "specs/PROGRESS.md": progress,
    }.items():
        for variant, snake in PIN_VARIANTS:
            if variant not in text and snake not in text:
                fail(f"{path} omits canonical pin type {variant}/{snake}", failures)

    for stale in STALE_TAXONOMY_CLAIMS:
        if stale in erc_spec:
            fail(f"specs/ERC_SPEC.md contains stale taxonomy claim: {stale!r}", failures)
        if stale in progress:
            fail(f"specs/PROGRESS.md contains stale taxonomy claim: {stale!r}", failures)

    for code in ERC_CODES:
        if f'"{code}"' not in erc_mod and code not in erc_mod:
            fail(f"ERC implementation no longer emits expected code {code}", failures)
        if f"`{code}`" not in erc_spec:
            fail(f"specs/ERC_SPEC.md does not document emitted ERC code `{code}`", failures)

    actual_diagnostics = quoted_kind_values(connectivity_mod)
    expected_diagnostics = set(CONNECTIVITY_DIAGNOSTICS)
    if actual_diagnostics != expected_diagnostics:
        fail(
            "connectivity diagnostic kind set drifted: "
            f"expected {sorted(expected_diagnostics)}, got {sorted(actual_diagnostics)}",
            failures,
        )
    for diagnostic in CONNECTIVITY_DIAGNOSTICS:
        if f"`{diagnostic}`" not in connectivity_spec:
            fail(
                "specs/SCHEMATIC_CONNECTIVITY_SPEC.md does not document "
                f"connectivity diagnostic `{diagnostic}`",
                failures,
            )

    if failures:
        print("ERC/connectivity parity check failed:")
        for item in failures:
            print(f" - {item}")
        return 1

    print(
        "ERC/connectivity parity OK "
        f"({len(PIN_VARIANTS)} pin types, {len(ERC_CODES)} ERC codes, "
        f"{len(CONNECTIVITY_DIAGNOSTICS)} connectivity diagnostics)"
    )
    return 0


if __name__ == "__main__":
    sys.exit(main())
