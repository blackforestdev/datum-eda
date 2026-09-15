use std::io::Write;

use super::*;
use crate::ir::serialization::to_json_deterministic;

#[test]
fn commit_journaled_records_inverse_operations_in_reverse_batch_order() {
    let root = temp_project_root("commit_inverse_order");
    let project_id = Uuid::new_v4();
    let board_id = Uuid::new_v4();
    let package_id = Uuid::new_v4();
    write_project_with_board_package(&root, project_id, board_id, package_id);
    let mut model = ProjectResolver::new(&root)
        .resolve()
        .expect("resolve should succeed");
    let before = model.model_revision.clone();

    let report = model
        .commit_journaled(
            &root,
            OperationBatch {
                batch_id: Uuid::new_v5(&project_id, b"inverse-order"),
                expected_model_revision: Some(before),
                provenance: CommitProvenance {
                    actor: "unit-test".to_string(),
                    source: CommitSource::Test,
                    reason: "prove inverse operation ordering".to_string(),
                },
                operations: vec![
                    Operation::SetBoardPackageValue {
                        package_id,
                        value: "MID".to_string(),
                    },
                    Operation::SetBoardPackageValue {
                        package_id,
                        value: "NEW".to_string(),
                    },
                ],
            },
        )
        .expect("journaled value commit should succeed");

    assert_eq!(
        report.transaction.inverse_operations,
        vec![
            Operation::SetBoardPackageValue {
                package_id,
                value: "MID".to_string(),
            },
            Operation::SetBoardPackageValue {
                package_id,
                value: "OLD".to_string(),
            },
        ]
    );
}

#[test]
fn commit_journaled_rejects_empty_operation_batch_without_disk_mutation() {
    let root = temp_project_root("commit_journaled_empty_batch");
    let project_id = Uuid::new_v4();
    let board_id = Uuid::new_v4();
    let package_id = Uuid::new_v4();
    write_project_with_board_package(&root, project_id, board_id, package_id);
    let mut model = ProjectResolver::new(&root)
        .resolve()
        .expect("resolve should succeed");
    let before = model.model_revision.clone();
    let batch_id = Uuid::new_v5(&project_id, b"journaled-empty");

    let error = model
        .commit_journaled(
            &root,
            OperationBatch {
                batch_id,
                expected_model_revision: Some(before.clone()),
                provenance: CommitProvenance {
                    actor: "unit-test".to_string(),
                    source: CommitSource::Test,
                    reason: "prove empty journaled guard".to_string(),
                },
                operations: Vec::new(),
            },
        )
        .expect_err("empty journaled commit should fail");

    assert!(error.to_string().contains("has no operations"));
    assert_eq!(model.model_revision, before);
    assert!(model.journal.is_empty());
    assert!(!transaction_journal_path(&root).exists());
    assert!(
        !root
            .join(".datum/stage")
            .join(batch_id.to_string())
            .exists()
    );
    assert!(!root.join(".datum/journal/cursor.json").exists());
}

#[test]
fn commit_journal_undo_and_redo_append_compensating_transactions() {
    let root = temp_project_root("commit_undo_redo");
    let project_id = Uuid::new_v4();
    let board_id = Uuid::new_v4();
    let package_id = Uuid::new_v4();
    write_project_with_board_package(&root, project_id, board_id, package_id);
    let mut model = ProjectResolver::new(&root)
        .resolve()
        .expect("resolve should succeed");
    let before = model.model_revision.clone();

    let original = model
        .commit_journaled(
            &root,
            OperationBatch {
                batch_id: Uuid::new_v5(&project_id, b"undo-redo-set-value"),
                expected_model_revision: Some(before),
                provenance: CommitProvenance {
                    actor: "unit-test".to_string(),
                    source: CommitSource::Test,
                    reason: "seed undo/redo transaction".to_string(),
                },
                operations: vec![Operation::SetBoardPackageValue {
                    package_id,
                    value: "NEW".to_string(),
                }],
            },
        )
        .expect("journaled value commit should succeed");

    let undo = model
        .commit_journal_undo(
            &root,
            CommitProvenance {
                actor: "unit-test".to_string(),
                source: CommitSource::Test,
                reason: "undo value edit".to_string(),
            },
        )
        .expect("undo should append compensating transaction");
    assert_eq!(undo.transaction.transaction_kind, TransactionKind::Undo);
    assert_eq!(
        undo.transaction.undo_of,
        Some(original.transaction.transaction_id)
    );
    assert_eq!(undo.transaction.redo_of, None);
    assert_eq!(
        undo.transaction.operations,
        vec![Operation::SetBoardPackageValue {
            package_id,
            value: "OLD".to_string(),
        }]
    );
    assert_eq!(
        undo.transaction.inverse_operations,
        vec![Operation::SetBoardPackageValue {
            package_id,
            value: "NEW".to_string(),
        }]
    );
    let board_value = read_json_value(&root.join("board/board.json")).expect("board should read");
    assert_eq!(
        board_value["packages"][package_id.to_string()]["value"],
        "OLD"
    );

    let redo = model
        .commit_journal_redo(
            &root,
            CommitProvenance {
                actor: "unit-test".to_string(),
                source: CommitSource::Test,
                reason: "redo value edit".to_string(),
            },
        )
        .expect("redo should append compensating transaction");
    assert_eq!(redo.transaction.transaction_kind, TransactionKind::Redo);
    assert_eq!(redo.transaction.undo_of, None);
    assert_eq!(
        redo.transaction.redo_of,
        Some(undo.transaction.transaction_id)
    );
    assert_eq!(
        redo.transaction.operations,
        vec![Operation::SetBoardPackageValue {
            package_id,
            value: "NEW".to_string(),
        }]
    );
    let board_value = read_json_value(&root.join("board/board.json")).expect("board should read");
    assert_eq!(
        board_value["packages"][package_id.to_string()]["value"],
        "NEW"
    );
    assert_eq!(model.journal.len(), 3);
}

#[test]
fn resolver_preserves_undo_redo_links_across_reopen() {
    let root = temp_project_root("commit_undo_redo_links_reopen");
    let project_id = Uuid::new_v4();
    let board_id = Uuid::new_v4();
    let package_id = Uuid::new_v4();
    write_project_with_board_package(&root, project_id, board_id, package_id);
    let mut model = ProjectResolver::new(&root)
        .resolve()
        .expect("resolve should succeed");
    let before = model.model_revision.clone();
    let original = model
        .commit_journaled(
            &root,
            OperationBatch {
                batch_id: Uuid::new_v5(&project_id, b"undo-redo-links"),
                expected_model_revision: Some(before),
                provenance: CommitProvenance {
                    actor: "unit-test".to_string(),
                    source: CommitSource::Test,
                    reason: "seed undo/redo link transaction".to_string(),
                },
                operations: vec![Operation::SetBoardPackageValue {
                    package_id,
                    value: "NEW".to_string(),
                }],
            },
        )
        .expect("journaled value commit should succeed");
    let undo = model
        .commit_journal_undo(
            &root,
            CommitProvenance {
                actor: "unit-test".to_string(),
                source: CommitSource::Test,
                reason: "undo linked value edit".to_string(),
            },
        )
        .expect("undo should append compensating transaction");
    let redo = model
        .commit_journal_redo(
            &root,
            CommitProvenance {
                actor: "unit-test".to_string(),
                source: CommitSource::Test,
                reason: "redo linked value edit".to_string(),
            },
        )
        .expect("redo should append compensating transaction");

    let reopened = ProjectResolver::new(&root)
        .resolve()
        .expect("reopen should preserve transaction links");
    assert_eq!(reopened.journal.len(), 3);
    assert_eq!(reopened.journal[1].transaction_kind, TransactionKind::Undo);
    assert_eq!(
        reopened.journal[1].undo_of,
        Some(original.transaction.transaction_id)
    );
    assert_eq!(reopened.journal[2].transaction_kind, TransactionKind::Redo);
    assert_eq!(
        reopened.journal[2].redo_of,
        Some(undo.transaction.transaction_id)
    );
    assert_eq!(
        reopened.journal[2].transaction_id,
        redo.transaction.transaction_id
    );
}

#[test]
fn normal_commit_after_undo_clears_redo_stack() {
    let root = temp_project_root("normal_commit_after_undo_clears_redo");
    let project_id = Uuid::new_v4();
    let board_id = Uuid::new_v4();
    let package_id = Uuid::new_v4();
    write_project_with_board_package(&root, project_id, board_id, package_id);
    let mut model = ProjectResolver::new(&root)
        .resolve()
        .expect("resolve should succeed");

    model
        .commit_journaled(
            &root,
            OperationBatch {
                batch_id: Uuid::new_v5(&project_id, b"branch-redo-original"),
                expected_model_revision: Some(model.model_revision.clone()),
                provenance: CommitProvenance {
                    actor: "unit-test".to_string(),
                    source: CommitSource::Test,
                    reason: "seed redo branch original".to_string(),
                },
                operations: vec![Operation::SetBoardPackageValue {
                    package_id,
                    value: "NEW".to_string(),
                }],
            },
        )
        .expect("original commit should succeed");
    model
        .commit_journal_undo(
            &root,
            CommitProvenance {
                actor: "unit-test".to_string(),
                source: CommitSource::Test,
                reason: "undo before branch".to_string(),
            },
        )
        .expect("undo should succeed");
    model
        .commit_journaled(
            &root,
            OperationBatch {
                batch_id: Uuid::new_v5(&project_id, b"branch-redo-normal"),
                expected_model_revision: Some(model.model_revision.clone()),
                provenance: CommitProvenance {
                    actor: "unit-test".to_string(),
                    source: CommitSource::Test,
                    reason: "branch after undo".to_string(),
                },
                operations: vec![Operation::SetBoardPackageValue {
                    package_id,
                    value: "BRANCH".to_string(),
                }],
            },
        )
        .expect("branch commit should succeed");

    let error = model
        .commit_journal_redo(
            &root,
            CommitProvenance {
                actor: "unit-test".to_string(),
                source: CommitSource::Test,
                reason: "redo after branch should fail".to_string(),
            },
        )
        .expect_err("redo should be cleared by branch commit");
    assert!(
        error
            .to_string()
            .contains("redo stack was cleared by a newer normal transaction")
    );
    let board_value = read_json_value(&root.join("board/board.json")).expect("board should read");
    assert_eq!(
        board_value["packages"][package_id.to_string()]["value"],
        "BRANCH"
    );
    assert_eq!(model.journal.len(), 3);
    assert_eq!(
        model
            .journal
            .last()
            .expect("branch transaction should exist")
            .transaction_kind,
        TransactionKind::Normal
    );

    let mut reopened = ProjectResolver::new(&root)
        .resolve()
        .expect("reopen should preserve branch policy state");
    assert_eq!(reopened.journal.len(), 3);
    assert_eq!(
        reopened
            .journal
            .last()
            .expect("branch transaction should exist after reopen")
            .transaction_kind,
        TransactionKind::Normal
    );
    let error = reopened
        .commit_journal_redo(
            &root,
            CommitProvenance {
                actor: "unit-test".to_string(),
                source: CommitSource::Test,
                reason: "redo after reopened branch should fail".to_string(),
            },
        )
        .expect_err("redo should still be cleared after reopen");
    assert!(
        error
            .to_string()
            .contains("redo stack was cleared by a newer normal transaction")
    );
}

#[test]
fn append_transaction_journal_rejects_identical_historical_transaction_id() {
    let root = temp_project_root("append_duplicate_historical_id");
    let project_id = Uuid::new_v4();
    let board_id = Uuid::new_v4();
    let package_id = Uuid::new_v4();
    write_project_with_board_package(&root, project_id, board_id, package_id);
    let mut model = ProjectResolver::new(&root)
        .resolve()
        .expect("resolve should succeed");
    let before = model.model_revision.clone();
    let first = model
        .commit_journaled(
            &root,
            OperationBatch {
                batch_id: Uuid::new_v5(&project_id, b"duplicate-historical-first"),
                expected_model_revision: Some(before),
                provenance: CommitProvenance {
                    actor: "unit-test".to_string(),
                    source: CommitSource::Test,
                    reason: "first transaction".to_string(),
                },
                operations: vec![Operation::SetBoardPackageValue {
                    package_id,
                    value: "MID".to_string(),
                }],
            },
        )
        .expect("first journaled commit should succeed");
    let second_before = model.model_revision.clone();
    model
        .commit_journaled(
            &root,
            OperationBatch {
                batch_id: Uuid::new_v5(&project_id, b"duplicate-historical-second"),
                expected_model_revision: Some(second_before),
                provenance: CommitProvenance {
                    actor: "unit-test".to_string(),
                    source: CommitSource::Test,
                    reason: "second transaction".to_string(),
                },
                operations: vec![Operation::SetBoardPackageValue {
                    package_id,
                    value: "NEW".to_string(),
                }],
            },
        )
        .expect("second journaled commit should succeed");

    let error = super::journal::append_transaction_journal(&root, &first.transaction)
        .expect_err("historical identical transaction id should be refused");
    assert!(
        error
            .to_string()
            .contains("already exists before journal tip")
    );
}

#[test]
fn commit_journaled_recovers_torn_trailing_journal_fragment_before_append() {
    let root = temp_project_root("append_recovers_torn_tail");
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
                batch_id: Uuid::new_v5(&project_id, b"torn-tail-first"),
                expected_model_revision: Some(before),
                provenance: CommitProvenance {
                    actor: "unit-test".to_string(),
                    source: CommitSource::Test,
                    reason: "first transaction before torn tail".to_string(),
                },
                operations: vec![Operation::SetBoardPackageValue {
                    package_id,
                    value: "MID".to_string(),
                }],
            },
        )
        .expect("first journaled commit should succeed");
    let mut journal = std::fs::OpenOptions::new()
        .append(true)
        .open(transaction_journal_path(&root))
        .expect("journal should open");
    journal
        .write_all(br#"{"partial_transaction""#)
        .expect("torn tail should write");

    let second_before = model.model_revision.clone();
    model
        .commit_journaled(
            &root,
            OperationBatch {
                batch_id: Uuid::new_v5(&project_id, b"torn-tail-second"),
                expected_model_revision: Some(second_before),
                provenance: CommitProvenance {
                    actor: "unit-test".to_string(),
                    source: CommitSource::Test,
                    reason: "append after torn tail".to_string(),
                },
                operations: vec![Operation::SetBoardPackageValue {
                    package_id,
                    value: "NEW".to_string(),
                }],
            },
        )
        .expect("append after torn tail should recover");

    let journal_text =
        std::fs::read_to_string(transaction_journal_path(&root)).expect("journal should read");
    assert!(!journal_text.contains("partial_transaction"));
    assert_eq!(journal_text.lines().count(), 2);
    let reopened = ProjectResolver::new(&root)
        .resolve()
        .expect("reopen should parse recovered journal");
    assert_eq!(reopened.journal.len(), 2);
}

#[test]
fn commit_journaled_refuses_malformed_complete_journal_line() {
    let root = temp_project_root("append_refuses_bad_complete_line");
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
                batch_id: Uuid::new_v5(&project_id, b"bad-line-first"),
                expected_model_revision: Some(before),
                provenance: CommitProvenance {
                    actor: "unit-test".to_string(),
                    source: CommitSource::Test,
                    reason: "first transaction before bad complete line".to_string(),
                },
                operations: vec![Operation::SetBoardPackageValue {
                    package_id,
                    value: "MID".to_string(),
                }],
            },
        )
        .expect("first journaled commit should succeed");
    let mut journal = std::fs::OpenOptions::new()
        .append(true)
        .open(transaction_journal_path(&root))
        .expect("journal should open");
    journal
        .write_all(
            br#"{"bad_complete_line"
"#,
        )
        .expect("bad complete line should write");

    let second_before = model.model_revision.clone();
    let error = model
        .commit_journaled(
            &root,
            OperationBatch {
                batch_id: Uuid::new_v5(&project_id, b"bad-line-second"),
                expected_model_revision: Some(second_before),
                provenance: CommitProvenance {
                    actor: "unit-test".to_string(),
                    source: CommitSource::Test,
                    reason: "append after bad complete line".to_string(),
                },
                operations: vec![Operation::SetBoardPackageValue {
                    package_id,
                    value: "NEW".to_string(),
                }],
            },
        )
        .expect_err("complete malformed line should be refused");
    assert!(error.to_string().contains("existing journal parse error"));
}

#[test]
fn resolver_reports_invalid_undo_link_and_keeps_valid_prefix() {
    let root = temp_project_root("invalid_undo_link");
    let project_id = Uuid::new_v4();
    let board_id = Uuid::new_v4();
    let package_id = Uuid::new_v4();
    write_project_with_board_package(&root, project_id, board_id, package_id);
    let mut model = ProjectResolver::new(&root)
        .resolve()
        .expect("resolve should succeed");
    let before = model.model_revision.clone();
    let original = model
        .commit_journaled(
            &root,
            OperationBatch {
                batch_id: Uuid::new_v5(&project_id, b"invalid-link-original"),
                expected_model_revision: Some(before),
                provenance: CommitProvenance {
                    actor: "unit-test".to_string(),
                    source: CommitSource::Test,
                    reason: "seed invalid link transaction".to_string(),
                },
                operations: vec![Operation::SetBoardPackageValue {
                    package_id,
                    value: "NEW".to_string(),
                }],
            },
        )
        .expect("journaled value commit should succeed");
    let mut undo = model
        .commit_journal_undo(
            &root,
            CommitProvenance {
                actor: "unit-test".to_string(),
                source: CommitSource::Test,
                reason: "undo with valid link".to_string(),
            },
        )
        .expect("undo should append compensating transaction")
        .transaction;
    undo.undo_of = Some(Uuid::new_v4());

    let journal_text = [
        to_json_deterministic(&original.transaction).expect("original should serialize"),
        to_json_deterministic(&undo).expect("undo should serialize"),
    ]
    .join("\n");
    std::fs::write(transaction_journal_path(&root), format!("{journal_text}\n"))
        .expect("journal should rewrite");

    let reopened = ProjectResolver::new(&root)
        .resolve()
        .expect("reopen should keep valid prefix");
    assert_eq!(reopened.journal.len(), 1);
    assert!(
        reopened
            .diagnostics
            .iter()
            .any(|diagnostic| diagnostic.code == "journal_link_mismatch")
    );
}

#[test]
fn commit_journal_undo_preserves_replayed_state_when_promoted_shard_is_stale() {
    let root = temp_project_root("commit_undo_over_stale_shard");
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
                batch_id: Uuid::new_v5(&project_id, b"stale-shard-value"),
                expected_model_revision: Some(before),
                provenance: CommitProvenance {
                    actor: "unit-test".to_string(),
                    source: CommitSource::Test,
                    reason: "seed stale shard transaction".to_string(),
                },
                operations: vec![Operation::SetBoardPackageValue {
                    package_id,
                    value: "NEW".to_string(),
                }],
            },
        )
        .expect("journaled value commit should succeed");
    write_project_with_board_package(&root, project_id, board_id, package_id);

    let mut reopened = ProjectResolver::new(&root)
        .resolve()
        .expect("resolve should replay over stale shard");
    reopened
        .commit_journal_undo(
            &root,
            CommitProvenance {
                actor: "unit-test".to_string(),
                source: CommitSource::Test,
                reason: "undo stale shard value edit".to_string(),
            },
        )
        .expect("undo should stage from materialized state");

    let board_value = read_json_value(&root.join("board/board.json")).expect("board should read");
    assert_eq!(
        board_value["packages"][package_id.to_string()]["value"],
        "OLD"
    );
    assert_eq!(reopened.journal.len(), 2);
}
