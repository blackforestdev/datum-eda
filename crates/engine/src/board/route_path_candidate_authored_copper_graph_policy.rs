use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::board::{
    Board, RoutePathCandidateAuthoredCopperGraphObstacleAwareReport,
    RoutePathCandidateAuthoredCopperGraphObstacleAwareStepKindView,
    RoutePathCandidateAuthoredCopperGraphReport,
    RoutePathCandidateAuthoredCopperGraphStepKindView,
    RoutePathCandidateAuthoredCopperGraphZoneAwarePath,
    RoutePathCandidateAuthoredCopperGraphZoneAwareReport,
    RoutePathCandidateAuthoredCopperGraphZoneAwareStepKindView,
    RoutePathCandidateAuthoredCopperGraphZoneObstacleAwarePath,
    RoutePathCandidateAuthoredCopperGraphZoneObstacleAwareReport,
    RoutePathCandidateAuthoredCopperGraphZoneObstacleAwareStepKindView,
    RoutePathCandidateAuthoredCopperGraphZoneObstacleAwareTopologyAwareLayerBalanceAwareReport,
    RoutePathCandidateAuthoredCopperGraphZoneObstacleAwareTopologyAwareLayerBalanceAwareStepKindView,
    RoutePathCandidateAuthoredCopperGraphZoneObstacleAwareTopologyAwarePath,
    RoutePathCandidateAuthoredCopperGraphZoneObstacleAwareTopologyAwareReport,
    RoutePathCandidateAuthoredCopperGraphZoneObstacleAwareTopologyAwareStepKindView,
    RoutePathCandidateError, RoutePathCandidateStatus, StackupLayer,
};
use crate::ir::geometry::{LayerId, Point};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum RoutePathCandidateAuthoredCopperGraphPolicy {
    Plain,
    ZoneAware,
    ObstacleAware,
    ZoneObstacleAware,
    ZoneObstacleTopologyAware,
    ZoneObstacleTopologyLayerBalanceAware,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum RoutePathCandidateAuthoredCopperGraphPolicyStepKindView {
    Track,
    Via,
    Zone,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct RoutePathCandidateAuthoredCopperGraphPolicyStep {
    pub kind: RoutePathCandidateAuthoredCopperGraphPolicyStepKindView,
    pub object_uuid: Uuid,
    pub layer: LayerId,
    pub from: Point,
    pub to: Point,
    pub from_layer: Option<LayerId>,
    pub to_layer: Option<LayerId>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct RoutePathCandidateAuthoredCopperGraphPolicyPath {
    pub steps: Vec<RoutePathCandidateAuthoredCopperGraphPolicyStep>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct RoutePathCandidateAuthoredCopperGraphPolicySummary {
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
pub struct RoutePathCandidateAuthoredCopperGraphPolicyReport {
    pub contract: String,
    pub persisted_native_board_state_only: bool,
    pub policy: RoutePathCandidateAuthoredCopperGraphPolicy,
    pub selection_rule: String,
    pub status: RoutePathCandidateStatus,
    pub net_uuid: Uuid,
    pub net_name: String,
    pub from_anchor_pad_uuid: Uuid,
    pub to_anchor_pad_uuid: Uuid,
    pub candidate_copper_layers: Vec<StackupLayer>,
    pub summary: RoutePathCandidateAuthoredCopperGraphPolicySummary,
    pub path: Option<RoutePathCandidateAuthoredCopperGraphPolicyPath>,
}

impl Board {
    pub fn route_path_candidate_authored_copper_graph_by_policy(
        &self,
        net_uuid: Uuid,
        from_anchor_pad_uuid: Uuid,
        to_anchor_pad_uuid: Uuid,
        policy: RoutePathCandidateAuthoredCopperGraphPolicy,
    ) -> Result<RoutePathCandidateAuthoredCopperGraphPolicyReport, RoutePathCandidateError> {
        match policy {
            RoutePathCandidateAuthoredCopperGraphPolicy::Plain => self
                .route_path_candidate_authored_copper_graph(
                    net_uuid,
                    from_anchor_pad_uuid,
                    to_anchor_pad_uuid,
                )
                .map(|report| map_plain_report(report, policy)),
            RoutePathCandidateAuthoredCopperGraphPolicy::ZoneAware => self
                .route_path_candidate_authored_copper_graph_zone_aware(
                    net_uuid,
                    from_anchor_pad_uuid,
                    to_anchor_pad_uuid,
                )
                .map(|report| map_zone_aware_report(report, policy)),
            RoutePathCandidateAuthoredCopperGraphPolicy::ObstacleAware => self
                .route_path_candidate_authored_copper_graph_obstacle_aware(
                    net_uuid,
                    from_anchor_pad_uuid,
                    to_anchor_pad_uuid,
                )
                .map(|report| map_obstacle_aware_report(report, policy)),
            RoutePathCandidateAuthoredCopperGraphPolicy::ZoneObstacleAware => self
                .route_path_candidate_authored_copper_graph_zone_obstacle_aware(
                    net_uuid,
                    from_anchor_pad_uuid,
                    to_anchor_pad_uuid,
                )
                .map(|report| map_zone_obstacle_aware_report(report, policy)),
            RoutePathCandidateAuthoredCopperGraphPolicy::ZoneObstacleTopologyAware => self
                .route_path_candidate_authored_copper_graph_zone_obstacle_aware_topology_aware(
                    net_uuid,
                    from_anchor_pad_uuid,
                    to_anchor_pad_uuid,
                )
                .map(|report| map_topology_aware_report(report, policy)),
            RoutePathCandidateAuthoredCopperGraphPolicy::ZoneObstacleTopologyLayerBalanceAware => self
                .route_path_candidate_authored_copper_graph_zone_obstacle_aware_topology_aware_layer_balance_aware(
                    net_uuid,
                    from_anchor_pad_uuid,
                    to_anchor_pad_uuid,
                )
                .map(|report| map_layer_balance_aware_report(report, policy)),
        }
    }
}

fn base_report(
    policy: RoutePathCandidateAuthoredCopperGraphPolicy,
    selection_rule: String,
    status: RoutePathCandidateStatus,
    net_uuid: Uuid,
    net_name: String,
    from_anchor_pad_uuid: Uuid,
    to_anchor_pad_uuid: Uuid,
    candidate_copper_layers: Vec<StackupLayer>,
    summary: RoutePathCandidateAuthoredCopperGraphPolicySummary,
    path: Option<RoutePathCandidateAuthoredCopperGraphPolicyPath>,
) -> RoutePathCandidateAuthoredCopperGraphPolicyReport {
    RoutePathCandidateAuthoredCopperGraphPolicyReport {
        contract: "m5_route_path_candidate_authored_copper_graph_policy_v1".to_string(),
        persisted_native_board_state_only: true,
        policy,
        selection_rule,
        status,
        net_uuid,
        net_name,
        from_anchor_pad_uuid,
        to_anchor_pad_uuid,
        candidate_copper_layers,
        summary,
        path,
    }
}

fn map_plain_report(
    report: RoutePathCandidateAuthoredCopperGraphReport,
    policy: RoutePathCandidateAuthoredCopperGraphPolicy,
) -> RoutePathCandidateAuthoredCopperGraphPolicyReport {
    base_report(
        policy,
        report.selection_rule,
        report.status,
        report.net_uuid,
        report.net_name,
        report.from_anchor_pad_uuid,
        report.to_anchor_pad_uuid,
        report.candidate_copper_layers,
        RoutePathCandidateAuthoredCopperGraphPolicySummary {
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
                .path
                .as_ref()
                .map(|path| {
                    path.steps
                        .iter()
                        .filter(|step| {
                            matches!(step.kind, RoutePathCandidateAuthoredCopperGraphStepKindView::Via)
                        })
                        .count()
                })
                .unwrap_or(0),
            path_zone_step_count: 0,
        },
        report.path.map(|path| RoutePathCandidateAuthoredCopperGraphPolicyPath {
            steps: path
                .steps
                .into_iter()
                .map(|step| RoutePathCandidateAuthoredCopperGraphPolicyStep {
                    kind: match step.kind {
                        RoutePathCandidateAuthoredCopperGraphStepKindView::Track => {
                            RoutePathCandidateAuthoredCopperGraphPolicyStepKindView::Track
                        }
                        RoutePathCandidateAuthoredCopperGraphStepKindView::Via => {
                            RoutePathCandidateAuthoredCopperGraphPolicyStepKindView::Via
                        }
                    },
                    object_uuid: step.object_uuid,
                    layer: step.layer,
                    from: step.from,
                    to: step.to,
                    from_layer: step.from_layer,
                    to_layer: step.to_layer,
                })
                .collect(),
        }),
    )
}

fn map_zone_aware_report(
    report: RoutePathCandidateAuthoredCopperGraphZoneAwareReport,
    policy: RoutePathCandidateAuthoredCopperGraphPolicy,
) -> RoutePathCandidateAuthoredCopperGraphPolicyReport {
    base_report(
        policy,
        report.selection_rule,
        report.status,
        report.net_uuid,
        report.net_name,
        report.from_anchor_pad_uuid,
        report.to_anchor_pad_uuid,
        report.candidate_copper_layers,
        RoutePathCandidateAuthoredCopperGraphPolicySummary {
            candidate_copper_layer_count: report.summary.candidate_copper_layer_count,
            candidate_track_count: report.summary.candidate_track_count,
            candidate_via_count: report.summary.candidate_via_count,
            candidate_zone_count: report.summary.candidate_zone_count,
            blocked_track_count: 0,
            blocked_via_count: 0,
            blocked_zone_connection_count: 0,
            path_step_count: report.summary.path_step_count,
            topology_transition_count: 0,
            layer_balance_score: 0,
            path_via_step_count: report
                .path
                .as_ref()
                .map(|path| {
                    path.steps
                        .iter()
                        .filter(|step| {
                            matches!(step.kind, RoutePathCandidateAuthoredCopperGraphZoneAwareStepKindView::Via)
                        })
                        .count()
                })
                .unwrap_or(0),
            path_zone_step_count: report
                .path
                .as_ref()
                .map(|path| {
                    path.steps
                        .iter()
                        .filter(|step| {
                            matches!(step.kind, RoutePathCandidateAuthoredCopperGraphZoneAwareStepKindView::Zone)
                        })
                        .count()
                })
                .unwrap_or(0),
        },
        report.path.map(|path| map_zone_path(path)),
    )
}

fn map_obstacle_aware_report(
    report: RoutePathCandidateAuthoredCopperGraphObstacleAwareReport,
    policy: RoutePathCandidateAuthoredCopperGraphPolicy,
) -> RoutePathCandidateAuthoredCopperGraphPolicyReport {
    base_report(
        policy,
        report.selection_rule,
        report.status,
        report.net_uuid,
        report.net_name,
        report.from_anchor_pad_uuid,
        report.to_anchor_pad_uuid,
        report.candidate_copper_layers,
        RoutePathCandidateAuthoredCopperGraphPolicySummary {
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
            path_via_step_count: report.summary.path_step_count.saturating_sub(
                report
                    .path
                    .as_ref()
                    .map(|path| {
                        path.steps
                            .iter()
                            .filter(|step| {
                                matches!(step.kind, RoutePathCandidateAuthoredCopperGraphObstacleAwareStepKindView::Track)
                            })
                            .count()
                    })
                    .unwrap_or(0),
            ),
            path_zone_step_count: 0,
        },
        report.path.map(|path| RoutePathCandidateAuthoredCopperGraphPolicyPath {
            steps: path
                .steps
                .into_iter()
                .map(|step| RoutePathCandidateAuthoredCopperGraphPolicyStep {
                    kind: match step.kind {
                        RoutePathCandidateAuthoredCopperGraphObstacleAwareStepKindView::Track => {
                            RoutePathCandidateAuthoredCopperGraphPolicyStepKindView::Track
                        }
                        RoutePathCandidateAuthoredCopperGraphObstacleAwareStepKindView::Via => {
                            RoutePathCandidateAuthoredCopperGraphPolicyStepKindView::Via
                        }
                    },
                    object_uuid: step.object_uuid,
                    layer: step.layer,
                    from: step.from,
                    to: step.to,
                    from_layer: step.from_layer,
                    to_layer: step.to_layer,
                })
                .collect(),
        }),
    )
}

fn map_zone_obstacle_aware_report(
    report: RoutePathCandidateAuthoredCopperGraphZoneObstacleAwareReport,
    policy: RoutePathCandidateAuthoredCopperGraphPolicy,
) -> RoutePathCandidateAuthoredCopperGraphPolicyReport {
    base_report(
        policy,
        report.selection_rule,
        report.status,
        report.net_uuid,
        report.net_name,
        report.from_anchor_pad_uuid,
        report.to_anchor_pad_uuid,
        report.candidate_copper_layers,
        RoutePathCandidateAuthoredCopperGraphPolicySummary {
            candidate_copper_layer_count: report.summary.candidate_copper_layer_count,
            candidate_track_count: report.summary.candidate_track_count,
            candidate_via_count: report.summary.candidate_via_count,
            candidate_zone_count: report.summary.candidate_zone_count,
            blocked_track_count: report.summary.blocked_track_count,
            blocked_via_count: report.summary.blocked_via_count,
            blocked_zone_connection_count: report.summary.blocked_zone_connection_count,
            path_step_count: report.summary.path_step_count,
            topology_transition_count: 0,
            layer_balance_score: 0,
            path_via_step_count: report
                .path
                .as_ref()
                .map(|path| {
                    path.steps
                        .iter()
                        .filter(|step| {
                            matches!(step.kind, RoutePathCandidateAuthoredCopperGraphZoneObstacleAwareStepKindView::Via)
                        })
                        .count()
                })
                .unwrap_or(0),
            path_zone_step_count: report
                .path
                .as_ref()
                .map(|path| {
                    path.steps
                        .iter()
                        .filter(|step| {
                            matches!(step.kind, RoutePathCandidateAuthoredCopperGraphZoneObstacleAwareStepKindView::Zone)
                        })
                        .count()
                })
                .unwrap_or(0),
        },
        report.path.map(|path| map_zone_obstacle_path(path)),
    )
}

fn map_topology_aware_report(
    report: RoutePathCandidateAuthoredCopperGraphZoneObstacleAwareTopologyAwareReport,
    policy: RoutePathCandidateAuthoredCopperGraphPolicy,
) -> RoutePathCandidateAuthoredCopperGraphPolicyReport {
    base_report(
        policy,
        report.selection_rule,
        report.status,
        report.net_uuid,
        report.net_name,
        report.from_anchor_pad_uuid,
        report.to_anchor_pad_uuid,
        report.candidate_copper_layers,
        RoutePathCandidateAuthoredCopperGraphPolicySummary {
            candidate_copper_layer_count: report.summary.candidate_copper_layer_count,
            candidate_track_count: report.summary.candidate_track_count,
            candidate_via_count: report.summary.candidate_via_count,
            candidate_zone_count: report.summary.candidate_zone_count,
            blocked_track_count: report.summary.blocked_track_count,
            blocked_via_count: report.summary.blocked_via_count,
            blocked_zone_connection_count: report.summary.blocked_zone_connection_count,
            path_step_count: report.summary.path_step_count,
            topology_transition_count: report.summary.topology_transition_count,
            layer_balance_score: 0,
            path_via_step_count: report.summary.path_via_step_count,
            path_zone_step_count: report.summary.path_zone_step_count,
        },
        report.path.map(|path| map_topology_path(path)),
    )
}

fn map_layer_balance_aware_report(
    report: RoutePathCandidateAuthoredCopperGraphZoneObstacleAwareTopologyAwareLayerBalanceAwareReport,
    policy: RoutePathCandidateAuthoredCopperGraphPolicy,
) -> RoutePathCandidateAuthoredCopperGraphPolicyReport {
    base_report(
        policy,
        report.selection_rule,
        report.status,
        report.net_uuid,
        report.net_name,
        report.from_anchor_pad_uuid,
        report.to_anchor_pad_uuid,
        report.candidate_copper_layers,
        RoutePathCandidateAuthoredCopperGraphPolicySummary {
            candidate_copper_layer_count: report.summary.candidate_copper_layer_count,
            candidate_track_count: report.summary.candidate_track_count,
            candidate_via_count: report.summary.candidate_via_count,
            candidate_zone_count: report.summary.candidate_zone_count,
            blocked_track_count: report.summary.blocked_track_count,
            blocked_via_count: report.summary.blocked_via_count,
            blocked_zone_connection_count: report.summary.blocked_zone_connection_count,
            path_step_count: report.summary.path_step_count,
            topology_transition_count: report.summary.topology_transition_count,
            layer_balance_score: report.summary.layer_balance_score,
            path_via_step_count: report.summary.path_via_step_count,
            path_zone_step_count: report.summary.path_zone_step_count,
        },
        report.path.map(|path| RoutePathCandidateAuthoredCopperGraphPolicyPath {
            steps: path
                .steps
                .into_iter()
                .map(|step| RoutePathCandidateAuthoredCopperGraphPolicyStep {
                    kind: match step.kind {
                        RoutePathCandidateAuthoredCopperGraphZoneObstacleAwareTopologyAwareLayerBalanceAwareStepKindView::Track => {
                            RoutePathCandidateAuthoredCopperGraphPolicyStepKindView::Track
                        }
                        RoutePathCandidateAuthoredCopperGraphZoneObstacleAwareTopologyAwareLayerBalanceAwareStepKindView::Via => {
                            RoutePathCandidateAuthoredCopperGraphPolicyStepKindView::Via
                        }
                        RoutePathCandidateAuthoredCopperGraphZoneObstacleAwareTopologyAwareLayerBalanceAwareStepKindView::Zone => {
                            RoutePathCandidateAuthoredCopperGraphPolicyStepKindView::Zone
                        }
                    },
                    object_uuid: step.object_uuid,
                    layer: step.layer,
                    from: step.from,
                    to: step.to,
                    from_layer: step.from_layer,
                    to_layer: step.to_layer,
                })
                .collect(),
        }),
    )
}

fn map_zone_path(
    path: RoutePathCandidateAuthoredCopperGraphZoneAwarePath,
) -> RoutePathCandidateAuthoredCopperGraphPolicyPath {
    RoutePathCandidateAuthoredCopperGraphPolicyPath {
        steps: path
            .steps
            .into_iter()
            .map(|step| RoutePathCandidateAuthoredCopperGraphPolicyStep {
                kind: match step.kind {
                    RoutePathCandidateAuthoredCopperGraphZoneAwareStepKindView::Track => {
                        RoutePathCandidateAuthoredCopperGraphPolicyStepKindView::Track
                    }
                    RoutePathCandidateAuthoredCopperGraphZoneAwareStepKindView::Via => {
                        RoutePathCandidateAuthoredCopperGraphPolicyStepKindView::Via
                    }
                    RoutePathCandidateAuthoredCopperGraphZoneAwareStepKindView::Zone => {
                        RoutePathCandidateAuthoredCopperGraphPolicyStepKindView::Zone
                    }
                },
                object_uuid: step.object_uuid,
                layer: step.layer,
                from: step.from,
                to: step.to,
                from_layer: step.from_layer,
                to_layer: step.to_layer,
            })
            .collect(),
    }
}

fn map_zone_obstacle_path(
    path: RoutePathCandidateAuthoredCopperGraphZoneObstacleAwarePath,
) -> RoutePathCandidateAuthoredCopperGraphPolicyPath {
    RoutePathCandidateAuthoredCopperGraphPolicyPath {
        steps: path
            .steps
            .into_iter()
            .map(|step| RoutePathCandidateAuthoredCopperGraphPolicyStep {
                kind: match step.kind {
                    RoutePathCandidateAuthoredCopperGraphZoneObstacleAwareStepKindView::Track => {
                        RoutePathCandidateAuthoredCopperGraphPolicyStepKindView::Track
                    }
                    RoutePathCandidateAuthoredCopperGraphZoneObstacleAwareStepKindView::Via => {
                        RoutePathCandidateAuthoredCopperGraphPolicyStepKindView::Via
                    }
                    RoutePathCandidateAuthoredCopperGraphZoneObstacleAwareStepKindView::Zone => {
                        RoutePathCandidateAuthoredCopperGraphPolicyStepKindView::Zone
                    }
                },
                object_uuid: step.object_uuid,
                layer: step.layer,
                from: step.from,
                to: step.to,
                from_layer: step.from_layer,
                to_layer: step.to_layer,
            })
            .collect(),
    }
}

fn map_topology_path(
    path: RoutePathCandidateAuthoredCopperGraphZoneObstacleAwareTopologyAwarePath,
) -> RoutePathCandidateAuthoredCopperGraphPolicyPath {
    RoutePathCandidateAuthoredCopperGraphPolicyPath {
        steps: path
            .steps
            .into_iter()
            .map(|step| RoutePathCandidateAuthoredCopperGraphPolicyStep {
                kind: match step.kind {
                    RoutePathCandidateAuthoredCopperGraphZoneObstacleAwareTopologyAwareStepKindView::Track => {
                        RoutePathCandidateAuthoredCopperGraphPolicyStepKindView::Track
                    }
                    RoutePathCandidateAuthoredCopperGraphZoneObstacleAwareTopologyAwareStepKindView::Via => {
                        RoutePathCandidateAuthoredCopperGraphPolicyStepKindView::Via
                    }
                    RoutePathCandidateAuthoredCopperGraphZoneObstacleAwareTopologyAwareStepKindView::Zone => {
                        RoutePathCandidateAuthoredCopperGraphPolicyStepKindView::Zone
                    }
                },
                object_uuid: step.object_uuid,
                layer: step.layer,
                from: step.from,
                to: step.to,
                from_layer: step.from_layer,
                to_layer: step.to_layer,
            })
            .collect(),
    }
}
