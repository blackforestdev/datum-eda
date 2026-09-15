use crate::board::*;

use super::route_path_candidate_authored_copper_graph_policy::{
    obstacle_board, plain_board, zone_board,
};

fn path_ids(report: &RoutePathCandidateAuthoredCopperGraphPolicyExplainReport) -> Vec<uuid::Uuid> {
    report
        .selected_path
        .as_ref()
        .map(|path| path.steps.iter().map(|step| step.object_uuid).collect())
        .unwrap_or_default()
}

#[test]
fn authored_copper_graph_policy_explain_preserves_plain_behavior() {
    let (board, net_uuid, from_anchor_pad_uuid, to_anchor_pad_uuid, _) = plain_board();
    let policy = board
        .route_path_candidate_authored_copper_graph_explain_by_policy(
            net_uuid,
            from_anchor_pad_uuid,
            to_anchor_pad_uuid,
            RoutePathCandidateAuthoredCopperGraphPolicy::Plain,
        )
        .expect("policy explain should succeed");
    let direct = board
        .route_path_candidate_authored_copper_graph_explain(
            net_uuid,
            from_anchor_pad_uuid,
            to_anchor_pad_uuid,
        )
        .expect("direct explain should succeed");

    assert_eq!(policy.status, direct.status);
    assert_eq!(policy.selection_rule, direct.selection_rule);
    assert_eq!(
        policy.summary.candidate_track_count,
        direct.summary.candidate_track_count
    );
    assert_eq!(
        policy.summary.candidate_via_count,
        direct.summary.candidate_via_count
    );
    assert_eq!(
        path_ids(&policy),
        direct
            .selected_path
            .unwrap()
            .steps
            .iter()
            .map(|s| s.object_uuid)
            .collect::<Vec<_>>()
    );
}

#[test]
fn authored_copper_graph_policy_explain_preserves_zone_aware_behavior() {
    let (board, net_uuid, from_anchor_pad_uuid, to_anchor_pad_uuid, _) = zone_board();
    let policy = board
        .route_path_candidate_authored_copper_graph_explain_by_policy(
            net_uuid,
            from_anchor_pad_uuid,
            to_anchor_pad_uuid,
            RoutePathCandidateAuthoredCopperGraphPolicy::ZoneAware,
        )
        .expect("policy explain should succeed");
    let direct = board
        .route_path_candidate_authored_copper_graph_zone_aware_explain(
            net_uuid,
            from_anchor_pad_uuid,
            to_anchor_pad_uuid,
        )
        .expect("direct explain should succeed");

    assert_eq!(policy.status, direct.status);
    assert_eq!(policy.selection_rule, direct.selection_rule);
    assert_eq!(
        policy.summary.candidate_zone_count,
        direct.summary.candidate_zone_count
    );
    assert_eq!(
        path_ids(&policy),
        direct
            .selected_path
            .unwrap()
            .steps
            .iter()
            .map(|s| s.object_uuid)
            .collect::<Vec<_>>()
    );
}

#[test]
fn authored_copper_graph_policy_explain_preserves_obstacle_aware_behavior() {
    let (board, net_uuid, from_anchor_pad_uuid, to_anchor_pad_uuid, _, _, _) = obstacle_board();
    let policy = board
        .route_path_candidate_authored_copper_graph_explain_by_policy(
            net_uuid,
            from_anchor_pad_uuid,
            to_anchor_pad_uuid,
            RoutePathCandidateAuthoredCopperGraphPolicy::ObstacleAware,
        )
        .expect("policy explain should succeed");
    let direct = board
        .route_path_candidate_authored_copper_graph_obstacle_aware_explain(
            net_uuid,
            from_anchor_pad_uuid,
            to_anchor_pad_uuid,
        )
        .expect("direct explain should succeed");

    assert_eq!(policy.status, direct.status);
    assert_eq!(policy.selection_rule, direct.selection_rule);
    assert_eq!(
        policy.summary.blocked_track_count,
        direct.summary.blocked_track_count
    );
    assert_eq!(
        policy.summary.blocked_via_count,
        direct.summary.blocked_via_count
    );
    assert_eq!(
        path_ids(&policy),
        direct
            .selected_path
            .unwrap()
            .steps
            .iter()
            .map(|s| s.object_uuid)
            .collect::<Vec<_>>()
    );
}

#[test]
fn authored_copper_graph_policy_explain_preserves_zone_obstacle_aware_behavior() {
    let (board, net_uuid, from_anchor_pad_uuid, to_anchor_pad_uuid, _, _, _) = obstacle_board();
    let policy = board
        .route_path_candidate_authored_copper_graph_explain_by_policy(
            net_uuid,
            from_anchor_pad_uuid,
            to_anchor_pad_uuid,
            RoutePathCandidateAuthoredCopperGraphPolicy::ZoneObstacleAware,
        )
        .expect("policy explain should succeed");
    let direct = board
        .route_path_candidate_authored_copper_graph_zone_obstacle_aware_explain(
            net_uuid,
            from_anchor_pad_uuid,
            to_anchor_pad_uuid,
        )
        .expect("direct explain should succeed");

    assert_eq!(policy.status, direct.status);
    assert_eq!(policy.selection_rule, direct.selection_rule);
    assert_eq!(
        policy.summary.blocked_zone_connection_count,
        direct.summary.blocked_zone_connection_count
    );
    assert_eq!(
        path_ids(&policy),
        direct
            .selected_path
            .unwrap()
            .steps
            .iter()
            .map(|s| s.object_uuid)
            .collect::<Vec<_>>()
    );
}

#[test]
fn authored_copper_graph_policy_explain_preserves_topology_aware_behavior() {
    let (board, net_uuid, from_anchor_pad_uuid, to_anchor_pad_uuid, _, _, _) = obstacle_board();
    let policy = board
        .route_path_candidate_authored_copper_graph_explain_by_policy(
            net_uuid,
            from_anchor_pad_uuid,
            to_anchor_pad_uuid,
            RoutePathCandidateAuthoredCopperGraphPolicy::ZoneObstacleTopologyAware,
        )
        .expect("policy explain should succeed");
    let direct = board
        .route_path_candidate_authored_copper_graph_zone_obstacle_aware_topology_aware_explain(
            net_uuid,
            from_anchor_pad_uuid,
            to_anchor_pad_uuid,
        )
        .expect("direct explain should succeed");

    assert_eq!(policy.status, direct.status);
    assert_eq!(policy.selection_rule, direct.selection_rule);
    assert_eq!(
        policy.summary.topology_transition_count,
        direct.summary.topology_transition_count
    );
    assert_eq!(
        path_ids(&policy),
        direct
            .selected_path
            .unwrap()
            .steps
            .iter()
            .map(|s| s.object_uuid)
            .collect::<Vec<_>>()
    );
}

#[test]
fn authored_copper_graph_policy_explain_preserves_layer_balance_aware_behavior() {
    let (board, net_uuid, from_anchor_pad_uuid, to_anchor_pad_uuid, _, _, _) = obstacle_board();
    let policy = board
        .route_path_candidate_authored_copper_graph_explain_by_policy(
            net_uuid,
            from_anchor_pad_uuid,
            to_anchor_pad_uuid,
            RoutePathCandidateAuthoredCopperGraphPolicy::ZoneObstacleTopologyLayerBalanceAware,
        )
        .expect("policy explain should succeed");
    let direct = board
        .route_path_candidate_authored_copper_graph_zone_obstacle_aware_topology_aware_layer_balance_aware_explain(
            net_uuid,
            from_anchor_pad_uuid,
            to_anchor_pad_uuid,
        )
        .expect("direct explain should succeed");

    assert_eq!(policy.status, direct.status);
    assert_eq!(policy.selection_rule, direct.selection_rule);
    assert_eq!(
        policy.summary.layer_balance_score,
        direct.summary.layer_balance_score
    );
    assert_eq!(
        path_ids(&policy),
        direct
            .selected_path
            .unwrap()
            .steps
            .iter()
            .map(|s| s.object_uuid)
            .collect::<Vec<_>>()
    );
}
