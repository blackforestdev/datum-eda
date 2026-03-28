use serde::Serialize;

use super::{NativeProjectGerberGeometryEntryView, append_gerber_geometry_entries};

#[derive(Debug, Clone, Serialize)]
pub(crate) struct NativeProjectGerberInspectionView {
    pub(crate) action: String,
    pub(crate) gerber_path: String,
    pub(crate) geometry_count: usize,
    pub(crate) stroke_count: usize,
    pub(crate) flash_count: usize,
    pub(crate) region_count: usize,
    pub(crate) entries: Vec<NativeProjectGerberGeometryEntryView>,
}

pub(crate) fn render_native_project_gerber_inspection_text(
    report: &NativeProjectGerberInspectionView,
) -> String {
    let mut lines = vec![
        format!("action: {}", report.action),
        format!("gerber_path: {}", report.gerber_path),
        format!("geometry_count: {}", report.geometry_count),
        format!("stroke_count: {}", report.stroke_count),
        format!("flash_count: {}", report.flash_count),
        format!("region_count: {}", report.region_count),
    ];
    append_gerber_geometry_entries(&mut lines, "entries", &report.entries);
    lines.join("\n")
}
