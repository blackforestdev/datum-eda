use std::collections::{BTreeMap, BTreeSet, HashMap};
use std::path::{Path, PathBuf};

#[path = "command_project_board_component_layer.rs"]
mod command_project_board_component_layer;
#[path = "command_project_board_component_mutations.rs"]
mod command_project_board_component_mutations;
#[path = "command_project_board_component_query.rs"]
mod command_project_board_component_query;
#[path = "command_project_board_component_reference.rs"]
mod command_project_board_component_reference;
#[path = "command_project_board_component_value.rs"]
mod command_project_board_component_value;
#[path = "command_project_board_diagnostics.rs"]
mod command_project_board_diagnostics;
#[path = "command_project_board_layout.rs"]
mod command_project_board_layout;
#[path = "command_project_board_netclass_dimension.rs"]
mod command_project_board_netclass_dimension;
#[path = "command_project_board_pad.rs"]
mod command_project_board_pad;
#[path = "command_project_board_routing_net.rs"]
mod command_project_board_routing_net;
#[path = "command_project_default_stackup.rs"]
mod command_project_default_stackup;
#[path = "command_project_drill.rs"]
mod command_project_drill;
#[path = "command_project_forward_annotation_apply_review.rs"]
mod command_project_forward_annotation_apply_review;
#[path = "command_project_forward_annotation_artifact.rs"]
mod command_project_forward_annotation_artifact;
#[path = "command_project_forward_annotation_artifact_review.rs"]
mod command_project_forward_annotation_artifact_review;
#[path = "command_project_forward_annotation_surface.rs"]
mod command_project_forward_annotation_surface;
#[path = "command_project_forward_annotation_proposal.rs"]
mod command_project_forward_annotation_proposal;
#[path = "command_project_gerber_inspect.rs"]
mod command_project_gerber_inspect;
#[path = "command_project_gerber_layers.rs"]
mod command_project_gerber_layers;
#[path = "command_project_gerber_mechanical.rs"]
mod command_project_gerber_mechanical;
#[path = "command_project_gerber_plan.rs"]
mod command_project_gerber_plan;
#[path = "command_project_gerber_semantics.rs"]
mod command_project_gerber_semantics;
#[path = "command_project_gerber_semantics_utils.rs"]
mod command_project_gerber_semantics_utils;
#[path = "command_project_gerber_silkscreen.rs"]
mod command_project_gerber_silkscreen;
#[path = "command_project_inventory.rs"]
mod command_project_inventory;
#[path = "command_project_inventory_surface.rs"]
mod command_project_inventory_surface;
#[path = "command_project_manufacturing.rs"]
mod command_project_manufacturing;
#[path = "command_project_native_inspect.rs"]
mod command_project_native_inspect;
#[path = "command_project_native_types.rs"]
mod command_project_native_types;
#[path = "command_project_pool_materialization.rs"]
mod command_project_pool_materialization;
#[path = "command_project_pool_query.rs"]
mod command_project_pool_query;
#[path = "command_project_project_core.rs"]
mod command_project_project_core;
#[path = "command_project_roots.rs"]
mod command_project_roots;
#[path = "command_project_schematic_connectivity_mutations.rs"]
mod command_project_schematic_connectivity_mutations;
#[path = "command_project_schematic_connectivity_queries.rs"]
mod command_project_schematic_connectivity_queries;
#[path = "command_project_schematic_helpers.rs"]
mod command_project_schematic_helpers;
#[path = "command_project_schematic_queries.rs"]
mod command_project_schematic_queries;
#[path = "command_project_schematic_symbol_mutations.rs"]
mod command_project_schematic_symbol_mutations;
#[path = "command_project_schematic_text_drawing_mutations.rs"]
mod command_project_schematic_text_drawing_mutations;
#[path = "command_project_summary.rs"]
mod command_project_summary;
#[path = "command_project_support.rs"]
mod command_project_support;
#[path = "command_project_views.rs"]
mod command_project_views;

pub(crate) use self::command_project_board_component_layer::set_native_project_board_component_layer;
pub(crate) use self::command_project_board_component_mutations::{
    delete_native_project_board_component, move_native_project_board_component,
    place_native_project_board_component, rotate_native_project_board_component,
    set_native_project_board_component_locked, set_native_project_board_component_package,
    set_native_project_board_component_part,
};
use self::command_project_board_component_query::{
    component_graphic_count, component_model_count, component_package_pad_count,
};
use self::command_project_board_component_query::{
    component_has_persisted_mechanical, component_has_persisted_silkscreen,
};
pub(crate) use self::command_project_board_component_query::{
    query_native_project_board_component_mechanical,
    query_native_project_board_component_view,
    query_native_project_board_component_models_3d, query_native_project_board_component_pads,
    query_native_project_board_component_silkscreen, query_native_project_board_component_views,
    query_native_project_board_components,
};
pub(crate) use self::command_project_board_component_reference::set_native_project_board_component_reference;
pub(crate) use self::command_project_board_component_value::set_native_project_board_component_value;
pub(crate) use self::command_project_board_diagnostics::{
    query_native_project_board_check, query_native_project_board_diagnostics,
    query_native_project_board_unrouted,
};
pub(crate) use self::command_project_board_layout::{
    delete_native_project_board_keepout, delete_native_project_board_text,
    edit_native_project_board_keepout, edit_native_project_board_text,
    place_native_project_board_keepout, place_native_project_board_text,
    query_native_project_board_keepouts, query_native_project_board_outline,
    query_native_project_board_stackup, query_native_project_board_texts,
    set_native_project_board_outline, set_native_project_board_stackup,
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
    query_native_project_board_pads, query_native_project_emitted_copper_pads,
    set_native_project_board_pad_net,
};
pub(crate) use self::command_project_board_routing_net::{
    delete_native_project_board_net, delete_native_project_board_track,
    delete_native_project_board_via, delete_native_project_board_zone,
    edit_native_project_board_net, place_native_project_board_net,
    place_native_project_board_track, place_native_project_board_via,
    place_native_project_board_zone, query_native_project_board_net,
    query_native_project_board_nets,
    query_native_project_board_tracks, query_native_project_board_vias,
    query_native_project_board_zones,
};
pub(crate) use self::command_project_default_stackup::add_native_project_default_top_stackup;
use self::command_project_default_stackup::default_native_project_stackup_layers;
pub(crate) use self::command_project_drill::{
    compare_native_project_drill, compare_native_project_excellon_drill,
    export_native_project_drill, export_native_project_excellon_drill, inspect_excellon_drill,
    inspect_native_project_drill, render_expected_native_project_drill_csv,
    report_native_project_drill_hole_classes, validate_native_project_drill,
    validate_native_project_excellon_drill,
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
    NativeComponentMechanicalText,
};
pub(crate) use self::command_project_gerber_mechanical::{
    compare_native_project_gerber_mechanical_layer, export_native_project_gerber_mechanical_layer,
    validate_native_project_gerber_mechanical_layer,
};
pub(crate) use self::command_project_gerber_plan::{
    compare_native_project_gerber_export_plan, compare_native_project_gerber_set,
    export_native_project_gerber_set, plan_native_project_gerber_export,
    validate_native_project_gerber_set,
};
pub(crate) use self::command_project_gerber_semantics::{
    DEFAULT_GERBER_OUTLINE_APERTURE_NM, ParsedGerber, ParsedGerberGeometry,
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
use self::command_project_gerber_silkscreen::{
    count_native_component_silkscreen_arcs, count_native_component_silkscreen_circles,
    count_native_component_silkscreen_lines, count_native_component_silkscreen_polygons,
    count_native_component_silkscreen_polylines, count_native_component_silkscreen_texts,
    resolve_native_project_silkscreen_context,
};
pub(crate) use self::command_project_inventory_surface::*;
pub(crate) use self::command_project_manufacturing::{
    compare_native_project_manufacturing_set, export_native_project_manufacturing_set,
    inspect_native_project_manufacturing_set, manifest_native_project_manufacturing_set,
    report_native_project_manufacturing, validate_native_project_manufacturing_set,
};
pub(crate) use self::command_project_native_inspect::inspect_native_project;
pub(crate) use self::command_project_native_types::{
    NativeBoardRoot, NativeComponentPad, NativeOutline, NativePoint, NativeStackup,
};
use self::command_project_pool_materialization::{
    materialize_supported_pool_package_graphics, resolve_native_project_pool_path,
};
use self::command_project_pool_query::collect_native_project_pool_ref_views;
pub(crate) use self::command_project_pool_query::query_native_project_pools;
use self::command_project_project_core::*;
use self::command_project_roots::*;
pub(crate) use self::command_project_roots::{create_native_project, query_native_project_rules};
pub(crate) use self::command_project_schematic_connectivity_mutations::{
    create_native_project_bus, delete_native_project_bus_entry, delete_native_project_junction,
    delete_native_project_label, delete_native_project_noconnect, delete_native_project_port,
    delete_native_project_wire, draw_native_project_wire, edit_native_project_bus_members,
    edit_native_project_port, place_native_project_bus_entry, place_native_project_junction,
    place_native_project_label, place_native_project_noconnect, place_native_project_port,
    rename_native_project_label,
};
pub(crate) use self::command_project_schematic_connectivity_queries::{
    query_native_project_bus_entries, query_native_project_buses, query_native_project_junctions,
    query_native_project_labels, query_native_project_noconnects, query_native_project_ports,
    query_native_project_wires,
};
use self::command_project_schematic_helpers::*;
pub(crate) use self::command_project_schematic_queries::{
    query_native_project_check, query_native_project_diagnostics, query_native_project_drawings,
    query_native_project_erc, query_native_project_nets, query_native_project_symbol_fields,
    query_native_project_symbol_pins, query_native_project_symbol_semantics,
    query_native_project_symbols, query_native_project_texts,
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
pub(crate) use self::command_project_summary::query_native_project_summary;
use self::command_project_support::*;
pub(crate) use self::command_project_support::{
    parse_native_field_position, parse_native_polygon_vertices, parse_native_stackup_layers,
};
use self::command_project_views::*;
use anyhow::{Context, Result, bail};
use eda_engine::api::{CheckCodeCount, CheckReport, CheckStatus, CheckSummary};
use eda_engine::board::{
    Board, BoardText, Dimension, Keepout, Net, NetClass, PadAperture, PadShape, PlacedPackage,
    PlacedPad, Stackup, StackupLayer, StackupLayerType, Track, Via, Zone,
};
use eda_engine::connectivity::{schematic_diagnostics, schematic_net_info};
use eda_engine::erc::{ErcFinding, run_prechecks};
use eda_engine::export::{
    render_rs274x_copper_layer, render_rs274x_outline_default, render_rs274x_paste_layer,
    render_rs274x_silkscreen_layer, render_rs274x_soldermask_layer,
};
use eda_engine::import::ids_sidecar::compute_source_hash_bytes;
use eda_engine::ir::geometry::Polygon;
use eda_engine::ir::geometry::{Arc, Point};
use eda_engine::ir::serialization::to_json_deterministic;
use eda_engine::rules::ast::Rule;
use eda_engine::schematic::{
    Bus, BusEntry, BusEntryInfo, BusInfo, CheckWaiver, ConnectivityDiagnosticInfo,
    HiddenPowerBehavior, HierarchicalPort, Junction, LabelInfo, LabelKind, NetLabel, NoConnectInfo,
    NoConnectMarker, PinDisplayOverride, PlacedSymbol, PortDirection, PortInfo, Schematic,
    SchematicNetInfo, SchematicPrimitive, SchematicText, SchematicWire, Sheet, SheetFrame,
    SymbolDisplayMode, SymbolField, SymbolFieldInfo, SymbolInfo, SymbolPin,
};
use serde::{Deserialize, Serialize};
use uuid::Uuid;
