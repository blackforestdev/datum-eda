use super::*;

pub(crate) fn place_native_project_board_track(
    root: &Path,
    net_uuid: Uuid,
    from: Point,
    to: Point,
    width_nm: i64,
    layer: i32,
) -> Result<NativeProjectBoardTrackMutationReportView> {
    let project = load_native_project_with_resolved_board(root)?;
    if !project.board.nets.contains_key(&net_uuid.to_string()) {
        bail!("board net not found in native project: {net_uuid}");
    }
    let track_uuid = Uuid::new_v4();
    let track = Track::straight(track_uuid, net_uuid, from, to, width_nm, layer);
    commit_board_routing_write(root, "draw board track", |model, provenance| {
        build_place_board_track(model, provenance, &track)
    })?;
    let project = load_native_project_with_resolved_board(root)?;
    Ok(native_project_board_track_report(
        "draw_board_track",
        &project,
        track,
    ))
}
