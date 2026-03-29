use std::path::Path;

use anyhow::{Result, anyhow};
use eda_engine::board::{
    RoutePathCandidateAuthoredCopperGraphObstacleAwareReport, RoutePathCandidateStatus,
};
use uuid::Uuid;

use super::super::{build_native_project_board, load_native_project};

pub(crate) fn query_native_project_route_path_candidate_authored_copper_graph_obstacle_aware(
    root: &Path,
    net_uuid: Uuid,
    from_anchor_pad_uuid: Uuid,
    to_anchor_pad_uuid: Uuid,
) -> Result<RoutePathCandidateAuthoredCopperGraphObstacleAwareReport> {
    let project = load_native_project(root)?;
    let board = build_native_project_board(&project)?;
    board
        .route_path_candidate_authored_copper_graph_obstacle_aware(
            net_uuid,
            from_anchor_pad_uuid,
            to_anchor_pad_uuid,
        )
        .map_err(|err| anyhow!(err))
}

pub(crate) fn render_native_project_route_path_candidate_authored_copper_graph_obstacle_aware_text(
    report: &RoutePathCandidateAuthoredCopperGraphObstacleAwareReport,
) -> String {
    let mut lines = vec![
        format!("contract: {}", report.contract),
        format!(
            "persisted_native_board_state_only: {}",
            report.persisted_native_board_state_only
        ),
        format!("selection_rule: {}", report.selection_rule),
        format!("status: {}", render_status(report)),
        format!("net_uuid: {}", report.net_uuid),
        format!("net_name: {}", report.net_name),
        format!("from_anchor_pad_uuid: {}", report.from_anchor_pad_uuid),
        format!("to_anchor_pad_uuid: {}", report.to_anchor_pad_uuid),
        format!(
            "candidate_copper_layers: {}",
            report.summary.candidate_copper_layer_count
        ),
        format!("candidate_tracks: {}", report.summary.candidate_track_count),
        format!("candidate_vias: {}", report.summary.candidate_via_count),
        format!("blocked_tracks: {}", report.summary.blocked_track_count),
        format!("blocked_vias: {}", report.summary.blocked_via_count),
    ];

    if let Some(path) = &report.path {
        lines.push(format!("path_steps: {}", path.steps.len()));
    } else {
        lines.push("path_steps: 0".to_string());
    }

    lines.join("\n")
}

fn render_status(
    report: &RoutePathCandidateAuthoredCopperGraphObstacleAwareReport,
) -> &'static str {
    match report.status {
        RoutePathCandidateStatus::DeterministicPathFound => "deterministic_path_found",
        RoutePathCandidateStatus::NoPathUnderCurrentAuthoredConstraints => {
            "no_path_under_current_authored_constraints"
        }
    }
}
