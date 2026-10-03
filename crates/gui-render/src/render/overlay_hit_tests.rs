use super::*;
#[test]
fn hit_testing_prefers_overlay_over_underlying_authored_geometry() {
    let state = datum_gui_protocol::load_fixture_workspace_state();
    let retained = RetainedScene::from_workspace(&state, 1280, 800);
    let prepared = PreparedScene::from_workspace(
        &state,
        1280,
        800,
        CameraState::fit_to_bounds(&state.scene.bounds),
        &retained,
    )
    .unwrap();
    let overlay_rect = prepared
        .hit_regions
        .iter()
        .rev()
        .find_map(|region| match &region.target {
            HitTarget::ProposalAction(id) if id == "action-1" => Some(region.rect),
            _ => None,
        })
        .expect("action overlay hit region should exist");
    let hit = prepared
        .hit_test(
            overlay_rect.x + overlay_rect.width / 2.0,
            overlay_rect.y + overlay_rect.height / 2.0,
        )
        .expect("topmost hit should exist");
    assert_eq!(hit, &HitTarget::ProposalAction("action-1".to_string()));
}
