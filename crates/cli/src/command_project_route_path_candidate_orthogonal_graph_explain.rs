use std::path::Path;

use anyhow::{Result, anyhow};
use eda_engine::board::{
    RoutePathCandidateOrthogonalGraphExplainKind, RoutePathCandidateOrthogonalGraphExplainReport,
    RoutePathCandidateStatus,
};
use uuid::Uuid;

use super::super::{build_native_project_board, load_native_project};

pub(crate) fn query_native_project_route_path_candidate_orthogonal_graph_explain(
    root: &Path,
    net_uuid: Uuid,
    from_anchor_pad_uuid: Uuid,
    to_anchor_pad_uuid: Uuid,
) -> Result<RoutePathCandidateOrthogonalGraphExplainReport> {
    let project = load_native_project(root)?;
    let board = build_native_project_board(&project)?;
    board
        .route_path_candidate_orthogonal_graph_explain(
            net_uuid,
            from_anchor_pad_uuid,
            to_anchor_pad_uuid,
        )
        .map_err(|err| anyhow!(err))
}

pub(crate) fn render_native_project_route_path_candidate_orthogonal_graph_explain_text(
    report: &RoutePathCandidateOrthogonalGraphExplainReport,
) -> String {
    let mut lines = vec![
        format!("contract: {}", report.contract),
        format!(
            "persisted_native_board_state_only: {}",
            report.persisted_native_board_state_only
        ),
        format!("status: {}", render_status(report.status.clone())),
        format!("explanation_kind: {}", render_kind(&report.explanation_kind)),
        format!("net_uuid: {}", report.net_uuid),
        format!("net_name: {}", report.net_name),
        format!("from_anchor_pad_uuid: {}", report.from_anchor_pad_uuid),
        format!("to_anchor_pad_uuid: {}", report.to_anchor_pad_uuid),
        format!("selection_rule: {}", report.selection_rule),
        format!(
            "candidate_copper_layers: {}",
            report.summary.candidate_copper_layer_count
        ),
        format!("graph_nodes: {}", report.summary.graph_node_count),
        format!("graph_edges: {}", report.summary.graph_edge_count),
        format!("blocked_edges: {}", report.summary.blocked_edge_count),
    ];
    if let Some(selected) = &report.selected_path {
        lines.push(format!("selected_path_layer: {}", selected.layer));
        lines.push(format!("selected_path_points: {}", selected.points.len()));
        lines.push(format!("selected_path_bends: {}", selected.cost.bend_count));
        lines.push(format!("selected_path_segments: {}", selected.cost.segment_count));
        lines.push(format!("selection_reason: {}", selected.selection_reason));
    } else {
        lines.push("selected_path_layer: none".to_string());
        lines.push("selected_path_points: 0".to_string());
        lines.push("selected_path_bends: 0".to_string());
        lines.push("selected_path_segments: 0".to_string());
    }
    lines.push(format!("blocked_edge_count: {}", report.blocked_edges.len()));
    lines.join("\n")
}

fn render_status(status: RoutePathCandidateStatus) -> &'static str {
    match status {
        RoutePathCandidateStatus::DeterministicPathFound => "deterministic_path_found",
        RoutePathCandidateStatus::NoPathUnderCurrentAuthoredConstraints => {
            "no_path_under_current_authored_constraints"
        }
    }
}

fn render_kind(kind: &RoutePathCandidateOrthogonalGraphExplainKind) -> &'static str {
    match kind {
        RoutePathCandidateOrthogonalGraphExplainKind::DeterministicPathFound => {
            "deterministic_path_found"
        }
        RoutePathCandidateOrthogonalGraphExplainKind::NoSameLayerGraphCandidate => {
            "no_same_layer_graph_candidate"
        }
        RoutePathCandidateOrthogonalGraphExplainKind::AllGraphPathsBlocked => {
            "all_graph_paths_blocked"
        }
    }
}
