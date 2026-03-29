use std::path::Path;

use anyhow::{Result, anyhow};
use eda_engine::board::{RoutePathCandidateSixViaReport, RoutePathCandidateStatus};
use uuid::Uuid;

use super::super::{build_native_project_board, load_native_project};

pub(crate) fn query_native_project_route_path_candidate_six_via(
    root: &Path,
    net_uuid: Uuid,
    from_anchor_pad_uuid: Uuid,
    to_anchor_pad_uuid: Uuid,
) -> Result<RoutePathCandidateSixViaReport> {
    let project = load_native_project(root)?;
    let board = build_native_project_board(&project)?;
    board
        .route_path_candidate_six_via(net_uuid, from_anchor_pad_uuid, to_anchor_pad_uuid)
        .map_err(|err| anyhow!(err))
}

pub(crate) fn render_native_project_route_path_candidate_six_via_text(
    report: &RoutePathCandidateSixViaReport,
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
        format!("candidate_vias: {}", report.summary.candidate_via_count),
        format!(
            "candidate_via_sextuples: {}",
            report.summary.candidate_via_sextuple_count
        ),
        format!(
            "matching_via_sextuples: {}",
            report.summary.matching_via_sextuple_count
        ),
        format!(
            "available_via_sextuples: {}",
            report.summary.available_via_sextuple_count
        ),
        format!(
            "blocked_via_sextuples: {}",
            report.summary.blocked_via_sextuple_count
        ),
    ];

    if let Some(path) = &report.path {
        lines.push(format!("path_via_a_uuid: {}", path.via_a_uuid));
        lines.push(format!("path_via_b_uuid: {}", path.via_b_uuid));
        lines.push(format!("path_via_c_uuid: {}", path.via_c_uuid));
        lines.push(format!("path_via_d_uuid: {}", path.via_d_uuid));
        lines.push(format!("path_via_e_uuid: {}", path.via_e_uuid));
        lines.push(format!("path_via_f_uuid: {}", path.via_f_uuid));
        lines.push(format!(
            "path_first_intermediate_layer: {}",
            path.first_intermediate_layer
        ));
        lines.push(format!(
            "path_second_intermediate_layer: {}",
            path.second_intermediate_layer
        ));
        lines.push(format!(
            "path_third_intermediate_layer: {}",
            path.third_intermediate_layer
        ));
        lines.push(format!(
            "path_fourth_intermediate_layer: {}",
            path.fourth_intermediate_layer
        ));
        lines.push(format!(
            "path_fifth_intermediate_layer: {}",
            path.fifth_intermediate_layer
        ));
        lines.push(format!("path_segments: {}", path.segments.len()));
    } else {
        lines.push("path_via_a_uuid: none".to_string());
        lines.push("path_via_b_uuid: none".to_string());
        lines.push("path_via_c_uuid: none".to_string());
        lines.push("path_via_d_uuid: none".to_string());
        lines.push("path_via_e_uuid: none".to_string());
        lines.push("path_via_f_uuid: none".to_string());
        lines.push("path_first_intermediate_layer: none".to_string());
        lines.push("path_second_intermediate_layer: none".to_string());
        lines.push("path_third_intermediate_layer: none".to_string());
        lines.push("path_fourth_intermediate_layer: none".to_string());
        lines.push("path_fifth_intermediate_layer: none".to_string());
        lines.push("path_segments: 0".to_string());
    }

    lines.join("\n")
}

fn render_status(report: &RoutePathCandidateSixViaReport) -> &'static str {
    match report.status {
        RoutePathCandidateStatus::DeterministicPathFound => "deterministic_path_found",
        RoutePathCandidateStatus::NoPathUnderCurrentAuthoredConstraints => {
            "no_path_under_current_authored_constraints"
        }
    }
}
