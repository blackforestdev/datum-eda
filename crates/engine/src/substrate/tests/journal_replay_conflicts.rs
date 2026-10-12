use std::io::Write;

use super::*;
use crate::ir::serialization::to_json_deterministic;

#[test]
fn journal_reader_compares_nonadjacent_duplicates_canonically_and_keeps_ordered_prefix() {
    let root = temp_project_root("journal_reader_canonical_duplicates");
    let project_id = Uuid::new_v4();
    let board_id = Uuid::new_v4();
    write_minimal_project(&root, project_id, board_id);
    let mut model = ProjectResolver::new(&root).resolve().unwrap();
    let mut records = Vec::new();
    for ordinal in 0..2 {
        let report = model
            .commit_journaled(
                &root,
                OperationBatch {
                    batch_id: Uuid::new_v5(&project_id, format!("reader-{ordinal}").as_bytes()),
                    expected_model_revision: Some(model.model_revision.clone()),
                    provenance: CommitProvenance {
                        actor: "unit-test".into(),
                        source: CommitSource::Test,
                        reason: "canonical duplicate reader control".into(),
                    },
                    operations: vec![Operation::BumpObjectRevision {
                        object_id: board_id,
                    }],
                },
            )
            .unwrap();
        records.push(report.transaction);
    }
    let first = to_json_deterministic(&records[0]).unwrap();
    let second = to_json_deterministic(&records[1]).unwrap();
    let mut equivalent = serde_json::to_value(&records[0]).unwrap();
    equivalent
        .as_object_mut()
        .unwrap()
        .remove("transaction_kind");
    equivalent["undo_of"] = serde_json::Value::Null;
    let original_diff = serde_json::to_string(&equivalent["diff"]).unwrap();
    let reordered_diff = format!(
        "{{ \"modified\":{}, \"deleted\":{}, \"created\":{} }}",
        equivalent["diff"]["modified"],
        equivalent["diff"]["deleted"],
        equivalent["diff"]["created"]
    );
    let equivalent = serde_json::to_string(&equivalent)
        .unwrap()
        .replace(&original_diff, &reordered_diff);
    assert_ne!(first, equivalent);
    let mut conflict = records[0].clone();
    conflict.operations = vec![Operation::BumpObjectRevision {
        object_id: project_id,
    }];
    let mut sentinel = records[1].clone();
    sentinel.transaction_id = Uuid::new_v4();
    let content = format!(
        "\n{first}\n{second}\n  \n{equivalent}\n{}\n{}\n",
        to_json_deterministic(&conflict).unwrap(),
        to_json_deterministic(&sentinel).unwrap()
    );
    let path = transaction_journal_path(&root);
    std::fs::write(&path, &content).unwrap();
    let (accepted, diagnostics) = super::super::journal_io::read_transaction_journal(&root);
    assert_eq!(accepted, records);
    assert_eq!(diagnostics.len(), 2);
    for (diagnostic, line, code) in [
        (&diagnostics[0], 5, "journal_duplicate_transaction_skipped"),
        (&diagnostics[1], 6, "journal_transaction_id_conflict"),
    ] {
        assert_eq!(diagnostic.code, code);
        assert_eq!(
            diagnostic.message,
            format!(
                "line {line}: duplicate transaction {}",
                records[0].transaction_id
            )
        );
        assert_eq!(diagnostic.path.as_ref(), Some(&path));
    }
    assert_eq!(std::fs::read(&path).unwrap(), content.as_bytes());
    std::fs::remove_dir_all(root).unwrap();
}

#[test]
fn resolver_reports_journal_transaction_id_conflict_and_keeps_valid_prefix() {
    let root = temp_project_root("commit_duplicate_conflict");
    let project_id = Uuid::new_v4();
    let board_id = Uuid::new_v4();
    write_minimal_project(&root, project_id, board_id);
    let mut model = ProjectResolver::new(&root)
        .resolve()
        .expect("resolve should succeed");
    let before = model.model_revision.clone();
    let report = model
        .commit_journaled(
            &root,
            OperationBatch {
                batch_id: Uuid::new_v5(&project_id, b"duplicate-conflict"),
                expected_model_revision: Some(before),
                provenance: CommitProvenance {
                    actor: "unit-test".to_string(),
                    source: CommitSource::Test,
                    reason: "prove duplicate conflict".to_string(),
                },
                operations: vec![Operation::BumpObjectRevision {
                    object_id: board_id,
                }],
            },
        )
        .expect("journaled commit should succeed");
    let mut conflicting = report.transaction.clone();
    conflicting.provenance.reason = "same id different payload".to_string();
    let line = format!(
        "{}\n",
        to_json_deterministic(&conflicting).expect("transaction should serialize")
    );
    let mut journal = std::fs::OpenOptions::new()
        .append(true)
        .open(transaction_journal_path(&root))
        .expect("journal should open");
    journal
        .write_all(line.as_bytes())
        .expect("conflicting transaction should append");

    let reopened = ProjectResolver::new(&root)
        .resolve()
        .expect("reopen should keep valid prefix");
    assert_eq!(reopened.journal.len(), 1);
    assert!(
        reopened
            .diagnostics
            .iter()
            .any(|diagnostic| { diagnostic.code == "journal_transaction_id_conflict" })
    );
    assert_eq!(
        reopened.objects[&board_id].object_revision,
        ObjectRevision(1)
    );
}

#[test]
fn resolver_reports_journal_chain_mismatch_and_keeps_valid_prefix() {
    let root = temp_project_root("commit_chain_mismatch");
    let project_id = Uuid::new_v4();
    let board_id = Uuid::new_v4();
    write_minimal_project(&root, project_id, board_id);
    let mut model = ProjectResolver::new(&root)
        .resolve()
        .expect("resolve should succeed");
    let before = model.model_revision.clone();
    let report = model
        .commit_journaled(
            &root,
            OperationBatch {
                batch_id: Uuid::new_v5(&project_id, b"chain-first"),
                expected_model_revision: Some(before),
                provenance: CommitProvenance {
                    actor: "unit-test".to_string(),
                    source: CommitSource::Test,
                    reason: "prove chain first".to_string(),
                },
                operations: vec![Operation::BumpObjectRevision {
                    object_id: board_id,
                }],
            },
        )
        .expect("first journaled commit should succeed");
    let mut stale_tail = report.transaction.clone();
    stale_tail.transaction_id = Uuid::new_v5(&project_id, b"chain-tail");
    stale_tail.batch_id = Uuid::new_v5(&project_id, b"chain-tail-batch");
    stale_tail.before_model_revision = ModelRevision("not-the-journal-tip".to_string());
    let line = format!(
        "{}\n",
        to_json_deterministic(&stale_tail).expect("transaction should serialize")
    );
    let mut journal = std::fs::OpenOptions::new()
        .append(true)
        .open(transaction_journal_path(&root))
        .expect("journal should open");
    journal
        .write_all(line.as_bytes())
        .expect("stale tail transaction should append");

    let reopened = ProjectResolver::new(&root)
        .resolve()
        .expect("reopen should keep valid prefix");
    assert_eq!(reopened.journal.len(), 1);
    assert!(
        reopened
            .diagnostics
            .iter()
            .any(|diagnostic| { diagnostic.code == "journal_chain_mismatch" })
    );
    assert_eq!(
        reopened.objects[&board_id].object_revision,
        ObjectRevision(1)
    );
}

#[test]
fn resolver_rejects_undo_link_that_does_not_target_current_tip() {
    let root = temp_project_root("commit_undo_non_tip");
    let project_id = Uuid::new_v4();
    let board_id = Uuid::new_v4();
    let package_id = Uuid::new_v4();
    write_project_with_board_package(&root, project_id, board_id, package_id);
    let mut model = ProjectResolver::new(&root)
        .resolve()
        .expect("resolve should succeed");

    let first = model
        .commit_journaled(
            &root,
            OperationBatch {
                batch_id: Uuid::new_v5(&project_id, b"semantic-link-first"),
                expected_model_revision: Some(model.model_revision.clone()),
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
        .expect("first commit should succeed");
    model
        .commit_journaled(
            &root,
            OperationBatch {
                batch_id: Uuid::new_v5(&project_id, b"semantic-link-second"),
                expected_model_revision: Some(model.model_revision.clone()),
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
        .expect("second commit should succeed");
    let mut corrupt_undo = model
        .commit_journal_undo(
            &root,
            CommitProvenance {
                actor: "unit-test".to_string(),
                source: CommitSource::Test,
                reason: "undo second transaction".to_string(),
            },
        )
        .expect("undo should succeed")
        .transaction;
    corrupt_undo.undo_of = Some(first.transaction.transaction_id);

    let journal_text = [
        to_json_deterministic(&first.transaction).expect("first should serialize"),
        to_json_deterministic(&model.journal[1]).expect("second should serialize"),
        to_json_deterministic(&corrupt_undo).expect("corrupt undo should serialize"),
    ]
    .join("\n");
    std::fs::write(transaction_journal_path(&root), format!("{journal_text}\n"))
        .expect("journal should rewrite");

    let reopened = ProjectResolver::new(&root)
        .resolve()
        .expect("reopen should keep valid prefix");
    assert_eq!(reopened.journal.len(), 2);
    assert!(reopened.diagnostics.iter().any(|diagnostic| {
        diagnostic.code == "journal_link_mismatch"
            && diagnostic
                .message
                .contains("does not reference the current journal tip")
    }));
}

#[test]
fn resolver_rejects_undo_link_with_wrong_inverse_payload() {
    let root = temp_project_root("commit_undo_wrong_inverse_payload");
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
                batch_id: Uuid::new_v5(&project_id, b"payload-link-original"),
                expected_model_revision: Some(model.model_revision.clone()),
                provenance: CommitProvenance {
                    actor: "unit-test".to_string(),
                    source: CommitSource::Test,
                    reason: "seed payload link original".to_string(),
                },
                operations: vec![Operation::SetBoardPackageValue {
                    package_id,
                    value: "NEW".to_string(),
                }],
            },
        )
        .expect("original commit should succeed");
    let mut corrupt_undo = model
        .commit_journal_undo(
            &root,
            CommitProvenance {
                actor: "unit-test".to_string(),
                source: CommitSource::Test,
                reason: "undo original transaction".to_string(),
            },
        )
        .expect("undo should succeed")
        .transaction;
    corrupt_undo.operations = vec![Operation::SetBoardPackageValue {
        package_id,
        value: "MID".to_string(),
    }];

    let journal_text = [
        to_json_deterministic(&model.journal[0]).expect("original should serialize"),
        to_json_deterministic(&corrupt_undo).expect("corrupt undo should serialize"),
    ]
    .join("\n");
    std::fs::write(transaction_journal_path(&root), format!("{journal_text}\n"))
        .expect("journal should rewrite");

    let reopened = ProjectResolver::new(&root)
        .resolve()
        .expect("reopen should keep valid prefix");
    assert_eq!(reopened.journal.len(), 1);
    assert!(reopened.diagnostics.iter().any(|diagnostic| {
        diagnostic.code == "journal_link_mismatch"
            && diagnostic
                .message
                .contains("undo operations do not match target inverse operations")
    }));
}

#[test]
fn resolver_reports_journal_after_revision_mismatch_without_accepting_bad_transaction() {
    let root = temp_project_root("commit_after_mismatch");
    let project_id = Uuid::new_v4();
    let board_id = Uuid::new_v4();
    write_minimal_project(&root, project_id, board_id);
    let mut model = ProjectResolver::new(&root)
        .resolve()
        .expect("resolve should succeed");
    let before = model.model_revision.clone();
    let report = model
        .commit_journaled(
            &root,
            OperationBatch {
                batch_id: Uuid::new_v5(&project_id, b"after-mismatch"),
                expected_model_revision: Some(before),
                provenance: CommitProvenance {
                    actor: "unit-test".to_string(),
                    source: CommitSource::Test,
                    reason: "prove after mismatch".to_string(),
                },
                operations: vec![Operation::BumpObjectRevision {
                    object_id: board_id,
                }],
            },
        )
        .expect("journaled commit should succeed");
    let mut corrupted = report.transaction.clone();
    corrupted.after_model_revision = ModelRevision("corrupt-after".to_string());
    std::fs::write(
        transaction_journal_path(&root),
        format!(
            "{}\n",
            to_json_deterministic(&corrupted).expect("transaction should serialize")
        ),
    )
    .expect("corrupted journal should write");

    let reopened = ProjectResolver::new(&root)
        .resolve()
        .expect("reopen should reject bad transaction");
    assert!(reopened.journal.is_empty());
    assert!(
        reopened
            .diagnostics
            .iter()
            .any(|diagnostic| { diagnostic.code == "journal_after_revision_mismatch" })
    );
    assert_eq!(
        reopened.objects[&board_id].object_revision,
        ObjectRevision(0)
    );
}

#[test]
fn append_transaction_journal_is_idempotent_for_identical_transaction() {
    let root = temp_project_root("commit_append_idempotent");
    let project_id = Uuid::new_v4();
    let board_id = Uuid::new_v4();
    write_minimal_project(&root, project_id, board_id);
    let mut model = ProjectResolver::new(&root)
        .resolve()
        .expect("resolve should succeed");
    let before = model.model_revision.clone();
    let report = model
        .commit_journaled(
            &root,
            OperationBatch {
                batch_id: Uuid::new_v5(&project_id, b"append-idempotent"),
                expected_model_revision: Some(before),
                provenance: CommitProvenance {
                    actor: "unit-test".to_string(),
                    source: CommitSource::Test,
                    reason: "prove append idempotency".to_string(),
                },
                operations: vec![Operation::BumpObjectRevision {
                    object_id: board_id,
                }],
            },
        )
        .expect("journaled commit should succeed");

    super::super::journal::append_transaction_journal(&root, &report.transaction)
        .expect("identical transaction append should be idempotent");
    let content =
        std::fs::read_to_string(transaction_journal_path(&root)).expect("journal should read");
    assert_eq!(content.lines().count(), 1);
}

#[test]
fn resolver_rejects_redo_cleared_by_newer_normal_transaction() {
    let root = temp_project_root("commit_stale_redo_after_branch");
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
                batch_id: Uuid::new_v5(&project_id, b"stale-redo-original"),
                expected_model_revision: Some(model.model_revision.clone()),
                provenance: CommitProvenance {
                    actor: "unit-test".to_string(),
                    source: CommitSource::Test,
                    reason: "seed stale redo original".to_string(),
                },
                operations: vec![Operation::SetBoardPackageValue {
                    package_id,
                    value: "NEW".to_string(),
                }],
            },
        )
        .expect("original commit should succeed");
    let undo = model
        .commit_journal_undo(
            &root,
            CommitProvenance {
                actor: "unit-test".to_string(),
                source: CommitSource::Test,
                reason: "undo before stale redo branch".to_string(),
            },
        )
        .expect("undo should succeed");
    model
        .commit_journaled(
            &root,
            OperationBatch {
                batch_id: Uuid::new_v5(&project_id, b"stale-redo-branch"),
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

    let mut stale_redo = undo.transaction.clone();
    stale_redo.transaction_id = Uuid::new_v5(&project_id, b"stale-redo-tail");
    stale_redo.batch_id = Uuid::new_v5(&project_id, b"stale-redo-tail-batch");
    stale_redo.transaction_kind = TransactionKind::Redo;
    stale_redo.undo_of = None;
    stale_redo.redo_of = Some(undo.transaction.transaction_id);
    stale_redo.before_model_revision = model.model_revision.clone();
    stale_redo.operations = undo.transaction.inverse_operations.clone();
    stale_redo.provenance.reason = "stale redo injected after branch".to_string();
    let mut journal = std::fs::OpenOptions::new()
        .append(true)
        .open(transaction_journal_path(&root))
        .expect("journal should open");
    writeln!(
        journal,
        "{}",
        to_json_deterministic(&stale_redo).expect("transaction should serialize")
    )
    .expect("stale redo should append");

    let reopened = ProjectResolver::new(&root)
        .resolve()
        .expect("reopen should keep valid prefix");
    assert_eq!(reopened.journal.len(), 3);
    assert!(
        reopened
            .diagnostics
            .iter()
            .any(|diagnostic| { diagnostic.code == "journal_link_mismatch" })
    );
    let board_value = reopened
        .materialized_source_shard_value(SourceShardKind::BoardRoot)
        .expect("board should materialize");
    assert_eq!(
        board_value["packages"][package_id.to_string()]["value"],
        "BRANCH"
    );
}
