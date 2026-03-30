use std::path::Path;

use anyhow::{Result, anyhow};
use eda_engine::board::{RoutePathCandidateOrthogonalTwoBendReport, RoutePathCandidateStatus};
use uuid::Uuid;

use super::super::{build_native_project_board, load_native_project};

pub(crate) fn query_native_project_route_path_candidate_orthogonal_two_bend(
    root: &Path,
    net_uuid: Uuid,
    from_anchor_pad_uuid: Uuid,
    to_anchor_pad_uuid: Uuid,
) -> Result<RoutePathCandidateOrthogonalTwoBendReport> {
    let project = load_native_project(root)?;
    let board = build_native_project_board(&project)?;
    board
        .route_path_candidate_orthogonal_two_bend(
            net_uuid,
            from_anchor_pad_uuid,
            to_anchor_pad_uuid,
        )
        .map_err(|err| anyhow!(err))
}

pub(crate) fn render_native_project_route_path_candidate_orthogonal_two_bend_text(
    report: &RoutePathCandidateOrthogonalTwoBendReport,
) -> String {
    let mut lines = vec![
        format!("contract: {}", report.contract),
        format!(
            "persisted_native_board_state_only: {}",
            report.persisted_native_board_state_only
        ),
        format!("selection_rule: {}", report.selection_rule),
        format!("status: {}", render_status(report.status.clone())),
        format!("net_uuid: {}", report.net_uuid),
        format!("net_name: {}", report.net_name),
        format!("from_anchor_pad_uuid: {}", report.from_anchor_pad_uuid),
        format!("to_anchor_pad_uuid: {}", report.to_anchor_pad_uuid),
        format!(
            "candidate_copper_layers: {}",
            report.summary.candidate_copper_layer_count
        ),
        format!("candidate_paths: {}", report.summary.candidate_path_count),
        format!("available_paths: {}", report.summary.available_path_count),
        format!("blocked_paths: {}", report.summary.blocked_path_count),
    ];
    if let Some(path) = &report.path {
        lines.push(format!("path_layer: {}", path.layer));
        lines.push(format!("path_points: {}", path.points.len()));
        lines.push(format!("detour_coordinate: {}", path.detour_coordinate));
    } else {
        lines.push("path_layer: none".to_string());
        lines.push("path_points: 0".to_string());
        lines.push("detour_coordinate: none".to_string());
    }
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
