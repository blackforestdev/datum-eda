use super::*;
#[test]
fn native_capsule_uses_current_source_width_and_canonical_members_without_writes() {
    let (fixture, mut state) = crate::native_selection_overlay::tests::selected_state();
    let bytes = fixture.source_bytes();
    let view = state.selection.clone();
    let rect = RectPx {
        x: 100.0,
        y: 200.0,
        width: 500.0,
        height: 400.0,
    };
    let camera = CameraState::fit_to_bounds(&state.scene.bounds);
    let projection = Projection::new(
        inset_rect(rect, 10.0, 10.0, 10.0, 10.0),
        &state.scene.bounds,
        camera,
    );
    let mut strokes = Vec::new();
    crate::native_selection_overlay::tests::push_native(
        &mut strokes,
        &mut Vec::new(),
        &mut Vec::new(),
        &state.scene,
        &state,
        rect,
        camera,
        datum_gui_protocol::PaneContent::Board,
    );
    assert_eq!(strokes.len(), 1);
    assert_eq!(
        strokes[0].material_width[3],
        (state.scene.tracks[0].width_nm as f32 * projection.scale).max(1.0)
    );
    assert_eq!(
        strokes[0].from,
        [
            projection.project_point(state.scene.tracks[0].path[0]).0,
            projection.project_point(state.scene.tracks[0].path[0]).1
        ]
    );
    assert_eq!(strokes[0].clip, [100.0, 200.0, 600.0, 600.0]);
    assert_eq!(strokes[0].high_contrast[0], 0.0);
    let before = strokes.clone();
    state.ui.global_preferences.high_contrast_noncolor = true;
    state.ui.global_preferences.reduced_motion = true;
    strokes.clear();
    crate::native_selection_overlay::tests::push_native(
        &mut strokes,
        &mut Vec::new(),
        &mut Vec::new(),
        &state.scene,
        &state,
        rect,
        camera,
        datum_gui_protocol::PaneContent::Board,
    );
    assert_eq!(strokes.len(), before.len());
    assert_eq!(strokes[0].high_contrast[0], 1.0);
    assert_eq!(strokes[0].material_width, before[0].material_width);
    assert_eq!(state.selection, view);
    for (path, before) in bytes {
        assert_eq!(std::fs::read(path).unwrap(), before);
    }
}

#[test]
fn native_schematic_wire_uses_shared_capsule_owner_and_existing_stroke_profile() {
    let (f, state) = crate::native_selection_overlay::tests::selected_wire_state();
    let before = f.source_bytes();
    let scene = state.schematic_scene.as_ref().unwrap();
    let SelectionTarget::Native(view) = &state.selection else {
        unreachable!()
    };
    assert_eq!(view.members.len(), 1);
    assert_eq!(
        view.members.iter().next().unwrap().class,
        datum_gui_protocol::AuthoredSelectionClass::Wire
    );
    assert!(view.unprojected.is_empty());
    let rect = RectPx {
        x: 400.0,
        y: 100.0,
        width: 400.0,
        height: 400.0,
    };
    let camera = CameraState::fit_to_bounds(&scene.bounds);
    let mut strokes = Vec::new();
    let mut quads = Vec::new();
    crate::native_selection_overlay::tests::push_native(
        &mut strokes,
        &mut quads,
        &mut Vec::new(),
        scene,
        &state,
        rect,
        camera,
        datum_gui_protocol::PaneContent::Schematic,
    );
    assert_eq!(strokes.len(), 1);
    assert!(
        quads.is_empty(),
        "no generic-width substitute for the source wire"
    );
    let projection = Projection::new(
        inset_rect(rect, 10.0, 10.0, 10.0, 10.0),
        &scene.bounds,
        camera,
    );
    assert_eq!(
        strokes[0].material_width[3],
        AuthoredStrokePrimitive::SchematicWire
            .weight()
            .resolve_px(projection.scale)
    );
    assert_eq!(strokes[0].clip, [400.0, 100.0, 800.0, 500.0]);
    for (path, bytes) in before {
        assert_eq!(std::fs::read(path).unwrap(), bytes);
    }
}
