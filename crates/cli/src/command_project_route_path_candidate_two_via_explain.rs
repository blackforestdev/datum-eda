use std::path::Path;

use anyhow::{Result, anyhow};
use eda_engine::board::{
    RoutePathCandidateStatus, RoutePathCandidateTwoViaExplainKind,
    RoutePathCandidateTwoViaExplainReport,
};
use uuid::Uuid;

use super::super::{build_native_project_board, load_native_project};

pub(crate) fn query_native_project_route_path_candidate_two_via_explain(
    root: &Path,
    net_uuid: Uuid,
    from_anchor_pad_uuid: Uuid,
    to_anchor_pad_uuid: Uuid,
) -> Result<RoutePathCandidateTwoViaExplainReport> {
    let project = load_native_project(root)?;
    let board = build_native_project_board(&project)?;
    board
        .route_path_candidate_two_via_explain(net_uuid, from_anchor_pad_uuid, to_anchor_pad_uuid)
        .map_err(|err| anyhow!(err))
}

pub(crate) fn render_native_project_route_path_candidate_two_via_explain_text(
    report: &RoutePathCandidateTwoViaExplainReport,
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
        format!("candidate_via_pairs: {}", report.summary.candidate_via_pair_count),
        format!("matching_via_pairs: {}", report.summary.matching_via_pair_count),
        format!("available_via_pairs: {}", report.summary.available_via_pair_count),
        format!("blocked_via_pairs: {}", report.summary.blocked_via_pair_count),
    ];

    if let Some(pair) = &report.selected_pair {
        lines.push(format!("selected_via_a_uuid: {}", pair.via_a_uuid));
        lines.push(format!("selected_via_b_uuid: {}", pair.via_b_uuid));
        lines.push(format!("selected_intermediate_layer: {}", pair.intermediate_layer));
        lines.push(format!("selection_reason: {}", pair.selection_reason));
    } else {
        lines.push("selected_via_a_uuid: none".to_string());
        lines.push("selected_via_b_uuid: none".to_string());
        lines.push("selected_intermediate_layer: none".to_string());
    }

    lines.push(format!(
        "blocked_matching_pair_count: {}",
        report.blocked_matching_pairs.len()
    ));
    lines.join("\n")
}

fn render_status(report: &RoutePathCandidateTwoViaExplainReport) -> &'static str {
    match report.status {
        RoutePathCandidateStatus::DeterministicPathFound => "deterministic_path_found",
        RoutePathCandidateStatus::NoPathUnderCurrentAuthoredConstraints => {
            "no_path_under_current_authored_constraints"
        }
    }
}

fn render_kind(report: &RoutePathCandidateTwoViaExplainReport) -> &'static str {
    match report.explanation_kind {
        RoutePathCandidateTwoViaExplainKind::DeterministicPathFound => "deterministic_path_found",
        RoutePathCandidateTwoViaExplainKind::NoMatchingAuthoredViaPair => {
            "no_matching_authored_via_pair"
        }
        RoutePathCandidateTwoViaExplainKind::AllMatchingViaPairsBlocked => {
            "all_matching_via_pairs_blocked"
        }
    }
}
