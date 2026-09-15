use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::board::{Board, RoutePathCandidateError, RoutePathCandidateStatus, StackupLayer};
use crate::ir::geometry::{LayerId, Point};

use super::route_path_candidate_authored_copper_graph_zone_aware_selection::{
    AuthoredCopperGraphZoneAwareStepKind,
    ROUTE_PATH_CANDIDATE_AUTHORED_COPPER_GRAPH_ZONE_AWARE_SELECTION_RULE,
    selected_authored_copper_graph_zone_aware_path,
};

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum RoutePathCandidateAuthoredCopperGraphZoneAwareStepKindView {
    Track,
    Via,
    Zone,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct RoutePathCandidateAuthoredCopperGraphZoneAwareStep {
    pub kind: RoutePathCandidateAuthoredCopperGraphZoneAwareStepKindView,
    pub object_uuid: Uuid,
    pub layer: LayerId,
    pub from: Point,
    pub to: Point,
    pub from_layer: Option<LayerId>,
    pub to_layer: Option<LayerId>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct RoutePathCandidateAuthoredCopperGraphZoneAwarePath {
    pub steps: Vec<RoutePathCandidateAuthoredCopperGraphZoneAwareStep>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct RoutePathCandidateAuthoredCopperGraphZoneAwareSummary {
    pub candidate_copper_layer_count: usize,
    pub candidate_track_count: usize,
    pub candidate_via_count: usize,
    pub candidate_zone_count: usize,
    pub path_step_count: usize,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct RoutePathCandidateAuthoredCopperGraphZoneAwareReport {
    pub contract: String,
    pub persisted_native_board_state_only: bool,
    pub selection_rule: String,
    pub status: RoutePathCandidateStatus,
    pub net_uuid: Uuid,
    pub net_name: String,
    pub from_anchor_pad_uuid: Uuid,
    pub to_anchor_pad_uuid: Uuid,
    pub candidate_copper_layers: Vec<StackupLayer>,
    pub summary: RoutePathCandidateAuthoredCopperGraphZoneAwareSummary,
    pub path: Option<RoutePathCandidateAuthoredCopperGraphZoneAwarePath>,
}

impl Board {
    pub fn route_path_candidate_authored_copper_graph_zone_aware(
        &self,
        net_uuid: Uuid,
        from_anchor_pad_uuid: Uuid,
        to_anchor_pad_uuid: Uuid,
    ) -> Result<RoutePathCandidateAuthoredCopperGraphZoneAwareReport, RoutePathCandidateError> {
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

        let (candidate_tracks, candidate_vias, candidate_zones, selected_path_match) =
            selected_authored_copper_graph_zone_aware_path(self, net_uuid, from_anchor, to_anchor);
        let path =
            selected_path_match.map(|entry| RoutePathCandidateAuthoredCopperGraphZoneAwarePath {
                steps: entry
                    .steps
                    .iter()
                    .map(|step| RoutePathCandidateAuthoredCopperGraphZoneAwareStep {
                        kind: match step.kind {
                            AuthoredCopperGraphZoneAwareStepKind::Track => {
                                RoutePathCandidateAuthoredCopperGraphZoneAwareStepKindView::Track
                            }
                            AuthoredCopperGraphZoneAwareStepKind::Via => {
                                RoutePathCandidateAuthoredCopperGraphZoneAwareStepKindView::Via
                            }
                            AuthoredCopperGraphZoneAwareStepKind::Zone => {
                                RoutePathCandidateAuthoredCopperGraphZoneAwareStepKindView::Zone
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
            });
        let status = if path.is_some() {
            RoutePathCandidateStatus::DeterministicPathFound
        } else {
            RoutePathCandidateStatus::NoPathUnderCurrentAuthoredConstraints
        };

        Ok(RoutePathCandidateAuthoredCopperGraphZoneAwareReport {
            contract: "m5_route_path_candidate_authored_copper_graph_zone_aware_v1".to_string(),
            persisted_native_board_state_only: true,
            selection_rule: ROUTE_PATH_CANDIDATE_AUTHORED_COPPER_GRAPH_ZONE_AWARE_SELECTION_RULE
                .to_string(),
            status,
            net_uuid: preflight.net_uuid,
            net_name: preflight.net_name,
            from_anchor_pad_uuid,
            to_anchor_pad_uuid,
            candidate_copper_layers: preflight.candidate_copper_layers.clone(),
            summary: RoutePathCandidateAuthoredCopperGraphZoneAwareSummary {
                candidate_copper_layer_count: preflight.candidate_copper_layers.len(),
                candidate_track_count: candidate_tracks.len(),
                candidate_via_count: candidate_vias.len(),
                candidate_zone_count: candidate_zones.len(),
                path_step_count: path.as_ref().map(|entry| entry.steps.len()).unwrap_or(0),
            },
            path,
        })
    }
}
