use std::path::Path;

use anyhow::{Result, anyhow};
use eda_engine::board::RoutePathCandidateOrthogonalGraphReport;
use uuid::Uuid;

use super::command_project_route_path_candidate_orthogonal_graph_spine::{
    push_orthogonal_graph_segment_evidence_lines, render_route_path_candidate_status,
    with_native_project_board,
};

pub(crate) fn query_native_project_route_path_candidate_orthogonal_graph(
    root: &Path,
    net_uuid: Uuid,
    from_anchor_pad_uuid: Uuid,
    to_anchor_pad_uuid: Uuid,
) -> Result<RoutePathCandidateOrthogonalGraphReport> {
    with_native_project_board(root, |board| {
        board
            .route_path_candidate_orthogonal_graph(
                net_uuid,
                from_anchor_pad_uuid,
                to_anchor_pad_uuid,
            )
            .map_err(|err| anyhow!(err))
    })
}

pub(crate) fn render_native_project_route_path_candidate_orthogonal_graph_text(
    report: &RoutePathCandidateOrthogonalGraphReport,
) -> String {
    let mut lines = vec![
        format!("contract: {}", report.contract),
        format!(
            "persisted_native_board_state_only: {}",
            report.persisted_native_board_state_only
        ),
        format!("selection_rule: {}", report.selection_rule),
        format!(
            "status: {}",
            render_route_path_candidate_status(report.status.clone())
        ),
        format!("net_uuid: {}", report.net_uuid),
        format!("net_name: {}", report.net_name),
        format!("from_anchor_pad_uuid: {}", report.from_anchor_pad_uuid),
        format!("to_anchor_pad_uuid: {}", report.to_anchor_pad_uuid),
        format!(
            "candidate_copper_layers: {}",
            report.summary.candidate_copper_layer_count
        ),
        format!("graph_nodes: {}", report.summary.graph_node_count),
        format!("graph_edges: {}", report.summary.graph_edge_count),
        format!("blocked_edges: {}", report.summary.blocked_edge_count),
    ];
    if let Some(path) = &report.path {
        lines.push(format!("path_layer: {}", path.layer));
        lines.push(format!("path_points: {}", path.points.len()));
        lines.push(format!("path_bends: {}", path.cost.bend_count));
        lines.push(format!("path_segments: {}", path.cost.segment_count));
    } else {
        lines.push("path_layer: none".to_string());
        lines.push("path_points: 0".to_string());
        lines.push("path_bends: 0".to_string());
        lines.push("path_segments: 0".to_string());
    }
    push_orthogonal_graph_segment_evidence_lines(
        &mut lines,
        report.segment_evidence.iter().map(|segment| {
            (
                segment.layer_segment_index,
                segment.layer_segment_count,
                segment.layer.to_string(),
                segment.bend_count,
                segment.point_count,
                segment.track_action_count,
            )
        }),
    );
    lines.join("\n")
}
