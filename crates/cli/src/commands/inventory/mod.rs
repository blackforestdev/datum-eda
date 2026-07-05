// commands/inventory — BOM and pick-and-place inventory: export, inspect,
// validate, compare, and the panel PnP projection, plus their CSV codecs.
//
// Wave 2 move. Files came from the command_project/ subdir
// (command_project_inventory*.rs, declared by command_project.rs with the
// surface re-export in command_project_surface.rs); the re-export below
// reproduces exactly what command_project_inventory_surface.rs exported
// (csv.rs and rows.rs stay module-private, as before).

mod csv;
mod inventory;
mod views;

pub(crate) use self::inventory::{
    compare_native_project_bom, compare_native_project_pnp, export_native_project_bom,
    export_native_project_panel_pnp, export_native_project_pnp,
    render_expected_native_project_panel_pnp_csv,
};
pub(crate) use self::views::*;
