use super::*;
#[test]
fn compact_native_glyphs_preserve_every_triangle_position_and_exact_material() {
    let (fixture, mut state) =
        crate::native_selection_overlay::tests::selected_schematic_text_state();
    let before = fixture.source_bytes();
    state.ui.layout.open_beside_root(
        datum_gui_protocol::PaneContent::Schematic,
        datum_gui_protocol::SplitOrientation::Vertical,
        0.5,
        true,
    );
    let retained =
        RetainedScene::from_workspace_schematic_for_surface(&state, 1280, 800, 1.0).unwrap();
    let scene = state.schematic_scene.as_ref().unwrap();
    let projection = Projection::new(
        RectPx {
            x: 0.,
            y: 0.,
            width: 1280.,
            height: 800.,
        },
        &scene.bounds,
        CameraState::fit_to_bounds(&scene.bounds),
    );
    let assets: std::collections::BTreeMap<_, _> = scene
        .glyph_mesh_assets
        .iter()
        .map(|a| (a.handle, a))
        .collect();
    let mut checked = 0;
    for text in &scene.board_text_geometries {
        let span = retained
            .presentation
            .glyphs
            .iter()
            .find(|s| s.key.as_deref() == Some(text.object_id.as_str()))
            .expect("actual native glyph source incidence");
        let material = retained
            .draw_commands
            .iter()
            .find_map(|c| match c {
                RetainedDrawCommand::Glyphs {
                    range, material, ..
                } if range.start as usize <= span.range.start
                    && range.end as usize >= span.range.end =>
                {
                    Some(retained.glyph.materials[*material as usize].color)
                }
                _ => None,
            })
            .expect("complete glyph painter material");
        let expected_color = board_graphic_world_color(&text.layer_id, &scene.layers, false);
        assert_eq!(material, expected_color);
        let mut reference = Vec::new();
        push_board_text_geometry_world(&mut reference, text, &assets, expected_color, &projection);
        let expected: Vec<_> = reference
            .iter()
            .flat_map(|q| gpu_data::quad_vertices(*q)[..3].to_vec())
            .collect();
        assert_eq!(expected.len(), span.range.len());
        for (point, vertex) in retained.glyph.points[span.range.clone()]
            .iter()
            .zip(expected)
        {
            assert_eq!(point.pos.map(f32::to_bits), vertex.pos.map(f32::to_bits));
            assert_eq!(material.map(f32::to_bits), vertex.color.map(f32::to_bits));
            checked += 1;
        }
    }
    assert!(checked > 0);
    assert_eq!(std::mem::size_of::<Point>(), 8);
    assert_eq!(fixture.source_bytes(), before);
}
#[test]
fn refused_glyph_emission_cannot_publish_points_material_or_commands() {
    let (_fixture, state) = crate::native_selection_overlay::tests::selected_schematic_text_state();
    let scene = state.schematic_scene.as_ref().unwrap();
    let projection = Projection::new(
        RectPx {
            x: 0.,
            y: 0.,
            width: 1280.,
            height: 800.,
        },
        &scene.bounds,
        CameraState::fit_to_bounds(&scene.bounds),
    );
    let mut quads = Vec::new();
    let mut commands = Vec::new();
    let mut points = crate::geometry_output::Admitted::<Point, _>::new(|_| {
        anyhow::bail!("glyph allocation refused")
    });
    let mut materials = Vec::new();
    push(
        &mut quads,
        &mut points,
        &mut materials,
        &mut commands,
        scene,
        &projection,
        &state,
    );
    assert!(points.finish_with_sources().is_err());
    assert!(quads.is_empty());
    assert!(materials.is_empty());
    assert!(commands.is_empty());
}

#[test]
fn refused_glyph_material_cannot_publish_a_draw_command() {
    let (_fixture, state) = crate::native_selection_overlay::tests::selected_schematic_text_state();
    let scene = state.schematic_scene.as_ref().unwrap();
    let projection = Projection::new(
        RectPx {
            x: 0.,
            y: 0.,
            width: 1280.,
            height: 800.,
        },
        &scene.bounds,
        CameraState::fit_to_bounds(&scene.bounds),
    );
    let mut quads = Vec::new();
    let mut points = Vec::new();
    let mut commands = Vec::new();
    let mut materials = crate::geometry_output::Admitted::<Material, _>::new(|_| {
        anyhow::bail!("glyph material refused")
    });
    push(
        &mut quads,
        &mut points,
        &mut materials,
        &mut commands,
        scene,
        &projection,
        &state,
    );
    assert!(materials.finish().is_err());
    assert!(!points.is_empty(), "only unpublished staging exists");
    assert!(quads.is_empty());
    assert!(commands.is_empty());
}

#[test]
fn custom_field_presentation_tether_is_selected_chrome_without_retained_paint_or_hit() {
    let (_fixture, mut state) =
        crate::native_selection_overlay::tests::selected_schematic_text_state();
    let before =
        RetainedScene::from_workspace_schematic_for_surface(&state, 1280, 800, 1.0).unwrap();
    let scene = state.schematic_scene.as_mut().unwrap();
    let text = &scene.board_texts[0];
    let tether_id = format!("{}:field-tether", text.object_id);
    scene
        .board_graphics
        .push(datum_gui_protocol::BoardGraphicPrimitive {
            object_id: tether_id.clone(),
            object_kind: "schematic_field_tether".into(),
            primitive_kind: "tether".into(),
            source_object_uuid: text.source_object_uuid.clone().unwrap(),
            layer_id: text.layer_id.clone(),
            path: vec![PointNm { x: 0, y: 0 }, text.position],
            holes: vec![],
            width_nm: None,
        });
    let retained =
        RetainedScene::from_workspace_schematic_for_surface(&state, 1280, 800, 1.0).unwrap();
    assert_eq!(before.glyph, retained.glyph);
    assert_eq!(
        bytemuck::cast_slice::<Vertex, u8>(before.world_vertices()),
        bytemuck::cast_slice::<Vertex, u8>(retained.world_vertices())
    );
    assert_eq!(
        bytemuck::cast_slice::<WorldStrokeInstance, u8>(before.world_strokes().as_ref()),
        bytemuck::cast_slice::<WorldStrokeInstance, u8>(retained.world_strokes().as_ref())
    );
    assert!(crate::interaction_overlay::board_hover_bounds(&retained, &tether_id).is_none());
    let scene = state.schematic_scene.as_ref().unwrap();
    let viewport = RectPx {
        x: 0.,
        y: 0.,
        width: 1280.,
        height: 800.,
    };
    let camera = CameraState::fit_to_bounds(&scene.bounds);
    let overlay = |state: &ReviewWorkspaceState, scene: &BoardReviewSceneV1| {
        let mut quads = Vec::new();
        crate::native_selection_overlay::tests::push_native(
            &mut Vec::new(),
            &mut quads,
            &mut Vec::new(),
            scene,
            state,
            viewport,
            camera,
            datum_gui_protocol::PaneContent::Schematic,
        );
        quads
    };
    let selected = overlay(&state, scene);
    let mut without = scene.clone();
    without.board_graphics.retain(|g| g.object_id != tether_id);
    assert!(
        selected.len() > overlay(&state, &without).len(),
        "selected owner gets tether"
    );
    let mut preview = state.clone();
    if let SelectionTarget::Native(view) = &mut preview.selection {
        view.schematic_keys.clear();
        for document in &mut view.schematic_documents {
            document.keys.clear();
        }
    }
    assert_eq!(
        overlay(&preview, scene).len(),
        overlay(&preview, &without).len()
    );
    use datum_gui_protocol::schematic_authoring::{
        CustomFieldDraft, FieldsEditDraft, selected::SymbolEditDraft,
    };
    let owner = scene.board_texts[0]
        .source_object_uuid
        .as_ref()
        .unwrap()
        .parse()
        .unwrap();
    preview.schematic_authoring.editing = Some(FieldsEditDraft::Custom(CustomFieldDraft {
        symbol: SymbolEditDraft {
            target: datum_gui_protocol::AuthoredSelectionIdentity {
                class: datum_gui_protocol::AuthoredSelectionClass::Symbol,
                id: owner,
                instance_path: vec![],
            },
            basis: datum_gui_protocol::AuthoringViewBasis {
                project: owner,
                revision: scene.source_revision.clone(),
                pane: datum_gui_protocol::PaneId(1),
                content: datum_gui_protocol::PaneContent::Schematic,
                source: owner,
                document: scene.schematic_document.clone(),
            },
            tip: None,
            original_reference: "R1".into(),
            original_value: "22k".into(),
        },
        original: eda_engine::schematic::PlacedSymbol {
            uuid: owner,
            part: None,
            entity: None,
            gate: None,
            lib_id: None,
            reference: "R1".into(),
            value: "22k".into(),
            fields: vec![],
            pins: vec![],
            position: eda_engine::ir::geometry::Point::new(0, 0),
            rotation: 0,
            mirrored: false,
            unit_selection: None,
            display_mode: eda_engine::schematic::SymbolDisplayMode::LibraryDefault,
            pin_overrides: vec![],
            hidden_power_behavior:
                eda_engine::schematic::HiddenPowerBehavior::SourceDefinedImplicit,
        },
        field_id: None,
        remove: false,
        visibility: Some(true),
        placing: true,
    }));
    assert!(
        overlay(&preview, scene).len() > overlay(&preview, &without).len(),
        "newly exposed text gets its active owner's tether before cached keys refresh"
    );
    preview.schematic_authoring.cancel();
    assert_eq!(
        overlay(&preview, scene).len(),
        overlay(&preview, &without).len()
    );
    let mut unselected = state.clone();
    unselected.selection = datum_gui_protocol::SelectionTarget::None;
    assert!(overlay(&unselected, scene).is_empty());
}
