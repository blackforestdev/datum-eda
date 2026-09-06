use super::*;
use datum_gui_protocol::{PaneId, WorkspaceLayout, load_fixture_workspace_state};

fn label_color(state: &ReviewWorkspaceState, label: &str) -> [f32; 3] {
    let retained = RetainedScene::from_workspace(state, 1280, 768);
    let prepared = PreparedScene::from_workspace(
        state,
        1280,
        768,
        CameraState::fit_to_bounds(&state.scene.bounds),
        &retained,
    );
    assert!(
        prepared.hit_regions.iter().any(|region| {
            matches!(&region.target, HitTarget::MenuItem { label: row, .. } if row == label)
        }),
        "unavailable rows must remain inspectable by pointer"
    );
    prepared
        .menu_overlay_text_runs()
        .iter()
        .find(|run| run.text == label)
        .unwrap()
        .color
}

#[test]
fn rendered_fit_readiness_tracks_context_and_missing_rows_stay_inspectable() {
    let mut state = load_fixture_workspace_state();
    state.ui.layout = WorkspaceLayout::board_schematic();
    state.schematic_scene = None;
    state.ui.active_menu = Some("View".to_owned());
    assert_eq!(label_color(&state, "Fit to Board"), TEXT_PRIMARY);
    assert_eq!(label_color(&state, "Layer Visibility"), TEXT_MUTED);
    state.ui.layout.focused = PaneId(1);
    assert_eq!(label_color(&state, "Fit to Board"), TEXT_MUTED);
    state.ui.layout.focused = PaneId(0);
    assert_eq!(label_color(&state, "Fit to Board"), TEXT_PRIMARY);
    state.ui.active_menu = Some("Help".to_owned());
    assert_eq!(label_color(&state, "About Datum"), TEXT_MUTED);
    state.ui.active_menu = Some("Window".to_owned());
    assert_eq!(label_color(&state, "Documents"), TEXT_MUTED);
}
