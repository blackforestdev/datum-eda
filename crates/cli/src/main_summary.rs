use serde::Serialize;

use super::NativeProjectInspectPoolRefView;

#[derive(Debug, Clone, Serialize)]
pub(crate) struct NativeProjectSummaryView {
    pub(crate) domain: &'static str,
    pub(crate) project_name: String,
    pub(crate) schema_version: u32,
    pub(crate) pools: usize,
    pub(crate) pool_refs: Vec<NativeProjectInspectPoolRefView>,
    pub(crate) schematic: NativeProjectSchematicSummaryView,
    pub(crate) board: NativeProjectBoardSummaryView,
    pub(crate) rules: NativeProjectRulesSummaryView,
}

#[derive(Debug, Clone, Serialize)]
pub(crate) struct NativeProjectSchematicSummaryView {
    pub(crate) sheets: usize,
    pub(crate) sheet_definitions: usize,
    pub(crate) sheet_instances: usize,
    pub(crate) variants: usize,
    pub(crate) symbols: usize,
    pub(crate) wires: usize,
    pub(crate) junctions: usize,
    pub(crate) labels: usize,
    pub(crate) ports: usize,
    pub(crate) buses: usize,
    pub(crate) bus_entries: usize,
    pub(crate) noconnects: usize,
    pub(crate) texts: usize,
    pub(crate) drawings: usize,
}

#[derive(Debug, Clone, Serialize)]
pub(crate) struct NativeProjectBoardSummaryView {
    pub(crate) name: String,
    pub(crate) layers: usize,
    pub(crate) components: usize,
    pub(crate) components_with_persisted_silkscreen: usize,
    pub(crate) components_with_persisted_mechanical: usize,
    pub(crate) components_with_persisted_pads: usize,
    pub(crate) components_with_persisted_models_3d: usize,
    pub(crate) pads: usize,
    pub(crate) nets: usize,
    pub(crate) net_classes: usize,
    pub(crate) tracks: usize,
    pub(crate) vias: usize,
    pub(crate) zones: usize,
    pub(crate) keepouts: usize,
    pub(crate) dimensions: usize,
    pub(crate) texts: usize,
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
    pub(crate) persisted_component_pads: usize,
    pub(crate) persisted_component_models_3d: usize,
}

#[derive(Debug, Clone, Serialize)]
pub(crate) struct NativeProjectRulesSummaryView {
    pub(crate) count: usize,
}

pub(crate) fn render_native_project_summary_text(report: &NativeProjectSummaryView) -> String {
    let mut lines = vec![
        format!("project_name: {}", report.project_name),
        format!("schema_version: {}", report.schema_version),
        format!("pools: {}", report.pools),
        format!("schematic_sheets: {}", report.schematic.sheets),
        format!(
            "schematic_sheet_definitions: {}",
            report.schematic.sheet_definitions
        ),
        format!(
            "schematic_sheet_instances: {}",
            report.schematic.sheet_instances
        ),
        format!("schematic_variants: {}", report.schematic.variants),
        format!("schematic_symbols: {}", report.schematic.symbols),
        format!("schematic_wires: {}", report.schematic.wires),
        format!("schematic_junctions: {}", report.schematic.junctions),
        format!("schematic_labels: {}", report.schematic.labels),
        format!("schematic_ports: {}", report.schematic.ports),
        format!("schematic_buses: {}", report.schematic.buses),
        format!("schematic_bus_entries: {}", report.schematic.bus_entries),
        format!("schematic_noconnects: {}", report.schematic.noconnects),
        format!("schematic_texts: {}", report.schematic.texts),
        format!("schematic_drawings: {}", report.schematic.drawings),
        format!("board_name: {}", report.board.name),
        format!("board_layers: {}", report.board.layers),
        format!("board_components: {}", report.board.components),
        format!(
            "board_components_with_persisted_silkscreen: {}",
            report.board.components_with_persisted_silkscreen
        ),
        format!(
            "board_components_with_persisted_mechanical: {}",
            report.board.components_with_persisted_mechanical
        ),
        format!(
            "board_components_with_persisted_pads: {}",
            report.board.components_with_persisted_pads
        ),
        format!(
            "board_components_with_persisted_models_3d: {}",
            report.board.components_with_persisted_models_3d
        ),
        format!("board_pads: {}", report.board.pads),
        format!("board_nets: {}", report.board.nets),
        format!("board_net_classes: {}", report.board.net_classes),
        format!("board_tracks: {}", report.board.tracks),
        format!("board_vias: {}", report.board.vias),
        format!("board_zones: {}", report.board.zones),
        format!("board_keepouts: {}", report.board.keepouts),
        format!("board_dimensions: {}", report.board.dimensions),
        format!("board_texts: {}", report.board.texts),
        format!(
            "board_persisted_component_silkscreen_texts: {}",
            report.board.persisted_component_silkscreen_texts
        ),
        format!(
            "board_persisted_component_silkscreen_lines: {}",
            report.board.persisted_component_silkscreen_lines
        ),
        format!(
            "board_persisted_component_silkscreen_arcs: {}",
            report.board.persisted_component_silkscreen_arcs
        ),
        format!(
            "board_persisted_component_silkscreen_circles: {}",
            report.board.persisted_component_silkscreen_circles
        ),
        format!(
            "board_persisted_component_silkscreen_polygons: {}",
            report.board.persisted_component_silkscreen_polygons
        ),
        format!(
            "board_persisted_component_silkscreen_polylines: {}",
            report.board.persisted_component_silkscreen_polylines
        ),
        format!(
            "board_persisted_component_mechanical_texts: {}",
            report.board.persisted_component_mechanical_texts
        ),
        format!(
            "board_persisted_component_mechanical_lines: {}",
            report.board.persisted_component_mechanical_lines
        ),
        format!(
            "board_persisted_component_mechanical_arcs: {}",
            report.board.persisted_component_mechanical_arcs
        ),
        format!(
            "board_persisted_component_mechanical_circles: {}",
            report.board.persisted_component_mechanical_circles
        ),
        format!(
            "board_persisted_component_mechanical_polygons: {}",
            report.board.persisted_component_mechanical_polygons
        ),
        format!(
            "board_persisted_component_mechanical_polylines: {}",
            report.board.persisted_component_mechanical_polylines
        ),
        format!(
            "board_persisted_component_pads: {}",
            report.board.persisted_component_pads
        ),
        format!(
            "board_persisted_component_models_3d: {}",
            report.board.persisted_component_models_3d
        ),
        format!("rule_count: {}", report.rules.count),
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
