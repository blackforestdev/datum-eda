use std::io::Write;

use super::*;

#[test]
fn resolver_reports_journal_parse_error_and_keeps_valid_prefix() {
    let root = temp_project_root("commit_parse_error");
    let project_id = Uuid::new_v4();
    let board_id = Uuid::new_v4();
    write_minimal_project(&root, project_id, board_id);
    let mut model = ProjectResolver::new(&root)
        .resolve()
        .expect("resolve should succeed");
    let before = model.model_revision.clone();
    model
        .commit_journaled(
            &root,
            OperationBatch {
                batch_id: Uuid::new_v5(&project_id, b"parse-error-prefix"),
                expected_model_revision: Some(before),
                provenance: CommitProvenance {
                    actor: "unit-test".to_string(),
                    source: CommitSource::Test,
                    reason: "prove journal valid prefix".to_string(),
                },
                operations: vec![Operation::BumpObjectRevision {
                    object_id: board_id,
                }],
            },
        )
        .expect("journaled commit should succeed");
    let mut journal = std::fs::OpenOptions::new()
        .append(true)
        .open(transaction_journal_path(&root))
        .expect("journal should open");
    journal
        .write_all(b"{not valid json}\n")
        .expect("bad journal line should append");

    let reopened = ProjectResolver::new(&root)
        .resolve()
        .expect("reopen should keep valid prefix");
    assert_eq!(reopened.journal.len(), 1);
    assert_eq!(
        reopened.objects[&board_id].object_revision,
        ObjectRevision(1)
    );
    assert!(reopened.diagnostics.iter().any(|diagnostic| {
        diagnostic.code == "journal_parse_error" && diagnostic.message.contains("line 2")
    }));
}
