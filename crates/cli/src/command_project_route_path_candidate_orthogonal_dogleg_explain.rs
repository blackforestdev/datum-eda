use std::path::Path;

use anyhow::{Result, anyhow};
use eda_engine::board::{
    RoutePathCandidateOrthogonalDoglegExplainKind,
    RoutePathCandidateOrthogonalDoglegExplainReport, RoutePathCandidateStatus,
};
use uuid::Uuid;

use super::super::{build_native_project_board, load_native_project};

pub(crate) fn query_native_project_route_path_candidate_orthogonal_dogleg_explain(
    root: &Path,
    net_uuid: Uuid,
    from_anchor_pad_uuid: Uuid,
    to_anchor_pad_uuid: Uuid,
) -> Result<RoutePathCandidateOrthogonalDoglegExplainReport> {
    let project = load_native_project(root)?;
    let board = build_native_project_board(&project)?;
    board
        .route_path_candidate_orthogonal_dogleg_explain(
            net_uuid,
            from_anchor_pad_uuid,
            to_anchor_pad_uuid,
        )
        .map_err(|err| anyhow!(err))
}

pub(crate) fn render_native_project_route_path_candidate_orthogonal_dogleg_explain_text(
    report: &RoutePathCandidateOrthogonalDoglegExplainReport,
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
        format!("candidate_corners: {}", report.summary.candidate_corner_count),
        format!("available_corners: {}", report.summary.available_corner_count),
        format!("blocked_corners: {}", report.summary.blocked_corner_count),
    ];
    if let Some(selected) = &report.selected_dogleg {
        lines.push(format!("selected_dogleg_layer: {}", selected.layer));
        lines.push(format!(
            "selected_corner: {},{}",
            selected.corner.x, selected.corner.y
        ));
        lines.push(format!("selection_reason: {}", selected.selection_reason));
    } else {
        lines.push("selected_dogleg_layer: none".to_string());
        lines.push("selected_corner: none".to_string());
    }
    lines.push(format!("blocked_doglegs: {}", report.blocked_doglegs.len()));
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

fn render_kind(kind: &RoutePathCandidateOrthogonalDoglegExplainKind) -> &'static str {
    match kind {
        RoutePathCandidateOrthogonalDoglegExplainKind::DeterministicPathFound => {
            "deterministic_path_found"
        }
        RoutePathCandidateOrthogonalDoglegExplainKind::NoSameLayerDoglegCandidate => {
            "no_same_layer_dogleg_candidate"
        }
        RoutePathCandidateOrthogonalDoglegExplainKind::AllDoglegCandidatesBlocked => {
            "all_dogleg_candidates_blocked"
        }
    }
}
