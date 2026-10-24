use super::*;
use crate::selection_accessibility::SelectionAccessibilitySnapshot;
use crate::terminal_accessibility_platform::selection_fixture as fixture;
use datum_gui_protocol::{
    AuthoredSelectionClass as Class, CompoundSelection, SelectionSubject,
    selection_resolution::NativeSelectionResolution, selection_view::build_selection_view,
};
use eda_engine::{board::Track, ir::geometry::Point, substrate::Operation};
use std::sync::Arc;

#[test]
fn singleton_inspector_public_fields_match_shared_class_inspection_without_focus() {
    let mut f = fixture::Fixture::new("accessible-singleton");
    let track = f.line(fixture::id(1), Point::new(0, 0), Point::new(10_000_000, 0));
    let before = f.source_bytes();
    let authority = NativeSelectionResolution::capture(&f.model).unwrap();
    let id = authority
        .authored_identities()
        .iter()
        .find(|id| id.id == track.uuid)
        .unwrap()
        .clone();
    let scene = datum_gui_protocol::load_fixture_workspace_state().scene;
    let group = SelectionSubject::Compound(CompoundSelection::new([id.clone()], None).unwrap());
    let view =
        build_selection_view(&f.model, &authority, group.clone(), &scene, None, |_| true).unwrap();
    assert_eq!(view.scope, Some(Class::Track));
    let mut state = ServiceState::new(
        crate::terminal_accessibility_platform::worker::empty_snapshot(),
        true,
        Vec::new(),
    );
    state.selection = Some(SelectionAccessibilitySnapshot {
        visual_lod: None,
        view: Some(Arc::new(view.clone())),
        inspection: None,
        focused: true,
        evaluating: None,
        pane: Some(datum_gui_protocol::PaneId(0)),
    });
    assert_eq!(
        children::paths(state.selection.as_ref().unwrap(), Node::Fields).count(),
        view.fields().len()
    );
    for field in view.fields() {
        let path = field_path(field);
        assert!(matches!(node(&view, &path), Some(Node::Field(_))));
        let reply = state.dispatch(1, &description_call(&path));
        assert_eq!(
            reply.body_reader().variant_string().unwrap(),
            view.field_display_preview(field, usize::MAX)
        );
        assert_eq!(attributes(&mut state, &path)["subject-kind"], "compound");
    }
    assert_eq!(attributes(&mut state, PATH)["subject-kind"], "compound");
    assert_eq!(
        attributes(&mut state, &member_path(view.project, &id))["selection-focus"],
        "false"
    );
    Arc::make_mut(state.selection.as_mut().unwrap().view.as_mut().unwrap()).scope = None;
    let reply = state.dispatch(1, &description_call(&field_path("copper_width_nm")));
    assert_eq!(
        reply.kind,
        MessageType::Error,
        "explicit All retains aggregate inspection"
    );
    let retained = state.selection.as_ref().unwrap().view.as_deref().unwrap();
    assert_eq!(retained.subject, group);
    assert_eq!(retained.members, view.members);
    assert!(retained.focus().is_none());
    fixture::Fixture::assert_bytes(&before);
}
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
    scene.source_revision = f.model.model_revision.0.clone();
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
    track.net_uuid = Some(f.net.to_string());
    scene.tracks = vec![track];
    let view =
        build_selection_view(&f.model, &authority, subject, &scene, None, |_| false).unwrap();
    let mut state = ServiceState::new(
        crate::terminal_accessibility_platform::worker::empty_snapshot(),
        true,
        Vec::new(),
    );
    state.selection = Some(SelectionAccessibilitySnapshot {
        visual_lod: None,
        view: Some(Arc::new(view)),
        inspection: None,
        focused: true,
        evaluating: None,
        pane: Some(datum_gui_protocol::PaneId(0)),
    });
    (f, state)
}
fn call(path: &str, interface: &str, member: &str, signature: &str, body: Vec<u8>) -> Message {
    Message::method_call(7, "ignored", path, interface, member, signature, body)
}
fn attributes(state: &mut ServiceState, path: &str) -> std::collections::BTreeMap<String, String> {
    let reply = state.dispatch(1, &call(path, ACCESSIBLE, "GetAttributes", "", vec![]));
    assert_eq!(reply.kind, MessageType::MethodReturn);
    let mut cursor = super::super::super::body::Cursor::new(
        &reply.body,
        super::super::super::body::Endian::Little,
    );
    cursor.u32().unwrap();
    cursor.align(8).unwrap();
    let mut values = std::collections::BTreeMap::new();
    while !cursor.is_done() {
        cursor.align(8).unwrap();
        values.insert(cursor.string().unwrap(), cursor.string().unwrap());
    }
    values
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
    assert_eq!(snapshot.view.as_ref().unwrap().members.len(), 258);
    assert_eq!(snapshot.view.as_ref().unwrap().hidden.len(), 1);
    let root_paths = state.root_child_paths();
    assert_eq!(root_paths.last().unwrap(), PATH);
    assert_eq!(root_paths[0], TERMINAL_PATH);
    let member = snapshot
        .view
        .as_ref()
        .unwrap()
        .members
        .first()
        .unwrap()
        .clone();
    let path = member_path(snapshot.view.as_ref().unwrap().project, &member);
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
    let member_description = reply.body_reader().variant_string().unwrap();
    assert!(!member_description.contains(&member.id.to_string()));
    assert!(member_description.contains("Track: object 1"));
    let identity: serde_json::Value =
        serde_json::from_str(&attributes(&mut state, &path)["source-identity"]).unwrap();
    assert_eq!(identity, serde_json::to_value(&member).unwrap());
    assert!(!description(state.selection.as_ref().unwrap(), Node::Root).contains("S5A"));
    assert!(member_description.contains("hidden true"));
    assert!(member_description.contains("selection focus true"));
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
        node(
            state.selection.as_ref().unwrap().view.as_deref().unwrap(),
            &last
        ),
        Some(Node::Member(_))
    ));
    fixture::Fixture::assert_bytes(&bytes);
}
#[test]
fn selection_accessibility_fields_explain_scope_mixed_and_unavailable_without_mutation() {
    let (f, mut state) = setup();
    let bytes = f.source_bytes();
    let members = state
        .selection
        .as_ref()
        .unwrap()
        .view
        .as_ref()
        .unwrap()
        .members
        .clone();
    let reply = state.dispatch(1, &description_call(&field_path("member_count")));
    assert_eq!(reply.body_reader().variant_string().unwrap(), "258");
    let class_field = state.dispatch(1, &description_call(&field_path("copper_width_nm")));
    assert_eq!(
        class_field.kind,
        MessageType::Error,
        "All has only class-independent aggregates"
    );
    Arc::make_mut(state.selection.as_mut().unwrap().view.as_mut().unwrap()).scope =
        Some(Class::Track);
    let reply = state.dispatch(2, &description_call(&field_path("copper_width_nm")));
    assert_eq!(reply.body_reader().variant_string().unwrap(), "Mixed");
    assert_eq!(
        attributes(&mut state, &field_path("copper_width_nm"))["value-state"],
        "mixed"
    );
    let count = attributes(&mut state, &field_path("member_types"));
    assert_eq!(
        count["canonical-value"],
        match state
            .selection
            .as_ref()
            .unwrap()
            .view
            .as_ref()
            .unwrap()
            .field("member_types")
        {
            InspectionValue::Common(value) => value.to_string(),
            _ => panic!("common counts"),
        }
    );
    assert_eq!(
        state
            .selection
            .as_ref()
            .unwrap()
            .view
            .as_ref()
            .unwrap()
            .members,
        members
    );
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
            state
                .selection
                .as_ref()
                .unwrap()
                .view
                .as_ref()
                .unwrap()
                .project,
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
    Arc::make_mut(state.selection.as_mut().unwrap().view.as_mut().unwrap()).notice =
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
    let view = Arc::make_mut(state.selection.as_mut().unwrap().view.as_mut().unwrap());
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
    let member_description = reply.body_reader().variant_string().unwrap();
    assert!(member_description.contains("Stale previous basis"));
    assert!(member_description.contains("1 dropped"));
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

#[test]
fn visual_lod_is_read_only_current_presentation_and_coalesces_without_reselection() {
    let (fixture, mut state) = setup();
    let bytes = fixture.source_bytes();
    let previous = state.selection.clone().unwrap();
    let pending = state
        .dispatch(1, &description_call(PATH))
        .body_reader()
        .variant_string()
        .unwrap();
    assert!(pending.contains("Selection appearance awaiting redraw"));
    state.selection.as_mut().unwrap().visual_lod =
        Some(datum_gui_render::SelectionVisualLod::ExactMask {
            panes: 2,
            minimum_cues: 100_000,
        });
    let description = state
        .dispatch(2, &description_call(PATH))
        .body_reader()
        .variant_string()
        .unwrap();
    assert!(description.contains("Exact mask · 100000 minimum · 2 panes"));
    let next = state.selection.clone().unwrap();
    let events = messages(|| 1, "", Some(&previous), Some(&next), &state);
    assert_eq!(
        events.len(),
        1,
        "one presentation update, no member selection event"
    );
    assert_eq!(
        next.view.as_ref().unwrap().members,
        previous.view.as_ref().unwrap().members
    );
    assert_eq!(
        next.view.as_ref().unwrap().subject,
        previous.view.as_ref().unwrap().subject
    );
    state.selection.as_mut().unwrap().visual_lod =
        Some(datum_gui_render::SelectionVisualLod::Unavailable);
    assert!(
        state
            .dispatch(3, &description_call(PATH))
            .body_reader()
            .variant_string()
            .unwrap()
            .contains("unavailable projection")
    );
    fixture::Fixture::assert_bytes(&bytes);
}

#[test]
fn related_parent_is_disclosed_without_becoming_an_accessible_selected_member() {
    let mut f = fixture::Fixture::new("accessible-related-parent");
    let package = fixture::id(610);
    let pad = fixture::id(611);
    f.commit([Operation::CreateBoardPackage {
        package_id: package,
        package: serde_json::json!({"uuid":package,"reference":"U1","value":"","position":{"x":0,"y":0},"rotation":0,"layer":1,"locked":false}),
        materialized: serde_json::json!({}),
    }]);
    let source_pad: eda_engine::board::PlacedPad = serde_json::from_value(serde_json::json!({"uuid":pad,"package":package,"name":"1","net":f.net,"position":{"x":0,"y":0},"layer":1})).unwrap();
    let write = eda_engine::api::native_write::board_routing::build_place_board_pad(
        &f.model,
        fixture::provenance(),
        &source_pad,
    )
    .unwrap();
    eda_engine::api::native_write::commit_prepared(&mut f.model, &f.root, write).unwrap();
    let bytes = f.source_bytes();
    let authority = NativeSelectionResolution::capture(&f.model).unwrap();
    let child = authority
        .authored_identities()
        .iter()
        .find(|id| id.id == pad)
        .unwrap()
        .clone();
    let mut scene = datum_gui_protocol::load_fixture_workspace_state().scene;
    scene.project_uuid = f.model.project.project_id.to_string();
    scene.board_uuid = f.board.to_string();
    scene.source_revision = f.model.model_revision.0.clone();
    let view = build_selection_view(
        &f.model,
        &authority,
        SelectionSubject::Object(child.clone()),
        &scene,
        None,
        |_| false,
    )
    .unwrap();
    assert_eq!(view.members, std::collections::BTreeSet::from([child]));
    assert_eq!(view.related.len(), 1);
    assert_eq!(view.related.first().unwrap().id, package);
    let mut state = ServiceState::new(
        crate::terminal_accessibility_platform::worker::empty_snapshot(),
        true,
        Vec::new(),
    );
    state.selection = Some(SelectionAccessibilitySnapshot {
        view: Some(Arc::new(view)),
        inspection: None,
        visual_lod: None,
        focused: true,
        evaluating: None,
        pane: None,
    });
    let reply = state.dispatch(1, &description_call(PATH));
    assert!(
        reply
            .body_reader()
            .variant_string()
            .unwrap()
            .contains("1 related identities keep normal authored appearance and are not selected")
    );
    let mut body = BodyWriter::new();
    body.string(ACCESSIBLE);
    body.string("ChildCount");
    let reply = state.dispatch(2, &call(MEMBERS, PROPERTIES, "Get", "ss", body.finish()));
    assert_eq!(reply.body_reader().variant_i32().unwrap(), 1);
    fixture::Fixture::assert_bytes(&bytes);
}

#[test]
fn quantity_accessibility_keeps_canonical_value_and_discloses_project_rounding() {
    let (mut f, mut state) = setup();
    use eda_engine::api::native_write;
    use eda_engine::ir::units::{
        PreFeatureProjectUnitsMigration, migrate_pre_feature_project_units,
    };
    let PreFeatureProjectUnitsMigration::Create { profile, receipt } =
        migrate_pre_feature_project_units(None).unwrap()
    else {
        panic!("fresh receipt")
    };
    let write = native_write::project::build_initialize_project_display_units(
        &f.model,
        native_write::WriteProvenance::new(
            "selection-test",
            eda_engine::substrate::CommitSource::Test,
            "actual preconfigured Project units",
        ),
        profile,
        &receipt,
    )
    .unwrap();
    native_write::commit_prepared(&mut f.model, &f.root, write).unwrap();
    let before = f.source_bytes();
    let authority = NativeSelectionResolution::capture(&f.model).unwrap();
    let identity = authority
        .authored_identities()
        .iter()
        .find(|id| id.class == Class::Track)
        .unwrap()
        .clone();
    let mut scene = datum_gui_protocol::load_fixture_workspace_state().scene;
    scene.project_uuid = f.model.project.project_id.to_string();
    scene.board_uuid = f.board.to_string();
    scene.source_revision = f.model.model_revision.0.clone();
    let mut view = build_selection_view(
        &f.model,
        &authority,
        SelectionSubject::Object(identity),
        &scene,
        None,
        |_| true,
    )
    .unwrap();
    view.scope = Some(Class::Track);
    state.selection.as_mut().unwrap().view = Some(Arc::new(view));
    let reply = state.dispatch(1, &description_call(&field_path("copper_width_nm")));
    let value = reply.body_reader().variant_string().unwrap();
    assert_eq!(value, "0.000mm (rounded)");
    let exact = attributes(&mut state, &field_path("copper_width_nm"));
    assert_eq!(exact["canonical-value"], "3");
    assert_eq!(exact["value-state"], "common");
    assert_eq!(
        f.source_bytes(),
        before,
        "accessibility does not author units or geometry"
    );
}
