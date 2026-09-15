use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::board::{Board, RoutePathCandidateError, RoutePathCandidateStatus, StackupLayer};
use crate::ir::geometry::{LayerId, Point};

use super::route_path_candidate_authored_copper_graph_obstacle_aware_selection::{
    AuthoredCopperGraphObstacleAwareStepKind,
    ROUTE_PATH_CANDIDATE_AUTHORED_COPPER_GRAPH_OBSTACLE_AWARE_SELECTION_RULE,
    selected_authored_copper_graph_obstacle_aware_path,
};

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum RoutePathCandidateAuthoredCopperGraphObstacleAwareStepKindView {
    Track,
    Via,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct RoutePathCandidateAuthoredCopperGraphObstacleAwareStep {
    pub kind: RoutePathCandidateAuthoredCopperGraphObstacleAwareStepKindView,
    pub object_uuid: Uuid,
    pub layer: LayerId,
    pub from: Point,
    pub to: Point,
    pub from_layer: Option<LayerId>,
    pub to_layer: Option<LayerId>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct RoutePathCandidateAuthoredCopperGraphObstacleAwarePath {
    pub steps: Vec<RoutePathCandidateAuthoredCopperGraphObstacleAwareStep>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct RoutePathCandidateAuthoredCopperGraphObstacleAwareSummary {
    pub candidate_copper_layer_count: usize,
    pub candidate_track_count: usize,
    pub candidate_via_count: usize,
    pub blocked_track_count: usize,
    pub blocked_via_count: usize,
    pub path_step_count: usize,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct RoutePathCandidateAuthoredCopperGraphObstacleAwareReport {
    pub contract: String,
    pub persisted_native_board_state_only: bool,
    pub selection_rule: String,
    pub status: RoutePathCandidateStatus,
    pub net_uuid: Uuid,
    pub net_name: String,
    pub from_anchor_pad_uuid: Uuid,
    pub to_anchor_pad_uuid: Uuid,
    pub candidate_copper_layers: Vec<StackupLayer>,
    pub summary: RoutePathCandidateAuthoredCopperGraphObstacleAwareSummary,
    pub path: Option<RoutePathCandidateAuthoredCopperGraphObstacleAwarePath>,
}

impl Board {
    pub fn route_path_candidate_authored_copper_graph_obstacle_aware(
        &self,
        net_uuid: Uuid,
        from_anchor_pad_uuid: Uuid,
        to_anchor_pad_uuid: Uuid,
    ) -> Result<RoutePathCandidateAuthoredCopperGraphObstacleAwareReport, RoutePathCandidateError>
    {
        if from_anchor_pad_uuid == to_anchor_pad_uuid {
            return Err(RoutePathCandidateError::DuplicateAnchorPair {
                pad_uuid: from_anchor_pad_uuid,
            });
        }

        let preflight = self
            .route_preflight(net_uuid)
            .ok_or(RoutePathCandidateError::NetNotFound { net_uuid })?;
        let from_anchor = preflight
            .anchors
            .iter()
            .find(|anchor| anchor.pad_uuid == from_anchor_pad_uuid)
            .ok_or(RoutePathCandidateError::AnchorNotOnNet {
                pad_uuid: from_anchor_pad_uuid,
                net_uuid,
            })?;
        let to_anchor = preflight
            .anchors
            .iter()
            .find(|anchor| anchor.pad_uuid == to_anchor_pad_uuid)
            .ok_or(RoutePathCandidateError::AnchorNotOnNet {
                pad_uuid: to_anchor_pad_uuid,
                net_uuid,
            })?;

        let (candidate_tracks, candidate_vias, blocked_track_count, blocked_via_count, path_match) =
            selected_authored_copper_graph_obstacle_aware_path(
                self,
                net_uuid,
                from_anchor,
                to_anchor,
            );

        let path = path_match.map(
            |entry| RoutePathCandidateAuthoredCopperGraphObstacleAwarePath {
                steps: entry
                    .steps
                    .iter()
                    .map(|step| {
                        RoutePathCandidateAuthoredCopperGraphObstacleAwareStep {
                    kind: match step.kind {
                        AuthoredCopperGraphObstacleAwareStepKind::Track => {
                            RoutePathCandidateAuthoredCopperGraphObstacleAwareStepKindView::Track
                        }
                        AuthoredCopperGraphObstacleAwareStepKind::Via => {
                            RoutePathCandidateAuthoredCopperGraphObstacleAwareStepKindView::Via
                        }
                    },
                    object_uuid: step.object_uuid,
                    layer: step.layer,
                    from: step.from,
                    to: step.to,
                    from_layer: step.from_layer,
                    to_layer: step.to_layer,
                }
                    })
                    .collect(),
            },
        );
        let status = if path.is_some() {
            RoutePathCandidateStatus::DeterministicPathFound
        } else {
            RoutePathCandidateStatus::NoPathUnderCurrentAuthoredConstraints
        };

        Ok(RoutePathCandidateAuthoredCopperGraphObstacleAwareReport {
            contract: "m5_route_path_candidate_authored_copper_graph_obstacle_aware_v1".to_string(),
            persisted_native_board_state_only: true,
            selection_rule:
                ROUTE_PATH_CANDIDATE_AUTHORED_COPPER_GRAPH_OBSTACLE_AWARE_SELECTION_RULE.to_string(),
            status,
            net_uuid: preflight.net_uuid,
            net_name: preflight.net_name,
            from_anchor_pad_uuid,
            to_anchor_pad_uuid,
            candidate_copper_layers: preflight.candidate_copper_layers.clone(),
            summary: RoutePathCandidateAuthoredCopperGraphObstacleAwareSummary {
                candidate_copper_layer_count: preflight.candidate_copper_layers.len(),
                candidate_track_count: candidate_tracks.len(),
                candidate_via_count: candidate_vias.len(),
                blocked_track_count,
                blocked_via_count,
                path_step_count: path.as_ref().map(|entry| entry.steps.len()).unwrap_or(0),
            },
            path,
        })
    }
}
