//! Reuse the admitted serialized hierarchy rather than synthesizing source IDs.
use super::*;
use crate::{PaneContent, WorkspaceLayout};
// ProjectResolver retains the canonical writer lease in each captured model.
// These two read-only cases deliberately share one pinned source directory.
static SOURCE_LEASE: std::sync::Mutex<()> = std::sync::Mutex::new(());

#[test]
fn repeated_source_occurrences_inspect_separately_and_open_in_the_recursive_tree() {
    let _lease = SOURCE_LEASE.lock().unwrap();
    let root = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join(
        "../test-harness/testdata/selection/s5a_v1/replay-inputs/native/s5a-c03-u5-semantic-v1/variants/semantic-scope",
    );
    let model = eda_engine::substrate::ProjectResolver::new(&root)
        .resolve()
        .unwrap();
    let mut layout = WorkspaceLayout::board_schematic();
    let mut documents =
        crate::native_schematic_scene::project_native_schematic_documents(&model, &root, &layout)
            .unwrap()
            .unwrap();
    let source = Uuid::parse_str("73000000-0000-4000-8000-000000000032").unwrap();
    let children: Vec<_> = documents
        .rows
        .iter()
        .filter(|row| row.key.source_sheet == source)
        .map(|row| row.key.clone())
        .collect();
    assert_eq!(children.len(), 2);
    assert_ne!(children[0].instance_path, children[1].instance_path);
    assert_eq!(documents.rows.len(), 3);
    let before_layout = layout.clone();
    let before_panes = documents.panes.clone();
    assert!(documents.inspect(&children[0]));
    assert_eq!(layout, before_layout);
    assert_eq!(documents.panes, before_panes);
    assert_eq!(documents.inspected.as_ref(), Some(&children[0]));

    documents
        .prepare_scene(&children[0], &root, &model)
        .unwrap();
    let a = documents.open_beside(&mut layout, &children[0]).unwrap();
    assert_eq!(layout.leaves().len(), 3);
    assert_eq!(layout.focused, a);
    assert_eq!(layout.content_for(PaneId(0)), Some(PaneContent::Board));
    assert_eq!(
        documents.panes.get(&PaneId(1)),
        before_panes.get(&PaneId(1))
    );
    documents
        .prepare_scene(&children[1], &root, &model)
        .unwrap();
    let b = documents.open_beside(&mut layout, &children[1]).unwrap();
    assert_ne!(a, b);
    assert_eq!(layout.leaves().len(), 4);
    let scene_a = documents.scene_for_pane(a).unwrap();
    let scene_b = documents.scene_for_pane(b).unwrap();
    assert_eq!(scene_a.board_uuid, scene_b.board_uuid);
    assert_ne!(scene_a.scene_id, scene_b.scene_id);
    assert_eq!(scene_a.source_revision, model.model_revision.0);
    assert_eq!(scene_b.source_revision, model.model_revision.0);
    let authority =
        crate::selection_resolution::NativeSelectionResolution::capture(&model).unwrap();
    let bindings_a = crate::selection_scene::SceneSelectionBindings::capture(scene_a, &authority);
    let bindings_b = crate::selection_scene::SceneSelectionBindings::capture(scene_b, &authority);
    assert!(bindings_a.iter().next().is_some());
    assert!(bindings_b.iter().next().is_some());
    assert!(
        bindings_a
            .iter()
            .all(|(_, binding)| binding.projected.instance_path == children[0].instance_path)
    );
    assert!(
        bindings_b
            .iter()
            .all(|(_, binding)| binding.projected.instance_path == children[1].instance_path)
    );
    // Repeated primitives retain their occurrence-specific ownership when both
    // documents are compiled into one global selection presentation.
    let (_, binding) = bindings_a.iter().next().unwrap();
    let selected = binding.projected.clone();
    let mut board = crate::load_fixture_workspace_state().scene;
    board.project_uuid = model.project.project_id.to_string();
    board.source_revision = model.model_revision.0.clone();
    let view = crate::selection_view::build_selection_view_for_documents(
        &model,
        &authority,
        crate::SelectionSubject::Object(selected.clone()),
        &board,
        &[scene_a, scene_b],
        |_| true,
    )
    .unwrap();
    assert_eq!(view.members, BTreeSet::from([selected.clone()]));
    let a_keys = view
        .projection_keys(scene_a, PaneContent::Schematic)
        .unwrap();
    let b_keys = view
        .projection_keys(scene_b, PaneContent::Schematic)
        .unwrap();
    assert!(!a_keys.is_empty());
    assert!(b_keys.is_empty());
    for key in a_keys {
        assert_eq!(
            view.presentation_owner(scene_a, PaneContent::Schematic, key),
            Some(&selected)
        );
        assert_eq!(
            view.presentation_owner(scene_b, PaneContent::Schematic, key),
            None
        );
    }
    assert!(!view.unprojected.contains(&selected));
    let mut wrong_document = scene_a.clone();
    wrong_document.schematic_document.as_mut().unwrap().project = Uuid::new_v4();
    assert!(
        view.projection_keys(&wrong_document, PaneContent::Schematic)
            .is_none()
    );
    let mut stale_scene = scene_a.clone();
    stale_scene.source_revision = "stale".into();
    assert!(
        view.projection_keys(&stale_scene, PaneContent::Schematic)
            .is_none()
    );
    assert_eq!(documents.open_beside(&mut layout, &children[0]), Some(a));
    assert_eq!(layout.leaves().len(), 4);

    let mut foreign = children[0].clone();
    foreign.project = Uuid::new_v4();
    let before_layout = layout.clone();
    assert!(!documents.inspect(&foreign));
    assert!(documents.prepare_scene(&foreign, &root, &model).is_err());
    assert_eq!(documents.open_beside(&mut layout, &foreign), None);
    assert_eq!(layout, before_layout);
}

#[test]
fn root_preview_uses_the_live_primary_scene_and_authoring_never_borrows_another_pane() {
    let _lease = SOURCE_LEASE.lock().unwrap();
    let root = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join(
        "../test-harness/testdata/selection/s5a_v1/replay-inputs/native/s5a-c03-u5-semantic-v1/variants/semantic-scope",
    );
    let model = eda_engine::substrate::ProjectResolver::new(&root)
        .resolve()
        .unwrap();
    let mut state = crate::load_fixture_workspace_state();
    state.scene.project_uuid = model.project.project_id.to_string();
    state.scene.source_revision = model.model_revision.0.clone();
    state.library = crate::library_browser::LibraryBrowser::capture(Arc::new(model.clone()));
    state.schematic_authoring =
        crate::schematic_authoring::SchematicAuthoring::from_model(&model).unwrap();
    state.schematic_documents = crate::native_schematic_scene::project_native_schematic_documents(
        &model,
        &root,
        &state.ui.layout,
    )
    .unwrap()
    .unwrap();
    state.schematic_scene = state.schematic_documents.scene_for_pane(PaneId(1)).cloned();
    state.ui.layout.focused = PaneId(1);
    let primary = state.schematic_documents.panes[&PaneId(1)].clone();
    state.schematic_scene.as_mut().unwrap().board_name = "Live root preview".into();
    assert_eq!(
        crate::camera_scene_for_pane(&state, PaneId(1))
            .unwrap()
            .board_name,
        "Live root preview"
    );
    let basis = crate::AuthoringViewBasis::capture(&state, PaneId(1)).unwrap();
    assert!(basis.is_current(&state, PaneId(1)));
    let selected = state.selection.clone();
    let other = state.ui.layout.split_focused_with(
        PaneContent::Schematic,
        crate::SplitOrientation::Vertical,
        false,
    );
    state
        .schematic_documents
        .panes
        .insert(other, primary.clone());
    assert!(!basis.is_current(&state, other));
    state.ui.layout.focused = other;
    assert!(!basis.is_current(&state, other));
    assert!(crate::AuthoringViewBasis::capture(&state, other).is_ok());
    let child = state
        .schematic_documents
        .rows
        .iter()
        .find(|row| !row.key.instance_path.is_empty())
        .unwrap()
        .key
        .clone();
    state
        .schematic_documents
        .prepare_scene(&child, &root, &model)
        .unwrap();
    let child_pane = state
        .schematic_documents
        .open_beside(&mut state.ui.layout, &child)
        .unwrap();
    assert!(crate::AuthoringViewBasis::capture(&state, child_pane).is_err());
    assert!(!basis.is_current(&state, child_pane));
    state.ui.layout.focused = PaneId(1);
    let mut foreign = basis.clone();
    foreign.project = Uuid::new_v4();
    assert!(!foreign.is_current(&state, PaneId(1)));
    state.scene.source_revision = "stale".into();
    assert!(!basis.is_current(&state, PaneId(1)));
    assert_eq!(state.selection, selected);
    // A foreign primary envelope never falls back to the cached root copy.
    state
        .schematic_scene
        .as_mut()
        .unwrap()
        .schematic_document
        .as_mut()
        .unwrap()
        .project = Uuid::new_v4();
    assert!(crate::camera_scene_for_pane(&state, PaneId(1)).is_none());
}

#[test]
fn check_scope_refuses_ambiguous_occurrences_and_accepts_only_explicit_affected_objects() {
    use crate::inspector_context::{InspectorContext, InspectorOwner, InspectorSectionId};
    // These are scope-qualification controls over admitted authored identities;
    // the report below is a test double, not actual ERC or native workflow proof.
    let _lease = SOURCE_LEASE.lock().unwrap();
    let root = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join(
        "../test-harness/testdata/selection/s5a_v1/replay-inputs/native/s5a-c03-u5-semantic-v1/variants/semantic-scope",
    );
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
    let request = crate::LiveReviewRequest {
        project_root: root.clone(),
        board_file: None,
        artifact_path: None,
        net_uuid: None,
        from_anchor_pad_uuid: None,
        to_anchor_pad_uuid: None,
        profile: None,
        kicad_board_source: None,
    };
    let mut state =
        crate::workspace_loading::project_board_editor_workspace(&request, model.clone()).unwrap();
    let children: Vec<_> = state
        .schematic_documents
        .rows
        .iter()
        .filter(|r| !r.key.instance_path.is_empty())
        .map(|r| r.key.clone())
        .collect();
    assert_eq!(children.len(), 2);
    let wire = *state
        .schematic_documents
        .source_sheet(&children[0])
        .unwrap()
        .wires
        .keys()
        .next()
        .unwrap();
    assert_eq!(
        state.schematic_documents.source_occurrences(wire).count(),
        2
    );
    state.inspector_context.owner = InspectorOwner::Navigator;
    state.schematic_documents.inspect(&children[0]);
    state.checks = crate::CheckRunReviewState {
        check_run_id: Some(Uuid::from_u128(1).to_string()),
        project_id: Some(model.project.project_id.to_string()),
        model_revision: Some(model.model_revision.0.clone()),
        profile_id: Some("erc".into()),
        persisted: true,
        finding_count: 1,
        findings: vec![crate::CheckFindingSummary {
            domain: "erc".into(),
            status: "active".into(),
            severity: "warning".into(),
            primary_target: serde_json::json!({"kind":"object_uuid", "id":wire}),
            ..Default::default()
        }],
        coverage: vec![crate::CheckRunCoverageSummary {
            domain: "erc".into(),
            rule_id: "schematic_connectivity".into(),
            status: "evaluated".into(),
            target_scope: "schematic".into(),
            ..Default::default()
        }],
        ..Default::default()
    };
    fn fields(state: &crate::ReviewWorkspaceState) -> Vec<(String, String)> {
        let context = InspectorContext::capture(state);
        context
            .sections
            .iter()
            .filter(|s| s.id == InspectorSectionId::Checks)
            .flat_map(|s| &s.fields)
            .map(|field| {
                (
                    field.label.to_string(),
                    context.field_text(field, usize::MAX),
                )
            })
            .collect()
    }
    let unqualified = fields(&state);
    assert!(
        unqualified
            .iter()
            .any(|(k, v)| k == "Scoped ERC" && v.contains("ambiguous"))
    );
    assert!(!unqualified.iter().any(|(_, v)| v.starts_with("Clean")));
    state.checks.findings[0].primary_target["instance_path"] =
        serde_json::json!(children[0].instance_path);
    assert!(
        fields(&state)
            .iter()
            .any(|(k, v)| k == "Scoped ERC" && v == "0 errors · 1 warnings · 0 infos")
    );
    state.checks.findings[0].primary_target["instance_path"] =
        serde_json::json!(children[1].instance_path);
    assert!(
        fields(&state)
            .iter()
            .any(|(k, v)| k == "Scoped ERC" && v == "0 errors · 0 warnings · 0 infos")
    );
    assert!(
        fields(&state)
            .iter()
            .any(|(_, v)| v == "Clean — schematic_connectivity only")
    );
    // Explicit payload affected objects count. Provenance/parent UUIDs do not.
    let root_key = state
        .schematic_documents
        .rows
        .iter()
        .find(|r| r.key.instance_path.is_empty())
        .unwrap()
        .key
        .clone();
    let root_id = state
        .schematic_documents
        .source_sheet(&root_key)
        .unwrap()
        .uuid;
    assert_eq!(
        state
            .schematic_documents
            .source_occurrences(root_id)
            .count(),
        1
    );
    state.checks.findings[0].primary_target =
        serde_json::json!({"kind":"object_uuid", "id":root_id});
    state.checks.findings[0].payload =
        serde_json::json!({"symbol_uuid":wire,"part":wire,"lib_id":wire});
    assert!(
        fields(&state)
            .iter()
            .any(|(k, v)| k == "Scoped ERC" && v == "0 errors · 0 warnings · 0 infos")
    );
    // A pin_evidence field cannot reinterpret an authored Wire UUID as a Pin.
    state.checks.findings[0].payload = serde_json::json!({"pin_evidence":[{"pin_uuid":wire,"instance_path":children[0].instance_path,"symbol_uuid":root_id}]});
    assert!(
        fields(&state)
            .iter()
            .any(|(k, v)| k == "Scoped ERC" && v.contains("not a captured schematic Pin"))
    );
    state.checks.findings[0].payload = serde_json::json!({"object_uuids":[wire]});
    assert!(
        fields(&state)
            .iter()
            .any(|(k, v)| k == "Scoped ERC" && v.contains("ambiguous"))
    );
    state.checks.findings[0].payload = serde_json::json!({});
    // Empty evaluated runs do not manufacture a scope for an empty Board or
    // an invalid Navigator row.
    let finding = state.checks.findings[0].clone();
    state.checks.findings.clear();
    state.checks.finding_count = 0;
    state.inspector_context.owner = InspectorOwner::Selection;
    state.selection = crate::SelectionTarget::None;
    state.ui.layout.focused = PaneId(0);
    assert!(
        fields(&state)
            .iter()
            .any(|(k, v)| k == "Scoped ERC" && v.contains("no qualified current schematic"))
    );
    assert!(!fields(&state).iter().any(|(_, v)| v.starts_with("Clean")));
    state.inspector_context.owner = InspectorOwner::Navigator;
    for foreign in [true, false] {
        let mut missing = children[0].clone();
        if foreign {
            missing.project = Uuid::nil();
        } else {
            missing.instance_path.push(root_id);
        }
        state.schematic_documents.inspected = Some(missing);
        assert!(
            fields(&state)
                .iter()
                .any(|(k, v)| k == "Scoped ERC" && v.contains("no qualified current schematic"))
        );
        assert!(!fields(&state).iter().any(|(_, v)| v.starts_with("Clean")));
    }
    state.schematic_documents.inspect(&children[0]);
    assert!(
        fields(&state)
            .iter()
            .any(|(_, v)| v == "Clean — schematic_connectivity only")
    );
    state.checks.findings = vec![finding];
    state.checks.finding_count = 1;
    // Schematic coverage cannot establish a zero for selected Board objects,
    // even if related-only metadata points into the schematic.
    use crate::selection_resolution::SelectionAuthority;
    let authority =
        crate::selection_resolution::NativeSelectionResolution::capture(&model).unwrap();
    let board = authority
        .authored_identities()
        .iter()
        .find(|id| id.class == crate::AuthoredSelectionClass::Track)
        .unwrap()
        .clone();
    let child_wire = authority
        .authored_identities()
        .iter()
        .find(|id| id.id == wire && id.instance_path == children[0].instance_path)
        .unwrap()
        .clone();
    state
        .schematic_documents
        .prepare_scene(&children[0], &root, &model)
        .unwrap();
    state
        .schematic_documents
        .open_beside(&mut state.ui.layout, &children[0])
        .unwrap();
    state.inspector_context.owner = InspectorOwner::Selection;
    let mut board_view = crate::selection_artifacts::build_workspace_selection_view(
        &model,
        &authority,
        crate::SelectionSubject::Object(board.clone()),
        &state,
    )
    .unwrap();
    // Deliberate related-only output control; this asserts no CI/engine relation.
    board_view.related.insert(child_wire.clone());
    state.selection = crate::SelectionTarget::Native(Box::new(board_view));
    assert!(
        fields(&state)
            .iter()
            .any(|(k, v)| k == "Scoped ERC" && v.contains("no schematic electrical targets"))
    );
    assert!(!fields(&state).iter().any(|(_, v)| v.starts_with("Clean")));
    let mixed = crate::CompoundSelection::new([board, child_wire.clone()], None).unwrap();
    state.selection = crate::SelectionTarget::Native(Box::new(
        crate::selection_artifacts::build_workspace_selection_view(
            &model,
            &authority,
            crate::SelectionSubject::Compound(mixed),
            &state,
        )
        .unwrap(),
    ));
    let mixed_fields = fields(&state);
    assert!(mixed_fields.iter().any(
        |(k, v)| k == "Scope coverage" && v.starts_with("1 schematic electrical members of 2")
    ));
    assert!(
        mixed_fields
            .iter()
            .any(|(k, v)| k == "Scoped ERC" && v == "0 errors · 0 warnings · 0 infos")
    );
    let membership = authority
        .snapshot()
        .net_for_origin(
            authority.revision(),
            &eda_engine::substrate::ElectricalOccurrence {
                class: "wires".into(),
                source_id: wire,
                instance_path: child_wire.instance_path.clone(),
            },
        )
        .unwrap();
    state.selection = crate::SelectionTarget::Native(Box::new(
        crate::selection_artifacts::build_workspace_selection_view(
            &model,
            &authority,
            crate::SelectionSubject::GlobalNet(membership.subject_id),
            &state,
        )
        .unwrap(),
    ));
    assert!(
        fields(&state)
            .iter()
            .any(|(k, v)| k == "Scoped ERC" && v == "0 errors · 0 warnings · 0 infos")
    );
    state.checks.coverage[0].status = "not_implemented".into();
    assert!(
        fields(&state)
            .iter()
            .any(|(k, v)| k == "Scoped ERC" && v.contains("no evaluated ERC coverage"))
    );
    assert!(!fields(&state).iter().any(|(_, v)| v.starts_with("Clean")));
    state.checks.model_revision = Some("historical".into());
    let stale = fields(&state);
    assert!(
        stale
            .iter()
            .any(|(k, v)| k == "State" && v.starts_with("Stale"))
    );
    assert!(
        stale
            .iter()
            .any(|(k, v)| k == "Historical Project findings" && v == "1")
    );
    assert!(!stale.iter().any(|(k, _)| k == "Scoped ERC"));
    assert_eq!(
        before,
        model
            .source_shards
            .iter()
            .map(|shard| std::fs::read(root.join(&shard.path)).unwrap())
            .collect::<Vec<_>>()
    );
}
