// commands/schematic — the schematic command family: connectivity
// (wires/labels/junctions/ports/buses/noconnects), symbols and their
// pool bindings, sheets, text/drawings, schematic queries, proposals,
// mutation-target helpers, and the schematic CLI view types.
//
// Wave 2 move. Files came from three legacy hosts; the re-exports below
// reproduce exactly what those hosts exported for this family:
//   - command_project.rs decls with named re-export lists in
//     command_project_surface.rs (connectivity_*, proposals, queries,
//     sheet_mutations, symbol_mutations, text_drawing_mutations).
//   - main.rs: the named cli_symbol_views.rs list (now views_symbol.rs) and
//     the schematic mutation view structs/renderers (now views_mutations.rs).
// helpers.rs items were pub(super) under command_project and are now
// pub(crate) because two consumers (command_project_project_core.rs,
// command_project_validate.rs) still live outside this family; the old
// surface file bridges them back into command_project scope. The
// symbol_component_instance / symbol_library_materialization /
// symbol_reports trio stays family-internal (module-path imports only),
// exactly as under command_project.
//
// `use super::*;` anchors the family scope on commands/, which in turn
// anchors on the crate root, keeping crate-root names visible to member
// files exactly as the old command_project chain did.

#[allow(unused_imports)]
use super::*;

mod connectivity_mutations;
mod connectivity_queries;
mod helpers;
mod proposals;
mod queries;
mod sheet_mutations;
mod symbol_component_instance;
mod symbol_library_materialization;
mod symbol_mutations;
mod symbol_reports;
mod text_drawing_mutations;
mod views_mutations;
mod views_symbol;

pub(crate) use self::connectivity_queries::{
    query_native_project_bus_entries, query_native_project_buses, query_native_project_junctions,
    query_native_project_labels, query_native_project_noconnects, query_native_project_ports,
    query_native_project_wires,
};
pub(crate) use self::helpers::*;
pub(crate) use self::queries::query_native_project_check_with_inputs;
pub(crate) use self::queries::{
    query_native_project_check, query_native_project_diagnostics, query_native_project_drawings,
    query_native_project_hierarchy, query_native_project_nets, query_native_project_sheets,
    query_native_project_symbol_fields, query_native_project_symbol_pins,
    query_native_project_symbol_semantics, query_native_project_symbols,
    query_native_project_texts,
};
pub(crate) use self::sheet_mutations::{
    bind_native_project_sheet_instance_port, create_native_project_sheet,
    create_native_project_sheet_definition, create_native_project_sheet_instance,
    delete_native_project_sheet, delete_native_project_sheet_instance,
    move_native_project_sheet_instance, rename_native_project_sheet,
    unbind_native_project_sheet_instance_port,
};
pub(crate) use self::text_drawing_mutations::{
    delete_native_project_drawing, delete_native_project_text, edit_native_project_drawing_arc,
    edit_native_project_drawing_circle, edit_native_project_drawing_line,
    edit_native_project_drawing_rect, edit_native_project_text, place_native_project_drawing_arc,
    place_native_project_drawing_circle, place_native_project_drawing_line,
    place_native_project_drawing_rect, place_native_project_text,
};
pub(crate) use self::views_mutations::*;
pub(crate) use self::views_symbol::*;
