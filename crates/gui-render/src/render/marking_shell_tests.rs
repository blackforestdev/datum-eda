use super::*;
#[test]
fn marking_menu_shell_renders_manifest_items_without_command_targets() {
    let mut state = datum_gui_protocol::load_fixture_workspace_state();
    state.ui.marking_menu = Some(datum_gui_protocol::MarkingMenuState {
        menu_key: "pcb.component".to_string(),
        target_object_id: Some("component:demo".to_string()),
        anchor_x_px: 640,
        anchor_y_px: 360,
        preview_slot: Some("N".to_string()),
        gesture_dx_px: 0,
        gesture_dy_px: -72,
        selection: None,
    });
    let retained = RetainedScene::from_workspace(&state, 1280, 768);
    let prepared = PreparedScene::from_workspace(
        &state,
        1280,
        768,
        CameraState::fit_to_bounds(&state.scene.bounds),
        &retained,
    )
    .unwrap();
    let rendered_text = prepared
        .text_runs
        .iter()
        .map(|run| run.text.as_str())
        .collect::<Vec<_>>();

    assert!(rendered_text.contains(&"Rotate"));
    assert!(rendered_text.contains(&"Delete"));
    assert!(rendered_text.contains(&"pcb.component"));
    assert!(prepared.hit_regions.iter().any(|region| matches!(
        &region.target,
        HitTarget::MarkingMenuItem {
            menu_key,
            slot,
            label
        } if menu_key == "pcb.component" && slot == "N" && label == "Rotate"
    )));
    assert!(
        prepared.hit_regions.iter().all(|region| {
            !matches!(
                region.target,
                HitTarget::ProductionTerminalCommand(_) | HitTarget::ProductionOutputJobRun(_)
            )
        }),
        "inert marking-menu shell must not expose terminal command hit targets"
    );
}
