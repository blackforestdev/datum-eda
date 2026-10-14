use super::*;
use crate::selection_accessibility::SelectionAccessibilitySnapshot;
use crate::terminal_accessibility_platform::selection_fixture as fixture;
use datum_gui_protocol::{
    AuthoredSelectionClass as Class, CompoundSelection, SelectionSubject,
    selection_resolution::NativeSelectionResolution, selection_view::build_selection_view,
};
use eda_engine::{board::Track, ir::geometry::Point, substrate::Operation};
use std::sync::Arc;

fn setup() -> (fixture::Fixture, ServiceState) {
    let mut f = fixture::Fixture::new("accessible-selected-state");
    let net = f.net;
    f.commit((1..=2).map(|n| {
        let track = Track::straight(
            fixture::id(n),
            net,
            Point::new(0, 0),
            Point::new(10, 0),
            3,
            1,
        );
        Operation::CreateBoardTrack {
            track_id: track.uuid,
            track: serde_json::to_value(track).unwrap(),
        }
    }));
    let package = fixture::id(610);
    f.commit([Operation::CreateBoardPackage {
        package_id: package,
        package: serde_json::json!({"uuid":package,"reference":"U1","value":"","position":{"x":0,"y":0},"rotation":0,"layer":1,"locked":false}),
        materialized: serde_json::json!({}),
    }]);
    let pad: eda_engine::board::PlacedPad = serde_json::from_value(serde_json::json!({"uuid":fixture::id(611),"package":package,"name":"1","net":f.net,"position":{"x":0,"y":0},"layer":1})).unwrap();
    let write = eda_engine::api::native_write::board_routing::build_place_board_pad(
        &f.model,
        fixture::provenance(),
        &pad,
    )
    .unwrap();
    eda_engine::api::native_write::commit_prepared(&mut f.model, &f.root, write).unwrap();
    let authority = NativeSelectionResolution::capture(&f.model).unwrap();
    let tracks: Vec<_> = authority
        .authored_identities()
        .iter()
        .filter(|id| id.class == Class::Track)
        .cloned()
        .collect();
    let mut state = ServiceState::new(
        crate::terminal_accessibility_platform::worker::empty_snapshot(),
        true,
        Vec::new(),
    );
    set_subject(
        &f,
        &mut state,
        SelectionSubject::Compound(
            CompoundSelection::new(tracks.clone(), Some(tracks[0].clone())).unwrap(),
        ),
    );
    (f, state)
}
fn set_subject(f: &fixture::Fixture, state: &mut ServiceState, subject: SelectionSubject) {
    let authority = NativeSelectionResolution::capture(&f.model).unwrap();
    let mut scene = datum_gui_protocol::load_fixture_workspace_state().scene;
    scene.project_uuid = f.model.project.project_id.to_string();
    scene.board_uuid = f.board.to_string();
    scene.source_revision = f.model.model_revision.0.clone();
    let view =
        build_selection_view(&f.model, &authority, subject, &scene, None, |_| false).unwrap();
    state.selection = Some(SelectionAccessibilitySnapshot {
        view: Arc::new(view),
        visual_lod: None,
        focused: true,
        evaluating: None,
        pane: Some(datum_gui_protocol::PaneId(0)),
    });
}
fn get_state(state: &mut ServiceState, path: &str) -> Message {
    state.dispatch(
        1,
        &Message::method_call(7, "ignored", path, ACCESSIBLE, "GetState", "", Vec::new()),
    )
}
fn words(state: &mut ServiceState, path: &str) -> Vec<u32> {
    let reply = get_state(state, path);
    assert_eq!(reply.kind, MessageType::MethodReturn);
    reply.body_reader().u32_array().unwrap()
}
#[test]
fn selected_state_matches_current_members_without_inventing_keyboard_focus() {
    let (f, mut state) = setup();
    let before = f.source_bytes();
    let view = state.selection.as_ref().unwrap().view.clone();
    let baseline = (1 << 8) | (1 << 11) | (1 << 24) | (1 << 25) | (1 << 30);
    for id in &view.members {
        assert_eq!(
            words(&mut state, &member_path(view.project, id)),
            vec![baseline | (1 << 23), 0]
        );
    }
    assert!(
        view.focus().is_some(),
        "optional selection focus exists but is not keyboard focus"
    );
    assert_eq!(words(&mut state, PATH), vec![baseline | (1 << 12), 0]);
    for path in [
        MEMBERS.to_owned(),
        FIELDS.to_owned(),
        field_path("member_count"),
    ] {
        let reply = get_state(&mut state, &path);
        assert_eq!(
            reply.body,
            super::super::state::state_body(false, false, Some((true, false)))
        );
        assert_eq!(reply.body_reader().u32_array().unwrap(), vec![baseline, 0]);
    }
    state.selection.as_mut().unwrap().focused = false;
    assert_eq!(words(&mut state, PATH), vec![baseline, 0]);
    fixture::Fixture::assert_bytes(&before);
}
#[test]
fn former_members_and_related_parents_are_not_selected_accessible_children() {
    let (f, mut state) = setup();
    let before = f.source_bytes();
    let view = state.selection.as_ref().unwrap().view.clone();
    let first = view.members.first().unwrap().clone();
    let last = view.members.last().unwrap().clone();
    set_subject(&f, &mut state, SelectionSubject::Object(last));
    let former = get_state(&mut state, &member_path(view.project, &first));
    assert_eq!(former.kind, MessageType::Error);
    assert_eq!(
        former.header.error_name.as_deref(),
        Some("org.freedesktop.DBus.Error.UnknownObject")
    );
    let authority = NativeSelectionResolution::capture(&f.model).unwrap();
    let pad = authority
        .authored_identities()
        .iter()
        .find(|id| id.class == Class::Pad)
        .unwrap()
        .clone();
    set_subject(&f, &mut state, SelectionSubject::Object(pad));
    let view = state.selection.as_ref().unwrap().view.clone();
    assert_eq!(view.related.len(), 1);
    let related = view.related.first().unwrap();
    let parent = get_state(&mut state, &member_path(view.project, related));
    assert_eq!(parent.kind, MessageType::Error);
    assert_eq!(
        parent.header.error_name.as_deref(),
        Some("org.freedesktop.DBus.Error.UnknownObject")
    );
    assert_ne!(
        words(
            &mut state,
            &member_path(view.project, view.members.first().unwrap())
        )[0] & (1 << 23),
        0
    );
    fixture::Fixture::assert_bytes(&before);
}
