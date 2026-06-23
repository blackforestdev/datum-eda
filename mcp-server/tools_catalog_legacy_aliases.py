"""Legacy flat -> canonical datum.* alias name map.

Extracted from tools_catalog_data.py to keep that module within its file-size
budget. Maps each remaining legacy flat MCP tool name to its canonical
datum.<group>.<verb> alias; the canonical alias dispatches to the same daemon
method as its flat counterpart (x_dispatch_method = flat name). The flat tool
is moved to compatibility-only dispatch so it stays callable but is hidden from
the public tools/list.
"""
from __future__ import annotations

_LEGACY_CANONICAL_ALIAS_NAMES: dict[str, str] = {
    "open_project": "datum.session.open",
    "close_project": "datum.session.close",
    "save": "datum.session.save",
    "validate_project": "datum.session.validate",
    "delete_track": "datum.board.delete_track",
    "delete_component": "datum.board.delete_component",
    "move_component": "datum.board.move_component",
    "rotate_component": "datum.board.rotate_component",
    "flip_component": "datum.board.flip_component",
    "set_value": "datum.board.set_component_value",
    "assign_part": "datum.board.assign_component_part",
    "set_package": "datum.board.set_component_package",
    "set_package_with_part": "datum.board.set_component_package_with_part",
    "replace_component": "datum.board.replace_component",
    "replace_components": "datum.board.replace_components",
    "set_reference": "datum.board.set_component_reference",
    "set_net_class": "datum.board.set_net_class",
    "delete_via": "datum.board.delete_via",
    "set_design_rule": "datum.board.set_design_rule",
    "apply_component_replacement_plan": "datum.replacement.apply_plan",
    "apply_component_replacement_policy": "datum.replacement.apply_policy",
    "apply_scoped_component_replacement_policy": "datum.replacement.apply_scoped_policy",
    "apply_scoped_component_replacement_plan": "datum.replacement.apply_scoped_plan",
    "get_component_replacement_plan": "datum.replacement.get_plan",
    "get_scoped_component_replacement_plan": "datum.replacement.get_scoped_plan",
    "edit_scoped_component_replacement_plan": "datum.replacement.edit_scoped_plan",
    "get_package_change_candidates": "datum.replacement.package_candidates",
    "get_part_change_candidates": "datum.replacement.part_candidates",
    "search_pool": "datum.pool.search",
    "get_part": "datum.pool.get_part",
    "get_package": "datum.pool.get_package",
    "get_net_info": "datum.query.net_info",
    "get_unrouted": "datum.query.unrouted",
    "get_hierarchy": "datum.query.imported_hierarchy",
    "export_route_path_proposal": "datum.route.export_path_proposal",
    "route_proposal": "datum.route.select_proposal",
    "review_route_proposal": "datum.route.review_proposal",
    "route_strategy_report": "datum.route.strategy_report",
    "route_strategy_compare": "datum.route.strategy_compare",
    "route_strategy_delta": "datum.route.strategy_delta",
    "write_route_strategy_curated_fixture_suite": "datum.route.write_strategy_fixture_suite",
    "capture_route_strategy_curated_baseline": "datum.route.capture_strategy_baseline",
    "route_strategy_batch_evaluate": "datum.route.strategy_batch_evaluate",
    "inspect_route_strategy_batch_result": "datum.route.inspect_strategy_batch_result",
    "validate_route_strategy_batch_result": "datum.route.validate_strategy_batch_result",
    "compare_route_strategy_batch_result": "datum.route.compare_strategy_batch_result",
    "gate_route_strategy_batch_result": "datum.route.gate_strategy_batch_result",
    "summarize_route_strategy_batch_results": "datum.route.summarize_strategy_batch_results",
    "route_proposal_explain": "datum.route.explain_proposal",
    "export_route_proposal": "datum.route.export_proposal",
    "route_apply": "datum.route.apply",
    "route_apply_selected": "datum.route.apply_selected",
    "inspect_route_proposal_artifact": "datum.route.inspect_proposal_artifact",
    "revalidate_route_proposal_artifact": "datum.route.revalidate_proposal_artifact",
    "apply_route_proposal_artifact": "datum.route.apply_proposal_artifact",
}
