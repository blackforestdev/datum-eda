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
    let mut f = fixture::Fixture::new("accessible-selection");
    let tracks: Vec<_> = (1..=257)
        .map(|n| {
            Track::straight(
                fixture::id(n),
                f.net,
                Point::new(0, 0),
                Point::new(10, 0),
                if n % 2 == 0 { 2 } else { 3 },
                1,
            )
        })
        .collect();
    f.commit(tracks.iter().map(|t| Operation::CreateBoardTrack {
        track_id: t.uuid,
        track: serde_json::to_value(t).unwrap(),
    }));
    f.zone(fixture::id(500));
    let authority = NativeSelectionResolution::capture(&f.model).unwrap();
    let ids: Vec<_> = authority
        .authored_identities()
        .iter()
        .filter(|id| matches!(id.class, Class::Track | Class::Zone))
        .cloned()
        .collect();
    let subject = SelectionSubject::Compound(
        CompoundSelection::new(ids.clone(), Some(ids[0].clone())).unwrap(),
    );
    let mut scene = datum_gui_protocol::load_fixture_workspace_state().scene;
    scene.project_uuid = f.model.project.project_id.to_string();
    scene.board_uuid = f.board.to_string();
    scene.components.clear();
    scene.pads.clear();
    scene.vias.clear();
    scene.zones.clear();
    scene.board_texts.clear();
    scene.board_graphics.clear();
    scene.outline.clear();
    let mut track = scene.tracks[0].clone();
    track.object_id = "native-track".into();
    track.source_object_uuid = tracks[0].uuid.to_string();
    track.track_uuid = track.source_object_uuid.clone();
    scene.tracks = vec![track];
    let view =
        build_selection_view(&f.model, &authority, subject, &scene, None, |_| false).unwrap();
    let mut state = ServiceState::new(
        crate::terminal_accessibility_platform::worker::empty_snapshot(),
        true,
        Vec::new(),
    );
    state.selection = Some(SelectionAccessibilitySnapshot {
        view: Arc::new(view),
        focused: true,
        evaluating: None,
        pane: Some(datum_gui_protocol::PaneId(0)),
    });
    (f, state)
}
fn call(path: &str, interface: &str, member: &str, signature: &str, body: Vec<u8>) -> Message {
    Message::method_call(7, "ignored", path, interface, member, signature, body)
}
fn description_call(path: &str) -> Message {
    let mut b = BodyWriter::new();
    b.string(ACCESSIBLE);
    b.string("Description");
    call(path, PROPERTIES, "Get", "ss", b.finish())
}
#[test]
fn selection_accessibility_inventory_is_complete_and_bound_to_source_identity() {
    let (f, mut state) = setup();
    let bytes = f.source_bytes();
    let snapshot = state.selection.as_ref().unwrap();
    assert_eq!(snapshot.view.members.len(), 258);
    assert_eq!(snapshot.view.hidden.len(), 1);
    let root_paths = state.root_child_paths();
    assert_eq!(root_paths.last().unwrap(), PATH);
    assert_eq!(root_paths[0], TERMINAL_PATH);
    let member = snapshot.view.members.first().unwrap().clone();
    let path = member_path(snapshot.view.project, &member);
    let mut body = BodyWriter::new();
    body.string(ACCESSIBLE);
    body.string("ChildCount");
    let reply = state.dispatch(1, &call(MEMBERS, PROPERTIES, "Get", "ss", body.finish()));
    assert_eq!(
        reply.body_reader().variant_i32().unwrap(),
        258,
        "not the context transport cap"
    );
    let reply = state.dispatch(2, &description_call(&path));
    let description = reply.body_reader().variant_string().unwrap();
    assert!(description.contains(&member.id.to_string()));
    assert!(description.contains("hidden true"));
    assert!(description.contains("selection focus true"));
    let mut body = BodyWriter::new();
    body.i32(257);
    let reply = state.dispatch(
        3,
        &call(MEMBERS, ACCESSIBLE, "GetChildAtIndex", "i", body.finish()),
    );
    assert_eq!(reply.kind, MessageType::MethodReturn);
    let mut reader = reply.body_reader();
    reader.string().unwrap();
    let last = reader.object_path().unwrap();
    assert_ne!(last, path);
    assert!(matches!(
        node(&state.selection.as_ref().unwrap().view, &last),
        Some(Node::Member(_))
    ));
    fixture::Fixture::assert_bytes(&bytes);
}
#[test]
fn selection_accessibility_fields_explain_scope_mixed_and_unavailable_without_mutation() {
    let (f, mut state) = setup();
    let bytes = f.source_bytes();
    let members = state.selection.as_ref().unwrap().view.members.clone();
    let reply = state.dispatch(1, &description_call(&field_path("member_count")));
    assert_eq!(reply.body_reader().variant_string().unwrap(), "258");
    let class_field = state.dispatch(1, &description_call(&field_path("copper_width_nm")));
    assert_eq!(
        class_field.kind,
        MessageType::Error,
        "All has only class-independent aggregates"
    );
    Arc::make_mut(&mut state.selection.as_mut().unwrap().view).scope = Some(Class::Track);
    let reply = state.dispatch(2, &description_call(&field_path("copper_width_nm")));
    assert_eq!(reply.body_reader().variant_string().unwrap(), "Mixed");
    assert_eq!(state.selection.as_ref().unwrap().view.members, members);
    let write = state.dispatch(
        3,
        &call(PATH, PROPERTIES, "Set", "ss", {
            let mut b = BodyWriter::new();
            b.string(ACCESSIBLE);
            b.string("Name");
            b.finish()
        }),
    );
    assert_eq!(write.kind, MessageType::Error);
    let action = state.dispatch(
        4,
        &call(PATH, "org.a11y.atspi.Action", "DoAction", "i", i32_body(0)),
    );
    assert_eq!(action.kind, MessageType::Error);
    let phantom = state.dispatch(
        5,
        &description_call(&member_path(
            state.selection.as_ref().unwrap().view.project,
            &Id {
                class: Class::Track,
                id: fixture::id(9999),
                instance_path: vec![],
            },
        )),
    );
    assert_eq!(phantom.kind, MessageType::Error);
    fixture::Fixture::assert_bytes(&bytes);
}
#[test]
fn selection_accessibility_events_are_quiet_for_noop_and_coalesce_dense_lifetime_changes() {
    let (_f, mut state) = setup();
    let previous = state.selection.clone().unwrap();
    let mut serial = 1;
    assert!(
        messages(
            || {
                serial += 1;
                serial
            },
            "",
            Some(&previous),
            Some(&previous),
            &state
        )
        .is_empty()
    );
    Arc::make_mut(&mut state.selection.as_mut().unwrap().view).notice =
        Some("Derived membership change refused; subject preserved".into());
    let next = state.selection.clone().unwrap();
    let events = messages(
        || {
            serial += 1;
            serial
        },
        "",
        Some(&previous),
        Some(&next),
        &state,
    );
    assert_eq!(events.len(), 1);
    assert_eq!(events[0].header.member.as_deref(), Some("PropertyChange"));
    let view = Arc::make_mut(&mut state.selection.as_mut().unwrap().view);
    let removed = view.members.pop_first().unwrap();
    view.dropped.insert(removed);
    view.stale = true;
    let next = state.selection.clone().unwrap();
    let events = messages(
        || {
            serial += 1;
            serial
        },
        "",
        Some(&previous),
        Some(&next),
        &state,
    );
    assert_eq!(
        events.len(),
        2,
        "constant notification count for complete dense changes"
    );
    let reply = state.dispatch(1, &description_call(PATH));
    let description = reply.body_reader().variant_string().unwrap();
    assert!(description.contains("Stale previous basis"));
    assert!(description.contains("1 dropped"));
    state.selection = None;
    let reply = state.dispatch(2, &description_call(PATH));
    assert_eq!(reply.kind, MessageType::Error);
}

#[test]
fn selection_accessibility_pane_switch_discloses_owner_without_changing_membership() {
    let (_f, mut state) = setup();
    let previous = state.selection.clone().unwrap();
    state.selection.as_mut().unwrap().pane = Some(datum_gui_protocol::PaneId(7));
    let next = state.selection.clone().unwrap();
    assert_eq!(previous.view, next.view);
    let reply = state.dispatch(1, &description_call(PATH));
    assert!(
        reply
            .body_reader()
            .variant_string()
            .unwrap()
            .contains("Editor pane 7")
    );
    let events = messages(|| 1, "", Some(&previous), Some(&next), &state);
    assert_eq!(events.len(), 1);
    assert_eq!(events[0].header.member.as_deref(), Some("PropertyChange"));
}

#[test]
fn selection_accessibility_evaluation_reports_pending_work_without_partial_membership() {
    let (_f, mut state) = setup();
    let previous = state.selection.clone().unwrap();
    state.selection.as_mut().unwrap().evaluating = Some((datum_gui_protocol::PaneId(0), 99));
    let next = state.selection.clone().unwrap();
    assert_eq!(previous.view, next.view);
    let reply = state.dispatch(1, &description_call(PATH));
    assert!(
        reply
            .body_reader()
            .variant_string()
            .unwrap()
            .contains("Evaluating selection: 99 candidates examined; Escape cancels")
    );
    let events = messages(|| 1, "", Some(&previous), Some(&next), &state);
    assert_eq!(events.len(), 1);
    assert_eq!(events[0].header.member.as_deref(), Some("PropertyChange"));
}
