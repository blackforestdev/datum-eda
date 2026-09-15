use super::*;
use crate::revision::{
    IntegrityCommitFaultPoint, IntegrityGeneration, ProjectWriteLease, RevisionAuthorityStore,
    RevisionStoreState, TechnicalValidationState, export_backup, preview_restore, restore_backup,
    transaction_tip, undo_restore, verify_backup,
};

fn rename_batch(model: &DesignModel, suffix: &str) -> OperationBatch {
    OperationBatch {
        batch_id: Uuid::new_v4(),
        expected_model_revision: Some(model.model_revision.clone()),
        provenance: CommitProvenance {
            actor: "revision-integrity-test".to_string(),
            source: CommitSource::Test,
            reason: format!("prove revision integrity {suffix}"),
        },
        operations: vec![Operation::SetProjectName {
            project_id: model.project.project_id,
            name: format!("revision-integrity-{suffix}"),
        }],
    }
}

fn committed_project(name: &str) -> (PathBuf, DesignModel, TransactionRecord) {
    let root = temp_project_root(name);
    let project_id = Uuid::new_v4();
    write_minimal_project(&root, project_id, Uuid::new_v4());
    let mut model = ProjectResolver::new(&root).resolve().expect("resolve");
    let report = model
        .commit_journaled(&root, rename_batch(&model, "committed"))
        .expect("commit should establish integrity authority");
    (root, model, report.transaction)
}

#[test]
fn journaled_commit_establishes_verified_algorithm_qualified_authority() {
    let (root, model, transaction) = committed_project("integrity_complete");
    let store = RevisionAuthorityStore::new(&root);
    let state = store.inspect_and_recover(model.project.project_id, &model.journal);
    let head = match state {
        RevisionStoreState::Complete { head } => head,
        other => panic!("expected complete integrity store, got {other:?}"),
    };

    assert_eq!(head.transaction_id, transaction.transaction_id);
    assert!(head.integrity_root.0.starts_with("sha256:"));
    assert_eq!(head.integrity_root.0.len(), 71);
    assert_eq!(
        transaction_tip(&model.journal),
        Some(transaction.transaction_id)
    );
    assert_ne!(
        transaction.after_model_revision.0,
        transaction.transaction_id.to_string(),
        "technical ModelRevision must not be equated with accepted transaction identity"
    );
    let generation_hex = head
        .integrity_root
        .0
        .strip_prefix("sha256:")
        .expect("algorithm-qualified root");
    let generation: IntegrityGeneration = serde_json::from_slice(
        &std::fs::read(
            root.join(".datum/revision/v1/generations")
                .join(format!("{generation_hex}.json")),
        )
        .expect("generation bytes"),
    )
    .expect("generation JSON");
    assert_eq!(
        generation.validation_state,
        TechnicalValidationState::AcceptedByMutationGuards
    );
    assert_eq!(
        generation.cache_invalidations,
        vec!["source_shard:project.json"]
    );

    let bytes = std::fs::read(root.join(".datum/revision/v1/head.json")).expect("head bytes");
    assert_eq!(bytes.last(), Some(&b'\n'));
    assert!(!bytes.windows(2).any(|window| window == b"  "));
}

#[test]
fn concurrent_writer_is_refused_and_release_is_reacquirable() {
    let root = temp_project_root("integrity_writer_lease");
    let first = ProjectWriteLease::acquire(&root).expect("first writer");
    let error = ProjectWriteLease::acquire(&root).expect_err("second writer must be refused");
    assert!(error.to_string().contains("live writer"));
    drop(first);
    ProjectWriteLease::acquire(&root).expect("released lease should be reacquirable");
}

#[test]
fn every_commit_boundary_recovers_only_journal_committed_authority() {
    for fault in [
        IntegrityCommitFaultPoint::AuthorityStage,
        IntegrityCommitFaultPoint::JournalAppend,
        IntegrityCommitFaultPoint::ShardPromotion,
        IntegrityCommitFaultPoint::AuthorityPromotion,
        IntegrityCommitFaultPoint::CursorWrite,
    ] {
        let root = temp_project_root(&format!("integrity_fault_{fault:?}"));
        let project_id = Uuid::new_v4();
        write_minimal_project(&root, project_id, Uuid::new_v4());
        let mut model = ProjectResolver::new(&root).resolve().expect("resolve");
        model
            .commit_journaled_with_integrity_fault(&root, rename_batch(&model, "fault"), fault)
            .expect_err("fault injection must interrupt the caller");

        let reopened = ProjectResolver::new(&root)
            .resolve()
            .expect("recovery resolve");
        match fault {
            IntegrityCommitFaultPoint::AuthorityStage => {
                assert!(reopened.journal.is_empty());
                assert!(!root.join(".datum/revision/v1/head.json").exists());
            }
            IntegrityCommitFaultPoint::JournalAppend
            | IntegrityCommitFaultPoint::ShardPromotion => {
                assert_eq!(reopened.journal.len(), 1);
                assert!(
                    reopened
                        .diagnostics
                        .iter()
                        .any(|diagnostic| diagnostic.code == "revision_integrity_recovered")
                );
            }
            IntegrityCommitFaultPoint::AuthorityPromotion
            | IntegrityCommitFaultPoint::CursorWrite => {
                assert_eq!(reopened.journal.len(), 1);
                assert!(
                    !reopened
                        .diagnostics
                        .iter()
                        .any(|diagnostic| diagnostic.code == "revision_integrity_unavailable")
                );
            }
        }
        if !reopened.journal.is_empty() {
            assert!(matches!(
                RevisionAuthorityStore::new(&root)
                    .inspect_and_recover(project_id, &reopened.journal),
                RevisionStoreState::Complete { .. }
            ));
        }
    }
}

#[test]
fn corrupt_head_preserves_bytes_and_exposes_last_complete_generation() {
    let (root, model, transaction) = committed_project("integrity_corrupt_head");
    let head_path = root.join(".datum/revision/v1/head.json");
    let corrupt = b"{\"truncated\":";
    std::fs::write(&head_path, corrupt).expect("inject corrupt head");

    let state = RevisionAuthorityStore::new(&root)
        .inspect_and_recover(model.project.project_id, &model.journal);
    match state {
        RevisionStoreState::ReadOnlyDiagnostic {
            last_complete: Some(last),
            diagnostics,
        } => {
            assert_eq!(last.transaction_id, transaction.transaction_id);
            assert_eq!(diagnostics[0].code, "revision_integrity_unavailable");
        }
        other => panic!("expected last-complete read-only state, got {other:?}"),
    }
    assert_eq!(
        std::fs::read(head_path).expect("corrupt head retained"),
        corrupt
    );
}

#[test]
fn missing_blob_is_read_only_and_never_repaired_in_place() {
    let (root, model, _) = committed_project("integrity_missing_blob");
    let blobs = root.join(".datum/revision/v1/blobs/sha256");
    let missing = std::fs::read_dir(&blobs)
        .expect("blob directory")
        .next()
        .expect("one blob")
        .expect("blob entry")
        .path();
    std::fs::remove_file(&missing).expect("inject missing blob");

    assert!(matches!(
        RevisionAuthorityStore::new(&root)
            .inspect_and_recover(model.project.project_id, &model.journal),
        RevisionStoreState::ReadOnlyDiagnostic { .. }
    ));
    assert!(
        !missing.exists(),
        "inspection must not synthesize missing bytes"
    );
}

#[test]
fn backup_is_independently_verified_and_restore_is_reversible() {
    let (root, mut model, first) = committed_project("integrity_backup");
    let backup = temp_project_root("integrity_backup_export");
    std::fs::remove_dir(&backup).expect("backup destination starts absent");
    let manifest = export_backup(&root, model.project.project_id, &backup).expect("export");
    assert_eq!(
        verify_backup(&backup).expect("independent verify"),
        manifest
    );

    let second = model
        .commit_journaled(&root, rename_batch(&model, "second"))
        .expect("second commit")
        .transaction;
    let preview = preview_restore(&root, &backup).expect("restore preview");
    assert_eq!(preview.incoming_head.transaction_id, first.transaction_id);
    assert_eq!(
        preview.current_head.expect("current head").transaction_id,
        second.transaction_id
    );
    let design_before_restore = std::fs::read(root.join("project.json")).expect("design bytes");
    let receipt = restore_backup(&root, &backup).expect("restore");
    assert_eq!(
        RevisionAuthorityStore::new(&root)
            .read_head()
            .expect("head exists")
            .expect("head parses")
            .transaction_id,
        first.transaction_id
    );
    for entry in &manifest.files {
        assert_eq!(
            std::fs::read(root.join(".datum/revision/v1").join(&entry.relative_path))
                .expect("restored authority file"),
            std::fs::read(backup.join(&entry.relative_path)).expect("backup authority file")
        );
    }
    assert_eq!(
        std::fs::read(root.join("project.json")).expect("design bytes after restore"),
        design_before_restore,
        "authority-store restore must not silently rewrite Design data"
    );
    undo_restore(&root, &receipt).expect("restore undo");
    assert_eq!(
        RevisionAuthorityStore::new(&root)
            .read_head()
            .expect("head exists")
            .expect("head parses")
            .transaction_id,
        second.transaction_id
    );

    let tampered = backup.join(&manifest.files[0].relative_path);
    std::fs::write(&tampered, b"tampered").expect("tamper backup");
    assert!(verify_backup(&backup).is_err());
}

fn copy_tree(source: &Path, destination: &Path) {
    std::fs::create_dir_all(destination).expect("destination directory");
    for entry in std::fs::read_dir(source).expect("fixture directory") {
        let entry = entry.expect("fixture entry");
        let target = destination.join(entry.file_name());
        if entry.file_type().expect("fixture type").is_dir() {
            copy_tree(&entry.path(), &target);
        } else {
            std::fs::copy(entry.path(), target).expect("fixture copy");
        }
    }
}

#[test]
fn checked_in_native_projects_keep_design_semantics_under_integrity_commits() {
    let manifest_dir = Path::new(env!("CARGO_MANIFEST_DIR"));
    for (name, fixture) in [
        (
            "native_authored_baseline_v1",
            "../test-harness/testdata/library/native_authored_baseline_v1",
        ),
        (
            "profile-divergence-authored-copper",
            "../test-harness/testdata/quality/route_strategy_curated_baseline_v1/profile-divergence-authored-copper",
        ),
        (
            "via-available",
            "../test-harness/testdata/quality/route_strategy_curated_baseline_v1/via-available",
        ),
    ] {
        let root = temp_project_root(&format!("integrity_real_{name}"));
        copy_tree(&manifest_dir.join(fixture), &root);
        let mut model = ProjectResolver::new(&root)
            .resolve()
            .expect("fixture resolve");
        let report = model
            .commit_journaled(&root, rename_batch(&model, name))
            .expect("fixture journaled rename");
        let reopened = ProjectResolver::new(&root)
            .resolve()
            .expect("fixture reopen");
        assert_eq!(reopened.project.name, format!("revision-integrity-{name}"));
        assert_eq!(
            reopened.model_revision,
            report.transaction.after_model_revision
        );
        assert!(matches!(
            RevisionAuthorityStore::new(&root)
                .inspect_and_recover(reopened.project.project_id, &reopened.journal),
            RevisionStoreState::Complete { .. }
        ));
    }
}
