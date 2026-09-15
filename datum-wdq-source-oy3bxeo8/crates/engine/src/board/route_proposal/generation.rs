//! Per-candidate route-proposal action builders and the candidate
//! dispatcher.
//!
//! Moved verbatim from `crates/cli/src/command_project_route_proposal.rs`
//! (family F of the native-write migration): each builder queries the
//! deterministic route-path-candidate kernel on a resolved [`Board`], demands
//! a deterministic path plus persisted net-class facts, and lowers the
//! selected path into [`RouteProposalAction`]s. Action ids, reasons, error
//! strings, and per-action path facts are byte-for-byte the historical CLI
//! behavior — they land in exported artifacts and locked CLI tests.

use uuid::Uuid;

use crate::board::{
    Board, RoutePathCandidateAuthoredCopperGraphObstacleAwareStepKindView,
    RoutePathCandidateAuthoredCopperGraphPolicy,
    RoutePathCandidateAuthoredCopperGraphPolicyStepKindView,
    RoutePathCandidateAuthoredCopperGraphZoneAwareStepKindView,
    RoutePathCandidateAuthoredCopperGraphZoneObstacleAwareStepKindView,
    RoutePathCandidateAuthoredCopperGraphZoneObstacleAwareTopologyAwareLayerBalanceAwareStepKindView,
    RoutePathCandidateAuthoredCopperGraphZoneObstacleAwareTopologyAwareStepKindView,
    RoutePathCandidateAuthoredCopperPlusOneGapStepKindView, RoutePathCandidateStatus,
};
use crate::import::ids_sidecar::compute_source_hash_bytes;
use crate::ir::geometry::Point;

use super::*;

/// Build the deterministic proposal actions for one candidate strategy.
///
/// This is the single dispatch surface the CLI, selection, and revalidation
/// all build through; the authored-copper-graph candidate carries its policy
/// in the [`RouteProposalCandidate`] value.
pub fn build_route_proposal_actions(
    board: &Board,
    net_uuid: Uuid,
    from_anchor_pad_uuid: Uuid,
    to_anchor_pad_uuid: Uuid,
    candidate: RouteProposalCandidate,
) -> Result<Vec<RouteProposalAction>, String> {
    match candidate {
        RouteProposalCandidate::RoutePathCandidate => build_route_path_candidate_proposal_actions(
            board,
            net_uuid,
            from_anchor_pad_uuid,
            to_anchor_pad_uuid,
        ),
        RouteProposalCandidate::RoutePathCandidateVia => {
            build_route_path_candidate_via_proposal_actions(
                board,
                net_uuid,
                from_anchor_pad_uuid,
                to_anchor_pad_uuid,
            )
        }
        RouteProposalCandidate::RoutePathCandidateTwoVia => {
            build_route_path_candidate_two_via_proposal_actions(
                board,
                net_uuid,
                from_anchor_pad_uuid,
                to_anchor_pad_uuid,
            )
        }
        RouteProposalCandidate::RoutePathCandidateThreeVia => {
            build_route_path_candidate_three_via_proposal_actions(
                board,
                net_uuid,
                from_anchor_pad_uuid,
                to_anchor_pad_uuid,
            )
        }
        RouteProposalCandidate::RoutePathCandidateFourVia => {
            build_route_path_candidate_four_via_proposal_actions(
                board,
                net_uuid,
                from_anchor_pad_uuid,
                to_anchor_pad_uuid,
            )
        }
        RouteProposalCandidate::RoutePathCandidateFiveVia => {
            build_route_path_candidate_five_via_proposal_actions(
                board,
                net_uuid,
                from_anchor_pad_uuid,
                to_anchor_pad_uuid,
            )
        }
        RouteProposalCandidate::RoutePathCandidateSixVia => {
            build_route_path_candidate_six_via_proposal_actions(
                board,
                net_uuid,
                from_anchor_pad_uuid,
                to_anchor_pad_uuid,
            )
        }
        RouteProposalCandidate::RoutePathCandidateAuthoredViaChain => {
            build_route_path_candidate_authored_via_chain_proposal_actions(
                board,
                net_uuid,
                from_anchor_pad_uuid,
                to_anchor_pad_uuid,
            )
        }
        RouteProposalCandidate::RoutePathCandidateOrthogonalDogleg => {
            build_route_path_candidate_orthogonal_dogleg_proposal_actions(
                board,
                net_uuid,
                from_anchor_pad_uuid,
                to_anchor_pad_uuid,
            )
        }
        RouteProposalCandidate::RoutePathCandidateOrthogonalTwoBend => {
            build_route_path_candidate_orthogonal_two_bend_proposal_actions(
                board,
                net_uuid,
                from_anchor_pad_uuid,
                to_anchor_pad_uuid,
            )
        }
        RouteProposalCandidate::RoutePathCandidateOrthogonalGraph => {
            build_route_path_candidate_orthogonal_graph_proposal_actions(
                board,
                net_uuid,
                from_anchor_pad_uuid,
                to_anchor_pad_uuid,
            )
        }
        RouteProposalCandidate::RoutePathCandidateOrthogonalGraphVia => {
            build_route_path_candidate_orthogonal_graph_via_proposal_actions(
                board,
                net_uuid,
                from_anchor_pad_uuid,
                to_anchor_pad_uuid,
            )
        }
        RouteProposalCandidate::RoutePathCandidateOrthogonalGraphTwoVia => {
            build_route_path_candidate_orthogonal_graph_two_via_proposal_actions(
                board,
                net_uuid,
                from_anchor_pad_uuid,
                to_anchor_pad_uuid,
            )
        }
        RouteProposalCandidate::RoutePathCandidateOrthogonalGraphThreeVia => {
            build_route_path_candidate_orthogonal_graph_three_via_proposal_actions(
                board,
                net_uuid,
                from_anchor_pad_uuid,
                to_anchor_pad_uuid,
            )
        }
        RouteProposalCandidate::RoutePathCandidateOrthogonalGraphFourVia => {
            build_route_path_candidate_orthogonal_graph_four_via_proposal_actions(
                board,
                net_uuid,
                from_anchor_pad_uuid,
                to_anchor_pad_uuid,
            )
        }
        RouteProposalCandidate::RoutePathCandidateOrthogonalGraphFiveVia => {
            build_route_path_candidate_orthogonal_graph_five_via_proposal_actions(
                board,
                net_uuid,
                from_anchor_pad_uuid,
                to_anchor_pad_uuid,
            )
        }
        RouteProposalCandidate::RoutePathCandidateOrthogonalGraphSixVia => {
            build_route_path_candidate_orthogonal_graph_six_via_proposal_actions(
                board,
                net_uuid,
                from_anchor_pad_uuid,
                to_anchor_pad_uuid,
            )
        }
        RouteProposalCandidate::AuthoredCopperPlusOneGap => {
            build_plus_one_gap_route_proposal_actions(
                board,
                net_uuid,
                from_anchor_pad_uuid,
                to_anchor_pad_uuid,
            )
        }
        RouteProposalCandidate::AuthoredCopperGraph(policy) => {
            build_route_path_candidate_authored_copper_graph_policy_proposal_actions(
                board,
                net_uuid,
                from_anchor_pad_uuid,
                to_anchor_pad_uuid,
                policy,
            )
        }
        RouteProposalCandidate::RoutePathCandidateAuthoredCopperGraphZoneAware => {
            build_route_path_candidate_authored_copper_graph_zone_aware_proposal_actions(
                board,
                net_uuid,
                from_anchor_pad_uuid,
                to_anchor_pad_uuid,
            )
        }
        RouteProposalCandidate::RoutePathCandidateAuthoredCopperGraphZoneObstacleAware => {
            build_route_path_candidate_authored_copper_graph_zone_obstacle_aware_proposal_actions(
                board,
                net_uuid,
                from_anchor_pad_uuid,
                to_anchor_pad_uuid,
            )
        }
        RouteProposalCandidate::RoutePathCandidateAuthoredCopperGraphZoneObstacleAwareTopologyAware => {
            build_route_path_candidate_authored_copper_graph_zone_obstacle_aware_topology_aware_proposal_actions(
                board,
                net_uuid,
                from_anchor_pad_uuid,
                to_anchor_pad_uuid,
            )
        }
        RouteProposalCandidate::RoutePathCandidateAuthoredCopperGraphZoneObstacleAwareTopologyAwareLayerBalanceAware => {
            build_route_path_candidate_authored_copper_graph_zone_obstacle_aware_topology_aware_layer_balance_aware_proposal_actions(
                board,
                net_uuid,
                from_anchor_pad_uuid,
                to_anchor_pad_uuid,
            )
        }
        RouteProposalCandidate::RoutePathCandidateAuthoredCopperGraphObstacleAware => {
            build_route_path_candidate_authored_copper_graph_obstacle_aware_proposal_actions(
                board,
                net_uuid,
                from_anchor_pad_uuid,
                to_anchor_pad_uuid,
            )
        }
    }
}

pub(crate) fn build_plus_one_gap_route_proposal_actions(
    board: &Board,
    net_uuid: Uuid,
    from_anchor_pad_uuid: Uuid,
    to_anchor_pad_uuid: Uuid,
) -> Result<Vec<RouteProposalAction>, String> {
    let report = board
        .route_path_candidate_authored_copper_plus_one_gap(
            net_uuid,
            from_anchor_pad_uuid,
            to_anchor_pad_uuid,
        )
        .map_err(|err| err.to_string())?;
    if report.status != RoutePathCandidateStatus::DeterministicPathFound {
        return Err(format!(
            "route proposal requires deterministic plus-one-gap path for net {} between {} and {}",
            net_uuid, from_anchor_pad_uuid, to_anchor_pad_uuid
        ));
    }

    let preflight = board
        .route_preflight(net_uuid)
        .ok_or_else(|| format!("board net not found in native project: {net_uuid}"))?;
    let net_class = preflight.persisted_constraints.net_class.ok_or_else(|| {
        format!(
            "route proposal requires persisted net-class facts for net {}",
            net_uuid
        )
    })?;

    let path = report.path.ok_or_else(|| {
        format!(
            "route proposal requires selected path data for net {} between {} and {}",
            net_uuid, from_anchor_pad_uuid, to_anchor_pad_uuid
        )
    })?;
    let gap_steps = path
        .steps
        .iter()
        .enumerate()
        .filter(|(_, step)| {
            matches!(
                step.kind,
                RoutePathCandidateAuthoredCopperPlusOneGapStepKindView::Gap
            )
        })
        .collect::<Vec<_>>();
    if gap_steps.len() != 1 {
        return Err(format!(
            "route proposal requires exactly one eligible gap, found {} for net {}",
            gap_steps.len(),
            net_uuid
        ));
    }
    let (selected_gap_step_index, gap_step) = gap_steps[0];
    let action_id = route_proposal_action_id(
        &report.contract,
        "draw_track",
        ROUTE_PROPOSAL_REASON_AUTHORED_COPPER_PLUS_ONE_GAP,
        net_uuid,
        from_anchor_pad_uuid,
        to_anchor_pad_uuid,
        gap_step.layer,
        gap_step.from,
        gap_step.to,
        net_class.track_width_nm,
        None,
        &[],
        None,
        None,
        None,
        None,
    );

    Ok(vec![RouteProposalAction {
        action_id,
        proposal_action: "draw_track".to_string(),
        reason: ROUTE_PROPOSAL_REASON_AUTHORED_COPPER_PLUS_ONE_GAP.to_string(),
        contract: report.contract,
        net_uuid: report.net_uuid,
        net_name: report.net_name,
        from_anchor_pad_uuid: report.from_anchor_pad_uuid,
        to_anchor_pad_uuid: report.to_anchor_pad_uuid,
        layer: gap_step.layer,
        width_nm: net_class.track_width_nm,
        from: gap_step.from,
        to: gap_step.to,
        reused_via_uuid: None,
        reused_via_uuids: Vec::new(),
        reused_object_kind: None,
        reused_object_uuid: None,
        reused_object_from_layer: None,
        reused_object_to_layer: None,
        selected_path_bend_count: 0,
        selected_path_point_count: path.steps.len() + 1,
        selected_path_segment_index: selected_gap_step_index,
        selected_path_segment_count: path.steps.len(),
        selected_path_layer_segment_index: None,
        selected_path_layer_segment_count: None,
        selected_path_layer_segment_bend_count: None,
        selected_path_layer_segment_point_count: None,
    }])
}

pub(crate) fn build_route_path_candidate_proposal_actions(
    board: &Board,
    net_uuid: Uuid,
    from_anchor_pad_uuid: Uuid,
    to_anchor_pad_uuid: Uuid,
) -> Result<Vec<RouteProposalAction>, String> {
    let report = board
        .route_path_candidate(net_uuid, from_anchor_pad_uuid, to_anchor_pad_uuid)
        .map_err(|err| err.to_string())?;
    if report.status != RoutePathCandidateStatus::DeterministicPathFound {
        return Err(format!(
            "route proposal requires deterministic single-layer path for net {} between {} and {}",
            net_uuid, from_anchor_pad_uuid, to_anchor_pad_uuid
        ));
    }

    let preflight = board
        .route_preflight(net_uuid)
        .ok_or_else(|| format!("board net not found in native project: {net_uuid}"))?;
    let net_class = preflight.persisted_constraints.net_class.ok_or_else(|| {
        format!(
            "route proposal requires persisted net-class facts for net {}",
            net_uuid
        )
    })?;

    let path = report.path.ok_or_else(|| {
        format!(
            "route proposal requires selected path data for net {} between {} and {}",
            net_uuid, from_anchor_pad_uuid, to_anchor_pad_uuid
        )
    })?;
    if path.points.len() < 2 {
        return Err(format!(
            "route proposal requires at least two path points for net {}",
            net_uuid
        ));
    }
    let selected_path_segment_count = path.points.len() - 1;
    let actions = path
        .points
        .windows(2)
        .enumerate()
        .map(|(selected_path_segment_index, segment)| {
            let from = segment[0];
            let to = segment[1];
            let action_id = route_proposal_action_id(
                &report.contract,
                "draw_track",
                ROUTE_PROPOSAL_REASON_ROUTE_PATH_CANDIDATE,
                net_uuid,
                from_anchor_pad_uuid,
                to_anchor_pad_uuid,
                path.layer,
                from,
                to,
                net_class.track_width_nm,
                None,
                &[],
                None,
                None,
                None,
                None,
            );
            RouteProposalAction {
                action_id,
                proposal_action: "draw_track".to_string(),
                reason: ROUTE_PROPOSAL_REASON_ROUTE_PATH_CANDIDATE.to_string(),
                contract: report.contract.clone(),
                net_uuid: report.net_uuid,
                net_name: report.net_name.clone(),
                from_anchor_pad_uuid: report.from_anchor_pad_uuid,
                to_anchor_pad_uuid: report.to_anchor_pad_uuid,
                layer: path.layer,
                width_nm: net_class.track_width_nm,
                from,
                to,
                reused_via_uuid: None,
                reused_via_uuids: Vec::new(),
                reused_object_kind: None,
                reused_object_uuid: None,
                reused_object_from_layer: None,
                reused_object_to_layer: None,
                selected_path_bend_count: 0,
                selected_path_point_count: path.points.len(),
                selected_path_segment_index,
                selected_path_segment_count,
                selected_path_layer_segment_index: None,
                selected_path_layer_segment_count: None,
                selected_path_layer_segment_bend_count: None,
                selected_path_layer_segment_point_count: None,
            }
        })
        .collect::<Vec<_>>();
    Ok(actions)
}

pub(crate) fn build_route_path_candidate_orthogonal_dogleg_proposal_actions(
    board: &Board,
    net_uuid: Uuid,
    from_anchor_pad_uuid: Uuid,
    to_anchor_pad_uuid: Uuid,
) -> Result<Vec<RouteProposalAction>, String> {
    let report = board
        .route_path_candidate_orthogonal_dogleg(net_uuid, from_anchor_pad_uuid, to_anchor_pad_uuid)
        .map_err(|err| err.to_string())?;
    if report.status != RoutePathCandidateStatus::DeterministicPathFound {
        return Err(format!(
            "route proposal requires deterministic orthogonal dogleg path for net {} between {} and {}",
            net_uuid, from_anchor_pad_uuid, to_anchor_pad_uuid
        ));
    }

    let preflight = board
        .route_preflight(net_uuid)
        .ok_or_else(|| format!("board net not found in native project: {net_uuid}"))?;
    let net_class = preflight.persisted_constraints.net_class.ok_or_else(|| {
        format!(
            "route proposal requires persisted net-class facts for net {}",
            net_uuid
        )
    })?;

    let path = report.path.ok_or_else(|| {
        format!(
            "route proposal requires selected orthogonal dogleg path data for net {} between {} and {}",
            net_uuid,
            from_anchor_pad_uuid,
            to_anchor_pad_uuid
        )
    })?;
    if path.points.len() < 2 {
        return Err(format!(
            "route proposal requires at least two path points for net {}",
            net_uuid
        ));
    }
    let selected_path_segment_count = path.points.len() - 1;
    let actions = path
        .points
        .windows(2)
        .enumerate()
        .map(|(selected_path_segment_index, segment)| {
            let from = segment[0];
            let to = segment[1];
            let action_id = route_proposal_action_id(
                &report.contract,
                "draw_track",
                ROUTE_PROPOSAL_REASON_ROUTE_PATH_CANDIDATE_ORTHOGONAL_DOGLEG,
                net_uuid,
                from_anchor_pad_uuid,
                to_anchor_pad_uuid,
                path.layer,
                from,
                to,
                net_class.track_width_nm,
                None,
                &[],
                None,
                None,
                None,
                None,
            );
            RouteProposalAction {
                action_id,
                proposal_action: "draw_track".to_string(),
                reason: ROUTE_PROPOSAL_REASON_ROUTE_PATH_CANDIDATE_ORTHOGONAL_DOGLEG.to_string(),
                contract: report.contract.clone(),
                net_uuid: report.net_uuid,
                net_name: report.net_name.clone(),
                from_anchor_pad_uuid: report.from_anchor_pad_uuid,
                to_anchor_pad_uuid: report.to_anchor_pad_uuid,
                layer: path.layer,
                width_nm: net_class.track_width_nm,
                from,
                to,
                reused_via_uuid: None,
                reused_via_uuids: Vec::new(),
                reused_object_kind: None,
                reused_object_uuid: None,
                reused_object_from_layer: None,
                reused_object_to_layer: None,
                selected_path_bend_count: 0,
                selected_path_point_count: path.points.len(),
                selected_path_segment_index,
                selected_path_segment_count,
                selected_path_layer_segment_index: None,
                selected_path_layer_segment_count: None,
                selected_path_layer_segment_bend_count: None,
                selected_path_layer_segment_point_count: None,
            }
        })
        .collect::<Vec<_>>();
    Ok(actions)
}

pub(crate) fn build_route_path_candidate_orthogonal_two_bend_proposal_actions(
    board: &Board,
    net_uuid: Uuid,
    from_anchor_pad_uuid: Uuid,
    to_anchor_pad_uuid: Uuid,
) -> Result<Vec<RouteProposalAction>, String> {
    let report = board
        .route_path_candidate_orthogonal_two_bend(
            net_uuid,
            from_anchor_pad_uuid,
            to_anchor_pad_uuid,
        )
        .map_err(|err| err.to_string())?;
    if report.status != RoutePathCandidateStatus::DeterministicPathFound {
        return Err(format!(
            "route proposal requires deterministic orthogonal two-bend path for net {} between {} and {}",
            net_uuid, from_anchor_pad_uuid, to_anchor_pad_uuid
        ));
    }

    let preflight = board
        .route_preflight(net_uuid)
        .ok_or_else(|| format!("board net not found in native project: {net_uuid}"))?;
    let net_class = preflight.persisted_constraints.net_class.ok_or_else(|| {
        format!(
            "route proposal requires persisted net-class facts for net {}",
            net_uuid
        )
    })?;

    let path = report.path.ok_or_else(|| {
        format!(
            "route proposal requires selected orthogonal two-bend path data for net {} between {} and {}",
            net_uuid,
            from_anchor_pad_uuid,
            to_anchor_pad_uuid
        )
    })?;
    if path.points.len() < 2 {
        return Err(format!(
            "route proposal requires at least two path points for net {}",
            net_uuid
        ));
    }
    let selected_path_segment_count = path.points.len() - 1;
    let actions = path
        .points
        .windows(2)
        .enumerate()
        .map(|(selected_path_segment_index, segment)| {
            let from = segment[0];
            let to = segment[1];
            let action_id = route_proposal_action_id(
                &report.contract,
                "draw_track",
                ROUTE_PROPOSAL_REASON_ROUTE_PATH_CANDIDATE_ORTHOGONAL_TWO_BEND,
                net_uuid,
                from_anchor_pad_uuid,
                to_anchor_pad_uuid,
                path.layer,
                from,
                to,
                net_class.track_width_nm,
                None,
                &[],
                None,
                None,
                None,
                None,
            );
            RouteProposalAction {
                action_id,
                proposal_action: "draw_track".to_string(),
                reason: ROUTE_PROPOSAL_REASON_ROUTE_PATH_CANDIDATE_ORTHOGONAL_TWO_BEND.to_string(),
                contract: report.contract.clone(),
                net_uuid: report.net_uuid,
                net_name: report.net_name.clone(),
                from_anchor_pad_uuid: report.from_anchor_pad_uuid,
                to_anchor_pad_uuid: report.to_anchor_pad_uuid,
                layer: path.layer,
                width_nm: net_class.track_width_nm,
                from,
                to,
                reused_via_uuid: None,
                reused_via_uuids: Vec::new(),
                reused_object_kind: None,
                reused_object_uuid: None,
                reused_object_from_layer: None,
                reused_object_to_layer: None,
                selected_path_bend_count: 0,
                selected_path_point_count: path.points.len(),
                selected_path_segment_index,
                selected_path_segment_count,
                selected_path_layer_segment_index: None,
                selected_path_layer_segment_count: None,
                selected_path_layer_segment_bend_count: None,
                selected_path_layer_segment_point_count: None,
            }
        })
        .collect::<Vec<_>>();
    Ok(actions)
}

pub(crate) fn build_route_path_candidate_orthogonal_graph_proposal_actions(
    board: &Board,
    net_uuid: Uuid,
    from_anchor_pad_uuid: Uuid,
    to_anchor_pad_uuid: Uuid,
) -> Result<Vec<RouteProposalAction>, String> {
    let report = board
        .route_path_candidate_orthogonal_graph(net_uuid, from_anchor_pad_uuid, to_anchor_pad_uuid)
        .map_err(|err| err.to_string())?;
    if report.status != RoutePathCandidateStatus::DeterministicPathFound {
        return Err(format!(
            "route proposal requires deterministic orthogonal graph path for net {} between {} and {}",
            net_uuid, from_anchor_pad_uuid, to_anchor_pad_uuid
        ));
    }

    let preflight = board
        .route_preflight(net_uuid)
        .ok_or_else(|| format!("board net not found in native project: {net_uuid}"))?;
    let net_class = preflight.persisted_constraints.net_class.ok_or_else(|| {
        format!(
            "route proposal requires persisted net-class facts for net {}",
            net_uuid
        )
    })?;

    let path = report.path.ok_or_else(|| {
        format!(
            "route proposal requires selected orthogonal graph path data for net {} between {} and {}",
            net_uuid,
            from_anchor_pad_uuid,
            to_anchor_pad_uuid
        )
    })?;
    if path.points.len() < 2 {
        return Err(format!(
            "route proposal requires at least two path points for net {}",
            net_uuid
        ));
    }
    let selected_path_segment_count = path.points.len() - 1;
    let actions = path
        .points
        .windows(2)
        .enumerate()
        .map(|(selected_path_segment_index, segment)| {
            let from = segment[0];
            let to = segment[1];
            let action_id = route_proposal_action_id(
                &report.contract,
                "draw_track",
                ROUTE_PROPOSAL_REASON_ROUTE_PATH_CANDIDATE_ORTHOGONAL_GRAPH,
                net_uuid,
                from_anchor_pad_uuid,
                to_anchor_pad_uuid,
                path.layer,
                from,
                to,
                net_class.track_width_nm,
                None,
                &[],
                None,
                None,
                None,
                None,
            );
            RouteProposalAction {
                action_id,
                proposal_action: "draw_track".to_string(),
                reason: ROUTE_PROPOSAL_REASON_ROUTE_PATH_CANDIDATE_ORTHOGONAL_GRAPH.to_string(),
                contract: report.contract.clone(),
                net_uuid: report.net_uuid,
                net_name: report.net_name.clone(),
                from_anchor_pad_uuid: report.from_anchor_pad_uuid,
                to_anchor_pad_uuid: report.to_anchor_pad_uuid,
                layer: path.layer,
                width_nm: net_class.track_width_nm,
                from,
                to,
                reused_via_uuid: None,
                reused_via_uuids: Vec::new(),
                reused_object_kind: None,
                reused_object_uuid: None,
                reused_object_from_layer: None,
                reused_object_to_layer: None,
                selected_path_bend_count: path.cost.bend_count,
                selected_path_point_count: path.points.len(),
                selected_path_segment_index,
                selected_path_segment_count,
                selected_path_layer_segment_index: Some(0),
                selected_path_layer_segment_count: Some(1),
                selected_path_layer_segment_bend_count: Some(path.cost.bend_count),
                selected_path_layer_segment_point_count: Some(path.points.len()),
            }
        })
        .collect::<Vec<_>>();
    Ok(actions)
}

pub(crate) fn build_route_path_candidate_orthogonal_graph_via_proposal_actions(
    board: &Board,
    net_uuid: Uuid,
    from_anchor_pad_uuid: Uuid,
    to_anchor_pad_uuid: Uuid,
) -> Result<Vec<RouteProposalAction>, String> {
    let report = board
        .route_path_candidate_orthogonal_graph_via(
            net_uuid,
            from_anchor_pad_uuid,
            to_anchor_pad_uuid,
        )
        .map_err(|err| err.to_string())?;
    if report.status != RoutePathCandidateStatus::DeterministicPathFound {
        return Err(format!(
            "route proposal requires deterministic orthogonal graph via path for net {} between {} and {}",
            net_uuid, from_anchor_pad_uuid, to_anchor_pad_uuid
        ));
    }

    let preflight = board
        .route_preflight(net_uuid)
        .ok_or_else(|| format!("board net not found in native project: {net_uuid}"))?;
    let net_class = preflight.persisted_constraints.net_class.ok_or_else(|| {
        format!(
            "route proposal requires persisted net-class facts for net {}",
            net_uuid
        )
    })?;

    let path = report.path.ok_or_else(|| {
        format!(
            "route proposal requires selected orthogonal graph via path data for net {} between {} and {}",
            net_uuid,
            from_anchor_pad_uuid,
            to_anchor_pad_uuid
        )
    })?;
    let selected_path_segment_count = path
        .segments
        .iter()
        .map(|segment| segment.points.len().saturating_sub(1))
        .sum::<usize>();
    let selected_path_layer_segment_count = path.segments.len();
    let actions = path
        .segments
        .iter()
        .enumerate()
        .flat_map(|(selected_path_layer_segment_index, segment)| {
            segment
                .points
                .windows(2)
                .enumerate()
                .map(move |(edge_index, window)| {
                    (
                        selected_path_layer_segment_index,
                        edge_index,
                        segment.layer,
                        segment.cost.bend_count,
                        segment.points.len(),
                        window[0],
                        window[1],
                    )
                })
        })
        .scan(0usize, |selected_path_segment_index, segment| {
            let current = *selected_path_segment_index;
            *selected_path_segment_index += 1;
            Some((current, segment))
        })
        .map(
            |(
                selected_path_segment_index,
                (
                    selected_path_layer_segment_index,
                    _edge_index,
                    layer,
                    selected_path_layer_segment_bend_count,
                    selected_path_layer_segment_point_count,
                    from,
                    to,
                ),
            )| {
                let action_id = route_proposal_action_id(
                    &report.contract,
                    "draw_track",
                    ROUTE_PROPOSAL_REASON_ROUTE_PATH_CANDIDATE_ORTHOGONAL_GRAPH_VIA,
                    net_uuid,
                    from_anchor_pad_uuid,
                    to_anchor_pad_uuid,
                    layer,
                    from,
                    to,
                    net_class.track_width_nm,
                    Some(path.via_uuid),
                    &[path.via_uuid],
                    None,
                    None,
                    None,
                    None,
                );
                RouteProposalAction {
                    action_id,
                    proposal_action: "draw_track".to_string(),
                    reason: ROUTE_PROPOSAL_REASON_ROUTE_PATH_CANDIDATE_ORTHOGONAL_GRAPH_VIA
                        .to_string(),
                    contract: report.contract.clone(),
                    net_uuid: report.net_uuid,
                    net_name: report.net_name.clone(),
                    from_anchor_pad_uuid: report.from_anchor_pad_uuid,
                    to_anchor_pad_uuid: report.to_anchor_pad_uuid,
                    layer,
                    width_nm: net_class.track_width_nm,
                    from,
                    to,
                    reused_via_uuid: Some(path.via_uuid),
                    reused_via_uuids: vec![path.via_uuid],
                    reused_object_kind: None,
                    reused_object_uuid: None,
                    reused_object_from_layer: None,
                    reused_object_to_layer: None,
                    selected_path_bend_count: path
                        .segments
                        .iter()
                        .map(|segment| segment.cost.bend_count)
                        .sum(),
                    selected_path_point_count: path
                        .segments
                        .iter()
                        .map(|segment| segment.points.len())
                        .sum(),
                    selected_path_segment_index,
                    selected_path_segment_count,
                    selected_path_layer_segment_index: Some(selected_path_layer_segment_index),
                    selected_path_layer_segment_count: Some(selected_path_layer_segment_count),
                    selected_path_layer_segment_bend_count: Some(
                        selected_path_layer_segment_bend_count,
                    ),
                    selected_path_layer_segment_point_count: Some(
                        selected_path_layer_segment_point_count,
                    ),
                }
            },
        )
        .collect::<Vec<_>>();
    Ok(actions)
}

pub(crate) fn build_route_path_candidate_orthogonal_graph_two_via_proposal_actions(
    board: &Board,
    net_uuid: Uuid,
    from_anchor_pad_uuid: Uuid,
    to_anchor_pad_uuid: Uuid,
) -> Result<Vec<RouteProposalAction>, String> {
    let report = board
        .route_path_candidate_orthogonal_graph_two_via(
            net_uuid,
            from_anchor_pad_uuid,
            to_anchor_pad_uuid,
        )
        .map_err(|err| err.to_string())?;
    if report.status != RoutePathCandidateStatus::DeterministicPathFound {
        return Err(format!(
            "route proposal requires deterministic orthogonal graph two-via path for net {} between {} and {}",
            net_uuid, from_anchor_pad_uuid, to_anchor_pad_uuid
        ));
    }

    let preflight = board
        .route_preflight(net_uuid)
        .ok_or_else(|| format!("board net not found in native project: {net_uuid}"))?;
    let net_class = preflight.persisted_constraints.net_class.ok_or_else(|| {
        format!(
            "route proposal requires persisted net-class facts for net {}",
            net_uuid
        )
    })?;

    let path = report.path.ok_or_else(|| {
        format!(
            "route proposal requires selected orthogonal graph two-via path data for net {} between {} and {}",
            net_uuid,
            from_anchor_pad_uuid,
            to_anchor_pad_uuid
        )
    })?;
    let reused_via_uuids = vec![path.via_a_uuid, path.via_b_uuid];
    let selected_path_segment_count = path
        .segments
        .iter()
        .map(|segment| segment.points.len().saturating_sub(1))
        .sum::<usize>();
    let selected_path_point_count = path
        .segments
        .iter()
        .map(|segment| segment.points.len())
        .sum();
    let selected_path_layer_segment_count = path.segments.len();
    let actions = path
        .segments
        .iter()
        .enumerate()
        .flat_map(|(selected_path_layer_segment_index, segment)| {
            segment
                .points
                .windows(2)
                .enumerate()
                .map(move |(edge_index, window)| {
                    (
                        selected_path_layer_segment_index,
                        edge_index,
                        segment.layer,
                        segment.cost.bend_count,
                        segment.points.len(),
                        window[0],
                        window[1],
                    )
                })
        })
        .scan(0usize, |selected_path_segment_index, segment| {
            let current = *selected_path_segment_index;
            *selected_path_segment_index += 1;
            Some((current, segment))
        })
        .map(
            |(
                selected_path_segment_index,
                (
                    selected_path_layer_segment_index,
                    _edge_index,
                    layer,
                    selected_path_layer_segment_bend_count,
                    selected_path_layer_segment_point_count,
                    from,
                    to,
                ),
            )| {
                let action_id = route_proposal_action_id(
                    &report.contract,
                    "draw_track",
                    ROUTE_PROPOSAL_REASON_ROUTE_PATH_CANDIDATE_ORTHOGONAL_GRAPH_TWO_VIA,
                    net_uuid,
                    from_anchor_pad_uuid,
                    to_anchor_pad_uuid,
                    layer,
                    from,
                    to,
                    net_class.track_width_nm,
                    Some(path.via_a_uuid),
                    &reused_via_uuids,
                    None,
                    None,
                    None,
                    None,
                );
                RouteProposalAction {
                    action_id,
                    proposal_action: "draw_track".to_string(),
                    reason: ROUTE_PROPOSAL_REASON_ROUTE_PATH_CANDIDATE_ORTHOGONAL_GRAPH_TWO_VIA
                        .to_string(),
                    contract: report.contract.clone(),
                    net_uuid: report.net_uuid,
                    net_name: report.net_name.clone(),
                    from_anchor_pad_uuid: report.from_anchor_pad_uuid,
                    to_anchor_pad_uuid: report.to_anchor_pad_uuid,
                    layer,
                    width_nm: net_class.track_width_nm,
                    from,
                    to,
                    reused_via_uuid: Some(path.via_a_uuid),
                    reused_via_uuids: reused_via_uuids.clone(),
                    reused_object_kind: None,
                    reused_object_uuid: None,
                    reused_object_from_layer: None,
                    reused_object_to_layer: None,
                    selected_path_bend_count: path
                        .segments
                        .iter()
                        .map(|segment| segment.cost.bend_count)
                        .sum(),
                    selected_path_point_count,
                    selected_path_segment_index,
                    selected_path_segment_count,
                    selected_path_layer_segment_index: Some(selected_path_layer_segment_index),
                    selected_path_layer_segment_count: Some(selected_path_layer_segment_count),
                    selected_path_layer_segment_bend_count: Some(
                        selected_path_layer_segment_bend_count,
                    ),
                    selected_path_layer_segment_point_count: Some(
                        selected_path_layer_segment_point_count,
                    ),
                }
            },
        )
        .collect::<Vec<_>>();
    Ok(actions)
}

pub(crate) fn build_route_path_candidate_orthogonal_graph_three_via_proposal_actions(
    board: &Board,
    net_uuid: Uuid,
    from_anchor_pad_uuid: Uuid,
    to_anchor_pad_uuid: Uuid,
) -> Result<Vec<RouteProposalAction>, String> {
    let report = board
        .route_path_candidate_orthogonal_graph_three_via(
            net_uuid,
            from_anchor_pad_uuid,
            to_anchor_pad_uuid,
        )
        .map_err(|err| err.to_string())?;
    if report.status != RoutePathCandidateStatus::DeterministicPathFound {
        return Err(format!(
            "route proposal requires deterministic orthogonal graph three-via path for net {} between {} and {}",
            net_uuid, from_anchor_pad_uuid, to_anchor_pad_uuid
        ));
    }
    let preflight = board
        .route_preflight(net_uuid)
        .ok_or_else(|| format!("board net not found in native project: {net_uuid}"))?;
    let net_class = preflight.persisted_constraints.net_class.ok_or_else(|| {
        format!(
            "route proposal requires persisted net-class facts for net {}",
            net_uuid
        )
    })?;
    let path = report.path.ok_or_else(|| {
        format!(
            "route proposal requires selected orthogonal graph three-via path data for net {} between {} and {}",
            net_uuid,
            from_anchor_pad_uuid,
            to_anchor_pad_uuid
        )
    })?;
    let reused_via_uuids = vec![path.via_a_uuid, path.via_b_uuid, path.via_c_uuid];
    let selected_path_segment_count = path
        .segments
        .iter()
        .map(|segment| segment.points.len().saturating_sub(1))
        .sum::<usize>();
    let selected_path_point_count = path
        .segments
        .iter()
        .map(|segment| segment.points.len())
        .sum();
    let selected_path_layer_segment_count = path.segments.len();
    let actions = path
        .segments
        .iter()
        .enumerate()
        .flat_map(|(selected_path_layer_segment_index, segment)| {
            segment
                .points
                .windows(2)
                .enumerate()
                .map(move |(edge_index, window)| {
                    (
                        selected_path_layer_segment_index,
                        edge_index,
                        segment.layer,
                        segment.cost.bend_count,
                        segment.points.len(),
                        window[0],
                        window[1],
                    )
                })
        })
        .scan(0usize, |selected_path_segment_index, segment| {
            let current = *selected_path_segment_index;
            *selected_path_segment_index += 1;
            Some((current, segment))
        })
        .map(
            |(
                selected_path_segment_index,
                (
                    selected_path_layer_segment_index,
                    _edge_index,
                    layer,
                    selected_path_layer_segment_bend_count,
                    selected_path_layer_segment_point_count,
                    from,
                    to,
                ),
            )| {
                let action_id = route_proposal_action_id(
                    &report.contract,
                    "draw_track",
                    ROUTE_PROPOSAL_REASON_ROUTE_PATH_CANDIDATE_ORTHOGONAL_GRAPH_THREE_VIA,
                    net_uuid,
                    from_anchor_pad_uuid,
                    to_anchor_pad_uuid,
                    layer,
                    from,
                    to,
                    net_class.track_width_nm,
                    Some(path.via_a_uuid),
                    &reused_via_uuids,
                    None,
                    None,
                    None,
                    None,
                );
                RouteProposalAction {
                    action_id,
                    proposal_action: "draw_track".to_string(),
                    reason: ROUTE_PROPOSAL_REASON_ROUTE_PATH_CANDIDATE_ORTHOGONAL_GRAPH_THREE_VIA
                        .to_string(),
                    contract: report.contract.clone(),
                    net_uuid: report.net_uuid,
                    net_name: report.net_name.clone(),
                    from_anchor_pad_uuid: report.from_anchor_pad_uuid,
                    to_anchor_pad_uuid: report.to_anchor_pad_uuid,
                    layer,
                    width_nm: net_class.track_width_nm,
                    from,
                    to,
                    reused_via_uuid: Some(path.via_a_uuid),
                    reused_via_uuids: reused_via_uuids.clone(),
                    reused_object_kind: None,
                    reused_object_uuid: None,
                    reused_object_from_layer: None,
                    reused_object_to_layer: None,
                    selected_path_bend_count: path
                        .segments
                        .iter()
                        .map(|segment| segment.cost.bend_count)
                        .sum(),
                    selected_path_point_count,
                    selected_path_segment_index,
                    selected_path_segment_count,
                    selected_path_layer_segment_index: Some(selected_path_layer_segment_index),
                    selected_path_layer_segment_count: Some(selected_path_layer_segment_count),
                    selected_path_layer_segment_bend_count: Some(
                        selected_path_layer_segment_bend_count,
                    ),
                    selected_path_layer_segment_point_count: Some(
                        selected_path_layer_segment_point_count,
                    ),
                }
            },
        )
        .collect::<Vec<_>>();
    Ok(actions)
}

pub(crate) fn build_route_path_candidate_orthogonal_graph_four_via_proposal_actions(
    board: &Board,
    net_uuid: Uuid,
    from_anchor_pad_uuid: Uuid,
    to_anchor_pad_uuid: Uuid,
) -> Result<Vec<RouteProposalAction>, String> {
    let report = board
        .route_path_candidate_orthogonal_graph_four_via(
            net_uuid,
            from_anchor_pad_uuid,
            to_anchor_pad_uuid,
        )
        .map_err(|err| err.to_string())?;
    if report.status != RoutePathCandidateStatus::DeterministicPathFound {
        return Err(format!(
            "route proposal requires deterministic orthogonal graph four-via path for net {} between {} and {}",
            net_uuid, from_anchor_pad_uuid, to_anchor_pad_uuid
        ));
    }
    let preflight = board
        .route_preflight(net_uuid)
        .ok_or_else(|| format!("board net not found in native project: {net_uuid}"))?;
    let net_class = preflight.persisted_constraints.net_class.ok_or_else(|| {
        format!(
            "route proposal requires persisted net-class facts for net {}",
            net_uuid
        )
    })?;
    let path = report.path.ok_or_else(|| {
        format!(
            "route proposal requires selected orthogonal graph four-via path data for net {} between {} and {}",
            net_uuid,
            from_anchor_pad_uuid,
            to_anchor_pad_uuid
        )
    })?;
    let reused_via_uuids = vec![
        path.via_a_uuid,
        path.via_b_uuid,
        path.via_c_uuid,
        path.via_d_uuid,
    ];
    let selected_path_segment_count = path
        .segments
        .iter()
        .map(|segment| segment.points.len().saturating_sub(1))
        .sum::<usize>();
    let selected_path_point_count = path
        .segments
        .iter()
        .map(|segment| segment.points.len())
        .sum();
    let selected_path_layer_segment_count = path.segments.len();
    let actions = path
        .segments
        .iter()
        .enumerate()
        .flat_map(|(selected_path_layer_segment_index, segment)| {
            segment
                .points
                .windows(2)
                .enumerate()
                .map(move |(edge_index, window)| {
                    (
                        selected_path_layer_segment_index,
                        edge_index,
                        segment.layer,
                        segment.cost.bend_count,
                        segment.points.len(),
                        window[0],
                        window[1],
                    )
                })
        })
        .scan(0usize, |selected_path_segment_index, segment| {
            let current = *selected_path_segment_index;
            *selected_path_segment_index += 1;
            Some((current, segment))
        })
        .map(
            |(
                selected_path_segment_index,
                (
                    selected_path_layer_segment_index,
                    _edge_index,
                    layer,
                    selected_path_layer_segment_bend_count,
                    selected_path_layer_segment_point_count,
                    from,
                    to,
                ),
            )| {
                let action_id = route_proposal_action_id(
                    &report.contract,
                    "draw_track",
                    ROUTE_PROPOSAL_REASON_ROUTE_PATH_CANDIDATE_ORTHOGONAL_GRAPH_FOUR_VIA,
                    net_uuid,
                    from_anchor_pad_uuid,
                    to_anchor_pad_uuid,
                    layer,
                    from,
                    to,
                    net_class.track_width_nm,
                    Some(path.via_a_uuid),
                    &reused_via_uuids,
                    None,
                    None,
                    None,
                    None,
                );
                RouteProposalAction {
                    action_id,
                    proposal_action: "draw_track".to_string(),
                    reason: ROUTE_PROPOSAL_REASON_ROUTE_PATH_CANDIDATE_ORTHOGONAL_GRAPH_FOUR_VIA
                        .to_string(),
                    contract: report.contract.clone(),
                    net_uuid: report.net_uuid,
                    net_name: report.net_name.clone(),
                    from_anchor_pad_uuid: report.from_anchor_pad_uuid,
                    to_anchor_pad_uuid: report.to_anchor_pad_uuid,
                    layer,
                    width_nm: net_class.track_width_nm,
                    from,
                    to,
                    reused_via_uuid: Some(path.via_a_uuid),
                    reused_via_uuids: reused_via_uuids.clone(),
                    reused_object_kind: None,
                    reused_object_uuid: None,
                    reused_object_from_layer: None,
                    reused_object_to_layer: None,
                    selected_path_bend_count: path
                        .segments
                        .iter()
                        .map(|segment| segment.cost.bend_count)
                        .sum(),
                    selected_path_point_count,
                    selected_path_segment_index,
                    selected_path_segment_count,
                    selected_path_layer_segment_index: Some(selected_path_layer_segment_index),
                    selected_path_layer_segment_count: Some(selected_path_layer_segment_count),
                    selected_path_layer_segment_bend_count: Some(
                        selected_path_layer_segment_bend_count,
                    ),
                    selected_path_layer_segment_point_count: Some(
                        selected_path_layer_segment_point_count,
                    ),
                }
            },
        )
        .collect::<Vec<_>>();
    Ok(actions)
}

pub(crate) fn build_route_path_candidate_orthogonal_graph_five_via_proposal_actions(
    board: &Board,
    net_uuid: Uuid,
    from_anchor_pad_uuid: Uuid,
    to_anchor_pad_uuid: Uuid,
) -> Result<Vec<RouteProposalAction>, String> {
    let report = board
        .route_path_candidate_orthogonal_graph_five_via(
            net_uuid,
            from_anchor_pad_uuid,
            to_anchor_pad_uuid,
        )
        .map_err(|err| err.to_string())?;
    if report.status != RoutePathCandidateStatus::DeterministicPathFound {
        return Err(format!(
            "route proposal requires deterministic orthogonal graph five-via path for net {} between {} and {}",
            net_uuid, from_anchor_pad_uuid, to_anchor_pad_uuid
        ));
    }
    let preflight = board
        .route_preflight(net_uuid)
        .ok_or_else(|| format!("board net not found in native project: {net_uuid}"))?;
    let net_class = preflight.persisted_constraints.net_class.ok_or_else(|| {
        format!(
            "route proposal requires persisted net-class facts for net {}",
            net_uuid
        )
    })?;
    let path = report.path.ok_or_else(|| {
        format!(
            "route proposal requires selected orthogonal graph five-via path data for net {} between {} and {}",
            net_uuid,
            from_anchor_pad_uuid,
            to_anchor_pad_uuid
        )
    })?;
    let reused_via_uuids = vec![
        path.via_a_uuid,
        path.via_b_uuid,
        path.via_c_uuid,
        path.via_d_uuid,
        path.via_e_uuid,
    ];
    let selected_path_segment_count = path
        .segments
        .iter()
        .map(|segment| segment.points.len().saturating_sub(1))
        .sum::<usize>();
    let selected_path_point_count = path
        .segments
        .iter()
        .map(|segment| segment.points.len())
        .sum();
    let selected_path_layer_segment_count = path.segments.len();
    let actions = path
        .segments
        .iter()
        .enumerate()
        .flat_map(|(selected_path_layer_segment_index, segment)| {
            segment
                .points
                .windows(2)
                .enumerate()
                .map(move |(edge_index, window)| {
                    (
                        selected_path_layer_segment_index,
                        edge_index,
                        segment.layer,
                        segment.cost.bend_count,
                        segment.points.len(),
                        window[0],
                        window[1],
                    )
                })
        })
        .scan(0usize, |selected_path_segment_index, segment| {
            let current = *selected_path_segment_index;
            *selected_path_segment_index += 1;
            Some((current, segment))
        })
        .map(
            |(
                selected_path_segment_index,
                (
                    selected_path_layer_segment_index,
                    _edge_index,
                    layer,
                    selected_path_layer_segment_bend_count,
                    selected_path_layer_segment_point_count,
                    from,
                    to,
                ),
            )| {
                let action_id = route_proposal_action_id(
                    &report.contract,
                    "draw_track",
                    ROUTE_PROPOSAL_REASON_ROUTE_PATH_CANDIDATE_ORTHOGONAL_GRAPH_FIVE_VIA,
                    net_uuid,
                    from_anchor_pad_uuid,
                    to_anchor_pad_uuid,
                    layer,
                    from,
                    to,
                    net_class.track_width_nm,
                    Some(path.via_a_uuid),
                    &reused_via_uuids,
                    None,
                    None,
                    None,
                    None,
                );
                RouteProposalAction {
                    action_id,
                    proposal_action: "draw_track".to_string(),
                    reason: ROUTE_PROPOSAL_REASON_ROUTE_PATH_CANDIDATE_ORTHOGONAL_GRAPH_FIVE_VIA
                        .to_string(),
                    contract: report.contract.clone(),
                    net_uuid: report.net_uuid,
                    net_name: report.net_name.clone(),
                    from_anchor_pad_uuid: report.from_anchor_pad_uuid,
                    to_anchor_pad_uuid: report.to_anchor_pad_uuid,
                    layer,
                    width_nm: net_class.track_width_nm,
                    from,
                    to,
                    reused_via_uuid: Some(path.via_a_uuid),
                    reused_via_uuids: reused_via_uuids.clone(),
                    reused_object_kind: None,
                    reused_object_uuid: None,
                    reused_object_from_layer: None,
                    reused_object_to_layer: None,
                    selected_path_bend_count: path
                        .segments
                        .iter()
                        .map(|segment| segment.cost.bend_count)
                        .sum(),
                    selected_path_point_count,
                    selected_path_segment_index,
                    selected_path_segment_count,
                    selected_path_layer_segment_index: Some(selected_path_layer_segment_index),
                    selected_path_layer_segment_count: Some(selected_path_layer_segment_count),
                    selected_path_layer_segment_bend_count: Some(
                        selected_path_layer_segment_bend_count,
                    ),
                    selected_path_layer_segment_point_count: Some(
                        selected_path_layer_segment_point_count,
                    ),
                }
            },
        )
        .collect::<Vec<_>>();
    Ok(actions)
}

pub(crate) fn build_route_path_candidate_orthogonal_graph_six_via_proposal_actions(
    board: &Board,
    net_uuid: Uuid,
    from_anchor_pad_uuid: Uuid,
    to_anchor_pad_uuid: Uuid,
) -> Result<Vec<RouteProposalAction>, String> {
    let report = board
        .route_path_candidate_orthogonal_graph_six_via(
            net_uuid,
            from_anchor_pad_uuid,
            to_anchor_pad_uuid,
        )
        .map_err(|err| err.to_string())?;
    if report.status != RoutePathCandidateStatus::DeterministicPathFound {
        return Err(format!(
            "route proposal requires deterministic orthogonal graph six-via path for net {} between {} and {}",
            net_uuid, from_anchor_pad_uuid, to_anchor_pad_uuid
        ));
    }
    let preflight = board
        .route_preflight(net_uuid)
        .ok_or_else(|| format!("board net not found in native project: {net_uuid}"))?;
    let net_class = preflight.persisted_constraints.net_class.ok_or_else(|| {
        format!(
            "route proposal requires persisted net-class facts for net {}",
            net_uuid
        )
    })?;
    let path = report.path.ok_or_else(|| {
        format!(
            "route proposal requires selected orthogonal graph six-via path data for net {} between {} and {}",
            net_uuid,
            from_anchor_pad_uuid,
            to_anchor_pad_uuid
        )
    })?;
    let reused_via_uuids = vec![
        path.via_a_uuid,
        path.via_b_uuid,
        path.via_c_uuid,
        path.via_d_uuid,
        path.via_e_uuid,
        path.via_f_uuid,
    ];
    let selected_path_segment_count = path
        .segments
        .iter()
        .map(|segment| segment.points.len().saturating_sub(1))
        .sum::<usize>();
    let selected_path_point_count = path
        .segments
        .iter()
        .map(|segment| segment.points.len())
        .sum();
    let selected_path_layer_segment_count = path.segments.len();
    let actions = path
        .segments
        .iter()
        .enumerate()
        .flat_map(|(selected_path_layer_segment_index, segment)| {
            segment
                .points
                .windows(2)
                .enumerate()
                .map(move |(edge_index, window)| {
                    (
                        selected_path_layer_segment_index,
                        edge_index,
                        segment.layer,
                        segment.cost.bend_count,
                        segment.points.len(),
                        window[0],
                        window[1],
                    )
                })
        })
        .scan(0usize, |selected_path_segment_index, segment| {
            let current = *selected_path_segment_index;
            *selected_path_segment_index += 1;
            Some((current, segment))
        })
        .map(
            |(
                selected_path_segment_index,
                (
                    selected_path_layer_segment_index,
                    _edge_index,
                    layer,
                    selected_path_layer_segment_bend_count,
                    selected_path_layer_segment_point_count,
                    from,
                    to,
                ),
            )| {
                let action_id = route_proposal_action_id(
                    &report.contract,
                    "draw_track",
                    ROUTE_PROPOSAL_REASON_ROUTE_PATH_CANDIDATE_ORTHOGONAL_GRAPH_SIX_VIA,
                    net_uuid,
                    from_anchor_pad_uuid,
                    to_anchor_pad_uuid,
                    layer,
                    from,
                    to,
                    net_class.track_width_nm,
                    Some(path.via_a_uuid),
                    &reused_via_uuids,
                    None,
                    None,
                    None,
                    None,
                );
                RouteProposalAction {
                    action_id,
                    proposal_action: "draw_track".to_string(),
                    reason: ROUTE_PROPOSAL_REASON_ROUTE_PATH_CANDIDATE_ORTHOGONAL_GRAPH_SIX_VIA
                        .to_string(),
                    contract: report.contract.clone(),
                    net_uuid: report.net_uuid,
                    net_name: report.net_name.clone(),
                    from_anchor_pad_uuid: report.from_anchor_pad_uuid,
                    to_anchor_pad_uuid: report.to_anchor_pad_uuid,
                    layer,
                    width_nm: net_class.track_width_nm,
                    from,
                    to,
                    reused_via_uuid: Some(path.via_a_uuid),
                    reused_via_uuids: reused_via_uuids.clone(),
                    reused_object_kind: None,
                    reused_object_uuid: None,
                    reused_object_from_layer: None,
                    reused_object_to_layer: None,
                    selected_path_bend_count: path
                        .segments
                        .iter()
                        .map(|segment| segment.cost.bend_count)
                        .sum(),
                    selected_path_point_count,
                    selected_path_segment_index,
                    selected_path_segment_count,
                    selected_path_layer_segment_index: Some(selected_path_layer_segment_index),
                    selected_path_layer_segment_count: Some(selected_path_layer_segment_count),
                    selected_path_layer_segment_bend_count: Some(
                        selected_path_layer_segment_bend_count,
                    ),
                    selected_path_layer_segment_point_count: Some(
                        selected_path_layer_segment_point_count,
                    ),
                }
            },
        )
        .collect::<Vec<_>>();
    Ok(actions)
}

pub(crate) fn build_route_path_candidate_via_proposal_actions(
    board: &Board,
    net_uuid: Uuid,
    from_anchor_pad_uuid: Uuid,
    to_anchor_pad_uuid: Uuid,
) -> Result<Vec<RouteProposalAction>, String> {
    let report = board
        .route_path_candidate_via(net_uuid, from_anchor_pad_uuid, to_anchor_pad_uuid)
        .map_err(|err| err.to_string())?;
    if report.status != RoutePathCandidateStatus::DeterministicPathFound {
        return Err(format!(
            "route proposal requires deterministic single-via path for net {} between {} and {}",
            net_uuid, from_anchor_pad_uuid, to_anchor_pad_uuid
        ));
    }

    let preflight = board
        .route_preflight(net_uuid)
        .ok_or_else(|| format!("board net not found in native project: {net_uuid}"))?;
    let net_class = preflight.persisted_constraints.net_class.ok_or_else(|| {
        format!(
            "route proposal requires persisted net-class facts for net {}",
            net_uuid
        )
    })?;

    let path = report.path.ok_or_else(|| {
        format!(
            "route proposal requires selected via path data for net {} between {} and {}",
            net_uuid, from_anchor_pad_uuid, to_anchor_pad_uuid
        )
    })?;
    let selected_path_segment_count = path.segments.len();
    let actions = path
        .segments
        .iter()
        .enumerate()
        .flat_map(|(selected_path_segment_index, segment)| {
            segment
                .points
                .windows(2)
                .map(move |pair| (selected_path_segment_index, segment.layer, pair))
        })
        .map(|(selected_path_segment_index, layer, pair)| {
            let from = pair[0];
            let to = pair[1];
            let action_id = route_proposal_action_id(
                &report.contract,
                "draw_track",
                ROUTE_PROPOSAL_REASON_ROUTE_PATH_CANDIDATE_VIA,
                net_uuid,
                from_anchor_pad_uuid,
                to_anchor_pad_uuid,
                layer,
                from,
                to,
                net_class.track_width_nm,
                Some(path.via_uuid),
                &[path.via_uuid],
                None,
                None,
                None,
                None,
            );
            RouteProposalAction {
                action_id,
                proposal_action: "draw_track".to_string(),
                reason: ROUTE_PROPOSAL_REASON_ROUTE_PATH_CANDIDATE_VIA.to_string(),
                contract: report.contract.clone(),
                net_uuid: report.net_uuid,
                net_name: report.net_name.clone(),
                from_anchor_pad_uuid: report.from_anchor_pad_uuid,
                to_anchor_pad_uuid: report.to_anchor_pad_uuid,
                layer,
                width_nm: net_class.track_width_nm,
                from,
                to,
                reused_via_uuid: Some(path.via_uuid),
                reused_via_uuids: vec![path.via_uuid],
                reused_object_kind: None,
                reused_object_uuid: None,
                reused_object_from_layer: None,
                reused_object_to_layer: None,
                selected_path_bend_count: 0,
                selected_path_point_count: path
                    .segments
                    .get(selected_path_segment_index)
                    .map(|segment| segment.points.len())
                    .unwrap_or(0),
                selected_path_segment_index,
                selected_path_segment_count,
                selected_path_layer_segment_index: None,
                selected_path_layer_segment_count: None,
                selected_path_layer_segment_bend_count: None,
                selected_path_layer_segment_point_count: None,
            }
        })
        .collect::<Vec<_>>();
    Ok(actions)
}

pub(crate) fn build_route_path_candidate_two_via_proposal_actions(
    board: &Board,
    net_uuid: Uuid,
    from_anchor_pad_uuid: Uuid,
    to_anchor_pad_uuid: Uuid,
) -> Result<Vec<RouteProposalAction>, String> {
    let report = board
        .route_path_candidate_two_via(net_uuid, from_anchor_pad_uuid, to_anchor_pad_uuid)
        .map_err(|err| err.to_string())?;
    if report.status != RoutePathCandidateStatus::DeterministicPathFound {
        return Err(format!(
            "route proposal requires deterministic two-via path for net {} between {} and {}",
            net_uuid, from_anchor_pad_uuid, to_anchor_pad_uuid
        ));
    }

    let preflight = board
        .route_preflight(net_uuid)
        .ok_or_else(|| format!("board net not found in native project: {net_uuid}"))?;
    let net_class = preflight.persisted_constraints.net_class.ok_or_else(|| {
        format!(
            "route proposal requires persisted net-class facts for net {}",
            net_uuid
        )
    })?;

    let path = report.path.ok_or_else(|| {
        format!(
            "route proposal requires selected two-via path data for net {} between {} and {}",
            net_uuid, from_anchor_pad_uuid, to_anchor_pad_uuid
        )
    })?;
    let reused_via_uuids = vec![path.via_a_uuid, path.via_b_uuid];
    Ok(build_segmented_route_proposal_actions(
        &report.contract,
        ROUTE_PROPOSAL_REASON_ROUTE_PATH_CANDIDATE_TWO_VIA,
        report.net_uuid,
        &report.net_name,
        report.from_anchor_pad_uuid,
        report.to_anchor_pad_uuid,
        net_class.track_width_nm,
        path.segments
            .iter()
            .map(|segment| (segment.layer, segment.points.as_slice()))
            .collect(),
        &reused_via_uuids,
    ))
}

pub(crate) fn build_route_path_candidate_three_via_proposal_actions(
    board: &Board,
    net_uuid: Uuid,
    from_anchor_pad_uuid: Uuid,
    to_anchor_pad_uuid: Uuid,
) -> Result<Vec<RouteProposalAction>, String> {
    let report = board
        .route_path_candidate_three_via(net_uuid, from_anchor_pad_uuid, to_anchor_pad_uuid)
        .map_err(|err| err.to_string())?;
    if report.status != RoutePathCandidateStatus::DeterministicPathFound {
        return Err(format!(
            "route proposal requires deterministic three-via path for net {} between {} and {}",
            net_uuid, from_anchor_pad_uuid, to_anchor_pad_uuid
        ));
    }

    let preflight = board
        .route_preflight(net_uuid)
        .ok_or_else(|| format!("board net not found in native project: {net_uuid}"))?;
    let net_class = preflight.persisted_constraints.net_class.ok_or_else(|| {
        format!(
            "route proposal requires persisted net-class facts for net {}",
            net_uuid
        )
    })?;
    let path = report.path.ok_or_else(|| {
        format!(
            "route proposal requires selected three-via path data for net {} between {} and {}",
            net_uuid, from_anchor_pad_uuid, to_anchor_pad_uuid
        )
    })?;
    let reused_via_uuids = vec![path.via_a_uuid, path.via_b_uuid, path.via_c_uuid];
    Ok(build_segmented_route_proposal_actions(
        &report.contract,
        ROUTE_PROPOSAL_REASON_ROUTE_PATH_CANDIDATE_THREE_VIA,
        report.net_uuid,
        &report.net_name,
        report.from_anchor_pad_uuid,
        report.to_anchor_pad_uuid,
        net_class.track_width_nm,
        path.segments
            .iter()
            .map(|segment| (segment.layer, segment.points.as_slice()))
            .collect(),
        &reused_via_uuids,
    ))
}

pub(crate) fn build_route_path_candidate_four_via_proposal_actions(
    board: &Board,
    net_uuid: Uuid,
    from_anchor_pad_uuid: Uuid,
    to_anchor_pad_uuid: Uuid,
) -> Result<Vec<RouteProposalAction>, String> {
    let report = board
        .route_path_candidate_four_via(net_uuid, from_anchor_pad_uuid, to_anchor_pad_uuid)
        .map_err(|err| err.to_string())?;
    if report.status != RoutePathCandidateStatus::DeterministicPathFound {
        return Err(format!(
            "route proposal requires deterministic four-via path for net {} between {} and {}",
            net_uuid, from_anchor_pad_uuid, to_anchor_pad_uuid
        ));
    }

    let preflight = board
        .route_preflight(net_uuid)
        .ok_or_else(|| format!("board net not found in native project: {net_uuid}"))?;
    let net_class = preflight.persisted_constraints.net_class.ok_or_else(|| {
        format!(
            "route proposal requires persisted net-class facts for net {}",
            net_uuid
        )
    })?;
    let path = report.path.ok_or_else(|| {
        format!(
            "route proposal requires selected four-via path data for net {} between {} and {}",
            net_uuid, from_anchor_pad_uuid, to_anchor_pad_uuid
        )
    })?;
    let reused_via_uuids = vec![
        path.via_a_uuid,
        path.via_b_uuid,
        path.via_c_uuid,
        path.via_d_uuid,
    ];
    Ok(build_segmented_route_proposal_actions(
        &report.contract,
        ROUTE_PROPOSAL_REASON_ROUTE_PATH_CANDIDATE_FOUR_VIA,
        report.net_uuid,
        &report.net_name,
        report.from_anchor_pad_uuid,
        report.to_anchor_pad_uuid,
        net_class.track_width_nm,
        path.segments
            .iter()
            .map(|segment| (segment.layer, segment.points.as_slice()))
            .collect(),
        &reused_via_uuids,
    ))
}

pub(crate) fn build_route_path_candidate_five_via_proposal_actions(
    board: &Board,
    net_uuid: Uuid,
    from_anchor_pad_uuid: Uuid,
    to_anchor_pad_uuid: Uuid,
) -> Result<Vec<RouteProposalAction>, String> {
    let report = board
        .route_path_candidate_five_via(net_uuid, from_anchor_pad_uuid, to_anchor_pad_uuid)
        .map_err(|err| err.to_string())?;
    if report.status != RoutePathCandidateStatus::DeterministicPathFound {
        return Err(format!(
            "route proposal requires deterministic five-via path for net {} between {} and {}",
            net_uuid, from_anchor_pad_uuid, to_anchor_pad_uuid
        ));
    }

    let preflight = board
        .route_preflight(net_uuid)
        .ok_or_else(|| format!("board net not found in native project: {net_uuid}"))?;
    let net_class = preflight.persisted_constraints.net_class.ok_or_else(|| {
        format!(
            "route proposal requires persisted net-class facts for net {}",
            net_uuid
        )
    })?;
    let path = report.path.ok_or_else(|| {
        format!(
            "route proposal requires selected five-via path data for net {} between {} and {}",
            net_uuid, from_anchor_pad_uuid, to_anchor_pad_uuid
        )
    })?;
    let reused_via_uuids = vec![
        path.via_a_uuid,
        path.via_b_uuid,
        path.via_c_uuid,
        path.via_d_uuid,
        path.via_e_uuid,
    ];
    Ok(build_segmented_route_proposal_actions(
        &report.contract,
        ROUTE_PROPOSAL_REASON_ROUTE_PATH_CANDIDATE_FIVE_VIA,
        report.net_uuid,
        &report.net_name,
        report.from_anchor_pad_uuid,
        report.to_anchor_pad_uuid,
        net_class.track_width_nm,
        path.segments
            .iter()
            .map(|segment| (segment.layer, segment.points.as_slice()))
            .collect(),
        &reused_via_uuids,
    ))
}

pub(crate) fn build_route_path_candidate_six_via_proposal_actions(
    board: &Board,
    net_uuid: Uuid,
    from_anchor_pad_uuid: Uuid,
    to_anchor_pad_uuid: Uuid,
) -> Result<Vec<RouteProposalAction>, String> {
    let report = board
        .route_path_candidate_six_via(net_uuid, from_anchor_pad_uuid, to_anchor_pad_uuid)
        .map_err(|err| err.to_string())?;
    if report.status != RoutePathCandidateStatus::DeterministicPathFound {
        return Err(format!(
            "route proposal requires deterministic six-via path for net {} between {} and {}",
            net_uuid, from_anchor_pad_uuid, to_anchor_pad_uuid
        ));
    }

    let preflight = board
        .route_preflight(net_uuid)
        .ok_or_else(|| format!("board net not found in native project: {net_uuid}"))?;
    let net_class = preflight.persisted_constraints.net_class.ok_or_else(|| {
        format!(
            "route proposal requires persisted net-class facts for net {}",
            net_uuid
        )
    })?;
    let path = report.path.ok_or_else(|| {
        format!(
            "route proposal requires selected six-via path data for net {} between {} and {}",
            net_uuid, from_anchor_pad_uuid, to_anchor_pad_uuid
        )
    })?;
    let reused_via_uuids = vec![
        path.via_a_uuid,
        path.via_b_uuid,
        path.via_c_uuid,
        path.via_d_uuid,
        path.via_e_uuid,
        path.via_f_uuid,
    ];
    Ok(build_segmented_route_proposal_actions(
        &report.contract,
        ROUTE_PROPOSAL_REASON_ROUTE_PATH_CANDIDATE_SIX_VIA,
        report.net_uuid,
        &report.net_name,
        report.from_anchor_pad_uuid,
        report.to_anchor_pad_uuid,
        net_class.track_width_nm,
        path.segments
            .iter()
            .map(|segment| (segment.layer, segment.points.as_slice()))
            .collect(),
        &reused_via_uuids,
    ))
}

pub(crate) fn build_route_path_candidate_authored_via_chain_proposal_actions(
    board: &Board,
    net_uuid: Uuid,
    from_anchor_pad_uuid: Uuid,
    to_anchor_pad_uuid: Uuid,
) -> Result<Vec<RouteProposalAction>, String> {
    let report = board
        .route_path_candidate_authored_via_chain(net_uuid, from_anchor_pad_uuid, to_anchor_pad_uuid)
        .map_err(|err| err.to_string())?;
    if report.status != RoutePathCandidateStatus::DeterministicPathFound {
        return Err(format!(
            "route proposal requires deterministic authored via chain path for net {} between {} and {}",
            net_uuid, from_anchor_pad_uuid, to_anchor_pad_uuid
        ));
    }

    let preflight = board
        .route_preflight(net_uuid)
        .ok_or_else(|| format!("board net not found in native project: {net_uuid}"))?;
    let net_class = preflight.persisted_constraints.net_class.ok_or_else(|| {
        format!(
            "route proposal requires persisted net-class facts for net {}",
            net_uuid
        )
    })?;
    let path = report.path.ok_or_else(|| {
        format!(
            "route proposal requires selected authored via chain path data for net {} between {} and {}",
            net_uuid,
            from_anchor_pad_uuid,
            to_anchor_pad_uuid
        )
    })?;
    let reused_via_uuids = path
        .via_chain
        .iter()
        .map(|via| via.via_uuid)
        .collect::<Vec<_>>();
    Ok(build_segmented_route_proposal_actions(
        &report.contract,
        ROUTE_PROPOSAL_REASON_ROUTE_PATH_CANDIDATE_AUTHORED_VIA_CHAIN,
        report.net_uuid,
        &report.net_name,
        report.from_anchor_pad_uuid,
        report.to_anchor_pad_uuid,
        net_class.track_width_nm,
        path.segments
            .iter()
            .map(|segment| (segment.layer, segment.points.as_slice()))
            .collect(),
        &reused_via_uuids,
    ))
}

pub(crate) fn build_route_path_candidate_authored_copper_graph_zone_aware_proposal_actions(
    board: &Board,
    net_uuid: Uuid,
    from_anchor_pad_uuid: Uuid,
    to_anchor_pad_uuid: Uuid,
) -> Result<Vec<RouteProposalAction>, String> {
    let report = board
        .route_path_candidate_authored_copper_graph_zone_aware(
            net_uuid,
            from_anchor_pad_uuid,
            to_anchor_pad_uuid,
        )
        .map_err(|err| err.to_string())?;
    if report.status != RoutePathCandidateStatus::DeterministicPathFound {
        return Err(format!(
            "route proposal requires deterministic zone-aware authored-copper path for net {} between {} and {}",
            net_uuid, from_anchor_pad_uuid, to_anchor_pad_uuid
        ));
    }

    let preflight = board
        .route_preflight(net_uuid)
        .ok_or_else(|| format!("board net not found in native project: {net_uuid}"))?;
    let net_class = preflight.persisted_constraints.net_class.ok_or_else(|| {
        format!(
            "route proposal requires persisted net-class facts for net {}",
            net_uuid
        )
    })?;
    let path = report.path.ok_or_else(|| {
        format!(
            "route proposal requires selected zone-aware authored-copper path data for net {} between {} and {}",
            net_uuid,
            from_anchor_pad_uuid,
            to_anchor_pad_uuid
        )
    })?;
    let selected_path_segment_count = path.steps.len();
    let actions = path
        .steps
        .iter()
        .enumerate()
        .map(|(selected_path_segment_index, step)| {
            let reused_object_kind = match step.kind {
                RoutePathCandidateAuthoredCopperGraphZoneAwareStepKindView::Track => "track",
                RoutePathCandidateAuthoredCopperGraphZoneAwareStepKindView::Via => "via",
                RoutePathCandidateAuthoredCopperGraphZoneAwareStepKindView::Zone => "zone",
            };
            let action_id = route_proposal_action_id(
                &report.contract,
                "reuse_existing_copper_step",
                ROUTE_PROPOSAL_REASON_ROUTE_PATH_CANDIDATE_AUTHORED_COPPER_GRAPH_ZONE_AWARE,
                net_uuid,
                from_anchor_pad_uuid,
                to_anchor_pad_uuid,
                step.layer,
                step.from,
                step.to,
                net_class.track_width_nm,
                None,
                &[],
                Some(reused_object_kind),
                Some(step.object_uuid),
                step.from_layer,
                step.to_layer,
            );
            RouteProposalAction {
                action_id,
                proposal_action: "reuse_existing_copper_step".to_string(),
                reason: ROUTE_PROPOSAL_REASON_ROUTE_PATH_CANDIDATE_AUTHORED_COPPER_GRAPH_ZONE_AWARE
                    .to_string(),
                contract: report.contract.clone(),
                net_uuid: report.net_uuid,
                net_name: report.net_name.clone(),
                from_anchor_pad_uuid: report.from_anchor_pad_uuid,
                to_anchor_pad_uuid: report.to_anchor_pad_uuid,
                layer: step.layer,
                width_nm: net_class.track_width_nm,
                from: step.from,
                to: step.to,
                reused_via_uuid: None,
                reused_via_uuids: Vec::new(),
                reused_object_kind: Some(reused_object_kind.to_string()),
                reused_object_uuid: Some(step.object_uuid),
                reused_object_from_layer: step.from_layer,
                reused_object_to_layer: step.to_layer,
                selected_path_bend_count: 0,
                selected_path_point_count: path.steps.len() + 1,
                selected_path_segment_index,
                selected_path_segment_count,
                selected_path_layer_segment_index: None,
                selected_path_layer_segment_count: None,
                selected_path_layer_segment_bend_count: None,
                selected_path_layer_segment_point_count: None,
            }
        })
        .collect::<Vec<_>>();
    Ok(actions)
}

pub(crate) fn build_route_path_candidate_authored_copper_graph_zone_obstacle_aware_proposal_actions(
    board: &Board,
    net_uuid: Uuid,
    from_anchor_pad_uuid: Uuid,
    to_anchor_pad_uuid: Uuid,
) -> Result<Vec<RouteProposalAction>, String> {
    let report = board
        .route_path_candidate_authored_copper_graph_zone_obstacle_aware(
            net_uuid,
            from_anchor_pad_uuid,
            to_anchor_pad_uuid,
        )
        .map_err(|err| err.to_string())?;
    if report.status != RoutePathCandidateStatus::DeterministicPathFound {
        return Err(format!(
            "route proposal requires deterministic zone-obstacle-aware authored-copper path for net {} between {} and {}",
            net_uuid, from_anchor_pad_uuid, to_anchor_pad_uuid
        ));
    }

    let preflight = board
        .route_preflight(net_uuid)
        .ok_or_else(|| format!("board net not found in native project: {net_uuid}"))?;
    let net_class = preflight.persisted_constraints.net_class.ok_or_else(|| {
        format!(
            "route proposal requires persisted net-class facts for net {}",
            net_uuid
        )
    })?;
    let path = report.path.ok_or_else(|| {
        format!(
            "route proposal requires selected zone-obstacle-aware authored-copper path data for net {} between {} and {}",
            net_uuid,
            from_anchor_pad_uuid,
            to_anchor_pad_uuid
        )
    })?;
    let selected_path_segment_count = path.steps.len();
    let actions = path
        .steps
        .iter()
        .enumerate()
        .map(|(selected_path_segment_index, step)| {
            let reused_object_kind = match step.kind {
                RoutePathCandidateAuthoredCopperGraphZoneObstacleAwareStepKindView::Track => {
                    "track"
                }
                RoutePathCandidateAuthoredCopperGraphZoneObstacleAwareStepKindView::Via => "via",
                RoutePathCandidateAuthoredCopperGraphZoneObstacleAwareStepKindView::Zone => "zone",
            };
            let action_id = route_proposal_action_id(
                &report.contract,
                "reuse_existing_copper_step",
                ROUTE_PROPOSAL_REASON_ROUTE_PATH_CANDIDATE_AUTHORED_COPPER_GRAPH_ZONE_OBSTACLE_AWARE,
                net_uuid,
                from_anchor_pad_uuid,
                to_anchor_pad_uuid,
                step.layer,
                step.from,
                step.to,
                net_class.track_width_nm,
                None,
                &[],
                Some(reused_object_kind),
                Some(step.object_uuid),
                step.from_layer,
                step.to_layer,
            );
            RouteProposalAction {
                action_id,
                proposal_action: "reuse_existing_copper_step".to_string(),
                reason: ROUTE_PROPOSAL_REASON_ROUTE_PATH_CANDIDATE_AUTHORED_COPPER_GRAPH_ZONE_OBSTACLE_AWARE
                    .to_string(),
                contract: report.contract.clone(),
                net_uuid: report.net_uuid,
                net_name: report.net_name.clone(),
                from_anchor_pad_uuid: report.from_anchor_pad_uuid,
                to_anchor_pad_uuid: report.to_anchor_pad_uuid,
                layer: step.layer,
                width_nm: net_class.track_width_nm,
                from: step.from,
                to: step.to,
                reused_via_uuid: None,
                reused_via_uuids: Vec::new(),
                reused_object_kind: Some(reused_object_kind.to_string()),
                reused_object_uuid: Some(step.object_uuid),
                reused_object_from_layer: step.from_layer,
                reused_object_to_layer: step.to_layer,
                selected_path_bend_count: 0,
                selected_path_point_count: path.steps.len() + 1,
                selected_path_segment_index,
                selected_path_segment_count,
                selected_path_layer_segment_index: None,
                selected_path_layer_segment_count: None,
                selected_path_layer_segment_bend_count: None,
                selected_path_layer_segment_point_count: None,
            }
        })
        .collect::<Vec<_>>();
    Ok(actions)
}

pub(crate) fn build_route_path_candidate_authored_copper_graph_zone_obstacle_aware_topology_aware_proposal_actions(
    board: &Board,
    net_uuid: Uuid,
    from_anchor_pad_uuid: Uuid,
    to_anchor_pad_uuid: Uuid,
) -> Result<Vec<RouteProposalAction>, String> {
    let report = board
        .route_path_candidate_authored_copper_graph_zone_obstacle_aware_topology_aware(
            net_uuid,
            from_anchor_pad_uuid,
            to_anchor_pad_uuid,
        )
        .map_err(|err| err.to_string())?;
    if report.status != RoutePathCandidateStatus::DeterministicPathFound {
        return Err(format!(
            "route proposal requires deterministic topology-aware zone-obstacle-aware authored-copper path for net {} between {} and {}",
            net_uuid, from_anchor_pad_uuid, to_anchor_pad_uuid
        ));
    }

    let preflight = board
        .route_preflight(net_uuid)
        .ok_or_else(|| format!("board net not found in native project: {net_uuid}"))?;
    let net_class = preflight.persisted_constraints.net_class.ok_or_else(|| {
        format!(
            "route proposal requires persisted net-class facts for net {}",
            net_uuid
        )
    })?;
    let path = report.path.ok_or_else(|| {
        format!(
            "route proposal requires selected topology-aware zone-obstacle-aware authored-copper path data for net {} between {} and {}",
            net_uuid,
            from_anchor_pad_uuid,
            to_anchor_pad_uuid
        )
    })?;
    let selected_path_segment_count = path.steps.len();
    let actions = path
        .steps
        .iter()
        .enumerate()
        .map(|(selected_path_segment_index, step)| {
            let reused_object_kind = match step.kind {
                RoutePathCandidateAuthoredCopperGraphZoneObstacleAwareTopologyAwareStepKindView::Track => "track",
                RoutePathCandidateAuthoredCopperGraphZoneObstacleAwareTopologyAwareStepKindView::Via => "via",
                RoutePathCandidateAuthoredCopperGraphZoneObstacleAwareTopologyAwareStepKindView::Zone => "zone",
            };
            let action_id = route_proposal_action_id(
                &report.contract,
                "reuse_existing_copper_step",
                ROUTE_PROPOSAL_REASON_ROUTE_PATH_CANDIDATE_AUTHORED_COPPER_GRAPH_ZONE_OBSTACLE_AWARE_TOPOLOGY_AWARE,
                net_uuid,
                from_anchor_pad_uuid,
                to_anchor_pad_uuid,
                step.layer,
                step.from,
                step.to,
                net_class.track_width_nm,
                None,
                &[],
                Some(reused_object_kind),
                Some(step.object_uuid),
                step.from_layer,
                step.to_layer,
            );
            RouteProposalAction {
                action_id,
                proposal_action: "reuse_existing_copper_step".to_string(),
                reason: ROUTE_PROPOSAL_REASON_ROUTE_PATH_CANDIDATE_AUTHORED_COPPER_GRAPH_ZONE_OBSTACLE_AWARE_TOPOLOGY_AWARE
                    .to_string(),
                contract: report.contract.clone(),
                net_uuid: report.net_uuid,
                net_name: report.net_name.clone(),
                from_anchor_pad_uuid: report.from_anchor_pad_uuid,
                to_anchor_pad_uuid: report.to_anchor_pad_uuid,
                layer: step.layer,
                width_nm: net_class.track_width_nm,
                from: step.from,
                to: step.to,
                reused_via_uuid: None,
                reused_via_uuids: Vec::new(),
                reused_object_kind: Some(reused_object_kind.to_string()),
                reused_object_uuid: Some(step.object_uuid),
                reused_object_from_layer: step.from_layer,
                reused_object_to_layer: step.to_layer,
                selected_path_bend_count: 0,
                selected_path_point_count: path.steps.len() + 1,
                selected_path_segment_index,
                selected_path_segment_count,
                selected_path_layer_segment_index: None,
                selected_path_layer_segment_count: None,
                selected_path_layer_segment_bend_count: None,
                selected_path_layer_segment_point_count: None,
            }
        })
        .collect::<Vec<_>>();
    Ok(actions)
}

pub(crate) fn build_route_path_candidate_authored_copper_graph_zone_obstacle_aware_topology_aware_layer_balance_aware_proposal_actions(
    board: &Board,
    net_uuid: Uuid,
    from_anchor_pad_uuid: Uuid,
    to_anchor_pad_uuid: Uuid,
) -> Result<Vec<RouteProposalAction>, String> {
    let report = board
        .route_path_candidate_authored_copper_graph_zone_obstacle_aware_topology_aware_layer_balance_aware(net_uuid, from_anchor_pad_uuid, to_anchor_pad_uuid)
        .map_err(|err| err.to_string())?;
    if report.status != RoutePathCandidateStatus::DeterministicPathFound {
        return Err(format!(
            "route proposal requires deterministic layer-balance-aware topology-aware zone-obstacle-aware authored-copper path for net {} between {} and {}",
            net_uuid, from_anchor_pad_uuid, to_anchor_pad_uuid
        ));
    }

    let preflight = board
        .route_preflight(net_uuid)
        .ok_or_else(|| format!("board net not found in native project: {net_uuid}"))?;
    let net_class = preflight.persisted_constraints.net_class.ok_or_else(|| {
        format!(
            "route proposal requires persisted net-class facts for net {}",
            net_uuid
        )
    })?;
    let path = report.path.ok_or_else(|| {
        format!(
            "route proposal requires selected layer-balance-aware topology-aware zone-obstacle-aware authored-copper path data for net {} between {} and {}",
            net_uuid,
            from_anchor_pad_uuid,
            to_anchor_pad_uuid
        )
    })?;
    let selected_path_segment_count = path.steps.len();
    let actions = path
        .steps
        .iter()
        .enumerate()
        .map(|(selected_path_segment_index, step)| {
            let reused_object_kind = match step.kind {
                RoutePathCandidateAuthoredCopperGraphZoneObstacleAwareTopologyAwareLayerBalanceAwareStepKindView::Track => "track",
                RoutePathCandidateAuthoredCopperGraphZoneObstacleAwareTopologyAwareLayerBalanceAwareStepKindView::Via => "via",
                RoutePathCandidateAuthoredCopperGraphZoneObstacleAwareTopologyAwareLayerBalanceAwareStepKindView::Zone => "zone",
            };
            let action_id = route_proposal_action_id(
                &report.contract,
                "reuse_existing_copper_step",
                ROUTE_PROPOSAL_REASON_ROUTE_PATH_CANDIDATE_AUTHORED_COPPER_GRAPH_ZONE_OBSTACLE_AWARE_TOPOLOGY_AWARE_LAYER_BALANCE_AWARE,
                net_uuid,
                from_anchor_pad_uuid,
                to_anchor_pad_uuid,
                step.layer,
                step.from,
                step.to,
                net_class.track_width_nm,
                None,
                &[],
                Some(reused_object_kind),
                Some(step.object_uuid),
                step.from_layer,
                step.to_layer,
            );
            RouteProposalAction {
                action_id,
                proposal_action: "reuse_existing_copper_step".to_string(),
                reason: ROUTE_PROPOSAL_REASON_ROUTE_PATH_CANDIDATE_AUTHORED_COPPER_GRAPH_ZONE_OBSTACLE_AWARE_TOPOLOGY_AWARE_LAYER_BALANCE_AWARE
                    .to_string(),
                contract: report.contract.clone(),
                net_uuid: report.net_uuid,
                net_name: report.net_name.clone(),
                from_anchor_pad_uuid: report.from_anchor_pad_uuid,
                to_anchor_pad_uuid: report.to_anchor_pad_uuid,
                layer: step.layer,
                width_nm: net_class.track_width_nm,
                from: step.from,
                to: step.to,
                reused_via_uuid: None,
                reused_via_uuids: Vec::new(),
                reused_object_kind: Some(reused_object_kind.to_string()),
                reused_object_uuid: Some(step.object_uuid),
                reused_object_from_layer: step.from_layer,
                reused_object_to_layer: step.to_layer,
                selected_path_bend_count: 0,
                selected_path_point_count: path.steps.len() + 1,
                selected_path_segment_index,
                selected_path_segment_count,
                selected_path_layer_segment_index: None,
                selected_path_layer_segment_count: None,
                selected_path_layer_segment_bend_count: None,
                selected_path_layer_segment_point_count: None,
            }
        })
        .collect::<Vec<_>>();
    Ok(actions)
}

pub(crate) fn build_route_path_candidate_authored_copper_graph_obstacle_aware_proposal_actions(
    board: &Board,
    net_uuid: Uuid,
    from_anchor_pad_uuid: Uuid,
    to_anchor_pad_uuid: Uuid,
) -> Result<Vec<RouteProposalAction>, String> {
    let report = board
        .route_path_candidate_authored_copper_graph_obstacle_aware(
            net_uuid,
            from_anchor_pad_uuid,
            to_anchor_pad_uuid,
        )
        .map_err(|err| err.to_string())?;
    if report.status != RoutePathCandidateStatus::DeterministicPathFound {
        return Err(format!(
            "route proposal requires deterministic obstacle-aware authored-copper path for net {} between {} and {}",
            net_uuid, from_anchor_pad_uuid, to_anchor_pad_uuid
        ));
    }

    let preflight = board
        .route_preflight(net_uuid)
        .ok_or_else(|| format!("board net not found in native project: {net_uuid}"))?;
    let net_class = preflight.persisted_constraints.net_class.ok_or_else(|| {
        format!(
            "route proposal requires persisted net-class facts for net {}",
            net_uuid
        )
    })?;
    let path = report.path.ok_or_else(|| {
        format!(
            "route proposal requires selected obstacle-aware authored-copper path data for net {} between {} and {}",
            net_uuid,
            from_anchor_pad_uuid,
            to_anchor_pad_uuid
        )
    })?;
    let selected_path_segment_count = path.steps.len();
    let actions = path
        .steps
        .iter()
        .enumerate()
        .map(|(selected_path_segment_index, step)| {
            let reused_object_kind = match step.kind {
                RoutePathCandidateAuthoredCopperGraphObstacleAwareStepKindView::Track => "track",
                RoutePathCandidateAuthoredCopperGraphObstacleAwareStepKindView::Via => "via",
            };
            let action_id = route_proposal_action_id(
                &report.contract,
                "reuse_existing_copper_step",
                ROUTE_PROPOSAL_REASON_ROUTE_PATH_CANDIDATE_AUTHORED_COPPER_GRAPH_OBSTACLE_AWARE,
                net_uuid,
                from_anchor_pad_uuid,
                to_anchor_pad_uuid,
                step.layer,
                step.from,
                step.to,
                net_class.track_width_nm,
                None,
                &[],
                Some(reused_object_kind),
                Some(step.object_uuid),
                step.from_layer,
                step.to_layer,
            );
            RouteProposalAction {
                action_id,
                proposal_action: "reuse_existing_copper_step".to_string(),
                reason:
                    ROUTE_PROPOSAL_REASON_ROUTE_PATH_CANDIDATE_AUTHORED_COPPER_GRAPH_OBSTACLE_AWARE
                        .to_string(),
                contract: report.contract.clone(),
                net_uuid: report.net_uuid,
                net_name: report.net_name.clone(),
                from_anchor_pad_uuid: report.from_anchor_pad_uuid,
                to_anchor_pad_uuid: report.to_anchor_pad_uuid,
                layer: step.layer,
                width_nm: net_class.track_width_nm,
                from: step.from,
                to: step.to,
                reused_via_uuid: None,
                reused_via_uuids: Vec::new(),
                reused_object_kind: Some(reused_object_kind.to_string()),
                reused_object_uuid: Some(step.object_uuid),
                reused_object_from_layer: step.from_layer,
                reused_object_to_layer: step.to_layer,
                selected_path_bend_count: 0,
                selected_path_point_count: path.steps.len() + 1,
                selected_path_segment_index,
                selected_path_segment_count,
                selected_path_layer_segment_index: None,
                selected_path_layer_segment_count: None,
                selected_path_layer_segment_bend_count: None,
                selected_path_layer_segment_point_count: None,
            }
        })
        .collect::<Vec<_>>();
    Ok(actions)
}

pub(crate) fn build_route_path_candidate_authored_copper_graph_policy_proposal_actions(
    board: &Board,
    net_uuid: Uuid,
    from_anchor_pad_uuid: Uuid,
    to_anchor_pad_uuid: Uuid,
    policy: RoutePathCandidateAuthoredCopperGraphPolicy,
) -> Result<Vec<RouteProposalAction>, String> {
    let report = board
        .route_path_candidate_authored_copper_graph_by_policy(
            net_uuid,
            from_anchor_pad_uuid,
            to_anchor_pad_uuid,
            policy,
        )
        .map_err(|err| err.to_string())?;
    if report.status != RoutePathCandidateStatus::DeterministicPathFound {
        return Err(format!(
            "route proposal requires deterministic authored-copper graph path for net {} between {} and {} under policy {:?}",
            net_uuid, from_anchor_pad_uuid, to_anchor_pad_uuid, policy
        ));
    }

    let preflight = board
        .route_preflight(net_uuid)
        .ok_or_else(|| format!("board net not found in native project: {net_uuid}"))?;
    let net_class = preflight.persisted_constraints.net_class.ok_or_else(|| {
        format!(
            "route proposal requires persisted net-class facts for net {}",
            net_uuid
        )
    })?;
    let path = report.path.ok_or_else(|| {
        format!(
            "route proposal requires selected authored-copper graph path data for net {} between {} and {} under policy {:?}",
            net_uuid,
            from_anchor_pad_uuid,
            to_anchor_pad_uuid,
            policy
        )
    })?;
    let reason = route_path_candidate_authored_copper_graph_policy_reason(policy);
    let selected_path_segment_count = path.steps.len();
    let actions = path
        .steps
        .iter()
        .enumerate()
        .map(|(selected_path_segment_index, step)| {
            let reused_object_kind = match step.kind {
                RoutePathCandidateAuthoredCopperGraphPolicyStepKindView::Track => "track",
                RoutePathCandidateAuthoredCopperGraphPolicyStepKindView::Via => "via",
                RoutePathCandidateAuthoredCopperGraphPolicyStepKindView::Zone => "zone",
            };
            let action_id = route_proposal_action_id(
                &report.contract,
                "reuse_existing_copper_step",
                reason,
                net_uuid,
                from_anchor_pad_uuid,
                to_anchor_pad_uuid,
                step.layer,
                step.from,
                step.to,
                net_class.track_width_nm,
                None,
                &[],
                Some(reused_object_kind),
                Some(step.object_uuid),
                step.from_layer,
                step.to_layer,
            );
            RouteProposalAction {
                action_id,
                proposal_action: "reuse_existing_copper_step".to_string(),
                reason: reason.to_string(),
                contract: report.contract.clone(),
                net_uuid: report.net_uuid,
                net_name: report.net_name.clone(),
                from_anchor_pad_uuid: report.from_anchor_pad_uuid,
                to_anchor_pad_uuid: report.to_anchor_pad_uuid,
                layer: step.layer,
                width_nm: net_class.track_width_nm,
                from: step.from,
                to: step.to,
                reused_via_uuid: None,
                reused_via_uuids: Vec::new(),
                reused_object_kind: Some(reused_object_kind.to_string()),
                reused_object_uuid: Some(step.object_uuid),
                reused_object_from_layer: step.from_layer,
                reused_object_to_layer: step.to_layer,
                selected_path_bend_count: 0,
                selected_path_point_count: path.steps.len() + 1,
                selected_path_segment_index,
                selected_path_segment_count,
                selected_path_layer_segment_index: None,
                selected_path_layer_segment_count: None,
                selected_path_layer_segment_bend_count: None,
                selected_path_layer_segment_point_count: None,
            }
        })
        .collect::<Vec<_>>();
    Ok(actions)
}

pub(crate) fn route_path_candidate_authored_copper_graph_policy_reason(
    policy: RoutePathCandidateAuthoredCopperGraphPolicy,
) -> &'static str {
    match policy {
        RoutePathCandidateAuthoredCopperGraphPolicy::Plain => {
            ROUTE_PROPOSAL_REASON_ROUTE_PATH_CANDIDATE_AUTHORED_COPPER_GRAPH_POLICY_PLAIN
        }
        RoutePathCandidateAuthoredCopperGraphPolicy::ZoneAware => {
            ROUTE_PROPOSAL_REASON_ROUTE_PATH_CANDIDATE_AUTHORED_COPPER_GRAPH_POLICY_ZONE_AWARE
        }
        RoutePathCandidateAuthoredCopperGraphPolicy::ObstacleAware => {
            ROUTE_PROPOSAL_REASON_ROUTE_PATH_CANDIDATE_AUTHORED_COPPER_GRAPH_POLICY_OBSTACLE_AWARE
        }
        RoutePathCandidateAuthoredCopperGraphPolicy::ZoneObstacleAware => {
            ROUTE_PROPOSAL_REASON_ROUTE_PATH_CANDIDATE_AUTHORED_COPPER_GRAPH_POLICY_ZONE_OBSTACLE_AWARE
        }
        RoutePathCandidateAuthoredCopperGraphPolicy::ZoneObstacleTopologyAware => {
            ROUTE_PROPOSAL_REASON_ROUTE_PATH_CANDIDATE_AUTHORED_COPPER_GRAPH_POLICY_ZONE_OBSTACLE_TOPOLOGY_AWARE
        }
        RoutePathCandidateAuthoredCopperGraphPolicy::ZoneObstacleTopologyLayerBalanceAware => {
            ROUTE_PROPOSAL_REASON_ROUTE_PATH_CANDIDATE_AUTHORED_COPPER_GRAPH_POLICY_ZONE_OBSTACLE_TOPOLOGY_LAYER_BALANCE_AWARE
        }
    }
}

pub(crate) fn route_path_candidate_authored_copper_graph_policy_from_reason(
    reason: &str,
) -> Option<RoutePathCandidateAuthoredCopperGraphPolicy> {
    match reason {
        ROUTE_PROPOSAL_REASON_ROUTE_PATH_CANDIDATE_AUTHORED_COPPER_GRAPH_POLICY_PLAIN => {
            Some(RoutePathCandidateAuthoredCopperGraphPolicy::Plain)
        }
        ROUTE_PROPOSAL_REASON_ROUTE_PATH_CANDIDATE_AUTHORED_COPPER_GRAPH_POLICY_ZONE_AWARE => {
            Some(RoutePathCandidateAuthoredCopperGraphPolicy::ZoneAware)
        }
        ROUTE_PROPOSAL_REASON_ROUTE_PATH_CANDIDATE_AUTHORED_COPPER_GRAPH_POLICY_OBSTACLE_AWARE => {
            Some(RoutePathCandidateAuthoredCopperGraphPolicy::ObstacleAware)
        }
        ROUTE_PROPOSAL_REASON_ROUTE_PATH_CANDIDATE_AUTHORED_COPPER_GRAPH_POLICY_ZONE_OBSTACLE_AWARE => {
            Some(RoutePathCandidateAuthoredCopperGraphPolicy::ZoneObstacleAware)
        }
        ROUTE_PROPOSAL_REASON_ROUTE_PATH_CANDIDATE_AUTHORED_COPPER_GRAPH_POLICY_ZONE_OBSTACLE_TOPOLOGY_AWARE => {
            Some(RoutePathCandidateAuthoredCopperGraphPolicy::ZoneObstacleTopologyAware)
        }
        ROUTE_PROPOSAL_REASON_ROUTE_PATH_CANDIDATE_AUTHORED_COPPER_GRAPH_POLICY_ZONE_OBSTACLE_TOPOLOGY_LAYER_BALANCE_AWARE => {
            Some(RoutePathCandidateAuthoredCopperGraphPolicy::ZoneObstacleTopologyLayerBalanceAware)
        }
        _ => None,
    }
}

// Routing selection threads many path-candidate/via/layer parameters.
#[allow(clippy::too_many_arguments)]
pub(crate) fn build_segmented_route_proposal_actions(
    contract: &str,
    reason: &str,
    net_uuid: Uuid,
    net_name: &str,
    from_anchor_pad_uuid: Uuid,
    to_anchor_pad_uuid: Uuid,
    width_nm: i64,
    segments: Vec<(i32, &[Point])>,
    reused_via_uuids: &[Uuid],
) -> Vec<RouteProposalAction> {
    let selected_path_segment_count = segments.len();
    let primary_reused_via_uuid = reused_via_uuids.first().copied();
    segments
        .into_iter()
        .enumerate()
        .flat_map(|(selected_path_segment_index, (layer, points))| {
            points.windows(2).map(move |pair| {
                (
                    selected_path_segment_index,
                    layer,
                    points.len(),
                    pair[0],
                    pair[1],
                )
            })
        })
        .map(
            |(selected_path_segment_index, layer, selected_path_point_count, from, to)| {
                let action_id = route_proposal_action_id(
                    contract,
                    "draw_track",
                    reason,
                    net_uuid,
                    from_anchor_pad_uuid,
                    to_anchor_pad_uuid,
                    layer,
                    from,
                    to,
                    width_nm,
                    primary_reused_via_uuid,
                    reused_via_uuids,
                    None,
                    None,
                    None,
                    None,
                );
                RouteProposalAction {
                    action_id,
                    proposal_action: "draw_track".to_string(),
                    reason: reason.to_string(),
                    contract: contract.to_string(),
                    net_uuid,
                    net_name: net_name.to_string(),
                    from_anchor_pad_uuid,
                    to_anchor_pad_uuid,
                    layer,
                    width_nm,
                    from,
                    to,
                    reused_via_uuid: primary_reused_via_uuid,
                    reused_via_uuids: reused_via_uuids.to_vec(),
                    reused_object_kind: None,
                    reused_object_uuid: None,
                    reused_object_from_layer: None,
                    reused_object_to_layer: None,
                    selected_path_bend_count: 0,
                    selected_path_point_count,
                    selected_path_segment_index,
                    selected_path_segment_count,
                    selected_path_layer_segment_index: None,
                    selected_path_layer_segment_count: None,
                    selected_path_layer_segment_bend_count: None,
                    selected_path_layer_segment_point_count: None,
                }
            },
        )
        .collect()
}

// Routing selection threads many path-candidate/via/layer parameters.
#[allow(clippy::too_many_arguments)]
fn route_proposal_action_id(
    contract: &str,
    proposal_action: &str,
    reason: &str,
    net_uuid: Uuid,
    from_anchor_pad_uuid: Uuid,
    to_anchor_pad_uuid: Uuid,
    layer: i32,
    from: Point,
    to: Point,
    width_nm: i64,
    reused_via_uuid: Option<Uuid>,
    reused_via_uuids: &[Uuid],
    reused_object_kind: Option<&str>,
    reused_object_uuid: Option<Uuid>,
    reused_object_from_layer: Option<i32>,
    reused_object_to_layer: Option<i32>,
) -> String {
    let reused_via_uuid_sequence = reused_via_uuids
        .iter()
        .map(Uuid::to_string)
        .collect::<Vec<_>>()
        .join(",");
    let reused_object_kind = reused_object_kind.unwrap_or_default();
    let stable_key = format!(
        "{contract}|{proposal_action}|{reason}|{net_uuid}|{from_anchor_pad_uuid}|{to_anchor_pad_uuid}|{layer}|{}:{}|{}:{}|{width_nm}|{}|{reused_via_uuid_sequence}|{reused_object_kind}|{}|{}|{}",
        from.x,
        from.y,
        to.x,
        to.y,
        reused_via_uuid
            .map(|uuid| uuid.to_string())
            .unwrap_or_default(),
        reused_object_uuid
            .map(|uuid| uuid.to_string())
            .unwrap_or_default(),
        reused_object_from_layer
            .map(|layer| layer.to_string())
            .unwrap_or_default(),
        reused_object_to_layer
            .map(|layer| layer.to_string())
            .unwrap_or_default(),
    );
    compute_source_hash_bytes(stable_key.as_bytes())
}
