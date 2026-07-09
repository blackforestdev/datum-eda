//! Route-proposal artifact revalidation: rebuild the artifact's contract
//! against the live board and classify drift.
//!
//! Moved from `crates/cli/src/command_project_route_proposal.rs` (family F).
//! `live_board` is a `Result` so a failed project load is analyzed as a live
//! rebuild failure (historically the load happened inside each builder), and
//! the unsupported-contract error takes precedence over the load error —
//! exactly the historical CLI ordering.

use crate::board::Board;

use super::drift::{
    OrthogonalGraphArtifactDriftKind, orthogonal_graph_route_proposal_artifact_drift_kind,
    render_orthogonal_graph_route_proposal_drift_message,
};
use super::generation::{
    build_route_proposal_actions, route_path_candidate_authored_copper_graph_policy_from_reason,
};
use super::{
    ROUTE_PROPOSAL_REASON_AUTHORED_COPPER_PLUS_ONE_GAP, ROUTE_PROPOSAL_REASON_ROUTE_PATH_CANDIDATE,
    ROUTE_PROPOSAL_REASON_ROUTE_PATH_CANDIDATE_AUTHORED_COPPER_GRAPH_OBSTACLE_AWARE,
    ROUTE_PROPOSAL_REASON_ROUTE_PATH_CANDIDATE_AUTHORED_COPPER_GRAPH_ZONE_AWARE,
    ROUTE_PROPOSAL_REASON_ROUTE_PATH_CANDIDATE_AUTHORED_COPPER_GRAPH_ZONE_OBSTACLE_AWARE,
    ROUTE_PROPOSAL_REASON_ROUTE_PATH_CANDIDATE_AUTHORED_COPPER_GRAPH_ZONE_OBSTACLE_AWARE_TOPOLOGY_AWARE,
    ROUTE_PROPOSAL_REASON_ROUTE_PATH_CANDIDATE_AUTHORED_COPPER_GRAPH_ZONE_OBSTACLE_AWARE_TOPOLOGY_AWARE_LAYER_BALANCE_AWARE,
    ROUTE_PROPOSAL_REASON_ROUTE_PATH_CANDIDATE_AUTHORED_VIA_CHAIN,
    ROUTE_PROPOSAL_REASON_ROUTE_PATH_CANDIDATE_FIVE_VIA,
    ROUTE_PROPOSAL_REASON_ROUTE_PATH_CANDIDATE_FOUR_VIA,
    ROUTE_PROPOSAL_REASON_ROUTE_PATH_CANDIDATE_ORTHOGONAL_DOGLEG,
    ROUTE_PROPOSAL_REASON_ROUTE_PATH_CANDIDATE_ORTHOGONAL_GRAPH,
    ROUTE_PROPOSAL_REASON_ROUTE_PATH_CANDIDATE_ORTHOGONAL_GRAPH_FIVE_VIA,
    ROUTE_PROPOSAL_REASON_ROUTE_PATH_CANDIDATE_ORTHOGONAL_GRAPH_FOUR_VIA,
    ROUTE_PROPOSAL_REASON_ROUTE_PATH_CANDIDATE_ORTHOGONAL_GRAPH_SIX_VIA,
    ROUTE_PROPOSAL_REASON_ROUTE_PATH_CANDIDATE_ORTHOGONAL_GRAPH_THREE_VIA,
    ROUTE_PROPOSAL_REASON_ROUTE_PATH_CANDIDATE_ORTHOGONAL_GRAPH_TWO_VIA,
    ROUTE_PROPOSAL_REASON_ROUTE_PATH_CANDIDATE_ORTHOGONAL_GRAPH_VIA,
    ROUTE_PROPOSAL_REASON_ROUTE_PATH_CANDIDATE_ORTHOGONAL_TWO_BEND,
    ROUTE_PROPOSAL_REASON_ROUTE_PATH_CANDIDATE_SIX_VIA,
    ROUTE_PROPOSAL_REASON_ROUTE_PATH_CANDIDATE_THREE_VIA,
    ROUTE_PROPOSAL_REASON_ROUTE_PATH_CANDIDATE_TWO_VIA,
    ROUTE_PROPOSAL_REASON_ROUTE_PATH_CANDIDATE_VIA, RouteProposalAction, RouteProposalCandidate,
};

/// Artifact-vs-live revalidation facts for one route-proposal artifact.
pub struct RouteProposalArtifactRevalidation {
    pub live_actions: Result<Vec<RouteProposalAction>, String>,
    pub matches_live: bool,
    pub drift_kind: Option<OrthogonalGraphArtifactDriftKind>,
    pub drift_message: Option<String>,
}

/// Rebuild the artifact's contract against the live board.
pub fn rebuild_route_proposal_artifact_live_actions(
    live_board: &Result<Board, String>,
    contract: &str,
    first_action: &RouteProposalAction,
) -> Result<Vec<RouteProposalAction>, String> {
    let candidate =
        route_proposal_artifact_candidate(contract, &first_action.reason).ok_or_else(|| {
            format!(
                "route proposal artifact apply is not supported for contract={} reason={}",
                contract, first_action.reason
            )
        })?;
    let board = live_board.as_ref().map_err(Clone::clone)?;
    build_route_proposal_actions(
        board,
        first_action.net_uuid,
        first_action.from_anchor_pad_uuid,
        first_action.to_anchor_pad_uuid,
        candidate,
    )
}

/// Rebuild the live actions and classify orthogonal-graph drift.
pub fn analyze_route_proposal_artifact_revalidation(
    live_board: &Result<Board, String>,
    contract: &str,
    first_action: &RouteProposalAction,
    artifact_actions: &[RouteProposalAction],
) -> RouteProposalArtifactRevalidation {
    let live_actions =
        rebuild_route_proposal_artifact_live_actions(live_board, contract, first_action);
    let drift_kind = orthogonal_graph_route_proposal_artifact_drift_kind(
        contract,
        artifact_actions,
        &live_actions,
    );
    let drift_message = drift_kind.map(|kind| {
        let artifact_first = artifact_actions
            .first()
            .expect("route proposal artifact revalidation requires one action");
        render_orthogonal_graph_route_proposal_drift_message(
            kind,
            artifact_first,
            live_actions
                .as_ref()
                .ok()
                .and_then(|actions| actions.first()),
            live_actions.as_ref().err(),
        )
    });
    let matches_live = match &live_actions {
        Ok(actions) => actions == artifact_actions,
        Err(_) => false,
    };

    RouteProposalArtifactRevalidation {
        live_actions,
        matches_live,
        drift_kind,
        drift_message,
    }
}

/// The rebuild candidate for a persisted artifact's contract + reason pair.
/// `None` marks a contract/reason pair the apply path does not support.
fn route_proposal_artifact_candidate(
    contract: &str,
    reason: &str,
) -> Option<RouteProposalCandidate> {
    if contract == "m5_route_path_candidate_authored_copper_graph_policy_v1" {
        return route_path_candidate_authored_copper_graph_policy_from_reason(reason)
            .map(RouteProposalCandidate::AuthoredCopperGraph);
    }

    match (contract, reason) {
        ("m5_route_path_candidate_v2", ROUTE_PROPOSAL_REASON_ROUTE_PATH_CANDIDATE) => {
            Some(RouteProposalCandidate::RoutePathCandidate)
        }
        ("m5_route_path_candidate_via_v1", ROUTE_PROPOSAL_REASON_ROUTE_PATH_CANDIDATE_VIA) => {
            Some(RouteProposalCandidate::RoutePathCandidateVia)
        }
        (
            "m5_route_path_candidate_two_via_v1",
            ROUTE_PROPOSAL_REASON_ROUTE_PATH_CANDIDATE_TWO_VIA,
        ) => Some(RouteProposalCandidate::RoutePathCandidateTwoVia),
        (
            "m5_route_path_candidate_three_via_v1",
            ROUTE_PROPOSAL_REASON_ROUTE_PATH_CANDIDATE_THREE_VIA,
        ) => Some(RouteProposalCandidate::RoutePathCandidateThreeVia),
        (
            "m5_route_path_candidate_four_via_v1",
            ROUTE_PROPOSAL_REASON_ROUTE_PATH_CANDIDATE_FOUR_VIA,
        ) => Some(RouteProposalCandidate::RoutePathCandidateFourVia),
        (
            "m5_route_path_candidate_five_via_v1",
            ROUTE_PROPOSAL_REASON_ROUTE_PATH_CANDIDATE_FIVE_VIA,
        ) => Some(RouteProposalCandidate::RoutePathCandidateFiveVia),
        (
            "m5_route_path_candidate_six_via_v1",
            ROUTE_PROPOSAL_REASON_ROUTE_PATH_CANDIDATE_SIX_VIA,
        ) => Some(RouteProposalCandidate::RoutePathCandidateSixVia),
        (
            "m5_route_path_candidate_authored_via_chain_v1",
            ROUTE_PROPOSAL_REASON_ROUTE_PATH_CANDIDATE_AUTHORED_VIA_CHAIN,
        ) => Some(RouteProposalCandidate::RoutePathCandidateAuthoredViaChain),
        (
            "m5_route_path_candidate_orthogonal_dogleg_v1",
            ROUTE_PROPOSAL_REASON_ROUTE_PATH_CANDIDATE_ORTHOGONAL_DOGLEG,
        ) => Some(RouteProposalCandidate::RoutePathCandidateOrthogonalDogleg),
        (
            "m5_route_path_candidate_orthogonal_two_bend_v1",
            ROUTE_PROPOSAL_REASON_ROUTE_PATH_CANDIDATE_ORTHOGONAL_TWO_BEND,
        ) => Some(RouteProposalCandidate::RoutePathCandidateOrthogonalTwoBend),
        (
            "m5_route_path_candidate_orthogonal_graph_v1",
            ROUTE_PROPOSAL_REASON_ROUTE_PATH_CANDIDATE_ORTHOGONAL_GRAPH,
        ) => Some(RouteProposalCandidate::RoutePathCandidateOrthogonalGraph),
        (
            "m5_route_path_candidate_orthogonal_graph_via_v1",
            ROUTE_PROPOSAL_REASON_ROUTE_PATH_CANDIDATE_ORTHOGONAL_GRAPH_VIA,
        ) => Some(RouteProposalCandidate::RoutePathCandidateOrthogonalGraphVia),
        (
            "m5_route_path_candidate_orthogonal_graph_two_via_v1",
            ROUTE_PROPOSAL_REASON_ROUTE_PATH_CANDIDATE_ORTHOGONAL_GRAPH_TWO_VIA,
        ) => Some(RouteProposalCandidate::RoutePathCandidateOrthogonalGraphTwoVia),
        (
            "m5_route_path_candidate_orthogonal_graph_three_via_v1",
            ROUTE_PROPOSAL_REASON_ROUTE_PATH_CANDIDATE_ORTHOGONAL_GRAPH_THREE_VIA,
        ) => Some(RouteProposalCandidate::RoutePathCandidateOrthogonalGraphThreeVia),
        (
            "m5_route_path_candidate_orthogonal_graph_four_via_v1",
            ROUTE_PROPOSAL_REASON_ROUTE_PATH_CANDIDATE_ORTHOGONAL_GRAPH_FOUR_VIA,
        ) => Some(RouteProposalCandidate::RoutePathCandidateOrthogonalGraphFourVia),
        (
            "m5_route_path_candidate_orthogonal_graph_five_via_v1",
            ROUTE_PROPOSAL_REASON_ROUTE_PATH_CANDIDATE_ORTHOGONAL_GRAPH_FIVE_VIA,
        ) => Some(RouteProposalCandidate::RoutePathCandidateOrthogonalGraphFiveVia),
        (
            "m5_route_path_candidate_orthogonal_graph_six_via_v1",
            ROUTE_PROPOSAL_REASON_ROUTE_PATH_CANDIDATE_ORTHOGONAL_GRAPH_SIX_VIA,
        ) => Some(RouteProposalCandidate::RoutePathCandidateOrthogonalGraphSixVia),
        (
            "m5_route_path_candidate_authored_copper_graph_zone_aware_v1",
            ROUTE_PROPOSAL_REASON_ROUTE_PATH_CANDIDATE_AUTHORED_COPPER_GRAPH_ZONE_AWARE,
        ) => Some(RouteProposalCandidate::RoutePathCandidateAuthoredCopperGraphZoneAware),
        (
            "m5_route_path_candidate_authored_copper_graph_zone_obstacle_aware_v1",
            ROUTE_PROPOSAL_REASON_ROUTE_PATH_CANDIDATE_AUTHORED_COPPER_GRAPH_ZONE_OBSTACLE_AWARE,
        ) => Some(RouteProposalCandidate::RoutePathCandidateAuthoredCopperGraphZoneObstacleAware),
        (
            "m5_route_path_candidate_authored_copper_graph_zone_obstacle_aware_topology_aware_v1",
            ROUTE_PROPOSAL_REASON_ROUTE_PATH_CANDIDATE_AUTHORED_COPPER_GRAPH_ZONE_OBSTACLE_AWARE_TOPOLOGY_AWARE,
        ) => Some(
            RouteProposalCandidate::RoutePathCandidateAuthoredCopperGraphZoneObstacleAwareTopologyAware,
        ),
        (
            "m5_route_path_candidate_authored_copper_graph_zone_obstacle_aware_topology_aware_layer_balance_aware_v1",
            ROUTE_PROPOSAL_REASON_ROUTE_PATH_CANDIDATE_AUTHORED_COPPER_GRAPH_ZONE_OBSTACLE_AWARE_TOPOLOGY_AWARE_LAYER_BALANCE_AWARE,
        ) => Some(
            RouteProposalCandidate::RoutePathCandidateAuthoredCopperGraphZoneObstacleAwareTopologyAwareLayerBalanceAware,
        ),
        (
            "m5_route_path_candidate_authored_copper_graph_obstacle_aware_v1",
            ROUTE_PROPOSAL_REASON_ROUTE_PATH_CANDIDATE_AUTHORED_COPPER_GRAPH_OBSTACLE_AWARE,
        ) => Some(RouteProposalCandidate::RoutePathCandidateAuthoredCopperGraphObstacleAware),
        (
            "m5_route_path_candidate_authored_copper_plus_one_gap_v1",
            ROUTE_PROPOSAL_REASON_AUTHORED_COPPER_PLUS_ONE_GAP,
        ) => Some(RouteProposalCandidate::AuthoredCopperPlusOneGap),
        _ => None,
    }
}
