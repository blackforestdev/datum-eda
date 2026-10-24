//! Actual shared-service dispatch over an admitted occurrence tree, without a selection.
use super::*;
use crate::selection_accessibility::{
    SelectionAccessibilitySnapshot, workspace::WorkspaceInspection,
};
use datum_gui_protocol::inspector_context::{InspectorOwner, InspectorSectionId};
use std::sync::Arc;
// Resolving this admitted source takes the engine's normal project lease.
// Keep the two readers on this same fixture in one test-owned lease lane.
static SOURCE_LEASE: std::sync::Mutex<()> = std::sync::Mutex::new(());

fn property(service: &mut ServiceState, path: &str, key: &str) -> Message {
    let mut body = BodyWriter::new();
    body.string(ACCESSIBLE);
    body.string(key);
    service.dispatch(
        1,
        &Message::method_call(7, "ignored", path, PROPERTIES, "Get", "ss", body.finish()),
    )
}
fn states(service: &mut ServiceState, path: &str) -> Vec<u32> {
    let answer = service.dispatch(
        1,
        &Message::method_call(7, "ignored", path, ACCESSIBLE, "GetState", "", vec![]),
    );
    assert_eq!(answer.kind, MessageType::MethodReturn);
    answer.body_reader().u32_array().unwrap()
}

#[test]
fn document_inspection_uses_named_accessible_roots_without_authored_selection() {
    let _lease = SOURCE_LEASE.lock().unwrap();
    let root = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("../test-harness/testdata/selection/s5a_v1/replay-inputs/native/s5a-c03-u5-semantic-v1/variants/semantic-scope");
    let model = Arc::new(
        eda_engine::substrate::ProjectResolver::new(&root)
            .resolve()
            .unwrap(),
    );
    let before: Vec<_> = model
        .source_shards
        .iter()
        .map(|shard| std::fs::read(root.join(&shard.path)).unwrap())
        .collect();
    let request = datum_gui_protocol::LiveReviewRequest {
        project_root: root.clone(),
        board_file: None,
        artifact_path: None,
        net_uuid: None,
        from_anchor_pad_uuid: None,
        to_anchor_pad_uuid: None,
        profile: None,
        kicad_board_source: None,
    };
    let mut editor = datum_gui_protocol::workspace_loading::project_board_editor_workspace(
        &request,
        model.clone(),
    )
    .unwrap();
    let child = editor
        .schematic_documents
        .rows
        .iter()
        .find(|row| !row.key.instance_path.is_empty())
        .unwrap()
        .key
        .clone();
    editor.schematic_documents.inspect(&child);
    editor.inspector_context.owner = InspectorOwner::Navigator;
    editor.ui.focus = datum_gui_protocol::ApplicationFocus::Navigator;
    let mut service = ServiceState::new(
        crate::terminal_accessibility_platform::worker::empty_snapshot(),
        true,
        vec![],
    );
    let publish = |service: &mut ServiceState,
                   editor: &datum_gui_protocol::ReviewWorkspaceState| {
        service.selection = Some(SelectionAccessibilitySnapshot {
            view: None,
            inspection: WorkspaceInspection::capture(editor).map(Arc::new),
            visual_lod: None,
            focused: false,
            evaluating: None,
            pane: Some(editor.ui.layout.focused),
        });
    };
    publish(&mut service, &editor);
    assert!(service.root_child_paths().contains(&NAVIGATOR.to_owned()));
    assert!(service.root_child_paths().contains(&PATH.to_owned()));
    assert_eq!(
        property(&mut service, NAVIGATOR, "Name")
            .body_reader()
            .variant_string()
            .unwrap(),
        "Navigator"
    );
    assert!(
        property(&mut service, PATH, "Name")
            .body_reader()
            .variant_string()
            .unwrap()
            .starts_with("Inspector")
    );
    let path = workspace::document_path(&child);
    let words = states(&mut service, &path);
    assert_ne!(
        words[0] & (1 << 12),
        0,
        "inspected Navigator row has keyboard focus"
    );
    assert_eq!(
        words[0] & (1 << 23),
        0,
        "inspection never publishes authored selected state"
    );
    let snapshot = service.selection.as_ref().unwrap();
    assert!(snapshot.view.is_none());
    let ui = snapshot.inspection.as_ref().unwrap();
    assert_eq!(ui.documents.len(), editor.schematic_documents.rows.len());
    let checks_path = workspace::section_path(InspectorSectionId::Checks);
    let check_section = ui
        .sections
        .iter()
        .find(|s| s.id == InspectorSectionId::Checks)
        .unwrap();
    let run_path = format!("{checks_path}/run_erc");
    assert_eq!(
        children::paths(snapshot, Node::Section(check_section)).count(),
        check_section.fields.len() + 1
    );
    assert_ne!(states(&mut service, &run_path)[0] & (1 << 8), 0);
    editor.ui.focus = datum_gui_protocol::ApplicationFocus::Inspector;
    editor.inspector_context.focused = Some(InspectorSectionId::Checks);
    editor
        .inspector_context
        .collapsed
        .insert(InspectorSectionId::Checks);
    let job = datum_gui_protocol::CheckProducerRequest::from_workspace(&editor).unwrap();
    editor.check_producer.begin(job.clone());
    publish(&mut service, &editor);
    assert_ne!(states(&mut service, &checks_path)[0] & (1 << 12), 0);
    assert_ne!(states(&mut service, &checks_path)[0] & (1 << 5), 0);
    assert_eq!(
        states(&mut service, &run_path)[0] & (1 << 8),
        0,
        "pending job disables explicit rerun"
    );
    assert_eq!(
        states(
            &mut service,
            &workspace::field_path(InspectorSectionId::Checks, 0)
        )[0] & (1 << 30),
        0
    );
    editor.check_producer.accept(job.acceptance());
    publish(&mut service, &editor);
    let snapshot = service.selection.as_ref().unwrap();
    let ui = snapshot.inspection.as_ref().unwrap();
    let section = ui
        .sections
        .iter()
        .find(|section| section.id == InspectorSectionId::Checks)
        .unwrap();
    let field = section
        .fields
        .iter()
        .position(|field| field.label == "Job")
        .unwrap();
    assert!(
        property(
            &mut service,
            &workspace::field_path(InspectorSectionId::Checks, field),
            "Name"
        )
        .body_reader()
        .variant_string()
        .unwrap()
        .contains("Running")
    );
    assert!(matches!(
        editor.selection,
        datum_gui_protocol::SelectionTarget::None
    ));
    for (shard, bytes) in model.source_shards.iter().zip(before) {
        assert_eq!(std::fs::read(root.join(&shard.path)).unwrap(), bytes);
    }
}

fn public_children(service: &mut ServiceState, parent: &str) -> Vec<String> {
    let count = property(service, parent, "ChildCount")
        .body_reader()
        .variant_i32()
        .unwrap();
    (0..count)
        .map(|index| {
            let mut body = BodyWriter::new();
            body.i32(index);
            let answer = service.dispatch(
                1,
                &Message::method_call(
                    7,
                    "ignored",
                    parent,
                    ACCESSIBLE,
                    "GetChildAtIndex",
                    "i",
                    body.finish(),
                ),
            );
            assert_eq!(answer.kind, MessageType::MethodReturn);
            let mut body = answer.body_reader();
            body.string().unwrap();
            body.object_path().unwrap()
        })
        .collect()
}

#[test]
fn refreshed_document_events_reconcile_cached_trees_and_lazy_field_values() {
    let _lease = SOURCE_LEASE.lock().unwrap();
    use datum_gui_protocol::{
        AuthoredSelectionClass, SelectionSubject,
        selection_artifacts::build_workspace_selection_view,
        selection_resolution::NativeSelectionResolution,
    };
    use std::collections::BTreeMap;
    let root = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("../test-harness/testdata/selection/s5a_v1/replay-inputs/native/s5a-c03-u5-semantic-v1/variants/semantic-scope");
    let model = Arc::new(
        eda_engine::substrate::ProjectResolver::new(&root)
            .resolve()
            .unwrap(),
    );
    let before: Vec<_> = model
        .source_shards
        .iter()
        .map(|shard| std::fs::read(root.join(&shard.path)).unwrap())
        .collect();
    let request = datum_gui_protocol::LiveReviewRequest {
        project_root: root.clone(),
        board_file: None,
        artifact_path: None,
        net_uuid: None,
        from_anchor_pad_uuid: None,
        to_anchor_pad_uuid: None,
        profile: None,
        kicad_board_source: None,
    };
    let mut editor = datum_gui_protocol::workspace_loading::project_board_editor_workspace(
        &request,
        model.clone(),
    )
    .unwrap();
    let authority = NativeSelectionResolution::capture(&model).unwrap();
    let wire = authority
        .authored_identities()
        .iter()
        .find(|id| id.class == AuthoredSelectionClass::Wire && id.instance_path.is_empty())
        .unwrap()
        .clone();
    let view =
        build_workspace_selection_view(&model, &authority, SelectionSubject::Object(wire), &editor)
            .unwrap();
    editor.selection = datum_gui_protocol::SelectionTarget::Native(Box::new(view.clone()));
    let mut previous = SelectionAccessibilitySnapshot {
        view: Some(Arc::new(view)),
        inspection: WorkspaceInspection::capture(&editor).map(Arc::new),
        visual_lod: None,
        focused: false,
        evaluating: None,
        pane: Some(editor.ui.layout.focused),
    };
    let root_row = previous
        .inspection
        .as_ref()
        .unwrap()
        .documents
        .iter()
        .find(|row| row.parent.is_none())
        .unwrap()
        .clone();
    // Explicit service-input metadata doubles exercise source-refresh deltas;
    // they are not invented canonical edits or native workflow proof.
    let mut removed_root = root_row.clone();
    removed_root.key.source_sheet = uuid::Uuid::new_v4();
    removed_root.name = "Removed root control".into();
    let mut removed_nested = removed_root.clone();
    removed_nested.key.instance_path.push(uuid::Uuid::new_v4());
    removed_nested.parent = Some(removed_root.key.clone());
    removed_nested.name = "Removed nested control".into();
    Arc::make_mut(previous.inspection.as_mut().unwrap())
        .documents
        .extend([removed_root.clone(), removed_nested.clone()]);
    let mut service = ServiceState::new(
        crate::terminal_accessibility_platform::worker::empty_snapshot(),
        true,
        vec![],
    );
    service.selection = Some(previous.clone());
    let parents: Vec<_> = std::iter::once(NAVIGATOR.to_owned())
        .chain(
            previous
                .inspection
                .as_ref()
                .unwrap()
                .documents
                .iter()
                .map(|row| workspace::document_path(&row.key)),
        )
        .collect();
    let mut cached: BTreeMap<_, _> = parents
        .into_iter()
        .map(|path| {
            let children = public_children(&mut service, &path);
            (path, children)
        })
        .collect();
    let mut next = previous.clone();
    let ui = Arc::make_mut(next.inspection.as_mut().unwrap());
    ui.documents
        .retain(|row| row.key != removed_root.key && row.key != removed_nested.key);
    let retained = ui
        .documents
        .iter_mut()
        .find(|row| row.key == root_row.key)
        .unwrap();
    retained.name = "Renamed retained source control".into();
    retained.description = "New source revision description control".into();
    let mut added_root = root_row.clone();
    added_root.key.source_sheet = uuid::Uuid::new_v4();
    added_root.name = "Added root control".into();
    let mut added_nested = added_root.clone();
    added_nested.key.instance_path.push(uuid::Uuid::new_v4());
    added_nested.parent = Some(added_root.key.clone());
    added_nested.name = "Added nested control".into();
    ui.documents
        .extend([added_root.clone(), added_nested.clone()]);
    let field = "section_start_nm";
    let changed = Arc::make_mut(next.view.as_mut().unwrap());
    let selected_scope = changed.scope;
    let scope = changed
        .inspection
        .iter_mut()
        .find(|scope| scope.scope == selected_scope)
        .unwrap();
    scope.fields.insert(
        field.into(),
        datum_gui_protocol::selection_view::InspectionValue::Common(
            serde_json::json!({"x":1234567,"y":-2345678}),
        ),
    );
    scope.quantities.remove(field);
    scope.display_labels.remove(field);
    assert_eq!(
        next.inspection.as_ref().unwrap().sections,
        previous.inspection.as_ref().unwrap().sections,
        "native field names/references are unchanged while lazy values change"
    );
    assert_eq!(
        next.view.as_ref().unwrap().members,
        previous.view.as_ref().unwrap().members
    );
    assert_eq!(
        next.view.as_ref().unwrap().subject,
        previous.view.as_ref().unwrap().subject
    );
    service.selection = Some(next.clone());
    let mut serial = 1;
    let events = messages(
        || {
            serial += 1;
            serial
        },
        "unit-bus",
        Some(&previous),
        Some(&next),
        &service,
    );
    let mut defunct = Vec::new();
    let mut properties = BTreeMap::new();
    for message in &events {
        let path = message.header.path.as_ref().unwrap();
        let mut body = message.body_reader();
        let detail = body.string().unwrap();
        let index = body.i32().unwrap();
        body.i32().unwrap();
        match message.header.member.as_deref() {
            Some("ChildrenChanged")
                if path == NAVIGATOR || path.starts_with(&format!("{NAVIGATOR}/")) =>
            {
                let mut cursor = crate::terminal_accessibility_platform::body::Cursor::new(
                    &message.body,
                    crate::terminal_accessibility_platform::body::Endian::Little,
                );
                cursor.string().unwrap();
                cursor.i32().unwrap();
                cursor.i32().unwrap();
                assert_eq!(cursor.signature().unwrap(), "(so)");
                cursor.align(8).unwrap();
                cursor.string().unwrap();
                let child = cursor.string().unwrap();
                let siblings = cached.entry(path.clone()).or_default();
                if detail == "remove" {
                    assert_eq!(siblings.remove(index as usize), child);
                } else {
                    assert_eq!(detail, "add");
                    siblings.insert(index as usize, child);
                }
            }
            Some("StateChanged") if detail == "defunct" => {
                assert!(body.variant_bool().unwrap());
                defunct.push(path.clone());
            }
            Some("PropertyChange") => {
                properties.insert((path.clone(), detail), body.variant_string().unwrap());
            }
            _ => {}
        }
    }
    for row in [&removed_root, &removed_nested] {
        let path = workspace::document_path(&row.key);
        assert!(defunct.contains(&path));
        assert_eq!(
            property(&mut service, &path, "Name").kind,
            MessageType::Error
        );
    }
    let current = next.inspection.as_ref().unwrap();
    for parent in std::iter::once(NAVIGATOR.to_owned()).chain(
        current
            .documents
            .iter()
            .map(|row| workspace::document_path(&row.key)),
    ) {
        assert_eq!(
            cached.entry(parent.clone()).or_default(),
            &public_children(&mut service, &parent)
        );
    }
    for (key, property_name) in [
        ("accessible-name", "Name"),
        ("accessible-description", "Description"),
    ] {
        let path = workspace::document_path(&root_row.key);
        assert_eq!(
            properties[&(path.clone(), key.into())],
            property(&mut service, &path, property_name)
                .body_reader()
                .variant_string()
                .unwrap()
        );
    }
    let section = current
        .sections
        .iter()
        .find(|section| {
            section
                .fields
                .iter()
                .any(|value| value.native_field.as_deref() == Some(field))
        })
        .unwrap();
    let index = section
        .fields
        .iter()
        .position(|value| value.native_field.as_deref() == Some(field))
        .unwrap();
    let path = workspace::field_path(section.id, index);
    assert_eq!(
        properties[&(path.clone(), "accessible-name".into())],
        property(&mut service, &path, "Name")
            .body_reader()
            .variant_string()
            .unwrap()
    );
    assert!(messages(|| 1, "unit-bus", Some(&next), Some(&next), &service).is_empty());
    for (shard, bytes) in model.source_shards.iter().zip(before) {
        assert_eq!(std::fs::read(root.join(&shard.path)).unwrap(), bytes);
    }
}
