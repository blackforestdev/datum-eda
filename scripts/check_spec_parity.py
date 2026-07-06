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
    return set(re.findall(r'^\s*"([a-z_.]+)"\s*=>', text, flags=re.M))


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
    variants = re.findall(r"^\s{4}([A-Z][A-Za-z0-9]+)(?:\s*\(|\s*\{|\s*,|\s*$)", body, flags=re.M)
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


def mcp_migrated_prefixes() -> set[str]:
    """Verb-family prefixes whose MCP tool dicts are generated from the
    verb-registry catalog (``MIGRATED_PREFIXES`` in tools_catalog_generated.py)
    instead of being hand-written in tools_catalog_datum.py."""
    text = read_text(ROOT / "mcp-server/tools_catalog_generated.py")
    match = re.search(r"MIGRATED_PREFIXES[^=]*=\s*frozenset\(\s*\{(.*?)\}\s*\)", text, flags=re.S)
    if match is None:
        return set()
    return set(re.findall(r'"([^"]+)"', match.group(1)))


def datum_tool_names(prefix: str) -> list[str]:
    text = read_text(ROOT / "mcp-server/tools_catalog_datum.py")
    names = set(re.findall(r'"name":\s*"([^"]+)"', text))
    # Migrated families no longer appear as hand-written literals; derive
    # their tool names from the generated verb-registry catalog instead.
    migrated = mcp_migrated_prefixes()
    catalog = json.loads(read_text(ROOT / "mcp-server/datum_tool_catalog.json"))
    for verb in catalog.get("verbs", []):
        name = str(verb["name"])
        if ".".join(name.split(".")[:2]) in migrated:
            names.add(name)
    return sorted(name for name in names if name.startswith(prefix))


def project_command_variants() -> list[str]:
    return rust_enum_variants(ROOT / "crates/cli/src/args/project.rs", "ProjectCommands")


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


def standards_check_surface() -> list[str]:
    """Current standards-aware check/repair identity surface.

    This inventory is intentionally a marker set rather than every check test:
    it freezes the implemented schema fields, repair/query public tools,
    persisted CheckRun operation, and current standards-profile finding codes.
    """
    items: set[str] = set()
    operations = set(rust_enum_variants(ROOT / "crates/engine/src/substrate/operation.rs", "Operation"))
    if "SetCheckRun" in operations:
        items.add("operation:SetCheckRun")

    commands = set(project_command_variants())
    for command in ("GenerateStandardsRepairProposals", "WaiveFinding", "AcceptDeviation"):
        if command in commands:
            items.add(f"cli:{command}")

    for name in datum_tool_names("datum.check."):
        if name in {
            "datum.check.repair_standards",
            "datum.check.run",
            "datum.check.runs",
            "datum.check.show_run",
            "datum.check.waive",
            "datum.check.accept_deviation",
        }:
            items.add(f"mcp:{name}")
    for name in datum_tool_names("datum.query."):
        if name == "datum.query.zone_fills":
            items.add(f"mcp:{name}")

    check_run_text = read_text(ROOT / "crates/engine/src/substrate/check_run.rs")
    for field in ("standards_basis", "rule_revision", "import_key"):
        if re.search(rf"\bpub\s+{field}\s*:\s*Option<String>", check_run_text):
            items.add(f"check_finding_field:{field}")

    identity_text = read_text(ROOT / "crates/cli/src/commands/check/finding_identity.rs")
    for function in (
        "is_standards_profile_finding",
        "check_finding_standards_basis",
        "check_finding_rule_revision",
        "check_finding_import_key",
        "check_finding_fingerprint",
    ):
        if re.search(rf"\bfn\s+{function}\b", identity_text):
            items.add(f"identity_fn:{function}")

    for code in sorted(set(re.findall(r'"([a-z_]+(?:_below_min|_missing|_below_rule|_out_of_range|_unfilled|_stale|_unsupported|_inherited_from_copper|_inconsistent_with_peer_footprint))"', identity_text))):
        items.add(f"finding_code:{code}")

    return sorted(items)


def pool_library_surface() -> list[str]:
    """Current pool/library operation, CLI, and public MCP surface."""
    items: set[str] = set()
    for variant in rust_enum_variants(ROOT / "crates/engine/src/substrate/operation.rs", "Operation"):
        if variant.startswith("CreatePool") or variant.startswith("SetPool") or variant.startswith("DeletePool"):
            items.add(f"operation:{variant}")

    for variant in project_command_variants():
        if "Pool" in variant or "Library" in variant:
            items.add(f"cli:{variant}")

    for name in datum_tool_names("datum.library."):
        items.add(f"mcp:{name}")

    return sorted(items)


PIN_ELECTRICAL_TYPES = (
    "Input",
    "Output",
    "Bidirectional",
    "Passive",
    "PowerIn",
    "PowerOut",
    "OpenCollector",
    "OpenEmitter",
    "TriState",
    "NoConnect",
)

ERC_FINDING_CODES = (
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
)

CONNECTIVITY_DIAGNOSTIC_KINDS = (
    "unsupported_bus_member_syntax",
    "dangling_component_pin",
    "dangling_interface_port",
    "anonymous_multi_pin_net",
    "missing_hierarchical_port_target",
    "multiply_mapped_hierarchical_port",
)


def erc_pin_taxonomy_surface() -> list[str]:
    """Canonical pin taxonomy and current ERC finding-code surface."""
    items: set[str] = set()
    pool_pin = read_text(ROOT / "crates/engine/src/pool/pin.rs")
    schematic = read_text(ROOT / "crates/engine/src/schematic/mod.rs")
    electrical = read_text(ROOT / "crates/engine/src/erc/electrical.rs")
    erc_mod = read_text(ROOT / "crates/engine/src/erc/mod.rs")

    actual_variants = set(
        rust_enum_variants(
            ROOT / "crates/engine/src/pool/pin.rs",
            "LibraryPinElectricalType",
        )
    )
    for variant in PIN_ELECTRICAL_TYPES:
        if variant in actual_variants:
            items.add(f"pin_type:{variant}")

    if "pub type PinDirection = LibraryPinElectricalType;" in pool_pin:
        items.add("alias:pool.PinDirection=LibraryPinElectricalType")
    if "pub use crate::pool::LibraryPinElectricalType as PinElectricalType;" in schematic:
        items.add("alias:schematic.PinElectricalType=LibraryPinElectricalType")
    if 'PIN_ELECTRICAL_TAXONOMY_REVISION: &str = "LibraryPinElectricalType:v1"' in electrical:
        items.add("taxonomy_revision:LibraryPinElectricalType:v1")

    for variant, name in re.findall(
        r'PinElectricalType::([A-Za-z0-9_]+)\s*=>\s*"([a-z0-9_]+)"',
        electrical,
    ):
        items.add(f"canonical_name:{variant}={name}")

    for code in ERC_FINDING_CODES:
        if code in erc_mod:
            items.add(f"erc_code:{code}")

    return sorted(items)


def schematic_connectivity_surface() -> list[str]:
    """Current schematic connectivity diagnostic kind surface."""
    text = read_text(ROOT / "crates/engine/src/connectivity/mod.rs")
    items: set[str] = set()
    for kind in CONNECTIVITY_DIAGNOSTIC_KINDS:
        if f'kind: "{kind}".into()' in text:
            items.add(f"connectivity_diagnostic:{kind}")
    if "pub fn schematic_net_info" in text:
        items.add("query:schematic_net_info")
    if "pub fn schematic_diagnostics" in text:
        items.add("query:schematic_diagnostics")
    if "pub fn schematic_hierarchy_info" in text:
        items.add("query:schematic_hierarchy_info")
    return sorted(items)


def zone_fill_surface() -> list[str]:
    """Native ZoneFill generated-evidence format and projection surface."""
    items: set[str] = set()
    operations = set(rust_enum_variants(ROOT / "crates/engine/src/substrate/operation.rs", "Operation"))
    for variant in ("SetZoneFill", "DeleteZoneFill"):
        if variant in operations:
            items.add(f"operation:{variant}")

    substrate_mod = read_text(ROOT / "crates/engine/src/substrate/mod.rs")
    if "ZoneFill," in substrate_mod:
        items.add("source_shard_taxon:ZoneFill")
    if "pub zone_fills: BTreeMap<ObjectId, ZoneFill>" in substrate_mod:
        items.add("design_model_field:zone_fills")

    zone_fill = read_text(ROOT / "crates/engine/src/substrate/zone_fill.rs")
    for marker in (
        "pub const ZONE_FILL_SCHEMA_VERSION: u64 = 1;",
        'persist_generated_evidence(project_root, ".datum/zone_fills", &fill.zone_id, fill)',
        "pub fn compute_bounded_zone_fill",
        "pub fn zone_fill_copper_projection_zones",
        "pub(super) fn derive_model_zone_fills",
        "SourceShardKind::ZoneFill",
        "SourceShardAuthority::GeneratedEvidence",
        "SourceShardTaxon::ZoneFill",
        "ZoneFillState::Filled",
        "ZoneFillState::Unfilled",
        "ZoneFillState::Stale",
        "ZoneFillState::Unsupported",
    ):
        if marker in zone_fill:
            items.add(f"marker:{marker}")

    board_routing = read_text(ROOT / "crates/engine/src/api/native_write/board_routing.rs")
    for marker in ("pub fn build_set_zone_fills", "Operation::SetZoneFill", "Operation::DeleteZoneFill"):
        if marker in board_routing:
            items.add(f"native_write:{marker}")

    for name in datum_tool_names("datum.check."):
        if name == "datum.check.fill_zones":
            items.add(f"mcp:{name}")
    for name in datum_tool_names("datum.query."):
        if name == "datum.query.zone_fills":
            items.add(f"mcp:{name}")

    return sorted(items)


def gui_supervision_surface() -> list[str]:
    """Current read-only GUI supervision/status reflection surface."""
    items: set[str] = set()
    protocol = read_text(ROOT / "crates/gui-protocol/src/lib.rs")
    render = read_text(ROOT / "crates/gui-render/src/outputs_lane.rs")
    layout = read_text(ROOT / "crates/gui-render/src/outputs_lane_layout.rs")

    for marker in (
        'GUI_SUPERVISION_SNAPSHOT_CONTRACT: &str = "datum_gui_supervision_snapshot_v1"',
        "pub supervision: GuiSupervisionSnapshot",
        "pub struct GuiSupervisionSnapshot",
        "pub struct GuiJournalSupervision",
        "pub struct GuiSceneSupervision",
        "pub struct GuiCheckSupervision",
        "pub struct GuiDataSupervision",
        "fn load_gui_supervision_snapshot",
        "ProjectResolver::new(&request.project_root).resolve()",
        "snapshot.journal.applied_transaction_count",
        "snapshot.journal.accepted_transaction_tip",
        "SourceShardStatusSummary",
    ):
        if marker in protocol:
            items.add(f"protocol:{marker}")

    for marker in (
        "ProjectResolver::new(root).resolve()",
        "materialized_source_shard_value(SourceShardKind::BoardRoot)",
        "scene.source_revision = model.model_revision.0.clone()",
    ):
        if marker in protocol:
            items.add(f"scene_loader:{marker}")

    for marker in (
        "ENGINE SUPERVISION",
        "render_engine_supervision_section",
        "engine_supervision_section_height",
        "snapshot.source_shards.attention_count()",
    ):
        if marker in render:
            items.add(f"renderer:{marker}")

    if "Supervision," in layout:
        items.add("layout:OutputsBodySectionKind::Supervision")

    return sorted(items)


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
    if kind == "standards_check_surface":
        return standards_check_surface()
    if kind == "pool_library_surface":
        return pool_library_surface()
    if kind == "erc_pin_taxonomy_surface":
        return erc_pin_taxonomy_surface()
    if kind == "schematic_connectivity_surface":
        return schematic_connectivity_surface()
    if kind == "zone_fill_surface":
        return zone_fill_surface()
    if kind == "gui_supervision_surface":
        return gui_supervision_surface()
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
