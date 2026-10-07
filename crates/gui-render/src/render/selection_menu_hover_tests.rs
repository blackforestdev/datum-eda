use super::*;
use crate::interaction_overlay::{
    board_hover_bounds, capture_menu_hover as capture, schematic_hover_bounds,
};
use crate::{CameraState, PreparedScene};
use datum_gui_protocol::{
    PaneContent, PointNm, SelectionTarget, selection_menu::SelectionMenu,
    selection_resolution::NativeSelectionResolution, selection_scene::SceneSelectionBindings,
};

fn open_menu(state: &mut ReviewWorkspaceState, root: &std::path::Path, schematic: bool) {
    let model = eda_engine::substrate::ProjectResolver::new(root)
        .resolve()
        .unwrap();
    let authority = NativeSelectionResolution::capture(&model).unwrap();
    let SelectionTarget::Native(view) = &state.selection else {
        panic!("native fixture subject");
    };
    let id = view.members.first().unwrap().clone();
    let scene = if schematic {
        state.schematic_scene.as_ref().unwrap()
    } else {
        &state.scene
    };
    let bindings = SceneSelectionBindings::capture(scene, &authority);
    let mut menu = SelectionMenu::capture(
        &authority,
        vec![(id.clone(), id)],
        PointNm { x: 0, y: 0 },
        state.ui.filters.clone(),
    );
    menu.schematic = schematic;
    menu.source_scene = scene.scene_id.clone();
    menu.bind_presentation(scene, &bindings, &authority);
    assert!(!menu.preview_keys.is_empty());
    menu.open = true;
    menu.preview = Some(0);
    state.ui.marking_menu = Some(datum_gui_protocol::MarkingMenuState {
        menu_key: "selection-proof".into(),
        target_object_id: None,
        anchor_x_px: 0,
        anchor_y_px: 0,
        preview_slot: None,
        gesture_dx_px: 0,
        gesture_dy_px: 0,
        selection: Some(menu),
    });
}

#[test]
fn native_conflict_candidate_previews_source_geometry_without_changing_selection_or_design() {
    let (fixture, mut state) = crate::native_selection_overlay::tests::selected_state();
    let before = fixture.source_bytes();
    open_menu(&mut state, fixture.root(), false);
    let selected = state.selection.clone();
    let mut session = crate::RenderSession::default();
    assert!(session.ensure_board(&state, 1600, 1000, 1.0));
    let retained = session.board().unwrap();
    assert_eq!(
        capture(&state, retained),
        Some((None, None)),
        "selected visuals win"
    );
    state.selection = SelectionTarget::None;
    let hover = capture(&state, retained)
        .unwrap()
        .0
        .expect("native track pre-highlight");
    assert_eq!(
        Some(hover),
        board_hover_bounds(retained, &state.scene.tracks[0].object_id)
    );
    let menu = state
        .ui
        .marking_menu
        .as_ref()
        .unwrap()
        .selection
        .as_ref()
        .unwrap();
    assert!(menu.items[0].label.contains("Net "));
    assert!(menu.items[0].label.contains("F.Cu"));
    let prepared = PreparedScene::from_workspace_for_surface(
        &state,
        1600,
        1000,
        1.0,
        CameraState::fit_to_bounds(&state.scene.bounds),
        retained,
    )
    .unwrap();
    assert_eq!(prepared.menu_hover, Some((Some(hover), None)));
    assert!(!prepared.board_interaction_vertices.is_empty());
    let preparation = Preparation::workspace(&state, retained, [1600, 1000]);
    session.install_prepared(prepared.clone(), preparation);
    let resolves = crate::retained_scene_resolve_count();
    assert!(session.update_pointer(crate::render_input::PointerUpdate {
        generation: session.pointer_generation(),
        cursor: Some(datum_gui_protocol::ScreenPointPx {
            x: prepared.scene_viewport.x + prepared.scene_viewport.width / 2.0,
            y: prepared.scene_viewport.y + prepared.scene_viewport.height / 2.0,
        }),
        hover: None,
        style: datum_gui_protocol::CrosshairStyle::None,
    }));
    assert_eq!(
        session.prepared().unwrap().board_interaction_vertices,
        prepared.board_interaction_vertices
    );
    assert_eq!(crate::retained_scene_resolve_count(), resolves);
    let retained = session.board().unwrap();
    let menu = state
        .ui
        .marking_menu
        .as_mut()
        .unwrap()
        .selection
        .as_mut()
        .unwrap();
    menu.preview = None;
    assert_eq!(capture(&state, retained), Some((None, None)));
    let menu = state
        .ui
        .marking_menu
        .as_mut()
        .unwrap()
        .selection
        .as_mut()
        .unwrap();
    menu.preview = Some(0);
    menu.revision.push_str("stale");
    assert_eq!(capture(&state, retained), Some((None, None)));
    state.ui.marking_menu = None;
    assert_eq!(capture(&state, retained), None);
    assert_eq!(state.selection, SelectionTarget::None);
    state.selection = selected;
    assert_eq!(fixture.source_bytes(), before);
}

#[test]
fn native_schematic_candidate_uses_certified_text_layout_and_preserves_source_bytes() {
    let (fixture, mut state) =
        crate::native_selection_overlay::tests::selected_schematic_text_state();
    let before = fixture.source_bytes();
    open_menu(&mut state, fixture.root(), true);
    state.selection = SelectionTarget::None;
    let retained = RetainedScene::from_workspace_for_surface(&state, 1600, 1000, 1.0);
    let scene = state.schematic_scene.as_ref().unwrap();
    let text = &scene.board_text_geometries[0];
    assert_eq!(
        capture(&state, &retained),
        Some((None, schematic_hover_bounds(scene, &text.object_id)))
    );
    assert!(capture(&state, &retained).unwrap().1.is_some());
    state.ui.layout.set_focused_content(PaneContent::Schematic);
    let prepared = PreparedScene::from_workspace_for_surface(
        &state,
        1600,
        1000,
        1.0,
        CameraState::fit_to_bounds(&state.scene.bounds),
        &retained,
    )
    .unwrap();
    assert!(!prepared.schematic_overlay_vertices.is_empty());
    assert_eq!(state.selection, SelectionTarget::None);
    assert_eq!(fixture.source_bytes(), before);
}
