use std::path::Path;

use anyhow::{Result, anyhow};
use eda_engine::board::{
    RoutePathCandidateAuthoredCopperGraphPolicy as EngineAuthoredCopperGraphPolicy,
    RoutePathCandidateAuthoredCopperGraphPolicyExplainKind,
    RoutePathCandidateAuthoredCopperGraphPolicyExplainReport, RoutePathCandidateStatus,
};
use uuid::Uuid;

use crate::cli_args::NativeRoutePathCandidateAuthoredCopperGraphPolicy;

use super::super::{build_native_project_board, load_native_project};

pub(crate) fn query_native_project_route_path_candidate_authored_copper_graph_explain(
    root: &Path,
    net_uuid: Uuid,
    from_anchor_pad_uuid: Uuid,
    to_anchor_pad_uuid: Uuid,
    policy: NativeRoutePathCandidateAuthoredCopperGraphPolicy,
) -> Result<RoutePathCandidateAuthoredCopperGraphPolicyExplainReport> {
    let project = load_native_project(root)?;
    let board = build_native_project_board(&project)?;
    board
        .route_path_candidate_authored_copper_graph_explain_by_policy(
            net_uuid,
            from_anchor_pad_uuid,
            to_anchor_pad_uuid,
            map_policy(policy),
        )
        .map_err(|err| anyhow!(err))
}

pub(crate) fn render_native_project_route_path_candidate_authored_copper_graph_explain_text(
    report: &RoutePathCandidateAuthoredCopperGraphPolicyExplainReport,
) -> String {
    let mut lines = vec![
        format!("contract: {}", report.contract),
        format!(
            "persisted_native_board_state_only: {}",
            report.persisted_native_board_state_only
        ),
        format!("policy: {}", render_policy(report.policy)),
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
        format!("candidate_tracks: {}", report.summary.candidate_track_count),
        format!("candidate_vias: {}", report.summary.candidate_via_count),
        format!("candidate_zones: {}", report.summary.candidate_zone_count),
        format!("blocked_tracks: {}", report.summary.blocked_track_count),
        format!("blocked_vias: {}", report.summary.blocked_via_count),
        format!(
            "blocked_zone_connections: {}",
            report.summary.blocked_zone_connection_count
        ),
        format!(
            "topology_transitions: {}",
            report.summary.topology_transition_count
        ),
        format!(
            "layer_balance_score: {}",
            report.summary.layer_balance_score
        ),
        format!("path_via_steps: {}", report.summary.path_via_step_count),
        format!("path_zone_steps: {}", report.summary.path_zone_step_count),
    ];

    if let Some(path) = &report.selected_path {
        lines.push(format!("selected_path_steps: {}", path.steps.len()));
        lines.push(format!("selection_reason: {}", path.selection_reason));
    } else {
        lines.push("selected_path_steps: 0".to_string());
    }

    lines.join("\n")
}

fn render_status(
    report: &RoutePathCandidateAuthoredCopperGraphPolicyExplainReport,
) -> &'static str {
    match report.status {
        RoutePathCandidateStatus::DeterministicPathFound => "deterministic_path_found",
        RoutePathCandidateStatus::NoPathUnderCurrentAuthoredConstraints => {
            "no_path_under_current_authored_constraints"
        }
    }
}

fn render_kind(report: &RoutePathCandidateAuthoredCopperGraphPolicyExplainReport) -> &'static str {
    match report.explanation_kind {
        RoutePathCandidateAuthoredCopperGraphPolicyExplainKind::DeterministicPathFound => {
            "deterministic_path_found"
        }
        RoutePathCandidateAuthoredCopperGraphPolicyExplainKind::NoExistingAuthoredCopperPath => {
            "no_existing_authored_copper_path"
        }
    }
}

fn map_policy(
    policy: NativeRoutePathCandidateAuthoredCopperGraphPolicy,
) -> EngineAuthoredCopperGraphPolicy {
    match policy {
        NativeRoutePathCandidateAuthoredCopperGraphPolicy::Plain => {
            EngineAuthoredCopperGraphPolicy::Plain
        }
        NativeRoutePathCandidateAuthoredCopperGraphPolicy::ZoneAware => {
            EngineAuthoredCopperGraphPolicy::ZoneAware
        }
        NativeRoutePathCandidateAuthoredCopperGraphPolicy::ObstacleAware => {
            EngineAuthoredCopperGraphPolicy::ObstacleAware
        }
        NativeRoutePathCandidateAuthoredCopperGraphPolicy::ZoneObstacleAware => {
            EngineAuthoredCopperGraphPolicy::ZoneObstacleAware
        }
        NativeRoutePathCandidateAuthoredCopperGraphPolicy::ZoneObstacleTopologyAware => {
            EngineAuthoredCopperGraphPolicy::ZoneObstacleTopologyAware
        }
        NativeRoutePathCandidateAuthoredCopperGraphPolicy::ZoneObstacleTopologyLayerBalanceAware => {
            EngineAuthoredCopperGraphPolicy::ZoneObstacleTopologyLayerBalanceAware
        }
    }
}

fn render_policy(policy: EngineAuthoredCopperGraphPolicy) -> &'static str {
    match policy {
        EngineAuthoredCopperGraphPolicy::Plain => "plain",
        EngineAuthoredCopperGraphPolicy::ZoneAware => "zone_aware",
        EngineAuthoredCopperGraphPolicy::ObstacleAware => "obstacle_aware",
        EngineAuthoredCopperGraphPolicy::ZoneObstacleAware => "zone_obstacle_aware",
        EngineAuthoredCopperGraphPolicy::ZoneObstacleTopologyAware => {
            "zone_obstacle_topology_aware"
        }
        EngineAuthoredCopperGraphPolicy::ZoneObstacleTopologyLayerBalanceAware => {
            "zone_obstacle_topology_layer_balance_aware"
        }
    }
}
