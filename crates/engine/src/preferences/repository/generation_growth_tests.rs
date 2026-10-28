//! Generation-growth regression suite for GP-764: v2 receipt deltas, full-log
//! reconstruction over the verified parent chain, mixed v1/v2 stores, and
//! recovery-candidate chain admission (a descendant of a corrupt ancestor is
//! never advertised).

use super::io::{canonical_bytes, digest_bytes, generation_dir, promote_head, write_new_file};
use super::model::{
    CANONICALIZATION_VERSION, CURRENT_WRITE_VERSION, GenerationManifest,
    REPOSITORY_FORMAT_VERSION_V1,
};
use super::{
    GenerationRef, MutationMetadata, PreferenceMutation, PreferencePartition, PreferenceRepository,
    RepositoryError, RepositorySnapshot, RepositoryStatus,
};
use crate::preferences::{PreferenceKey, active_v1_registry};
use serde_json::json;
use std::path::Path;
use uuid::Uuid;

const MANIFEST_FILE: &str = "manifest.json";

struct TempRoot(std::path::PathBuf);

impl TempRoot {
    fn new() -> Self {
        Self(std::env::temp_dir().join(format!("datum-gp764-{}", Uuid::new_v4())))
    }
}

impl Drop for TempRoot {
    fn drop(&mut self) {
        if self.0.starts_with(std::env::temp_dir()) {
            let _ = std::fs::remove_dir_all(&self.0);
        }
    }
}

fn repo(root: &TempRoot) -> PreferenceRepository {
    PreferenceRepository::new(&root.0, active_v1_registry())
}

fn meta(reason: &str) -> MutationMetadata {
    MutationMetadata {
        actor: "gp764".to_owned(),
        reason: reason.to_owned(),
        writer_instance: "gp764-proc".to_owned(),
        audit: None,
    }
}

fn set_motion(on: bool) -> PreferenceMutation {
    PreferenceMutation::Set {
        partition: PreferencePartition::User,
        key: PreferenceKey::parse("datum.accessibility.reduced_motion").unwrap(),
        value: json!(on),
    }
}

fn ready(repository: &PreferenceRepository) -> RepositorySnapshot {
    match repository.inspect() {
        RepositoryStatus::Ready(snapshot) => snapshot,
        other => panic!("expected ready repository, got {other:?}"),
    }
}

fn read_manifest(root: &Path, generation: u64) -> (Vec<u8>, GenerationManifest) {
    let bytes = std::fs::read(generation_dir(root, generation).join(MANIFEST_FILE)).unwrap();
    let manifest = serde_json::from_slice(&bytes).unwrap();
    (bytes, manifest)
}

#[test]
fn v2_generations_store_receipt_deltas_and_reconstruct_the_full_log() {
    let root = TempRoot::new();
    let repository = repo(&root);
    let mut head = repository.initialize(&meta("initialize")).unwrap();
    for index in 0..12 {
        head = repository
            .commit_mutations(&head, &[set_motion(index % 2 == 0)], &meta("toggle"))
            .unwrap();
    }
    assert_eq!(head.generation, 12);
    assert_eq!(head.format_version, CURRENT_WRITE_VERSION);

    // Each generation manifest embeds only its own delta, so per-generation size
    // does not grow with history (the quadratic regression guard).
    let mut steady = Vec::new();
    for generation in 0..=12 {
        let (bytes, manifest) = read_manifest(&root.0, generation);
        assert_eq!(manifest.format_version, CURRENT_WRITE_VERSION);
        assert_eq!(manifest.state.receipts.len(), 1);
        assert!(
            manifest
                .state
                .receipts
                .iter()
                .all(|receipt| receipt.resulting_generation == generation),
            "generation {generation} embedded a foreign receipt"
        );
        if generation >= 2 {
            steady.push(bytes.len());
        }
    }
    let spread = steady.iter().max().unwrap() - steady.iter().min().unwrap();
    assert!(
        spread <= 16,
        "per-generation manifest grew with history: {spread}"
    );

    // The public snapshot still exposes the complete ordered audit log.
    let snapshot = ready(&repository);
    assert_eq!(snapshot.receipts.len(), 13);
    assert!(
        snapshot
            .receipts
            .iter()
            .enumerate()
            .all(|(generation, receipt)| receipt.resulting_generation == generation as u64)
    );
}

fn receipt_json(resulting_generation: u64) -> serde_json::Value {
    json!({
        "operation": "CommitPreferenceMutation",
        "actor": "legacy",
        "reason": "legacy",
        "expected_generation": null,
        "resulting_generation": resulting_generation,
        "affected_keys": [],
        "before_digests": {},
        "after_digests": {},
        "redacted_keys": [],
    })
}

fn forge_v1_generation(root: &Path, repository_id: &str, generation: u64) -> GenerationRef {
    // A legacy v1 genesis embeds the full accumulated log and carries no parent
    // integrity digest. Written by hand because this build's writer only emits v2.
    let manifest = json!({
        "format_version": REPOSITORY_FORMAT_VERSION_V1,
        "canonicalization_version": CANONICALIZATION_VERSION,
        "repository_id": repository_id,
        "generation": generation,
        "parent_generation": null,
        "committed_order": generation,
        "writer_instance": "legacy-datum",
        "state": {
            "installation": {},
            "user": {},
            "unknown_envelopes": {},
            "receipts": [receipt_json(generation)],
        },
    });
    let bytes = canonical_bytes(&manifest).unwrap();
    write_new_file(
        &generation_dir(root, generation).join(MANIFEST_FILE),
        &bytes,
    )
    .unwrap();
    GenerationRef {
        repository_id: repository_id.to_owned(),
        generation,
        parent_generation: None,
        canonical_manifest_digest: digest_bytes(&bytes),
        committed_order: generation,
        writer_instance: "legacy-datum".to_owned(),
        format_version: REPOSITORY_FORMAT_VERSION_V1,
        canonicalization_version: CANONICALIZATION_VERSION,
    }
}

#[test]
fn mixed_v1_and_v2_store_reconstructs_and_corrupt_ancestor_is_not_advertised() {
    let root = TempRoot::new();
    let v1_head = forge_v1_generation(&root.0, "legacy-repo", 0);
    promote_head(&root.0, &v1_head).unwrap();
    let repository = repo(&root);
    // A pure v1 store returns its embedded full log directly.
    assert_eq!(ready(&repository).receipts.len(), 1);

    // Advancing writes v2 delta generations above the v1 base.
    let middle = repository
        .commit_mutations(&v1_head, &[set_motion(true)], &meta("v2 over v1"))
        .unwrap();
    let head = repository
        .commit_mutations(&middle, &[set_motion(false)], &meta("v2 again"))
        .unwrap();
    assert_eq!(head.format_version, CURRENT_WRITE_VERSION);

    // Reconstruction spans the v1 base and the v2 deltas, in generation order.
    let snapshot = ready(&repository);
    assert_eq!(snapshot.receipts.len(), 3);
    assert_eq!(snapshot.receipts[0].resulting_generation, 0);
    assert_eq!(snapshot.receipts[2].resulting_generation, 2);

    // Corrupt the MIDDLE ancestor so the chain can no longer be validated.
    let manifest = generation_dir(&root.0, middle.generation).join(MANIFEST_FILE);
    let mut bytes = std::fs::read(&manifest).unwrap();
    bytes.extend_from_slice(b"!corrupt");
    std::fs::write(&manifest, &bytes).unwrap();

    // The head no longer loads: the verified walk refuses the altered parent.
    let evidence = match repository.inspect() {
        RepositoryStatus::Unreadable(evidence) => evidence,
        other => panic!("expected tampered-chain diagnostic, got {other:?}"),
    };
    // Finding 3: a descendant of a corrupt ancestor is NOT advertised as a
    // recovery candidate; only the independently loadable v1 base is.
    assert_eq!(evidence.recovery_candidates, vec![v1_head.clone()]);
    assert!(!evidence.recovery_candidates.contains(&head));
}

#[test]
fn exact_backup_reconstructs_the_full_audit_log_after_the_live_head_advances() {
    let root = TempRoot::new();
    let repository = repo(&root);
    let g0 = repository.initialize(&meta("initialize")).unwrap();
    let g1 = repository
        .commit_mutations(&g0, &[set_motion(true)], &meta("first"))
        .unwrap();
    let g2 = repository
        .commit_mutations(&g1, &[set_motion(false)], &meta("second"))
        .unwrap();
    let backup = repository
        .create_exact_backup(&g2, &meta("backup at g2"))
        .unwrap();
    // Advance the live head beyond the backed-up generation.
    let g3 = repository
        .commit_mutations(&g2, &[set_motion(true)], &meta("third"))
        .unwrap();
    // The backup still yields its own complete 3-receipt log, reconstructed from
    // the ancestor manifests copied beside it, not from the live store.
    let plan = repository.plan_restore(&g3, &backup).unwrap();
    assert_eq!(plan.target_state.receipts.len(), 3);
}

#[test]
fn exact_backup_with_a_corrupt_ancestor_is_incomplete_and_changes_nothing() {
    let root = TempRoot::new();
    let repository = repo(&root);
    let g0 = repository.initialize(&meta("initialize")).unwrap();
    let g1 = repository
        .commit_mutations(&g0, &[set_motion(true)], &meta("first"))
        .unwrap();
    let g2 = repository
        .commit_mutations(&g1, &[set_motion(false)], &meta("second"))
        .unwrap();
    let backup = repository
        .create_exact_backup(&g2, &meta("backup at g2"))
        .unwrap();
    let g3 = repository
        .commit_mutations(&g2, &[set_motion(true)], &meta("third"))
        .unwrap();
    // Corrupt an ancestor manifest inside the backup; the live store stays valid.
    let ancestor = root
        .0
        .join("backups")
        .join(&backup.backup_id)
        .join("generations")
        .join(format!("g{:020}", g1.generation))
        .join(MANIFEST_FILE);
    let mut bytes = std::fs::read(&ancestor).unwrap();
    bytes.extend_from_slice(b"!corrupt");
    std::fs::write(&ancestor, &bytes).unwrap();
    assert!(matches!(
        repository.plan_restore(&g3, &backup),
        Err(RepositoryError::BackupIncomplete(_))
    ));
    assert_eq!(ready(&repository).generation, g3);
}
