use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::board::{
    Board, RoutePathCandidateAuthoredCopperGraphExplainReport,
    RoutePathCandidateAuthoredCopperGraphObstacleAwareExplainReport,
    RoutePathCandidateAuthoredCopperGraphPolicy,
    RoutePathCandidateAuthoredCopperGraphPolicyStep,
    RoutePathCandidateAuthoredCopperGraphPolicyStepKindView,
    RoutePathCandidateAuthoredCopperGraphZoneAwareExplainReport,
    RoutePathCandidateAuthoredCopperGraphZoneObstacleAwareExplainReport,
    RoutePathCandidateAuthoredCopperGraphZoneObstacleAwareTopologyAwareExplainReport,
    RoutePathCandidateAuthoredCopperGraphZoneObstacleAwareTopologyAwareLayerBalanceAwareExplainReport,
    RoutePathCandidateError, RoutePathCandidateStatus, StackupLayer,
};

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum RoutePathCandidateAuthoredCopperGraphPolicyExplainKind {
    DeterministicPathFound,
    NoExistingAuthoredCopperPath,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct RoutePathCandidateAuthoredCopperGraphPolicyExplainSelectedPath {
    pub steps: Vec<RoutePathCandidateAuthoredCopperGraphPolicyStep>,
    pub selection_reason: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct RoutePathCandidateAuthoredCopperGraphPolicyExplainSummary {
    pub candidate_copper_layer_count: usize,
    pub candidate_track_count: usize,
    pub candidate_via_count: usize,
    pub candidate_zone_count: usize,
    pub blocked_track_count: usize,
    pub blocked_via_count: usize,
    pub blocked_zone_connection_count: usize,
    pub path_step_count: usize,
    pub topology_transition_count: usize,
    pub layer_balance_score: usize,
    pub path_via_step_count: usize,
    pub path_zone_step_count: usize,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct RoutePathCandidateAuthoredCopperGraphPolicyExplainReport {
    pub contract: String,
    pub persisted_native_board_state_only: bool,
    pub policy: RoutePathCandidateAuthoredCopperGraphPolicy,
    pub status: RoutePathCandidateStatus,
    pub explanation_kind: RoutePathCandidateAuthoredCopperGraphPolicyExplainKind,
    pub net_uuid: Uuid,
    pub net_name: String,
    pub from_anchor_pad_uuid: Uuid,
    pub to_anchor_pad_uuid: Uuid,
    pub selection_rule: String,
    pub candidate_copper_layers: Vec<StackupLayer>,
    pub summary: RoutePathCandidateAuthoredCopperGraphPolicyExplainSummary,
    pub selected_path: Option<RoutePathCandidateAuthoredCopperGraphPolicyExplainSelectedPath>,
}

impl Board {
    pub fn route_path_candidate_authored_copper_graph_explain_by_policy(
        &self,
        net_uuid: Uuid,
        from_anchor_pad_uuid: Uuid,
        to_anchor_pad_uuid: Uuid,
        policy: RoutePathCandidateAuthoredCopperGraphPolicy,
    ) -> Result<RoutePathCandidateAuthoredCopperGraphPolicyExplainReport, RoutePathCandidateError>
    {
        match policy {
            RoutePathCandidateAuthoredCopperGraphPolicy::Plain => self
                .route_path_candidate_authored_copper_graph_explain(
                    net_uuid,
                    from_anchor_pad_uuid,
                    to_anchor_pad_uuid,
                )
                .map(|report| map_plain_explain_report(report, policy)),
            RoutePathCandidateAuthoredCopperGraphPolicy::ZoneAware => self
                .route_path_candidate_authored_copper_graph_zone_aware_explain(
                    net_uuid,
                    from_anchor_pad_uuid,
                    to_anchor_pad_uuid,
                )
                .map(|report| map_zone_aware_explain_report(report, policy)),
            RoutePathCandidateAuthoredCopperGraphPolicy::ObstacleAware => self
                .route_path_candidate_authored_copper_graph_obstacle_aware_explain(
                    net_uuid,
                    from_anchor_pad_uuid,
                    to_anchor_pad_uuid,
                )
                .map(|report| map_obstacle_aware_explain_report(report, policy)),
            RoutePathCandidateAuthoredCopperGraphPolicy::ZoneObstacleAware => self
                .route_path_candidate_authored_copper_graph_zone_obstacle_aware_explain(
                    net_uuid,
                    from_anchor_pad_uuid,
                    to_anchor_pad_uuid,
                )
                .map(|report| map_zone_obstacle_aware_explain_report(report, policy)),
            RoutePathCandidateAuthoredCopperGraphPolicy::ZoneObstacleTopologyAware => self
                .route_path_candidate_authored_copper_graph_zone_obstacle_aware_topology_aware_explain(
                    net_uuid,
                    from_anchor_pad_uuid,
                    to_anchor_pad_uuid,
                )
                .map(|report| map_topology_aware_explain_report(report, policy)),
            RoutePathCandidateAuthoredCopperGraphPolicy::ZoneObstacleTopologyLayerBalanceAware => self
                .route_path_candidate_authored_copper_graph_zone_obstacle_aware_topology_aware_layer_balance_aware_explain(
                    net_uuid,
                    from_anchor_pad_uuid,
                    to_anchor_pad_uuid,
                )
                .map(|report| map_layer_balance_aware_explain_report(report, policy)),
        }
    }
}

fn explanation_kind(
    status: &RoutePathCandidateStatus,
) -> RoutePathCandidateAuthoredCopperGraphPolicyExplainKind {
    match status {
        RoutePathCandidateStatus::DeterministicPathFound => {
            RoutePathCandidateAuthoredCopperGraphPolicyExplainKind::DeterministicPathFound
        }
        RoutePathCandidateStatus::NoPathUnderCurrentAuthoredConstraints => {
            RoutePathCandidateAuthoredCopperGraphPolicyExplainKind::NoExistingAuthoredCopperPath
        }
    }
}

fn base_report(
    policy: RoutePathCandidateAuthoredCopperGraphPolicy,
    status: RoutePathCandidateStatus,
    net_uuid: Uuid,
    net_name: String,
    from_anchor_pad_uuid: Uuid,
    to_anchor_pad_uuid: Uuid,
    selection_rule: String,
    candidate_copper_layers: Vec<StackupLayer>,
    summary: RoutePathCandidateAuthoredCopperGraphPolicyExplainSummary,
    selected_path: Option<RoutePathCandidateAuthoredCopperGraphPolicyExplainSelectedPath>,
) -> RoutePathCandidateAuthoredCopperGraphPolicyExplainReport {
    RoutePathCandidateAuthoredCopperGraphPolicyExplainReport {
        contract: "m5_route_path_candidate_authored_copper_graph_policy_explain_v1".to_string(),
        persisted_native_board_state_only: true,
        policy,
        status: status.clone(),
        explanation_kind: explanation_kind(&status),
        net_uuid,
        net_name,
        from_anchor_pad_uuid,
        to_anchor_pad_uuid,
        selection_rule,
        candidate_copper_layers,
        summary,
        selected_path,
    }
}

fn map_plain_explain_report(
    report: RoutePathCandidateAuthoredCopperGraphExplainReport,
    policy: RoutePathCandidateAuthoredCopperGraphPolicy,
) -> RoutePathCandidateAuthoredCopperGraphPolicyExplainReport {
    base_report(
        policy,
        report.status,
        report.net_uuid,
        report.net_name,
        report.from_anchor_pad_uuid,
        report.to_anchor_pad_uuid,
        report.selection_rule,
        report.candidate_copper_layers,
        RoutePathCandidateAuthoredCopperGraphPolicyExplainSummary {
            candidate_copper_layer_count: report.summary.candidate_copper_layer_count,
            candidate_track_count: report.summary.candidate_track_count,
            candidate_via_count: report.summary.candidate_via_count,
            candidate_zone_count: 0,
            blocked_track_count: 0,
            blocked_via_count: 0,
            blocked_zone_connection_count: 0,
            path_step_count: report.summary.path_step_count,
            topology_transition_count: 0,
            layer_balance_score: 0,
            path_via_step_count: report
                .selected_path
                .as_ref()
                .map(|path| {
                    path.steps
                        .iter()
                        .filter(|step| matches!(step.kind, super::route_path_candidate_authored_copper_graph::RoutePathCandidateAuthoredCopperGraphStepKindView::Via))
                        .count()
                })
                .unwrap_or(0),
            path_zone_step_count: 0,
        },
        report.selected_path.map(|path| RoutePathCandidateAuthoredCopperGraphPolicyExplainSelectedPath {
            steps: path.steps.into_iter().map(|step| RoutePathCandidateAuthoredCopperGraphPolicyStep {
                kind: match step.kind {
                    super::route_path_candidate_authored_copper_graph::RoutePathCandidateAuthoredCopperGraphStepKindView::Track => RoutePathCandidateAuthoredCopperGraphPolicyStepKindView::Track,
                    super::route_path_candidate_authored_copper_graph::RoutePathCandidateAuthoredCopperGraphStepKindView::Via => RoutePathCandidateAuthoredCopperGraphPolicyStepKindView::Via,
                },
                object_uuid: step.object_uuid,
                layer: step.layer,
                from: step.from,
                to: step.to,
                from_layer: step.from_layer,
                to_layer: step.to_layer,
            }).collect(),
            selection_reason: path.selection_reason,
        }),
    )
}

fn map_zone_aware_explain_report(
    report: RoutePathCandidateAuthoredCopperGraphZoneAwareExplainReport,
    policy: RoutePathCandidateAuthoredCopperGraphPolicy,
) -> RoutePathCandidateAuthoredCopperGraphPolicyExplainReport {
    map_zone_like_explain_report(
        policy,
        report.status,
        report.net_uuid,
        report.net_name,
        report.from_anchor_pad_uuid,
        report.to_anchor_pad_uuid,
        report.selection_rule,
        report.candidate_copper_layers,
        report.summary.candidate_copper_layer_count,
        report.summary.candidate_track_count,
        report.summary.candidate_via_count,
        report.summary.candidate_zone_count,
        0,
        0,
        0,
        report.summary.path_step_count,
        0,
        0,
        report.selected_path.map(|path| {
            let via_count = path
                .steps
                .iter()
                .filter(|step| matches!(step.kind, super::route_path_candidate_authored_copper_graph_zone_aware::RoutePathCandidateAuthoredCopperGraphZoneAwareStepKindView::Via))
                .count();
            let zone_count = path
                .steps
                .iter()
                .filter(|step| matches!(step.kind, super::route_path_candidate_authored_copper_graph_zone_aware::RoutePathCandidateAuthoredCopperGraphZoneAwareStepKindView::Zone))
                .count();
            (
                RoutePathCandidateAuthoredCopperGraphPolicyExplainSelectedPath {
                    steps: path.steps.into_iter().map(|step| RoutePathCandidateAuthoredCopperGraphPolicyStep {
                        kind: match step.kind {
                            super::route_path_candidate_authored_copper_graph_zone_aware::RoutePathCandidateAuthoredCopperGraphZoneAwareStepKindView::Track => RoutePathCandidateAuthoredCopperGraphPolicyStepKindView::Track,
                            super::route_path_candidate_authored_copper_graph_zone_aware::RoutePathCandidateAuthoredCopperGraphZoneAwareStepKindView::Via => RoutePathCandidateAuthoredCopperGraphPolicyStepKindView::Via,
                            super::route_path_candidate_authored_copper_graph_zone_aware::RoutePathCandidateAuthoredCopperGraphZoneAwareStepKindView::Zone => RoutePathCandidateAuthoredCopperGraphPolicyStepKindView::Zone,
                        },
                        object_uuid: step.object_uuid,
                        layer: step.layer,
                        from: step.from,
                        to: step.to,
                        from_layer: step.from_layer,
                        to_layer: step.to_layer,
                    }).collect(),
                    selection_reason: path.selection_reason,
                },
                via_count,
                zone_count,
            )
        }),
    )
}

fn map_obstacle_aware_explain_report(
    report: RoutePathCandidateAuthoredCopperGraphObstacleAwareExplainReport,
    policy: RoutePathCandidateAuthoredCopperGraphPolicy,
) -> RoutePathCandidateAuthoredCopperGraphPolicyExplainReport {
    base_report(
        policy,
        report.status,
        report.net_uuid,
        report.net_name,
        report.from_anchor_pad_uuid,
        report.to_anchor_pad_uuid,
        report.selection_rule,
        report.candidate_copper_layers,
        RoutePathCandidateAuthoredCopperGraphPolicyExplainSummary {
            candidate_copper_layer_count: report.summary.candidate_copper_layer_count,
            candidate_track_count: report.summary.candidate_track_count,
            candidate_via_count: report.summary.candidate_via_count,
            candidate_zone_count: 0,
            blocked_track_count: report.summary.blocked_track_count,
            blocked_via_count: report.summary.blocked_via_count,
            blocked_zone_connection_count: 0,
            path_step_count: report.summary.path_step_count,
            topology_transition_count: 0,
            layer_balance_score: 0,
            path_via_step_count: report
                .selected_path
                .as_ref()
                .map(|path| {
                    path.steps
                        .iter()
                        .filter(|step| matches!(step.kind, super::route_path_candidate_authored_copper_graph_obstacle_aware::RoutePathCandidateAuthoredCopperGraphObstacleAwareStepKindView::Via))
                        .count()
                })
                .unwrap_or(0),
            path_zone_step_count: 0,
        },
        report.selected_path.map(|path| RoutePathCandidateAuthoredCopperGraphPolicyExplainSelectedPath {
            steps: path.steps.into_iter().map(|step| RoutePathCandidateAuthoredCopperGraphPolicyStep {
                kind: match step.kind {
                    super::route_path_candidate_authored_copper_graph_obstacle_aware::RoutePathCandidateAuthoredCopperGraphObstacleAwareStepKindView::Track => RoutePathCandidateAuthoredCopperGraphPolicyStepKindView::Track,
                    super::route_path_candidate_authored_copper_graph_obstacle_aware::RoutePathCandidateAuthoredCopperGraphObstacleAwareStepKindView::Via => RoutePathCandidateAuthoredCopperGraphPolicyStepKindView::Via,
                },
                object_uuid: step.object_uuid,
                layer: step.layer,
                from: step.from,
                to: step.to,
                from_layer: step.from_layer,
                to_layer: step.to_layer,
            }).collect(),
            selection_reason: path.selection_reason,
        }),
    )
}

fn map_zone_obstacle_aware_explain_report(
    report: RoutePathCandidateAuthoredCopperGraphZoneObstacleAwareExplainReport,
    policy: RoutePathCandidateAuthoredCopperGraphPolicy,
) -> RoutePathCandidateAuthoredCopperGraphPolicyExplainReport {
    map_zone_like_explain_report(
        policy,
        report.status,
        report.net_uuid,
        report.net_name,
        report.from_anchor_pad_uuid,
        report.to_anchor_pad_uuid,
        report.selection_rule,
        report.candidate_copper_layers,
        report.summary.candidate_copper_layer_count,
        report.summary.candidate_track_count,
        report.summary.candidate_via_count,
        report.summary.candidate_zone_count,
        report.summary.blocked_track_count,
        report.summary.blocked_via_count,
        report.summary.blocked_zone_connection_count,
        report.summary.path_step_count,
        0,
        0,
        report.selected_path.map(|path| {
            let via_count = path.steps.iter().filter(|step| matches!(step.kind, super::route_path_candidate_authored_copper_graph_zone_obstacle_aware::RoutePathCandidateAuthoredCopperGraphZoneObstacleAwareStepKindView::Via)).count();
            let zone_count = path.steps.iter().filter(|step| matches!(step.kind, super::route_path_candidate_authored_copper_graph_zone_obstacle_aware::RoutePathCandidateAuthoredCopperGraphZoneObstacleAwareStepKindView::Zone)).count();
            (
                RoutePathCandidateAuthoredCopperGraphPolicyExplainSelectedPath {
                    steps: path.steps.into_iter().map(|step| RoutePathCandidateAuthoredCopperGraphPolicyStep {
                        kind: match step.kind {
                            super::route_path_candidate_authored_copper_graph_zone_obstacle_aware::RoutePathCandidateAuthoredCopperGraphZoneObstacleAwareStepKindView::Track => RoutePathCandidateAuthoredCopperGraphPolicyStepKindView::Track,
                            super::route_path_candidate_authored_copper_graph_zone_obstacle_aware::RoutePathCandidateAuthoredCopperGraphZoneObstacleAwareStepKindView::Via => RoutePathCandidateAuthoredCopperGraphPolicyStepKindView::Via,
                            super::route_path_candidate_authored_copper_graph_zone_obstacle_aware::RoutePathCandidateAuthoredCopperGraphZoneObstacleAwareStepKindView::Zone => RoutePathCandidateAuthoredCopperGraphPolicyStepKindView::Zone,
                        },
                        object_uuid: step.object_uuid,
                        layer: step.layer,
                        from: step.from,
                        to: step.to,
                        from_layer: step.from_layer,
                        to_layer: step.to_layer,
                    }).collect(),
                    selection_reason: path.selection_reason,
                },
                via_count,
                zone_count,
            )
        }),
    )
}

fn map_topology_aware_explain_report(
    report: RoutePathCandidateAuthoredCopperGraphZoneObstacleAwareTopologyAwareExplainReport,
    policy: RoutePathCandidateAuthoredCopperGraphPolicy,
) -> RoutePathCandidateAuthoredCopperGraphPolicyExplainReport {
    map_zone_like_explain_report(
        policy,
        report.status,
        report.net_uuid,
        report.net_name,
        report.from_anchor_pad_uuid,
        report.to_anchor_pad_uuid,
        report.selection_rule,
        report.candidate_copper_layers,
        report.summary.candidate_copper_layer_count,
        report.summary.candidate_track_count,
        report.summary.candidate_via_count,
        report.summary.candidate_zone_count,
        report.summary.blocked_track_count,
        report.summary.blocked_via_count,
        report.summary.blocked_zone_connection_count,
        report.summary.path_step_count,
        report.summary.topology_transition_count,
        0,
        report.selected_path.map(|path| {
            let via_count = path.steps.iter().filter(|step| matches!(step.kind, super::route_path_candidate_authored_copper_graph_zone_obstacle_aware_topology_aware::RoutePathCandidateAuthoredCopperGraphZoneObstacleAwareTopologyAwareStepKindView::Via)).count();
            let zone_count = path.steps.iter().filter(|step| matches!(step.kind, super::route_path_candidate_authored_copper_graph_zone_obstacle_aware_topology_aware::RoutePathCandidateAuthoredCopperGraphZoneObstacleAwareTopologyAwareStepKindView::Zone)).count();
            (
                RoutePathCandidateAuthoredCopperGraphPolicyExplainSelectedPath {
                    steps: path.steps.into_iter().map(|step| RoutePathCandidateAuthoredCopperGraphPolicyStep {
                        kind: match step.kind {
                            super::route_path_candidate_authored_copper_graph_zone_obstacle_aware_topology_aware::RoutePathCandidateAuthoredCopperGraphZoneObstacleAwareTopologyAwareStepKindView::Track => RoutePathCandidateAuthoredCopperGraphPolicyStepKindView::Track,
                            super::route_path_candidate_authored_copper_graph_zone_obstacle_aware_topology_aware::RoutePathCandidateAuthoredCopperGraphZoneObstacleAwareTopologyAwareStepKindView::Via => RoutePathCandidateAuthoredCopperGraphPolicyStepKindView::Via,
                            super::route_path_candidate_authored_copper_graph_zone_obstacle_aware_topology_aware::RoutePathCandidateAuthoredCopperGraphZoneObstacleAwareTopologyAwareStepKindView::Zone => RoutePathCandidateAuthoredCopperGraphPolicyStepKindView::Zone,
                        },
                        object_uuid: step.object_uuid,
                        layer: step.layer,
                        from: step.from,
                        to: step.to,
                        from_layer: step.from_layer,
                        to_layer: step.to_layer,
                    }).collect(),
                    selection_reason: path.selection_reason,
                },
                via_count,
                zone_count,
            )
        }),
    )
}

fn map_layer_balance_aware_explain_report(
    report: RoutePathCandidateAuthoredCopperGraphZoneObstacleAwareTopologyAwareLayerBalanceAwareExplainReport,
    policy: RoutePathCandidateAuthoredCopperGraphPolicy,
) -> RoutePathCandidateAuthoredCopperGraphPolicyExplainReport {
    map_zone_like_explain_report(
        policy,
        report.status,
        report.net_uuid,
        report.net_name,
        report.from_anchor_pad_uuid,
        report.to_anchor_pad_uuid,
        report.selection_rule,
        report.candidate_copper_layers,
        report.summary.candidate_copper_layer_count,
        report.summary.candidate_track_count,
        report.summary.candidate_via_count,
        report.summary.candidate_zone_count,
        report.summary.blocked_track_count,
        report.summary.blocked_via_count,
        report.summary.blocked_zone_connection_count,
        report.summary.path_step_count,
        report.summary.topology_transition_count,
        report.summary.layer_balance_score,
        report.selected_path.map(|path| {
            let via_count = path.steps.iter().filter(|step| matches!(step.kind, super::route_path_candidate_authored_copper_graph_zone_obstacle_aware_topology_aware_layer_balance_aware::RoutePathCandidateAuthoredCopperGraphZoneObstacleAwareTopologyAwareLayerBalanceAwareStepKindView::Via)).count();
            let zone_count = path.steps.iter().filter(|step| matches!(step.kind, super::route_path_candidate_authored_copper_graph_zone_obstacle_aware_topology_aware_layer_balance_aware::RoutePathCandidateAuthoredCopperGraphZoneObstacleAwareTopologyAwareLayerBalanceAwareStepKindView::Zone)).count();
            (
                RoutePathCandidateAuthoredCopperGraphPolicyExplainSelectedPath {
                    steps: path.steps.into_iter().map(|step| RoutePathCandidateAuthoredCopperGraphPolicyStep {
                        kind: match step.kind {
                            super::route_path_candidate_authored_copper_graph_zone_obstacle_aware_topology_aware_layer_balance_aware::RoutePathCandidateAuthoredCopperGraphZoneObstacleAwareTopologyAwareLayerBalanceAwareStepKindView::Track => RoutePathCandidateAuthoredCopperGraphPolicyStepKindView::Track,
                            super::route_path_candidate_authored_copper_graph_zone_obstacle_aware_topology_aware_layer_balance_aware::RoutePathCandidateAuthoredCopperGraphZoneObstacleAwareTopologyAwareLayerBalanceAwareStepKindView::Via => RoutePathCandidateAuthoredCopperGraphPolicyStepKindView::Via,
                            super::route_path_candidate_authored_copper_graph_zone_obstacle_aware_topology_aware_layer_balance_aware::RoutePathCandidateAuthoredCopperGraphZoneObstacleAwareTopologyAwareLayerBalanceAwareStepKindView::Zone => RoutePathCandidateAuthoredCopperGraphPolicyStepKindView::Zone,
                        },
                        object_uuid: step.object_uuid,
                        layer: step.layer,
                        from: step.from,
                        to: step.to,
                        from_layer: step.from_layer,
                        to_layer: step.to_layer,
                    }).collect(),
                    selection_reason: path.selection_reason,
                },
                via_count,
                zone_count,
            )
        }),
    )
}

fn map_zone_like_explain_report(
    policy: RoutePathCandidateAuthoredCopperGraphPolicy,
    status: RoutePathCandidateStatus,
    net_uuid: Uuid,
    net_name: String,
    from_anchor_pad_uuid: Uuid,
    to_anchor_pad_uuid: Uuid,
    selection_rule: String,
    candidate_copper_layers: Vec<StackupLayer>,
    candidate_copper_layer_count: usize,
    candidate_track_count: usize,
    candidate_via_count: usize,
    candidate_zone_count: usize,
    blocked_track_count: usize,
    blocked_via_count: usize,
    blocked_zone_connection_count: usize,
    path_step_count: usize,
    topology_transition_count: usize,
    layer_balance_score: usize,
    selected_path_and_counts: Option<(
        RoutePathCandidateAuthoredCopperGraphPolicyExplainSelectedPath,
        usize,
        usize,
    )>,
) -> RoutePathCandidateAuthoredCopperGraphPolicyExplainReport {
    let (selected_path, path_via_step_count, path_zone_step_count) =
        if let Some((selected_path, path_via_step_count, path_zone_step_count)) =
            selected_path_and_counts
        {
            (Some(selected_path), path_via_step_count, path_zone_step_count)
        } else {
            (None, 0, 0)
        };

    base_report(
        policy,
        status,
        net_uuid,
        net_name,
        from_anchor_pad_uuid,
        to_anchor_pad_uuid,
        selection_rule,
        candidate_copper_layers,
        RoutePathCandidateAuthoredCopperGraphPolicyExplainSummary {
            candidate_copper_layer_count,
            candidate_track_count,
            candidate_via_count,
            candidate_zone_count,
            blocked_track_count,
            blocked_via_count,
            blocked_zone_connection_count,
            path_step_count,
            topology_transition_count,
            layer_balance_score,
            path_via_step_count,
            path_zone_step_count,
        },
        selected_path,
    )
}
