"""Generated datum.* MCP tool specs loaded from the verb-registry catalog.

`datum_tool_catalog.json` is the checked-in projection of the single-source
verb registry (`crates/verb-registry`). It is emitted by
`cargo run -p datum-verb-registry --bin datum-verb-catalog -- --write` and
drift-gated by `-- --check` in `scripts/run_drift_gates.sh`.

Verb families migrate out of the hand-written catalogs one prefix at a time:
a prefix listed in ``MIGRATED_PREFIXES`` is owned by the generated catalog and
its hand-written entries must be deleted. The registry also declares verbs for
prefixes that are not yet migrated here (e.g. families whose GUI terminal
catalog is registry-projected while the MCP server still uses its hand-written
dicts); this loader filters strictly by ``MIGRATED_PREFIXES`` so those verbs
are never double-registered. Generated specs carry the exact same dict shape
as hand-written ones (``name``/``description``/``inputSchema`` plus
``x_dispatch_*`` and write-surface keys), so runtime dispatch through
``tool_dispatch.dispatch_tool_call`` and the ``server_runtime`` bridge methods
is byte-identical.
"""

from __future__ import annotations

import json
from pathlib import Path

# Verb-family prefixes whose tool specs come from the generated catalog.
MIGRATED_PREFIXES: frozenset[str] = frozenset({
    "datum.artifact",
    "datum.check",
    "datum.component_instance",
    "datum.context",
    "datum.journal",
    "datum.library",
    "datum.manufacturing",
    "datum.output_job",
    "datum.pcb",
    "datum.pool",
    "datum.project",
    "datum.proposal",
    "datum.query",
    "datum.replacement",
    "datum.route",
    "datum.schematic",
    "datum.session",
})

_CATALOG_PATH = Path(__file__).resolve().parent / "datum_tool_catalog.json"

_PREFIX_ORDERS: dict[str, tuple[str, ...]] = {
    "datum.pcb": (
        "datum.pcb.place_component",
        "datum.pcb.generate_board_components",
        "datum.pcb.move_component",
        "datum.pcb.rotate_component",
        "datum.pcb.flip_component",
        "datum.pcb.delete_component",
        "datum.pcb.set_component_reference",
        "datum.pcb.set_component_value",
        "datum.pcb.set_component_part",
        "datum.pcb.set_component_package",
        "datum.pcb.lock_component",
        "datum.pcb.unlock_component",
        "datum.pcb.draw_track",
        "datum.pcb.edit_track",
        "datum.pcb.delete_track",
        "datum.pcb.place_via",
        "datum.pcb.edit_via",
        "datum.pcb.delete_via",
        "datum.pcb.place_zone",
        "datum.pcb.edit_zone",
        "datum.pcb.delete_zone",
        "datum.pcb.place_pad",
        "datum.pcb.edit_pad",
        "datum.pcb.delete_pad",
        "datum.pcb.set_pad_net",
        "datum.pcb.clear_pad_net",
        "datum.pcb.place_net",
        "datum.pcb.edit_net",
        "datum.pcb.delete_net",
        "datum.pcb.set_board_name",
        "datum.pcb.set_outline",
        "datum.pcb.set_stackup",
        "datum.pcb.add_default_top_stackup",
        "datum.pcb.place_keepout",
        "datum.pcb.edit_keepout",
        "datum.pcb.delete_keepout",
        "datum.pcb.place_dimension",
        "datum.pcb.edit_dimension",
        "datum.pcb.delete_dimension",
        "datum.pcb.place_text",
        "datum.pcb.edit_text",
        "datum.pcb.delete_text",
        "datum.pcb.place_net_class",
        "datum.pcb.edit_net_class",
        "datum.pcb.delete_net_class",
    ),
    "datum.schematic": (
        "datum.schematic.create_sheet",
        "datum.schematic.delete_sheet",
        "datum.schematic.rename_sheet",
        "datum.schematic.create_sheet_definition",
        "datum.schematic.create_sheet_instance",
        "datum.schematic.delete_sheet_instance",
        "datum.schematic.move_sheet_instance",
        "datum.schematic.bind_sheet_instance_port",
        "datum.schematic.unbind_sheet_instance_port",
        "datum.schematic.draw_wire",
        "datum.schematic.delete_wire",
        "datum.schematic.place_junction",
        "datum.schematic.delete_junction",
        "datum.schematic.place_noconnect",
        "datum.schematic.delete_noconnect",
        "datum.schematic.place_label",
        "datum.schematic.rename_label",
        "datum.schematic.delete_label",
        "datum.schematic.place_port",
        "datum.schematic.edit_port",
        "datum.schematic.delete_port",
        "datum.schematic.create_bus",
        "datum.schematic.edit_bus_members",
        "datum.schematic.delete_bus",
        "datum.schematic.place_bus_entry",
        "datum.schematic.delete_bus_entry",
        "datum.schematic.place_text",
        "datum.schematic.edit_text",
        "datum.schematic.delete_text",
        "datum.schematic.place_drawing_line",
        "datum.schematic.place_drawing_rect",
        "datum.schematic.place_drawing_circle",
        "datum.schematic.place_drawing_arc",
        "datum.schematic.edit_drawing_line",
        "datum.schematic.edit_drawing_rect",
        "datum.schematic.edit_drawing_circle",
        "datum.schematic.edit_drawing_arc",
        "datum.schematic.delete_drawing",
        "datum.schematic.place_symbol",
        "datum.schematic.move_symbol",
        "datum.schematic.rotate_symbol",
        "datum.schematic.mirror_symbol",
        "datum.schematic.delete_symbol",
        "datum.schematic.set_symbol_reference",
        "datum.schematic.set_symbol_value",
        "datum.schematic.set_symbol_display_mode",
        "datum.schematic.set_symbol_hidden_power_behavior",
        "datum.schematic.set_symbol_unit",
        "datum.schematic.clear_symbol_unit",
        "datum.schematic.set_symbol_gate",
        "datum.schematic.clear_symbol_gate",
        "datum.schematic.set_symbol_entity",
        "datum.schematic.clear_symbol_entity",
        "datum.schematic.set_symbol_part",
        "datum.schematic.clear_symbol_part",
        "datum.schematic.set_symbol_lib_id",
        "datum.schematic.clear_symbol_lib_id",
        "datum.schematic.set_pin_override",
        "datum.schematic.clear_pin_override",
        "datum.schematic.add_symbol_field",
        "datum.schematic.edit_symbol_field",
        "datum.schematic.delete_symbol_field",
    ),
    "datum.library": (
        "datum.library.list_objects",
        "datum.library.show_object",
        "datum.library.pool_models",
        "datum.library.gc_pool_models",
        "datum.library.create_object",
        "datum.library.create_unit",
        "datum.library.set_unit_pin",
        "datum.library.create_symbol",
        "datum.library.add_symbol_line",
        "datum.library.add_symbol_rect",
        "datum.library.add_symbol_circle",
        "datum.library.add_symbol_arc",
        "datum.library.add_symbol_polygon",
        "datum.library.add_symbol_text",
        "datum.library.set_symbol_pin_anchor",
        "datum.library.create_entity",
        "datum.library.create_padstack",
        "datum.library.create_package",
        "datum.library.create_footprint",
        "datum.library.generate_ipc7351b_soic",
        "datum.library.set_footprint_pad",
        "datum.library.set_footprint_courtyard_rect",
        "datum.library.set_footprint_courtyard_polygon",
        "datum.library.add_footprint_silkscreen_line",
        "datum.library.add_footprint_silkscreen_rect",
        "datum.library.add_footprint_silkscreen_circle",
        "datum.library.add_footprint_silkscreen_polygon",
        "datum.library.set_package_pad",
        "datum.library.set_package_courtyard_rect",
        "datum.library.set_package_courtyard_polygon",
        "datum.library.add_package_silkscreen_line",
        "datum.library.add_package_silkscreen_rect",
        "datum.library.add_package_silkscreen_polygon",
        "datum.library.add_package_silkscreen_circle",
        "datum.library.add_package_silkscreen_arc",
        "datum.library.add_package_silkscreen_text",
        "datum.library.add_package_model_3d",
        "datum.library.set_package_body_heights",
        "datum.library.create_part",
        "datum.library.set_part_metadata",
        "datum.library.set_part_parametric",
        "datum.library.set_part_orderable_mpns",
        "datum.library.set_part_tags",
        "datum.library.set_part_packaging_options",
        "datum.library.set_part_supply_chain",
        "datum.library.set_part_behavioural_models",
        "datum.library.attach_part_model",
        "datum.library.detach_part_model",
        "datum.library.set_part_thermal",
        "datum.library.set_part_pad_map_entry",
        "datum.library.set_part_pad_map",
        "datum.library.create_pin_pad_map",
        "datum.library.set_pin_pad_map",
        "datum.library.set_object",
        "datum.library.delete_object",
    ),
}


def _tool_prefix(name: str) -> str:
    return ".".join(name.split(".")[:2])


def _spec_from_verb(verb: dict[str, object]) -> dict[str, object]:
    spec: dict[str, object] = {
        "name": verb["name"],
        "description": verb["description"],
        "inputSchema": verb["inputSchema"],
    }
    dispatch = verb.get("dispatch") or {}
    method = dispatch.get("method") if isinstance(dispatch, dict) else None
    if method:
        spec["x_dispatch_method"] = method
    if verb.get("dispatch_args"):
        spec["x_dispatch_args"] = list(verb["dispatch_args"])
    if verb.get("dispatch_defaults"):
        spec["x_dispatch_defaults"] = dict(verb["dispatch_defaults"])
    write_surface = verb.get("write_surface")
    if write_surface:
        spec["x_public_write_surface_class"] = write_surface["class"]
        spec["x_write_surface_evidence"] = write_surface["evidence"]
    public_metadata = verb.get("public_metadata")
    if isinstance(public_metadata, dict):
        spec.update(public_metadata)
    return spec


def _load_generated_specs() -> list[dict[str, object]]:
    catalog = json.loads(_CATALOG_PATH.read_text(encoding="utf-8"))
    specs: list[dict[str, object]] = []
    seen: set[str] = set()
    for verb in catalog["verbs"]:
        name = str(verb["name"])
        if name in seen:
            raise RuntimeError(f"duplicate tool name in generated catalog: {name}")
        seen.add(name)
        if _tool_prefix(name) not in MIGRATED_PREFIXES:
            # Registry-declared but not yet MCP-migrated (e.g. GUI-terminal-only
            # families); the hand-written catalog still owns this prefix.
            continue
        if verb.get("status") != "public":
            raise RuntimeError(
                f"generated catalog verb {name} has unsupported status "
                f"{verb.get('status')!r}; hidden/retired generated verbs are not wired yet"
            )
        specs.append(_spec_from_verb(verb))
    return specs


GENERATED_TOOL_SPECS: list[dict[str, object]] = _load_generated_specs()
GENERATED_TOOL_NAMES: frozenset[str] = frozenset(
    str(spec["name"]) for spec in GENERATED_TOOL_SPECS
)


def generated_specs_for_prefix(prefix: str) -> list[dict[str, object]]:
    """Tool specs for one migrated verb-family prefix in public catalog order."""
    if prefix not in MIGRATED_PREFIXES:
        raise RuntimeError(f"prefix {prefix} is not migrated to the generated catalog")
    specs = [
        spec for spec in GENERATED_TOOL_SPECS if _tool_prefix(str(spec["name"])) == prefix
    ]
    order = _PREFIX_ORDERS.get(prefix)
    if order is None:
        return specs
    by_name = {str(spec["name"]): spec for spec in specs}
    if set(by_name) != set(order):
        missing = sorted(set(order) - set(by_name))
        extra = sorted(set(by_name) - set(order))
        raise RuntimeError(
            f"generated order for {prefix} does not match specs; "
            f"missing={missing}, extra={extra}"
        )
    return [by_name[name] for name in order]


def reject_hand_written_duplicates(hand_written_names: list[str]) -> None:
    """Raise at import if a hand-written spec collides with a generated one."""
    duplicates = sorted(
        name for name in hand_written_names if name in GENERATED_TOOL_NAMES
    )
    if duplicates:
        raise RuntimeError(
            "tool names defined both hand-written and in the generated "
            "verb-registry catalog: " + ", ".join(duplicates)
        )
