use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::board::{
    Board, RouteCorridorSpanBlockage, RoutePathCandidateError,
    RoutePathCandidateOrthogonalGraphPathCost, RoutePathCandidateOrthogonalGraphSixViaReport,
    RoutePathCandidateStatus, StackupLayer,
};
use crate::ir::geometry::{LayerId, Point};

use super::route_path_candidate_orthogonal_graph_selection::{
    orthogonal_graph_path_cost, search_orthogonal_graph_layer,
};
use super::route_path_candidate_six_via_selection::candidate_six_via_matches;

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum RoutePathCandidateOrthogonalGraphSixViaExplainKind {
    DeterministicPathFound,
    NoMatchingAuthoredViaSextuple,
    AllMatchingViaSextuplesBlocked,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct RoutePathCandidateOrthogonalGraphSixViaExplainSegment {
    pub layer: LayerId,
    pub points: Vec<Point>,
    pub cost: RoutePathCandidateOrthogonalGraphPathCost,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct RoutePathCandidateOrthogonalGraphSixViaExplainBlockedSextuple {
    pub via_a_uuid: Uuid,
    pub via_a_position: Point,
    pub via_b_uuid: Uuid,
    pub via_b_position: Point,
    pub via_c_uuid: Uuid,
    pub via_c_position: Point,
    pub via_d_uuid: Uuid,
    pub via_d_position: Point,
    pub via_e_uuid: Uuid,
    pub via_e_position: Point,
    pub via_f_uuid: Uuid,
    pub via_f_position: Point,
    pub first_intermediate_layer: LayerId,
    pub second_intermediate_layer: LayerId,
    pub third_intermediate_layer: LayerId,
    pub fourth_intermediate_layer: LayerId,
    pub fifth_intermediate_layer: LayerId,
    pub source_segment: Option<RoutePathCandidateOrthogonalGraphSixViaExplainSegment>,
    pub first_middle_segment: Option<RoutePathCandidateOrthogonalGraphSixViaExplainSegment>,
    pub second_middle_segment: Option<RoutePathCandidateOrthogonalGraphSixViaExplainSegment>,
    pub third_middle_segment: Option<RoutePathCandidateOrthogonalGraphSixViaExplainSegment>,
    pub fourth_middle_segment: Option<RoutePathCandidateOrthogonalGraphSixViaExplainSegment>,
    pub fifth_middle_segment: Option<RoutePathCandidateOrthogonalGraphSixViaExplainSegment>,
    pub target_segment: Option<RoutePathCandidateOrthogonalGraphSixViaExplainSegment>,
    pub source_blockages: Vec<RouteCorridorSpanBlockage>,
    pub first_middle_blockages: Vec<RouteCorridorSpanBlockage>,
    pub second_middle_blockages: Vec<RouteCorridorSpanBlockage>,
    pub third_middle_blockages: Vec<RouteCorridorSpanBlockage>,
    pub fourth_middle_blockages: Vec<RouteCorridorSpanBlockage>,
    pub fifth_middle_blockages: Vec<RouteCorridorSpanBlockage>,
    pub target_blockages: Vec<RouteCorridorSpanBlockage>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct RoutePathCandidateOrthogonalGraphSixViaExplainSelectedSextuple {
    pub via_a_uuid: Uuid,
    pub via_a_position: Point,
    pub via_b_uuid: Uuid,
    pub via_b_position: Point,
    pub via_c_uuid: Uuid,
    pub via_c_position: Point,
    pub via_d_uuid: Uuid,
    pub via_d_position: Point,
    pub via_e_uuid: Uuid,
    pub via_e_position: Point,
    pub via_f_uuid: Uuid,
    pub via_f_position: Point,
    pub first_intermediate_layer: LayerId,
    pub second_intermediate_layer: LayerId,
    pub third_intermediate_layer: LayerId,
    pub fourth_intermediate_layer: LayerId,
    pub fifth_intermediate_layer: LayerId,
    pub source_segment: RoutePathCandidateOrthogonalGraphSixViaExplainSegment,
    pub first_middle_segment: RoutePathCandidateOrthogonalGraphSixViaExplainSegment,
    pub second_middle_segment: RoutePathCandidateOrthogonalGraphSixViaExplainSegment,
    pub third_middle_segment: RoutePathCandidateOrthogonalGraphSixViaExplainSegment,
    pub fourth_middle_segment: RoutePathCandidateOrthogonalGraphSixViaExplainSegment,
    pub fifth_middle_segment: RoutePathCandidateOrthogonalGraphSixViaExplainSegment,
    pub target_segment: RoutePathCandidateOrthogonalGraphSixViaExplainSegment,
    pub selection_reason: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct RoutePathCandidateOrthogonalGraphSixViaExplainSummary {
    pub candidate_copper_layer_count: usize,
    pub candidate_via_count: usize,
    pub matching_via_sextuple_count: usize,
    pub blocked_via_sextuple_count: usize,
    pub available_via_sextuple_count: usize,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct RoutePathCandidateOrthogonalGraphSixViaExplainReport {
    pub contract: String,
    pub persisted_native_board_state_only: bool,
    pub status: RoutePathCandidateStatus,
    pub explanation_kind: RoutePathCandidateOrthogonalGraphSixViaExplainKind,
    pub net_uuid: Uuid,
    pub net_name: String,
    pub from_anchor_pad_uuid: Uuid,
    pub to_anchor_pad_uuid: Uuid,
    pub selection_rule: String,
    pub component_selection_rules: Vec<String>,
    pub candidate_copper_layers: Vec<StackupLayer>,
    pub summary: RoutePathCandidateOrthogonalGraphSixViaExplainSummary,
    pub selected_sextuple: Option<RoutePathCandidateOrthogonalGraphSixViaExplainSelectedSextuple>,
    pub blocked_matching_sextuples:
        Vec<RoutePathCandidateOrthogonalGraphSixViaExplainBlockedSextuple>,
}

impl Board {
    pub fn route_path_candidate_orthogonal_graph_six_via_explain(
        &self,
        net_uuid: Uuid,
        from_anchor_pad_uuid: Uuid,
        to_anchor_pad_uuid: Uuid,
    ) -> Result<RoutePathCandidateOrthogonalGraphSixViaExplainReport, RoutePathCandidateError> {
        let path_candidate = self.route_path_candidate_orthogonal_graph_six_via(
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

        let (_candidate_vias, matching_sextuples) =
            candidate_six_via_matches(self, net_uuid, from_anchor, to_anchor);
        let sextuple_searches = matching_sextuples
            .iter()
            .map(|entry| {
                let source_search = search_orthogonal_graph_layer(
                    self,
                    net_uuid,
                    from_anchor.layer,
                    from_anchor.position,
                    entry.via_a.position,
                );
                let first_middle_search = search_orthogonal_graph_layer(
                    self,
                    net_uuid,
                    entry.first_intermediate_layer,
                    entry.via_a.position,
                    entry.via_b.position,
                );
                let second_middle_search = search_orthogonal_graph_layer(
                    self,
                    net_uuid,
                    entry.second_intermediate_layer,
                    entry.via_b.position,
                    entry.via_c.position,
                );
                let third_middle_search = search_orthogonal_graph_layer(
                    self,
                    net_uuid,
                    entry.third_intermediate_layer,
                    entry.via_c.position,
                    entry.via_d.position,
                );
                let fourth_middle_search = search_orthogonal_graph_layer(
                    self,
                    net_uuid,
                    entry.fourth_intermediate_layer,
                    entry.via_d.position,
                    entry.via_e.position,
                );
                let fifth_middle_search = search_orthogonal_graph_layer(
                    self,
                    net_uuid,
                    entry.fifth_intermediate_layer,
                    entry.via_e.position,
                    entry.via_f.position,
                );
                let target_search = search_orthogonal_graph_layer(
                    self,
                    net_uuid,
                    to_anchor.layer,
                    entry.via_f.position,
                    to_anchor.position,
                );
                (
                    entry,
                    source_search,
                    first_middle_search,
                    second_middle_search,
                    third_middle_search,
                    fourth_middle_search,
                    fifth_middle_search,
                    target_search,
                )
            })
            .collect::<Vec<_>>();

        let selected_sextuple = sextuple_searches.iter().find_map(
            |(
                entry,
                source_search,
                first_middle_search,
                second_middle_search,
                third_middle_search,
                fourth_middle_search,
                fifth_middle_search,
                target_search,
            )| {
                Some(RoutePathCandidateOrthogonalGraphSixViaExplainSelectedSextuple {
                    via_a_uuid: entry.via_a.uuid,
                    via_a_position: entry.via_a.position,
                    via_b_uuid: entry.via_b.uuid,
                    via_b_position: entry.via_b.position,
                    via_c_uuid: entry.via_c.uuid,
                    via_c_position: entry.via_c.position,
                    via_d_uuid: entry.via_d.uuid,
                    via_d_position: entry.via_d.position,
                    via_e_uuid: entry.via_e.uuid,
                    via_e_position: entry.via_e.position,
                    via_f_uuid: entry.via_f.uuid,
                    via_f_position: entry.via_f.position,
                    first_intermediate_layer: entry.first_intermediate_layer,
                    second_intermediate_layer: entry.second_intermediate_layer,
                    third_intermediate_layer: entry.third_intermediate_layer,
                    fourth_intermediate_layer: entry.fourth_intermediate_layer,
                    fifth_intermediate_layer: entry.fifth_intermediate_layer,
                    source_segment: RoutePathCandidateOrthogonalGraphSixViaExplainSegment {
                        layer: from_anchor.layer,
                        points: source_search.path.clone()?,
                        cost: orthogonal_graph_path_cost(source_search.path.as_ref()?),
                    },
                    first_middle_segment: RoutePathCandidateOrthogonalGraphSixViaExplainSegment {
                        layer: entry.first_intermediate_layer,
                        points: first_middle_search.path.clone()?,
                        cost: orthogonal_graph_path_cost(first_middle_search.path.as_ref()?),
                    },
                    second_middle_segment: RoutePathCandidateOrthogonalGraphSixViaExplainSegment {
                        layer: entry.second_intermediate_layer,
                        points: second_middle_search.path.clone()?,
                        cost: orthogonal_graph_path_cost(second_middle_search.path.as_ref()?),
                    },
                    third_middle_segment: RoutePathCandidateOrthogonalGraphSixViaExplainSegment {
                        layer: entry.third_intermediate_layer,
                        points: third_middle_search.path.clone()?,
                        cost: orthogonal_graph_path_cost(third_middle_search.path.as_ref()?),
                    },
                    fourth_middle_segment: RoutePathCandidateOrthogonalGraphSixViaExplainSegment {
                        layer: entry.fourth_intermediate_layer,
                        points: fourth_middle_search.path.clone()?,
                        cost: orthogonal_graph_path_cost(fourth_middle_search.path.as_ref()?),
                    },
                    fifth_middle_segment: RoutePathCandidateOrthogonalGraphSixViaExplainSegment {
                        layer: entry.fifth_intermediate_layer,
                        points: fifth_middle_search.path.clone()?,
                        cost: orthogonal_graph_path_cost(fifth_middle_search.path.as_ref()?),
                    },
                    target_segment: RoutePathCandidateOrthogonalGraphSixViaExplainSegment {
                        layer: to_anchor.layer,
                        points: target_search.path.clone()?,
                        cost: orthogonal_graph_path_cost(target_search.path.as_ref()?),
                    },
                    selection_reason: format!(
                        "selected because it is the first matching authored via sextuple whose source-layer, first-intermediate-layer, second-intermediate-layer, third-intermediate-layer, fourth-intermediate-layer, fifth-intermediate-layer, and target-layer orthogonal graph searches all yield deterministic paths between layers {} and {}; each layer-side path is the lowest-cost graph path under the accepted ranking rule",
                        from_anchor.layer, to_anchor.layer
                    ),
                })
            },
        );

        let blocked_matching_sextuples = sextuple_searches
            .iter()
            .filter(|(_, source_search, first_middle_search, second_middle_search, third_middle_search, fourth_middle_search, fifth_middle_search, target_search)| {
                source_search.path.is_none()
                    || first_middle_search.path.is_none()
                    || second_middle_search.path.is_none()
                    || third_middle_search.path.is_none()
                    || fourth_middle_search.path.is_none()
                    || fifth_middle_search.path.is_none()
                    || target_search.path.is_none()
            })
            .map(|(entry, source_search, first_middle_search, second_middle_search, third_middle_search, fourth_middle_search, fifth_middle_search, target_search)| {
                RoutePathCandidateOrthogonalGraphSixViaExplainBlockedSextuple {
                    via_a_uuid: entry.via_a.uuid,
                    via_a_position: entry.via_a.position,
                    via_b_uuid: entry.via_b.uuid,
                    via_b_position: entry.via_b.position,
                    via_c_uuid: entry.via_c.uuid,
                    via_c_position: entry.via_c.position,
                    via_d_uuid: entry.via_d.uuid,
                    via_d_position: entry.via_d.position,
                    via_e_uuid: entry.via_e.uuid,
                    via_e_position: entry.via_e.position,
                    via_f_uuid: entry.via_f.uuid,
                    via_f_position: entry.via_f.position,
                    first_intermediate_layer: entry.first_intermediate_layer,
                    second_intermediate_layer: entry.second_intermediate_layer,
                    third_intermediate_layer: entry.third_intermediate_layer,
                    fourth_intermediate_layer: entry.fourth_intermediate_layer,
                    fifth_intermediate_layer: entry.fifth_intermediate_layer,
                    source_segment: source_search.path.clone().map(|points| {
                        let cost = orthogonal_graph_path_cost(&points);
                        RoutePathCandidateOrthogonalGraphSixViaExplainSegment { layer: from_anchor.layer, points, cost }
                    }),
                    first_middle_segment: first_middle_search.path.clone().map(|points| {
                        let cost = orthogonal_graph_path_cost(&points);
                        RoutePathCandidateOrthogonalGraphSixViaExplainSegment { layer: entry.first_intermediate_layer, points, cost }
                    }),
                    second_middle_segment: second_middle_search.path.clone().map(|points| {
                        let cost = orthogonal_graph_path_cost(&points);
                        RoutePathCandidateOrthogonalGraphSixViaExplainSegment { layer: entry.second_intermediate_layer, points, cost }
                    }),
                    third_middle_segment: third_middle_search.path.clone().map(|points| {
                        let cost = orthogonal_graph_path_cost(&points);
                        RoutePathCandidateOrthogonalGraphSixViaExplainSegment { layer: entry.third_intermediate_layer, points, cost }
                    }),
                    fourth_middle_segment: fourth_middle_search.path.clone().map(|points| {
                        let cost = orthogonal_graph_path_cost(&points);
                        RoutePathCandidateOrthogonalGraphSixViaExplainSegment { layer: entry.fourth_intermediate_layer, points, cost }
                    }),
                    fifth_middle_segment: fifth_middle_search.path.clone().map(|points| {
                        let cost = orthogonal_graph_path_cost(&points);
                        RoutePathCandidateOrthogonalGraphSixViaExplainSegment { layer: entry.fifth_intermediate_layer, points, cost }
                    }),
                    target_segment: target_search.path.clone().map(|points| {
                        let cost = orthogonal_graph_path_cost(&points);
                        RoutePathCandidateOrthogonalGraphSixViaExplainSegment { layer: to_anchor.layer, points, cost }
                    }),
                    source_blockages: source_search.blocked_edges.iter().flat_map(|edge| edge.blockages.clone()).collect(),
                    first_middle_blockages: first_middle_search.blocked_edges.iter().flat_map(|edge| edge.blockages.clone()).collect(),
                    second_middle_blockages: second_middle_search.blocked_edges.iter().flat_map(|edge| edge.blockages.clone()).collect(),
                    third_middle_blockages: third_middle_search.blocked_edges.iter().flat_map(|edge| edge.blockages.clone()).collect(),
                    fourth_middle_blockages: fourth_middle_search.blocked_edges.iter().flat_map(|edge| edge.blockages.clone()).collect(),
                    fifth_middle_blockages: fifth_middle_search.blocked_edges.iter().flat_map(|edge| edge.blockages.clone()).collect(),
                    target_blockages: target_search.blocked_edges.iter().flat_map(|edge| edge.blockages.clone()).collect(),
                }
            })
            .collect::<Vec<_>>();

        Ok(RoutePathCandidateOrthogonalGraphSixViaExplainReport {
            contract: "m5_route_path_candidate_orthogonal_graph_six_via_explain_v1".to_string(),
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
            summary: RoutePathCandidateOrthogonalGraphSixViaExplainSummary {
                candidate_copper_layer_count: path_candidate.summary.candidate_copper_layer_count,
                candidate_via_count: path_candidate.summary.candidate_via_count,
                matching_via_sextuple_count: path_candidate.summary.matching_via_sextuple_count,
                blocked_via_sextuple_count: path_candidate.summary.blocked_via_sextuple_count,
                available_via_sextuple_count: path_candidate.summary.available_via_sextuple_count,
            },
            selected_sextuple,
            blocked_matching_sextuples,
        })
    }
}

fn explanation_kind(
    report: &RoutePathCandidateOrthogonalGraphSixViaReport,
) -> RoutePathCandidateOrthogonalGraphSixViaExplainKind {
    match report.status {
        RoutePathCandidateStatus::DeterministicPathFound => {
            RoutePathCandidateOrthogonalGraphSixViaExplainKind::DeterministicPathFound
        }
        RoutePathCandidateStatus::NoPathUnderCurrentAuthoredConstraints
            if report.summary.matching_via_sextuple_count == 0 =>
        {
            RoutePathCandidateOrthogonalGraphSixViaExplainKind::NoMatchingAuthoredViaSextuple
        }
        RoutePathCandidateStatus::NoPathUnderCurrentAuthoredConstraints => {
            RoutePathCandidateOrthogonalGraphSixViaExplainKind::AllMatchingViaSextuplesBlocked
        }
    }
}
