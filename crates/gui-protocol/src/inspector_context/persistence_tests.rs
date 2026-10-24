//! Reopened canonical journal authority stays distinct from materialization/preview.
use super::*;
use eda_engine::substrate::{
    CommitProvenance, CommitSource, Operation, OperationBatch, ProjectResolver,
};
use std::{path::Path, sync::Arc};

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
