//! Exercise the same CPU ownership transition used by Runtime surface invalidation.
use super::*;

#[test]
fn resize_keeps_complete_active_charge_while_retiring_obsolete_history() {
    let mut state = datum_gui_protocol::load_fixture_workspace_state();
    state.scene.unrouted_primitives.clear();
    state
        .scene
        .component_graphics
        .retain(|graphic| !(graphic.closed && graphic.render_role == "component_mechanical"));
    let scene = RetainedScene::from_workspace(&state, 960, 720);
    assert!(scene.can_reuse_for_surface_resize());
    let payload = scene.heap_payload_bytes().unwrap();
    let geometry = scene.geometry_observer();
    assert!(
        payload > geometry.heap_bytes_excluding([]),
        "the fixture must expose command/hit ownership beyond shared geometry"
    );
    let mut history = RetainedSceneHistory::default();
    history.insert(tests::key(0), tests::scene());
    history.limit_for_active(&scene);
    assert_eq!(history.entries.len(), 1);
    let mut active = Some(scene);
    for _ in 0..1000 {
        history.invalidate_surface_size(&mut active);
        assert!(active.is_some());
        assert_eq!(history.entries.capacity(), 0);
        assert_eq!(history.active_bytes, payload);
        assert_eq!(history.accounted_bytes(), payload);
        assert_eq!(
            history
                .active_geometry
                .as_ref()
                .unwrap()
                .heap_bytes_excluding([&geometry]),
            0,
            "resize must preserve the same geometry owner"
        );
    }
    // Full invalidation still retires the active registration when its owner closes.
    active = None;
    history.invalidate_surface_size(&mut active);
    assert_eq!(history.accounted_bytes(), 0);
    assert!(history.active_geometry.is_none());
}

#[test]
fn dependent_scene_retires_active_and_history_charges_for_reconstruction() {
    let mut state = datum_gui_protocol::load_fixture_workspace_state();
    state
        .scene
        .component_graphics
        .push(datum_gui_protocol::ComponentGraphicPrimitive {
            graphic_id: "resize-dependent".into(),
            component_uuid: "resize-component".into(),
            layer_id: None,
            primitive_kind: "polyline".into(),
            render_role: "component_mechanical".into(),
            width_nm: Some(100_000),
            closed: true,
            path: Vec::new(),
            holes: Vec::new(),
        });
    let scene = RetainedScene::from_workspace(&state, 960, 720);
    assert!(!scene.can_reuse_for_surface_resize());
    let mut history = RetainedSceneHistory::default();
    history.insert(tests::key(0), tests::scene());
    history.limit_for_active(&scene);
    let mut active = Some(scene);
    history.invalidate_surface_size(&mut active);
    assert!(active.is_none());
    assert_eq!(history.entries.capacity(), 0);
    assert_eq!(history.active_bytes, 0);
    assert!(history.active_geometry.is_none());
    assert_eq!(history.accounted_bytes(), 0);
}
