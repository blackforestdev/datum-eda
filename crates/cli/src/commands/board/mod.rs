// commands/board — the board command family: component mutations and
// queries, layout (outline/stackup/keepouts/texts), net classes and
// dimensions, pads, routing nets/tracks/vias/zones (incl. production
// ZoneFill), diagnostics, schematic→board handoff, the default stackup,
// and the board CLI view types.
//
// Wave 2 move. Files came from three legacy hosts; the re-exports below
// reproduce exactly what those hosts exported for this family:
//   - command_project.rs decls with named re-export lists in
//     command_project_surface.rs (component_*, diagnostics, layout,
//     netclass_dimension, pad, routing_net, default_stackup).
//   - command_project_surface.rs decl + list for handoff.
//   - main.rs: glob re-export of main_board_component.rs (now views.rs) and
//     the board mutation view structs/renderers (now views_mutations.rs).
// component_query's helper fns and diagnostics' query_native_project_drc_with_rules
// were pub(super) under command_project and are now pub(crate) because their
// consumers (command_project_root_imports.rs, commands/schematic/queries.rs)
// live outside this family. zone_fill_projection.rs stays a #[path] child of
// diagnostics.rs and commands/gerber/layers.rs; zone_fill_context.rs stays a
// #[path] child of routing_net.rs — they are deliberately not declared here.
//
// `use super::*;` anchors the family scope on commands/, which in turn
// anchors on the crate root, keeping crate-root names visible to member
// files exactly as the old command_project chain did.

#[allow(unused_imports)]
use super::*;

mod component_layer;
mod component_mutations;
mod component_query;
mod component_reference;
mod component_value;
mod default_stackup;
mod diagnostics;
mod handoff;
mod layout;
mod netclass_dimension;
mod pad;
mod routing_net;
mod views;
mod views_mutations;

pub(crate) use self::component_layer::set_native_project_board_component_layer;
pub(crate) use self::component_mutations::{
    board_package_materialization_payload_for_component,
    current_board_component_materialization_payload, delete_native_project_board_component,
    move_native_project_board_component, place_native_project_board_component,
    rotate_native_project_board_component, set_native_project_board_component_locked,
    set_native_project_board_component_package, set_native_project_board_component_part,
};
pub(crate) use self::component_query::{
    component_graphic_count, component_has_persisted_mechanical,
    component_has_persisted_silkscreen, component_model_count, component_package_pad_count,
};
pub(crate) use self::component_query::{
    query_native_project_board_component_mechanical,
    query_native_project_board_component_models_3d, query_native_project_board_component_pads,
    query_native_project_board_component_silkscreen, query_native_project_board_component_view,
    query_native_project_board_component_views, query_native_project_board_components,
};
pub(crate) use self::component_reference::set_native_project_board_component_reference;
pub(crate) use self::component_value::set_native_project_board_component_value;
pub(crate) use self::default_stackup::add_native_project_default_top_stackup;
pub(crate) use self::default_stackup::default_native_project_stackup_layers;
pub(crate) use self::diagnostics::query_native_project_drc_with_rules;
pub(crate) use self::diagnostics::{
    query_native_project_board_check, query_native_project_board_diagnostics,
    query_native_project_board_unrouted,
};
pub(crate) use self::handoff::{
    generate_native_project_board_components, render_native_project_board_handoff_text,
};
pub(crate) use self::layout::{
    delete_native_project_board_keepout, delete_native_project_board_text,
    edit_native_project_board_keepout, edit_native_project_board_text,
    place_native_project_board_keepout, place_native_project_board_text,
    query_native_project_board_keepouts, query_native_project_board_outline,
    query_native_project_board_stackup, query_native_project_board_texts,
    set_native_project_board_name, set_native_project_board_outline,
    set_native_project_board_stackup,
};
pub(crate) use self::netclass_dimension::{
    delete_native_project_board_dimension, delete_native_project_board_net_class,
    edit_native_project_board_dimension, edit_native_project_board_net_class,
    place_native_project_board_dimension, place_native_project_board_net_class,
    query_native_project_board_dimensions, query_native_project_board_net_class,
    query_native_project_board_net_classes,
};
pub(crate) use self::pad::{
    delete_native_project_board_pad, edit_native_project_board_pad, place_native_project_board_pad,
    query_native_project_board_pads, set_native_project_board_pad_net,
};
pub(crate) use self::routing_net::zone_fill_copper_context;
pub(crate) use self::routing_net::{
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
pub(crate) use self::views::*;
pub(crate) use self::views_mutations::*;
