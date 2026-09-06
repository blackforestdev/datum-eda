use std::collections::BTreeMap;
use std::path::PathBuf;

use serde::{Deserialize, Serialize};
use serde_json::Value;
use uuid::Uuid;

use crate::preferences::PreferenceKey;

pub const REPOSITORY_FORMAT_VERSION: u32 = 1;
pub const CANONICALIZATION_VERSION: u32 = 1;

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct GenerationRef {
    pub repository_id: String,
    pub generation: u64,
    pub parent_generation: Option<u64>,
    pub canonical_manifest_digest: String,
    pub committed_order: u64,
    pub writer_instance: String,
    pub format_version: u32,
    pub canonicalization_version: u32,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum PreferencePartition {
    Installation,
    User,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct StoredPreferenceValue {
    pub schema_version: u32,
    pub value: Value,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct UnknownEnvelopeRef {
    pub identity: String,
    pub provider: Option<String>,
    pub scope: String,
    pub source_version: String,
    pub required_extension: Option<String>,
    pub ordering: Option<String>,
    pub payload_digest: String,
    pub payload_len: u64,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct UnknownEnvelope {
    pub identity: String,
    pub provider: Option<String>,
    pub scope: String,
    pub source_version: String,
    pub required_extension: Option<String>,
    pub ordering: Option<String>,
    pub exact_bytes: Vec<u8>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct MutationReceipt {
    pub operation: String,
    pub actor: String,
    pub reason: String,
    pub expected_generation: Option<u64>,
    pub resulting_generation: u64,
    pub affected_keys: Vec<String>,
    pub before_digests: BTreeMap<String, String>,
    pub after_digests: BTreeMap<String, String>,
    pub redacted_keys: Vec<String>,
    #[serde(default)]
    pub request_id: Option<Uuid>,
    #[serde(default)]
    pub canonical_request_digest: Option<String>,
    #[serde(default)]
    pub actor_kind: Option<String>,
    #[serde(default)]
    pub local_actor_id: Option<String>,
    #[serde(default)]
    pub actor_session_id: Option<String>,
    #[serde(default)]
    pub invocation_id: Option<Uuid>,
    #[serde(default)]
    pub expected_generation_ref: Option<GenerationRef>,
    #[serde(default)]
    pub proposal_id: Option<Uuid>,
    #[serde(default)]
    pub proposal_digest: Option<String>,
    #[serde(default)]
    pub acceptance_id: Option<Uuid>,
    #[serde(default)]
    pub requesting_actor: Option<String>,
    #[serde(default)]
    pub accepting_actor: Option<String>,
    #[serde(default)]
    pub originating_mcp_session: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub(crate) struct RepositoryState {
    pub installation: BTreeMap<String, StoredPreferenceValue>,
    pub user: BTreeMap<String, StoredPreferenceValue>,
    pub unknown_envelopes: BTreeMap<String, UnknownEnvelopeRef>,
    pub receipts: Vec<MutationReceipt>,
}

impl RepositoryState {
    pub(crate) fn empty() -> Self {
        Self {
            installation: BTreeMap::new(),
            user: BTreeMap::new(),
            unknown_envelopes: BTreeMap::new(),
            receipts: Vec::new(),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub(crate) struct GenerationManifest {
    pub format_version: u32,
    pub canonicalization_version: u32,
    pub repository_id: String,
    pub generation: u64,
    pub parent_generation: Option<u64>,
    pub committed_order: u64,
    pub writer_instance: String,
    pub state: RepositoryState,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub(crate) struct HeadRecord {
    pub generation: GenerationRef,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RepositorySnapshot {
    pub generation: GenerationRef,
    pub installation: BTreeMap<String, StoredPreferenceValue>,
    pub user: BTreeMap<String, StoredPreferenceValue>,
    pub unknown_envelopes: BTreeMap<String, UnknownEnvelopeRef>,
    pub receipts: Vec<MutationReceipt>,
}

impl RepositorySnapshot {
    pub fn value(
        &self,
        partition: PreferencePartition,
        key: &PreferenceKey,
    ) -> Option<&StoredPreferenceValue> {
        match partition {
            PreferencePartition::Installation => self.installation.get(key.as_str()),
            PreferencePartition::User => self.user.get(key.as_str()),
        }
    }

    pub(crate) fn state(&self) -> RepositoryState {
        RepositoryState {
            installation: self.installation.clone(),
            user: self.user.clone(),
            unknown_envelopes: self.unknown_envelopes.clone(),
            receipts: self.receipts.clone(),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct UnreadableEvidence {
    pub identity: String,
    pub head_digest: String,
    pub source_path: PathBuf,
    pub exact_bytes: Vec<u8>,
    pub reason: String,
    pub recovery_candidates: Vec<GenerationRef>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum RepositoryStatus {
    Missing,
    Ready(RepositorySnapshot),
    MigrationRequired {
        snapshot: RepositorySnapshot,
        issues: Vec<MigrationIssue>,
    },
    Unreadable(UnreadableEvidence),
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct MigrationIssue {
    pub partition: PreferencePartition,
    pub key: String,
    pub stored_schema_version: u32,
    pub registered_schema_version: Option<u32>,
    pub reason: String,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum HeadExpectation {
    Missing,
    Generation(GenerationRef),
    UnreadableDigest(String),
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct MutationMetadata {
    pub actor: String,
    pub reason: String,
    pub writer_instance: String,
    pub audit: Option<MutationAuditMetadata>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct MutationAuditMetadata {
    pub request_id: Uuid,
    pub canonical_request_digest: String,
    pub actor_kind: String,
    pub local_actor_id: String,
    pub actor_session_id: String,
    pub invocation_id: Uuid,
    pub expected_generation_ref: Option<GenerationRef>,
    pub proposal_id: Option<Uuid>,
    pub proposal_digest: Option<String>,
    pub acceptance_id: Option<Uuid>,
    pub requesting_actor: Option<String>,
    pub accepting_actor: Option<String>,
    pub originating_mcp_session: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum PreferenceMutation {
    Set {
        partition: PreferencePartition,
        key: PreferenceKey,
        value: Value,
    },
    Remove {
        partition: PreferencePartition,
        key: PreferenceKey,
    },
    PutUnknown(UnknownEnvelope),
    RemoveUnknown {
        identity: String,
    },
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ExactBackupRef {
    pub backup_id: String,
    pub source_generation: GenerationRef,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RestorePlan {
    pub expected_head: GenerationRef,
    pub backup: ExactBackupRef,
    pub changed_keys: Vec<String>,
    pub unchanged_keys: Vec<String>,
    pub(crate) target_state: RepositoryState,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RestoreOutcome {
    pub generation: GenerationRef,
    pub displaced_backup: ExactBackupRef,
}

#[derive(Debug, thiserror::Error)]
pub enum RepositoryError {
    #[error("repository I/O failed: {0}")]
    Io(#[from] std::io::Error),
    #[error("repository JSON failed: {0}")]
    Json(#[from] serde_json::Error),
    #[error("writer lease is unavailable")]
    WriterLeaseUnavailable,
    #[error("expected repository head does not match current head")]
    ExpectedGenerationMismatch,
    #[error("repository is unreadable and preserved as {identity}: {reason}")]
    UnreadableStorePreserved { identity: String, reason: String },
    #[error("unsupported repository format {0}")]
    UnsupportedRepositoryVersion(u32),
    #[error("unknown preference key {0}")]
    UnknownPreferenceKey(String),
    #[error("invalid value for preference {0}")]
    InvalidPreferenceValue(String),
    #[error("preference source is ineligible for {0}")]
    IneligiblePreferenceSource(String),
    #[error("unknown-envelope identity already has different bytes: {0}")]
    UnknownEnvelopeConflict(String),
    #[error("backup is incomplete: {0}")]
    BackupIncomplete(String),
    #[error("restore preview is stale")]
    RestorePreviewStale,
    #[error("migration transform unavailable: {0}")]
    MigrationTransformUnavailable(String),
    #[error("migration value requires a user choice: {0}")]
    MigrationValueChoiceRequired(String),
    #[error("alias migration would create two live identities: {0}")]
    AliasCollision(String),
    #[error("repository invariant failed: {0}")]
    Invariant(String),
}
