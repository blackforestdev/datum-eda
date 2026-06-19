use std::path::Path;

use anyhow::{Result, anyhow};
use eda_engine::board::RoutePathCandidateAuthoredCopperGraphZoneAwareReport;
use uuid::Uuid;

use super::super::{build_native_project_board, load_native_project};

pub(crate) fn query_native_project_route_path_candidate_authored_copper_graph_zone_aware(
    root: &Path,
    net_uuid: Uuid,
    from_anchor_pad_uuid: Uuid,
    to_anchor_pad_uuid: Uuid,
) -> Result<RoutePathCandidateAuthoredCopperGraphZoneAwareReport> {
    let project = load_native_project(root)?;
    let board = build_native_project_board(&project)?;
    board
        .route_path_candidate_authored_copper_graph_zone_aware(
            net_uuid,
            from_anchor_pad_uuid,
            to_anchor_pad_uuid,
        )
        .map_err(|err| anyhow!(err))
}
