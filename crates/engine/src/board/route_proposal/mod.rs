//! Route-proposal domain logic: deterministic proposal-action generation,
//! selection/strategy comparison, and artifact revalidation + drift analysis.
//!
//! Family F of the native-write migration moved this logic out of
//! `crates/cli/src/command_project_route_proposal.rs`. The CLI keeps its
//! view/report structs and artifact-file (de)serialization and delegates the
//! domain work here: everything in this module takes resolved board data
//! (`crate::board::Board`) and returns typed results — no disk access and no
//! operation authoring. Errors are plain `String`s carrying byte-for-byte the
//! historical CLI error messages (the CLI wraps them in `anyhow` without
//! reformatting), because selection candidate messages and drift diagnostics
//! land in exported artifacts and locked CLI test expectations.
//!
//! Module map:
//! - [`generation`] — per-candidate proposal-action builders and the
//!   candidate dispatcher (the ~20-strategy deterministic kernel surface)
//! - [`selection`] — profile-ordered first-win candidate selection
//! - [`revalidation`] — artifact-vs-live rebuild and drift classification
//! - [`drift`] — orthogonal-graph drift kinds, messages, and segment facts
//! - [`apply`] — accepted-proposal composition over the family-D
//!   `native_write::board_routing` builders (the one place in this module
//!   that touches operations, and only through the facade)
//! - [`fixtures`] — route-strategy regression fixture board composition over
//!   the native-write facade (deterministic fixture generation only)

mod apply;
mod drift;
mod fixtures;
mod generation;
mod revalidation;
mod selection;

pub use apply::{BuiltRouteProposal, BuiltRouteTrack, build_accepted_route_proposal};
pub use drift::{
    OrthogonalGraphArtifactDriftKind, OrthogonalGraphSegmentComparison,
    OrthogonalGraphSegmentFacts, orthogonal_graph_route_proposal_segment_comparison,
    orthogonal_graph_route_proposal_segment_facts,
};
pub use fixtures::{
    RouteStrategyFixtureBoardSpec, build_route_strategy_fixture_board_write,
    build_route_strategy_fixture_net_class_clear,
};
pub use generation::build_route_proposal_actions;
pub use revalidation::{
    RouteProposalArtifactRevalidation, analyze_route_proposal_artifact_revalidation,
    rebuild_route_proposal_artifact_live_actions,
};
pub use selection::{
    RouteProposalSelectionCandidate, RouteProposalSelectionOutcome, route_proposal_selection_specs,
    run_route_proposal_selection,
};

use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::board::RoutePathCandidateAuthoredCopperGraphPolicy;
use crate::ir::geometry::Point;

/// One deterministic route-proposal action. Moved verbatim from the CLI's
/// `NativeProjectRouteProposalActionView` (the CLI re-exports this type under
/// that name); the serde shape is persisted in exported route-proposal
/// artifacts and must not drift.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct RouteProposalAction {
    pub action_id: String,
    pub proposal_action: String,
    pub reason: String,
    pub contract: String,
    pub net_uuid: Uuid,
    pub net_name: String,
    pub from_anchor_pad_uuid: Uuid,
    pub to_anchor_pad_uuid: Uuid,
    pub layer: i32,
    pub width_nm: i64,
    pub from: Point,
    pub to: Point,
    pub reused_via_uuid: Option<Uuid>,
    #[serde(default)]
    pub reused_via_uuids: Vec<Uuid>,
    #[serde(default)]
    pub reused_object_kind: Option<String>,
    #[serde(default)]
    pub reused_object_uuid: Option<Uuid>,
    #[serde(default)]
    pub reused_object_from_layer: Option<i32>,
    #[serde(default)]
    pub reused_object_to_layer: Option<i32>,
    #[serde(default)]
    pub selected_path_bend_count: usize,
    pub selected_path_point_count: usize,
    pub selected_path_segment_index: usize,
    pub selected_path_segment_count: usize,
    #[serde(default)]
    pub selected_path_layer_segment_index: Option<usize>,
    #[serde(default)]
    pub selected_path_layer_segment_count: Option<usize>,
    #[serde(default)]
    pub selected_path_layer_segment_bend_count: Option<usize>,
    #[serde(default)]
    pub selected_path_layer_segment_point_count: Option<usize>,
}

/// One route-proposal candidate strategy. The authored-copper-graph family
/// carries its bounded policy, so a candidate value is a complete build spec.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum RouteProposalCandidate {
    RoutePathCandidate,
    RoutePathCandidateVia,
    RoutePathCandidateTwoVia,
    RoutePathCandidateThreeVia,
    RoutePathCandidateFourVia,
    RoutePathCandidateFiveVia,
    RoutePathCandidateSixVia,
    RoutePathCandidateAuthoredViaChain,
    RoutePathCandidateOrthogonalDogleg,
    RoutePathCandidateOrthogonalTwoBend,
    RoutePathCandidateOrthogonalGraph,
    RoutePathCandidateOrthogonalGraphVia,
    RoutePathCandidateOrthogonalGraphTwoVia,
    RoutePathCandidateOrthogonalGraphThreeVia,
    RoutePathCandidateOrthogonalGraphFourVia,
    RoutePathCandidateOrthogonalGraphFiveVia,
    RoutePathCandidateOrthogonalGraphSixVia,
    AuthoredCopperPlusOneGap,
    AuthoredCopperGraph(RoutePathCandidateAuthoredCopperGraphPolicy),
    /// Legacy dedicated zone-aware authored-copper-graph contract; only
    /// reachable from persisted artifacts (revalidation/apply rebuilds), not
    /// from the CLI candidate surface.
    RoutePathCandidateAuthoredCopperGraphZoneAware,
    /// Legacy dedicated contract; artifact rebuilds only (see above).
    RoutePathCandidateAuthoredCopperGraphZoneObstacleAware,
    /// Legacy dedicated contract; artifact rebuilds only (see above).
    RoutePathCandidateAuthoredCopperGraphZoneObstacleAwareTopologyAware,
    /// Legacy dedicated contract; artifact rebuilds only (see above).
    RoutePathCandidateAuthoredCopperGraphZoneObstacleAwareTopologyAwareLayerBalanceAware,
    /// Legacy dedicated contract; artifact rebuilds only (see above).
    RoutePathCandidateAuthoredCopperGraphObstacleAware,
}

/// Accepted deterministic selection profiles (M6 objective/profile table).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum RouteProposalProfile {
    Default,
    AuthoredCopperPriority,
}

/// Canonical CLI-visible candidate name (historical `--candidate` values).
pub fn candidate_name(candidate: RouteProposalCandidate) -> &'static str {
    match candidate {
        RouteProposalCandidate::RoutePathCandidate => "route-path-candidate",
        RouteProposalCandidate::RoutePathCandidateVia => "route-path-candidate-via",
        RouteProposalCandidate::RoutePathCandidateTwoVia => "route-path-candidate-two-via",
        RouteProposalCandidate::RoutePathCandidateThreeVia => "route-path-candidate-three-via",
        RouteProposalCandidate::RoutePathCandidateFourVia => "route-path-candidate-four-via",
        RouteProposalCandidate::RoutePathCandidateFiveVia => "route-path-candidate-five-via",
        RouteProposalCandidate::RoutePathCandidateSixVia => "route-path-candidate-six-via",
        RouteProposalCandidate::RoutePathCandidateAuthoredViaChain => {
            "route-path-candidate-authored-via-chain"
        }
        RouteProposalCandidate::RoutePathCandidateOrthogonalDogleg => {
            "route-path-candidate-orthogonal-dogleg"
        }
        RouteProposalCandidate::RoutePathCandidateOrthogonalTwoBend => {
            "route-path-candidate-orthogonal-two-bend"
        }
        RouteProposalCandidate::RoutePathCandidateOrthogonalGraph => {
            "route-path-candidate-orthogonal-graph"
        }
        RouteProposalCandidate::RoutePathCandidateOrthogonalGraphVia => {
            "route-path-candidate-orthogonal-graph-via"
        }
        RouteProposalCandidate::RoutePathCandidateOrthogonalGraphTwoVia => {
            "route-path-candidate-orthogonal-graph-two-via"
        }
        RouteProposalCandidate::RoutePathCandidateOrthogonalGraphThreeVia => {
            "route-path-candidate-orthogonal-graph-three-via"
        }
        RouteProposalCandidate::RoutePathCandidateOrthogonalGraphFourVia => {
            "route-path-candidate-orthogonal-graph-four-via"
        }
        RouteProposalCandidate::RoutePathCandidateOrthogonalGraphFiveVia => {
            "route-path-candidate-orthogonal-graph-five-via"
        }
        RouteProposalCandidate::RoutePathCandidateOrthogonalGraphSixVia => {
            "route-path-candidate-orthogonal-graph-six-via"
        }
        RouteProposalCandidate::AuthoredCopperPlusOneGap => "authored-copper-plus-one-gap",
        RouteProposalCandidate::AuthoredCopperGraph(_) => "authored-copper-graph",
        RouteProposalCandidate::RoutePathCandidateAuthoredCopperGraphZoneAware => {
            "route-path-candidate-authored-copper-graph-zone-aware"
        }
        RouteProposalCandidate::RoutePathCandidateAuthoredCopperGraphZoneObstacleAware => {
            "route-path-candidate-authored-copper-graph-zone-obstacle-aware"
        }
        RouteProposalCandidate::RoutePathCandidateAuthoredCopperGraphZoneObstacleAwareTopologyAware => {
            "route-path-candidate-authored-copper-graph-zone-obstacle-aware-topology-aware"
        }
        RouteProposalCandidate::RoutePathCandidateAuthoredCopperGraphZoneObstacleAwareTopologyAwareLayerBalanceAware => {
            "route-path-candidate-authored-copper-graph-zone-obstacle-aware-topology-aware-layer-balance-aware"
        }
        RouteProposalCandidate::RoutePathCandidateAuthoredCopperGraphObstacleAware => {
            "route-path-candidate-authored-copper-graph-obstacle-aware"
        }
    }
}

/// The bounded policy carried by an authored-copper-graph candidate.
pub fn candidate_policy(
    candidate: RouteProposalCandidate,
) -> Option<RoutePathCandidateAuthoredCopperGraphPolicy> {
    match candidate {
        RouteProposalCandidate::AuthoredCopperGraph(policy) => Some(policy),
        _ => None,
    }
}

/// Canonical CLI-visible policy name for an authored-copper-graph candidate.
pub fn candidate_policy_name(candidate: RouteProposalCandidate) -> Option<String> {
    candidate_policy(candidate).map(policy_name)
}

/// Canonical policy name (historical `--policy` reporting values).
pub fn policy_name(policy: RoutePathCandidateAuthoredCopperGraphPolicy) -> String {
    match policy {
        RoutePathCandidateAuthoredCopperGraphPolicy::Plain => "plain".to_string(),
        RoutePathCandidateAuthoredCopperGraphPolicy::ZoneAware => "zone_aware".to_string(),
        RoutePathCandidateAuthoredCopperGraphPolicy::ObstacleAware => "obstacle_aware".to_string(),
        RoutePathCandidateAuthoredCopperGraphPolicy::ZoneObstacleAware => {
            "zone_obstacle_aware".to_string()
        }
        RoutePathCandidateAuthoredCopperGraphPolicy::ZoneObstacleTopologyAware => {
            "zone_obstacle_topology_aware".to_string()
        }
        RoutePathCandidateAuthoredCopperGraphPolicy::ZoneObstacleTopologyLayerBalanceAware => {
            "zone_obstacle_topology_layer_balance_aware".to_string()
        }
    }
}

/// Selection-spec name: candidate name, plus `:{policy}` for the
/// authored-copper-graph family (historical selection-rule rendering).
pub fn candidate_spec_name(candidate: &RouteProposalCandidate) -> String {
    if let Some(policy) = candidate_policy(*candidate) {
        format!("{}:{}", candidate_name(*candidate), policy_name(policy))
    } else {
        candidate_name(*candidate).to_string()
    }
}

/// Canonical profile name (historical `--profile` values).
pub fn profile_name(profile: RouteProposalProfile) -> &'static str {
    match profile {
        RouteProposalProfile::Default => "default",
        RouteProposalProfile::AuthoredCopperPriority => "authored-copper-priority",
    }
}

/// The accepted route-strategy comparison order (deterministic).
pub fn accepted_route_strategy_profiles() -> [RouteProposalProfile; 2] {
    [
        RouteProposalProfile::Default,
        RouteProposalProfile::AuthoredCopperPriority,
    ]
}

/// One-line distinction between the accepted profiles (report text).
pub fn profile_distinction(profile: RouteProposalProfile) -> &'static str {
    match profile {
        RouteProposalProfile::Default => {
            "baseline profile: preserves the accepted selector family order exactly"
        }
        RouteProposalProfile::AuthoredCopperPriority => {
            "reuse-priority profile: prepends the accepted authored-copper-graph policy family ahead of the unchanged default order"
        }
    }
}

/// Reject proposal actions that the apply path does not support.
pub fn validate_route_proposal_actions(actions: &[RouteProposalAction]) -> Result<(), String> {
    for action in actions {
        if action.proposal_action != "draw_track"
            && action.proposal_action != "reuse_existing_copper_step"
        {
            return Err(format!(
                "route proposal apply is not supported for {} reason={}",
                action.proposal_action, action.reason
            ));
        }
    }
    Ok(())
}

pub(crate) const ROUTE_PROPOSAL_REASON_AUTHORED_COPPER_PLUS_ONE_GAP: &str =
    "authored_copper_plus_one_gap";
pub(crate) const ROUTE_PROPOSAL_REASON_ROUTE_PATH_CANDIDATE: &str = "route_path_candidate";
pub(crate) const ROUTE_PROPOSAL_REASON_ROUTE_PATH_CANDIDATE_VIA: &str = "route_path_candidate_via";
pub(crate) const ROUTE_PROPOSAL_REASON_ROUTE_PATH_CANDIDATE_TWO_VIA: &str =
    "route_path_candidate_two_via";
pub(crate) const ROUTE_PROPOSAL_REASON_ROUTE_PATH_CANDIDATE_THREE_VIA: &str =
    "route_path_candidate_three_via";
pub(crate) const ROUTE_PROPOSAL_REASON_ROUTE_PATH_CANDIDATE_FOUR_VIA: &str =
    "route_path_candidate_four_via";
pub(crate) const ROUTE_PROPOSAL_REASON_ROUTE_PATH_CANDIDATE_FIVE_VIA: &str =
    "route_path_candidate_five_via";
pub(crate) const ROUTE_PROPOSAL_REASON_ROUTE_PATH_CANDIDATE_SIX_VIA: &str =
    "route_path_candidate_six_via";
pub(crate) const ROUTE_PROPOSAL_REASON_ROUTE_PATH_CANDIDATE_AUTHORED_VIA_CHAIN: &str =
    "route_path_candidate_authored_via_chain";
pub(crate) const ROUTE_PROPOSAL_REASON_ROUTE_PATH_CANDIDATE_ORTHOGONAL_DOGLEG: &str =
    "route_path_candidate_orthogonal_dogleg";
pub(crate) const ROUTE_PROPOSAL_REASON_ROUTE_PATH_CANDIDATE_ORTHOGONAL_TWO_BEND: &str =
    "route_path_candidate_orthogonal_two_bend";
pub(crate) const ROUTE_PROPOSAL_REASON_ROUTE_PATH_CANDIDATE_ORTHOGONAL_GRAPH: &str =
    "route_path_candidate_orthogonal_graph";
pub(crate) const ROUTE_PROPOSAL_REASON_ROUTE_PATH_CANDIDATE_ORTHOGONAL_GRAPH_VIA: &str =
    "route_path_candidate_orthogonal_graph_via";
pub(crate) const ROUTE_PROPOSAL_REASON_ROUTE_PATH_CANDIDATE_ORTHOGONAL_GRAPH_TWO_VIA: &str =
    "route_path_candidate_orthogonal_graph_two_via";
pub(crate) const ROUTE_PROPOSAL_REASON_ROUTE_PATH_CANDIDATE_ORTHOGONAL_GRAPH_THREE_VIA: &str =
    "route_path_candidate_orthogonal_graph_three_via";
pub(crate) const ROUTE_PROPOSAL_REASON_ROUTE_PATH_CANDIDATE_ORTHOGONAL_GRAPH_FOUR_VIA: &str =
    "route_path_candidate_orthogonal_graph_four_via";
pub(crate) const ROUTE_PROPOSAL_REASON_ROUTE_PATH_CANDIDATE_ORTHOGONAL_GRAPH_FIVE_VIA: &str =
    "route_path_candidate_orthogonal_graph_five_via";
pub(crate) const ROUTE_PROPOSAL_REASON_ROUTE_PATH_CANDIDATE_ORTHOGONAL_GRAPH_SIX_VIA: &str =
    "route_path_candidate_orthogonal_graph_six_via";
pub(crate) const ROUTE_PROPOSAL_REASON_ROUTE_PATH_CANDIDATE_AUTHORED_COPPER_GRAPH_ZONE_AWARE: &str =
    "route_path_candidate_authored_copper_graph_zone_aware";
pub(crate) const ROUTE_PROPOSAL_REASON_ROUTE_PATH_CANDIDATE_AUTHORED_COPPER_GRAPH_ZONE_OBSTACLE_AWARE: &str =
    "route_path_candidate_authored_copper_graph_zone_obstacle_aware";
pub(crate) const ROUTE_PROPOSAL_REASON_ROUTE_PATH_CANDIDATE_AUTHORED_COPPER_GRAPH_ZONE_OBSTACLE_AWARE_TOPOLOGY_AWARE: &str =
    "route_path_candidate_authored_copper_graph_zone_obstacle_aware_topology_aware";
pub(crate) const ROUTE_PROPOSAL_REASON_ROUTE_PATH_CANDIDATE_AUTHORED_COPPER_GRAPH_ZONE_OBSTACLE_AWARE_TOPOLOGY_AWARE_LAYER_BALANCE_AWARE: &str =
    "route_path_candidate_authored_copper_graph_zone_obstacle_aware_topology_aware_layer_balance_aware";
pub(crate) const ROUTE_PROPOSAL_REASON_ROUTE_PATH_CANDIDATE_AUTHORED_COPPER_GRAPH_OBSTACLE_AWARE:
    &str = "route_path_candidate_authored_copper_graph_obstacle_aware";
pub(crate) const ROUTE_PROPOSAL_REASON_ROUTE_PATH_CANDIDATE_AUTHORED_COPPER_GRAPH_POLICY_PLAIN:
    &str = "route_path_candidate_authored_copper_graph_policy_plain";
pub(crate) const ROUTE_PROPOSAL_REASON_ROUTE_PATH_CANDIDATE_AUTHORED_COPPER_GRAPH_POLICY_ZONE_AWARE: &str =
    "route_path_candidate_authored_copper_graph_policy_zone_aware";
pub(crate) const ROUTE_PROPOSAL_REASON_ROUTE_PATH_CANDIDATE_AUTHORED_COPPER_GRAPH_POLICY_OBSTACLE_AWARE: &str =
    "route_path_candidate_authored_copper_graph_policy_obstacle_aware";
pub(crate) const ROUTE_PROPOSAL_REASON_ROUTE_PATH_CANDIDATE_AUTHORED_COPPER_GRAPH_POLICY_ZONE_OBSTACLE_AWARE: &str =
    "route_path_candidate_authored_copper_graph_policy_zone_obstacle_aware";
pub(crate) const ROUTE_PROPOSAL_REASON_ROUTE_PATH_CANDIDATE_AUTHORED_COPPER_GRAPH_POLICY_ZONE_OBSTACLE_TOPOLOGY_AWARE: &str =
    "route_path_candidate_authored_copper_graph_policy_zone_obstacle_topology_aware";
pub(crate) const ROUTE_PROPOSAL_REASON_ROUTE_PATH_CANDIDATE_AUTHORED_COPPER_GRAPH_POLICY_ZONE_OBSTACLE_TOPOLOGY_LAYER_BALANCE_AWARE: &str =
    "route_path_candidate_authored_copper_graph_policy_zone_obstacle_topology_layer_balance_aware";

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn candidate_names_match_historical_cli_values() {
        assert_eq!(
            candidate_name(RouteProposalCandidate::RoutePathCandidate),
            "route-path-candidate"
        );
        assert_eq!(
            candidate_name(RouteProposalCandidate::AuthoredCopperPlusOneGap),
            "authored-copper-plus-one-gap"
        );
        assert_eq!(
            candidate_name(RouteProposalCandidate::AuthoredCopperGraph(
                RoutePathCandidateAuthoredCopperGraphPolicy::Plain
            )),
            "authored-copper-graph"
        );
        assert_eq!(
            candidate_spec_name(&RouteProposalCandidate::AuthoredCopperGraph(
                RoutePathCandidateAuthoredCopperGraphPolicy::ZoneObstacleTopologyLayerBalanceAware
            )),
            "authored-copper-graph:zone_obstacle_topology_layer_balance_aware"
        );
    }

    #[test]
    fn validate_rejects_unsupported_proposal_actions() {
        let action = RouteProposalAction {
            action_id: "a".to_string(),
            proposal_action: "place_via".to_string(),
            reason: "unit_test".to_string(),
            contract: "c".to_string(),
            net_uuid: Uuid::nil(),
            net_name: "SIG".to_string(),
            from_anchor_pad_uuid: Uuid::nil(),
            to_anchor_pad_uuid: Uuid::nil(),
            layer: 1,
            width_nm: 1,
            from: Point { x: 0, y: 0 },
            to: Point { x: 1, y: 1 },
            reused_via_uuid: None,
            reused_via_uuids: Vec::new(),
            reused_object_kind: None,
            reused_object_uuid: None,
            reused_object_from_layer: None,
            reused_object_to_layer: None,
            selected_path_bend_count: 0,
            selected_path_point_count: 2,
            selected_path_segment_index: 0,
            selected_path_segment_count: 1,
            selected_path_layer_segment_index: None,
            selected_path_layer_segment_count: None,
            selected_path_layer_segment_bend_count: None,
            selected_path_layer_segment_point_count: None,
        };
        assert_eq!(
            validate_route_proposal_actions(std::slice::from_ref(&action)),
            Err("route proposal apply is not supported for place_via reason=unit_test".to_string())
        );
        let mut supported = action;
        supported.proposal_action = "draw_track".to_string();
        assert_eq!(validate_route_proposal_actions(&[supported]), Ok(()));
    }
}
