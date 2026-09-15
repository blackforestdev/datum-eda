// commands/drill — drill CSV/Excellon export, inspection, panels, and the
// CLI view types for drill commands.

#[allow(unused_imports)]
use super::*;

// Deliberate commands/<family>/<family>.rs layout (see CLAUDE.md repository layout).
#[allow(clippy::module_inception)]
mod drill;
mod views;

pub(crate) use self::drill::{
    compare_native_project_excellon_drill, export_native_project_drill,
    export_native_project_excellon_drill, export_native_project_panel_drill,
    export_native_project_panel_excellon_drill, inspect_excellon_drill,
    render_expected_native_project_drill_csv, render_expected_native_project_panel_drill_csv,
    render_expected_native_project_panel_excellon_drill, report_native_project_drill_hole_classes,
    validate_native_project_excellon_drill,
};
pub(crate) use self::views::*;
