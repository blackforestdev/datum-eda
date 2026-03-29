use std::path::Path;

use anyhow::{Result, anyhow};
use eda_engine::board::{
    RoutePathCandidateFourViaExplainKind, RoutePathCandidateFourViaExplainReport,
    RoutePathCandidateStatus,
};
use uuid::Uuid;

use super::super::{build_native_project_board, load_native_project};

pub(crate) fn query_native_project_route_path_candidate_four_via_explain(
    root: &Path,
    net_uuid: Uuid,
    from_anchor_pad_uuid: Uuid,
    to_anchor_pad_uuid: Uuid,
) -> Result<RoutePathCandidateFourViaExplainReport> {
    let project = load_native_project(root)?;
    let board = build_native_project_board(&project)?;
    board
        .route_path_candidate_four_via_explain(net_uuid, from_anchor_pad_uuid, to_anchor_pad_uuid)
        .map_err(|err| anyhow!(err))
}

pub(crate) fn render_native_project_route_path_candidate_four_via_explain_text(
    report: &RoutePathCandidateFourViaExplainReport,
) -> String {
    let mut lines = vec![
        format!("contract: {}", report.contract),
        format!(
            "persisted_native_board_state_only: {}",
            report.persisted_native_board_state_only
        ),
        format!("status: {}", render_status(report)),
        format!("explanation_kind: {}", render_kind(report)),
        format!("net_uuid: {}", report.net_uuid),
        format!("net_name: {}", report.net_name),
        format!("from_anchor_pad_uuid: {}", report.from_anchor_pad_uuid),
        format!("to_anchor_pad_uuid: {}", report.to_anchor_pad_uuid),
        format!("selection_rule: {}", report.selection_rule),
        format!(
            "candidate_copper_layers: {}",
            report.summary.candidate_copper_layer_count
        ),
        format!("candidate_vias: {}", report.summary.candidate_via_count),
        format!(
            "candidate_via_quadruples: {}",
            report.summary.candidate_via_quadruple_count
        ),
        format!(
            "matching_via_quadruples: {}",
            report.summary.matching_via_quadruple_count
        ),
        format!(
            "available_via_quadruples: {}",
            report.summary.available_via_quadruple_count
        ),
        format!(
            "blocked_via_quadruples: {}",
            report.summary.blocked_via_quadruple_count
        ),
    ];

    if let Some(quadruple) = &report.selected_quadruple {
        lines.push(format!("selected_via_a_uuid: {}", quadruple.via_a_uuid));
        lines.push(format!("selected_via_b_uuid: {}", quadruple.via_b_uuid));
        lines.push(format!("selected_via_c_uuid: {}", quadruple.via_c_uuid));
        lines.push(format!("selected_via_d_uuid: {}", quadruple.via_d_uuid));
        lines.push(format!(
            "selected_first_intermediate_layer: {}",
            quadruple.first_intermediate_layer
        ));
        lines.push(format!(
            "selected_second_intermediate_layer: {}",
            quadruple.second_intermediate_layer
        ));
        lines.push(format!(
            "selected_third_intermediate_layer: {}",
            quadruple.third_intermediate_layer
        ));
        lines.push(format!("selection_reason: {}", quadruple.selection_reason));
    } else {
        lines.push("selected_via_a_uuid: none".to_string());
        lines.push("selected_via_b_uuid: none".to_string());
        lines.push("selected_via_c_uuid: none".to_string());
        lines.push("selected_via_d_uuid: none".to_string());
        lines.push("selected_first_intermediate_layer: none".to_string());
        lines.push("selected_second_intermediate_layer: none".to_string());
        lines.push("selected_third_intermediate_layer: none".to_string());
    }

    lines.push(format!(
        "blocked_matching_quadruple_count: {}",
        report.blocked_matching_quadruples.len()
    ));
    lines.join("\n")
}

fn render_status(report: &RoutePathCandidateFourViaExplainReport) -> &'static str {
    match report.status {
        RoutePathCandidateStatus::DeterministicPathFound => "deterministic_path_found",
        RoutePathCandidateStatus::NoPathUnderCurrentAuthoredConstraints => {
            "no_path_under_current_authored_constraints"
        }
    }
}

fn render_kind(report: &RoutePathCandidateFourViaExplainReport) -> &'static str {
    match report.explanation_kind {
        RoutePathCandidateFourViaExplainKind::DeterministicPathFound => "deterministic_path_found",
        RoutePathCandidateFourViaExplainKind::NoMatchingAuthoredViaQuadruple => {
            "no_matching_authored_via_quadruple"
        }
        RoutePathCandidateFourViaExplainKind::AllMatchingViaQuadruplesBlocked => {
            "all_matching_via_quadruples_blocked"
        }
    }
}
