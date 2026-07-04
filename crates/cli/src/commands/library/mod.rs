// commands/library — native pool/library authoring: pool library objects
// (entities, symbols, units, parts, padstacks), footprints, packages,
// pin/pad maps, part bindings, geometry, and their proposal variants.
//
// Wave 2 move. Files came from the command_project_library_* chain hosted by
// command_project_surface.rs; the re-exports below reproduce exactly the
// named lists that host exported for this family (payload.rs stays
// module-private, as before).

mod footprint;
mod footprint_proposals;
mod library;
mod package;
mod package_geometry;
mod package_geometry_proposals;
mod package_pad;
mod package_proposals;
mod pad_map;
mod part_bindings;
mod payload;
mod pin_pad_map;
mod proposals;
mod symbol_geometry;
mod unit_pin;

pub(crate) use self::footprint::{
    add_native_project_pool_footprint_silkscreen_circle,
    add_native_project_pool_footprint_silkscreen_line,
    add_native_project_pool_footprint_silkscreen_polygon,
    add_native_project_pool_footprint_silkscreen_rect, create_native_project_pool_footprint,
    generate_native_project_ipc7351b_two_terminal_chip,
    set_native_project_pool_footprint_courtyard_polygon,
    set_native_project_pool_footprint_courtyard_rect, set_native_project_pool_footprint_pad,
};
pub(crate) use self::footprint_proposals::{
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
pub(crate) use self::library::{
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
pub(crate) use self::package::create_native_project_pool_package;
pub(crate) use self::package_geometry::{
    add_native_project_pool_package_model_3d, add_native_project_pool_package_silkscreen_arc,
    add_native_project_pool_package_silkscreen_circle,
    add_native_project_pool_package_silkscreen_line,
    add_native_project_pool_package_silkscreen_polygon,
    add_native_project_pool_package_silkscreen_rect,
    add_native_project_pool_package_silkscreen_text, set_native_project_pool_package_body_heights,
    set_native_project_pool_package_courtyard_polygon,
    set_native_project_pool_package_courtyard_rect,
};
pub(crate) use self::package_geometry_proposals::{
    propose_set_native_project_pool_package_courtyard_polygon,
    propose_set_native_project_pool_package_courtyard_rect,
    propose_set_native_project_pool_package_pad,
};
pub(crate) use self::package_pad::set_native_project_pool_package_pad;
pub(crate) use self::package_proposals::propose_create_native_project_pool_package;
pub(crate) use self::pad_map::{
    set_native_project_pool_part_pad_map_entry, set_native_project_pool_part_pad_map_from_entries,
};
pub(crate) use self::part_bindings::set_native_project_pool_part_bindings;
pub(crate) use self::pin_pad_map::{
    create_native_project_pool_pin_pad_map, set_native_project_pool_pin_pad_map,
};
pub(crate) use self::proposals::{
    propose_create_native_project_pool_entity, propose_create_native_project_pool_library_object,
    propose_create_native_project_pool_padstack, propose_create_native_project_pool_pin_pad_map,
    propose_create_native_project_pool_symbol, propose_create_native_project_pool_unit,
    propose_set_native_project_pool_pin_pad_map,
};
pub(crate) use self::symbol_geometry::{
    add_native_project_pool_symbol_arc, add_native_project_pool_symbol_circle,
    add_native_project_pool_symbol_line, add_native_project_pool_symbol_polygon,
    add_native_project_pool_symbol_rect, add_native_project_pool_symbol_text,
    set_native_project_pool_symbol_pin_anchor,
};
pub(crate) use self::unit_pin::set_native_project_pool_unit_pin;
