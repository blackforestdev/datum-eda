"""Tool-name to daemon-method dispatch for tools/call."""

from __future__ import annotations

from typing import Any


def dispatch_tool_call(daemon: Any, name: str, arguments: dict[str, Any]) -> Any:
    if name == "open_project":
        return daemon.open_project(arguments["path"])
    if name == "close_project":
        return daemon.close_project()
    if name == "save":
        return daemon.save(arguments.get("path"))
    if name == "delete_track":
        return daemon.delete_track(arguments["uuid"])
    if name == "delete_component":
        return daemon.delete_component(arguments["uuid"])
    if name == "delete_via":
        return daemon.delete_via(arguments["uuid"])
    if name == "move_component":
        return daemon.move_component(
            arguments["uuid"],
            arguments["x_mm"],
            arguments["y_mm"],
            arguments.get("rotation_deg"),
        )
    if name == "rotate_component":
        return daemon.rotate_component(arguments["uuid"], arguments["rotation_deg"])
    if name == "set_value":
        return daemon.set_value(arguments["uuid"], arguments["value"])
    if name == "assign_part":
        return daemon.assign_part(arguments["uuid"], arguments["part_uuid"])
    if name == "set_package":
        return daemon.set_package(arguments["uuid"], arguments["package_uuid"])
    if name == "set_reference":
        return daemon.set_reference(arguments["uuid"], arguments["reference"])
    if name == "set_net_class":
        return daemon.set_net_class(
            arguments["net_uuid"],
            arguments["class_name"],
            arguments["clearance"],
            arguments["track_width"],
            arguments["via_drill"],
            arguments["via_diameter"],
            arguments.get("diffpair_width", 0),
            arguments.get("diffpair_gap", 0),
        )
    if name == "set_design_rule":
        return daemon.set_design_rule(
            arguments["rule_type"],
            arguments["scope"],
            arguments["parameters"],
            arguments["priority"],
            arguments.get("name"),
        )
    if name == "undo":
        return daemon.undo()
    if name == "redo":
        return daemon.redo()
    if name == "search_pool":
        return daemon.search_pool(arguments["query"])
    if name == "get_part":
        return daemon.get_part(arguments["uuid"])
    if name == "get_package":
        return daemon.get_package(arguments["uuid"])
    if name == "get_components":
        return daemon.get_components()
    if name == "get_netlist":
        return daemon.get_netlist()
    if name == "get_board_summary":
        return daemon.get_board_summary()
    if name == "get_schematic_summary":
        return daemon.get_schematic_summary()
    if name == "get_sheets":
        return daemon.get_sheets()
    if name == "get_labels":
        return daemon.get_labels()
    if name == "get_symbols":
        return daemon.get_symbols()
    if name == "get_symbol_fields":
        return daemon.get_symbol_fields(arguments["symbol_uuid"])
    if name == "get_ports":
        return daemon.get_ports()
    if name == "get_buses":
        return daemon.get_buses()
    if name == "get_bus_entries":
        return daemon.get_bus_entries()
    if name == "get_noconnects":
        return daemon.get_noconnects()
    if name == "get_hierarchy":
        return daemon.get_hierarchy()
    if name == "get_net_info":
        return daemon.get_net_info()
    if name == "get_unrouted":
        return daemon.get_unrouted()
    if name == "get_schematic_net_info":
        return daemon.get_schematic_net_info()
    if name == "get_check_report":
        return daemon.get_check_report()
    if name == "get_connectivity_diagnostics":
        return daemon.get_connectivity_diagnostics()
    if name == "get_design_rules":
        return daemon.get_design_rules()
    if name == "run_erc":
        return daemon.run_erc()
    if name == "run_drc":
        return daemon.run_drc()
    if name == "explain_violation":
        return daemon.explain_violation(arguments["domain"], arguments["index"])
    raise RuntimeError(f"unknown tool: {name}")
