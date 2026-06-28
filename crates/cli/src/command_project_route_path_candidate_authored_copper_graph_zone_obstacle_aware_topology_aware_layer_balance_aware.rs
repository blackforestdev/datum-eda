use std::path::Path;

use anyhow::{Result, anyhow};
use eda_engine::board::RoutePathCandidateAuthoredCopperGraphZoneObstacleAwareTopologyAwareLayerBalanceAwareReport;
use uuid::Uuid;

use super::super::{build_native_project_board, load_native_project_with_resolved_board};

pub(crate) fn query_native_project_route_path_candidate_authored_copper_graph_zone_obstacle_aware_topology_aware_layer_balance_aware(
    root: &Path,
    net_uuid: Uuid,
    from_anchor_pad_uuid: Uuid,
    to_anchor_pad_uuid: Uuid,
) -> Result<
    RoutePathCandidateAuthoredCopperGraphZoneObstacleAwareTopologyAwareLayerBalanceAwareReport,
> {
    let project = load_native_project_with_resolved_board(root)?;
    let board = build_native_project_board(&project)?;
    board
        .route_path_candidate_authored_copper_graph_zone_obstacle_aware_topology_aware_layer_balance_aware(
            net_uuid,
            from_anchor_pad_uuid,
            to_anchor_pad_uuid,
        )
        .map_err(|err| anyhow!(err))
}
