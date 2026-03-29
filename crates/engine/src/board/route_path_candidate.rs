use std::error::Error;
use std::fmt::{Display, Formatter};

use crate::ir::geometry::{LayerId, Point};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use super::route_path_candidate_selection::{
    ROUTE_PATH_CANDIDATE_SELECTION_RULE, matching_corridor_spans, oriented_span_points,
    selected_matching_span,
};
use crate::board::{Board, StackupLayer};

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum RoutePathCandidateError {
    NetNotFound { net_uuid: Uuid },
    AnchorNotOnNet { pad_uuid: Uuid, net_uuid: Uuid },
    DuplicateAnchorPair { pad_uuid: Uuid },
}

impl Display for RoutePathCandidateError {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::NetNotFound { net_uuid } => {
                write!(f, "board net not found in native project: {net_uuid}")
            }
            Self::AnchorNotOnNet { pad_uuid, net_uuid } => {
                write!(f, "anchor pad {pad_uuid} is not an authored anchor on net {net_uuid}")
            }
            Self::DuplicateAnchorPair { pad_uuid } => {
                write!(f, "source and target anchors must differ: {pad_uuid}")
            }
        }
    }
}

impl Error for RoutePathCandidateError {}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum RoutePathCandidateStatus {
    DeterministicPathFound,
    NoPathUnderCurrentAuthoredConstraints,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct RoutePathCandidatePath {
    pub layer: LayerId,
    pub points: Vec<Point>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct RoutePathCandidateSummary {
    pub candidate_copper_layer_count: usize,
    pub matching_span_count: usize,
    pub blocked_span_count: usize,
    pub available_span_count: usize,
    pub path_point_count: usize,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct RoutePathCandidateReport {
    pub contract: String,
    pub persisted_native_board_state_only: bool,
    pub selection_rule: String,
    pub status: RoutePathCandidateStatus,
    pub net_uuid: Uuid,
    pub net_name: String,
    pub from_anchor_pad_uuid: Uuid,
    pub to_anchor_pad_uuid: Uuid,
    pub candidate_copper_layers: Vec<StackupLayer>,
    pub summary: RoutePathCandidateSummary,
    pub path: Option<RoutePathCandidatePath>,
}

impl Board {
    pub fn route_path_candidate(
        &self,
        net_uuid: Uuid,
        from_anchor_pad_uuid: Uuid,
        to_anchor_pad_uuid: Uuid,
    ) -> Result<RoutePathCandidateReport, RoutePathCandidateError> {
        if from_anchor_pad_uuid == to_anchor_pad_uuid {
            return Err(RoutePathCandidateError::DuplicateAnchorPair {
                pad_uuid: from_anchor_pad_uuid,
            });
        }

        let preflight = self
            .route_preflight(net_uuid)
            .ok_or(RoutePathCandidateError::NetNotFound { net_uuid })?;
        if !preflight
            .anchors
            .iter()
            .any(|anchor| anchor.pad_uuid == from_anchor_pad_uuid)
        {
            return Err(RoutePathCandidateError::AnchorNotOnNet {
                pad_uuid: from_anchor_pad_uuid,
                net_uuid,
            });
        }
        if !preflight
            .anchors
            .iter()
            .any(|anchor| anchor.pad_uuid == to_anchor_pad_uuid)
        {
            return Err(RoutePathCandidateError::AnchorNotOnNet {
                pad_uuid: to_anchor_pad_uuid,
                net_uuid,
            });
        }
        let corridor = self
            .route_corridor(net_uuid)
            .ok_or(RoutePathCandidateError::NetNotFound { net_uuid })?;

        let matching_spans = matching_corridor_spans(
            &corridor.corridor_spans,
            from_anchor_pad_uuid,
            to_anchor_pad_uuid,
        );
        let candidate_copper_layers = corridor
            .candidate_copper_layers
            .iter()
            .filter(|layer| matching_spans.iter().any(|span| span.layer == layer.id))
            .cloned()
            .collect::<Vec<_>>();
        let blocked_span_count = matching_spans.iter().filter(|span| span.blocked).count();
        let available_span_count = matching_spans.len().saturating_sub(blocked_span_count);
        let candidate_copper_layer_count = candidate_copper_layers.len();
        let selected_span = selected_matching_span(&matching_spans);
        let path = selected_span.map(|span| RoutePathCandidatePath {
            layer: span.layer,
            points: oriented_span_points(span, from_anchor_pad_uuid, to_anchor_pad_uuid),
        });
        let path_point_count = path.as_ref().map(|entry| entry.points.len()).unwrap_or(0);
        let status = if path.is_some() {
            RoutePathCandidateStatus::DeterministicPathFound
        } else {
            RoutePathCandidateStatus::NoPathUnderCurrentAuthoredConstraints
        };

        Ok(RoutePathCandidateReport {
            contract: "m5_route_path_candidate_v2".to_string(),
            persisted_native_board_state_only: true,
            selection_rule: ROUTE_PATH_CANDIDATE_SELECTION_RULE.to_string(),
            status,
            net_uuid: preflight.net_uuid,
            net_name: preflight.net_name,
            from_anchor_pad_uuid,
            to_anchor_pad_uuid,
            candidate_copper_layers,
            summary: RoutePathCandidateSummary {
                candidate_copper_layer_count,
                matching_span_count: matching_spans.len(),
                blocked_span_count,
                available_span_count,
                path_point_count,
            },
            path,
        })
    }
}
