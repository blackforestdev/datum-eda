use std::path::Path;

use anyhow::{Result, anyhow};
use eda_engine::board::{
    RoutePathCandidateFiveViaExplainKind, RoutePathCandidateFiveViaExplainReport,
    RoutePathCandidateStatus,
};
use uuid::Uuid;

use super::super::{build_native_project_board, load_native_project};

pub(crate) fn query_native_project_route_path_candidate_five_via_explain(
    root: &Path,
    net_uuid: Uuid,
    from_anchor_pad_uuid: Uuid,
    to_anchor_pad_uuid: Uuid,
) -> Result<RoutePathCandidateFiveViaExplainReport> {
    let project = load_native_project(root)?;
    let board = build_native_project_board(&project)?;
    board
        .route_path_candidate_five_via_explain(net_uuid, from_anchor_pad_uuid, to_anchor_pad_uuid)
        .map_err(|err| anyhow!(err))
}

pub(crate) fn render_native_project_route_path_candidate_five_via_explain_text(
    report: &RoutePathCandidateFiveViaExplainReport,
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
            "candidate_via_quintuples: {}",
            report.summary.candidate_via_quintuple_count
        ),
        format!(
            "matching_via_quintuples: {}",
            report.summary.matching_via_quintuple_count
        ),
        format!(
            "available_via_quintuples: {}",
            report.summary.available_via_quintuple_count
        ),
        format!(
            "blocked_via_quintuples: {}",
            report.summary.blocked_via_quintuple_count
        ),
    ];

    if let Some(quintuple) = &report.selected_quintuple {
        lines.push(format!("selected_via_a_uuid: {}", quintuple.via_a_uuid));
        lines.push(format!("selected_via_b_uuid: {}", quintuple.via_b_uuid));
        lines.push(format!("selected_via_c_uuid: {}", quintuple.via_c_uuid));
        lines.push(format!("selected_via_d_uuid: {}", quintuple.via_d_uuid));
        lines.push(format!("selected_via_e_uuid: {}", quintuple.via_e_uuid));
        lines.push(format!(
            "selected_first_intermediate_layer: {}",
            quintuple.first_intermediate_layer
        ));
        lines.push(format!(
            "selected_second_intermediate_layer: {}",
            quintuple.second_intermediate_layer
        ));
        lines.push(format!(
            "selected_third_intermediate_layer: {}",
            quintuple.third_intermediate_layer
        ));
        lines.push(format!(
            "selected_fourth_intermediate_layer: {}",
            quintuple.fourth_intermediate_layer
        ));
        lines.push(format!("selection_reason: {}", quintuple.selection_reason));
    } else {
        lines.push("selected_via_a_uuid: none".to_string());
        lines.push("selected_via_b_uuid: none".to_string());
        lines.push("selected_via_c_uuid: none".to_string());
        lines.push("selected_via_d_uuid: none".to_string());
        lines.push("selected_via_e_uuid: none".to_string());
        lines.push("selected_first_intermediate_layer: none".to_string());
        lines.push("selected_second_intermediate_layer: none".to_string());
        lines.push("selected_third_intermediate_layer: none".to_string());
        lines.push("selected_fourth_intermediate_layer: none".to_string());
    }

    lines.push(format!(
        "blocked_matching_quintuple_count: {}",
        report.blocked_matching_quintuples.len()
    ));
    lines.join("\n")
}

fn render_status(report: &RoutePathCandidateFiveViaExplainReport) -> &'static str {
    match report.status {
        RoutePathCandidateStatus::DeterministicPathFound => "deterministic_path_found",
        RoutePathCandidateStatus::NoPathUnderCurrentAuthoredConstraints => {
            "no_path_under_current_authored_constraints"
        }
    }
}

fn render_kind(report: &RoutePathCandidateFiveViaExplainReport) -> &'static str {
    match report.explanation_kind {
        RoutePathCandidateFiveViaExplainKind::DeterministicPathFound => "deterministic_path_found",
        RoutePathCandidateFiveViaExplainKind::NoMatchingAuthoredViaQuintuple => {
            "no_matching_authored_via_quintuple"
        }
        RoutePathCandidateFiveViaExplainKind::AllMatchingViaQuintuplesBlocked => {
            "all_matching_via_quintuples_blocked"
        }
    }
}
