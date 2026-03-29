use std::path::Path;

use anyhow::{Result, anyhow};
use eda_engine::board::{RouteCorridorReport, RouteCorridorStatus};
use uuid::Uuid;

use super::super::{build_native_project_board, load_native_project};

pub(crate) fn query_native_project_route_corridor(
    root: &Path,
    net_uuid: Uuid,
) -> Result<RouteCorridorReport> {
    let project = load_native_project(root)?;
    let board = build_native_project_board(&project)?;
    board
        .route_corridor(net_uuid)
        .ok_or_else(|| anyhow!("board net not found in native project: {net_uuid}"))
}

pub(crate) fn render_native_project_route_corridor_text(report: &RouteCorridorReport) -> String {
    [
        format!("contract: {}", report.contract),
        format!(
            "persisted_native_board_state_only: {}",
            report.persisted_native_board_state_only
        ),
        format!("status: {}", render_route_corridor_status(report)),
        format!("net_uuid: {}", report.net_uuid),
        format!("net_name: {}", report.net_name),
        format!("net_class_uuid: {}", report.net_class_uuid),
        format!("anchors: {}", report.summary.anchor_count),
        format!(
            "candidate_copper_layers: {}",
            report.summary.candidate_copper_layer_count
        ),
        format!("anchor_pairs: {}", report.summary.anchor_pair_count),
        format!("authored_obstacles: {}", report.summary.obstacle_count),
        format!("corridor_spans: {}", report.summary.span_count),
        format!("available_spans: {}", report.summary.available_span_count),
        format!("blocked_spans: {}", report.summary.blocked_span_count),
    ]
    .join("\n")
}

fn render_route_corridor_status(report: &RouteCorridorReport) -> &'static str {
    match report.status {
        RouteCorridorStatus::CorridorAvailable => "corridor_available",
        RouteCorridorStatus::CorridorBlocked => "corridor_blocked",
        RouteCorridorStatus::InsufficientAuthoredInputs => "insufficient_authored_inputs",
    }
}
