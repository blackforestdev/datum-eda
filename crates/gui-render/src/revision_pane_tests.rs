use super::*;
use datum_gui_protocol::{PaneContent, RevisionPane, RevisionSurface, SplitOrientation};

#[test]
fn shell_layout_reserves_bottom_dock_and_viewport() {
    let layout = ShellLayout::for_window(1280, 800, None);
    assert!(layout.viewport.width > 0.0);
    assert_eq!(layout.bottom_strip.height, design_tokens::spacing::SP_07);
    assert!(layout.left_sidebar.width > 0.0);
    assert!(layout.right_sidebar.width > 0.0);
}

#[test]
fn revision_pane_keeps_design_surfaces_board_hits_and_witness_summary_live() {
    let schematic = std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("../engine/testdata/import/kicad/simple-demo.kicad_sch");
    let projected = datum_gui_protocol::load_kicad_schematic_workspace_state(&schematic)
        .expect("simple schematic fixture should load");
    let mut state = datum_gui_protocol::load_fixture_workspace_state();
    state.schematic_scene = Some(projected.scene);
    state.ui.layout.open_beside(
        PaneContent::Revision(RevisionPane::Surface(RevisionSurface::Impact)),
        SplitOrientation::Vertical,
        true,
    );
    let retained = RetainedScene::from_workspace(&state, 1600, 1000);
    let prepared = PreparedScene::from_workspace(
        &state,
        1600,
        1000,
        CameraState::fit_to_bounds(&state.scene.bounds),
        &retained,
    );

    assert!(
        prepared
            .surface_passes
            .iter()
            .any(|pass| pass.surface == SceneSurface::Board)
    );
    assert!(
        prepared
            .surface_passes
            .iter()
            .any(|pass| pass.surface == SceneSurface::Schematic)
    );
    assert!(prepared.hit_regions.iter().any(|region| {
        matches!(
            region.target,
            HitTarget::AuthoredObject(_) | HitTarget::ReviewAction(_)
        )
    }));
    assert!(
        prepared
            .hit_regions
            .iter()
            .any(|region| { matches!(region.target, HitTarget::CloseRevisionSurface) })
    );

    state.ui.revision.selected_witness = Some("J3/U7/output".to_owned());
    state.ui.layout.open_beside(
        PaneContent::Revision(RevisionPane::Witness),
        SplitOrientation::Vertical,
        true,
    );
    let prepared = PreparedScene::from_workspace(
        &state,
        1600,
        1000,
        CameraState::fit_to_bounds(&state.scene.bounds),
        &retained,
    );
    assert!(
        prepared
            .text_runs
            .iter()
            .any(|run| run.text == "Impact · CHG-0031")
    );
    assert!(
        prepared
            .text_runs
            .iter()
            .any(|run| run.text == "Canonical witness")
    );
    assert!(
        prepared
            .surface_passes
            .iter()
            .any(|pass| pass.surface == SceneSurface::Board)
    );
}
