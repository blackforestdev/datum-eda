use std::path::Path;

use anyhow::{Result, anyhow};
use eda_engine::board::RoutePathCandidateOrthogonalGraphViaReport;
use uuid::Uuid;

use super::command_project_route_path_candidate_orthogonal_graph_spine::{
    render_route_path_candidate_status, with_native_project_board,
};

pub(crate) fn query_native_project_route_path_candidate_orthogonal_graph_via(
    root: &Path,
    net_uuid: Uuid,
    from_anchor_pad_uuid: Uuid,
    to_anchor_pad_uuid: Uuid,
) -> Result<RoutePathCandidateOrthogonalGraphViaReport> {
    with_native_project_board(root, |board| {
        board
            .route_path_candidate_orthogonal_graph_via(
                net_uuid,
                from_anchor_pad_uuid,
                to_anchor_pad_uuid,
            )
            .map_err(|err| anyhow!(err))
    })
}

pub(crate) fn render_native_project_route_path_candidate_orthogonal_graph_via_text(
    report: &RoutePathCandidateOrthogonalGraphViaReport,
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
        format!("candidate_vias: {}", report.summary.candidate_via_count),
        format!("matching_vias: {}", report.summary.matching_via_count),
        format!("available_vias: {}", report.summary.available_via_count),
        format!("blocked_vias: {}", report.summary.blocked_via_count),
    ];
    if let Some(path) = &report.path {
        lines.push(format!("via_uuid: {}", path.via_uuid));
        lines.push(format!("path_segments: {}", path.segments.len()));
    } else {
        lines.push("via_uuid: none".to_string());
        lines.push("path_segments: 0".to_string());
    }
    lines.join("\n")
}
