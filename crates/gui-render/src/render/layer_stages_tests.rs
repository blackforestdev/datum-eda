use super::*;
#[test]
fn declared_copper_without_side_retains_fallback_material_and_atomic_layer_order() {
    let layers = vec![
        datum_gui_protocol::SceneLayer {
            layer_id: "a".into(),
            name: "Top Copper".into(),
            kind: "copper".into(),
            render_order: 20,
            visible_by_default: true,
        },
        datum_gui_protocol::SceneLayer {
            layer_id: "z".into(),
            name: "Copper foil".into(),
            kind: "copper".into(),
            render_order: 10,
            visible_by_default: true,
        },
        datum_gui_protocol::SceneLayer {
            layer_id: "noncopper".into(),
            name: "Top Copper".into(),
            kind: "mechanical".into(),
            render_order: 30,
            visible_by_default: true,
        },
    ];
    assert_eq!(
        render_stage_for_layer("a", &layers),
        RenderStage::UnresolvedCopper
    );
    assert_eq!(
        render_stage_for_layer("z", &layers),
        RenderStage::UnresolvedCopper
    );
    assert_eq!(
        render_stage_for_layer("noncopper", &layers),
        RenderStage::Other
    );
    assert_eq!(
        resolve_layer_appearance_with_scene(Some("a"), &layers).authored_track,
        resolve_layer_appearance(None).authored_track
    );
    let mut commands = vec![
        RetainedDrawCommand::Quads {
            layer_id: Some("a".into()),
            range: 0..6,
        },
        RetainedDrawCommand::Strokes {
            layer_id: Some("z".into()),
            range: 0..1,
        },
        RetainedDrawCommand::Quads {
            layer_id: Some("z".into()),
            range: 6..12,
        },
    ];
    scene_retained_access::sort_retained_draw_commands(&mut commands, &layers, |_| Ok(())).unwrap();
    assert!(matches!(&commands[0],RetainedDrawCommand::Strokes{layer_id:Some(id),..} if id=="z"));
    assert!(matches!(&commands[1],RetainedDrawCommand::Quads{layer_id:Some(id),..} if id=="z"));
    assert!(matches!(&commands[2],RetainedDrawCommand::Quads{layer_id:Some(id),..} if id=="a"));
    for (name, stage) in [
        ("B.Cu", RenderStage::BottomCopper),
        ("F.Cu", RenderStage::TopCopper),
        ("In1.Cu", RenderStage::InnerCopper),
    ] {
        assert_eq!(render_stage_for_layer(name, &[]), stage);
    }
}
#[test]
fn actual_genesis_copper_track_is_painted_and_selected_without_mutation_or_resize_rebuild() {
    let (fixture, mut state) =
        crate::native_selection_overlay::tests::selected_track_state_with_layer(
            10_000_000,
            "Top Copper",
        );
    let before = fixture.source_bytes();
    let retained = RetainedScene::from_workspace(&state, 1280, 800);
    assert_eq!(retained.world_strokes.len(), 1);
    let mut prepared = PreparedScene::from_workspace(
        &state,
        1280,
        800,
        CameraState::fit_to_bounds(&state.scene.bounds),
        &retained,
    )
    .unwrap();
    prepared.append_native_selection(&state).unwrap();
    let ranges: Vec<_> = prepared.selected_paint_ranges(&retained, None).collect();
    assert_eq!(ranges.len(), 1);
    assert_eq!(ranges[0].range, 0..1);
    assert!(ranges[0].stroke);
    assert!(retained.can_reuse_for_surface_resize());
    state.selection = SelectionTarget::None;
    let cleared = RetainedScene::from_workspace(&state, 1280, 800);
    assert_eq!(
        bytemuck::cast_slice::<WorldStrokeInstance, u8>(retained.world_strokes.as_ref()),
        bytemuck::cast_slice::<WorldStrokeInstance, u8>(cleared.world_strokes.as_ref())
    );
    assert_eq!(fixture.source_bytes(), before);
}
