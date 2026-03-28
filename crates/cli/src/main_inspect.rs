use serde::Serialize;

#[derive(Debug, Clone, Serialize)]
pub(crate) struct NativeProjectInspectReportView {
    pub(crate) project_root: String,
    pub(crate) project_name: String,
    pub(crate) schema_version: u32,
    pub(crate) project_uuid: String,
    pub(crate) schematic_uuid: String,
    pub(crate) board_uuid: String,
    pub(crate) pools: usize,
    pub(crate) pool_refs: Vec<NativeProjectInspectPoolRefView>,
    pub(crate) schematic_path: String,
    pub(crate) board_path: String,
    pub(crate) rules_path: String,
    pub(crate) sheet_count: usize,
    pub(crate) sheet_definition_count: usize,
    pub(crate) sheet_instance_count: usize,
    pub(crate) variant_count: usize,
    pub(crate) board_package_count: usize,
    pub(crate) board_components_with_persisted_silkscreen: usize,
    pub(crate) board_components_with_persisted_mechanical: usize,
    pub(crate) board_pad_count: usize,
    pub(crate) board_net_count: usize,
    pub(crate) board_track_count: usize,
    pub(crate) board_via_count: usize,
    pub(crate) board_zone_count: usize,
    pub(crate) persisted_component_silkscreen_texts: usize,
    pub(crate) persisted_component_silkscreen_lines: usize,
    pub(crate) persisted_component_silkscreen_arcs: usize,
    pub(crate) persisted_component_silkscreen_circles: usize,
    pub(crate) persisted_component_silkscreen_polygons: usize,
    pub(crate) persisted_component_silkscreen_polylines: usize,
    pub(crate) persisted_component_mechanical_texts: usize,
    pub(crate) persisted_component_mechanical_lines: usize,
    pub(crate) persisted_component_mechanical_arcs: usize,
    pub(crate) persisted_component_mechanical_circles: usize,
    pub(crate) persisted_component_mechanical_polygons: usize,
    pub(crate) persisted_component_mechanical_polylines: usize,
    pub(crate) rule_count: usize,
}

#[derive(Debug, Clone, Serialize)]
pub(crate) struct NativeProjectInspectPoolRefView {
    pub(crate) manifest_path: String,
    pub(crate) priority: u32,
    pub(crate) resolved_path: String,
    pub(crate) exists: bool,
}

pub(crate) fn render_native_project_inspect_report_text(
    report: &NativeProjectInspectReportView,
) -> String {
    let mut lines = vec![
        format!("project_root: {}", report.project_root),
        format!("project_name: {}", report.project_name),
        format!("schema_version: {}", report.schema_version),
        format!("project_uuid: {}", report.project_uuid),
        format!("schematic_uuid: {}", report.schematic_uuid),
        format!("board_uuid: {}", report.board_uuid),
        format!("pools: {}", report.pools),
        format!("schematic_path: {}", report.schematic_path),
        format!("board_path: {}", report.board_path),
        format!("rules_path: {}", report.rules_path),
        format!("sheet_count: {}", report.sheet_count),
        format!("sheet_definition_count: {}", report.sheet_definition_count),
        format!("sheet_instance_count: {}", report.sheet_instance_count),
        format!("variant_count: {}", report.variant_count),
        format!("board_package_count: {}", report.board_package_count),
        format!(
            "board_components_with_persisted_silkscreen: {}",
            report.board_components_with_persisted_silkscreen
        ),
        format!(
            "board_components_with_persisted_mechanical: {}",
            report.board_components_with_persisted_mechanical
        ),
        format!("board_pad_count: {}", report.board_pad_count),
        format!("board_net_count: {}", report.board_net_count),
        format!("board_track_count: {}", report.board_track_count),
        format!("board_via_count: {}", report.board_via_count),
        format!("board_zone_count: {}", report.board_zone_count),
        format!(
            "persisted_component_silkscreen_texts: {}",
            report.persisted_component_silkscreen_texts
        ),
        format!(
            "persisted_component_silkscreen_lines: {}",
            report.persisted_component_silkscreen_lines
        ),
        format!(
            "persisted_component_silkscreen_arcs: {}",
            report.persisted_component_silkscreen_arcs
        ),
        format!(
            "persisted_component_silkscreen_circles: {}",
            report.persisted_component_silkscreen_circles
        ),
        format!(
            "persisted_component_silkscreen_polygons: {}",
            report.persisted_component_silkscreen_polygons
        ),
        format!(
            "persisted_component_silkscreen_polylines: {}",
            report.persisted_component_silkscreen_polylines
        ),
        format!(
            "persisted_component_mechanical_texts: {}",
            report.persisted_component_mechanical_texts
        ),
        format!(
            "persisted_component_mechanical_lines: {}",
            report.persisted_component_mechanical_lines
        ),
        format!(
            "persisted_component_mechanical_arcs: {}",
            report.persisted_component_mechanical_arcs
        ),
        format!(
            "persisted_component_mechanical_circles: {}",
            report.persisted_component_mechanical_circles
        ),
        format!(
            "persisted_component_mechanical_polygons: {}",
            report.persisted_component_mechanical_polygons
        ),
        format!(
            "persisted_component_mechanical_polylines: {}",
            report.persisted_component_mechanical_polylines
        ),
        format!("rule_count: {}", report.rule_count),
    ];
    if !report.pool_refs.is_empty() {
        lines.push("pool_refs:".to_string());
        for pool_ref in &report.pool_refs {
            lines.push(format!(
                "  path={} priority={} resolved_path={} exists={}",
                pool_ref.manifest_path, pool_ref.priority, pool_ref.resolved_path, pool_ref.exists
            ));
        }
    }
    lines.join("\n")
}
