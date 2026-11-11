//! Library graphics must survive the Board-only clip in an ordinary split frame.
use super::*;
use datum_gui_protocol::{
    PaneContent, SplitOrientation,
    library_browser::acquisition::{LibraryAcquisition, LibraryAcquisitionPreview},
};

#[test]
fn library_body_and_controls_remain_outside_board_clip_after_selection_composition() {
    let mut state = datum_gui_protocol::load_fixture_workspace_state();
    let library = state.ui.layout.open_beside_root(
        PaneContent::Library,
        SplitOrientation::Vertical,
        0.5,
        true,
    );
    let mut acquisition = LibraryAcquisition::begin(state.scene.source_revision.clone(), None);
    acquisition.names = vec!["Body preview".into()];
    acquisition.loaded = true;
    acquisition.selected = Some("Body preview".into());
    acquisition.preview = Some(LibraryAcquisitionPreview {
        name: "Body preview".into(),
        source_digest: "display-fixture".into(),
        source_path: "display-fixture.kicad_sym".into(),
        notices: Vec::new(),
        unit: r#"{"pins":{}}"#.parse().unwrap(),
        symbol: r#"{
            "uuid":"70000000-0000-4000-8000-000000000001", "name":"Body preview",
            "unit":"70000000-0000-4000-8000-000000000002", "pin_anchors":[],
            "drawings":[
                {"Rect":{"min":{"x":-1000000,"y":-1000000},"max":{"x":1000000,"y":1000000},"width":200000}},
                {"Line":{"from":{"x":-1000000,"y":0},"to":{"x":1000000,"y":0},"width":200000}}
            ]
        }"#.parse().unwrap(),
    });
    state.library.acquisition = Some(acquisition);
    let retained = RetainedScene::from_workspace(&state, 1600, 1000);
    let mut prepared = PreparedScene::from_workspace(
        &state,
        1600,
        1000,
        CameraState::fit_to_bounds(&state.scene.bounds),
        &retained,
    )
    .unwrap();
    let library_rect = prepared
        .layout
        .viewport_panes(&state.ui.layout)
        .panes
        .into_iter()
        .find(|pane| pane.id == library)
        .unwrap()
        .rect
        .scene;
    assert!(library_rect.x >= prepared.scene_viewport.x + prepared.scene_viewport.width);
    let boundary = prepared.selection_overlay_split;
    let vertices = prepared.viewport_overlay_vertices();
    assert!(
        boundary < vertices.len(),
        "Library cannot be in the Board-clipped prefix"
    );
    assert!(
        boundary > 0,
        "fixture retains a distinct Board overlay prefix"
    );
    let suffix = vertices[boundary..].to_vec();
    assert!(
        suffix
            .iter()
            .all(|v| library_rect.contains(v.pos[0], v.pos[1])),
        "Library suffix is already clipped to its own pane"
    );
    assert!(
        suffix.iter().any(|v| v.color == TEXT_PRIMARY),
        "native body geometry must reach the draw stream"
    );
    assert!(
        suffix
            .iter()
            .any(|v| v.color == design_tokens::chrome::SURFACE_02),
        "controls must reach the same draw stream"
    );
    assert!(
        vertices[..boundary]
            .iter()
            .all(|v| v.pos[0] <= prepared.scene_viewport.x + prepared.scene_viewport.width),
        "Board-owned prefix must not consume the Library pane"
    );
    prepared.append_native_selection(&state).unwrap();
    assert_eq!(
        prepared.selection_overlay_split, boundary,
        "selection composition must not restore Board scissoring over Library content"
    );
    assert_eq!(
        &prepared.viewport_overlay_vertices()[boundary..boundary + suffix.len()],
        suffix.as_slice(),
        "native selection appends after, preserving the actual Library geometry"
    );
}

#[test]
fn native_pin_label_clearance_library_uses_world_ink_without_duplicate_screen_labels() {
    let raw: datum_gui_protocol::LibraryJsonValue = r#"{"uuid":"70000000-0000-4000-8000-000000000001","name":"Thick body",
        "unit":"70000000-0000-4000-8000-000000000002",
        "drawings":[{"Rect":{"min":{"x":-2540000,"y":-2540000},"max":{"x":2540000,"y":2540000},"width":254000}}],
        "pin_anchors":[{"pin":"70000000-0000-4000-8000-000000000003","position":{"x":-5080000,"y":0},
            "style":{"orientation":"Right","length_nm":2540000}}]} "#.parse().unwrap();
    let rect = RectPx {
        x: 20.,
        y: 20.,
        width: 400.,
        height: 240.,
    };
    let draw = |name: &str| {
        let mut unit: datum_gui_protocol::LibraryJsonValue =
            r#"{"pins":{"70000000-0000-4000-8000-000000000003":{}}}"#
                .parse()
                .unwrap();
        unit["pins"]["70000000-0000-4000-8000-000000000003"]["name"] = name.into();
        let mut quads = Vec::new();
        let mut screen_text = Vec::new();
        crate::side_panels::library_preview::electrical::symbol(
            &raw,
            Some(&unit),
            rect,
            &mut quads,
            &mut screen_text,
        );
        assert!(screen_text.is_empty());
        quads
    };
    let unnamed = draw("");
    let named = draw("IN");
    assert!(
        named.len() > unnamed.len(),
        "native label adds rendered glyph ink"
    );
}
