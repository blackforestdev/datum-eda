use std::path::PathBuf;

use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::substrate::ModelRevision;

pub const REVISION_STORE_SCHEMA_VERSION: u64 = 1;
pub(crate) const CANONICAL_ENCODING: &str = "datum-canonical-json-v1";
pub(crate) const DIGEST_ALGORITHM: &str = "sha256";

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
#[serde(transparent)]
pub struct AlgorithmQualifiedDigest(pub String);

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum AffectedShardAction {
    Write,
    Delete,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum TechnicalValidationState {
    AcceptedByMutationGuards,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct AffectedShard {
    pub relative_path: String,
    pub action: AffectedShardAction,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub postimage_blob: Option<AlgorithmQualifiedDigest>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct StagedShardPostimage {
    pub relative_path: String,
    pub bytes: Option<Vec<u8>>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct IntegrityGeneration {
    pub schema_version: u64,
    pub canonical_encoding: String,
    pub digest_algorithm: String,
    pub project_id: Uuid,
    pub transaction_id: Uuid,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub parent_transaction_id: Option<Uuid>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub parent_integrity_root: Option<AlgorithmQualifiedDigest>,
    pub before_model_revision: ModelRevision,
    pub after_model_revision: ModelRevision,
    pub transaction_blob: AlgorithmQualifiedDigest,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub authority_snapshot_blob: Option<AlgorithmQualifiedDigest>,
    pub affected_shards: Vec<AffectedShard>,
    pub validation_state: TechnicalValidationState,
    pub cache_invalidations: Vec<String>,
    pub integrity_root: AlgorithmQualifiedDigest,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub(crate) struct IntegrityGenerationMaterial {
    pub schema_version: u64,
    pub canonical_encoding: String,
    pub digest_algorithm: String,
    pub project_id: Uuid,
    pub transaction_id: Uuid,
    pub parent_transaction_id: Option<Uuid>,
    pub parent_integrity_root: Option<AlgorithmQualifiedDigest>,
    pub before_model_revision: ModelRevision,
    pub after_model_revision: ModelRevision,
    pub transaction_blob: AlgorithmQualifiedDigest,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub authority_snapshot_blob: Option<AlgorithmQualifiedDigest>,
    pub affected_shards: Vec<AffectedShard>,
    pub validation_state: TechnicalValidationState,
    pub cache_invalidations: Vec<String>,
}

impl IntegrityGeneration {
    pub(crate) fn material(&self) -> IntegrityGenerationMaterial {
        IntegrityGenerationMaterial {
            schema_version: self.schema_version,
            canonical_encoding: self.canonical_encoding.clone(),
            digest_algorithm: self.digest_algorithm.clone(),
            project_id: self.project_id,
            transaction_id: self.transaction_id,
            parent_transaction_id: self.parent_transaction_id,
            parent_integrity_root: self.parent_integrity_root.clone(),
            before_model_revision: self.before_model_revision.clone(),
            after_model_revision: self.after_model_revision.clone(),
            transaction_blob: self.transaction_blob.clone(),
            authority_snapshot_blob: self.authority_snapshot_blob.clone(),
            affected_shards: self.affected_shards.clone(),
            validation_state: self.validation_state,
            cache_invalidations: self.cache_invalidations.clone(),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct IntegrityHead {
    pub schema_version: u64,
    pub project_id: Uuid,
    pub transaction_id: Uuid,
    pub integrity_root: AlgorithmQualifiedDigest,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case", tag = "state")]
pub enum RevisionStoreState {
    Empty,
    Complete {
        head: IntegrityHead,
    },
    Recovered {
        head: IntegrityHead,
        recovered_stages: usize,
    },
    ReadOnlyDiagnostic {
        last_complete: Option<IntegrityHead>,
        diagnostics: Vec<IntegrityDiagnostic>,
    },
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct IntegrityDiagnostic {
    pub code: String,
    pub message: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub path: Option<PathBuf>,
}
