//! Durable machine-local Global Preferences repository.
//!
//! Mutations build complete immutable generations and promote one small head
//! reference as the commit point. Inspection and planning are side-effect free.
//! Project policy, GUI state, Revision state, providers, and networking are not
//! repository authorities.

use std::collections::{BTreeMap, BTreeSet};
use std::path::{Path, PathBuf};

use uuid::Uuid;

use crate::preferences::{DescriptorRegistry, ResolutionSource};

mod io;
mod migration;
mod model;
mod recovery;

pub use migration::{
    MigrationChange, MigrationPlan, MigrationRegistry, MigrationTransform,
    MigrationTransformResult, PreferenceMigration,
};
pub use model::{
    ExactBackupRef, GenerationRef, HeadExpectation, MigrationIssue, MutationMetadata,
    MutationReceipt, PreferenceMutation, PreferencePartition, RepositoryError, RepositorySnapshot,
    RepositoryStatus, RestoreOutcome, RestorePlan, StoredPreferenceValue, UnknownEnvelope,
    UnknownEnvelopeRef, UnreadableEvidence,
};

use io::{
    HEAD_FILE, WriterLease, canonical_bytes, copy_unknown_bytes, digest_bytes, generation_dir,
    load_snapshot, next_generation_number, promote_head, read_head, repository_is_pristine,
    suspect_generation_bytes, unreadable_evidence, write_generation,
};
use model::{HeadRecord, RepositoryState};

#[derive(Debug, Clone)]
pub struct PreferenceRepository {
    root: PathBuf,
    registry: DescriptorRegistry,
}

impl PreferenceRepository {
    pub fn new(root: impl Into<PathBuf>, registry: DescriptorRegistry) -> Self {
        Self {
            root: root.into(),
            registry,
        }
    }

    pub fn root(&self) -> &Path {
        &self.root
    }

    /// Inspect the repository without creating, repairing, migrating, or
    /// acknowledging any on-disk state.
    pub fn inspect(&self) -> RepositoryStatus {
        let head_path = self.root.join(HEAD_FILE);
        if !head_path.exists() {
            if repository_is_pristine(&self.root) {
                return RepositoryStatus::Missing;
            }
            return RepositoryStatus::Unreadable(unreadable_evidence(
                &self.root,
                digest_bytes(&[]),
                head_path,
                Vec::new(),
                "repository head is missing while repository content remains".to_owned(),
            ));
        }
        let head_bytes = match std::fs::read(&head_path) {
            Ok(bytes) => bytes,
            Err(error) => {
                return RepositoryStatus::Unreadable(unreadable_evidence(
                    &self.root,
                    "sha256:unreadable-head".to_owned(),
                    head_path,
                    Vec::new(),
                    error.to_string(),
                ));
            }
        };
        let head_digest = digest_bytes(&head_bytes);
        let head: HeadRecord = match serde_json::from_slice(&head_bytes) {
            Ok(head) => head,
            Err(error) => {
                return RepositoryStatus::Unreadable(unreadable_evidence(
                    &self.root,
                    head_digest,
                    head_path,
                    head_bytes,
                    error.to_string(),
                ));
            }
        };
        match load_snapshot(&self.root, &head.generation) {
            Ok(snapshot) => {
                let issues = self.migration_issues(&snapshot);
                if issues.is_empty() {
                    RepositoryStatus::Ready(snapshot)
                } else {
                    RepositoryStatus::MigrationRequired { snapshot, issues }
                }
            }
            Err(error) => {
                let (path, bytes) = suspect_generation_bytes(&self.root, &head.generation);
                RepositoryStatus::Unreadable(unreadable_evidence(
                    &self.root,
                    head_digest,
                    path,
                    bytes,
                    error.to_string(),
                ))
            }
        }
    }

    pub fn initialize(
        &self,
        metadata: &MutationMetadata,
    ) -> Result<GenerationRef, RepositoryError> {
        self.initialize_with_mutations(&[], metadata)
    }

    /// Atomically publish generation zero with its initial values. This is the
    /// only valid first-write path: callers must not create an empty generation
    /// and follow it with a second commit.
    pub fn initialize_with_mutations(
        &self,
        mutations: &[PreferenceMutation],
        metadata: &MutationMetadata,
    ) -> Result<GenerationRef, RepositoryError> {
        let _lease = WriterLease::acquire(&self.root, &metadata.writer_instance)?;
        self.verify_expectation(&HeadExpectation::Missing)?;
        let repository_id = Uuid::new_v4().to_string();
        let before = RepositoryState::empty();
        let mut state = RepositoryState::empty();
        let mut unknown_bytes = BTreeMap::new();
        let mut affected = BTreeSet::new();
        for mutation in mutations {
            self.apply_mutation(&mut state, &mut unknown_bytes, mutation, &mut affected)?;
        }
        state.receipts.push(receipt(
            if mutations.is_empty() {
                "InitializePreferenceRepository"
            } else {
                "CommitPreferenceMutation"
            },
            metadata,
            None,
            0,
            affected.into_iter().collect(),
            &before,
            &state,
        )?);
        let generation = write_generation(
            &self.root,
            &repository_id,
            0,
            None,
            &metadata.writer_instance,
            &state,
            &unknown_bytes,
        )?;
        promote_head(&self.root, &generation)?;
        Ok(generation)
    }

    pub fn commit_mutations(
        &self,
        expected: &GenerationRef,
        mutations: &[PreferenceMutation],
        metadata: &MutationMetadata,
    ) -> Result<GenerationRef, RepositoryError> {
        let _lease = WriterLease::acquire(&self.root, &metadata.writer_instance)?;
        self.verify_expectation(&HeadExpectation::Generation(expected.clone()))?;
        let snapshot = load_snapshot(&self.root, expected)?;
        self.ensure_snapshot_ready(&snapshot)?;
        let before = snapshot.state();
        let mut state = before.clone();
        let mut unknown_bytes = copy_unknown_bytes(&self.root, &snapshot)?;
        let mut affected = BTreeSet::new();
        for mutation in mutations {
            self.apply_mutation(&mut state, &mut unknown_bytes, mutation, &mut affected)?;
        }
        let next = next_generation_number(&self.root);
        state.receipts.push(receipt(
            "CommitPreferenceMutation",
            metadata,
            Some(expected.generation),
            next,
            affected.iter().cloned().collect(),
            &before,
            &state,
        )?);
        self.commit_state_locked(expected, next, state, unknown_bytes, metadata)
    }

    pub fn read_unknown_exact(
        &self,
        snapshot: &RepositorySnapshot,
        identity: &str,
    ) -> Result<Vec<u8>, RepositoryError> {
        let envelope = snapshot
            .unknown_envelopes
            .get(identity)
            .ok_or_else(|| RepositoryError::UnknownPreferenceKey(identity.to_owned()))?;
        Ok(std::fs::read(io::opaque_path(
            &generation_dir(&self.root, snapshot.generation.generation),
            envelope,
        ))?)
    }

    fn apply_mutation(
        &self,
        state: &mut RepositoryState,
        unknown_bytes: &mut BTreeMap<String, Vec<u8>>,
        mutation: &PreferenceMutation,
        affected: &mut BTreeSet<String>,
    ) -> Result<(), RepositoryError> {
        match mutation {
            PreferenceMutation::Set {
                partition,
                key,
                value,
            } => {
                let descriptor = self.registry.get(key).ok_or_else(|| {
                    RepositoryError::UnknownPreferenceKey(key.as_str().to_owned())
                })?;
                let source = match partition {
                    PreferencePartition::Installation => ResolutionSource::Installation,
                    PreferencePartition::User => ResolutionSource::User,
                };
                if !descriptor.allowed_sources.contains(&source) {
                    return Err(RepositoryError::IneligiblePreferenceSource(
                        key.as_str().to_owned(),
                    ));
                }
                if !descriptor.validates(value) {
                    return Err(RepositoryError::InvalidPreferenceValue(
                        key.as_str().to_owned(),
                    ));
                }
                values_mut(state, partition).insert(
                    key.as_str().to_owned(),
                    StoredPreferenceValue {
                        schema_version: descriptor.schema_version,
                        value: value.clone(),
                    },
                );
                affected.insert(key.as_str().to_owned());
            }
            PreferenceMutation::Remove { partition, key } => {
                let descriptor = self.registry.get(key).ok_or_else(|| {
                    RepositoryError::UnknownPreferenceKey(key.as_str().to_owned())
                })?;
                let source = match partition {
                    PreferencePartition::Installation => ResolutionSource::Installation,
                    PreferencePartition::User => ResolutionSource::User,
                };
                if !descriptor.allowed_sources.contains(&source) {
                    return Err(RepositoryError::IneligiblePreferenceSource(
                        key.as_str().to_owned(),
                    ));
                }
                values_mut(state, partition).remove(key.as_str());
                affected.insert(key.as_str().to_owned());
            }
            PreferenceMutation::PutUnknown(envelope) => {
                let digest = digest_bytes(&envelope.exact_bytes);
                if let Some(existing) = state.unknown_envelopes.get(&envelope.identity)
                    && (existing.payload_digest != digest
                        || existing.payload_len != envelope.exact_bytes.len() as u64)
                {
                    return Err(RepositoryError::UnknownEnvelopeConflict(
                        envelope.identity.clone(),
                    ));
                }
                state.unknown_envelopes.insert(
                    envelope.identity.clone(),
                    UnknownEnvelopeRef {
                        identity: envelope.identity.clone(),
                        provider: envelope.provider.clone(),
                        scope: envelope.scope.clone(),
                        source_version: envelope.source_version.clone(),
                        required_extension: envelope.required_extension.clone(),
                        ordering: envelope.ordering.clone(),
                        payload_digest: digest,
                        payload_len: envelope.exact_bytes.len() as u64,
                    },
                );
                unknown_bytes.insert(envelope.identity.clone(), envelope.exact_bytes.clone());
                affected.insert(format!("unknown:{}", envelope.identity));
            }
            PreferenceMutation::RemoveUnknown { identity } => {
                state.unknown_envelopes.remove(identity);
                unknown_bytes.remove(identity);
                affected.insert(format!("unknown:{identity}"));
            }
        }
        Ok(())
    }

    fn commit_state_locked(
        &self,
        expected: &GenerationRef,
        next_generation: u64,
        state: RepositoryState,
        unknown_bytes: BTreeMap<String, Vec<u8>>,
        metadata: &MutationMetadata,
    ) -> Result<GenerationRef, RepositoryError> {
        let generation = write_generation(
            &self.root,
            &expected.repository_id,
            next_generation,
            Some(expected.generation),
            &metadata.writer_instance,
            &state,
            &unknown_bytes,
        )?;
        promote_head(&self.root, &generation)?;
        Ok(generation)
    }

    fn verify_expectation(&self, expected: &HeadExpectation) -> Result<(), RepositoryError> {
        match (expected, read_head(&self.root)) {
            (HeadExpectation::Missing, Ok(None)) if repository_is_pristine(&self.root) => Ok(()),
            (HeadExpectation::Generation(expected), Ok(Some((actual, _))))
                if actual.generation == *expected =>
            {
                Ok(())
            }
            (HeadExpectation::UnreadableDigest(expected), _) => {
                let bytes = match std::fs::read(self.root.join(HEAD_FILE)) {
                    Ok(bytes) => bytes,
                    Err(error) if error.kind() == std::io::ErrorKind::NotFound => Vec::new(),
                    Err(error) => return Err(RepositoryError::Io(error)),
                };
                (digest_bytes(&bytes) == *expected)
                    .then_some(())
                    .ok_or(RepositoryError::ExpectedGenerationMismatch)
            }
            _ => Err(RepositoryError::ExpectedGenerationMismatch),
        }
    }

    fn ensure_snapshot_ready(&self, snapshot: &RepositorySnapshot) -> Result<(), RepositoryError> {
        let issues = self.migration_issues(snapshot);
        if issues.is_empty() {
            Ok(())
        } else {
            Err(RepositoryError::MigrationTransformUnavailable(
                issues
                    .into_iter()
                    .map(|issue| format!("{}: {}", issue.key, issue.reason))
                    .collect::<Vec<_>>()
                    .join("; "),
            ))
        }
    }

    fn migration_issues(&self, snapshot: &RepositorySnapshot) -> Vec<MigrationIssue> {
        let mut issues = Vec::new();
        for (partition, values) in [
            (PreferencePartition::Installation, &snapshot.installation),
            (PreferencePartition::User, &snapshot.user),
        ] {
            for (stored_key, stored) in values {
                let parsed = crate::preferences::PreferenceKey::parse(stored_key.clone()).ok();
                let descriptor = parsed.as_ref().and_then(|key| self.registry.get(key));
                let reason = if let Some(descriptor) = descriptor {
                    if stored.schema_version != descriptor.schema_version {
                        Some(format!(
                            "stored schema {} differs from registered schema {}",
                            stored.schema_version, descriptor.schema_version
                        ))
                    } else if !descriptor.validates(&stored.value) {
                        Some("stored value is outside the registered schema".to_owned())
                    } else {
                        None
                    }
                } else if self.registry.resolve_alias(stored_key).is_some() {
                    Some("retired alias requires explicit migration".to_owned())
                } else {
                    Some("unregistered known value must remain inactive".to_owned())
                };
                if let Some(reason) = reason {
                    issues.push(MigrationIssue {
                        partition: partition.clone(),
                        key: stored_key.clone(),
                        stored_schema_version: stored.schema_version,
                        registered_schema_version: descriptor
                            .map(|descriptor| descriptor.schema_version),
                        reason,
                    });
                }
            }
        }
        issues
    }
}

fn values_mut<'a>(
    state: &'a mut RepositoryState,
    partition: &PreferencePartition,
) -> &'a mut BTreeMap<String, StoredPreferenceValue> {
    match partition {
        PreferencePartition::Installation => &mut state.installation,
        PreferencePartition::User => &mut state.user,
    }
}

fn flattened_values(state: &RepositoryState) -> BTreeMap<String, StoredPreferenceValue> {
    state
        .installation
        .iter()
        .map(|(key, value)| (format!("installation:{key}"), value.clone()))
        .chain(
            state
                .user
                .iter()
                .map(|(key, value)| (format!("user:{key}"), value.clone())),
        )
        .collect()
}

fn receipt(
    operation: &str,
    metadata: &MutationMetadata,
    expected_generation: Option<u64>,
    resulting_generation: u64,
    affected_keys: Vec<String>,
    before: &RepositoryState,
    after: &RepositoryState,
) -> Result<MutationReceipt, RepositoryError> {
    let before_digests = state_digests(before)?;
    let after_digests = state_digests(after)?;
    Ok(MutationReceipt {
        operation: operation.to_owned(),
        actor: metadata.actor.clone(),
        reason: metadata.reason.clone(),
        expected_generation,
        resulting_generation,
        affected_keys,
        before_digests,
        after_digests,
        redacted_keys: Vec::new(),
    })
}

fn state_digests(state: &RepositoryState) -> Result<BTreeMap<String, String>, RepositoryError> {
    flattened_values(state)
        .into_iter()
        .map(|(key, value)| Ok((key, digest_bytes(&canonical_bytes(&value)?))))
        .chain(state.unknown_envelopes.iter().map(|(identity, envelope)| {
            Ok((
                format!("unknown:{identity}"),
                digest_bytes(&canonical_bytes(envelope)?),
            ))
        }))
        .collect()
}

#[cfg(test)]
mod tests;
