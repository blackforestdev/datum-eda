use std::path::Path;

use anyhow::{Context, Result};
use eda_engine::board::{RoutingComponentPad, RoutingSubstrateReport};
use eda_engine::ir::geometry::Point;
use uuid::Uuid;

use super::super::{LoadedNativeProject, build_native_project_board, load_native_project};

pub(crate) fn query_native_project_routing_substrate(
    root: &Path,
) -> Result<RoutingSubstrateReport> {
    let project = load_native_project(root)?;
    let board = build_native_project_board(&project)?;
    let component_pads = collect_native_project_component_pads(&project)?;
    Ok(board.routing_substrate(&component_pads))
}

pub(crate) fn render_native_project_routing_substrate_text(
    report: &RoutingSubstrateReport,
) -> String {
    let mut lines = vec![
        format!("contract: {}", report.contract),
        format!(
            "persisted_native_board_state_only: {}",
            report.persisted_native_board_state_only
        ),
        format!("outline_vertices: {}", report.summary.outline_vertex_count),
        format!("layers: {}", report.summary.layer_count),
        format!("copper_layers: {}", report.summary.copper_layer_count),
        format!("keepouts: {}", report.summary.keepout_count),
        format!("board_pads: {}", report.summary.board_pad_count),
        format!("component_pads: {}", report.summary.component_pad_count),
        format!("tracks: {}", report.summary.track_count),
        format!("vias: {}", report.summary.via_count),
        format!("zones: {}", report.summary.zone_count),
        format!("nets: {}", report.summary.net_count),
        format!("net_classes: {}", report.summary.net_class_count),
    ];

    if !report.copper_layer_ids.is_empty() {
        lines.push(format!(
            "copper_layer_ids: {}",
            report
                .copper_layer_ids
                .iter()
                .map(i32::to_string)
                .collect::<Vec<_>>()
                .join(",")
        ));
    }

    lines.join("\n")
}

fn collect_native_project_component_pads(
    project: &LoadedNativeProject,
) -> Result<Vec<RoutingComponentPad>> {
    let mut pads = Vec::new();
    for (component_key, component_pads) in &project.board.component_pads {
        let component_uuid = Uuid::parse_str(component_key).with_context(|| {
            format!(
                "failed to parse component UUID in {}",
                project.board_path.display()
            )
        })?;
        for pad in component_pads {
            pads.push(RoutingComponentPad {
                component_uuid,
                uuid: pad.uuid,
                name: pad.name.clone(),
                position: Point {
                    x: pad.position.x,
                    y: pad.position.y,
                },
                padstack_uuid: pad.padstack,
                layer: pad.layer,
                drill_nm: pad.drill_nm,
                shape: pad.shape,
                diameter_nm: pad.diameter_nm,
                width_nm: pad.width_nm,
                height_nm: pad.height_nm,
            });
        }
    }
    pads.sort_by(|a, b| {
        a.component_uuid
            .cmp(&b.component_uuid)
            .then_with(|| a.name.cmp(&b.name))
            .then_with(|| a.uuid.cmp(&b.uuid))
    });
    Ok(pads)
}
