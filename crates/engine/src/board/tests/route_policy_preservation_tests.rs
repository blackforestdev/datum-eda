use super::*;

#[test]
fn authored_copper_graph_policy_preserves_plain_behavior() {
    let (board, net_uuid, from_pad_uuid, to_pad_uuid, track_uuid) = plain_board();
    let report = board
        .route_path_candidate_authored_copper_graph_by_policy(
            net_uuid,
            from_pad_uuid,
            to_pad_uuid,
            RoutePathCandidateAuthoredCopperGraphPolicy::Plain,
        )
        .expect("policy query should succeed");
    let direct = board
        .route_path_candidate_authored_copper_graph(net_uuid, from_pad_uuid, to_pad_uuid)
        .expect("direct query should succeed");

    assert_eq!(report.status, direct.status);
    assert_eq!(report.selection_rule, direct.selection_rule);
    assert_eq!(
        report.summary.candidate_track_count,
        direct.summary.candidate_track_count
    );
    assert_eq!(path_ids(&report), vec![track_uuid]);
}

#[test]
fn authored_copper_graph_policy_preserves_zone_aware_behavior() {
    let (board, net_uuid, from_pad_uuid, to_pad_uuid, zone_uuid) = zone_board();
    let report = board
        .route_path_candidate_authored_copper_graph_by_policy(
            net_uuid,
            from_pad_uuid,
            to_pad_uuid,
            RoutePathCandidateAuthoredCopperGraphPolicy::ZoneAware,
        )
        .expect("policy query should succeed");
    let direct = board
        .route_path_candidate_authored_copper_graph_zone_aware(net_uuid, from_pad_uuid, to_pad_uuid)
        .expect("direct query should succeed");

    assert_eq!(report.status, direct.status);
    assert_eq!(report.selection_rule, direct.selection_rule);
    assert_eq!(
        report.summary.candidate_zone_count,
        direct.summary.candidate_zone_count
    );
    assert_eq!(path_ids(&report), vec![zone_uuid]);
}

#[test]
fn authored_copper_graph_policy_preserves_obstacle_aware_behavior() {
    let (board, net_uuid, from_pad_uuid, to_pad_uuid, track_a_uuid, via_uuid, track_b_uuid) =
        obstacle_board();
    let report = board
        .route_path_candidate_authored_copper_graph_by_policy(
            net_uuid,
            from_pad_uuid,
            to_pad_uuid,
            RoutePathCandidateAuthoredCopperGraphPolicy::ObstacleAware,
        )
        .expect("policy query should succeed");
    let direct = board
        .route_path_candidate_authored_copper_graph_obstacle_aware(
            net_uuid,
            from_pad_uuid,
            to_pad_uuid,
        )
        .expect("direct query should succeed");

    assert_eq!(report.status, direct.status);
    assert_eq!(report.selection_rule, direct.selection_rule);
    assert_eq!(
        report.summary.blocked_track_count,
        direct.summary.blocked_track_count
    );
    assert_eq!(
        path_ids(&report),
        vec![track_a_uuid, via_uuid, track_b_uuid]
    );
}

#[test]
fn authored_copper_graph_policy_preserves_zone_obstacle_aware_behavior() {
    let (board, net_uuid, from_pad_uuid, to_pad_uuid, zone_uuid) = zone_obstacle_board();
    let report = board
        .route_path_candidate_authored_copper_graph_by_policy(
            net_uuid,
            from_pad_uuid,
            to_pad_uuid,
            RoutePathCandidateAuthoredCopperGraphPolicy::ZoneObstacleAware,
        )
        .expect("policy query should succeed");
    let direct = board
        .route_path_candidate_authored_copper_graph_zone_obstacle_aware(
            net_uuid,
            from_pad_uuid,
            to_pad_uuid,
        )
        .expect("direct query should succeed");

    assert_eq!(report.status, direct.status);
    assert_eq!(report.selection_rule, direct.selection_rule);
    assert_eq!(
        report.summary.blocked_zone_connection_count,
        direct.summary.blocked_zone_connection_count
    );
    assert_eq!(path_ids(&report), vec![zone_uuid]);
}

#[test]
fn authored_copper_graph_policy_preserves_topology_aware_behavior() {
    let (board, net_uuid, from_pad_uuid, to_pad_uuid, via_uuid, track_a_uuid, track_b_uuid) =
        topology_board();
    let report = board
        .route_path_candidate_authored_copper_graph_by_policy(
            net_uuid,
            from_pad_uuid,
            to_pad_uuid,
            RoutePathCandidateAuthoredCopperGraphPolicy::ZoneObstacleTopologyAware,
        )
        .expect("policy query should succeed");
    let direct = board
        .route_path_candidate_authored_copper_graph_zone_obstacle_aware_topology_aware(
            net_uuid,
            from_pad_uuid,
            to_pad_uuid,
        )
        .expect("direct query should succeed");

    assert_eq!(report.status, direct.status);
    assert_eq!(report.selection_rule, direct.selection_rule);
    assert_eq!(
        report.summary.topology_transition_count,
        direct.summary.topology_transition_count
    );
    assert_eq!(
        path_ids(&report),
        vec![via_uuid, track_a_uuid, track_b_uuid]
    );
}

#[test]
fn authored_copper_graph_policy_preserves_layer_balance_aware_behavior() {
    let (board, net_uuid, from_pad_uuid, to_pad_uuid, via_uuid, track_uuid) = layer_balance_board();
    let report = board
        .route_path_candidate_authored_copper_graph_by_policy(
            net_uuid,
            from_pad_uuid,
            to_pad_uuid,
            RoutePathCandidateAuthoredCopperGraphPolicy::ZoneObstacleTopologyLayerBalanceAware,
        )
        .expect("policy query should succeed");
    let direct = board
        .route_path_candidate_authored_copper_graph_zone_obstacle_aware_topology_aware_layer_balance_aware(
            net_uuid,
            from_pad_uuid,
            to_pad_uuid,
        )
        .expect("direct query should succeed");

    assert_eq!(report.status, direct.status);
    assert_eq!(report.selection_rule, direct.selection_rule);
    assert_eq!(
        report.summary.layer_balance_score,
        direct.summary.layer_balance_score
    );
    assert_eq!(path_ids(&report), vec![via_uuid, track_uuid]);
}
