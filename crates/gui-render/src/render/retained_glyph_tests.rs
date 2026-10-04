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
