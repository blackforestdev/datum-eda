// commands/imports — one-time import converters (Eagle library, KiCad board /
// schematic / footprint) and the Import Map query.
//
// Wave 2 move. Files came from the legacy command_project_surface.rs host;
// the re-exports below reproduce exactly what that host exported for this
// family. eagle_import_map.rs and schematic_identities.rs are family-internal
// helpers (consumed by imports.rs / schematic.rs), exactly as before.

#[allow(unused_imports)]
use super::*;

mod eagle_import_map;
mod import_map;
mod import_report;
// Deliberate commands/<family>/<family>.rs layout (see CLAUDE.md repository layout).
#[allow(clippy::module_inception)]
mod imports;
mod kicad_footprint;
mod schematic;
mod schematic_identities;

pub(crate) use self::import_map::query_native_project_import_map;
pub(crate) use self::import_report::*;
pub(crate) use self::kicad_footprint::import_native_project_kicad_footprint;
pub(crate) use self::schematic::import_native_project_kicad_schematic;
