//! Reopened canonical journal authority stays distinct from materialization/preview.
use super::*;
use eda_engine::substrate::{
    CommitProvenance, CommitSource, Operation, OperationBatch, ProjectResolver,
};
use std::{path::Path, sync::Arc};

#[test]
fn drc_report_retains_profile_for_rerun_and_never_presents_scoped_erc() {
    use crate::check_runs::producer::CheckProducerProfile as P;
    use eda_engine::api::native_write::genesis::{GenesisSpec, bootstrap_native_project};
    let root = std::env::temp_dir().join(format!("datum-inspector-drc-{}", Uuid::new_v4()));
    bootstrap_native_project(
        &root,
        GenesisSpec {
            project_name: "DRC inspector".into(),
            existing_ids: None,
        },
    )
    .unwrap();
    let model = ProjectResolver::new(&root).resolve().unwrap();
    let mut state = crate::load_fixture_workspace_state();
    state.project_unopened = false;
    state.ui.layout = crate::WorkspaceLayout::single();
    state.scene.project_uuid = model.project.project_id.to_string();
    state.scene.source_revision = model.model_revision.0.clone();
    state.schematic_documents.project = Some(model.project.project_id);
    state.library = crate::library_browser::LibraryBrowser::capture(Arc::new(model));
    state.backing = Some(crate::WorkspaceBacking {
        request: crate::LiveReviewRequest {
            project_root: root.clone(),
            board_file: None,
            artifact_path: None,
            net_uuid: None,
            from_anchor_pad_uuid: None,
            to_anchor_pad_uuid: None,
            profile: None,
            kicad_board_source: None,
        },
        board_path: root.join("board/board.json"),
        native_write_verbs: Default::default(),
    });
    // Presentation control, not persisted DRC evidence. Actual source/evidence
    // production and cold readback are exercised by CLI and ordinary App proof.
    state.checks = crate::CheckRunReviewState {
        project_id: Some(state.scene.project_uuid.clone()),
        model_revision: Some(state.scene.source_revision.clone()),
        check_run_id: Some(Uuid::new_v4().to_string()),
        profile_id: Some("drc".into()),
        native_drc_zone_copper_basis: Some(
            state
                .library
                .basis
                .as_ref()
                .unwrap()
                .native_drc_zone_copper_basis()
                .unwrap(),
        ),
        persisted: true,
        coverage: vec![crate::CheckRunCoverageSummary {
            domain: "drc".into(),
            rule_id: "connectivity".into(),
            status: "evaluated".into(),
            target_scope: "board".into(),
            ..Default::default()
        }],
        ..Default::default()
    };
    let context = InspectorContext::capture(&state);
    assert_eq!(context.rerun_profile, Some(P::Drc));
    assert!(context.rerun_available);
    let checks = context
        .sections
        .iter()
        .find(|s| s.id == InspectorSectionId::Checks)
        .unwrap();
    assert!(checks.fields.iter().any(|f| f.label == "DRC coverage"));
    assert!(!checks.fields.iter().any(|f| f.label.contains("ERC")));
    let header = pane_header(&state, crate::PaneId(0), PaneContent::Board)
        .unwrap()
        .1;
    assert!(header.contains("DRC 0 Project findings"));
    assert!(!header.contains("ERC"));
    state.set_tool(crate::WorkspaceTool::Move);
    state.begin_authoring_gesture(crate::PointNm { x: 0, y: 0 }, None);
    assert!(!InspectorContext::capture(&state).rerun_available);
    state.cancel_authoring_gesture();
    state.checks.profile_id = Some("manufacturing".into());
    let context = InspectorContext::capture(&state);
    assert!(context.rerun_profile.is_none());
    assert!(
        !context.rerun_available,
        "unsupported profile cannot silently rerun ERC"
    );
    state.checks.profile_id = Some("erc".into());
    assert_eq!(
        InspectorContext::capture(&state).rerun_profile,
        Some(P::Erc)
    );
    for (displayed, attempted) in [(P::Erc, P::Drc), (P::Drc, P::Erc)] {
        state.checks.profile_id = Some(displayed.id().into());
        let request = crate::CheckProducerRequest::for_profile(&state, attempted).unwrap();
        state.check_producer.begin(request.clone());
        let header = pane_header(&state, crate::PaneId(0), PaneContent::Board)
            .unwrap()
            .1;
        assert!(header.contains(&format!("{} starting", attempted.label())));
        state
            .check_producer
            .refuse("Presentation control: attempt refused".into());
        let header = pane_header(&state, crate::PaneId(0), PaneContent::Board)
            .unwrap()
            .1;
        assert!(header.contains(&format!("{} 0 Project findings", displayed.label())));
        assert!(
            !header.contains(attempted.label()),
            "a failed attempt cannot relabel displayed report facts"
        );
        state.check_producer.phase = crate::CheckProducerPhase::Complete;
        let header = pane_header(&state, crate::PaneId(0), PaneContent::Board)
            .unwrap()
            .1;
        assert!(header.contains(&format!("{} 0 Project findings", displayed.label())));
    }
    let logical_target = Uuid::new_v4();
    state.check_producer = Default::default();
    state.checks.profile_id = Some("drc".into());
    state.checks.finding_count = 1;
    state.checks.findings = vec![crate::CheckFindingSummary {
        domain: "drc".into(),
        severity: "warning".into(),
        status: "active".into(),
        code: "connectivity_unrouted_net".into(),
        rule_id: "connectivity".into(),
        fingerprint: "recorded-net-control".into(),
        primary_target: serde_json::json!({"kind":"object_uuid","id":logical_target}),
        message: "Net SIG has two disconnected terminal copper components".into(),
        ..Default::default()
    }];
    let assert_message = |state: &crate::ReviewWorkspaceState, historical: bool| {
        let context = InspectorContext::capture(state);
        let fields = &context
            .sections
            .iter()
            .find(|s| s.id == InspectorSectionId::Checks)
            .unwrap()
            .fields;
        assert!(fields.iter().any(|f| {
            f.label == "Finding message"
                && context
                    .field_text(f, usize::MAX)
                    .contains("Net SIG has two disconnected")
        }));
        assert!(fields.iter().any(|f| {
            f.label == "Finding target"
                && context
                    .field_text(f, usize::MAX)
                    .contains(&logical_target.to_string())
        }));
        assert!(fields.iter().any(|f| f.label
            == if historical {
                "Historical finding"
            } else {
                "Finding"
            }));
    };
    assert_message(&state, false);
    state.checks.finding_count = 2;
    let context = InspectorContext::capture(&state);
    assert!(
        context
            .sections
            .iter()
            .flat_map(|s| &s.fields)
            .any(|f| f.label == "Finding details"
                && context.field_text(f, usize::MAX).contains("1 of 2"))
    );
    state.scene.source_revision = "historical-control".into();
    assert_message(&state, true);
    std::fs::remove_dir_all(root).unwrap();
}

fn copy_source(from: &Path, to: &Path) {
    std::fs::create_dir_all(to).unwrap();
    for entry in std::fs::read_dir(from).unwrap() {
        let entry = entry.unwrap();
        if entry.file_name() == ".datum" {
            continue;
        }
        assert!(!entry.file_type().unwrap().is_symlink());
        let target = to.join(entry.file_name());
        if entry.file_type().unwrap().is_dir() {
            copy_source(&entry.path(), &target);
        } else {
            std::fs::copy(entry.path(), target).unwrap();
        }
    }
}
#[test]
fn persisted_journal_over_stale_shard_is_saved_but_editor_activity_is_not() {
    let repo = Path::new(env!("CARGO_MANIFEST_DIR")).join("../..");
    let input = repo.join("crates/test-harness/testdata/selection/s5a_v1/replay-inputs/native/s5a-c03-u5-semantic-v1/variants/semantic-scope");
    let root = repo
        .join(".datum/scratch/gui-p2-inspector-insp01-20261023/persistence-control")
        .join(Uuid::new_v4().to_string());
    copy_source(&input, &root);
    let original = std::fs::read(root.join("project.json")).unwrap();
    let mut model = ProjectResolver::new(&root).resolve().unwrap();
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
    let mut normal =
        crate::workspace_loading::project_board_editor_workspace(&request, Arc::new(model.clone()))
            .unwrap();
    normal.inspector_context.activity = Some(InspectorActivity::Quiet);
    assert_eq!(
        saved_state(&normal),
        "Saved captured canonical basis",
        "normal reopened durable source has a positive saved state"
    );
    normal.scene.source_revision.push_str("-stale");
    assert!(saved_state(&normal).starts_with("Unavailable"));
    model
        .commit_journaled(
            &root,
            OperationBatch {
                batch_id: Uuid::new_v4(),
                expected_model_revision: Some(model.model_revision.clone()),
                provenance: CommitProvenance {
                    actor: "unit-test".into(),
                    source: CommitSource::Test,
                    reason: "existing journal replay materialization control".into(),
                },
                operations: vec![Operation::SetProjectName {
                    project_id: model.project.project_id,
                    name: "Durable journal replay control".into(),
                }],
            },
        )
        .unwrap();
    // Same stale-materialization negative control as the engine journal test.
    // The journal itself remains durable and unchanged by inspection below.
    std::fs::write(root.join("project.json"), &original).unwrap();
    let model = Arc::new(ProjectResolver::new(&root).resolve().unwrap());
    let journal = eda_engine::substrate::transaction_journal_path(&root);
    let journal_before = std::fs::read(&journal).unwrap();
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
    assert!(state.source_shards.dirty > 0);
    state.inspector_context.activity = Some(InspectorActivity::Quiet);
    assert_eq!(saved_state(&state), "Saved captured canonical basis");
    assert!(materialization_state(&state).contains("differences do not imply unsaved edits"));
    let header = pane_header(
        &state,
        state.ui.layout.focused,
        state.ui.layout.focused_content(),
    )
    .unwrap();
    assert!(header.1.contains("Saved captured canonical basis"));
    for activity in [
        InspectorActivity::Pending,
        InspectorActivity::Preview,
        InspectorActivity::Unconfirmed,
    ] {
        state.inspector_context.activity = Some(activity);
        assert!(!saved_state(&state).starts_with("Saved"));
        assert!(!InspectorContext::capture(&state).rerun_available);
    }
    state.inspector_context.activity = None;
    assert!(saved_state(&state).starts_with("Unavailable"));
    state.inspector_context.activity = Some(InspectorActivity::Quiet);
    state.supervision.journal.accepted_transaction_tip = None;
    assert!(saved_state(&state).starts_with("Unavailable"));
    assert_eq!(std::fs::read(journal).unwrap(), journal_before);
    assert_eq!(std::fs::read(root.join("project.json")).unwrap(), original);
    std::fs::remove_dir_all(root).unwrap();
}
