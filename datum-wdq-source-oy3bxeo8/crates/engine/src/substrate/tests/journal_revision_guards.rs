use super::*;

#[test]
fn commit_rejects_empty_operation_batch_without_mutation() {
    let root = temp_project_root("commit_empty_batch");
    let project_id = Uuid::new_v4();
    let board_id = Uuid::new_v4();
    write_minimal_project(&root, project_id, board_id);
    let mut model = ProjectResolver::new(&root)
        .resolve()
        .expect("resolve should succeed");
    let before = model.model_revision.clone();

    let error = model
        .commit(OperationBatch {
            batch_id: Uuid::new_v5(&project_id, b"commit-empty"),
            expected_model_revision: Some(before.clone()),
            provenance: CommitProvenance {
                actor: "unit-test".to_string(),
                source: CommitSource::Test,
                reason: "prove empty commit guard".to_string(),
            },
            operations: Vec::new(),
        })
        .expect_err("empty commit should fail");

    assert!(error.to_string().contains("has no operations"));
    assert_eq!(model.model_revision, before);
    assert_eq!(model.objects[&board_id].object_revision, ObjectRevision(0));
    assert!(model.journal.is_empty());
}

#[test]
fn commit_journaled_rejects_missing_expected_model_revision_without_staging_or_mutation() {
    let root = temp_project_root("commit_journaled_missing_revision");
    let project_id = Uuid::new_v4();
    let board_id = Uuid::new_v4();
    let package_id = Uuid::new_v4();
    write_project_with_board_package(&root, project_id, board_id, package_id);
    let mut model = ProjectResolver::new(&root)
        .resolve()
        .expect("resolve should succeed");
    let before = model.clone();
    let batch_id = Uuid::new_v5(&project_id, b"journaled-missing-revision");

    let error = model
        .commit_journaled(
            &root,
            OperationBatch {
                batch_id,
                expected_model_revision: None,
                provenance: CommitProvenance {
                    actor: "unit-test".to_string(),
                    source: CommitSource::Test,
                    reason: "prove missing journaled revision guard".to_string(),
                },
                operations: vec![Operation::SetBoardPackageValue {
                    package_id,
                    value: "NEW".to_string(),
                }],
            },
        )
        .expect_err("missing journaled revision guard should fail");

    assert!(
        error
            .to_string()
            .contains("journaled commit requires expected_model_revision")
    );
    assert_eq!(model, before);
    assert!(!transaction_journal_path(&root).exists());
    assert!(
        !root
            .join(".datum/stage")
            .join(batch_id.to_string())
            .exists()
    );
}

#[test]
fn commit_journaled_rejects_stale_object_revision_guard_without_staging_or_mutation() {
    let root = temp_project_root("commit_journaled_stale_object_guard");
    let project_id = Uuid::new_v4();
    let board_id = Uuid::new_v4();
    let package_id = Uuid::new_v4();
    write_project_with_board_package(&root, project_id, board_id, package_id);
    let mut model = ProjectResolver::new(&root)
        .resolve()
        .expect("resolve should succeed");
    let before = model.clone();
    let batch_id = Uuid::new_v5(&project_id, b"journaled-stale-object-guard");

    let error = model
        .commit_journaled(
            &root,
            OperationBatch {
                batch_id,
                expected_model_revision: Some(model.model_revision.clone()),
                provenance: CommitProvenance {
                    actor: "unit-test".to_string(),
                    source: CommitSource::Test,
                    reason: "prove stale object revision guard".to_string(),
                },
                operations: vec![
                    Operation::GuardObjectRevision {
                        object_id: package_id,
                        expected_object_revision: ObjectRevision(99),
                    },
                    Operation::SetBoardPackageValue {
                        package_id,
                        value: "NEW".to_string(),
                    },
                ],
            },
        )
        .expect_err("stale object revision guard should fail");

    assert!(error.to_string().contains("object revision mismatch"));
    assert_eq!(model, before);
    assert!(!transaction_journal_path(&root).exists());
    assert!(
        !root
            .join(".datum/stage")
            .join(batch_id.to_string())
            .exists()
    );
}

#[test]
fn commit_journaled_accepts_matching_object_revision_guard_and_bumps_object_revision() {
    let root = temp_project_root("commit_journaled_matching_object_guard");
    let project_id = Uuid::new_v4();
    let board_id = Uuid::new_v4();
    let package_id = Uuid::new_v4();
    write_project_with_board_package(&root, project_id, board_id, package_id);
    let mut model = ProjectResolver::new(&root)
        .resolve()
        .expect("resolve should succeed");
    let before_revision = model.objects[&package_id].object_revision;

    model
        .commit_journaled(
            &root,
            OperationBatch {
                batch_id: Uuid::new_v5(&project_id, b"journaled-matching-object-guard"),
                expected_model_revision: Some(model.model_revision.clone()),
                provenance: CommitProvenance {
                    actor: "unit-test".to_string(),
                    source: CommitSource::Test,
                    reason: "prove matching object revision guard".to_string(),
                },
                operations: vec![
                    Operation::GuardObjectRevision {
                        object_id: package_id,
                        expected_object_revision: before_revision,
                    },
                    Operation::SetBoardPackageValue {
                        package_id,
                        value: "NEW".to_string(),
                    },
                ],
            },
        )
        .expect("matching object revision guard should commit");

    assert_eq!(
        model.objects[&package_id].object_revision,
        ObjectRevision(before_revision.0 + 1)
    );
    assert_eq!(model.journal.len(), 1);
    assert_eq!(model.journal[0].diff.modified, vec![package_id]);
    assert_eq!(
        model.journal[0].operations,
        vec![Operation::SetBoardPackageValue {
            package_id,
            value: "NEW".to_string(),
        }]
    );
}

#[test]
fn commit_journal_undo_rejects_stale_in_memory_cursor_without_append() {
    let root = temp_project_root("commit_undo_rejects_stale_cursor");
    let project_id = Uuid::new_v4();
    let board_id = Uuid::new_v4();
    let package_id = Uuid::new_v4();
    write_project_with_board_package(&root, project_id, board_id, package_id);
    let mut model = ProjectResolver::new(&root)
        .resolve()
        .expect("resolve should succeed");
    let before = model.model_revision.clone();
    model
        .commit_journaled(
            &root,
            OperationBatch {
                batch_id: Uuid::new_v5(&project_id, b"stale-cursor-source"),
                expected_model_revision: Some(before),
                provenance: CommitProvenance {
                    actor: "unit-test".to_string(),
                    source: CommitSource::Test,
                    reason: "seed stale cursor undo".to_string(),
                },
                operations: vec![Operation::SetBoardPackageValue {
                    package_id,
                    value: "NEW".to_string(),
                }],
            },
        )
        .expect("journaled value commit should succeed");
    model.journal_cursor.applied_transaction_count = 0;
    let journal_before = model.journal.len();

    let error = model
        .commit_journal_undo(
            &root,
            CommitProvenance {
                actor: "unit-test".to_string(),
                source: CommitSource::Test,
                reason: "undo should reject stale cursor".to_string(),
            },
        )
        .expect_err("undo should reject stale in-memory cursor");

    assert!(error.to_string().contains("journal cursor mismatch"));
    assert_eq!(model.journal.len(), journal_before);
}

#[test]
fn commit_journal_redo_rejects_stale_in_memory_cursor_without_append() {
    let root = temp_project_root("commit_redo_rejects_stale_cursor");
    let project_id = Uuid::new_v4();
    let board_id = Uuid::new_v4();
    let package_id = Uuid::new_v4();
    write_project_with_board_package(&root, project_id, board_id, package_id);
    let mut model = ProjectResolver::new(&root)
        .resolve()
        .expect("resolve should succeed");
    let before = model.model_revision.clone();
    model
        .commit_journaled(
            &root,
            OperationBatch {
                batch_id: Uuid::new_v5(&project_id, b"stale-cursor-redo-source"),
                expected_model_revision: Some(before),
                provenance: CommitProvenance {
                    actor: "unit-test".to_string(),
                    source: CommitSource::Test,
                    reason: "seed stale cursor redo".to_string(),
                },
                operations: vec![Operation::SetBoardPackageValue {
                    package_id,
                    value: "NEW".to_string(),
                }],
            },
        )
        .expect("journaled value commit should succeed");
    model
        .commit_journal_undo(
            &root,
            CommitProvenance {
                actor: "unit-test".to_string(),
                source: CommitSource::Test,
                reason: "seed redoable undo".to_string(),
            },
        )
        .expect("undo should succeed");
    model.journal_cursor.applied_transaction_count = 1;
    let journal_before = model.journal.len();

    let error = model
        .commit_journal_redo(
            &root,
            CommitProvenance {
                actor: "unit-test".to_string(),
                source: CommitSource::Test,
                reason: "redo should reject stale cursor".to_string(),
            },
        )
        .expect_err("redo should reject stale in-memory cursor");

    assert!(error.to_string().contains("journal cursor mismatch"));
    assert_eq!(model.journal.len(), journal_before);
}
