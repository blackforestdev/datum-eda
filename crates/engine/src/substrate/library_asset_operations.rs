//! Canonical library persistence operations; no GUI/CLI filesystem writer.
use std::collections::BTreeMap;

use serde::{Deserialize, Serialize};
use serde_json::{Value, json};
use uuid::Uuid;

use crate::{
    error::EngineError,
    library_assets::{
        AssetDigest, AssetProvenance, AssetRevision, decode_metadata,
        lock::{ADOPTED_PROJECT_SCHEMA_VERSION, LOCK_RELATIVE_PATH, ProjectLibraryLock},
        manifest::{LibraryDiscovery, LibraryReleaseManifest},
        review::AssetReview,
    },
};

pub const CATALOG_RELATIVE_PATH: &str = "library/library.json";

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum LibraryRecordType {
    AssetRevision,
    Provenance,
    Review,
    ReleaseManifest,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "snake_case", deny_unknown_fields)]
pub enum LibraryContent {
    Record {
        record_type: LibraryRecordType,
        fingerprint: String,
        bytes: Vec<u8>,
    },
    Blob {
        fingerprint: String,
        bytes: Vec<u8>,
    },
}

impl LibraryContent {
    pub(crate) fn bytes(&self) -> &[u8] {
        match self {
            Self::Record { bytes, .. } | Self::Blob { bytes, .. } => bytes,
        }
    }

    pub(crate) fn fingerprint(&self) -> Result<AssetDigest, EngineError> {
        let raw = match self {
            Self::Record { fingerprint, .. } | Self::Blob { fingerprint, .. } => fingerprint,
        };
        AssetDigest::parse(raw.clone())
    }

    pub(crate) fn relative_path(&self) -> Result<String, EngineError> {
        let digest = self.fingerprint()?;
        Ok(match self {
            Self::Record { .. } => format!("library/objects/sha256/{}.json", digest.hex()),
            Self::Blob { .. } => format!("library/blobs/sha256/{}", digest.hex()),
        })
    }

    pub(crate) fn validate(
        &self,
        metadata_limit: usize,
        blob_limit: usize,
    ) -> Result<(), EngineError> {
        let expected = self.fingerprint()?;
        let actual = match self {
            Self::Blob { bytes, .. } => {
                if bytes.len() > blob_limit {
                    return Err(refusal("content exceeds declared blob limit"));
                }
                AssetDigest::blob(bytes)
            }
            Self::Record {
                record_type, bytes, ..
            } => match record_type {
                LibraryRecordType::AssetRevision => {
                    decode_metadata::<AssetRevision>(bytes, metadata_limit)?
                        .reference()?
                        .revision_id
                }
                LibraryRecordType::Provenance => {
                    decode_metadata::<AssetProvenance>(bytes, metadata_limit)?.fingerprint()?
                }
                LibraryRecordType::Review => {
                    decode_metadata::<AssetReview>(bytes, metadata_limit)?.fingerprint()?
                }
                LibraryRecordType::ReleaseManifest => {
                    decode_metadata::<LibraryReleaseManifest>(bytes, metadata_limit)?
                        .fingerprint()?
                }
            },
        };
        if expected != actual {
            return Err(refusal("content type/fingerprint mismatch"));
        }
        Ok(())
    }
}

/// Exact canonical metadata bytes are retained through the operation and journal.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct LibraryStateValues {
    pub lock: Vec<u8>,
    pub catalog: Vec<u8>,
}

impl LibraryStateValues {
    pub(crate) fn decode(
        &self,
        limit: usize,
    ) -> Result<(ProjectLibraryLock, LibraryDiscovery), EngineError> {
        let lock: ProjectLibraryLock = decode_metadata(&self.lock, limit)?;
        lock.validate()?;
        let discovery: LibraryDiscovery = decode_metadata(&self.catalog, limit)?;
        Ok((lock, discovery))
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(tag = "action", rename_all = "snake_case", deny_unknown_fields)]
pub enum LibraryAssetOperation {
    AdoptProfile {
        project_id: Uuid,
        preimage: String,
        state: LibraryStateValues,
        content: Vec<LibraryContent>,
    },
    SetState {
        project_id: Uuid,
        previous: LibraryStateValues,
        state: LibraryStateValues,
        content: Vec<LibraryContent>,
    },
    RefuseProfileInverse {
        project_id: Uuid,
    },
}

impl LibraryAssetOperation {
    pub(crate) fn owner_id(&self) -> Uuid {
        match self {
            Self::AdoptProfile { project_id, .. }
            | Self::SetState { project_id, .. }
            | Self::RefuseProfileInverse { project_id } => *project_id,
        }
    }

    pub(crate) fn state(&self) -> Result<&LibraryStateValues, EngineError> {
        match self {
            Self::AdoptProfile { state, .. } | Self::SetState { state, .. } => Ok(state),
            Self::RefuseProfileInverse { .. } => {
                Err(refusal("library profile adoption is forward-only"))
            }
        }
    }

    pub(crate) fn content(&self) -> &[LibraryContent] {
        match self {
            Self::AdoptProfile { content, .. } | Self::SetState { content, .. } => content,
            Self::RefuseProfileInverse { .. } => &[],
        }
    }

    pub(crate) fn content_map(
        &self,
        metadata_limit: usize,
        blob_limit: usize,
        file_limit: usize,
        total_limit: usize,
    ) -> Result<BTreeMap<String, &[u8]>, EngineError> {
        if self.content().len() > file_limit {
            return Err(refusal("content inventory exceeds declared limit"));
        }
        let total = self
            .content()
            .iter()
            .try_fold(0usize, |n, c| n.checked_add(c.bytes().len()))
            .ok_or_else(|| refusal("content aggregate size overflow"))?;
        if total > total_limit {
            return Err(refusal(
                "content aggregate exceeds server raw allocation policy",
            ));
        }
        let mut files = BTreeMap::new();
        for content in self.content() {
            content.validate(metadata_limit, blob_limit)?;
            if files
                .insert(content.relative_path()?, content.bytes())
                .is_some()
            {
                return Err(refusal("duplicate immutable content path"));
            }
        }
        Ok(files)
    }
}

pub(super) fn apply_manifest(
    value: &mut Value,
    op: &LibraryAssetOperation,
) -> Result<bool, EngineError> {
    match op {
        LibraryAssetOperation::AdoptProfile { project_id, .. } => {
            if value
                .get("uuid")
                .and_then(Value::as_str)
                .and_then(|v| v.parse::<Uuid>().ok())
                != Some(*project_id)
            {
                return Err(refusal("profile project identity differs"));
            }
            value["schema_version"] = json!(ADOPTED_PROJECT_SCHEMA_VERSION);
            value["library_lock"] = json!({"path":LOCK_RELATIVE_PATH,"schema_version":1});
            Ok(true)
        }
        LibraryAssetOperation::SetState { .. } => Ok(false),
        LibraryAssetOperation::RefuseProfileInverse { .. } => {
            Err(refusal("library profile adoption inverse refuses"))
        }
    }
}

/// Materialization replays declared postimages; fresh preimages are checked once
/// against the complete batch during admission, not against an already-promoted head.
pub(super) fn apply_shard(
    kind: &super::SourceShardKind,
    value: &mut Value,
    op: &LibraryAssetOperation,
) -> Result<bool, EngineError> {
    let state = op.state()?;
    let bytes = match kind {
        super::SourceShardKind::LibraryLock => &state.lock,
        super::SourceShardKind::LibraryCatalog => &state.catalog,
        _ => return Ok(false),
    };
    *value = serde_json::from_slice(bytes)?;
    Ok(true)
}

pub(super) fn inverse(op: &LibraryAssetOperation) -> Result<LibraryAssetOperation, EngineError> {
    Ok(match op {
        LibraryAssetOperation::AdoptProfile { project_id, .. } => {
            LibraryAssetOperation::RefuseProfileInverse {
                project_id: *project_id,
            }
        }
        LibraryAssetOperation::SetState {
            project_id,
            previous,
            state,
            ..
        } => LibraryAssetOperation::SetState {
            project_id: *project_id,
            previous: state.clone(),
            state: previous.clone(),
            content: vec![],
        },
        LibraryAssetOperation::RefuseProfileInverse { .. } => {
            return Err(refusal("profile inverse cannot replay"));
        }
    })
}

pub(crate) fn refusal(message: impl Into<String>) -> EngineError {
    EngineError::Validation(format!("library adoption: {}", message.into()))
}

pub(super) fn apply_model(
    model: &mut super::DesignModel,
    op: &LibraryAssetOperation,
    diff: &mut super::CommitDiff,
) -> Result<(), EngineError> {
    // Remains unsupported until a vetted host policy is installed, even for a
    // caller that constructs raw operations without the native preparation API.
    super::library_asset_resolver::server_policy()?;
    if op.owner_id() != model.project.project_id {
        return Err(refusal("foreign library project owner"));
    }
    let state = op.state()?;
    state.decode(state.lock.len().max(state.catalog.len()))?;
    let root = model
        .source_shards
        .iter()
        .find(|s| s.kind == super::SourceShardKind::ProjectManifest)
        .and_then(|s| s.path.parent())
        .ok_or_else(|| refusal("project manifest source missing"))?
        .to_path_buf();
    if matches!(op, LibraryAssetOperation::AdoptProfile { .. }) {
        if model.project.schema_version != Some(1) {
            return Err(refusal("profile adoption requires version1"));
        }
        model.project.schema_version = Some(u64::from(ADOPTED_PROJECT_SCHEMA_VERSION));
    } else if model.project.schema_version != Some(u64::from(ADOPTED_PROJECT_SCHEMA_VERSION)) {
        return Err(refusal("library state requires the adopted profile"));
    }
    for (kind, path, bytes) in [
        (
            super::SourceShardKind::LibraryLock,
            LOCK_RELATIVE_PATH,
            &state.lock,
        ),
        (
            super::SourceShardKind::LibraryCatalog,
            CATALOG_RELATIVE_PATH,
            &state.catalog,
        ),
    ] {
        let value = serde_json::from_slice(bytes)?;
        let reference = super::source_shard_ref_builders::source_shard_ref_for_value(
            &root,
            kind.clone(),
            path.into(),
            &value,
        )?;
        if let Some(existing) = model
            .source_shards
            .iter_mut()
            .find(|s| s.relative_path == path)
        {
            if existing.kind != kind {
                return Err(refusal("library metadata source kind collision"));
            }
            *existing = reference;
        } else {
            model.source_shards.push(reference);
        }
    }
    super::operation_application_object_revision::bump_existing_object(
        &mut model.objects,
        op.owner_id(),
        Some(diff),
    )
}

/// Recover only the profile that the actual recorded prefix establishes.
/// Replay placeholders never acquire an invented schema to admit an operation.
pub(in crate::substrate) fn replay_profile_before(
    project: Uuid,
    transaction: &super::TransactionRecord,
    prefix: &[super::TransactionRecord],
) -> Result<Option<u64>, EngineError> {
    let baseline = prefix.iter().find_map(|record| {
        record.operations.iter().find_map(|op| match op {
            super::Operation::LibraryAssets {
                operation:
                    LibraryAssetOperation::AdoptProfile {
                        project_id,
                        preimage,
                        ..
                    },
            } => Some((record.transaction_id, *project_id, preimage)),
            _ => None,
        })
    });
    let Some((start, owner, preimage)) = baseline else {
        return Ok(None);
    };
    if owner != project
        || preimage.len() > super::library_asset_resolver::server_policy()?.metadata_bytes
    {
        return Err(refusal(
            "foreign or oversized recorded library format baseline",
        ));
    }
    let mut header: Value = serde_json::from_str(preimage)?;
    crate::library_assets::verify_native_snapshot(preimage.as_bytes(), &header)?;
    if header["uuid"] != json!(project)
        || header["schema_version"].as_u64() != Some(1)
        || header.get("library_lock").is_some()
    {
        return Err(refusal(
            "recorded library baseline is not the owning version1 header",
        ));
    }
    let mut started = false;
    for record in prefix {
        started |= record.transaction_id == start;
        if record.transaction_id == transaction.transaction_id {
            return Ok(header.get("schema_version").and_then(Value::as_u64));
        }
        if started {
            for op in &record.operations {
                if let super::Operation::LibraryAssets {
                    operation: LibraryAssetOperation::AdoptProfile { .. },
                } = op
                    && header["schema_version"].as_u64() != Some(1)
                {
                    return Err(refusal(
                        "duplicate library format adoption in recorded prefix",
                    ));
                }
                super::project_manifest_journal_ops::apply_project_manifest_operation(
                    &mut header,
                    op,
                )?;
            }
        }
    }
    Err(refusal(
        "actual replay transaction missing from its recorded prefix",
    ))
}
