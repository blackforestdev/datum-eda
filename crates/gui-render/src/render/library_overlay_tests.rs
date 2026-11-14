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
            native_part: None,
        name: "Body preview".into(),
        source_digest: "display-fixture".into(),
        source_path: "display-fixture.kicad_sym".into(),
        notices: Vec::new(),
        footprint: None,
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

fn preview_status_state(kind: &str) -> datum_gui_protocol::ReviewWorkspaceState {
    use datum_gui_protocol::{library_authoring::LibraryDraft, library_browser::LibraryEntry};
    let mut state = datum_gui_protocol::load_fixture_workspace_state();
    let unit: datum_gui_protocol::LibraryJsonValue = r#"{
        "uuid":"70000000-0000-4000-8000-000000000002","name":"Logic",
        "pins":{}}"#
        .parse()
        .unwrap();
    state.library.entries.push(LibraryEntry {
        id: "70000000-0000-4000-8000-000000000002".parse().unwrap(),
        kind: "units".into(),
        name: "Logic".into(),
        source_path: "pool/units/70000000-0000-4000-8000-000000000002.json".into(),
        revision: Some(1),
        raw: unit.clone(),
        warning: None,
    });
    let mut draft = LibraryDraft::begin(&state.library, None, kind, "test".into(), None).unwrap();
    draft.preview = Some(match kind {
        "units" => unit,
        "symbols" => r#"{"uuid":"70000000-0000-4000-8000-000000000001",
            "name":"Logic Symbol","unit":"70000000-0000-4000-8000-000000000002",
            "drawings":[],"pin_anchors":[]}"#
            .parse()
            .unwrap(),
        "padstacks" => r#"{"name":"Through pad","drill_nm":300000}"#.parse().unwrap(),
        _ => unreachable!(),
    });
    state.library.draft = Some(draft);
    state
}

fn preview_status_presentation(
    state: &datum_gui_protocol::ReviewWorkspaceState,
) -> (Vec<TextRun>, Vec<HitRegion>, Vec<TextRun>) {
    let rect = RectPx {
        x: 0.,
        y: 0.,
        width: 600.,
        height: 2400.,
    };
    let (mut quads, mut text, mut hits, mut inspector) =
        (Vec::new(), Vec::new(), Vec::new(), Vec::new());
    crate::side_panels::library_surface::render(state, rect, &mut quads, &mut text, &mut hits);
    crate::side_panels::library_surface::inspector(state, rect, &mut quads, &mut inspector);
    (text, hits, inspector)
}

#[test]
fn valid_electrical_previews_show_success_without_refusal_or_drill() {
    use datum_gui_protocol::library_browser::LibraryControl as C;
    for kind in ["units", "symbols"] {
        let state = preview_status_state(kind);
        let (text, hits, inspector) = preview_status_presentation(&state);
        assert!(
            !hits
                .iter()
                .any(|h| h.target == HitTarget::Library(C::Notice))
        );
        assert!(
            !text
                .iter()
                .any(|t| t.text.contains("Inspect complete refusal"))
        );
        assert!(
            inspector
                .iter()
                .any(|t| t.text == "Validated preview · no source / journal writes")
        );
        assert!(
            !inspector
                .iter()
                .any(|t| t.text.starts_with("Drill:") || t.text == "Preview refused")
        );
    }
}

#[test]
fn genuine_draft_and_browser_refusals_keep_console_access_and_reason() {
    use datum_gui_protocol::library_browser::LibraryControl as C;
    for browser_reason in [false, true] {
        let mut state = preview_status_state("units");
        let mut draft = state.library.draft.take().unwrap();
        // A missing resolved project is a real production preview refusal.
        let reason = draft
            .validate_preview(&state.library, std::path::Path::new("."))
            .unwrap_err();
        draft.preview = None;
        if browser_reason {
            state.library.refusal = Some(reason.clone());
        } else {
            draft.refusal = Some(reason.clone());
        }
        state.library.draft = Some(draft);
        let (text, hits, inspector) = preview_status_presentation(&state);
        assert!(
            hits.iter()
                .any(|h| h.target == HitTarget::Library(C::Notice))
        );
        assert!(
            text.iter()
                .any(|t| t.text == "Inspect complete refusal in Console")
        );
        assert!(inspector.iter().any(|t| t.text == reason));
        assert!(inspector.iter().any(|t| t.text == "Preview refused"));
        assert!(!inspector.iter().any(|t| t.text.starts_with("Drill:")));
    }
}

#[test]
fn physical_padstack_retains_drill_and_process_reason_inspection() {
    use datum_gui_protocol::library_browser::LibraryControl as C;
    let state = preview_status_state("padstacks");
    let (text, hits, inspector) = preview_status_presentation(&state);
    assert!(
        hits.iter()
            .any(|h| h.target == HitTarget::Library(C::Notice))
    );
    assert!(
        text.iter()
            .any(|t| t.text == "Inspect complete refusal / process reason in Console")
    );
    assert!(inspector.iter().any(|t| t.text == "Drill: 300000 nm"));
}
