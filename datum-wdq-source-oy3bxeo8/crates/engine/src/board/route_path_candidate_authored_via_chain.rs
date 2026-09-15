use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::board::{Board, RoutePathCandidateError, RoutePathCandidateStatus, StackupLayer};
use crate::ir::geometry::{LayerId, Point};

use super::route_path_candidate_authored_via_chain_selection::{
    ROUTE_PATH_CANDIDATE_AUTHORED_VIA_CHAIN_SELECTION_RULE, authored_via_chain_path_points,
    candidate_authored_via_chain_matches, selected_matching_authored_via_chain,
};

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct RoutePathCandidateAuthoredViaChainSegment {
    pub layer: LayerId,
    pub points: Vec<Point>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct RoutePathCandidateAuthoredViaChainVia {
    pub via_uuid: Uuid,
    pub via_position: Point,
    pub from_layer: LayerId,
    pub to_layer: LayerId,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct RoutePathCandidateAuthoredViaChainPath {
    pub via_chain: Vec<RoutePathCandidateAuthoredViaChainVia>,
    pub segments: Vec<RoutePathCandidateAuthoredViaChainSegment>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct RoutePathCandidateAuthoredViaChainSummary {
    pub candidate_copper_layer_count: usize,
    pub candidate_via_count: usize,
    pub matching_via_chain_count: usize,
    pub blocked_via_chain_count: usize,
    pub available_via_chain_count: usize,
    pub path_segment_count: usize,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct RoutePathCandidateAuthoredViaChainReport {
    pub contract: String,
    pub persisted_native_board_state_only: bool,
    pub selection_rule: String,
    pub status: RoutePathCandidateStatus,
    pub net_uuid: Uuid,
    pub net_name: String,
    pub from_anchor_pad_uuid: Uuid,
    pub to_anchor_pad_uuid: Uuid,
    pub candidate_copper_layers: Vec<StackupLayer>,
    pub summary: RoutePathCandidateAuthoredViaChainSummary,
    pub path: Option<RoutePathCandidateAuthoredViaChainPath>,
}

impl Board {
    pub fn route_path_candidate_authored_via_chain(
        &self,
        net_uuid: Uuid,
        from_anchor_pad_uuid: Uuid,
        to_anchor_pad_uuid: Uuid,
    ) -> Result<RoutePathCandidateAuthoredViaChainReport, RoutePathCandidateError> {
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

        let (candidate_vias, matching_chains) =
            candidate_authored_via_chain_matches(self, net_uuid, from_anchor, to_anchor);
        let blocked_via_chain_count = matching_chains
            .iter()
            .filter(|entry| {
                entry
                    .segment_analyses
                    .iter()
                    .any(|segment| !segment.blockages.is_empty())
            })
            .count();
        let available_via_chain_count = matching_chains
            .len()
            .saturating_sub(blocked_via_chain_count);
        let selected_path = selected_matching_authored_via_chain(&matching_chains).map(|entry| {
            let points = authored_via_chain_path_points(entry, from_anchor, to_anchor);
            RoutePathCandidateAuthoredViaChainPath {
                via_chain: entry
                    .vias
                    .iter()
                    .map(|via| RoutePathCandidateAuthoredViaChainVia {
                        via_uuid: via.uuid,
                        via_position: via.position,
                        from_layer: via.from_layer,
                        to_layer: via.to_layer,
                    })
                    .collect(),
                segments: points
                    .iter()
                    .enumerate()
                    .map(
                        |(index, segment)| RoutePathCandidateAuthoredViaChainSegment {
                            layer: entry.segment_layers[index],
                            points: segment.to_vec(),
                        },
                    )
                    .collect(),
            }
        });
        let status = if selected_path.is_some() {
            RoutePathCandidateStatus::DeterministicPathFound
        } else {
            RoutePathCandidateStatus::NoPathUnderCurrentAuthoredConstraints
        };

        Ok(RoutePathCandidateAuthoredViaChainReport {
            contract: "m5_route_path_candidate_authored_via_chain_v1".to_string(),
            persisted_native_board_state_only: true,
            selection_rule: ROUTE_PATH_CANDIDATE_AUTHORED_VIA_CHAIN_SELECTION_RULE.to_string(),
            status,
            net_uuid: preflight.net_uuid,
            net_name: preflight.net_name,
            from_anchor_pad_uuid,
            to_anchor_pad_uuid,
            candidate_copper_layers: preflight.candidate_copper_layers.clone(),
            summary: RoutePathCandidateAuthoredViaChainSummary {
                candidate_copper_layer_count: preflight.candidate_copper_layers.len(),
                candidate_via_count: candidate_vias.len(),
                matching_via_chain_count: matching_chains.len(),
                blocked_via_chain_count,
                available_via_chain_count,
                path_segment_count: selected_path
                    .as_ref()
                    .map(|path| path.segments.len())
                    .unwrap_or(0),
            },
            path: selected_path,
        })
    }
}
