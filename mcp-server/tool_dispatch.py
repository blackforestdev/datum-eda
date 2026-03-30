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
    if name == "set_package_with_part":
        return daemon.set_package_with_part(
            arguments["uuid"], arguments["package_uuid"], arguments["part_uuid"]
        )
    if name == "replace_component":
        return daemon.replace_component(
            arguments["uuid"], arguments["package_uuid"], arguments["part_uuid"]
        )
    if name == "replace_components":
        return daemon.replace_components(arguments["replacements"])
    if name == "apply_component_replacement_plan":
        return daemon.apply_component_replacement_plan(arguments["replacements"])
    if name == "apply_component_replacement_policy":
        return daemon.apply_component_replacement_policy(arguments["replacements"])
    if name == "apply_scoped_component_replacement_policy":
        return daemon.apply_scoped_component_replacement_policy(
            arguments["scope"], arguments["policy"]
        )
    if name == "apply_scoped_component_replacement_plan":
        return daemon.apply_scoped_component_replacement_plan(arguments["plan"])
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
    if name == "get_package_change_candidates":
        return daemon.get_package_change_candidates(arguments["uuid"])
    if name == "get_part_change_candidates":
        return daemon.get_part_change_candidates(arguments["uuid"])
    if name == "get_component_replacement_plan":
        return daemon.get_component_replacement_plan(arguments["uuid"])
    if name == "get_scoped_component_replacement_plan":
        return daemon.get_scoped_component_replacement_plan(
            arguments["scope"], arguments["policy"]
        )
    if name == "edit_scoped_component_replacement_plan":
        return daemon.edit_scoped_component_replacement_plan(
            arguments["plan"],
            arguments.get("exclude_component_uuids", []),
            arguments.get("overrides", []),
        )
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
    if name == "export_route_path_proposal":
        return daemon.export_route_path_proposal(
            arguments["path"],
            arguments["net_uuid"],
            arguments["from_anchor_pad_uuid"],
            arguments["to_anchor_pad_uuid"],
            arguments["candidate"],
            arguments.get("policy"),
            arguments["out"],
        )
    if name == "route_proposal":
        return daemon.route_proposal(
            arguments["path"],
            arguments["net_uuid"],
            arguments["from_anchor_pad_uuid"],
            arguments["to_anchor_pad_uuid"],
        )
    if name == "route_proposal_explain":
        return daemon.route_proposal_explain(
            arguments["path"],
            arguments["net_uuid"],
            arguments["from_anchor_pad_uuid"],
            arguments["to_anchor_pad_uuid"],
        )
    if name == "export_route_proposal":
        return daemon.export_route_proposal(
            arguments["path"],
            arguments["net_uuid"],
            arguments["from_anchor_pad_uuid"],
            arguments["to_anchor_pad_uuid"],
            arguments["out"],
        )
    if name == "route_apply":
        return daemon.route_apply(
            arguments["path"],
            arguments["net_uuid"],
            arguments["from_anchor_pad_uuid"],
            arguments["to_anchor_pad_uuid"],
            arguments["candidate"],
            arguments.get("policy"),
        )
    if name == "route_apply_selected":
        return daemon.route_apply_selected(
            arguments["path"],
            arguments["net_uuid"],
            arguments["from_anchor_pad_uuid"],
            arguments["to_anchor_pad_uuid"],
        )
    if name == "inspect_route_proposal_artifact":
        return daemon.inspect_route_proposal_artifact(arguments["artifact"])
    if name == "apply_route_proposal_artifact":
        return daemon.apply_route_proposal_artifact(
            arguments["path"],
            arguments["artifact"],
        )
    raise RuntimeError(f"unknown tool: {name}")
