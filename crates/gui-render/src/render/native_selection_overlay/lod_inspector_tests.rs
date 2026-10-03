//! The Inspector consumes final presentation state, never a membership substitute.
use super::tests::selected_track_state;
use super::*;
fn prepared(state: &ReviewWorkspaceState, zoom: f32, scale: f32) -> PreparedScene {
    let retained = RetainedScene::from_workspace(state, 1280, 800);
    let mut camera = CameraState::fit_to_bounds(&state.scene.bounds);
    camera.zoom = zoom;
    camera.center_x_nm = 500_000.0;
    camera.center_y_nm = 0.0;
    let mut scene = PreparedScene::from_workspace_with_terminal_renderer(
        state,
        (1280.0 * scale) as u32,
        (800.0 * scale) as u32,
        scale,
        camera,
        &retained,
        &[],
        None,
        false,
    )
    .unwrap();
    scene.append_native_selection(state).unwrap();
    scene
}
#[test]
fn inspector_reports_final_mask_and_minimum_cues_without_changing_complete_subject() {
    let (fixture, state) = selected_track_state(1_000_000);
    let source = fixture.source_bytes();
    let subject = state.selection.clone();
    for scale in [1.0, 1.25, 2.0] {
        for (zoom, minimum) in [(8.0, 0), (0.00001, 1)] {
            let scene = prepared(&state, zoom, scale);
            let (rect, factor) = scene.selection_lod_row.expect("visible LOD row");
            assert_eq!(factor, scale);
            let label = scene
                .text_runs
                .iter()
                .find(|r| r.text.starts_with("Exact mask"))
                .expect("camera-qualified summary");
            assert!(
                label.text.contains(&format!("{minimum} minimum")),
                "{} at zoom {zoom} scale {scale}",
                label.text
            );
            assert_eq!(label.clip_bounds, Some(rect));
            assert_eq!(label.size, 11.0 * scale);
            assert_eq!(scene.selection_masks.len(), 1);
        }
    }
    assert_eq!(state.selection, subject);
    assert_eq!(fixture.source_bytes(), source);
}
#[test]
fn stale_projection_and_clipped_inspector_do_not_report_a_fabricated_visual_state() {
    let (fixture, mut state) = selected_track_state(1_000_000);
    let source = fixture.source_bytes();
    if let SelectionTarget::Native(view) = &mut state.selection {
        view.stale = true;
    }
    let scene = prepared(&state, 1.0, 1.0);
    assert!(scene.selection_masks.is_empty());
    assert!(
        scene
            .text_runs
            .iter()
            .any(|r| r.text == "Visual LOD: unavailable projection")
    );
    state.ui.selection_inspector.inventory_expanded = true;
    state.ui.selection_inspector.filters_expanded = true;
    state.ui.selection_inspector.scroll_offset = usize::MAX;
    let scrolled = prepared(&state, 1.0, 1.0);
    assert!(scrolled.selection_lod_row.is_none());
    assert!(
        !scrolled
            .text_runs
            .iter()
            .any(|r| r.text.starts_with("Visual LOD:"))
    );
    let SelectionTarget::Native(view) = &state.selection else {
        unreachable!()
    };
    assert_eq!(view.members.len(), 1);
    assert_eq!(fixture.source_bytes(), source);
}
