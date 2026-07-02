from __future__ import annotations

from tools_catalog_legacy_aliases import _LEGACY_CANONICAL_ALIAS_NAMES

HIDDEN_COMPATIBILITY_REPLACEMENTS: dict[str, list[str]] = {
    "move_component": ["datum.pcb.move_component"],
    "rotate_component": ["datum.pcb.rotate_component"],
    "flip_component": ["datum.pcb.flip_component"],
    "set_value": ["datum.pcb.set_component_value"],
    "set_reference": ["datum.pcb.set_component_reference"],
    "assign_part": ["datum.pcb.set_component_part"],
    "set_package": ["datum.pcb.set_component_package"],
    "set_package_with_part": [
        "datum.pcb.set_component_package",
        "datum.pcb.set_component_part",
    ],
    "replace_component": [
        "datum.proposal.create_board_component_replacement",
        "datum.proposal.accept_apply",
    ],
    "replace_components": [
        "datum.proposal.create_board_component_replacements",
        "datum.proposal.accept_apply",
    ],
    "apply_component_replacement_plan": [
        "datum.proposal.create_board_component_replacement_plan",
        "datum.proposal.accept_apply",
    ],
    "apply_component_replacement_policy": [
        "pending:journaled_component_replacement_policy_apply"
    ],
    "apply_scoped_component_replacement_policy": [
        "pending:journaled_scoped_component_replacement_policy_apply"
    ],
    "apply_scoped_component_replacement_plan": [
        "pending:journaled_scoped_component_replacement_plan_apply"
    ],
    "set_net_class": ["pending:journaled_net_class_assignment"],
    "get_check_report": ["datum.check.run"],
    "run_erc": ["datum.check.run"],
    "run_drc": ["datum.check.run"],
    "explain_violation": ["datum.check.explain_violation"],
    "undo": ["datum.journal.undo"],
    "redo": ["datum.journal.redo"],
    "create_panel_projection_proposal": ["datum.proposal.create_panel_projection"],
    "update_panel_projection_proposal": ["datum.proposal.update_panel_projection"],
    "delete_panel_projection_proposal": ["datum.proposal.delete_panel_projection"],
    "create_manufacturing_plan_proposal": ["datum.proposal.create_manufacturing_plan"],
    "update_manufacturing_plan_proposal": ["datum.proposal.update_manufacturing_plan"],
    "delete_manufacturing_plan_proposal": ["datum.proposal.delete_manufacturing_plan"],
    "create_panel_projection": ["datum.manufacturing.create_panel_projection"],
    "update_panel_projection": ["datum.manufacturing.update_panel_projection"],
    "delete_panel_projection": ["datum.manufacturing.delete_panel_projection"],
    "create_manufacturing_plan": ["datum.manufacturing.create_plan"],
    "update_manufacturing_plan": ["datum.manufacturing.update_plan"],
    "delete_manufacturing_plan": ["datum.manufacturing.delete_plan"],
    "create_gerber_output_job": ["datum.output_job.create_gerber_set"],
    "create_output_job": ["datum.output_job.create"],
    "create_output_job_proposal": ["datum.proposal.create_output_job"],
    "update_output_job": ["datum.output_job.update"],
    "update_output_job_proposal": ["datum.proposal.update_output_job"],
    "run_output_job": ["datum.output_job.run"],
    "delete_output_job": ["datum.output_job.delete"],
    "delete_output_job_proposal": ["datum.proposal.delete_output_job"],
}

DEFAULT_HIDDEN_ALIAS_RETIREMENT_CRITERIA = (
    "Remove after the canonical datum.* replacement is public, journal/proposal "
    "backed where it mutates source, documented in MCP_API_SPEC.md, and current "
    "client compatibility requirements no longer require the alias."
)

CHECK_ALIAS_REPLACEMENTS: dict[str, str] = {
    "get_check_run": "datum.check.run_profile",
    "get_check_runs": "datum.check.list",
    "show_check_run": "datum.check.show",
    "get_check_profiles": "datum.check.profiles",
    "fill_zones": "datum.check.fill_zones",
    "generate_standards_repair_proposals": "datum.check.repair_standards",
    "waive_finding": "datum.check.waive",
    "accept_deviation": "datum.check.accept_deviation",
}

ARTIFACT_ALIAS_REPLACEMENTS: dict[str, str] = {
    "generate_artifacts": "datum.artifact.generate",
    "get_artifacts": "datum.artifact.list",
    "show_artifact": "datum.artifact.show",
    "get_artifact_files": "datum.artifact.files",
    "preview_artifact_file": "datum.artifact.preview",
    "compare_artifacts": "datum.artifact.compare",
    "validate_artifact": "datum.artifact.validate",
    "start_output_job_run": "datum.artifact.start_output_job_run",
    "cancel_output_job_run": "datum.artifact.cancel_output_job_run",
    "export_manufacturing_set": "datum.artifact.export_manufacturing_set",
    "validate_manufacturing_set": "datum.artifact.validate_manufacturing_set",
}

COMPONENT_INSTANCE_ALIAS_REPLACEMENTS: dict[str, str] = {
    "bind_component_instance": "datum.component_instance.bind",
    "set_component_instance": "datum.component_instance.set",
    "delete_component_instance": "datum.component_instance.delete",
}

POOL_ALIAS_REPLACEMENTS: dict[str, str] = {
    "search_pool": "datum.pool.search",
    "get_part": "datum.pool.get_part",
    "get_package": "datum.pool.get_package",
}

QUERY_ALIAS_REPLACEMENTS: dict[str, str] = {
    "get_zone_fills": "datum.query.zone_fills",
    "get_panel_projections": "datum.query.panel_projections",
    "get_manufacturing_plans": "datum.query.manufacturing_plans",
    "get_output_jobs": "datum.query.output_jobs",
    "get_component_instances": "datum.query.component_instances",
    "get_relationships": "datum.query.relationships",
    "get_variants": "datum.query.variants",
    "get_import_map": "datum.query.import_map",
    "get_components": "datum.query.components",
    "get_netlist": "datum.query.netlist",
    "get_schematic_net_info": "datum.query.schematic_nets",
    "get_board_summary": "datum.query.board_summary",
    "get_schematic_summary": "datum.query.schematic_summary",
    "get_sheets": "datum.query.sheets",
    "get_labels": "datum.query.labels",
    "get_symbols": "datum.query.symbols",
    "get_symbol_fields": "datum.query.symbol_fields",
    "get_ports": "datum.query.ports",
    "get_buses": "datum.query.buses",
    "get_bus_entries": "datum.query.bus_entries",
    "get_noconnects": "datum.query.noconnects",
    "get_connectivity_diagnostics": "datum.query.connectivity_diagnostics",
    "get_design_rules": "datum.query.design_rules",
    "get_net_info": "datum.query.net_info",
    "get_unrouted": "datum.query.unrouted",
    "get_hierarchy": "datum.query.imported_hierarchy",
}

REPLACEMENT_ALIAS_REPLACEMENTS: dict[str, str] = {
    "get_package_change_candidates": "datum.replacement.package_candidates",
    "get_part_change_candidates": "datum.replacement.part_candidates",
    "get_component_replacement_plan": "datum.replacement.get_plan",
    "get_scoped_component_replacement_plan": "datum.replacement.get_scoped_plan",
    "edit_scoped_component_replacement_plan": "datum.replacement.edit_scoped_plan",
}

MANUFACTURING_ALIAS_REPLACEMENTS: dict[str, str] = {
    "create_panel_projection_proposal": "datum.proposal.create_panel_projection",
    "update_panel_projection_proposal": "datum.proposal.update_panel_projection",
    "delete_panel_projection_proposal": "datum.proposal.delete_panel_projection",
    "create_manufacturing_plan_proposal": "datum.proposal.create_manufacturing_plan",
    "update_manufacturing_plan_proposal": "datum.proposal.update_manufacturing_plan",
    "delete_manufacturing_plan_proposal": "datum.proposal.delete_manufacturing_plan",
    "create_panel_projection": "datum.manufacturing.create_panel_projection",
    "update_panel_projection": "datum.manufacturing.update_panel_projection",
    "delete_panel_projection": "datum.manufacturing.delete_panel_projection",
    "create_manufacturing_plan": "datum.manufacturing.create_plan",
    "update_manufacturing_plan": "datum.manufacturing.update_plan",
    "delete_manufacturing_plan": "datum.manufacturing.delete_plan",
}

OUTPUT_JOB_ALIAS_REPLACEMENTS: dict[str, str] = {
    "create_output_job_proposal": "datum.proposal.create_output_job",
    "update_output_job_proposal": "datum.proposal.update_output_job",
    "delete_output_job_proposal": "datum.proposal.delete_output_job",
    "create_gerber_output_job": "datum.output_job.create_gerber_set",
    "create_output_job": "datum.output_job.create",
    "update_output_job": "datum.output_job.update",
    "run_output_job": "datum.output_job.run",
    "delete_output_job": "datum.output_job.delete",
}

ROUTE_ALIAS_REPLACEMENTS: dict[str, str] = {
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

PROPOSAL_ALIAS_REPLACEMENTS: dict[str, str] = {
    "create_proposal": "datum.proposal.create",
    "create_draw_wire_proposal": "datum.proposal.create_draw_wire",
    "create_place_label_proposal": "datum.proposal.create_place_label",
    "create_place_symbol_proposal": "datum.proposal.create_place_symbol",
    "create_board_component_replacement_proposal": "datum.proposal.create_board_component_replacement",
    "create_board_component_replacements_proposal": "datum.proposal.create_board_component_replacements",
    "create_board_component_replacement_plan_proposal": "datum.proposal.create_board_component_replacement_plan",
    "get_proposals": "datum.proposal.list",
    "show_proposal": "datum.proposal.show",
    "preview_proposal": "datum.proposal.preview",
    "validate_proposal": "datum.proposal.validate",
    "defer_proposal": "datum.proposal.defer",
    "reject_proposal": "datum.proposal.reject",
    "review_proposal": "datum.proposal.review",
    "accept_apply_proposal": "datum.proposal.accept_apply",
    "apply_proposal": "datum.proposal.apply",
}

PCB_ALIAS_REPLACEMENTS: dict[str, str] = {
}

LIBRARY_ALIAS_REPLACEMENTS: dict[str, str] = {
    "get_pool_library_objects": "datum.library.list_objects",
    "show_pool_library_object": "datum.library.show_object",
    "get_pool_model_blobs": "datum.library.pool_models",
    "gc_pool_model_blobs": "datum.library.gc_pool_models",
    "create_pool_library_object": "datum.library.create_object",
    "delete_pool_library_object": "datum.library.delete_object",
    "create_pool_unit": "datum.library.create_unit",
    "set_pool_unit_pin": "datum.library.set_unit_pin",
    "create_pool_symbol": "datum.library.create_symbol",
    "add_pool_symbol_line": "datum.library.add_symbol_line",
    "add_pool_symbol_rect": "datum.library.add_symbol_rect",
    "add_pool_symbol_circle": "datum.library.add_symbol_circle",
    "add_pool_symbol_arc": "datum.library.add_symbol_arc",
    "add_pool_symbol_polygon": "datum.library.add_symbol_polygon",
    "add_pool_symbol_text": "datum.library.add_symbol_text",
    "set_pool_symbol_pin_anchor": "datum.library.set_symbol_pin_anchor",
    "create_pool_entity": "datum.library.create_entity",
    "create_pool_padstack": "datum.library.create_padstack",
    "create_pool_package": "datum.library.create_package",
    "set_pool_package_pad": "datum.library.set_package_pad",
    "set_pool_package_courtyard_rect": "datum.library.set_package_courtyard_rect",
    "set_pool_package_courtyard_polygon": "datum.library.set_package_courtyard_polygon",
    "add_pool_package_silkscreen_line": "datum.library.add_package_silkscreen_line",
    "add_pool_package_silkscreen_rect": "datum.library.add_package_silkscreen_rect",
    "add_pool_package_silkscreen_polygon": "datum.library.add_package_silkscreen_polygon",
    "add_pool_package_silkscreen_circle": "datum.library.add_package_silkscreen_circle",
    "add_pool_package_silkscreen_arc": "datum.library.add_package_silkscreen_arc",
    "add_pool_package_silkscreen_text": "datum.library.add_package_silkscreen_text",
    "add_pool_package_model_3d": "datum.library.add_package_model_3d",
    "set_pool_package_body_heights": "datum.library.set_package_body_heights",
    "create_pool_part": "datum.library.create_part",
    "set_pool_part_metadata": "datum.library.set_part_metadata",
    "set_pool_part_parametric": "datum.library.set_part_parametric",
    "set_pool_part_orderable_mpns": "datum.library.set_part_orderable_mpns",
    "set_pool_part_tags": "datum.library.set_part_tags",
    "set_pool_part_packaging_options": "datum.library.set_part_packaging_options",
    "set_pool_part_supply_chain": "datum.library.set_part_supply_chain",
    "set_pool_part_behavioural_models": "datum.library.set_part_behavioural_models",
    "attach_pool_part_model": "datum.library.attach_part_model",
    "detach_pool_part_model": "datum.library.detach_part_model",
    "set_pool_part_thermal": "datum.library.set_part_thermal",
    "set_pool_part_pad_map_entry": "datum.library.set_part_pad_map_entry",
    "set_pool_part_pad_map": "datum.library.set_part_pad_map",
    "set_pool_library_object": "datum.library.set_object",
}

SESSION_ALIAS_REPLACEMENTS: dict[str, str] = {
    "open_project": "datum.session.open",
    "close_project": "datum.session.close",
    "save": "datum.session.save",
    "validate_project": "datum.session.validate",
}

HIDDEN_COMPATIBILITY_RETIREMENT_OVERRIDES: dict[str, dict[str, str]] = {
    "get_check_report": {
        "status": "deprecated",
        "criteria": (
            "Deprecated: use datum.check.run for canonical check_run_v1 evidence; "
            "remove after supported clients consume canonical check envelopes."
        ),
    },
    "run_erc": {
        "status": "deprecated",
        "criteria": (
            "Deprecated: use datum.check.run with the ERC profile; remove after "
            "supported clients consume canonical check envelopes."
        ),
    },
    "run_drc": {
        "status": "deprecated",
        "criteria": (
            "Deprecated: use datum.check.run with the DRC profile; remove after "
            "supported clients consume canonical check envelopes."
        ),
    },
    "explain_violation": {
        "status": "deprecated",
        "criteria": (
            "Deprecated: use datum.check.explain_violation with a stable finding "
            "fingerprint; legacy index fallback remains only for compatibility."
        ),
    },
    "get_journal_list": {
        "status": "deprecated",
        "criteria": (
            "Deprecated: use datum.journal.list for project-wide transaction "
            "history; remove after supported clients consume canonical journal "
            "envelopes."
        ),
    },
    "get_journal_transaction": {
        "status": "deprecated",
        "criteria": (
            "Deprecated: use datum.journal.show for transaction detail; remove "
            "after supported clients consume canonical journal envelopes."
        ),
    },
    "journal_undo": {
        "status": "deprecated",
        "criteria": (
            "Deprecated: use datum.journal.undo for compensating project commits; "
            "remove after supported clients stop calling the flat alias."
        ),
    },
    "journal_redo": {
        "status": "deprecated",
        "criteria": (
            "Deprecated: use datum.journal.redo for compensating project commits; "
            "remove after supported clients stop calling the flat alias."
        ),
    },
    "undo": {
        "status": "deprecated",
        "criteria": (
            "Deprecated: use datum.journal.undo for compensating project commits; "
            "remove after supported clients stop calling the session alias."
        ),
    },
    "redo": {
        "status": "deprecated",
        "criteria": (
            "Deprecated: use datum.journal.redo for compensating project commits; "
            "remove after supported clients stop calling the session alias."
        ),
    },
    **{
        name: {
            "status": "deprecated",
            "criteria": (
                f"Deprecated: use {replacement} for canonical check, repair, "
                "waiver, deviation, or ZoneFill evidence workflows; remove after "
                "supported clients consume datum.check.* envelopes."
            ),
        }
        for name, replacement in CHECK_ALIAS_REPLACEMENTS.items()
    },
    **{
        name: {
            "status": "deprecated",
            "criteria": (
                f"Deprecated: use {replacement} for canonical artifact/output "
                "evidence access; remove after supported clients consume the "
                "datum.artifact.* envelope."
            ),
        }
        for name, replacement in ARTIFACT_ALIAS_REPLACEMENTS.items()
    },
    **{
        name: {
            "status": "deprecated",
            "criteria": (
                f"Deprecated: use {replacement} for canonical ComponentInstance "
                "authoring; remove after supported clients stop calling the flat "
                "component-instance compatibility alias."
            ),
        }
        for name, replacement in COMPONENT_INSTANCE_ALIAS_REPLACEMENTS.items()
    },
    **{
        name: {
            "status": "deprecated",
            "criteria": (
                f"Deprecated: use {replacement} for canonical pool/library lookup; "
                "remove after supported clients consume datum.pool.* query envelopes."
            ),
        }
        for name, replacement in POOL_ALIAS_REPLACEMENTS.items()
    },
    **{
        name: {
            "status": "deprecated",
            "criteria": (
                f"Deprecated: use {replacement} for canonical read-only model "
                "queries; remove after supported clients consume datum.query.* "
                "envelopes."
            ),
        }
        for name, replacement in QUERY_ALIAS_REPLACEMENTS.items()
    },
    **{
        name: {
            "status": "deprecated",
            "criteria": (
                f"Deprecated: use {replacement} for canonical replacement planning; "
                "remove after supported clients consume datum.replacement.* "
                "planning envelopes."
            ),
        }
        for name, replacement in REPLACEMENT_ALIAS_REPLACEMENTS.items()
    },
    **{
        name: {
            "status": "deprecated",
            "criteria": (
                f"Deprecated: use {replacement} for canonical manufacturing "
                "proposal authoring; remove after supported clients consume "
                "datum.manufacturing.* or datum.proposal.* envelopes."
            ),
        }
        for name, replacement in MANUFACTURING_ALIAS_REPLACEMENTS.items()
    },
    **{
        name: {
            "status": "deprecated",
            "criteria": (
                f"Deprecated: use {replacement} for canonical OutputJob "
                "authoring or execution; remove after supported clients consume "
                "datum.output_job.* or datum.proposal.* envelopes."
            ),
        }
        for name, replacement in OUTPUT_JOB_ALIAS_REPLACEMENTS.items()
    },
    **{
        name: {
            "status": "deprecated",
            "criteria": (
                f"Deprecated: use {replacement} for canonical route proposal, "
                "strategy, fixture, artifact, or journaled apply workflows; "
                "remove after supported clients consume datum.route.* envelopes."
            ),
        }
        for name, replacement in ROUTE_ALIAS_REPLACEMENTS.items()
    },
    **{
        name: {
            "status": "deprecated",
            "criteria": (
                f"Deprecated: use {replacement} for canonical proposal creation, "
                "review, preview, validation, or apply-gateway workflows; remove "
                "after supported clients consume datum.proposal.* envelopes."
            ),
        }
        for name, replacement in PROPOSAL_ALIAS_REPLACEMENTS.items()
    },
    **{
        name: {
            "status": "deprecated",
            "criteria": (
                f"Deprecated: use {replacement} for canonical journaled PCB "
                "authoring; remove after supported clients consume datum.pcb.* "
                "envelopes."
            ),
        }
        for name, replacement in PCB_ALIAS_REPLACEMENTS.items()
    },
    **{
        name: {
            "status": "deprecated",
            "criteria": (
                f"Deprecated: use {replacement} for canonical library authoring "
                "or lookup; remove after supported clients consume "
                "datum.library.* envelopes and proposal policy where required."
            ),
        }
        for name, replacement in LIBRARY_ALIAS_REPLACEMENTS.items()
    },
    "open_project": {
        "status": "deprecated",
        "criteria": (
            "Deprecated: use datum.session.open for explicit session import/open "
            "compatibility; remove the flat alias after supported clients stop "
            "calling open_project."
        ),
    },
    "close_project": {
        "status": "deprecated",
        "criteria": (
            "Deprecated: use datum.session.close for explicit session lifecycle "
            "compatibility; remove the flat alias after supported clients stop "
            "calling close_project."
        ),
    },
    "save": {
        "status": "deprecated",
        "criteria": (
            "Deprecated: use datum.session.save only for legacy imported-design "
            "write-back compatibility; target authored writes commit at apply "
            "time and no new public save mutation should be introduced."
        ),
    },
    "validate_project": {
        "status": "deprecated",
        "criteria": (
            "Deprecated: use datum.session.validate for resolver-backed project "
            "validation compatibility; remove the flat alias after supported "
            "clients stop calling validate_project."
        ),
    },
}


def canonical_replacements_for_hidden_alias(
    annotated: dict[str, object],
    public_replacements_by_dispatch: dict[str, str],
) -> list[str]:
    name = str(annotated["name"])
    dispatch_method = str(annotated.get("x_dispatch_method", name))
    explicit = HIDDEN_COMPATIBILITY_REPLACEMENTS.get(dispatch_method)
    if explicit:
        return explicit
    explicit = HIDDEN_COMPATIBILITY_REPLACEMENTS.get(name)
    if explicit:
        return explicit
    if dispatch_method in public_replacements_by_dispatch:
        return [public_replacements_by_dispatch[dispatch_method]]
    if name in _LEGACY_CANONICAL_ALIAS_NAMES:
        return [_LEGACY_CANONICAL_ALIAS_NAMES[name]]
    return []


def retirement_override_for_hidden_alias(
    annotated: dict[str, object],
) -> dict[str, str]:
    name = str(annotated["name"])
    dispatch_method = str(annotated.get("x_dispatch_method", name))
    return (
        HIDDEN_COMPATIBILITY_RETIREMENT_OVERRIDES.get(dispatch_method)
        or HIDDEN_COMPATIBILITY_RETIREMENT_OVERRIDES.get(name)
        or {}
    )
