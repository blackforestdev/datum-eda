use std::collections::{BTreeMap, BTreeSet};
use std::path::PathBuf;

use serde::{Deserialize, Serialize};
use uuid::Uuid;

use super::io::{
    self, WriterLease, canonical_bytes, copy_unknown_bytes, digest_bytes, generation_dir,
    load_snapshot, next_generation_number, promote_head, sync_parent, write_generation,
    write_immutable_content, write_new_file,
};
use super::model::{GenerationManifest, RepositoryState};
use super::{
    ExactBackupRef, GenerationRef, HeadExpectation, MutationMetadata, PreferenceRepository,
    RepositoryError, RepositorySnapshot, RestoreOutcome, RestorePlan, UnreadableEvidence,
    flattened_values, receipt, state_digests,
};

impl PreferenceRepository {
    pub fn preserve_unreadable(
        &self,
        evidence: &UnreadableEvidence,
        writer_instance: &str,
    ) -> Result<PathBuf, RepositoryError> {
        let _lease = WriterLease::acquire(&self.root, writer_instance)?;
        self.verify_expectation(&HeadExpectation::UnreadableDigest(
            evidence.head_digest.clone(),
        ))?;
        self.preserve_unreadable_locked(evidence)
    }

    pub fn create_exact_backup(
        &self,
        expected: &GenerationRef,
        metadata: &MutationMetadata,
    ) -> Result<ExactBackupRef, RepositoryError> {
        let _lease = WriterLease::acquire(&self.root, &metadata.writer_instance)?;
        self.verify_expectation(&HeadExpectation::Generation(expected.clone()))?;
        self.create_backup_locked(expected, metadata)
    }

    pub fn plan_restore(
        &self,
        expected: &GenerationRef,
        backup: &ExactBackupRef,
    ) -> Result<RestorePlan, RepositoryError> {
        self.verify_expectation(&HeadExpectation::Generation(expected.clone()))?;
        let current = load_snapshot(&self.root, expected)?;
        let target = self.load_backup(backup)?;
        let current_values = state_digests(&current.state())?;
        let target_values = state_digests(&target.state())?;
        let all_keys: BTreeSet<_> = current_values
            .keys()
            .chain(target_values.keys())
            .cloned()
            .collect();
        let (changed_keys, unchanged_keys) = all_keys
            .into_iter()
            .partition(|key| current_values.get(key) != target_values.get(key));
        Ok(RestorePlan {
            expected_head: expected.clone(),
            backup: backup.clone(),
            changed_keys,
            unchanged_keys,
            target_state: target.state(),
        })
    }

    pub fn commit_restore(
        &self,
        plan: &RestorePlan,
        metadata: &MutationMetadata,
    ) -> Result<RestoreOutcome, RepositoryError> {
        let _lease = WriterLease::acquire(&self.root, &metadata.writer_instance)?;
        self.verify_expectation(&HeadExpectation::Generation(plan.expected_head.clone()))
            .map_err(|_| RepositoryError::RestorePreviewStale)?;
        let displaced_backup = self.create_backup_locked(&plan.expected_head, metadata)?;
        let target_snapshot = self.load_backup(&plan.backup)?;
        let before = load_snapshot(&self.root, &plan.expected_head)?.state();
        let mut state = plan.target_state.clone();
        state.receipts = before.receipts.clone();
        let next = next_generation_number(&self.root);
        state.receipts.push(receipt(
            "CommitExactRestore",
            metadata,
            Some(plan.expected_head.generation),
            next,
            plan.changed_keys.clone(),
            &before,
            &state,
        )?);
        let unknown_bytes = self.backup_unknown_bytes(&plan.backup, &target_snapshot)?;
        let generation =
            self.commit_state_locked(&plan.expected_head, next, state, unknown_bytes, metadata)?;
        Ok(RestoreOutcome {
            generation,
            displaced_backup,
        })
    }

    pub fn restore_recovery_candidate(
        &self,
        evidence: &UnreadableEvidence,
        candidate: &GenerationRef,
        metadata: &MutationMetadata,
    ) -> Result<GenerationRef, RepositoryError> {
        let _lease = WriterLease::acquire(&self.root, &metadata.writer_instance)?;
        self.verify_expectation(&HeadExpectation::UnreadableDigest(
            evidence.head_digest.clone(),
        ))?;
        self.preserve_unreadable_locked(evidence)?;
        let candidate_snapshot = load_snapshot(&self.root, candidate)?;
        let mut state = candidate_snapshot.state();
        let next = next_generation_number(&self.root);
        state.receipts.push(receipt(
            "RestoreRecoveryCandidate",
            metadata,
            None,
            next,
            flattened_values(&state).into_keys().collect(),
            &RepositoryState::empty(),
            &state,
        )?);
        let unknown_bytes = copy_unknown_bytes(&self.root, &candidate_snapshot)?;
        let generation = write_generation(
            &self.root,
            &candidate.repository_id,
            next,
            Some(candidate.generation),
            &metadata.writer_instance,
            &state,
            &unknown_bytes,
        )?;
        promote_head(&self.root, &generation)?;
        Ok(generation)
    }

    fn preserve_unreadable_locked(
        &self,
        evidence: &UnreadableEvidence,
    ) -> Result<PathBuf, RepositoryError> {
        let directory = self.root.join("unreadable");
        std::fs::create_dir_all(&directory)?;
        let digest = evidence
            .identity
            .strip_prefix("sha256:")
            .unwrap_or(&evidence.identity);
        let path = directory.join(format!("{digest}.bin"));
        if path.exists() {
            if std::fs::read(&path)? != evidence.exact_bytes {
                return Err(RepositoryError::Invariant(
                    "unreadable evidence identity collision".to_owned(),
                ));
            }
        } else {
            write_new_file(&path, &evidence.exact_bytes)?;
        }
        Ok(path)
    }

    pub(super) fn create_backup_locked(
        &self,
        expected: &GenerationRef,
        metadata: &MutationMetadata,
    ) -> Result<ExactBackupRef, RepositoryError> {
        let source = generation_dir(&self.root, expected.generation);
        let backup_id = format!("g{:020}-{}", expected.generation, Uuid::new_v4());
        let target = self.root.join("backups").join(&backup_id);
        std::fs::create_dir_all(&target)?;
        let snapshot = load_snapshot(&self.root, expected)?;
        write_new_file(
            &target.join("manifest.json"),
            &std::fs::read(source.join("manifest.json"))?,
        )?;
        for envelope in snapshot.unknown_envelopes.values() {
            let source_path = io::opaque_path(&source, envelope);
            let target_path = io::opaque_path(&target, envelope);
            write_immutable_content(&target_path, &std::fs::read(source_path)?)?;
        }
        let backup = ExactBackupRef {
            backup_id,
            source_generation: expected.clone(),
        };
        let record = BackupRecord {
            backup: backup.clone(),
            actor: metadata.actor.clone(),
            reason: metadata.reason.clone(),
        };
        write_new_file(&target.join("backup.json"), &canonical_bytes(&record)?)?;
        sync_parent(&target)?;
        Ok(backup)
    }

    fn load_backup(&self, backup: &ExactBackupRef) -> Result<RepositorySnapshot, RepositoryError> {
        let root = self.root.join("backups").join(&backup.backup_id);
        let record: BackupRecord = serde_json::from_slice(
            &std::fs::read(root.join("backup.json"))
                .map_err(|_| RepositoryError::BackupIncomplete(backup.backup_id.clone()))?,
        )?;
        if record.backup != *backup {
            return Err(RepositoryError::BackupIncomplete(backup.backup_id.clone()));
        }
        let bytes = std::fs::read(root.join("manifest.json"))?;
        if digest_bytes(&bytes) != backup.source_generation.canonical_manifest_digest {
            return Err(RepositoryError::BackupIncomplete(backup.backup_id.clone()));
        }
        let manifest: GenerationManifest = serde_json::from_slice(&bytes)?;
        let snapshot = RepositorySnapshot {
            generation: backup.source_generation.clone(),
            installation: manifest.state.installation,
            user: manifest.state.user,
            unknown_envelopes: manifest.state.unknown_envelopes,
            receipts: manifest.state.receipts,
        };
        self.backup_unknown_bytes(backup, &snapshot)?;
        Ok(snapshot)
    }

    fn backup_unknown_bytes(
        &self,
        backup: &ExactBackupRef,
        snapshot: &RepositorySnapshot,
    ) -> Result<BTreeMap<String, Vec<u8>>, RepositoryError> {
        let root = self.root.join("backups").join(&backup.backup_id);
        snapshot
            .unknown_envelopes
            .iter()
            .map(|(identity, envelope)| {
                let bytes = std::fs::read(io::opaque_path(&root, envelope))?;
                if digest_bytes(&bytes) != envelope.payload_digest
                    || bytes.len() as u64 != envelope.payload_len
                {
                    return Err(RepositoryError::BackupIncomplete(backup.backup_id.clone()));
                }
                Ok((identity.clone(), bytes))
            })
            .collect()
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
struct BackupRecord {
    backup: ExactBackupRef,
    actor: String,
    reason: String,
}
