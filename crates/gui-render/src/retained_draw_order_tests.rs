use super::*;
use datum_gui_protocol::PointNm;

#[test]
fn retained_commands_interleave_strokes_with_their_semantic_layers() {
    let mut state = datum_gui_protocol::load_fixture_workspace_state();
    let bottom_copper = "bottom-copper".to_string();
    let top_copper = "top-copper".to_string();
    let top_paste = "top-paste".to_string();
    state.scene.layers = vec![
        datum_gui_protocol::SceneLayer { layer_id: bottom_copper.clone(), name: "B.Cu".into(), kind: "copper".into(), render_order: 0, visible_by_default: true },
        datum_gui_protocol::SceneLayer { layer_id: top_copper.clone(), name: "F.Cu".into(), kind: "copper".into(), render_order: 1, visible_by_default: true },
        datum_gui_protocol::SceneLayer { layer_id: top_paste.clone(), name: "F.Paste".into(), kind: "paste".into(), render_order: 2, visible_by_default: true },
    ];
    state.scene.outline.clear();
    state.scene.components.clear();
    state.scene.component_graphics.clear();
    state.scene.component_texts.clear();
    state.scene.pads.clear();
    state.scene.tracks = (0..64).map(|index| datum_gui_protocol::TrackPrimitive {
        object_id: format!("bottom-track-{index}"), object_kind: "track".into(), source_object_uuid: format!("bottom-track-{index}"), track_uuid: format!("bottom-track-{index}"), net_uuid: None,
        layer_id: bottom_copper.clone(), width_nm: 200_000, path: vec![PointNm { x: index * 1_000_000, y: 0 }, PointNm { x: (index + 1) * 1_000_000, y: 0 }],
    }).collect();
    state.scene.vias.clear();
    let polygon = vec![PointNm { x: 0, y: 0 }, PointNm { x: 1_000_000, y: 0 }, PointNm { x: 1_000_000, y: 1_000_000 }, PointNm { x: 0, y: 0 }];
    state.scene.zones = vec![
        datum_gui_protocol::ZonePrimitive { object_id: "bottom-zone".into(), object_kind: "zone".into(), source_object_uuid: "bottom-zone".into(), zone_uuid: "bottom-zone".into(), net_uuid: None, layer_id: bottom_copper.clone(), polygon: polygon.clone() },
        datum_gui_protocol::ZonePrimitive { object_id: "top-zone".into(), object_kind: "zone".into(), source_object_uuid: "top-zone".into(), zone_uuid: "top-zone".into(), net_uuid: None, layer_id: top_copper.clone(), polygon },
    ];
    state.scene.board_graphics = vec![datum_gui_protocol::BoardGraphicPrimitive {
        object_id: "top-paste-fill".into(), object_kind: "board_graphic".into(), primitive_kind: "polygon".into(), source_object_uuid: "top-paste-fill".into(), layer_id: top_paste.clone(),
        path: vec![PointNm { x: 0, y: 0 }, PointNm { x: 1_000_000, y: 0 }, PointNm { x: 1_000_000, y: 1_000_000 }], holes: Vec::new(), width_nm: None,
    }];
    state.scene.board_texts.clear();
    state.scene.board_text_geometries.clear();
    state.scene.unrouted_primitives.clear();

    let retained = RetainedScene::from_workspace(&state, 1280, 800);
    let commands = retained.all_draw_commands();
    let kinds_for = |layer: &str| commands.iter().filter_map(|command| match command {
        RetainedDrawCommand::Quads { layer_id, .. } if layer_id.as_deref() == Some(layer) => Some("quad"),
        RetainedDrawCommand::Strokes { layer_id, .. } if layer_id.as_deref() == Some(layer) => Some("stroke"),
        _ => None,
    }).collect::<Vec<_>>();
    assert_eq!(kinds_for(&bottom_copper), vec!["quad", "stroke"]);
    assert_eq!(commands.iter().filter(|command| matches!(command,
        RetainedDrawCommand::Strokes { layer_id, .. } if layer_id.as_deref() == Some(&bottom_copper))).count(), 1,
        "zone outline plus 64 contiguous same-layer tracks must collapse into one stroke command");
    assert_eq!(kinds_for(&top_copper), vec!["quad", "stroke"]);
    let find = |layer: &str, strokes: bool| commands.iter().position(|command| match command {
        RetainedDrawCommand::Quads { layer_id, .. } => !strokes && layer_id.as_deref() == Some(layer),
        RetainedDrawCommand::Strokes { layer_id, .. } => strokes && layer_id.as_deref() == Some(layer),
    }).expect("expected retained command");
    assert!(find(&bottom_copper, true) < find(&top_copper, false));
    assert!(find(&top_copper, false) < find(&top_copper, true));
    assert!(find(&top_copper, true) < find(&top_paste, false));
}
