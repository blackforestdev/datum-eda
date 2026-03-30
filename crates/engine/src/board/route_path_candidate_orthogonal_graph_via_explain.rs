use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::board::{
    Board, RouteCorridorSpanBlockage, RoutePathCandidateError,
    RoutePathCandidateOrthogonalGraphViaReport, RoutePathCandidateStatus, StackupLayer,
};
use crate::ir::geometry::{LayerId, Point};

use super::route_path_candidate_orthogonal_graph_selection::search_orthogonal_graph_layer;
use super::route_path_candidate_via_selection::{
    candidate_vias_for_net, via_matches_anchor_layers,
};

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum RoutePathCandidateOrthogonalGraphViaExplainKind {
    DeterministicPathFound,
    NoMatchingAuthoredVia,
    AllMatchingViasBlocked,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct RoutePathCandidateOrthogonalGraphViaExplainSegment {
    pub layer: LayerId,
    pub points: Vec<Point>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct RoutePathCandidateOrthogonalGraphViaExplainBlockedVia {
    pub via_uuid: Uuid,
    pub via_position: Point,
    pub source_segment: Option<RoutePathCandidateOrthogonalGraphViaExplainSegment>,
    pub target_segment: Option<RoutePathCandidateOrthogonalGraphViaExplainSegment>,
    pub source_blockages: Vec<RouteCorridorSpanBlockage>,
    pub target_blockages: Vec<RouteCorridorSpanBlockage>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct RoutePathCandidateOrthogonalGraphViaExplainSelectedVia {
    pub via_uuid: Uuid,
    pub via_position: Point,
    pub source_segment: RoutePathCandidateOrthogonalGraphViaExplainSegment,
    pub target_segment: RoutePathCandidateOrthogonalGraphViaExplainSegment,
    pub selection_reason: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct RoutePathCandidateOrthogonalGraphViaExplainSummary {
    pub candidate_copper_layer_count: usize,
    pub candidate_via_count: usize,
    pub matching_via_count: usize,
    pub blocked_via_count: usize,
    pub available_via_count: usize,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct RoutePathCandidateOrthogonalGraphViaExplainReport {
    pub contract: String,
    pub persisted_native_board_state_only: bool,
    pub status: RoutePathCandidateStatus,
    pub explanation_kind: RoutePathCandidateOrthogonalGraphViaExplainKind,
    pub net_uuid: Uuid,
    pub net_name: String,
    pub from_anchor_pad_uuid: Uuid,
    pub to_anchor_pad_uuid: Uuid,
    pub selection_rule: String,
    pub component_selection_rules: Vec<String>,
    pub candidate_copper_layers: Vec<StackupLayer>,
    pub summary: RoutePathCandidateOrthogonalGraphViaExplainSummary,
    pub selected_via: Option<RoutePathCandidateOrthogonalGraphViaExplainSelectedVia>,
    pub blocked_matching_vias: Vec<RoutePathCandidateOrthogonalGraphViaExplainBlockedVia>,
}

impl Board {
    pub fn route_path_candidate_orthogonal_graph_via_explain(
        &self,
        net_uuid: Uuid,
        from_anchor_pad_uuid: Uuid,
        to_anchor_pad_uuid: Uuid,
    ) -> Result<RoutePathCandidateOrthogonalGraphViaExplainReport, RoutePathCandidateError> {
        let path_candidate = self.route_path_candidate_orthogonal_graph_via(
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

        let candidate_vias = candidate_vias_for_net(self, net_uuid);
        let matching_vias = candidate_vias
            .iter()
            .filter(|via| via_matches_anchor_layers(via, from_anchor.layer, to_anchor.layer))
            .cloned()
            .collect::<Vec<_>>();

        let via_searches = matching_vias
            .iter()
            .map(|via| {
                let source_search = search_orthogonal_graph_layer(
                    self,
                    net_uuid,
                    from_anchor.layer,
                    from_anchor.position,
                    via.position,
                );
                let target_search = search_orthogonal_graph_layer(
                    self,
                    net_uuid,
                    to_anchor.layer,
                    via.position,
                    to_anchor.position,
                );
                (via.clone(), source_search, target_search)
            })
            .collect::<Vec<_>>();

        let selected_via = via_searches.iter().find_map(|(via, source_search, target_search)| {
            Some(RoutePathCandidateOrthogonalGraphViaExplainSelectedVia {
                via_uuid: via.uuid,
                via_position: via.position,
                source_segment: RoutePathCandidateOrthogonalGraphViaExplainSegment {
                    layer: from_anchor.layer,
                    points: source_search.path.clone()?,
                },
                target_segment: RoutePathCandidateOrthogonalGraphViaExplainSegment {
                    layer: to_anchor.layer,
                    points: target_search.path.clone()?,
                },
                selection_reason: format!(
                    "selected because it is the first matching authored via whose source-layer and target-layer orthogonal graph searches both yield deterministic paths between layers {} and {}",
                    from_anchor.layer, to_anchor.layer
                ),
            })
        });

        let blocked_matching_vias = via_searches
            .iter()
            .filter(|(_, source_search, target_search)| {
                source_search.path.is_none() || target_search.path.is_none()
            })
            .map(|(via, source_search, target_search)| {
                RoutePathCandidateOrthogonalGraphViaExplainBlockedVia {
                    via_uuid: via.uuid,
                    via_position: via.position,
                    source_segment: source_search.path.clone().map(|points| {
                        RoutePathCandidateOrthogonalGraphViaExplainSegment {
                            layer: from_anchor.layer,
                            points,
                        }
                    }),
                    target_segment: target_search.path.clone().map(|points| {
                        RoutePathCandidateOrthogonalGraphViaExplainSegment {
                            layer: to_anchor.layer,
                            points,
                        }
                    }),
                    source_blockages: source_search
                        .blocked_edges
                        .iter()
                        .flat_map(|edge| edge.blockages.clone())
                        .collect(),
                    target_blockages: target_search
                        .blocked_edges
                        .iter()
                        .flat_map(|edge| edge.blockages.clone())
                        .collect(),
                }
            })
            .collect::<Vec<_>>();

        Ok(RoutePathCandidateOrthogonalGraphViaExplainReport {
            contract: "m5_route_path_candidate_orthogonal_graph_via_explain_v1".to_string(),
            persisted_native_board_state_only: true,
            status: path_candidate.status.clone(),
            explanation_kind: explanation_kind(&path_candidate),
            net_uuid: path_candidate.net_uuid,
            net_name: path_candidate.net_name,
            from_anchor_pad_uuid,
            to_anchor_pad_uuid,
            selection_rule: path_candidate.selection_rule,
            component_selection_rules: path_candidate.component_selection_rules,
            candidate_copper_layers: path_candidate.candidate_copper_layers,
            summary: RoutePathCandidateOrthogonalGraphViaExplainSummary {
                candidate_copper_layer_count: path_candidate.summary.candidate_copper_layer_count,
                candidate_via_count: path_candidate.summary.candidate_via_count,
                matching_via_count: path_candidate.summary.matching_via_count,
                blocked_via_count: path_candidate.summary.blocked_via_count,
                available_via_count: path_candidate.summary.available_via_count,
            },
            selected_via,
            blocked_matching_vias,
        })
    }
}

fn explanation_kind(
    report: &RoutePathCandidateOrthogonalGraphViaReport,
) -> RoutePathCandidateOrthogonalGraphViaExplainKind {
    match report.status {
        RoutePathCandidateStatus::DeterministicPathFound => {
            RoutePathCandidateOrthogonalGraphViaExplainKind::DeterministicPathFound
        }
        RoutePathCandidateStatus::NoPathUnderCurrentAuthoredConstraints
            if report.summary.matching_via_count == 0 =>
        {
            RoutePathCandidateOrthogonalGraphViaExplainKind::NoMatchingAuthoredVia
        }
        RoutePathCandidateStatus::NoPathUnderCurrentAuthoredConstraints => {
            RoutePathCandidateOrthogonalGraphViaExplainKind::AllMatchingViasBlocked
        }
    }
}
