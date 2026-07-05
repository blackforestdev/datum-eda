pub(crate) use super::*;

pub(crate) use crate::context as command_context; // Wave 2 compat: exec dispatch still calls command_context::*; Wave 3 removes.
#[path = "command_exec_artifact.rs"]
mod command_exec_artifact;
#[path = "command_exec_board_component.rs"]
mod command_exec_board_component;
#[path = "command_exec_board_net.rs"]
mod command_exec_board_net;
#[path = "command_exec_board_stackup.rs"]
mod command_exec_board_stackup;
#[path = "command_exec_check.rs"]
mod command_exec_check;
#[path = "command_exec_context.rs"]
mod command_exec_context;
#[path = "command_exec_dispatch.rs"]
mod command_exec_dispatch;
#[path = "command_exec_drill.rs"]
mod command_exec_drill;
#[path = "command_exec_forward_annotation.rs"]
mod command_exec_forward_annotation;
#[path = "command_exec_gerber_plan.rs"]
mod command_exec_gerber_plan;
#[path = "command_exec_inventory.rs"]
mod command_exec_inventory;
#[path = "command_exec_journal.rs"]
mod command_exec_journal;
#[path = "command_exec_manufacturing.rs"]
mod command_exec_manufacturing;
#[path = "command_exec_native_support.rs"]
mod command_exec_native_support;
#[path = "command_exec_plan.rs"]
mod command_exec_plan;
#[path = "command_exec_project_board_handoff.rs"]
mod command_exec_project_board_handoff;
#[path = "command_exec_project_import.rs"]
mod command_exec_project_import;
#[path = "command_exec_project_inspect.rs"]
mod command_exec_project_inspect;
#[path = "command_exec_project_library.rs"]
mod command_exec_project_library;
#[path = "command_exec_project_library_footprint.rs"]
mod command_exec_project_library_footprint;
#[path = "command_exec_project_library_part_bindings.rs"]
mod command_exec_project_library_part_bindings;
#[path = "command_exec_project_library_pin_pad_map.rs"]
mod command_exec_project_library_pin_pad_map;
#[path = "command_exec_project_library_symbol_pin.rs"]
mod command_exec_project_library_symbol_pin;
#[path = "command_exec_project_proposal_lifecycle.rs"]
mod command_exec_project_proposal_lifecycle;
#[path = "command_exec_project_query.rs"]
mod command_exec_project_query;
#[path = "command_exec_project_query_route_graph.rs"]
mod command_exec_project_query_route_graph;
#[path = "command_exec_project_schematic_connectivity.rs"]
mod command_exec_project_schematic_connectivity;
#[path = "command_exec_project_schematic_symbols.rs"]
mod command_exec_project_schematic_symbols;
#[path = "command_exec_proposal.rs"]
mod command_exec_proposal;
#[path = "command_exec_proposal_library.rs"]
mod command_exec_proposal_library;
#[path = "command_exec_query.rs"]
mod command_exec_query;
#[path = "command_exec_route_proposal.rs"]
mod command_exec_route_proposal;

pub(crate) use self::command_exec_dispatch::execute_with_exit_code;
pub(super) use self::command_exec_project_import::execute_project_import_command;
pub(super) use self::command_exec_project_library_symbol_pin::execute_project_library_symbol_pin_command;

// Wave 3: the unified project router (commands/dispatch.rs) calls the exec
// layer through these crate-visible re-exports; they disappear when the exec
// fns convert to run() impls in the next wave.
pub(crate) use self::command_exec_board_component::{
    execute_move_board_component, execute_rotate_board_component,
    execute_set_board_component_layer, execute_set_board_component_locked,
    execute_set_board_component_package, execute_set_board_component_part,
    execute_set_board_component_reference, execute_set_board_component_value,
};
pub(crate) use self::command_exec_board_net::{
    execute_delete_board_net, execute_delete_board_net_class, execute_edit_board_net,
    execute_edit_board_net_class, execute_place_board_net, execute_place_board_net_class,
};
pub(crate) use self::command_exec_board_stackup::{
    execute_add_default_top_stackup, execute_set_board_stackup,
};
pub(crate) use self::command_exec_drill::execute_drill_command;
pub(crate) use self::command_exec_forward_annotation::execute_forward_annotation_command;
pub(crate) use self::command_exec_gerber_plan::execute_gerber_workflow_command;
pub(crate) use self::command_exec_inventory::execute_inventory_command;
pub(crate) use self::command_exec_manufacturing::execute_manufacturing_command;
pub(crate) use self::command_exec_project_board_handoff::{
    execute_generate_board_components, execute_place_board_component, execute_set_board_name,
    execute_set_board_outline,
};
pub(crate) use self::command_exec_project_inspect::{
    execute_project_excellon_drill_inspection, execute_project_gerber_inspection,
};
pub(crate) use self::command_exec_project_library::execute_project_library_command;
pub(crate) use self::command_exec_project_library_footprint::execute_project_library_footprint_command;
pub(crate) use self::command_exec_project_library_part_bindings::execute_project_import_or_part_binding_command;
pub(crate) use self::command_exec_project_library_pin_pad_map::execute_project_library_pin_pad_map_command;
pub(crate) use self::command_exec_project_proposal_lifecycle::execute_project_proposal_lifecycle_command;
pub(crate) use self::command_exec_project_query::execute_native_project_query_command;
pub(crate) use self::command_exec_project_schematic_connectivity::execute_project_schematic_connectivity_command;
pub(crate) use self::command_exec_project_schematic_symbols::execute_project_schematic_symbols_command;
pub(crate) use self::command_exec_route_proposal::execute_route_proposal_command;
