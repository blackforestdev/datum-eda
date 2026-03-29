use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::board::{
    Board, RouteCorridorSpanBlockage, RoutePathCandidateAuthoredViaChainReport,
    RoutePathCandidateError, RoutePathCandidateStatus, StackupLayer,
};
use crate::ir::geometry::{LayerId, Point};

use super::route_path_candidate_authored_via_chain_selection::{
    authored_via_chain_path_points, candidate_authored_via_chain_matches,
    selected_matching_authored_via_chain,
};

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum RoutePathCandidateAuthoredViaChainExplainKind {
    DeterministicPathFound,
    NoMatchingAuthoredViaChain,
    AllMatchingViaChainsBlocked,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct RoutePathCandidateAuthoredViaChainExplainSegment {
    pub layer: LayerId,
    pub points: Vec<Point>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct RoutePathCandidateAuthoredViaChainExplainVia {
    pub via_uuid: Uuid,
    pub via_position: Point,
    pub from_layer: LayerId,
    pub to_layer: LayerId,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct RoutePathCandidateAuthoredViaChainExplainBlockedChain {
    pub via_chain: Vec<RoutePathCandidateAuthoredViaChainExplainVia>,
    pub segments: Vec<RoutePathCandidateAuthoredViaChainExplainSegment>,
    pub segment_blockages: Vec<Vec<RouteCorridorSpanBlockage>>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct RoutePathCandidateAuthoredViaChainExplainSelectedChain {
    pub via_chain: Vec<RoutePathCandidateAuthoredViaChainExplainVia>,
    pub segments: Vec<RoutePathCandidateAuthoredViaChainExplainSegment>,
    pub selection_reason: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct RoutePathCandidateAuthoredViaChainExplainSummary {
    pub candidate_copper_layer_count: usize,
    pub candidate_via_count: usize,
    pub matching_via_chain_count: usize,
    pub blocked_via_chain_count: usize,
    pub available_via_chain_count: usize,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct RoutePathCandidateAuthoredViaChainExplainReport {
    pub contract: String,
    pub persisted_native_board_state_only: bool,
    pub status: RoutePathCandidateStatus,
    pub explanation_kind: RoutePathCandidateAuthoredViaChainExplainKind,
    pub net_uuid: Uuid,
    pub net_name: String,
    pub from_anchor_pad_uuid: Uuid,
    pub to_anchor_pad_uuid: Uuid,
    pub selection_rule: String,
    pub candidate_copper_layers: Vec<StackupLayer>,
    pub summary: RoutePathCandidateAuthoredViaChainExplainSummary,
    pub selected_chain: Option<RoutePathCandidateAuthoredViaChainExplainSelectedChain>,
    pub blocked_matching_chains: Vec<RoutePathCandidateAuthoredViaChainExplainBlockedChain>,
}

impl Board {
    pub fn route_path_candidate_authored_via_chain_explain(
        &self,
        net_uuid: Uuid,
        from_anchor_pad_uuid: Uuid,
        to_anchor_pad_uuid: Uuid,
    ) -> Result<RoutePathCandidateAuthoredViaChainExplainReport, RoutePathCandidateError> {
        let path_candidate = self.route_path_candidate_authored_via_chain(
            net_uuid,
            from_anchor_pad_uuid,
            to_anchor_pad_uuid,
        )?;
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

        let (_, matching_chains) =
            candidate_authored_via_chain_matches(self, net_uuid, from_anchor, to_anchor);

        let selected_chain = selected_matching_authored_via_chain(&matching_chains).map(|entry| {
            let points = authored_via_chain_path_points(entry, from_anchor, to_anchor);
            RoutePathCandidateAuthoredViaChainExplainSelectedChain {
                via_chain: entry
                    .vias
                    .iter()
                    .map(|via| RoutePathCandidateAuthoredViaChainExplainVia {
                        via_uuid: via.uuid,
                        via_position: via.position,
                        from_layer: via.from_layer,
                        to_layer: via.to_layer,
                    })
                    .collect(),
                segments: points
                    .iter()
                    .enumerate()
                    .map(|(index, segment)| RoutePathCandidateAuthoredViaChainExplainSegment {
                        layer: entry.segment_layers[index],
                        points: segment.to_vec(),
                    })
                    .collect(),
                selection_reason: format!(
                    "selected because it is the first unblocked matching authored via chain under the deterministic selection rule with via_count {} and via_uuid_sequence {}",
                    entry.vias.len(),
                    entry
                        .vias
                        .iter()
                        .map(|via| via.uuid.to_string())
                        .collect::<Vec<_>>()
                        .join(",")
                ),
            }
        });

        let blocked_matching_chains = matching_chains
            .iter()
            .filter(|entry| {
                entry
                    .segment_analyses
                    .iter()
                    .any(|segment| !segment.blockages.is_empty())
            })
            .map(|entry| {
                let points = authored_via_chain_path_points(entry, from_anchor, to_anchor);
                RoutePathCandidateAuthoredViaChainExplainBlockedChain {
                    via_chain: entry
                        .vias
                        .iter()
                        .map(|via| RoutePathCandidateAuthoredViaChainExplainVia {
                            via_uuid: via.uuid,
                            via_position: via.position,
                            from_layer: via.from_layer,
                            to_layer: via.to_layer,
                        })
                        .collect(),
                    segments: points
                        .iter()
                        .enumerate()
                        .map(|(index, segment)| RoutePathCandidateAuthoredViaChainExplainSegment {
                            layer: entry.segment_layers[index],
                            points: segment.to_vec(),
                        })
                        .collect(),
                    segment_blockages: entry
                        .segment_analyses
                        .iter()
                        .map(|segment| segment.blockages.clone())
                        .collect(),
                }
            })
            .collect::<Vec<_>>();

        Ok(RoutePathCandidateAuthoredViaChainExplainReport {
            contract: "m5_route_path_candidate_authored_via_chain_explain_v1".to_string(),
            persisted_native_board_state_only: true,
            status: path_candidate.status.clone(),
            explanation_kind: explanation_kind(&path_candidate),
            net_uuid: path_candidate.net_uuid,
            net_name: path_candidate.net_name,
            from_anchor_pad_uuid,
            to_anchor_pad_uuid,
            selection_rule: path_candidate.selection_rule,
            candidate_copper_layers: path_candidate.candidate_copper_layers,
            summary: RoutePathCandidateAuthoredViaChainExplainSummary {
                candidate_copper_layer_count: path_candidate.summary.candidate_copper_layer_count,
                candidate_via_count: path_candidate.summary.candidate_via_count,
                matching_via_chain_count: path_candidate.summary.matching_via_chain_count,
                blocked_via_chain_count: path_candidate.summary.blocked_via_chain_count,
                available_via_chain_count: path_candidate.summary.available_via_chain_count,
            },
            selected_chain,
            blocked_matching_chains,
        })
    }
}

fn explanation_kind(
    report: &RoutePathCandidateAuthoredViaChainReport,
) -> RoutePathCandidateAuthoredViaChainExplainKind {
    match report.status {
        RoutePathCandidateStatus::DeterministicPathFound => {
            RoutePathCandidateAuthoredViaChainExplainKind::DeterministicPathFound
        }
        RoutePathCandidateStatus::NoPathUnderCurrentAuthoredConstraints
            if report.summary.matching_via_chain_count == 0 =>
        {
            RoutePathCandidateAuthoredViaChainExplainKind::NoMatchingAuthoredViaChain
        }
        RoutePathCandidateStatus::NoPathUnderCurrentAuthoredConstraints => {
            RoutePathCandidateAuthoredViaChainExplainKind::AllMatchingViaChainsBlocked
        }
    }
}
