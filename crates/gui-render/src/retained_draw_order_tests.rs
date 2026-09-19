use super::*;
use datum_gui_protocol::PointNm;

#[test]
fn retained_commands_interleave_strokes_with_their_semantic_layers() {
    let mut state = datum_gui_protocol::load_fixture_workspace_state();
    let bottom_copper = "bottom-copper".to_string();
    let top_copper = "top-copper".to_string();
    let top_paste = "top-paste".to_string();
    state.scene.layers = vec![
        datum_gui_protocol::SceneLayer {
            layer_id: bottom_copper.clone(),
            name: "B.Cu".into(),
            kind: "copper".into(),
            render_order: 0,
            visible_by_default: true,
        },
        datum_gui_protocol::SceneLayer {
            layer_id: top_copper.clone(),
            name: "F.Cu".into(),
            kind: "copper".into(),
            render_order: 1,
            visible_by_default: true,
        },
        datum_gui_protocol::SceneLayer {
            layer_id: top_paste.clone(),
            name: "F.Paste".into(),
            kind: "paste".into(),
            render_order: 2,
            visible_by_default: true,
        },
    ];
    state.scene.outline.clear();
    state.scene.components.clear();
    state.scene.component_graphics.clear();
    state.scene.component_texts.clear();
    state.scene.pads.clear();
    state.scene.tracks = (0..64)
        .map(|index| datum_gui_protocol::TrackPrimitive {
            object_id: format!("bottom-track-{index}"),
            object_kind: "track".into(),
            source_object_uuid: format!("bottom-track-{index}"),
            track_uuid: format!("bottom-track-{index}"),
            net_uuid: None,
            layer_id: bottom_copper.clone(),
            width_nm: 200_000,
            path: vec![
                PointNm {
                    x: index * 1_000_000,
                    y: 0,
                },
                PointNm {
                    x: (index + 1) * 1_000_000,
                    y: 0,
                },
            ],
        })
        .collect();
    state.scene.vias.clear();
    let polygon = vec![
        PointNm { x: 0, y: 0 },
        PointNm { x: 1_000_000, y: 0 },
        PointNm {
            x: 1_000_000,
            y: 1_000_000,
        },
        PointNm { x: 0, y: 0 },
    ];
    state.scene.zones = vec![
        datum_gui_protocol::ZonePrimitive {
            object_id: "bottom-zone".into(),
            object_kind: "zone".into(),
            source_object_uuid: "bottom-zone".into(),
            zone_uuid: "bottom-zone".into(),
            net_uuid: None,
            layer_id: bottom_copper.clone(),
            polygon: polygon.clone(),
        },
        datum_gui_protocol::ZonePrimitive {
            object_id: "top-zone".into(),
            object_kind: "zone".into(),
            source_object_uuid: "top-zone".into(),
            zone_uuid: "top-zone".into(),
            net_uuid: None,
            layer_id: top_copper.clone(),
            polygon,
        },
    ];
    state.scene.board_graphics = vec![datum_gui_protocol::BoardGraphicPrimitive {
        object_id: "top-paste-fill".into(),
        object_kind: "board_graphic".into(),
        primitive_kind: "polygon".into(),
        source_object_uuid: "top-paste-fill".into(),
        layer_id: top_paste.clone(),
        path: vec![
            PointNm { x: 0, y: 0 },
            PointNm { x: 1_000_000, y: 0 },
            PointNm {
                x: 1_000_000,
                y: 1_000_000,
            },
        ],
        holes: Vec::new(),
        width_nm: None,
    }];
    state.scene.board_texts.clear();
    state.scene.board_text_geometries.clear();
    state.scene.unrouted_primitives.clear();

    let retained = RetainedScene::from_workspace(&state, 1280, 800);
    let commands = retained.all_draw_commands();
    let kinds_for = |layer: &str| {
        commands
            .iter()
            .filter_map(|command| match command {
                RetainedDrawCommand::Quads { layer_id, .. }
                    if layer_id.as_deref() == Some(layer) =>
                {
                    Some("quad")
                }
                RetainedDrawCommand::Strokes { layer_id, .. }
                    if layer_id.as_deref() == Some(layer) =>
                {
                    Some("stroke")
                }
                _ => None,
            })
            .collect::<Vec<_>>()
    };
    assert_eq!(kinds_for(&bottom_copper), vec!["quad", "stroke"]);
    assert_eq!(commands.iter().filter(|command| matches!(command,
        RetainedDrawCommand::Strokes { layer_id, .. } if layer_id.as_deref() == Some(&bottom_copper))).count(), 1,
        "zone outline plus 64 contiguous same-layer tracks must collapse into one stroke command");
    assert_eq!(kinds_for(&top_copper), vec!["quad", "stroke"]);
    let find = |layer: &str, strokes: bool| {
        commands
            .iter()
            .position(|command| match command {
                RetainedDrawCommand::Quads { layer_id, .. } => {
                    !strokes && layer_id.as_deref() == Some(layer)
                }
                RetainedDrawCommand::Strokes { layer_id, .. } => {
                    strokes && layer_id.as_deref() == Some(layer)
                }
            })
            .expect("expected retained command")
    };
    assert!(find(&bottom_copper, true) < find(&top_copper, false));
    assert!(find(&top_copper, false) < find(&top_copper, true));
    assert!(find(&top_copper, true) < find(&top_paste, false));
}

fn resize_independent_workspace() -> ReviewWorkspaceState {
    let mut state = datum_gui_protocol::load_fixture_workspace_state();
    state.scene.component_graphics.clear();
    state.scene.unrouted_primitives.clear();
    state.scene.layers = vec![datum_gui_protocol::SceneLayer {
        layer_id: "resize-copper".into(),
        name: "F.Cu".into(),
        kind: "copper".into(),
        render_order: 0,
        visible_by_default: true,
    }];
    state.scene.zones = vec![datum_gui_protocol::ZonePrimitive {
        object_id: "resize-zone".into(),
        object_kind: "zone".into(),
        source_object_uuid: "resize-zone".into(),
        zone_uuid: "resize-zone".into(),
        net_uuid: None,
        layer_id: "resize-copper".into(),
        polygon: vec![
            PointNm { x: 0, y: 0 },
            PointNm {
                x: 10_000_000,
                y: 0,
            },
            PointNm {
                x: 10_000_000,
                y: 10_000_000,
            },
            PointNm { x: 0, y: 0 },
        ],
    }];
    state.scene.tracks = vec![datum_gui_protocol::TrackPrimitive {
        object_id: "resize-track".into(),
        object_kind: "track".into(),
        source_object_uuid: "resize-track".into(),
        track_uuid: "resize-track".into(),
        net_uuid: None,
        layer_id: "resize-copper".into(),
        width_nm: 200_000,
        path: vec![
            PointNm { x: 0, y: 0 },
            PointNm {
                x: 10_000_000,
                y: 10_000_000,
            },
        ],
    }];
    state
}

#[test]
fn retained_resize_reuse_matches_fresh_geometry_in_both_axes() {
    let state = resize_independent_workspace();
    let original = RetainedScene::from_workspace(&state, 1280, 800);
    assert!(original.can_reuse_for_surface_resize());
    assert!(!original.world_vertices.is_empty());
    assert!(!original.world_strokes.is_empty());
    assert!(!original.draw_commands.is_empty());
    for (width, height) in [(1600, 800), (1280, 1050), (1600, 1050)] {
        let fresh = RetainedScene::from_workspace(&state, width, height);
        assert_eq!(original.world_vertices, fresh.world_vertices);
        assert_eq!(original.world_strokes, fresh.world_strokes);
        assert_eq!(original.draw_commands, fresh.draw_commands);
        assert_eq!(original.world_hit_index, fresh.world_hit_index);
    }
}

#[test]
fn retained_resize_rejects_scale_dependent_unrouted_geometry() {
    let mut state = resize_independent_workspace();
    state.scene.unrouted_primitives.push(UnroutedPrimitive {
        object_id: "resize-airwire".into(),
        object_kind: "unrouted".into(),
        source_object_uuid: "resize-airwire".into(),
        net_uuid: "resize-net".into(),
        from_component: "a".into(),
        from_pin: "1".into(),
        to_component: "b".into(),
        to_pin: "1".into(),
        path: vec![
            PointNm { x: 0, y: 0 },
            PointNm {
                x: 10_000_000,
                y: 10_000_000,
            },
        ],
    });
    state.ui.filters.show_unrouted = true;
    let small = RetainedScene::from_workspace(&state, 1280, 800);
    let large = RetainedScene::from_workspace(&state, 1800, 1200);
    assert!(!small.can_reuse_for_surface_resize());
    assert_ne!(
        small.world_vertices, large.world_vertices,
        "airwire must be visible in this negative control"
    );
}

#[test]
fn retained_resize_rejects_scale_dependent_mechanical_dashes() {
    let mut state = resize_independent_workspace();
    // The largest closed shape is the selected-body source and is skipped by
    // this pass. Include a second, smaller closed shape to exercise real dashes.
    for (id, side) in [("outer", 20_000_000), ("inner", 10_000_000)] {
        state
            .scene
            .component_graphics
            .push(ComponentGraphicPrimitive {
                graphic_id: id.into(),
                component_uuid: "resize-part".into(),
                layer_id: None,
                primitive_kind: "polyline".into(),
                render_role: "component_mechanical".into(),
                width_nm: Some(100_000),
                closed: true,
                holes: vec![],
                path: vec![
                    PointNm { x: 0, y: 0 },
                    PointNm { x: side, y: 0 },
                    PointNm { x: side, y: side },
                    PointNm { x: 0, y: side },
                ],
            });
    }
    let small = RetainedScene::from_workspace(&state, 1280, 800);
    let large = RetainedScene::from_workspace(&state, 1800, 1200);
    assert!(!small.can_reuse_for_surface_resize());
    assert_ne!(
        small.world_vertices, large.world_vertices,
        "mechanical dashes must be visible in this negative control"
    );
    // Dependency classification is conservative even when the layer is hidden.
    state.ui.filters.show_authored = false;
    assert!(!RetainedScene::from_workspace(&state, 1280, 800).can_reuse_for_surface_resize());
}

#[test]
fn retained_resize_classifies_schematic_independently() {
    let mut state = resize_independent_workspace();
    state.schematic_scene = Some(state.scene.clone());
    let small =
        RetainedScene::from_workspace_schematic_for_surface(&state, 1280, 800, 1.0).unwrap();
    let large =
        RetainedScene::from_workspace_schematic_for_surface(&state, 1800, 1200, 1.0).unwrap();
    assert!(small.can_reuse_for_surface_resize());
    assert_eq!(small.world_vertices, large.world_vertices);
    assert_eq!(small.world_strokes, large.world_strokes);
    assert_eq!(small.draw_commands, large.draw_commands);
    state
        .schematic_scene
        .as_mut()
        .unwrap()
        .component_graphics
        .push(ComponentGraphicPrimitive {
            graphic_id: "schematic-only".into(),
            component_uuid: "schematic-part".into(),
            layer_id: None,
            primitive_kind: "polyline".into(),
            render_role: "component_mechanical".into(),
            width_nm: Some(100_000),
            closed: true,
            path: vec![],
            holes: vec![],
        });
    assert!(RetainedScene::from_workspace(&state, 1280, 800).can_reuse_for_surface_resize());
    assert!(
        !RetainedScene::from_workspace_schematic_for_surface(&state, 1280, 800, 1.0)
            .unwrap()
            .can_reuse_for_surface_resize()
    );
}
