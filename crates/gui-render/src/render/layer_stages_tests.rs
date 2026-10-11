use super::*;
use crate::{
    CameraState, PreparedScene, RetainedDrawCommand, RetainedScene, WorldStrokeInstance,
    board_graphic_world_color, resolve_layer_appearance, resolve_layer_appearance_with_scene,
    scene_retained_access,
};
use datum_gui_protocol::{ReviewWorkspaceState, SelectionTarget};
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

#[test]
fn native_text_and_outline_unknown_layers_keep_existing_fallback_visible() {
    use datum_gui_protocol::{LiveReviewRequest, load_board_editor_workspace_with_model};
    let root = std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("../engine/testdata/golden/text/native/text-transform-repro");
    let request = LiveReviewRequest {
        project_root: root,
        board_file: None,
        artifact_path: None,
        net_uuid: None,
        from_anchor_pad_uuid: None,
        to_anchor_pad_uuid: None,
        profile: None,
        kicad_board_source: None,
    };
    let (mut state, model) = load_board_editor_workspace_with_model(&request).unwrap();
    let source: Vec<_> = model
        .source_shards
        .iter()
        .map(|s| (s.path.clone(), std::fs::read(&s.path).unwrap()))
        .collect();
    assert_eq!(state.scene.board_text_geometries.len(), 8);
    assert!(
        state
            .scene
            .board_text_geometries
            .iter()
            .all(|t| t.layer_id == "L37")
    );
    assert_eq!(
        state.scene.board_graphics.len(),
        4,
        "complete authored outline incidences"
    );
    assert!(
        state
            .scene
            .board_graphics
            .iter()
            .all(|g| g.layer_id == "L44")
    );
    let original_scene = format!("{:?}", state.scene);
    let assert_visible =
        |state: &ReviewWorkspaceState| {
            let retained = RetainedScene::from_workspace(state, 1280, 800);
            for text in &state.scene.board_text_geometries {
                assert_eq!(
                    render_stage_for_layer(&text.layer_id, &state.scene.layers),
                    RenderStage::Other
                );
                let span = retained
                    .presentation
                    .glyphs
                    .iter()
                    .find(|s| s.key.as_deref() == Some(text.object_id.as_str()))
                    .unwrap();
                assert!(!span.range.is_empty(), "complete source text stays painted");
                assert!(
                    retained
                        .visible_draw_commands(state)
                        .iter()
                        .any(|c| matches!(c,
                RetainedDrawCommand::Glyphs { material, range, .. }
                if range.start as usize <= span.range.start && range.end as usize >= span.range.end
                    && retained.glyph.materials[*material as usize].color ==
                        board_graphic_world_color(&text.layer_id, &state.scene.layers, false)))
                );
            }
            for graphic in &state.scene.board_graphics {
                assert_eq!(
                    render_stage_for_layer(&graphic.layer_id, &state.scene.layers),
                    RenderStage::Other
                );
                let span = retained
                    .presentation
                    .strokes
                    .iter()
                    .find(|s| s.key.as_deref() == Some(graphic.object_id.as_str()))
                    .unwrap();
                assert!(!span.range.is_empty());
                assert!(retained.visible_draw_commands(state).iter().any(
                    |command| matches!(command,
                RetainedDrawCommand::Strokes { layer_id, range }
                    if layer_id.as_deref() == Some("L44")
                        && range.start as usize <= span.range.start
                        && range.end as usize >= span.range.end)
                ));
            }
        };
    assert_visible(&state);
    // A declared custom non-copper layer keeps the same existing fallback.
    state.scene.layers.push(datum_gui_protocol::SceneLayer {
        layer_id: "L37".into(),
        name: "Custom annotation".into(),
        kind: "mechanical".into(),
        render_order: 5,
        visible_by_default: true,
    });
    assert_visible(&state);
    state.scene.layers.pop();
    assert_eq!(format!("{:?}", state.scene), original_scene);
    state
        .ui
        .filters
        .layer_visibility
        .insert("L37".into(), false);
    state
        .ui
        .filters
        .layer_visibility
        .insert("L44".into(), false);
    let hidden = RetainedScene::from_workspace(&state, 1280, 800);
    assert!(hidden.presentation.glyphs.is_empty());
    let visible = hidden.visible_draw_commands(&state);
    for graphic in &state.scene.board_graphics {
        let span = hidden
            .presentation
            .strokes
            .iter()
            .find(|s| s.key.as_deref() == Some(graphic.object_id.as_str()))
            .unwrap();
        assert!(
            !span.range.is_empty(),
            "retained geometry is separate from visibility"
        );
        assert!(
            !visible.iter().any(|command| matches!(command,
            RetainedDrawCommand::Strokes { layer_id, range }
                if layer_id.as_deref() == Some("L44")
                    && (range.start as usize) < span.range.end
                    && (range.end as usize) > span.range.start)),
            "hidden authored outline must not enter the visible painter sequence"
        );
    }
    assert!(!visible.iter().any(|command| matches!(command,
        RetainedDrawCommand::Quads { layer_id, .. }
            | RetainedDrawCommand::Glyphs { layer_id, .. }
            | RetainedDrawCommand::Strokes { layer_id, .. }
            if matches!(layer_id.as_deref(), Some("L37" | "L44")))));
    state.ui.filters.layer_visibility.clear();
    state.ui.filters.show_authored = false;
    let hidden = RetainedScene::from_workspace(&state, 1280, 800);
    assert!(hidden.presentation.glyphs.is_empty());
    assert!(hidden.visible_draw_commands(&state).is_empty());
    assert!(matches!(state.selection, SelectionTarget::None));
    for (path, bytes) in source {
        assert_eq!(std::fs::read(path).unwrap(), bytes);
    }
}
