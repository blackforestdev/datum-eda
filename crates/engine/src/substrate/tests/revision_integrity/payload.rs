use super::*;
use std::collections::BTreeMap;

fn write_records(root: &Path, records: &[TransactionRecord]) {
    let mut text = String::new();
    for record in records {
        text.push_str(&to_json_deterministic(record).expect("serialize journal record"));
        text.push('\n');
    }
    std::fs::write(transaction_journal_path(root), text).unwrap();
}

fn authority_bytes(root: &Path) -> BTreeMap<PathBuf, Vec<u8>> {
    fn visit(path: &Path, files: &mut BTreeMap<PathBuf, Vec<u8>>) {
        for entry in std::fs::read_dir(path).unwrap() {
            let entry = entry.unwrap();
            if entry.file_type().unwrap().is_dir() {
                visit(&entry.path(), files);
            } else {
                files.insert(entry.path(), std::fs::read(entry.path()).unwrap());
            }
        }
    }
    let mut files = BTreeMap::new();
    visit(&root.join(".datum/revision/v1"), &mut files);
    for path in [
        transaction_journal_path(root),
        root.join("project.json"),
        root.join(".datum/journal/cursor.json"),
    ] {
        if path.exists() {
            files.insert(path.clone(), std::fs::read(path).unwrap());
        }
    }
    files
}

fn change_provenance(record: &mut TransactionRecord) {
    record.provenance.actor = "substituted actor".into();
    record.provenance.reason = "substituted reason".into();
}

#[test]
fn tip_and_earlier_covered_payload_tampering_refuse_inspection_reopen_and_write() {
    for index in [0, 1] {
        let (root, mut model, first) = committed_project(&format!("payload_tamper_{index}"));
        model
            .commit_journaled(&root, rename_batch(&model, "second"))
            .unwrap();
        let mut changed = model.journal.clone();
        change_provenance(&mut changed[index]);
        write_records(&root, &changed);
        let before = authority_bytes(&root);
        // Deliberately pass the stale, untampered in-memory journal with the same tip.
        let state = RevisionAuthorityStore::new(&root)
            .inspect_and_recover(model.project.project_id, &model.journal);
        match state {
            RevisionStoreState::ReadOnlyDiagnostic {
                last_complete,
                diagnostics,
            } => {
                assert!(diagnostics.iter().any(|d| {
                    d.message
                        .contains("accepted journal payload differs from immutable transaction")
                }));
                assert_eq!(
                    last_complete.map(|h| h.transaction_id),
                    (index == 1).then_some(first.transaction_id)
                );
            }
            other => panic!("tampered payload retained writable integrity: {other:?}"),
        }
        let mut reopened = ProjectResolver::new(&root).resolve().unwrap();
        assert!(reopened.diagnostics.iter().any(|d| {
            d.code == "revision_integrity_unavailable"
                && d.message.contains("accepted journal payload differs")
        }));
        let error = reopened
            .commit_journaled(&root, rename_batch(&reopened, "refused"))
            .expect_err("tampered provenance must refuse ordinary writes");
        assert!(error.to_string().contains("integrity unavailable"));
        assert_eq!(
            authority_bytes(&root),
            before,
            "inspection/reopen/refused write preserve journal, authority and design bytes"
        );
    }
}

#[test]
fn staged_recovery_checks_payload_and_covered_parents_before_promotion() {
    for index in [0, 1] {
        let (root, mut model, _) = committed_project(&format!("payload_stage_{index}"));
        model
            .commit_journaled_with_integrity_fault(
                &root,
                rename_batch(&model, "pending"),
                IntegrityCommitFaultPoint::JournalAppend,
            )
            .unwrap_err();
        let (mut records, diagnostics) = crate::substrate::read_transaction_journal(&root);
        assert!(diagnostics.is_empty());
        assert_eq!(records.len(), 2);
        change_provenance(&mut records[index]);
        write_records(&root, &records);
        let before = authority_bytes(&root);
        assert!(matches!(
            RevisionAuthorityStore::new(&root)
                .inspect_and_recover(model.project.project_id, &records),
            RevisionStoreState::ReadOnlyDiagnostic { .. }
        ));
        let reopened = ProjectResolver::new(&root).resolve().unwrap();
        assert!(reopened.diagnostics.iter().any(|d| {
            d.code == "revision_integrity_unavailable"
                && d.message.contains("accepted journal payload differs")
        }));
        assert_eq!(
            authority_bytes(&root),
            before,
            "rejected stage and its evidence must remain unpromoted"
        );
    }
}

#[test]
fn last_complete_fallback_uses_fresh_payload_and_refuses_invalid_journal() {
    let (root, model, _) = committed_project("payload_fallback_fresh");
    let mut changed = model.journal.clone();
    change_provenance(&mut changed[0]);
    write_records(&root, &changed);
    std::fs::write(root.join(".datum/revision/v1/head.json"), b"{broken").unwrap();
    let before = authority_bytes(&root);
    assert!(matches!(
        RevisionAuthorityStore::new(&root)
            .inspect_and_recover(model.project.project_id, &model.journal),
        RevisionStoreState::ReadOnlyDiagnostic {
            last_complete: None,
            ..
        }
    ));
    assert_eq!(authority_bytes(&root), before);
    std::fs::write(transaction_journal_path(&root), b"{invalid journal\n").unwrap();
    let before = authority_bytes(&root);
    assert!(matches!(
        RevisionAuthorityStore::new(&root)
            .inspect_and_recover(model.project.project_id, &model.journal),
        RevisionStoreState::ReadOnlyDiagnostic {
            last_complete: None,
            ..
        }
    ));
    assert_eq!(authority_bytes(&root), before);
}

#[test]
fn first_backed_commit_after_legacy_history_reopens_and_supports_undo_redo() {
    let (root, _, legacy) = committed_project("payload_pre_integrity");
    // Model the supported on-disk journal predating the integrity store.
    std::fs::remove_dir_all(root.join(".datum/revision/v1")).unwrap();
    let mut model = ProjectResolver::new(&root).resolve().unwrap();
    assert!(matches!(
        RevisionAuthorityStore::new(&root)
            .inspect_and_recover(model.project.project_id, &model.journal),
        RevisionStoreState::Empty
    ));
    assert_eq!(model.journal[0], legacy);
    let accepted = model
        .commit_journaled(&root, rename_batch(&model, "first_backed"))
        .unwrap()
        .transaction;
    let mut reopened = ProjectResolver::new(&root).resolve().unwrap();
    assert!(matches!(
        RevisionAuthorityStore::new(&root)
            .inspect_and_recover(model.project.project_id, &reopened.journal),
        RevisionStoreState::Complete { .. }
    ));
    let provenance = rename_batch(&reopened, "undo_redo").provenance;
    let undo = reopened
        .commit_journal_undo(&root, provenance.clone())
        .unwrap();
    assert_eq!(undo.transaction.undo_of, Some(accepted.transaction_id));
    let mut reopened = ProjectResolver::new(&root).resolve().unwrap();
    let redo = reopened.commit_journal_redo(&root, provenance).unwrap();
    assert_eq!(
        redo.transaction.redo_of,
        Some(undo.transaction.transaction_id)
    );
    let reopened = ProjectResolver::new(&root).resolve().unwrap();
    assert_eq!(reopened.project.name, "revision-integrity-first_backed");
    assert!(matches!(
        RevisionAuthorityStore::new(&root)
            .inspect_and_recover(model.project.project_id, &reopened.journal),
        RevisionStoreState::Complete { .. }
    ));
}

#[test]
fn fallback_reuse_distinguishes_alternate_roots_and_fresh_journal_context() {
    use crate::revision::AlgorithmQualifiedDigest;
    use sha2::{Digest, Sha256};

    fn canonical<T: serde::Serialize>(value: &T) -> Vec<u8> {
        let mut bytes = to_json_deterministic(value).unwrap().into_bytes();
        bytes.push(b'\n');
        bytes
    }
    fn digest(bytes: &[u8]) -> AlgorithmQualifiedDigest {
        AlgorithmQualifiedDigest(format!("sha256:{:x}", Sha256::digest(bytes)))
    }
    fn publish(store: &RevisionAuthorityStore, generation: &mut IntegrityGeneration) {
        generation.integrity_root = digest(&canonical(&generation.material()));
        std::fs::write(
            store.generation_path(&generation.integrity_root),
            canonical(generation),
        )
        .unwrap();
    }

    let (root, mut model, first) = committed_project("payload_fallback_alternate_root");
    model
        .commit_journaled(&root, rename_batch(&model, "second"))
        .unwrap();
    let store = RevisionAuthorityStore::new(&root);
    let head = store.read_head().unwrap().unwrap();
    let mut second: IntegrityGeneration = serde_json::from_slice(
        &std::fs::read(store.generation_path(&head.integrity_root)).unwrap(),
    )
    .unwrap();
    let mut alternate_first: IntegrityGeneration = serde_json::from_slice(
        &std::fs::read(store.generation_path(second.parent_integrity_root.as_ref().unwrap()))
            .unwrap(),
    )
    .unwrap();
    let mut changed = model.journal.clone();
    change_provenance(&mut changed[0]);
    let alternate_payload = canonical(&changed[0]);
    alternate_first.transaction_blob = digest(&alternate_payload);
    std::fs::write(
        store.blob_path(&alternate_first.transaction_blob),
        alternate_payload,
    )
    .unwrap();
    publish(&store, &mut alternate_first);
    // Both first-generation roots have valid immutable blobs, but only the
    // original matches the currently accepted journal. The descendant references
    // the other root and must not borrow the original's successful check.
    std::fs::remove_file(store.generation_path(&head.integrity_root)).unwrap();
    second.parent_integrity_root = Some(alternate_first.integrity_root.clone());
    publish(&store, &mut second);
    std::fs::write(root.join(".datum/revision/v1/head.json"), b"{broken").unwrap();
    let before = authority_bytes(&root);
    match store.inspect_and_recover(model.project.project_id, &model.journal) {
        RevisionStoreState::ReadOnlyDiagnostic {
            last_complete: Some(last),
            ..
        } => {
            assert_eq!(last.transaction_id, first.transaction_id);
            assert_ne!(last.integrity_root, alternate_first.integrity_root);
        }
        other => panic!("expected only the original first generation: {other:?}"),
    }
    assert_eq!(authority_bytes(&root), before);
    // A separate inspection has a fresh journal context. Now that the alternate
    // payload is accepted, that branch and its descendant can be verified.
    write_records(&root, &changed);
    let before = authority_bytes(&root);
    match store.inspect_and_recover(model.project.project_id, &model.journal) {
        RevisionStoreState::ReadOnlyDiagnostic {
            last_complete: Some(last),
            ..
        } => {
            assert_eq!(last.transaction_id, second.transaction_id);
            assert_eq!(last.integrity_root, second.integrity_root);
        }
        other => panic!("expected freshly verified alternate branch: {other:?}"),
    }
    assert_eq!(authority_bytes(&root), before);
}
