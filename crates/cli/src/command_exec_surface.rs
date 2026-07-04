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
#[path = "command_exec_project_board_surface.rs"]
mod command_exec_project_board_surface;
#[path = "command_exec_project_command.rs"]
mod command_exec_project_command;
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
pub(super) use self::command_exec_project_library::execute_project_library_command;
pub(super) use self::command_exec_project_library_footprint::execute_project_library_footprint_command;
pub(super) use self::command_exec_project_library_part_bindings::execute_project_import_or_part_binding_command;
pub(super) use self::command_exec_project_library_pin_pad_map::execute_project_library_pin_pad_map_command;
pub(super) use self::command_exec_project_library_symbol_pin::execute_project_library_symbol_pin_command;
pub(super) use self::command_exec_project_proposal_lifecycle::execute_project_proposal_lifecycle_command;
