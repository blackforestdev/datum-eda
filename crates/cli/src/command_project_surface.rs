pub(crate) use super::*;

#[path = "command_project_artifact_runs.rs"]
mod command_project_artifact_runs;
#[path = "command_project_artifact_validation.rs"]
mod command_project_artifact_validation;
#[path = "command_project_artifacts.rs"]
mod command_project_artifacts;
#[path = "command_project_board_handoff.rs"]
mod command_project_board_handoff;
#[path = "command_project_check_gate.rs"]
mod command_project_check_gate;
#[path = "command_project_component_instances.rs"]
mod command_project_component_instances;
#[path = "command_project_import_map.rs"]
mod command_project_import_map;
#[path = "command_project_imports.rs"]
mod command_project_imports;
#[path = "command_project_imports_eagle_import_map.rs"]
mod command_project_imports_eagle_import_map;
#[path = "command_project_imports_kicad_footprint.rs"]
mod command_project_imports_kicad_footprint;
#[path = "command_project_imports_schematic.rs"]
mod command_project_imports_schematic;
#[path = "command_project_imports_schematic_identities.rs"]
mod command_project_imports_schematic_identities;
#[path = "command_project_library.rs"]
mod command_project_library;
#[path = "command_project_library_footprint.rs"]
mod command_project_library_footprint;
#[path = "command_project_library_footprint_proposals.rs"]
mod command_project_library_footprint_proposals;
#[path = "command_project_library_package.rs"]
mod command_project_library_package;
#[path = "command_project_library_package_geometry.rs"]
mod command_project_library_package_geometry;
#[path = "command_project_library_package_geometry_proposals.rs"]
mod command_project_library_package_geometry_proposals;
#[path = "command_project_library_package_pad.rs"]
mod command_project_library_package_pad;
#[path = "command_project_library_package_proposals.rs"]
mod command_project_library_package_proposals;
#[path = "command_project_library_pad_map.rs"]
mod command_project_library_pad_map;
#[path = "command_project_library_part_bindings.rs"]
mod command_project_library_part_bindings;
#[path = "command_project_library_payload.rs"]
mod command_project_library_payload;
#[path = "command_project_library_pin_pad_map.rs"]
mod command_project_library_pin_pad_map;
#[path = "command_project_library_proposals.rs"]
mod command_project_library_proposals;
#[path = "command_project_library_symbol_geometry.rs"]
mod command_project_library_symbol_geometry;
#[path = "command_project_library_unit_pin.rs"]
mod command_project_library_unit_pin;
#[path = "command_project_manufacturing_plan_proposals.rs"]
mod command_project_manufacturing_plan_proposals;
#[path = "command_project_manufacturing_plans.rs"]
mod command_project_manufacturing_plans;
#[path = "command_project_native_journal_mutation.rs"]
mod command_project_native_journal_mutation;
#[path = "command_project_output_job_include.rs"]
mod command_project_output_job_include;
#[path = "command_project_output_job_proposals.rs"]
mod command_project_output_job_proposals;
#[path = "command_project_output_jobs.rs"]
mod command_project_output_jobs;
#[path = "command_project_proposals.rs"]
mod command_project_proposals;
#[path = "command_project_relationships.rs"]
mod command_project_relationships;
#[path = "command_project_standards_clearance_repairs.rs"]
mod command_project_standards_clearance_repairs;
#[path = "command_project_standards_peer_aperture.rs"]
mod command_project_standards_peer_aperture;
#[path = "command_project_standards_repairs.rs"]
mod command_project_standards_repairs;
#[path = "command_project_standards_silk_repairs.rs"]
mod command_project_standards_silk_repairs;
#[path = "command_project_waivers.rs"]
mod command_project_waivers;

pub(crate) use self::command_project_artifact_validation::validate_native_project_artifact;
pub(crate) use self::command_project_artifacts::{
    compare_native_project_artifacts, generate_native_project_artifacts,
    preview_native_project_artifact_file, query_native_project_artifact,
    query_native_project_artifact_files, query_native_project_artifacts,
};
pub(crate) use self::command_project_board_component_layer::set_native_project_board_component_layer;
pub(crate) use self::command_project_board_component_mutations::{
    board_package_materialization_payload_for_component,
    current_board_component_materialization_payload, delete_native_project_board_component,
    move_native_project_board_component, place_native_project_board_component,
    rotate_native_project_board_component, set_native_project_board_component_locked,
    set_native_project_board_component_package, set_native_project_board_component_part,
};
pub(crate) use self::command_project_board_component_query::{
    query_native_project_board_component_mechanical,
    query_native_project_board_component_models_3d, query_native_project_board_component_pads,
    query_native_project_board_component_silkscreen, query_native_project_board_component_view,
    query_native_project_board_component_views, query_native_project_board_components,
};
pub(crate) use self::command_project_board_component_reference::set_native_project_board_component_reference;
pub(crate) use self::command_project_board_component_value::set_native_project_board_component_value;
pub(crate) use self::command_project_board_diagnostics::{
    query_native_project_board_check, query_native_project_board_diagnostics,
    query_native_project_board_unrouted,
};
pub(crate) use self::command_project_board_handoff::{
    generate_native_project_board_components, render_native_project_board_handoff_text,
};
pub(crate) use self::command_project_board_layout::{
    delete_native_project_board_keepout, delete_native_project_board_text,
    edit_native_project_board_keepout, edit_native_project_board_text,
    place_native_project_board_keepout, place_native_project_board_text,
    query_native_project_board_keepouts, query_native_project_board_outline,
    query_native_project_board_stackup, query_native_project_board_texts,
    set_native_project_board_name, set_native_project_board_outline,
    set_native_project_board_stackup,
};
pub(crate) use self::command_project_board_netclass_dimension::{
    delete_native_project_board_dimension, delete_native_project_board_net_class,
    edit_native_project_board_dimension, edit_native_project_board_net_class,
    place_native_project_board_dimension, place_native_project_board_net_class,
    query_native_project_board_dimensions, query_native_project_board_net_class,
    query_native_project_board_net_classes,
};
pub(crate) use self::command_project_board_pad::{
    delete_native_project_board_pad, edit_native_project_board_pad, place_native_project_board_pad,
    query_native_project_board_pads, set_native_project_board_pad_net,
};
pub(crate) use self::command_project_board_routing_net::{
    delete_native_project_board_net, delete_native_project_board_track,
    delete_native_project_board_via, delete_native_project_board_zone,
    edit_native_project_board_net, edit_native_project_board_track, edit_native_project_board_via,
    edit_native_project_board_zone, fill_native_project_zones, place_native_project_board_net,
    place_native_project_board_track, place_native_project_board_via,
    place_native_project_board_zone, query_native_project_board_net,
    query_native_project_board_nets, query_native_project_board_tracks,
    query_native_project_board_vias, query_native_project_board_zones,
    query_native_project_zone_fills,
};
pub(crate) use self::command_project_component_instances::{
    bind_native_project_component_instance, delete_native_project_component_instance,
    query_native_project_component_instances, set_native_project_component_instance,
};
pub(crate) use self::command_project_default_stackup::add_native_project_default_top_stackup;
pub(crate) use self::command_project_default_stackup::default_native_project_stackup_layers;
pub(crate) use self::command_project_drill::{
    compare_native_project_drill, compare_native_project_excellon_drill,
    export_native_project_drill, export_native_project_excellon_drill,
    export_native_project_panel_drill, export_native_project_panel_excellon_drill,
    inspect_excellon_drill, inspect_native_project_drill, render_expected_native_project_drill_csv,
    render_expected_native_project_panel_drill_csv,
    render_expected_native_project_panel_excellon_drill, report_native_project_drill_hole_classes,
    validate_native_project_drill, validate_native_project_excellon_drill,
};
pub(crate) use self::command_project_forward_annotation_surface::*;
pub(crate) use self::command_project_gerber_inspect::inspect_gerber;
pub(crate) use self::command_project_gerber_layers::{
    compare_native_project_gerber_copper_layer, compare_native_project_gerber_outline,
    compare_native_project_gerber_paste_layer, compare_native_project_gerber_silkscreen_layer,
    compare_native_project_gerber_soldermask_layer, export_native_project_gerber_copper_layer,
    export_native_project_gerber_outline, export_native_project_gerber_paste_layer,
    export_native_project_gerber_silkscreen_layer, export_native_project_gerber_soldermask_layer,
    validate_native_project_gerber_copper_layer, validate_native_project_gerber_outline,
    validate_native_project_gerber_paste_layer, validate_native_project_gerber_silkscreen_layer,
    validate_native_project_gerber_soldermask_layer,
};
pub(crate) use self::command_project_gerber_mechanical::{
    NativeComponentMechanicalArc, NativeComponentMechanicalCircle, NativeComponentMechanicalLine,
    NativeComponentMechanicalPolygon, NativeComponentMechanicalPolyline,
    NativeComponentMechanicalText, compare_native_project_gerber_mechanical_layer,
    export_native_project_gerber_mechanical_layer, validate_native_project_gerber_mechanical_layer,
};
pub(crate) use self::command_project_gerber_plan::{
    compare_native_project_gerber_export_plan, compare_native_project_gerber_set,
    export_native_project_gerber_set, export_native_project_gerber_set_from_plan,
    export_native_project_gerber_set_without_output_run,
    export_native_project_gerber_set_without_output_run_for_output_job, panelize_rs274x_gerber,
    plan_native_project_gerber_export, validate_native_project_gerber_set,
};
pub(crate) use self::command_project_gerber_semantics::{
    DEFAULT_GERBER_OUTLINE_APERTURE_NM, ParsedGerber, ParsedGerberAperture, ParsedGerberGeometry,
    classify_via_hole_class, compare_entry_views, gerber_copper_actual_entries,
    gerber_copper_expected_entries, gerber_outline_actual_entries, gerber_outline_expected_entries,
    gerber_silkscreen_expected_entries, gerber_soldermask_actual_entries,
    gerber_soldermask_expected_entries, parse_rs274x_subset, render_circular_flash_geometry,
    render_mm_6, render_pad_flash_geometry, render_parsed_flash_geometry, render_region_geometry,
    render_stroke_geometry, resolve_native_project_soldermask_context,
};
pub(crate) use self::command_project_gerber_silkscreen::{
    NativeComponentSilkscreenArc, NativeComponentSilkscreenCircle, NativeComponentSilkscreenLine,
    NativeComponentSilkscreenPolygon, NativeComponentSilkscreenPolyline,
    NativeComponentSilkscreenText,
};
pub(crate) use self::command_project_gerber_silkscreen::{
    count_native_component_silkscreen_arcs, count_native_component_silkscreen_circles,
    count_native_component_silkscreen_lines, count_native_component_silkscreen_polygons,
    count_native_component_silkscreen_polylines, count_native_component_silkscreen_texts,
    resolve_native_project_silkscreen_context,
};
pub(crate) use self::command_project_import_map::query_native_project_import_map;
pub(crate) use self::command_project_imports::{
    import_native_project_eagle_library, import_native_project_kicad_board,
};
pub(crate) use self::command_project_imports_kicad_footprint::import_native_project_kicad_footprint;
pub(crate) use self::command_project_imports_schematic::import_native_project_kicad_schematic;
pub(crate) use self::command_project_inventory_surface::*;
pub(crate) use self::command_project_library::{
    attach_native_project_pool_part_model, create_native_project_pool_entity,
    create_native_project_pool_library_object, create_native_project_pool_padstack,
    create_native_project_pool_part, create_native_project_pool_symbol,
    create_native_project_pool_unit, delete_native_project_pool_library_object,
    detach_native_project_pool_part_model, gc_native_project_pool_models,
    set_native_project_pool_library_object, set_native_project_pool_part_behavioural_models,
    set_native_project_pool_part_metadata, set_native_project_pool_part_orderable_mpns,
    set_native_project_pool_part_packaging_options, set_native_project_pool_part_parametric,
    set_native_project_pool_part_supply_chain, set_native_project_pool_part_tags,
    set_native_project_pool_part_thermal,
};
pub(crate) use self::command_project_library_footprint::{
    add_native_project_pool_footprint_silkscreen_circle,
    add_native_project_pool_footprint_silkscreen_line,
    add_native_project_pool_footprint_silkscreen_polygon,
    add_native_project_pool_footprint_silkscreen_rect, create_native_project_pool_footprint,
    generate_native_project_ipc7351b_two_terminal_chip,
    set_native_project_pool_footprint_courtyard_polygon,
    set_native_project_pool_footprint_courtyard_rect, set_native_project_pool_footprint_pad,
};
pub(crate) use self::command_project_library_footprint_proposals::{
    propose_add_native_project_pool_footprint_silkscreen_circle,
    propose_add_native_project_pool_footprint_silkscreen_line,
    propose_add_native_project_pool_footprint_silkscreen_polygon,
    propose_add_native_project_pool_footprint_silkscreen_rect,
    propose_create_native_project_pool_footprint,
    propose_generate_native_project_ipc7351b_two_terminal_chip,
    propose_set_native_project_pool_footprint_courtyard_polygon,
    propose_set_native_project_pool_footprint_courtyard_rect,
    propose_set_native_project_pool_footprint_pad,
};
pub(crate) use self::command_project_library_package::create_native_project_pool_package;
pub(crate) use self::command_project_library_package_geometry::{
    add_native_project_pool_package_model_3d, add_native_project_pool_package_silkscreen_arc,
    add_native_project_pool_package_silkscreen_circle,
    add_native_project_pool_package_silkscreen_line,
    add_native_project_pool_package_silkscreen_polygon,
    add_native_project_pool_package_silkscreen_rect,
    add_native_project_pool_package_silkscreen_text, set_native_project_pool_package_body_heights,
    set_native_project_pool_package_courtyard_polygon,
    set_native_project_pool_package_courtyard_rect,
};
pub(crate) use self::command_project_library_package_geometry_proposals::{
    propose_set_native_project_pool_package_courtyard_polygon,
    propose_set_native_project_pool_package_courtyard_rect,
    propose_set_native_project_pool_package_pad,
};
pub(crate) use self::command_project_library_package_pad::set_native_project_pool_package_pad;
pub(crate) use self::command_project_library_package_proposals::propose_create_native_project_pool_package;
pub(crate) use self::command_project_library_pad_map::{
    set_native_project_pool_part_pad_map_entry, set_native_project_pool_part_pad_map_from_entries,
};
pub(crate) use self::command_project_library_part_bindings::set_native_project_pool_part_bindings;
pub(crate) use self::command_project_library_pin_pad_map::{
    create_native_project_pool_pin_pad_map, set_native_project_pool_pin_pad_map,
};
pub(crate) use self::command_project_library_proposals::{
    propose_create_native_project_pool_entity, propose_create_native_project_pool_library_object,
    propose_create_native_project_pool_padstack, propose_create_native_project_pool_pin_pad_map,
    propose_create_native_project_pool_symbol, propose_create_native_project_pool_unit,
    propose_set_native_project_pool_pin_pad_map,
};
pub(crate) use self::command_project_library_symbol_geometry::{
    add_native_project_pool_symbol_arc, add_native_project_pool_symbol_circle,
    add_native_project_pool_symbol_line, add_native_project_pool_symbol_polygon,
    add_native_project_pool_symbol_rect, add_native_project_pool_symbol_text,
    set_native_project_pool_symbol_pin_anchor,
};
pub(crate) use self::command_project_library_unit_pin::set_native_project_pool_unit_pin;
pub(crate) use self::command_project_manufacturing::{
    compare_native_project_manufacturing_set, export_native_project_manufacturing_set,
    export_native_project_manufacturing_set_without_output_run,
    inspect_native_project_manufacturing_set, manifest_native_project_manufacturing_set,
    report_native_project_manufacturing, validate_native_project_manufacturing_set,
};
pub(crate) use self::command_project_manufacturing_plan_proposals::{
    propose_create_native_project_manufacturing_plan,
    propose_create_native_project_panel_projection,
    propose_delete_native_project_manufacturing_plan,
    propose_delete_native_project_panel_projection,
    propose_update_native_project_manufacturing_plan,
    propose_update_native_project_panel_projection,
};
pub(crate) use self::command_project_manufacturing_plans::{
    create_native_project_manufacturing_plan, create_native_project_panel_projection,
    delete_native_project_manufacturing_plan, delete_native_project_panel_projection,
    query_native_project_manufacturing_plans, query_native_project_panel_projections,
    update_native_project_manufacturing_plan, update_native_project_panel_projection,
};
pub(crate) use self::command_project_native_inspect::{
    NativeProjectCheckRunView, execute_native_project_resolve_debug_query, inspect_native_project,
    query_native_project_check_profiles, query_native_project_check_run,
    query_native_project_check_run_list, query_native_project_check_run_show,
    query_native_project_check_run_with_profile, query_native_project_journal_list,
    query_native_project_journal_show, run_native_project_check_with_profile,
};
pub(crate) use self::command_project_native_journal_mutation::{
    execute_native_project_journal_redo, execute_native_project_journal_undo,
};
pub(crate) use self::command_project_native_surface::*;
pub(crate) use self::command_project_native_types::{
    NativeBoardRoot, NativeComponentPad, NativeOutline, NativePoint, NativeStackup,
};
pub(crate) use self::command_project_output_job_proposals::{
    propose_create_native_project_output_job, propose_delete_native_project_output_job,
    propose_update_native_project_output_job,
};
pub(crate) use self::command_project_output_jobs::{
    cancel_native_project_output_job_run, create_native_project_gerber_set_output_job,
    create_native_project_output_job, delete_native_project_output_job,
    ensure_native_project_gerber_set_output_job,
    ensure_native_project_manufacturing_set_output_job, find_native_project_output_job_for_scope,
    next_output_job_run_sequence, query_native_project_output_jobs, run_native_project_output_job,
    start_native_project_output_job_run, update_native_project_output_job,
};
pub(crate) use self::command_project_pool_materialization::{
    materialize_supported_pool_package_graphics, resolve_native_project_pool_path,
};
pub(crate) use self::command_project_pool_query::{
    query_native_project_pool_library_objects, query_native_project_pool_models,
};
pub(super) use self::command_project_project_core::*;
pub(crate) use self::command_project_proposals::{
    BoardComponentReplacementPlanSelectionSpec, BoardComponentReplacementSpec,
    accept_and_apply_native_project_proposal, apply_native_project_proposal,
    create_native_project_proposal, defer_native_project_proposal, preview_native_project_proposal,
    propose_native_project_board_component_replacement,
    propose_native_project_board_component_replacement_plan,
    propose_native_project_board_component_replacements, query_native_project_proposals,
    review_native_project_proposal, show_native_project_proposal, validate_native_project_proposal,
};
pub(crate) use self::command_project_relationships::{
    query_native_project_relationships, query_native_project_variants,
};
pub(crate) use self::command_project_route_surface::*;
pub(crate) use self::command_project_schematic_connectivity_mutations::{
    create_native_project_bus, delete_native_project_bus, delete_native_project_bus_entry,
    delete_native_project_junction, delete_native_project_label, delete_native_project_noconnect,
    delete_native_project_port, delete_native_project_wire, draw_native_project_wire,
    edit_native_project_bus_members, edit_native_project_port, place_native_project_bus_entry,
    place_native_project_junction, place_native_project_label, place_native_project_noconnect,
    place_native_project_port, rename_native_project_label,
};
pub(crate) use self::command_project_schematic_connectivity_queries::{
    query_native_project_bus_entries, query_native_project_buses, query_native_project_junctions,
    query_native_project_labels, query_native_project_noconnects, query_native_project_ports,
    query_native_project_wires,
};
pub(super) use self::command_project_schematic_helpers::*;
pub(crate) use self::command_project_schematic_proposals::{
    propose_draw_native_project_wire, propose_place_native_project_label,
    propose_place_native_project_symbol,
};
pub(crate) use self::command_project_schematic_queries::{
    query_native_project_check, query_native_project_diagnostics, query_native_project_drawings,
    query_native_project_hierarchy, query_native_project_nets, query_native_project_sheets,
    query_native_project_symbol_fields, query_native_project_symbol_pins,
    query_native_project_symbol_semantics, query_native_project_symbols,
    query_native_project_texts,
};
pub(crate) use self::command_project_schematic_sheet_mutations::{
    bind_native_project_sheet_instance_port, create_native_project_sheet,
    create_native_project_sheet_definition, create_native_project_sheet_instance,
    delete_native_project_sheet, delete_native_project_sheet_instance,
    move_native_project_sheet_instance, rename_native_project_sheet,
    unbind_native_project_sheet_instance_port,
};
pub(crate) use self::command_project_schematic_symbol_mutations::{
    add_native_project_symbol_field, clear_native_project_symbol_entity,
    clear_native_project_symbol_gate, clear_native_project_symbol_lib_id,
    clear_native_project_symbol_part, clear_native_project_symbol_pin_override,
    clear_native_project_symbol_unit, delete_native_project_symbol,
    delete_native_project_symbol_field, edit_native_project_symbol_field,
    mirror_native_project_symbol, move_native_project_symbol, place_native_project_symbol,
    rotate_native_project_symbol, set_native_project_symbol_display_mode,
    set_native_project_symbol_entity, set_native_project_symbol_gate,
    set_native_project_symbol_hidden_power_behavior, set_native_project_symbol_lib_id,
    set_native_project_symbol_part, set_native_project_symbol_pin_override,
    set_native_project_symbol_reference, set_native_project_symbol_unit,
    set_native_project_symbol_value,
};
pub(crate) use self::command_project_schematic_text_drawing_mutations::{
    delete_native_project_drawing, delete_native_project_text, edit_native_project_drawing_arc,
    edit_native_project_drawing_circle, edit_native_project_drawing_line,
    edit_native_project_drawing_rect, edit_native_project_text, place_native_project_drawing_arc,
    place_native_project_drawing_circle, place_native_project_drawing_line,
    place_native_project_drawing_rect, place_native_project_text,
};
pub(crate) use self::command_project_standards_repairs::generate_native_project_standards_repair_proposals;
pub(crate) use self::command_project_summary::query_native_project_summary;
pub(crate) use self::command_project_support::*;
pub(crate) use self::command_project_support::{
    parse_native_field_position, parse_native_polygon_vertices, parse_native_stackup_layers,
};
pub(crate) use self::command_project_validate::validate_native_project;
pub(super) use self::command_project_views::*;
pub(crate) use self::command_project_waivers::{
    accept_native_project_deviation, waive_native_project_finding,
};
pub(crate) use super::command_project_pool_query::query_native_project_pools;
pub(crate) use anyhow::{Context, Result, bail};
pub(crate) use eda_engine::api::{CheckCodeCount, CheckReport, CheckStatus, CheckSummary};
pub(crate) use eda_engine::board::{
    Board, BoardText, Dimension, Keepout, Net, NetClass, PadAperture, PadShape, PlacedPackage,
    PlacedPad, Stackup, StackupLayer, StackupLayerType, Track, Via, Zone,
};
pub(crate) use eda_engine::connectivity::{schematic_diagnostics, schematic_net_info};
pub(crate) use eda_engine::erc::{ErcFinding, run_prechecks};
pub(crate) use eda_engine::export::{
    render_rs274x_copper_layer, render_rs274x_outline_default, render_rs274x_paste_layer,
    render_rs274x_silkscreen_layer, render_rs274x_soldermask_layer,
};
pub(crate) use eda_engine::import::ids_sidecar::compute_source_hash_bytes;
pub(crate) use eda_engine::ir::geometry::Polygon;
pub(crate) use eda_engine::ir::geometry::{Arc, Point};
pub(crate) use eda_engine::ir::serialization::to_json_deterministic;
pub(crate) use eda_engine::rules::ast::Rule;
pub(crate) use eda_engine::schematic::{
    Bus, BusEntry, BusEntryInfo, BusInfo, CheckWaiver, ConnectivityDiagnosticInfo,
    HiddenPowerBehavior, HierarchicalPort, HierarchyInfo, Junction, LabelInfo, LabelKind, NetLabel,
    NoConnectInfo, NoConnectMarker, PinDisplayOverride, PlacedSymbol, PortDirection, PortInfo,
    Schematic, SchematicNetInfo, SchematicPrimitive, SchematicText, SchematicWire, Sheet,
    SheetDefinition, SheetFrame, SheetInstance, SymbolDisplayMode, SymbolField, SymbolFieldInfo,
    SymbolInfo, SymbolPin,
};
pub(crate) use serde::{Deserialize, Serialize};
pub(crate) use uuid::Uuid;
