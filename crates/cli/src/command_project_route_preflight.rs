use std::path::Path;

use anyhow::{Result, anyhow};
use eda_engine::board::RoutePreflightReport;
use uuid::Uuid;

use super::super::{build_native_project_board, load_native_project};

pub(crate) fn query_native_project_route_preflight(
    root: &Path,
    net_uuid: Uuid,
) -> Result<RoutePreflightReport> {
    let project = load_native_project(root)?;
    let board = build_native_project_board(&project)?;
    board
        .route_preflight(net_uuid)
        .ok_or_else(|| anyhow!("board net not found in native project: {net_uuid}"))
}

pub(crate) fn render_native_project_route_preflight_text(report: &RoutePreflightReport) -> String {
    [
        format!("contract: {}", report.contract),
        format!(
            "persisted_native_board_state_only: {}",
            report.persisted_native_board_state_only
        ),
        format!("status: {}", render_route_preflight_status(report)),
        format!("net_uuid: {}", report.net_uuid),
        format!("net_name: {}", report.net_name),
        format!("net_class_uuid: {}", report.net_class_uuid),
        format!("anchors: {}", report.summary.anchor_count),
        format!(
            "candidate_copper_layers: {}",
            report.summary.candidate_copper_layer_count
        ),
        format!(
            "keepout_conflicts: {}",
            report.summary.keepout_conflict_count
        ),
        format!("foreign_tracks: {}", report.summary.foreign_track_count),
        format!("foreign_vias: {}", report.summary.foreign_via_count),
        format!("foreign_zones: {}", report.summary.foreign_zone_count),
        format!(
            "outside_outline_conflicts: {}",
            report.summary.outside_outline_count
        ),
    ]
    .join("\n")
}

fn render_route_preflight_status(report: &RoutePreflightReport) -> &'static str {
    match report.status {
        eda_engine::board::RoutePreflightStatus::PreflightReady => "preflight_ready",
        eda_engine::board::RoutePreflightStatus::BlockedByAuthoredObstacle => {
            "blocked_by_authored_obstacle"
        }
        eda_engine::board::RoutePreflightStatus::InsufficientAuthoredInputs => {
            "insufficient_authored_inputs"
        }
    }
}
