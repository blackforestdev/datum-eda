//! Bind accepted journal history to the immutable generations that cover it.
use super::{
    AlgorithmQualifiedDigest, BTreeMap, BTreeSet, CANONICAL_ENCODING, DIGEST_ALGORITHM,
    EngineError, IntegrityGeneration, IntegrityHead, Path, REVISION_STORE_SCHEMA_VERSION,
    RevisionAuthorityStore, TransactionRecord, Uuid, canonical_bytes, digest_bytes, read_json,
    transaction_tip,
};

impl RevisionAuthorityStore {
    // Fallback must never use an older caller snapshot after the fresh read failed.
    pub(super) fn find_last_complete_fresh(
        &self,
        project_id: Uuid,
    ) -> Result<Option<IntegrityHead>, EngineError> {
        let (journal, diagnostics) =
            crate::substrate::read_transaction_journal(self.project_root()?);
        if !diagnostics.is_empty() {
            return Ok(None);
        }
        self.find_last_complete(project_id, &journal)
    }

    pub(super) fn verify_accepted_generation(
        &self,
        project_id: Uuid,
        journal: &[TransactionRecord],
        generation: &IntegrityGeneration,
        stage_root: Option<&Path>,
    ) -> Result<(), EngineError> {
        if let Some(root) = stage_root {
            self.verify_staged(root, generation)?;
        } else {
            self.verify_generation(generation)?;
        }
        let mut index = journal
            .iter()
            .position(|record| record.transaction_id == generation.transaction_id)
            .ok_or_else(|| {
                EngineError::Validation("revision generation has no accepted journal record".into())
            })?;
        let mut current = generation.clone();
        loop {
            verify_accepted_record(project_id, journal, index, &current)?;
            // The first integrity-backed commit can extend a pre-integrity journal.
            // Only an absent parent root ends coverage; a missing referenced parent
            // is an error, never a legacy boundary.
            let Some(parent_root) = &current.parent_integrity_root else {
                return Ok(());
            };
            let parent_index = index.checked_sub(1).ok_or_else(|| {
                EngineError::Validation("revision covered parent missing from journal".into())
            })?;
            let parent: IntegrityGeneration = read_json(&self.generation_path(parent_root))?;
            if parent.integrity_root != *parent_root {
                return Err(EngineError::Validation(
                    "revision parent root does not name its generation".into(),
                ));
            }
            self.verify_generation(&parent)?;
            current = parent;
            index = parent_index;
        }
    }

    pub(super) fn verify_head(
        &self,
        project_id: Uuid,
        journal: &[TransactionRecord],
        head: &IntegrityHead,
    ) -> Result<IntegrityGeneration, EngineError> {
        if head.schema_version != REVISION_STORE_SCHEMA_VERSION || head.project_id != project_id {
            return Err(EngineError::Validation(
                "revision integrity head schema/project mismatch".to_string(),
            ));
        }
        if transaction_tip(journal) != Some(head.transaction_id) {
            return Err(EngineError::Validation(format!(
                "revision integrity head transaction {} is not the accepted journal tip",
                head.transaction_id
            )));
        }
        let generation =
            read_json::<IntegrityGeneration>(&self.generation_path(&head.integrity_root))?;
        self.verify_accepted_generation(project_id, journal, &generation, None)?;
        if generation.transaction_id != head.transaction_id
            || generation.integrity_root != head.integrity_root
        {
            return Err(EngineError::Validation(
                "revision integrity head does not name its generation".to_string(),
            ));
        }
        Ok(generation)
    }

    fn find_last_complete(
        &self,
        project_id: Uuid,
        journal: &[TransactionRecord],
    ) -> Result<Option<IntegrityHead>, EngineError> {
        let generations = self.root.join("generations");
        let Ok(entries) = std::fs::read_dir(generations) else {
            return Ok(None);
        };
        let journal_positions: BTreeMap<_, _> = journal
            .iter()
            .enumerate()
            .map(|(index, record)| (record.transaction_id, index))
            .collect();
        let mut candidates = Vec::new();
        for entry in entries.flatten() {
            let Ok(generation) = read_json::<IntegrityGeneration>(&entry.path()) else {
                continue;
            };
            let Some(index) = journal_positions.get(&generation.transaction_id).copied() else {
                continue;
            };
            if entry.path() == self.generation_path(&generation.integrity_root) {
                candidates.push((index, generation));
            }
        }
        candidates.sort_by_key(|(index, _)| *index);
        // This set belongs to this fresh accepted journal only. Process parents
        // before descendants; retain only fully verified ancestry, keyed by root
        // AND accepted position so another generation for the same UUID cannot
        // borrow a successful result from a different branch.
        let mut verified = BTreeSet::<(usize, AlgorithmQualifiedDigest)>::new();
        let mut last_complete = None;
        for (index, generation) in candidates {
            if self.verify_generation(&generation).is_err()
                || verify_accepted_record(project_id, journal, index, &generation).is_err()
            {
                continue;
            }
            if let Some(parent_root) = &generation.parent_integrity_root
                && !index
                    .checked_sub(1)
                    .is_some_and(|previous| verified.contains(&(previous, parent_root.clone())))
            {
                continue;
            }
            verified.insert((index, generation.integrity_root.clone()));
            last_complete = Some(IntegrityHead {
                schema_version: REVISION_STORE_SCHEMA_VERSION,
                project_id,
                transaction_id: generation.transaction_id,
                integrity_root: generation.integrity_root,
            });
        }
        Ok(last_complete)
    }
}

fn verify_accepted_record(
    project_id: Uuid,
    journal: &[TransactionRecord],
    index: usize,
    generation: &IntegrityGeneration,
) -> Result<(), EngineError> {
    if generation.project_id != project_id
        || generation.schema_version != REVISION_STORE_SCHEMA_VERSION
        || generation.canonical_encoding != CANONICAL_ENCODING
        || generation.digest_algorithm != DIGEST_ALGORITHM
    {
        return Err(EngineError::Validation(
            "revision journal generation schema/project/encoding mismatch".into(),
        ));
    }
    let record = &journal[index];
    if record.transaction_id != generation.transaction_id
        || record.before_model_revision != generation.before_model_revision
        || record.after_model_revision != generation.after_model_revision
        || digest_bytes(&canonical_bytes(record)?) != generation.transaction_blob
    {
        return Err(EngineError::Validation(format!(
            "accepted journal payload differs from immutable transaction {}",
            generation.transaction_id,
        )));
    }
    let previous = index.checked_sub(1).map(|i| journal[i].transaction_id);
    if generation.parent_transaction_id != previous {
        return Err(EngineError::Validation(
            "revision generation parent differs from accepted journal predecessor".into(),
        ));
    }
    Ok(())
}
