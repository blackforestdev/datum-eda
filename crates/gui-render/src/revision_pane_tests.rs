use super::*;
use datum_gui_protocol::{
    PaneContent, RevisionNavEntry, RevisionPane, RevisionSurface, SplitOrientation,
};

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
    state.ui.layout.open_beside_root(
        PaneContent::Revision(RevisionPane::Surface(RevisionSurface::Impact)),
        SplitOrientation::Vertical,
        0.66,
        true,
    );
    state.ui.revision.selected_entry = Some(RevisionNavEntry::Baselines);
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
    let panes = prepared.layout.viewport_panes(&state.ui.layout);
    assert!(
        panes
            .panes
            .iter()
            .all(|pane| pane.rect.frame.width >= 280.0),
        "root opening must keep every wide-layout pane usable: {:?}",
        panes
            .panes
            .iter()
            .map(|pane| pane.rect.frame.width)
            .collect::<Vec<_>>()
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

#[test]
fn every_revision_panel_text_quad_and_hit_is_clipped_to_its_pane() {
    let mut state = datum_gui_protocol::load_fixture_workspace_state();
    let rect = RectPx {
        x: 410.0,
        y: 80.0,
        width: 96.0,
        height: 360.0,
    };
    for surface in [
        RevisionSurface::Impact,
        RevisionSurface::Release,
        RevisionSurface::Change,
        RevisionSurface::Evidence,
    ] {
        let mut quads = Vec::new();
        let mut text = Vec::new();
        let mut hits = Vec::new();
        revision_workspace::render_pane(
            &state,
            RevisionPane::Surface(surface),
            rect,
            &mut quads,
            &mut text,
            &mut hits,
        );
        assert!(!text.is_empty(), "{surface:?} should render text");
        for quad in quads {
            assert!(
                quad.points.iter().all(|(x, y)| {
                    *x >= rect.x
                        && *x <= rect.x + rect.width
                        && *y >= rect.y
                        && *y <= rect.y + rect.height
                }),
                "{surface:?} quad escaped pane: {:?}",
                quad.points
            );
        }
        for run in text {
            let clip = run
                .clip_bounds
                .expect("every revision text pass must be clipped");
            assert!(clip.x >= rect.x && clip.y >= rect.y);
            assert!(clip.x + clip.width <= rect.x + rect.width);
            assert!(clip.y + clip.height <= rect.y + rect.height);
        }
        assert!(hits.iter().all(|hit| {
            hit.rect.x >= rect.x
                && hit.rect.y >= rect.y
                && hit.rect.x + hit.rect.width <= rect.x + rect.width
                && hit.rect.y + hit.rect.height <= rect.y + rect.height
        }));
    }

    state.ui.revision.selected_witness = Some("J3/U7/output".to_owned());
    let mut quads = Vec::new();
    let mut text = Vec::new();
    let mut hits = Vec::new();
    revision_workspace::render_pane(
        &state,
        RevisionPane::Witness,
        rect,
        &mut quads,
        &mut text,
        &mut hits,
    );
    assert!(text.iter().all(|run| run.clip_bounds.is_some()));
}

#[test]
fn navigator_is_single_select_and_exposes_the_local_entry_menu() {
    let mut state = datum_gui_protocol::load_fixture_workspace_state();
    state.ui.revision.selected_entry = Some(RevisionNavEntry::Evidence);
    let rect = RectPx {
        x: 0.0,
        y: 40.0,
        width: 250.0,
        height: 340.0,
    };
    let mut quads = Vec::new();
    let mut text = Vec::new();
    let mut hits = Vec::new();
    revision_workspace::render_navigator(&state, rect, &mut quads, &mut text, &mut hits);
    let entries = hits
        .iter()
        .filter_map(|hit| match hit.target {
            HitTarget::SelectRevisionNavEntry(entry) => Some(entry),
            _ => None,
        })
        .collect::<Vec<_>>();
    assert_eq!(entries, RevisionNavEntry::ALL);
    assert_eq!(
        state.ui.revision.selected_entry,
        Some(RevisionNavEntry::Evidence)
    );

    state.ui.revision.context_menu = Some(datum_gui_protocol::RevisionNavContextMenu {
        entry: RevisionNavEntry::Evidence,
        anchor_x_px: 60,
        anchor_y_px: 120,
    });
    revision_workspace::render_navigator_context_menu(
        &state,
        RectPx {
            x: 0.0,
            y: 0.0,
            width: 800.0,
            height: 600.0,
        },
        &mut quads,
        &mut text,
        &mut hits,
    );
    for label in [
        "Open Beside",
        "Properties",
        "Show Impact",
        "Prepare Revision",
        "Compare to Baseline",
    ] {
        assert!(text.iter().any(|run| run.text == label), "missing {label}");
    }
}
