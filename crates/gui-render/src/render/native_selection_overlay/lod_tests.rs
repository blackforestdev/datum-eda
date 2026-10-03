use super::*;
use crate::native_selection_overlay::tests::selected_track_state;
use datum_gui_protocol::PaneContent;
fn pane() -> RectPx {
    RectPx {
        x: 20.0,
        y: 30.0,
        width: 300.0,
        height: 250.0,
    }
}
fn camera() -> CameraState {
    CameraState {
        center_x_nm: 0.0,
        center_y_nm: 0.0,
        zoom: 0.001,
    }
}
#[test]
fn tiny_source_bound_track_has_one_anchor_ring_without_geometry_or_membership_changes() {
    let (fixture, mut state) = selected_track_state(1);
    let before = fixture.source_bytes();
    let selected = RetainedScene::from_workspace(&state, 1000, 800);
    let members = match &state.selection {
        SelectionTarget::Native(view) => view.members.clone(),
        _ => unreachable!(),
    };
    let mut quads = Vec::new();
    push_native(
        &mut Vec::new(),
        &mut quads,
        &mut Vec::new(),
        &state.scene,
        &state,
        pane(),
        camera(),
        PaneContent::Board,
    );
    assert_eq!(
        quads.len(),
        24,
        "one whole-owner ring instead of a detailed path"
    );
    assert!(quads.iter().all(|q| q.color == TEXT_ACCENT));
    let projection = Projection::new(
        inset_rect(pane(), 10.0, 10.0, 10.0, 10.0),
        &state.scene.bounds,
        camera(),
    );
    let center = projection.project_point(PointNm { x: 0, y: 0 });
    let bounds = lod::bounds(&quads.iter().flat_map(|q| q.points).collect::<Vec<_>>());
    assert!((bounds.x + bounds.width * 0.5 - center.0).abs() < 0.01);
    assert!((bounds.y + bounds.height * 0.5 - center.1).abs() < 0.01);
    assert!(bounds.width > 6.0 && bounds.width <= 8.1);
    state.ui.global_preferences.high_contrast_noncolor = true;
    state.ui.global_preferences.reduced_motion = true;
    let mut accessible = Vec::new();
    push_native(
        &mut Vec::new(),
        &mut accessible,
        &mut Vec::new(),
        &state.scene,
        &state,
        pane(),
        camera(),
        PaneContent::Board,
    );
    assert_eq!(
        quads, accessible,
        "minimum meaning never depends on glow or animation"
    );
    assert_eq!(
        match &state.selection {
            SelectionTarget::Native(view) => &view.members,
            _ => unreachable!(),
        },
        &members
    );
    state.selection = SelectionTarget::None;
    let baseline = RetainedScene::from_workspace(&state, 1000, 800);
    assert_eq!(
        bytemuck::cast_slice::<Vertex, u8>(selected.world_vertices()),
        bytemuck::cast_slice::<Vertex, u8>(baseline.world_vertices())
    );
    for (path, bytes) in before {
        assert_eq!(std::fs::read(path).unwrap(), bytes);
    }
}
#[test]
fn hidden_offscreen_and_unavailable_anchor_cases_never_fabricate_minimum_selection() {
    let (_fixture, mut state) = selected_track_state(1);
    let render = |state: &ReviewWorkspaceState, camera| {
        let mut q = Vec::new();
        let mut strokes = Vec::new();
        push_native(
            &mut strokes,
            &mut q,
            &mut Vec::new(),
            &state.scene,
            state,
            pane(),
            camera,
            PaneContent::Board,
        );
        vec![(); q.len() + strokes.len()]
    };
    let mut outside = camera();
    outside.center_x_nm = 1_000_000_000.0;
    assert!(render(&state, outside).is_empty());
    let layer = state.scene.tracks[0].layer_id.clone();
    state
        .ui
        .filters
        .layer_visibility
        .insert(layer.clone(), false);
    assert!(render(&state, camera()).is_empty());
    state.ui.filters.layer_visibility.insert(layer, true);
    if let SelectionTarget::Native(view) = &mut state.selection {
        view.values.clear();
        view.authored_anchors.clear();
    }
    assert_eq!(
        render(&state, camera()).len(),
        1,
        "no anchor authority: retain existing detailed path, never a bounds-center ring"
    );
}

#[test]
fn actual_native_junctions_keep_distinct_anchors_and_complete_membership() {
    let (fixture, state) = crate::native_selection_overlay::tests::selected_points_state();
    let source = fixture.source_bytes();
    let scene = state.schematic_scene.as_ref().unwrap();
    let SelectionTarget::Native(view) = &state.selection else {
        unreachable!()
    };
    assert_eq!(view.members.len(), 2);
    assert!(view.unprojected.is_empty());
    let mut quads = Vec::new();
    push_native(
        &mut Vec::new(),
        &mut quads,
        &mut Vec::new(),
        scene,
        &state,
        pane(),
        camera(),
        PaneContent::Schematic,
    );
    assert_eq!(
        quads.len(),
        48,
        "one ring for each complete authored junction"
    );
    let projection = Projection::new(
        inset_rect(pane(), 10.0, 10.0, 10.0, 10.0),
        &scene.bounds,
        camera(),
    );
    let anchors: std::collections::BTreeSet<_> = view
        .members
        .iter()
        .map(|id| {
            let p = view.authored_anchor(id).unwrap();
            (p.x, p.y)
        })
        .collect();
    assert_eq!(
        anchors,
        std::collections::BTreeSet::from([(0, 0), (1_000_000, 0)])
    );
    assert_ne!(
        projection.project_point(PointNm { x: 0, y: 0 }),
        projection.project_point(PointNm { x: 1_000_000, y: 0 })
    );
    assert!(
        quads
            .iter()
            .flat_map(|q| q.points)
            .all(|(x, y)| pane().contains(x, y))
    );
    assert_eq!(view.members.len(), 2);
    for (path, bytes) in source {
        assert_eq!(std::fs::read(path).unwrap(), bytes);
    }
}
