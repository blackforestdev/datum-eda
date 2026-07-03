//! Orthogonal-graph route-proposal drift classification and segment facts.
//!
//! Moved from `crates/cli/src/command_project_route_proposal.rs` (family F).
//! Drift kinds, message rendering, and per-layer-segment facts are
//! byte-for-byte the historical CLI behavior — the rendered messages gate
//! artifact apply and land in locked CLI tests.

use std::collections::BTreeMap;

use super::RouteProposalAction;

/// How a saved orthogonal-graph proposal artifact drifted from the live
/// deterministic rebuild.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum OrthogonalGraphArtifactDriftKind {
    CandidateAvailabilityChanged,
    DeterministicCostWinnerChanged,
    GeometryChanged,
}

impl OrthogonalGraphArtifactDriftKind {
    pub fn as_str(self) -> &'static str {
        match self {
            Self::CandidateAvailabilityChanged => "candidate_availability_changed",
            Self::DeterministicCostWinnerChanged => "deterministic_cost_winner_changed",
            Self::GeometryChanged => "geometry_changed",
        }
    }
}

/// Per-layer-segment facts extracted from an orthogonal-graph action set.
#[derive(Debug, Clone)]
pub struct OrthogonalGraphSegmentFacts {
    pub layer_segment_index: usize,
    pub layer_segment_count: usize,
    pub layer: i32,
    pub bend_count: usize,
    pub point_count: usize,
    pub track_action_count: usize,
}

/// Artifact-vs-live comparison of one layer segment.
#[derive(Debug, Clone)]
pub struct OrthogonalGraphSegmentComparison {
    pub layer_segment_index: usize,
    pub layer_segment_count: usize,
    pub artifact_layer: i32,
    pub artifact_bend_count: usize,
    pub artifact_point_count: usize,
    pub artifact_track_action_count: usize,
    pub live_layer: Option<i32>,
    pub live_bend_count: Option<usize>,
    pub live_point_count: Option<usize>,
    pub live_track_action_count: Option<usize>,
    pub matches_live: bool,
}

pub(super) fn orthogonal_graph_route_proposal_artifact_drift_kind(
    contract: &str,
    artifact_actions: &[RouteProposalAction],
    live_actions: &Result<Vec<RouteProposalAction>, String>,
) -> Option<OrthogonalGraphArtifactDriftKind> {
    if !is_orthogonal_graph_route_proposal_contract(contract) {
        return None;
    }

    let artifact_first = artifact_actions.first()?;
    match live_actions {
        Ok(live_actions) => {
            let live_first = live_actions.first()?;
            if live_actions == artifact_actions {
                return None;
            }
            if artifact_first.selected_path_bend_count != live_first.selected_path_bend_count
                || artifact_first.selected_path_point_count != live_first.selected_path_point_count
                || artifact_first.selected_path_segment_count
                    != live_first.selected_path_segment_count
                || artifact_first.reused_via_uuids != live_first.reused_via_uuids
            {
                Some(OrthogonalGraphArtifactDriftKind::DeterministicCostWinnerChanged)
            } else {
                Some(OrthogonalGraphArtifactDriftKind::GeometryChanged)
            }
        }
        Err(_) => Some(OrthogonalGraphArtifactDriftKind::CandidateAvailabilityChanged),
    }
}

fn is_orthogonal_graph_route_proposal_contract(contract: &str) -> bool {
    matches!(
        contract,
        "m5_route_path_candidate_orthogonal_graph_v1"
            | "m5_route_path_candidate_orthogonal_graph_via_v1"
            | "m5_route_path_candidate_orthogonal_graph_two_via_v1"
            | "m5_route_path_candidate_orthogonal_graph_three_via_v1"
            | "m5_route_path_candidate_orthogonal_graph_four_via_v1"
            | "m5_route_path_candidate_orthogonal_graph_five_via_v1"
            | "m5_route_path_candidate_orthogonal_graph_six_via_v1"
    )
}

pub(super) fn render_orthogonal_graph_route_proposal_drift_message(
    drift_kind: OrthogonalGraphArtifactDriftKind,
    artifact_action: &RouteProposalAction,
    live_action: Option<&RouteProposalAction>,
    error: Option<&String>,
) -> String {
    match drift_kind {
        OrthogonalGraphArtifactDriftKind::CandidateAvailabilityChanged => format!(
            "candidate availability changed under current authored constraints (artifact bends={}, points={}, segments={}; live rebuild failed: {})",
            artifact_action.selected_path_bend_count,
            artifact_action.selected_path_point_count,
            artifact_action.selected_path_segment_count,
            error
                .map(|error| error.to_string())
                .unwrap_or_else(|| "unknown live rebuild failure".to_string())
        ),
        OrthogonalGraphArtifactDriftKind::DeterministicCostWinnerChanged => {
            let live_action = live_action.expect("cost-winner drift requires a live action");
            format!(
                "deterministic cost winner changed (bends {} -> {}, points {} -> {}, segments {} -> {}, reused_vias {} -> {})",
                artifact_action.selected_path_bend_count,
                live_action.selected_path_bend_count,
                artifact_action.selected_path_point_count,
                live_action.selected_path_point_count,
                artifact_action.selected_path_segment_count,
                live_action.selected_path_segment_count,
                artifact_action.reused_via_uuids.len(),
                live_action.reused_via_uuids.len(),
            )
        }
        OrthogonalGraphArtifactDriftKind::GeometryChanged => {
            let live_action = live_action.expect("geometry drift requires a live action");
            format!(
                "geometry changed under the same ranked path (artifact action {} no longer matches live action {})",
                artifact_action.action_id, live_action.action_id
            )
        }
    }
}

/// Compare artifact vs live per-layer-segment facts. Returns `None` when the
/// artifact actions carry no layer-segment evidence.
pub fn orthogonal_graph_route_proposal_segment_comparison(
    artifact_actions: &[RouteProposalAction],
    live_actions: Option<&[RouteProposalAction]>,
) -> Option<Vec<OrthogonalGraphSegmentComparison>> {
    let artifact_segments = orthogonal_graph_route_proposal_segment_facts(artifact_actions)?;
    let live_segments = live_actions.and_then(orthogonal_graph_route_proposal_segment_facts);
    let live_segment_map = live_segments
        .unwrap_or_default()
        .into_iter()
        .map(|segment| (segment.layer_segment_index, segment))
        .collect::<BTreeMap<_, _>>();
    Some(
        artifact_segments
            .into_iter()
            .map(|artifact_segment| {
                let live_segment = live_segment_map.get(&artifact_segment.layer_segment_index);
                OrthogonalGraphSegmentComparison {
                    layer_segment_index: artifact_segment.layer_segment_index,
                    layer_segment_count: artifact_segment.layer_segment_count,
                    artifact_layer: artifact_segment.layer,
                    artifact_bend_count: artifact_segment.bend_count,
                    artifact_point_count: artifact_segment.point_count,
                    artifact_track_action_count: artifact_segment.track_action_count,
                    live_layer: live_segment.map(|segment| segment.layer),
                    live_bend_count: live_segment.map(|segment| segment.bend_count),
                    live_point_count: live_segment.map(|segment| segment.point_count),
                    live_track_action_count: live_segment.map(|segment| segment.track_action_count),
                    matches_live: live_segment.is_some_and(|segment| {
                        segment.layer == artifact_segment.layer
                            && segment.bend_count == artifact_segment.bend_count
                            && segment.point_count == artifact_segment.point_count
                            && segment.track_action_count == artifact_segment.track_action_count
                    }),
                }
            })
            .collect(),
    )
}

/// Group an action set's layer-segment evidence. Returns `None` when the
/// first grouping key is absent (non-layer-segmented contracts).
pub fn orthogonal_graph_route_proposal_segment_facts(
    actions: &[RouteProposalAction],
) -> Option<Vec<OrthogonalGraphSegmentFacts>> {
    let mut grouped = BTreeMap::<usize, Vec<&RouteProposalAction>>::new();
    for action in actions {
        let layer_segment_index = action.selected_path_layer_segment_index?;
        grouped.entry(layer_segment_index).or_default().push(action);
    }
    Some(
        grouped
            .into_iter()
            .map(|(layer_segment_index, grouped_actions)| {
                let first = grouped_actions[0];
                OrthogonalGraphSegmentFacts {
                    layer_segment_index,
                    layer_segment_count: first.selected_path_layer_segment_count.unwrap_or(0),
                    layer: first.layer,
                    bend_count: first.selected_path_layer_segment_bend_count.unwrap_or(0),
                    point_count: first.selected_path_layer_segment_point_count.unwrap_or(0),
                    track_action_count: grouped_actions.len(),
                }
            })
            .collect(),
    )
}

#[cfg(test)]
mod tests {
    use uuid::Uuid;

    use crate::ir::geometry::Point;

    use super::*;

    fn segmented_action(layer_segment_index: usize, layer: i32) -> RouteProposalAction {
        RouteProposalAction {
            action_id: format!("action-{layer_segment_index}-{layer}"),
            proposal_action: "draw_track".to_string(),
            reason: "route_path_candidate_orthogonal_graph_via".to_string(),
            contract: "m5_route_path_candidate_orthogonal_graph_via_v1".to_string(),
            net_uuid: Uuid::nil(),
            net_name: "SIG".to_string(),
            from_anchor_pad_uuid: Uuid::nil(),
            to_anchor_pad_uuid: Uuid::nil(),
            layer,
            width_nm: 200_000,
            from: Point { x: 0, y: 0 },
            to: Point { x: 1, y: 0 },
            reused_via_uuid: None,
            reused_via_uuids: Vec::new(),
            reused_object_kind: None,
            reused_object_uuid: None,
            reused_object_from_layer: None,
            reused_object_to_layer: None,
            selected_path_bend_count: 1,
            selected_path_point_count: 4,
            selected_path_segment_index: 0,
            selected_path_segment_count: 2,
            selected_path_layer_segment_index: Some(layer_segment_index),
            selected_path_layer_segment_count: Some(2),
            selected_path_layer_segment_bend_count: Some(1),
            selected_path_layer_segment_point_count: Some(2),
        }
    }

    #[test]
    fn segment_facts_group_actions_by_layer_segment_index() {
        let actions = vec![
            segmented_action(0, 1),
            segmented_action(0, 1),
            segmented_action(1, 3),
        ];
        let facts =
            orthogonal_graph_route_proposal_segment_facts(&actions).expect("facts should exist");
        assert_eq!(facts.len(), 2);
        assert_eq!(facts[0].layer_segment_index, 0);
        assert_eq!(facts[0].track_action_count, 2);
        assert_eq!(facts[1].layer, 3);
        assert_eq!(facts[1].track_action_count, 1);
    }

    #[test]
    fn segment_facts_are_none_for_non_layer_segmented_actions() {
        let mut action = segmented_action(0, 1);
        action.selected_path_layer_segment_index = None;
        assert!(orthogonal_graph_route_proposal_segment_facts(&[action]).is_none());
    }

    #[test]
    fn drift_kind_classifies_availability_winner_and_geometry_changes() {
        let contract = "m5_route_path_candidate_orthogonal_graph_via_v1";
        let artifact = vec![segmented_action(0, 1)];

        let unavailable = Err("no path".to_string());
        assert_eq!(
            orthogonal_graph_route_proposal_artifact_drift_kind(contract, &artifact, &unavailable),
            Some(OrthogonalGraphArtifactDriftKind::CandidateAvailabilityChanged)
        );

        let mut winner_changed = segmented_action(0, 1);
        winner_changed.selected_path_bend_count = 9;
        assert_eq!(
            orthogonal_graph_route_proposal_artifact_drift_kind(
                contract,
                &artifact,
                &Ok(vec![winner_changed]),
            ),
            Some(OrthogonalGraphArtifactDriftKind::DeterministicCostWinnerChanged)
        );

        let mut geometry_changed = segmented_action(0, 1);
        geometry_changed.to = Point { x: 2, y: 0 };
        assert_eq!(
            orthogonal_graph_route_proposal_artifact_drift_kind(
                contract,
                &artifact,
                &Ok(vec![geometry_changed]),
            ),
            Some(OrthogonalGraphArtifactDriftKind::GeometryChanged)
        );

        assert_eq!(
            orthogonal_graph_route_proposal_artifact_drift_kind(
                contract,
                &artifact,
                &Ok(artifact.clone()),
            ),
            None
        );
        assert_eq!(
            orthogonal_graph_route_proposal_artifact_drift_kind(
                "m5_route_path_candidate_v2",
                &artifact,
                &unavailable,
            ),
            None,
            "non-orthogonal-graph contracts carry no drift kind"
        );
    }
}
